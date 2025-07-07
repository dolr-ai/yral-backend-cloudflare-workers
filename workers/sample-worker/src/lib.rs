use serde::{Deserialize, Serialize};
use worker::*;

mod utils;

// fn log_request(req: &Request) {
//     console_log!("{} - [{}]", Date::now().to_string(), req.path());
// }

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let router = Router::new();

    #[derive(Serialize, Deserialize, Debug)]
    struct Country {
        city: String,
    }

    router
        .get("/", |_, _| Response::ok("Hello from Workers!"))
        .post_async("/:country", |mut req, ctx| async move {
            let country = ctx.param("country").unwrap();
            let city = match req.json::<Country>().await {
                Ok(c) => c.city,
                Err(_) => String::from(""),
            };
            if city.is_empty() {
                return Response::error("Bad Request", 400);
            };
            match ctx.kv("assets")?.put(country, &city)?.execute().await {
                Ok(_) => Response::ok(city),
                Err(_) => Response::error("Bad Request", 400),
            }
        })
        .get_async("/:country", |_req, ctx| async move {
            if let Some(country) = ctx.param("country") {
                return match ctx.kv("assets")?.get(country).text().await? {
                    Some(city) => Response::ok(city),
                    None => Response::error("Country not found", 404),
                };
            }
            Response::error("Bad Request", 400)
        })
        .run(req, env)
        .await
}
