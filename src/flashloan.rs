use crate::types::*;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_system_interface::instruction as system_instruction;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    sysvar,
};
use solana_message::{v0, VersionedMessage, AddressLookupTableAccount};
use solana_sdk::transaction::VersionedTransaction;
use spl_associated_token_account::{
    get_associated_token_address,
    instruction::create_associated_token_account_idempotent,
};
use spl_token::instruction::sync_native;
use orca_whirlpools::{swap_instructions, SwapType::ExactIn};
use rand::seq::SliceRandom;
use std::str::FromStr;

pub async fn execute_flash_loan(
    user: User,
    protocol_config: ProtocolConfig,
    asset: AssetConfig,
    pools: SwapPools
) -> Result<(), Box<dyn std::error::Error>> {
    let client: RpcClient = RpcClient::new_with_commitment(user.rpc_url, CommitmentConfig::confirmed());
    let payer: Keypair = Keypair::from_base58_string(&user.private_key);
    println!("wallet: {}", payer.pubkey());

    let lending_market = protocol_config.lending_market;
    let fee_receiver: Pubkey = protocol_config.fee_receiver;
    let vault: Pubkey = protocol_config.vault;
    let program_id: Pubkey = protocol_config.program_id;
    let reserve_account: Pubkey = asset.reserve;
    let reserve_liquidity_supply: Pubkey = asset.liquidity_supply;
    let loan_amount: u64 = asset.loan_amount;

    let wsol_mint = Pubkey::from_str_const(SOL_LIQUIDITY_MINT);
    let usdc_mint: Pubkey = Pubkey::from_str_const(USDC_LIQUIDITY_MINT);

    // jito tip
    let tips: [&str; 8] = [
        JITO_TIP1, JITO_TIP2, JITO_TIP3, JITO_TIP4,
        JITO_TIP5, JITO_TIP6, JITO_TIP7, JITO_TIP8,
    ];
    let tip_pubkey: Pubkey = Pubkey::from_str_const(tips.choose(&mut rand::thread_rng()).unwrap());
    let jito_tip_ix: Instruction = system_instruction::transfer(&payer.pubkey(), &tip_pubkey, 10_000);

    // alt
    let alt_address: Pubkey = Pubkey::from_str(
        &std::env::var("ALT_ADDRESS").expect("ALT_ADDRESS not set in .env")
    )?;
    let lut_account: solana_sdk::account::Account = client.get_account(&alt_address).await?;
    let lut_addresses: Vec<Pubkey> = lut_account.data[56..]
        .chunks(32)
        .filter(|c| c.len() == 32)
        .map(|c| Pubkey::try_from(c).unwrap())
        .collect();
    let lut: AddressLookupTableAccount = AddressLookupTableAccount { key: alt_address, addresses: lut_addresses };

    if asset.is_sol {
        let wsol_ata: Pubkey = get_associated_token_address(&payer.pubkey(), &wsol_mint);
        let fee_receiver_ata: Pubkey = get_associated_token_address(&fee_receiver, &wsol_mint);

        println!("quote: SOL -> USDC...");
        let swap_fwd: orca_whirlpools::SwapInstructions = swap_instructions(
            &client,
            Pubkey::from_str_const(ORCA_SOL_USDC_MARKET),
            loan_amount,
            Pubkey::from_str_const(SOL_LIQUIDITY_MINT),
            ExactIn,
            Some(10u16),
            Some(payer.pubkey()),
        ).await?;
        
        let usdc_received = match &swap_fwd.quote {
            orca_whirlpools::SwapQuote::ExactIn(q) => q.token_est_out,
            orca_whirlpools::SwapQuote::ExactOut(q) => q.token_est_in,
        };
        println!("  {:.6} SOL -> {:.6} USDC",
            loan_amount as f64 / 1_000_000_000.0,
            usdc_received as f64 / 1_000_000.0);

        println!("quote: USDC -> SOL...");
        let swap_back: orca_whirlpools::SwapInstructions = swap_instructions(
            &client,
            Pubkey::from_str(&pools.orca_pool)?,
            usdc_received,
            Pubkey::from_str_const(USDC_LIQUIDITY_MINT),
            ExactIn,
            Some(10u16),
            Some(payer.pubkey()),
        ).await?;
        
        let sol_received = match &swap_back.quote {
            orca_whirlpools::SwapQuote::ExactIn(q) => q.token_est_out,
            orca_whirlpools::SwapQuote::ExactOut(q) => q.token_est_in,
        };
        println!("  {:.6} USDC -> {:.6} SOL",
            usdc_received as f64 / 1_000_000.0,
            sol_received as f64 / 1_000_000_000.0);
        println!("pnl: {:.6} SOL", (sol_received as i64 - loan_amount as i64) as f64 / 1_000_000_000.0);

        let mut instructions: Vec<Instruction> = Vec::new();

        instructions.push(create_associated_token_account_idempotent(
            &payer.pubkey(), &payer.pubkey(), &wsol_mint, &spl_token::id(),
        ));
        instructions.push(create_associated_token_account_idempotent(
            &payer.pubkey(), &payer.pubkey(), &usdc_mint, &spl_token::id(),
        ));
        instructions.push(system_instruction::transfer(&payer.pubkey(), &wsol_ata, 10_000_000));
        instructions.push(sync_native(&spl_token::id(), &wsol_ata)?);

        let mut borrow_data: Vec<u8> = Vec::with_capacity(9);
        borrow_data.push(19u8);
        borrow_data.extend_from_slice(&loan_amount.to_le_bytes());
        instructions.push(Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(reserve_liquidity_supply, false),
                AccountMeta::new(wsol_ata, false),
                AccountMeta::new(reserve_account, false),
                AccountMeta::new_readonly(lending_market, false),
                AccountMeta::new_readonly(vault, false),
                AccountMeta::new_readonly(sysvar::instructions::id(), false),
                AccountMeta::new_readonly(spl_token::id(), false),
            ],
            data: borrow_data,
        });

        instructions.extend(swap_fwd.instructions);
        instructions.extend(swap_back.instructions);

        let mut repay_data = Vec::with_capacity(10);
        repay_data.push(20u8);
        repay_data.extend_from_slice(&loan_amount.to_le_bytes());
        repay_data.push(4);
        instructions.push(Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(wsol_ata, false),
                AccountMeta::new(reserve_liquidity_supply, false),
                AccountMeta::new(fee_receiver_ata, false),
                AccountMeta::new(wsol_ata, false),
                AccountMeta::new(reserve_account, false),
                AccountMeta::new_readonly(lending_market, false),
                AccountMeta::new_readonly(payer.pubkey(), true),
                AccountMeta::new_readonly(sysvar::instructions::id(), false),
                AccountMeta::new_readonly(spl_token::id(), false),
            ],
            data: repay_data,
        });

        instructions.push(jito_tip_ix);

        println!("instructions: {}", instructions.len());

        let mut signers: Vec<&Keypair> = vec![&payer];
        signers.extend(swap_fwd.additional_signers.iter());
        signers.extend(swap_back.additional_signers.iter());

        let blockhash = client.get_latest_blockhash().await?;
        let v0_msg: v0::Message = v0::Message::try_compile(&payer.pubkey(), &instructions, &[lut], blockhash)?;
        let vtx: VersionedTransaction = VersionedTransaction::try_new(VersionedMessage::V0(v0_msg), &signers)?;

        println!("simulating...");
        let sim = client.simulate_transaction(&vtx).await?;
        match sim.value.err {
            Some(err) => {
                println!("simulation failed: {:?}", err);
                if let Some(logs) = sim.value.logs {
                    for log in &logs { println!("  {}", log); }
                }
            }
            None => {
                println!("simulation ok, sending...");
                match client.send_and_confirm_transaction(&vtx).await {
                    Ok(sig) => println!("success: https://solscan.io/tx/{}", sig),
                    Err(e)  => println!("send error: {:?}", e),
                }
            }
        }

    } else {
        let usdc_ata: Pubkey = get_associated_token_address(&payer.pubkey(), &usdc_mint);
        let fee_receiver_ata: Pubkey = get_associated_token_address(&fee_receiver, &usdc_mint);

        // quotes (for future arbitrage logic)
        println!("quote: USDC -> SOL...");
        let swap_fwd = swap_instructions(
            &client,
            Pubkey::from_str(&pools.orca_pool)?,
            loan_amount,
            Pubkey::from_str_const(USDC_LIQUIDITY_MINT),
            ExactIn,
            Some(10u16),
            Some(payer.pubkey()),
        ).await?;
        
        let sol_received = match &swap_fwd.quote {
            orca_whirlpools::SwapQuote::ExactIn(q) => q.token_est_out,
            orca_whirlpools::SwapQuote::ExactOut(q) => q.token_est_in,
        };
        println!("  {:.6} USDC -> {:.6} SOL",
            loan_amount as f64 / 1_000_000.0,
            sol_received as f64 / 1_000_000_000.0);

        println!("quote: SOL -> USDC...");
        let swap_back = swap_instructions(
            &client,
            Pubkey::from_str_const(ORCA_SOL_USDC_MARKET),
            sol_received,
            Pubkey::from_str_const(SOL_LIQUIDITY_MINT),
            ExactIn,
            Some(10u16),
            Some(payer.pubkey()),
        ).await?;

        let usdc_received: u64 = match &swap_back.quote {
            orca_whirlpools::SwapQuote::ExactIn(q) => q.token_est_out,
            orca_whirlpools::SwapQuote::ExactOut(q) => q.token_est_in,
        };

        println!("  {:.6} SOL -> {:.6} USDC",
            sol_received as f64 / 1_000_000_000.0,
            usdc_received as f64 / 1_000_000.0);
        println!("pnl: {:.6} USDC", (usdc_received as i64 - loan_amount as i64) as f64 / 1_000_000.0);

        // build ixs
        let mut instructions: Vec<Instruction> = Vec::new();

        instructions.push(create_associated_token_account_idempotent(
            &payer.pubkey(), &payer.pubkey(), &usdc_mint, &spl_token::id(),
        ));
        instructions.push(create_associated_token_account_idempotent(
            &payer.pubkey(), &payer.pubkey(), &wsol_mint, &spl_token::id(),
        ));

        let mut borrow_data: Vec<u8> = Vec::with_capacity(9);
        borrow_data.push(19u8);
        borrow_data.extend_from_slice(&loan_amount.to_le_bytes());
        instructions.push(Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(reserve_liquidity_supply, false),
                AccountMeta::new(usdc_ata, false),
                AccountMeta::new(reserve_account, false),
                AccountMeta::new_readonly(lending_market, false),
                AccountMeta::new_readonly(vault, false),
                AccountMeta::new_readonly(sysvar::instructions::id(), false),
                AccountMeta::new_readonly(spl_token::id(), false),
            ],
            data: borrow_data,
        });

        instructions.extend(swap_fwd.instructions);
        instructions.extend(swap_back.instructions);

        let mut repay_data: Vec<u8> = Vec::with_capacity(10);
        repay_data.push(20u8);
        repay_data.extend_from_slice(&loan_amount.to_le_bytes());
        repay_data.push(2); //borrow byte
        instructions.push(Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(usdc_ata, false),
                AccountMeta::new(reserve_liquidity_supply, false),
                AccountMeta::new(fee_receiver_ata, false),
                AccountMeta::new(usdc_ata, false),
                AccountMeta::new(reserve_account, false),
                AccountMeta::new_readonly(lending_market, false),
                AccountMeta::new_readonly(payer.pubkey(), true),
                AccountMeta::new_readonly(sysvar::instructions::id(), false),
                AccountMeta::new_readonly(spl_token::id(), false),
            ],
            data: repay_data,
        });

        instructions.push(jito_tip_ix);

        println!("instructions: {}", instructions.len());

        let mut signers: Vec<&Keypair> = vec![&payer];
        signers.extend(swap_fwd.additional_signers.iter());
        signers.extend(swap_back.additional_signers.iter());

        let blockhash = client.get_latest_blockhash().await?;
        let v0_msg: v0::Message = v0::Message::try_compile(&payer.pubkey(), &instructions, &[lut], blockhash)?;
        let vtx: VersionedTransaction = VersionedTransaction::try_new(VersionedMessage::V0(v0_msg), &signers)?;

        println!("simulating...");
        let sim= client.simulate_transaction(&vtx).await?;
        match sim.value.err {
            Some(err) => {
                println!("simulation failed: {:?}", err);
                if let Some(logs) = sim.value.logs {
                    for log in &logs { println!("  {}", log); }
                }
            }
            None => {
                println!("simulation ok, sending...");
                match client.send_and_confirm_transaction(&vtx).await {
                    Ok(sig) => println!("success: https://solscan.io/tx/{}", sig),
                    Err(e)  => println!("send error: {:?}", e),
                }
            }
        }
    }

    Ok(())
}
