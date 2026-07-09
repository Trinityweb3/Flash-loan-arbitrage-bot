use crate::types::*;
use crate::meteora;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    sysvar,
};
use solana_system_interface::instruction as system_instruction;
use spl_associated_token_account::get_associated_token_address;
use orca_whirlpools::{swap_instructions, SwapType::ExactIn};
use std::str::FromStr;
use std::collections::HashSet;
use solana_message::{v0, VersionedMessage, AddressLookupTableAccount};
use solana_sdk::transaction::VersionedTransaction;
use solana_address_lookup_table_interface::instruction::extend_lookup_table;

pub struct SwapLeg {
    pub instructions: Vec<Instruction>,
    pub estimated_out: u64,
}

pub fn build_borrow_instruction(protocol_config: &ProtocolConfig, asset: &AssetConfig, dest_ata: Pubkey) -> Instruction {
    let mut data: Vec<u8> = Vec::with_capacity(9);
    data.push(19u8);
    data.extend_from_slice(&asset.loan_amount.to_le_bytes());
    Instruction {
        program_id: protocol_config.program_id,
        accounts: vec![
            AccountMeta::new(asset.liquidity_supply, false),
            AccountMeta::new(dest_ata, false),
            AccountMeta::new(asset.reserve, false),
            AccountMeta::new_readonly(protocol_config.lending_market, false),
            AccountMeta::new_readonly(protocol_config.vault, false),
            AccountMeta::new_readonly(sysvar::instructions::id(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
        data,
    }
}

pub fn build_repay_instruction(protocol_config: &ProtocolConfig, asset: &AssetConfig, source_ata: Pubkey, fee_receiver_ata: Pubkey, payer: Pubkey, borrow_ix_index: u8) -> Instruction {
    let mut data: Vec<u8> = Vec::with_capacity(10);
    data.push(20u8);
    data.extend_from_slice(&asset.loan_amount.to_le_bytes());
    data.push(borrow_ix_index);
    Instruction {
        program_id: protocol_config.program_id,
        accounts: vec![
            AccountMeta::new(source_ata, false),
            AccountMeta::new(asset.liquidity_supply, false),
            AccountMeta::new(fee_receiver_ata, false),
            AccountMeta::new(source_ata, false),
            AccountMeta::new(asset.reserve, false),
            AccountMeta::new_readonly(protocol_config.lending_market, false),
            AccountMeta::new_readonly(payer, true),
            AccountMeta::new_readonly(sysvar::instructions::id(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
        data,
    }
}

pub async fn build_swap_leg(
    client: &RpcClient,
    payer: Pubkey,
    dex: &Dex,
    input_mint: Pubkey,
    amount_in: u64,
    slippage_bps: u16,
) -> Result<SwapLeg, Box<dyn std::error::Error>> {
    match dex {
        Dex::Orca { pool } => {
            let swap = swap_instructions(
                client, 
                *pool, 
                amount_in, 
                input_mint, 
                ExactIn, 
                Some(slippage_bps), 
                Some(payer)
            ).await?;

            let estimated_out = match &swap.quote {
                orca_whirlpools::SwapQuote::ExactIn(q) => q.token_est_out,
                orca_whirlpools::SwapQuote::ExactOut(q) => q.token_est_in,
            };

            let orca_program = Pubkey::from_str_const("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc");
            let mut swap_ix = swap.instructions.iter()
                .find(|ix| ix.program_id == orca_program)
                .ok_or("Orca swap instruction not found")?
                .clone();

            if !swap.additional_signers.is_empty() {
                let temp_wsol = swap.additional_signers[0].pubkey();
                let wsol_mint = Pubkey::from_str_const("So11111111111111111111111111111111111111112");
                let wsol_ata = get_associated_token_address(&payer, &wsol_mint);

                for meta in swap_ix.accounts.iter_mut() {
                    if meta.pubkey == temp_wsol {
                        meta.pubkey = wsol_ata;
                        meta.is_writable = true;
                    }
                }
            }

            Ok(SwapLeg { 
                instructions: vec![swap_ix], 
                estimated_out 
            })
        }
        Dex::MeteoraDlmm { lb_pair } => {
            let rough = meteora::scout_quote(client, *lb_pair, input_mint, amount_in).await.unwrap_or(0);
            if rough == 0 {
                return Err("Meteora quote failed".into());
            }
            let min_out: u64 = 0; // Временно для теста
            let ix = meteora::build_swap_instruction(client, *lb_pair, payer, input_mint, amount_in, min_out).await?;
            Ok(SwapLeg { instructions: vec![ix], estimated_out: rough })
        }
    }
}

pub async fn execute(
    client: &RpcClient,
    payer: &Keypair,
    protocol_config: &ProtocolConfig,
    asset: &AssetConfig,
    alt_address: Pubkey,
    route: &[Dex],
) -> Result<(), Box<dyn std::error::Error>> {
    let wsol_mint = Pubkey::from_str_const(SOL_LIQUIDITY_MINT);
    let usdc_mint = Pubkey::from_str_const(USDC_LIQUIDITY_MINT);

    let start_ata = get_associated_token_address(&payer.pubkey(), &wsol_mint);
    let fee_receiver_ata = get_associated_token_address(&protocol_config.fee_receiver, &wsol_mint);

    let mut instructions: Vec<Instruction> = Vec::new();
    let borrow_ix_index: u8 = 0u8;
    
    instructions.push(build_borrow_instruction(protocol_config, asset, start_ata));

    let mut current_mint = wsol_mint;
    let mut current_amount = asset.loan_amount;
    for dex in route {
        let leg = build_swap_leg(client, payer.pubkey(), dex, current_mint, current_amount, 100).await?;
        instructions.extend(leg.instructions);
        current_amount = leg.estimated_out;
        current_mint = if current_mint == wsol_mint { usdc_mint } else { wsol_mint };
    }

    instructions.push(build_repay_instruction(protocol_config, asset, start_ata, fee_receiver_ata, payer.pubkey(), borrow_ix_index));

    let tip_pubkey: Pubkey = Pubkey::from_str(JITO_TIP4)?;
    instructions.push(system_instruction::transfer(&payer.pubkey(), &tip_pubkey, 1_000));

    let pnl: i64 = current_amount as i64 - asset.loan_amount as i64;
    println!("Route PnL ~ {} lamports", pnl);
    if pnl <= 0 {
        println!("Route isn't profitable, skipping...");
        return Ok(());
    }

    let lut_account = client.get_account(&alt_address).await?;
    let lut_addresses: Vec<Pubkey> = lut_account.data[56..]
        .chunks(32)
        .filter(|c| c.len() == 32)
        .map(|c| Pubkey::try_from(c).unwrap())
        .collect();
        
    let mut lut = AddressLookupTableAccount { key: alt_address, addresses: lut_addresses };
    
    let tip_set: HashSet<Pubkey> = [
        Pubkey::from_str(JITO_TIP1)?, Pubkey::from_str(JITO_TIP2)?,
        Pubkey::from_str(JITO_TIP3)?, Pubkey::from_str(JITO_TIP4)?,
        Pubkey::from_str(JITO_TIP5)?, Pubkey::from_str(JITO_TIP6)?,
        Pubkey::from_str(JITO_TIP7)?, Pubkey::from_str(JITO_TIP8)?,
    ].iter().cloned().collect();

    let needed: HashSet<Pubkey> = instructions.iter().flat_map(|ix| ix.accounts.iter().map(|a| a.pubkey)).collect();
    let existing: HashSet<Pubkey> = lut.addresses.iter().cloned().collect();
    let missing: Vec<Pubkey> = needed.iter().cloned()
        .filter(|k| !existing.contains(k) && *k != payer.pubkey() && !tip_set.contains(k))
        .collect();

    if !missing.is_empty() {
        println!("ALT: добавляю {} новых адресов...", missing.len());
        for chunk in missing.chunks(20) {
            let ix = extend_lookup_table(alt_address, payer.pubkey(), Some(payer.pubkey()), chunk.to_vec());
            let blockhash = client.get_latest_blockhash().await?;
            let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(&[ix], Some(&payer.pubkey()), &[payer], blockhash);
            let sig = client.send_and_confirm_transaction(&tx).await?;
            println!("  extend tx: {sig}");
        }
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        
        let lut_account = client.get_account(&alt_address).await?;
        lut.addresses = lut_account.data[56..]
            .chunks(32)
            .filter(|c| c.len() == 32)
            .map(|c| Pubkey::try_from(c).unwrap())
            .collect();
    }

    // --- Отправка ---
    println!("Total instructions: {}", instructions.len());
    let blockhash = client.get_latest_blockhash().await?;
    let v0_msg = v0::Message::try_compile(&payer.pubkey(), &instructions, std::slice::from_ref(&lut), blockhash)?;
    let vtx = VersionedTransaction::try_new(VersionedMessage::V0(v0_msg), &[payer])?;

    println!("Simulating...");
    let sim = client.simulate_transaction(&vtx).await?;
    match sim.value.err {
        Some(err) => {
            println!("Simulation failed: {:?}", err);
            if let Some(logs) = sim.value.logs {
                for log in &logs { println!("  {}", log); }
            }
        }
        None => {
            println!("Simulation ok. Sending via standard RPC...");
            match client.send_and_confirm_transaction(&vtx).await {
                Ok(sig) => println!("success: https://solscan.io/tx/{}", sig),
                Err(e) => println!("error! Logs: {:?}", e),
            }
        }
    }
    
    Ok(())
}
