use num_bigint::{BigInt, BigUint};
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
    yral_balance: StorageCell<BigUint>,
    yral_credited: DailyCumulativeLimit<{ MAX_CREDITED_PER_DAY_PER_USER_YRAL }>,
    yral_deducted: DailyCumulativeLimit<{ MAX_DEDUCTED_PER_DAY_PER_USER_YRAL }>,
}

impl UserYralCoinState {
    pub(crate) fn storage(&self) -> SafeStorage {
        self.state.storage().into()
    }

    async fn broadcast_balance_inner(&mut self) -> Result<()> {
        let storage = self.storage();
        let bal = YralBalanceInfo {
            balance: self.yral_balance.read(&storage).await?.clone(),
        };
        for ws in self.state.get_websockets() {
            let err = ws.send(&bal);
            if let Err(e) = err {
                console_warn!("failed to broadcast balance update: {e}");
            }
        }

        Ok(())
    }

    async fn broadcast_balance(&mut self) {
        if let Err(e) = self.broadcast_balance_inner().await {
            console_error!("failed to read balance data: {e}");
        }
    }

    pub async fn update_balance_for_external_client(
        &mut self,
        expected_balance: BigUint,
        delta: BigInt,
    ) -> StdResult<BigUint, (u16, WorkerError)> {
        if delta >= BigInt::ZERO {
            self.yral_credited
                .try_consume(&mut self.storage(), delta.to_biguint().unwrap())
                .await
                .map_err(|_| (400, WorkerError::YralCreditLimitReached))?;
        } else {
            self.yral_deducted
                .try_consume(&mut self.storage(), (-delta.clone()).to_biguint().unwrap())
                .await
                .map_err(|_| (400, WorkerError::YralDeductLimitReached))?;
        }

        let new_bal = self
            .yral_balance
            .try_get_update(&mut self.storage(), |balance| {
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
            })?;

        self.broadcast_balance().await;

        Ok(new_bal)
    }
}

#[durable_object]
impl DurableObject for UserYralCoinState {
    fn new(state: State, env: Env) -> Self {
        console_error_panic_hook::set_once();

        Self {
            state,
            env,
            yral_balance: StorageCell::new("yral_balance_v0", || BigUint::ZERO),
            yral_credited: DailyCumulativeLimit::new(YRAL_CREDITED_STORAGE_KEY),
            yral_deducted: DailyCumulativeLimit::new(YRAL_DEDUCTED_STORAGE_KEY),
        }
    }

    async fn fetch(&mut self, req: Request) -> Result<Response> {
        let env = self.env.clone();
        let router = Router::with_data(self);
        router
            .get_async("/balance", async |_, ctx| {
                let this = ctx.data;
                let storage = this.storage();
                let balance = this.yral_balance.read(&storage).await?.clone();
                Response::from_json(&YralBalanceInfo { balance })
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
        &mut self,
        ws: WebSocket,
        _message: WebSocketIncomingMessage,
    ) -> Result<()> {
        ws.send(&"not supported".to_string())
    }

    async fn websocket_error(&mut self, ws: WebSocket, error: worker::Error) -> Result<()> {
        ws.close(Some(500), Some(error.to_string()))
    }

    async fn websocket_close(
        &mut self,
        ws: WebSocket,
        code: usize,
        reason: String,
        _was_clean: bool,
    ) -> Result<()> {
        ws.close(Some(code as u16), Some(reason))
    }
}
