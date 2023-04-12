use indexmap::IndexMap;
use snarkvm_algorithms::snark::marlin::{CircuitProvingKey, MarlinHidingMode};
use snarkvm_console_account::{PrivateKey, ViewKey};
use snarkvm_console_network_environment::Environment;
use snarkvm_console_program::Network;
use snarkvm_utilities::FromBytes;
use std::str::FromStr;
use std::sync::Arc;
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
type MarlinProvingKey<N> = CircuitProvingKey<<N as Environment>::PairingCurve, MarlinHidingMode>;

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

pub(crate) async fn post_request(
    endpoint: &str,
    value: &serde_json::Value,
) -> anyhow::Result<Response> {
    let window = web_sys::window().unwrap();
    let mut request_init = RequestInit::new();
    request_init.method("POST");
    request_init.mode(web_sys::RequestMode::Cors);

    let headers =
        Headers::new().map_err(|js_value| anyhow::Error::msg(format!("{:?}", js_value)))?;
    headers
        .append("Content-Type", "application/json")
        .map_err(|js_value| anyhow::Error::msg(format!("{:?}", js_value)))?;
    request_init.headers(&headers.into());

    let body = JsValue::from_str(&value.to_string());
    request_init.body(Some(&body));

    let request = Request::new_with_str_and_init(endpoint, &request_init)
        .map_err(|js_value| anyhow::Error::msg(format!("{:?}", js_value)))?;

    let response = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|js_value| anyhow::Error::msg(format!("{:?}", js_value)))?;
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

// 验证邮箱

fn get_credits_proving_keys<E: Environment>(data: &[u8]) -> anyhow::Result<IndexMap<String, Arc<MarlinProvingKey<E>>>> {
    let credits_proving_keys_raw: IndexMap<String, Vec<u8>> = bincode::deserialize(data).map_err(|err| anyhow::Error::msg(format!("failed to deserialize data: {}", err)))?;
    let mut credits_proving_keys = IndexMap::new();
    for (k, v) in credits_proving_keys_raw.iter() {
        let le: Arc<MarlinProvingKey<E>> =
            Arc::new(MarlinProvingKey::<E>::read_le(v.as_slice()).map_err(|err|anyhow::Error::msg(format!("failed to read_le data: {}", err)))?);
        credits_proving_keys.insert(k.clone(), le);
    }
    Ok(credits_proving_keys)
}

#[test]
fn test_credits_proving_keys() {
    use crate::CurrentNetwork;
    use indexmap::IndexMap;
    use snarkvm_console_network::CREDITS_PROVING_KEYS;
    use snarkvm_console_network_environment::Console;
    use snarkvm_synthesizer::Program;
    use snarkvm_utilities::ToBytes;
    use std::fs::File;
    use std::io::{Read, Write};

    // type MarlinProvingKey<N> =
    //     CircuitProvingKey<<N as Environment>::PairingCurve, MarlinHidingMode>;

    let mut new_credits_proving_keys = IndexMap::new();

    let program = Program::<CurrentNetwork>::credits().unwrap();
    for k in program.functions().keys() {
        if let Some(v) = CREDITS_PROVING_KEYS.get(&k.to_string()) {
            new_credits_proving_keys.insert(k.to_string(), v.clone());
        }
    }
    println!("{:?}", new_credits_proving_keys.keys());
    assert_eq!(
        new_credits_proving_keys.len(),
        program.functions().keys().len()
    );

    let mut credits_proving_keys_1 = IndexMap::new();
    for (k, v) in new_credits_proving_keys.iter() {
        credits_proving_keys_1.insert(k.clone(), v.clone().to_bytes_le().unwrap());
    }

    let serialized_data = bincode::serialize(&credits_proving_keys_1).unwrap();
    let mut file = File::create("credits_proving_keys_test").unwrap();
    file.write_all(&serialized_data).unwrap();

    let mut file = File::open("credits_proving_keys_test").unwrap();
    let mut content = Vec::new();
    let _ = file.read_to_end(&mut content).unwrap();

    let credits_proving_keys_2: IndexMap<String, Vec<u8>> = bincode::deserialize(&content).unwrap();

    assert_eq!(credits_proving_keys_2, credits_proving_keys_1);

    // let mut credits_proving_keys_3 = IndexMap::new();
    // for (k, v) in credits_proving_keys_2.iter() {
    //     let le: Arc<MarlinProvingKey<Console>> =
    //         Arc::new(MarlinProvingKey::<Console>::read_le(v.as_slice()).unwrap());
    //     credits_proving_keys_3.insert(k.clone(), le);
    // }
    let credits_proving_keys_3 = get_credits_proving_keys::<Console>(&content).unwrap();
    assert_eq!(new_credits_proving_keys, credits_proving_keys_3)
}
