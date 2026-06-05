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

pub const ORCA_MARKET: &str = "Czfq3xZZDmsdGdUyrNLtRhGc47cXcZtLG4crryfu44zE";

pub const JITO_TIP1: &str = "96gYZGLnJYVFmbjzopPSU6QiEV5fGqZNyN9nmNhvrZU5";
pub const JITO_TIP2: &str = "HFqU5x63VTqvQss8hp11i4wVV8bD44PvwucfZ2bU7gRe";
pub const JITO_TIP3: &str = "Cw8CFyM9FkoMi7K7Crf6HNQqf4uEMzpKw6QNghXLvLkY";
pub const JITO_TIP4: &str = "ADaUMid9yfUytqMBgopwjb2DTLSokTSzL1zt6iGPaS49";
pub const JITO_TIP5: &str = "DfXygSm4jCyNCybVYYK6DwvWqjKee8pbDmJGcLWNDXjh";
pub const JITO_TIP6: &str = "ADuUkR4vqLUMWXxW9gh6D6L8pMSawimctcNZ5pGwDcEt";
pub const JITO_TIP7: &str = "DttWaMuVvTiduZRnguLF7jNxTgiMBZ1hyAumKUiL2KRL";
pub const JITO_TIP8: &str = "3AVi9Tg9Uo68tJfuvoKvqKNWKkC5wPdSSdeBnizKZ6jT";

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

pub struct SwapPools {
    pub orca_pool: Pubkey
}
