use std::path::PathBuf;
use std::str::FromStr;
use anyhow::{bail, ensure};
use serde_json::from_str;
use snarkvm_console_account::address::Address;
use snarkvm_console_account::PrivateKey;
use snarkvm_console_network::Network;
use snarkvm_console_program::{Identifier, Locator, Plaintext, ProgramID, Record, Value};
use snarkvm_synthesizer::{ConsensusMemory, ConsensusStore, Query, Transaction, VM};
use snarkvm_utilities::ToBytes;
use wasm_bindgen_futures::JsFuture;
use crate::utils::post_request;

/**
 * @Author IronC <apehuang123@gmail.com>
 * @create 2023/4/12 10:34
 * @Project aleo-wallet-test
 *
 * This file is part of aleo-wallet-test.
 */

pub(crate) async fn transfer_internal<N: Network>(
    private_key: String,
    record: String,
    amount: u64,
    recipient: String,
    query_endpoint: String,
    broadcast: String,
) -> anyhow::Result<String> {
    let record = Record::<N, Plaintext<N>>::from_str(&record)?;
    let recipient = Address::<N>::from_str(&recipient)?;

    // Specify the query
    let query = Query::from(&query_endpoint);

    // Retrieve the private key.
    let private_key = PrivateKey::from_str(&private_key)?;
    // Generate the transfer transaction.
    let execution = {
        // Initialize an RNG.
        let rng = &mut rand::thread_rng();

        // Initialize the VM.
        let store = ConsensusStore::<N, ConsensusMemory<N>>::open(None)?;
        let vm = VM::from(store)?;

        // Prepare the fees.
        // let fee = match self.fee_record {
        //     Some(record) => {
        //         let record = Record::<N, Plaintext<N>>::from_str(&record)?;
        //         let fee_amount = self.fee.unwrap_or(0);
        //
        //         Some((record, fee_amount))
        //     }
        //     None => None,
        // };

        // Prepare the inputs for a transfer.
        let inputs = vec![
            Value::Record(record.clone()),
            Value::from_str(&format!("{}", recipient))?,
            Value::from_str(&format!("{}u64", amount))?,
        ];

        // Create a new transaction.
        Transaction::execute(
            &vm,
            &private_key,
            ProgramID::from_str("credits.aleo")?,
            Identifier::from_str("transfer")?,
            inputs.iter(),
            None,
            Some(query),
            rng,
        )?
    };
    let locator = Locator::<N>::from_str("credits.aleo/transfer")?;
    // Determine if the transaction should be broadcast, stored, or displayed to user.
    handle_transaction(Some(broadcast), false, None, execution, locator.to_string()).await
}

async fn handle_transaction<N: Network>(
    broadcast: Option<String>,
    display: bool,
    store: Option<String>,
    transaction: Transaction<N>,
    operation: String,
) -> anyhow::Result<String> {
    // Get the transaction id.
    let transaction_id = transaction.id();

    // Determine if the transaction should be stored.
    if let Some(path) = store {
        match PathBuf::from_str(&path) {
            Ok(file_path) => {
                let transaction_bytes = transaction.to_bytes_le()?;
                std::fs::write(&file_path, transaction_bytes)?;
                println!("Transaction {transaction_id} was stored to {}", file_path.display());
            }
            Err(err) => {
                println!("The transaction was unable to be stored due to: {err}");
            }
        }
    };

    // Determine if the transaction should be broadcast or displayed to user.
    if let Some(endpoint) = broadcast {
        // Send the deployment request to the local development node.
        let transaction_json = serde_json::to_value(&transaction)?;
        match post_request(&endpoint, &transaction_json).await {
            Ok(response) => {
                let response_text_future = response.text().map_err(|js_value| anyhow::Error::msg(format!("{:?}", js_value)))?;
                let response_text = JsFuture::from(response_text_future).await.map_err(|js_value| anyhow::Error::msg(format!("{:?}", js_value)))?;
                let response_text_str = response_text.as_string().unwrap_or_default();
                let id: serde_json::Value = from_str(&response_text_str)?;
                ensure!(
                        id == transaction_id.to_string(),
                        "The response does not match the transaction id"
                    );

                match transaction {
                    Transaction::Deploy(..) => {
                        println!("✅ Successfully deployed '{}' to {}.", operation, endpoint)
                    }
                    Transaction::Execute(..) => {
                        println!("✅ Successfully broadcast execution '{}' to the {}.", operation, endpoint)
                    }
                }
            }
            Err(error) => {
                let error_message = format!("({})", error);

                match transaction {
                    Transaction::Deploy(..) => {
                        bail!("❌ Failed to deploy '{}' to {}: {}", operation, &endpoint, error_message)
                    }
                    Transaction::Execute(..) => {
                        bail!(
                                "❌ Failed to broadcast execution '{}' to {}: {}",
                                operation,
                                &endpoint,
                                error_message
                            )
                    }
                }
            }
        };

        // Output the transaction id.
        Ok(transaction_id.to_string())
    } else if display {
        // Output the transaction string.
        Ok(transaction.to_string())
    } else {
        // TODO (raychu86): Handle the case where the user does not specify a broadcast or display flag.
        Ok("".to_string())
    }
}


// wasm-pack test --chrome
// #[cfg(target_arch = "wasm32")]
mod tests {
    use wasm_bindgen_test::{console_log, wasm_bindgen_test, wasm_bindgen_test_configure};
    wasm_bindgen_test_configure!(run_in_browser);

    const TRANSFER_CONF_DATA: &'static [u8] = include_bytes!("transfer_conf");

    #[wasm_bindgen_test]
    async fn test_transfer_internal() {
        use std::path::Path;
        use std::fs::File;
        use std::str::FromStr;
        use std::io;
        use std::io::BufRead;
        use crate::transfer::transfer_internal;
        use crate::CurrentNetwork;

        let file_contents = std::str::from_utf8(TRANSFER_CONF_DATA).unwrap();
        let conf = file_contents.split('\n').into_iter().map(|c|c.to_string()).collect::<Vec<String>>();

        console_log!("{}", conf[3].clone());

        let msg = transfer_internal::<CurrentNetwork>(conf[0].clone(), conf[3].clone(), u64::from_str(&conf[4]).unwrap(), conf[5].clone(), conf[1].clone(), conf[2].clone()).await.unwrap();
        console_log!("{}", msg)
    }

    #[test]
    fn test_transfer_conf_data() {
        let file_contents = std::str::from_utf8(TRANSFER_CONF_DATA).unwrap();
        let split = file_contents.split('\n').into_iter().map(|c|c.to_string()).collect::<Vec<String>>();
        println!("{:?}", split);
    }
}