mod types;
mod flashloan;

use types::*;

use flashloan::execute_flash_loan;
use std::str::FromStr;
use solana_sdk::pubkey::Pubkey;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let user: User = User {
        rpc_url: std::env::var("RPC_URL")?,
        private_key: std::env::var("PRIVATE_KEY")?
    };

    let protocol_config: ProtocolConfig = ProtocolConfig { 
        program_id: Pubkey::from_str(types::PROGRAM_ID_STR)?, 
        lending_market: Pubkey::from_str(types::LENDING_MARKET_STR)?, 
        vault: Pubkey::from_str(types::VAULT_STR)?, 
        fee_receiver: Pubkey::from_str(types::FEE_RECEIVER)? 
    };

    let asset: AssetConfig = AssetConfig {
        loan_amount: 1_000_000,
        reserve: Pubkey::from_str(types::SOL_RESERVE)?,
        liquidity_supply: Pubkey::from_str(types::SOL_LIQUIDITY_SUPPLY)?,
        liquidity_mint: Pubkey::from_str(types::SOL_LIQUIDITY_MINT)?,
        is_sol: true,
    };

    let pools: SwapPools = SwapPools {
        orca_pool: types::ORCA_SOL_USDC_MARKET.to_string()
    };

    match execute_flash_loan(user, protocol_config, asset, pools).await {
        Ok(_) => {},
        Err(e) => eprintln!("process crashed with error: {:?}", e),
    }

    Ok(())
}
