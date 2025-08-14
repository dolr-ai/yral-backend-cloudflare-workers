use serde::Serialize;
use worker::*;

pub mod environment;
pub mod icp;
pub mod jwt;
pub mod storage;

#[derive(Default)]
pub struct RequestInitBuilder(RequestInit);

impl RequestInitBuilder {
    pub fn header(&mut self, k: &str, v: &str) -> Result<&mut Self> {
        self.0.headers.set(k, v)?;
        Ok(self)
    }

    pub fn replace_headers(&mut self, headers: Headers) -> &mut Self {
        self.0.headers = headers;
        self
    }

    pub fn method(&mut self, method: Method) -> &mut Self {
        self.0.method = method;
        self
    }

    // pub fn redirect(&mut self, redirect: RequestRedirect) -> &mut Self {
    //     self.0.redirect = redirect;
    //     self
    // }

    // pub fn cf_props(&mut self, props: CfProperties) -> &mut Self {
    //     self.0.cf = props;
    //     self
    // }

    pub fn json<T: Serialize>(&mut self, body: &T) -> Result<&mut Self> {
        let json = serde_json::to_string(body)?;
        self.0.body = Some(wasm_bindgen::JsValue::from_str(&json));

        self.header("Content-Type", "application/json; charset=utf-8")
    }

    pub fn build(&self) -> &RequestInit {
        &self.0
    }
}

#[macro_export]
macro_rules! parse_principal {
    ($ctx:ident, $param:literal) => {{
        let raw = $ctx.param($param).unwrap();
        let Ok(principal) = candid::Principal::from_text(raw) else {
            return Response::error(concat!("invalid ", $param), 400);
        };

        principal
    }};
}

pub fn err_to_resp<E>(status_code: u16, e: E) -> worker::Result<worker::Response>
where
    E: Serialize,
{
    Ok(worker::Response::from_json(&e)?.with_status(status_code))
}
