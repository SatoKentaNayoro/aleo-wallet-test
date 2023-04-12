use crate::utils::{get_request, parse_account};
use anyhow::{bail, ensure};
use js_sys::Array;
use snarkvm_console_account::{PrivateKey, ViewKey};
use snarkvm_console_program::{Ciphertext, Field, Network, Plaintext, Record};
use snarkvm_synthesizer::Block;

use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, Response};
use wasm_bindgen::prelude::*;

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

pub(crate) async fn request_records_internal<N: Network>(
    private_key: Option<String>,
    view_key: String,
    start: Option<u32>,
    end: Option<u32>,
    last: Option<u32>,
    endpoint: String,
) -> anyhow::Result<Vec<String>> {
    // Derive the view key and optional private key.
    let (private_key, view_key) = parse_account::<N>(private_key, view_key)?;

    // Find the start and end height to scan.
    let (start_height, end_height) = parse_block_range(start, end, last, endpoint.clone()).await?;

    // Fetch the records_web from the network.
    let records =
        fetch_records::<N>(private_key, &view_key, &endpoint, start_height, end_height).await?;

    // Output the decrypted records_web associated with the view key.
    if records.is_empty() {
        Err(anyhow::Error::msg("No records_web found".to_string()))
    } else {
        let mut res = Vec::new();
        for r in records {
            let s = serde_json::to_string_pretty(&r)?.replace("\\n", "");
            println!("{}", s);
            res.push(s)
        }
        Ok(res)
    }
}

async fn parse_block_range(
    start: Option<u32>,
    end: Option<u32>,
    last: Option<u32>,
    endpoint: String,
) -> anyhow::Result<(u32, u32)> {
    match (start, end, last) {
        (Some(start), Some(end), None) => {
            ensure!(
                end > start,
                "The given scan range is invalid (start = {start}, end = {end})"
            );

            Ok((start, end))
        }
        (Some(start), None, None) => {
            // Request the latest block height from the endpoint.
            let endpoint = format!("{}/testnet3/latest/height", endpoint);
            let latest_height = fetch_latest_height(&endpoint).await?;

            // Print warning message if the user is attempting to scan the whole chain.
            // if start == 0 {
            //     println!("⚠️  Attention - Scanning the entire chain. This may take a while...\n");
            // }

            Ok((start, latest_height))
        }
        (None, Some(end), None) => Ok((0, end)),
        (None, None, Some(last)) => {
            // Request the latest block height from the endpoint.
            let endpoint = format!("{}/testnet3/latest/height", endpoint);
            let latest_height = fetch_latest_height(&endpoint).await?;

            Ok((latest_height.saturating_sub(last), latest_height))
        }
        (None, None, None) => bail!("Missing data about block range."),
        _ => bail!("`last` flags can't be used with `start` or `end`"),
    }
}

async fn fetch_latest_height(endpoint: &str) -> anyhow::Result<u32> {
    let resp: Response = get_request(endpoint).await?;

    if resp.ok() {
        let resp = match resp.text() {
            Ok(res) => res,
            Err(err) => {
                return Err(anyhow::Error::msg(err.as_string().unwrap_or_default()));
            }
        };
        let resp_text = match JsFuture::from(resp).await {
            Ok(text) => text,
            Err(err) => {
                return Err(anyhow::Error::msg(err.as_string().unwrap_or_default()));
            }
        };
        let resp_string: String = match resp_text.as_string() {
            None => {
                return Err(anyhow::Error::msg("failed to convert resp_test to string"));
            }
            Some(s) => s,
        };
        let latest_height: u32 = resp_string
            .parse()
            .map_err(|_| anyhow::Error::msg("Failed to parse u32 from response"))?;
        Ok(latest_height)
    } else {
        Err(anyhow::Error::msg("Fetch request failed."))
    }
}

