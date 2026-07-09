mod types;
mod handlers;
mod meteora;

use types::*;
use handlers::execute;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer};
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let user: User = User {
        rpc_url: std::env::var("RPC_URL")?,
        private_key: std::env::var("PRIVATE_KEY")?,
    };

    let protocol_config: ProtocolConfig = ProtocolConfig {
        program_id: Pubkey::from_str(PROGRAM_ID_STR)?,
        lending_market: Pubkey::from_str(LENDING_MARKET_STR)?,
        vault: Pubkey::from_str(VAULT_STR)?,
        fee_receiver: Pubkey::from_str(FEE_RECEIVER)?,
    };

    let asset: AssetConfig = AssetConfig {
        loan_amount: 1_000_0000, // 0.1 SOL
        reserve: Pubkey::from_str(SOL_RESERVE)?,
        liquidity_supply: Pubkey::from_str(SOL_LIQUIDITY_SUPPLY)?,
        liquidity_mint: Pubkey::from_str(SOL_LIQUIDITY_MINT)?,
    };

    let client: RpcClient = RpcClient::new_with_commitment(user.rpc_url, CommitmentConfig::confirmed());
    let payer: Keypair = Keypair::from_base58_string(&user.private_key);
    println!("Wallet: {}", payer.pubkey());

    let alt_address = Pubkey::from_str(&std::env::var("ALT_ADDRESS")?)?;

    let route = vec![
        Dex::Orca { pool: Pubkey::from_str(ORCA_SOL_USDC_MARKET)? },
        Dex::MeteoraDlmm { lb_pair: Pubkey::from_str("BGm1tav58oGcsQJehL9WXBFXF7D27vZsKefj4xJKD5Y")? },
    ];

    execute(&client, &payer, &protocol_config, &asset, alt_address, &route).await
}
