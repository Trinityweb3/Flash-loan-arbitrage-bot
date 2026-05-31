use solana_sdk::pubkey::Pubkey;

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
