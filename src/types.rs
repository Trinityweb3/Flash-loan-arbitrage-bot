use solana_sdk::pubkey::Pubkey;

pub struct AssetConfig {
    pub reserve: Pubkey,
    pub liquidity_supply: Pubkey,
    pub liquidity_mint: Pubkey,
    pub is_sol: bool,
}
