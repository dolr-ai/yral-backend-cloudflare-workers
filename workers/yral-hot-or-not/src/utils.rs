use serde::Serialize;

pub fn err_to_resp<E>(status_code: u16, e: E) -> worker::Result<worker::Response>
where
    E: Serialize,
{
    Ok(worker::Response::from_json(&e)?.with_status(status_code))
}
