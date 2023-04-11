mod records;
mod utils;

use crate::records::request_records_internal;
use js_sys::Array;
use snarkvm_console_network::Testnet3;
use wasm_bindgen::prelude::*;

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
    ).await {
        Ok(records) => RecordScanner::new(
            "".to_string(),
            records.into_iter().map(|r| JsValue::from_str(&r)).collect(),
        ),
        Err(e) => RecordScanner::new(e.to_string(), Default::default()),
    }
}

#[wasm_bindgen]
#[derive(Debug)]
pub struct RecordScanner {
    msg: String,
    records: Array,
}

#[wasm_bindgen]
impl RecordScanner {
    #[wasm_bindgen(constructor)]
    pub fn new(msg: String, records: Array) -> Self {
        RecordScanner { msg, records }
    }

    #[wasm_bindgen(getter)]
    pub fn msg(&self) -> String {
        self.msg.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn records(&self) -> Array {
        self.records.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_msg(&mut self, msg: String) {
        self.msg = msg
    }

    #[wasm_bindgen(setter)]
    pub fn set_records(&mut self, records: Array) {
        self.records = records
    }
}
