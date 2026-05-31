use solana_sdk::pubkey::Pubkey;

//Using Save Finance
// see https://docs.save.finance/architecture/addresses/mainnet/main-pools for more
pub const PROGRAM_ID_STR: &str = "So1endDq2YkqhipRh3WViPa8hdiSpxWy6z3Z6tMCpAo";
pub const LENDING_MARKET_STR: &str = "4UpD2fh7xH3VP9QQaXtsS1YY3bxzWhtfpks7FatyKvdY";
pub const VAULT_STR: &str = "DdZR6zRFiUt4S5mg7AV1uKB2z1f1WzcNYCaTEEWPAuby";

pub const USDC_RESERVE: &str = "BgxfHJDzm44T7XG68MYKx7YisTjZu73tVovyZSjJMpmw";
pub const SOL_RESERVE: &str = "8PbodeaosQP19SjYFx855UMqWxH2HynZLdBXmsrbac36";

pub const USDC_LIQUIDITY_SUPPLY: &str = "8SheGtsopRUDzdiD6v6BR9a6bqZ9QwywYQY99Fp5meNf";
pub const SOL_LIQUIDITY_SUPPLY: &str = "8UviNr47S8eL6J3WfDxMRa3hvLta1VDJwNWqsDgtN3Cv";

pub const USDC_LIQUIDITY_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
pub const SOL_LIQUIDITY_MINT: &str = "So11111111111111111111111111111111111111112";

pub const FEE_RECEIVER: &str = "9RuqAN42PTUi9ya59k9suGATrkqzvb9gk2QABJtQzGP5";

pub struct AssetConfig {
    pub loan_amount: u64,
    pub reserve: Pubkey,
    pub liquidity_supply: Pubkey,
    pub liquidity_mint: Pubkey,
    pub is_sol: bool,
}

pub struct ProtocolConfig {
    pub program_id: Pubkey,
    pub lending_market: Pubkey,
    pub vault: Pubkey,
    pub fee_receiver: Pubkey
}

pub struct User {
    pub rpc_url: String,
    pub private_key: String
}
