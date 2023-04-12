/**
 * @Author IronC <apehuang123@gmail.com>
 * @create 2023/4/12 18:13
 * @Project aleo-wallet-test
 *
 * This file is part of aleo-wallet-test.
 */
use std::fs::File;
use std::io::Write;
use std::fs;
use indexmap::IndexMap;
use snarkvm_console_network::prelude::ToBytes;
use snarkvm_console_network::{Testnet3, CREDITS_PROVING_KEYS};
use snarkvm_synthesizer::Program;

const CREDITS_PROVING_KEYS_FILE_PATH: &str = "credits_proving_keys";

type CurrentNetwork = Testnet3;

fn main() {
    if let Ok(file) = fs::metadata(CREDITS_PROVING_KEYS_FILE_PATH) {
        if file.is_file() {
            return;
        }else {
            panic!("{} was existed, but not a file", CREDITS_PROVING_KEYS_FILE_PATH)
        }
    }
    write_credits_proving_keys_into_file()
}

fn write_credits_proving_keys_into_file() {
    let mut new_credits_proving_keys = IndexMap::new();

    let program = Program::<CurrentNetwork>::credits().unwrap();
    for k in program.functions().keys() {
        if let Some(v) = CREDITS_PROVING_KEYS.get(&k.to_string()) {
            new_credits_proving_keys.insert(k.to_string(), v.clone().to_bytes_le().unwrap());
        }
    }

    let serialized_data = bincode::serialize(&new_credits_proving_keys).unwrap();
    let mut file = File::create(CREDITS_PROVING_KEYS_FILE_PATH).unwrap();
    file.write_all(&serialized_data).unwrap();
}
