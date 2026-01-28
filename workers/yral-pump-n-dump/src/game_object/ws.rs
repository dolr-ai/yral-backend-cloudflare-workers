use candid::Principal;
use pump_n_dump_common::{
    rest::UserBetsResponse,
    ws::{WsMessage, WsRequest, WsResp, WsResponse},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use worker::{Result, WebSocket, WebSocketIncomingMessage};

use crate::game_object::GameObjReq;

use super::GameState;

#[derive(Serialize, Deserialize, Clone, Copy)]
struct WsState {
    game_canister: Principal,
    token_root: Principal,
    user_canister: Principal,
}

impl GameState {
    pub async fn handle_ws(
        &self,
        ws: WebSocket,
        game_canister: Principal,
        token_root: Principal,
        user_canister: Principal,
    ) -> Result<()> {
        self.state.accept_web_socket(&ws);
        ws.serialize_attachment(WsState {
            game_canister,
            token_root,
            user_canister,
        })?;

        self.ensure_bets_loaded().await?;
        let user_bets = self
            .bets
            .borrow()
            .as_ref()
            .unwrap()
            .get(&user_canister)
            .copied()
            .unwrap_or_default();

        ws.send(&WsResponse {
            request_id: Uuid::max(),
            response: WsResp::WelcomeEvent {
                round: self.round().await?,
                pool: self.pumps().await? + self.dumps().await?,
                player_count: self.state.get_websockets().len() as u64,
                user_bets: UserBetsResponse {
                    pumps: user_bets[0],
                    dumps: user_bets[1],
                },
            },
        })?;

        Ok(())
    }

    fn broadcast_event(&self, resp: WsResp) -> Result<()> {
        let resp = WsResponse {
            request_id: Uuid::max(),
            response: resp,
        };
        for ws in self.state.get_websockets() {
            ws.send(&resp)?;
        }

        Ok(())
    }

    pub async fn handle_ws_message(
        &self,
        ws: &WebSocket,
        msg: WebSocketIncomingMessage,
    ) -> Result<()> {
        let WebSocketIncomingMessage::String(raw_msg) = msg else {
            return ws.send(&WsResponse {
                request_id: Uuid::nil(),
                response: WsResp::error("unknown request"),
            });
        };
        let Ok(ws_req) = serde_json::from_str::<WsRequest>(&raw_msg) else {
            return ws.send(&WsResponse {
                request_id: Uuid::nil(),
                response: WsResp::error("unknown request"),
            });
        };
        let state: WsState = ws.deserialize_attachment()?.unwrap();
        let WsMessage::Bet { direction, round } = ws_req.msg;

        let res = self
            .game_request(GameObjReq {
                sender: state.user_canister,
                direction,
                creator: state.game_canister,
                token_root: state.token_root,
                round,
            })
            .await;

        let responses = match res {
            Ok(r) => r,
            Err(e) => {
                return ws.send(&WsResponse {
                    request_id: ws_req.request_id,
                    response: WsResp::bet_failure(e.to_string(), direction),
                })
            }
        };

        for resp in responses {
            match &resp {
                WsResp::GameResultEvent(_) | WsResp::WinningPoolEvent { .. } => {
                    self.broadcast_event(resp)?;
                }
                _ => {
                    ws.send(&WsResponse {
                        request_id: ws_req.request_id,
                        response: resp,
                    })?;
                }
            };
        }

        Ok(())
    }
}
