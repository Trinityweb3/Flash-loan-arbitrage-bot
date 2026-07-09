use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};
use spl_associated_token_account::get_associated_token_address;
use std::str::FromStr;

pub const METEORA_DLMM_PROGRAM_ID: &str = "LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo";
const MEMO_PROGRAM_ID: &str = "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr";
const MAX_BIN_PER_ARRAY: i32 = 70;
const SWAP2_DISCRIMINATOR: [u8; 8] = [65, 75, 63, 76, 235, 91, 91, 136];

pub struct LbPairInfo {
    pub active_id: i32,
    pub bin_step: u16,
    pub token_x_mint: Pubkey,
    pub token_y_mint: Pubkey,
    pub reserve_x: Pubkey,
    pub reserve_y: Pubkey,
    pub oracle: Pubkey,
}

pub async fn fetch_lb_pair(client: &RpcClient, lb_pair: Pubkey) -> Result<LbPairInfo, Box<dyn std::error::Error>> {
    let account = client.get_account(&lb_pair).await?;
    let d = &account.data;
    if d.len() < 584 {
        return Err("account too short to be a DLMM LbPair".into());
    }
    let active_id: i32 = i32::from_le_bytes(d[76..80].try_into()?);
    let bin_step: u16 = u16::from_le_bytes(d[80..82].try_into()?);
    let token_x_mint: Pubkey = Pubkey::try_from(&d[88..120])?;
    let token_y_mint: Pubkey = Pubkey::try_from(&d[120..152])?;
    let reserve_x: Pubkey = Pubkey::try_from(&d[152..184])?;
    let reserve_y: Pubkey = Pubkey::try_from(&d[184..216])?;
    let oracle: Pubkey = Pubkey::try_from(&d[552..584])?;
    Ok(LbPairInfo { active_id, bin_step, token_x_mint, token_y_mint, reserve_x, reserve_y, oracle })
}

pub async fn scout_quote(
    client: &RpcClient,
    lb_pair: Pubkey,
    input_mint: Pubkey,
    amount_in: u64,
) -> Result<u64, Box<dyn std::error::Error>> {
    let info = fetch_lb_pair(client, lb_pair).await?;
    let x_bal = client.get_token_account_balance(&info.reserve_x).await?.amount.parse::<u64>()?;
    let y_bal = client.get_token_account_balance(&info.reserve_y).await?.amount.parse::<u64>()?;
    let (in_reserve, out_reserve) = if input_mint == info.token_x_mint {
        (x_bal, y_bal)
    } else {
        (y_bal, x_bal)
    };
    let amount_in_with_fee = (amount_in as u128) * 998 / 1000;
    let out = amount_in_with_fee * out_reserve as u128 / (in_reserve as u128 + amount_in_with_fee);
    Ok(out as u64)
}

fn bin_id_to_array_index(bin_id: i32) -> i64 {
    let mut q = (bin_id / MAX_BIN_PER_ARRAY) as i64;
    let r = bin_id % MAX_BIN_PER_ARRAY;
    if bin_id < 0 && r != 0 {
        q -= 1;
    }
    q
}

fn derive_bin_array(lb_pair: &Pubkey, index: i64, program_id: &Pubkey) -> Pubkey {
    let (pda, _) = Pubkey::find_program_address(
        &[b"bin_array", lb_pair.as_ref(), &index.to_le_bytes()],
        program_id,
    );
    pda
}

fn derive_bitmap_extension(lb_pair: &Pubkey, program_id: &Pubkey) -> Pubkey {
    let (pda, _) = Pubkey::find_program_address(&[b"bitmap", lb_pair.as_ref()], program_id);
    pda
}

fn derive_event_authority(program_id: &Pubkey) -> Pubkey {
    let (pda, _) = Pubkey::find_program_address(&[b"__event_authority"], program_id);
    pda
}

pub async fn build_swap_instruction(
    client: &RpcClient,
    lb_pair: Pubkey,
    payer: Pubkey,
    input_mint: Pubkey,
    amount_in: u64,
    min_amount_out: u64,
) -> Result<Instruction, Box<dyn std::error::Error>> {
    let program_id = Pubkey::from_str(METEORA_DLMM_PROGRAM_ID)?;
    let memo_program = Pubkey::from_str(MEMO_PROGRAM_ID)?;
    let info = fetch_lb_pair(client, lb_pair).await?;

    let output_mint = if input_mint == info.token_x_mint { info.token_y_mint } else { info.token_x_mint };
    let user_token_in = get_associated_token_address(&payer, &input_mint);
    let user_token_out = get_associated_token_address(&payer, &output_mint);

    let bitmap_ext = derive_bitmap_extension(&lb_pair, &program_id);
    let bitmap_ext_account = client.get_account(&bitmap_ext).await.ok();
    let bitmap_ext_meta = match bitmap_ext_account {
        Some(_) => AccountMeta::new(bitmap_ext, false),
        None => AccountMeta::new_readonly(program_id, false)
    };

    let center: i64 = bin_id_to_array_index(info.active_id);
    let bin_arrays: Vec<Pubkey> = (center - 2..=center + 2)
        .map(|idx| derive_bin_array(&lb_pair, idx, &program_id))
        .collect();

    let event_authority = derive_event_authority(&program_id);
    let token_program = spl_token::id();

    let mut accounts = vec![
        AccountMeta::new(lb_pair, false),
        bitmap_ext_meta,
        AccountMeta::new(info.reserve_x, false),
        AccountMeta::new(info.reserve_y, false),
        AccountMeta::new(user_token_in, false),
        AccountMeta::new(user_token_out, false),
        AccountMeta::new_readonly(info.token_x_mint, false),
        AccountMeta::new_readonly(info.token_y_mint, false),
        AccountMeta::new(info.oracle, false),
        AccountMeta::new_readonly(program_id, false), 
        AccountMeta::new(payer, true),
        AccountMeta::new_readonly(token_program, false),
        AccountMeta::new_readonly(token_program, false),
        AccountMeta::new_readonly(memo_program, false),
        AccountMeta::new_readonly(event_authority, false),
        AccountMeta::new(program_id, false),
    ];
    for ba in bin_arrays {
        accounts.push(AccountMeta::new(ba, false));
    }

    let mut data = Vec::with_capacity(28);
    data.extend_from_slice(&SWAP2_DISCRIMINATOR);
    data.extend_from_slice(&amount_in.to_le_bytes());
    data.extend_from_slice(&min_amount_out.to_le_bytes());
    data.extend_from_slice(&0u32.to_le_bytes()); 

    Ok(Instruction { program_id, accounts, data })
}
