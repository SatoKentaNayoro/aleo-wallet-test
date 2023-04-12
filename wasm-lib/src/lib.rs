mod records;
mod utils;
mod transfer;

use crate::records::{RecordScanner, request_records_internal};
use js_sys::Array;
use snarkvm_console_account::address::Address;
use snarkvm_console_network::{Testnet3, Visibility};
use snarkvm_console_program::{Plaintext, Record};
use wasm_bindgen::prelude::*;
use crate::transfer::transfer_internal;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

type CurrentNetwork = Testnet3;

// #[wasm_bindgen]
// extern {
//     fn alert(s: &str);
// }
//
// #[wasm_bindgen]
// pub fn greet() {
//     alert("Hello, aleo-wallet-test!");
// }

#[wasm_bindgen]
pub async fn request_records(
    private_key: Option<String>,
    view_key: String,
    start: Option<u32>,
    end: Option<u32>,
    last: Option<u32>,
    endpoint: String,
) -> RecordScanner {
    match request_records_internal::<CurrentNetwork>(
        private_key,
        view_key,
        start,
        end,
        last,
        endpoint,
    )
        .await
    {
        Ok(records) => RecordScanner::new(
            "".to_string(),
            records.into_iter().map(|r| JsValue::from_str(&r)).collect(),
        ),
        Err(e) => RecordScanner::new(e.to_string(), Default::default()),
    }
}

#[wasm_bindgen]
pub async fn transfer(
    private_key: String,
    record: String,
    amount: u64,
    recipient: String,
    query_endpoint: String,
    broadcast: String,
) -> String {
    match transfer_internal::<CurrentNetwork>(private_key, record, amount, recipient, query_endpoint, broadcast).await {
        Ok(transaction) => transaction,
        Err(e) => format!("error: {}", e)
    }
}


