use num_bigint::{BigInt, BigUint};
use std::cell::RefCell;
use std::result::Result as StdResult;
use worker::*;
use worker_utils::{
    err_to_resp,
    storage::{daily_cumulative_limit::DailyCumulativeLimit, SafeStorage, StorageCell},
};

use crate::{
    consts::{
        MAX_CREDITED_PER_DAY_PER_USER_YRAL, MAX_DEDUCTED_PER_DAY_PER_USER_YRAL,
        YRAL_CREDITED_STORAGE_KEY, YRAL_DEDUCTED_STORAGE_KEY,
    },
    error::WorkerError,
    types::{YralBalanceInfo, YralBalanceUpdateRequest},
};

#[durable_object]
pub struct UserYralCoinState {
    state: State,
    pub(crate) env: Env,
    yral_balance: RefCell<StorageCell<BigUint>>,
    yral_credited: RefCell<DailyCumulativeLimit<{ MAX_CREDITED_PER_DAY_PER_USER_YRAL }>>,
    yral_deducted: RefCell<DailyCumulativeLimit<{ MAX_DEDUCTED_PER_DAY_PER_USER_YRAL }>>,
}

impl UserYralCoinState {
    pub(crate) fn storage(&self) -> SafeStorage {
        self.state.storage().into()
    }

    // SAFETY: RefCell borrows held across await points are safe in Cloudflare Workers
    // because Workers run in a single-threaded JavaScript runtime with no concurrent access.
    // The RefCell interior mutability pattern is required due to Worker 0.7.4 API changes
    // that mandate `&self` instead of `&mut self` for DurableObject trait methods.
    #[allow(clippy::await_holding_refcell_ref)]
    async fn broadcast_balance_inner(&self) -> Result<()> {
        let storage = self.storage();
        let balance = { self.yral_balance.borrow_mut().read(&storage).await?.clone() };
        let bal = YralBalanceInfo { balance };
        for ws in self.state.get_websockets() {
            let err = ws.send(&bal);
            if let Err(e) = err {
                console_warn!("failed to broadcast balance update: {e}");
            }
        }

        Ok(())
    }

    async fn broadcast_balance(&self) {
        if let Err(e) = self.broadcast_balance_inner().await {
            console_error!("failed to read balance data: {e}");
        }
    }

    // SAFETY: See comment on broadcast_balance_inner for safety rationale
    #[allow(clippy::await_holding_refcell_ref)]
    pub async fn update_balance_for_external_client(
        &self,
        expected_balance: BigUint,
        delta: BigInt,
    ) -> StdResult<BigUint, (u16, WorkerError)> {
        let mut storage = self.storage();
        if delta >= BigInt::ZERO {
            let result = {
                self.yral_credited
                    .borrow_mut()
                    .try_consume(&mut storage, delta.to_biguint().unwrap())
                    .await
            };
            result.map_err(|_| (400, WorkerError::YralCreditLimitReached))?;
        } else {
            let result = {
                self.yral_deducted
                    .borrow_mut()
                    .try_consume(&mut storage, (-delta.clone()).to_biguint().unwrap())
                    .await
            };
            result.map_err(|_| (400, WorkerError::YralDeductLimitReached))?;
        }

        let new_bal = {
            self.yral_balance
                .borrow_mut()
                .try_get_update(&mut storage, |balance| {
                    if expected_balance != *balance {
                        return Err((
                            409,
                            WorkerError::BalanceTransactionConflict {
                                new_balance: balance.clone(),
                            },
                        ));
                    }
                    let delta = delta.clone();
                    if delta >= BigInt::ZERO {
                        let delta = delta.to_biguint().unwrap();
                        *balance += delta;
                        return Ok(());
                    }
                    let neg_delta = (-delta).to_biguint().unwrap();
                    if neg_delta > *balance {
                        return Err((400, WorkerError::InsufficientFunds));
                    }
                    *balance -= neg_delta;

                    Ok(())
                })
                .await
                .map_err(|e| match e {
                    Ok(e) => e,
                    Err(e) => (500, WorkerError::Internal(e.to_string())),
                })?
        };

        self.broadcast_balance().await;

        Ok(new_bal)
    }
}

impl DurableObject for UserYralCoinState {
    fn new(state: State, env: Env) -> Self {
        console_error_panic_hook::set_once();

        Self {
            state,
            env,
            yral_balance: RefCell::new(StorageCell::new("yral_balance_v0", || BigUint::ZERO)),
            yral_credited: RefCell::new(DailyCumulativeLimit::new(YRAL_CREDITED_STORAGE_KEY)),
            yral_deducted: RefCell::new(DailyCumulativeLimit::new(YRAL_DEDUCTED_STORAGE_KEY)),
        }
    }

    async fn fetch(&self, req: Request) -> Result<Response> {
        let env = self.env.clone();
        let router = Router::with_data(self);
        router
            .get_async("/balance", {
                // SAFETY: See comment on broadcast_balance_inner for safety rationale
                #[allow(clippy::await_holding_refcell_ref)]
                async |_, ctx| {
                    let this = ctx.data;
                    let storage = this.storage();
                    let balance = { this.yral_balance.borrow_mut().read(&storage).await?.clone() };
                    Response::from_json(&YralBalanceInfo { balance })
                }
            })
            .post_async("/update_balance", async |mut req, ctx| {
                let req_data: YralBalanceUpdateRequest = serde_json::from_str(&req.text().await?)?;
                let this = ctx.data;

                match this
                    .update_balance_for_external_client(req_data.previous_balance, req_data.delta)
                    .await
                {
                    Ok(new_bal) => Response::ok(new_bal.to_string()),
                    Err((code, msg)) => err_to_resp(code, msg),
                }
            })
            .get_async("/ws/balance", |req, ctx| async move {
                let upgrade = req.headers().get("Upgrade")?;
                if upgrade.as_deref() != Some("websocket") {
                    return Response::error("expected websocket", 400);
                }

                let pair = WebSocketPair::new()?;
                let this = ctx.data;
                this.state.accept_web_socket(&pair.server);
                this.broadcast_balance().await;

                Response::from_websocket(pair.client)
            })
            .run(req, env)
            .await
    }

    async fn websocket_message(
        &self,
        ws: WebSocket,
        _message: WebSocketIncomingMessage,
    ) -> Result<()> {
        ws.send(&"not supported".to_string())
    }

    async fn websocket_error(&self, ws: WebSocket, error: worker::Error) -> Result<()> {
        ws.close(Some(500), Some(error.to_string()))
    }

    async fn websocket_close(
        &self,
        ws: WebSocket,
        code: usize,
        reason: String,
        _was_clean: bool,
    ) -> Result<()> {
        ws.close(Some(code as u16), Some(reason))
    }
}
