use snarkvm_console_account::{PrivateKey, ViewKey};
use snarkvm_console_program::Network;
use std::str::FromStr;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Headers, Request, RequestInit, Response};

// pub fn set_panic_hook() {
//     // When the `console_error_panic_hook` feature is enabled, we can call the
//     // `set_panic_hook` function at least once during initialization, and then
//     // we will get better error messages if our code ever panics.
//     //
//     // For more details see
//     // https://github.com/rustwasm/console_error_panic_hook#readme
//     #[cfg(feature = "console_error_panic_hook")]
//     console_error_panic_hook::set_once();
// }

pub(crate) fn parse_account<N: Network>(
    private_key: Option<String>,
    view_key: String,
) -> anyhow::Result<(Option<PrivateKey<N>>, ViewKey<N>)> {
    let mut pk = None;
    if let Some(..) = private_key {
        let result = PrivateKey::<N>::from_str(&private_key.unwrap());
        if let Ok(..) = result {
            pk = Some(result.unwrap())
        }
    }

    let view_key = ViewKey::from_str(&view_key)?;
    Ok((pk, view_key))
}

pub(crate) async fn post_request(endpoint: &str, value: &serde_json::Value) -> anyhow::Result<Response>{
    let window = web_sys::window().unwrap();
    let mut request_init = RequestInit::new();
    request_init.method("POST");
    request_init.mode(web_sys::RequestMode::Cors);

    let mut headers = Headers::new().map_err(|js_value| anyhow::Error::msg(format!("{:?}", js_value)))?;
    headers.append("Content-Type", "application/json").map_err(|js_value| anyhow::Error::msg(format!("{:?}", js_value)))?;
    request_init.headers(&headers.into());

    let body = JsValue::from_str(&value.to_string());
    request_init.body(Some(&body));

    let request = Request::new_with_str_and_init(&endpoint, &request_init).map_err(|js_value| anyhow::Error::msg(format!("{:?}", js_value)))?;

    let response = JsFuture::from(window.fetch_with_request(&request)).await.map_err(|js_value| anyhow::Error::msg(format!("{:?}", js_value)))?;
    let response = response.dyn_into::<Response>().unwrap();

    if response.ok() {
        Ok(response)
    } else {
        Err(anyhow::Error::msg("Error in response"))
    }
}

pub(crate) async fn get_request(endpoint: &str) -> anyhow::Result<Response> {
    let mut opts = RequestInit::new();
    opts.method("GET");

    let request = match Request::new_with_str_and_init(endpoint, &opts) {
        Ok(req) => req,
        Err(e) => {
            return Err(anyhow::Error::msg(e.as_string().unwrap_or_default()));
        }
    };

    match web_sys::window() {
        Some(window) => {
            let resp_value = match JsFuture::from(window.fetch_with_request(&request)).await {
                Ok(res_v) => res_v,
                Err(e) => {
                    return Err(anyhow::Error::msg(e.as_string().unwrap_or_default()));
                }
            };
            match resp_value.dyn_into() {
                Ok(res) => Ok(res),
                Err(e) => Err(anyhow::Error::msg(e.as_string().unwrap_or_default())),
            }
        }
        None => Err(anyhow::Error::msg("failed to load window")),
    }
}