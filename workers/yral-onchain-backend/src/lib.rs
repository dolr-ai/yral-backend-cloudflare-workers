use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use candid::{Encode, Principal};
use individual_user_canister::backup_restore::backup::individual_user_backup_handler;
use individual_user_canister::backup_restore::bulk_backup::{
    individual_user_bulk_backup, individual_user_bulk_backup_handler,
};
use individual_user_canister::backup_restore::bulk_restore::individual_user_bulk_restore_handler;
use individual_user_canister::backup_restore::restore::individual_user_restore_handler;
use platform_ochestrator::backup_restore::backup::platform_ochestrator_backup;
use platform_ochestrator::backup_restore::restore::platform_ochestrator_restore_handler;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use user_index::backup_restore::bulk_backup::user_index_bulk_backup;
use user_index::backup_restore::bulk_restore::user_index_bulk_restore_handler;
use utils::create_agent;
use wasm_bindgen::JsValue;
use worker::Router;
use worker::*;

mod individual_user_canister;
mod platform_ochestrator;
mod user_index;
mod utils;

#[derive(Deserialize, Serialize)]
struct RequestData {
    canister_id: Principal,
}

#[derive(Deserialize, Serialize)]
struct RequestMigration {
    new_canister: Principal,
}

// worker fetch request body interface
//      /backup => RequestData { canister_id }
//      /restore => RequestData { canister_id }
//      /bulk-restore => None

#[event(fetch)]
pub async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let router = Router::new();
    router
        .put_async("/individual-user/backup", individual_user_backup_handler)
        .put_async("/individual-user/restore", individual_user_restore_handler)
        .put_async(
            "/individual-user/bulk-restore",
            individual_user_bulk_restore_handler,
        )
        .put_async(
            "/individual-user/bulk-backup",
            individual_user_bulk_backup_handler,
        )
        .put_async(
            "/platform-orchestrator/restore",
            platform_ochestrator_restore_handler,
        )
        .put_async("/user-index/bulk-restore", user_index_bulk_restore_handler)
        .run(req, env)
        .await
}

#[event(scheduled)]
pub async fn cron_backup(_event: ScheduledEvent, env: Env, _ctx: ScheduleContext) {
    individual_user_bulk_backup(&env).await.unwrap();
    platform_ochestrator_backup(&env).await.unwrap();
    user_index_bulk_backup(&env).await.unwrap();
}

#[durable_object]
pub struct CanisterData {
    data: RefCell<Option<Vec<u8>>>,
    state: State,
    env: Env,
}

impl DurableObject for CanisterData {
    fn new(state: State, env: Env) -> Self {
        Self {
            data: RefCell::new(None),
            state,
            env,
        }
    }

    // Durable object request body interface
    //      /backup => String
    //      /restore => RequestData { canister_id }
    async fn fetch(&self, mut req: Request) -> Result<Response> {
        if self.data.borrow().is_none() {
            *self.data.borrow_mut() = self
                .state
                .storage()
                .get::<Vec<u8>>("data")
                .await
                .ok()
                .flatten();
        }

        match req.path().as_str() {
            s if s.ends_with("/backup") => {
                let data: String = req.text().await?;
                let data = BASE64_STANDARD.decode(data).map_err(|e| e.to_string())?;
                console_debug!("{}", String::from_utf8(data.clone()).unwrap());
                self.state.storage().put("data", data.clone()).await?;
                *self.data.borrow_mut() = Some(data);
                Response::ok("Backup Successful!")
            }
            s if s.ends_with("/restore") => {
                let agent = create_agent(&self.env).await;

                let RequestData { canister_id } = req.json().await?;
                let data = self.data.borrow().clone().expect("Data is None");
                let snapshot_length = data.len() as u64;
                let mut offset: u64 = 0;
                let chunk_size: u64 = 1_000_000; // Adjust chunk size as needed

                while offset < snapshot_length {
                    let length = std::cmp::min(chunk_size, snapshot_length - offset);
                    let chunk = &data[(offset as usize)..((offset + length) as usize)];
                    console_debug!("Chunk made: {}", length);
                    agent
                        .update(&canister_id, "receive_and_save_snaphot")
                        .with_arg(Encode!(&offset, &chunk).unwrap())
                        .call_and_wait()
                        .await
                        .map_err(|e| Error::RustError(e.to_string()))?;

                    offset += length;
                }

                console_debug!("Loading snapshot");
                agent
                    .update(&canister_id, "load_snapshot")
                    .with_arg(Encode!().unwrap())
                    .call_and_wait()
                    .await
                    .map_err(|e| Error::RustError(e.to_string()))?;
                Response::ok("Restore Sucessful!")
            }
            _ => Response::error("Not Found", 404),
        }
    }
}
