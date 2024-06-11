use anyhow::Result;
use solana_rpc_client::rpc_client::RpcClient;
use solana_sdk::{
    pubkey,
    signature::{Keypair, Signer},
};
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<()> {
    let url = "https://api.devnet.solana.com".to_string();
    // let url = "http://127.0.0.1:8899".to_string();
    let client = RpcClient::new(url);

    // already existing keypair
    let wallet_file_path = "/Users/username/.config/solana/id.json";
    let owner: Keypair = solana_sdk::signature::read_keypair_file(wallet_file_path)
        .map_err(|_| anyhow::format_err!("failed to read keypair from {}", wallet_file_path))?;
    println!("walletPubkey: {:?}", owner.pubkey());

    solana::helpers::mint_token(
        &client,
        &owner,
        1000000000000000u64,
        pubkey::Pubkey::from_str("3rWiSHrUs4Pb6ewVsNJ62z2tQwTfSaMQ3jvNL5EpcTPK")?,
        &pubkey::Pubkey::from_str("7EM5jrSy2QvmJuEndZhxZhjeN2h6XU1pkBQsa2UZ3xok")?,
    )?;

    Ok(())
}
