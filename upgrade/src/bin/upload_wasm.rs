use std::fs::File;
use std::io::prelude::*;

use sp_keyring::AccountKeyring;
use subxt::{tx::PairSigner, OnlineClient, SubstrateConfig};

#[subxt::subxt(runtime_metadata_path = "../subxtproxy/metadata.scale")]
pub mod substrate {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    //tracing_subscriber::fmt::init();
    let key = "WASMFILE_TOUPLOAD";
    let wasmfile_path: String = dotenv::var(key).unwrap();

    let signer = PairSigner::new(AccountKeyring::Alice.pair());

    // Create a client to use:
    let api = OnlineClient::<SubstrateConfig>::new().await?;

    // read wasm file content
    let mut file = File::open(wasmfile_path)?;
    let mut contents: Vec<u8> = Vec::new();
    file.read_to_end(&mut contents)?;
    println!("Wasm file content length: {}", contents.len());

    // Create a transaction to submit:
    let tx = substrate::tx().eight_fish_module().wasm_upgrade(contents);

    // Submit the transaction with default params:
    let hash = api.tx().sign_and_submit_default(&tx, &signer).await?;

    println!("Wasm file content upload transaction submitted: {}", hash);

    Ok(())
}
