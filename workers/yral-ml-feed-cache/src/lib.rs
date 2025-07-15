use serde::{Deserialize, Serialize};
use worker::*;

#[derive(Serialize, Deserialize, Debug)]
struct Country {
    city: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CustomMlFeedCacheItem {
    post_id: u64,
    canister_id: String,
    video_id: String,
    creator_principal_id: String,
}

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let router = Router::new();

    router
        .get("/", |_, _| Response::ok("Hello from Workers cache!"))
        .post_async("/feed-cache/:canister_id", |mut req, ctx| async move {
            let canister_id = ctx.param("canister_id").unwrap();
            let new_items: Vec<CustomMlFeedCacheItem> = req.json().await.unwrap_or_default();
            if new_items.is_empty() {
                return Response::error("Bad Request", 400);
            };

            // get the existing cache
            let existing_items = ctx
                .kv("yral-ml-feed-cache")?
                .get(canister_id)
                .json::<Vec<CustomMlFeedCacheItem>>()
                .await?
                .unwrap_or_default();

            let mut combined_items = new_items;
            combined_items.extend(existing_items);

            if combined_items.len() > 50 {
                combined_items.truncate(50);
            }

            let json_data = serde_json::to_string(&combined_items)?;

            match ctx
                .kv("yral-ml-feed-cache")?
                .put(canister_id, json_data)?
                .execute()
                .await
            {
                Ok(_) => Response::ok("Success"),
                Err(_) => Response::error("Bad Request", 400),
            }
        })
        .get_async("/feed-cache/:canister_id", |req, ctx| async move {
            let canister_id = ctx.param("canister_id").unwrap();

            let url = req.url()?;
            let query_params: std::collections::HashMap<String, String> = url
                .query_pairs()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();
            let _start = query_params
                .get("start")
                .unwrap_or(&"0".to_string())
                .parse::<usize>()
                .unwrap_or(0);
            let _limit = query_params
                .get("limit")
                .unwrap_or(&"50".to_string())
                .parse::<usize>()
                .unwrap_or(50);

            let items = ctx
                .kv("yral-ml-feed-cache")?
                .get(canister_id)
                .json::<Vec<CustomMlFeedCacheItem>>()
                .await?
                .unwrap_or_default();

            let json_data = serde_json::to_string(&items)?;

            Response::ok(json_data)
        })
        .run(req, env)
        .await
}