/// Fetch owned ciphertext records_web from the endpoint.
async fn fetch_records<N: Network>(
    private_key: Option<PrivateKey<N>>,
    view_key: &ViewKey<N>,
    endpoint: &str,
    start_height: u32,
    end_height: u32,
) -> anyhow::Result<Vec<Record<N, Plaintext<N>>>> {
    // Check the bounds of the request.
    if start_height > end_height {
        bail!("Invalid block range");
    }

    // Derive the x-coordinate of the address corresponding to the given view key.
    let address_x_coordinate = view_key.to_address().to_x_coordinate();

    const MAX_BLOCK_RANGE: u32 = 50;

    let mut records = Vec::new();

    // Calculate the number of blocks to scan.
    // let total_blocks = end_height.saturating_sub(start_height);

    // Scan the endpoint starting from the start height
    let mut request_start = start_height;
    while request_start <= end_height {
        // // Log the progress.
        // let percentage_complete = request_start.saturating_sub(start_height) as f64 * 100.0 / total_blocks as f64;
        // print!("\rScanning {total_blocks} blocks for records_web ({percentage_complete:.2}% complete)...");
        // stdout().flush()?;

        let num_blocks_to_request = std::cmp::min(
            MAX_BLOCK_RANGE,
            end_height.saturating_sub(request_start).saturating_add(1),
        );
        let request_end = request_start.saturating_add(num_blocks_to_request);

        // Establish the endpoint.
        let blocks_endpoint =
            format!("{endpoint}/testnet3/blocks?start={request_start}&end={request_end}");

        // Fetch blocks
        let blocks: Vec<Block<N>> = fetch_blocks(&blocks_endpoint).await?;

        // Scan the blocks for owned records_web.
        for block in &blocks {
            for (commitment, ciphertext_record) in block.records() {
                // Check if the record is owned by the given view key.
                if ciphertext_record
                    .is_owner_with_address_x_coordinate(view_key, &address_x_coordinate)
                {
                    // Decrypt and optionally filter the records_web.
                    if let Some(record) = decrypt_record(
                        private_key,
                        view_key,
                        endpoint,
                        *commitment,
                        ciphertext_record,
                    )
                        .await?
                    {
                        records.push(record);
                    }
                }
            }
        }

        request_start = request_start.saturating_add(num_blocks_to_request);
    }

    // Print final complete message.
    // println!("\rScanning {total_blocks} blocks for records_web (100% complete)...   \n");
    // stdout().flush()?;

    Ok(records)
}

/// Decrypts the ciphertext record and filters spend record if a private key was provided.
async fn decrypt_record<N: Network>(
    private_key: Option<PrivateKey<N>>,
    view_key: &ViewKey<N>,
    endpoint: &str,
    commitment: Field<N>,
    ciphertext_record: &Record<N, Ciphertext<N>>,
) -> anyhow::Result<Option<Record<N, Plaintext<N>>>> {
    // Check if a private key was provided.
    if let Some(private_key) = private_key {
        // Compute the serial number.
        let serial_number = Record::<N, Plaintext<N>>::serial_number(private_key, commitment)?;

        // Establish the endpoint.
        let endpoint = format!("{endpoint}/testnet3/find/transitionID/{serial_number}");

        // Check if the record is spent.
        match get_request(&endpoint).await {
            // On success, skip as the record is spent.
            Ok(_) => Ok(None),
            // On error, add the record.
            Err(_error) => {
                // TODO: Dedup the error types. We're adding the record as valid because the endpoint failed,
                //  meaning it couldn't find the serial number (ie. unspent). However if there's a DNS error or request error,
                //  we have a false positive here then.
                // Decrypt the record.
                Ok(Some(ciphertext_record.decrypt(view_key)?))
            }
        }
    } else {
        // If no private key was provided, return the record.
        Ok(Some(ciphertext_record.decrypt(view_key)?))
    }
}

async fn fetch_blocks<N: Network>(endpoint: &str) -> anyhow::Result<Vec<Block<N>>> {
    let resp: Response = get_request(endpoint).await?;

    if resp.ok() {
        let resp = match resp.text() {
            Ok(res) => res,
            Err(err) => {
                return Err(anyhow::Error::msg(err.as_string().unwrap_or_default()));
            }
        };
        let resp_text = match JsFuture::from(resp).await {
            Ok(text) => text,
            Err(err) => {
                return Err(anyhow::Error::msg(err.as_string().unwrap_or_default()));
            }
        };
        let resp_string: String = match resp_text.as_string() {
            None => {
                return Err(anyhow::Error::msg("failed to convert resp_test to string"));
            }
            Some(s) => s,
        };

        let blocks = serde_json::from_str(&resp_string)
            .map_err(|_| anyhow::Error::msg("Failed to parse Block from response"))?;
        Ok(blocks)
    } else {
        Err(anyhow::Error::msg("Fetch request failed."))
    }
}

// wasm-pack test --chrome
#[cfg(target_arch = "wasm32")]
mod tests {
    use wasm_bindgen_test::{console_log, wasm_bindgen_test, wasm_bindgen_test_configure};
    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    async fn test_request_records_internal() {
        use crate::records::request_records_internal;
        use crate::CurrentNetwork;
        match request_records_internal::<CurrentNetwork>(
            None,
            "AViewKey1mSnpFFC8Mj4fXbK5YiWgZ3mjiV8CxA79bYNa8ymUpTrw".to_string(),
            Some(82870),
            Some(82900),
            None,
            "http://115.231.235.242:33030".to_string(),
        )
            .await
        {
            Ok(records) => {
                for r in records {
                    console_log!("{}", r)
                }
            }
            Err(e) => {
                console_log!("{}", e);
            }
        }
    }
}
