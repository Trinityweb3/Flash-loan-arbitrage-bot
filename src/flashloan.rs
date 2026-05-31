use crate::types::*;

use solana_client::nonblocking::rpc_client::RpcClient; // for async feature
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    transaction::Transaction,
};

use spl_associated_token_account::instruction::create_associated_token_account_idempotent;
use solana_sdk::system_instruction;
use spl_token::instruction::sync_native;
use spl_associated_token_account::get_associated_token_address;

pub async fn execute_flash_loan(user: User, protocol_config: ProtocolConfig, asset: AssetConfig) -> Result<(), Box<dyn std::error::Error>> {
    let client: RpcClient = RpcClient::new_with_commitment(user.rpc_url, CommitmentConfig::confirmed());
    let payer: Keypair = Keypair::from_base58_string(&user.private_key);
    println!("Using wallet: {}", payer.pubkey());

    let lending_market: Pubkey = protocol_config.lending_market;
    let fee_receiver: Pubkey = protocol_config.fee_receiver;
    let vault: Pubkey = protocol_config.vault;
    let program_id: Pubkey = protocol_config.program_id;

    let liquidity_mint: Pubkey = asset.liquidity_mint;
    let reserve_account: Pubkey = asset.reserve;
    let reserve_liquidity_supply: Pubkey = asset.liquidity_supply;
    let loan_amount: u64 = asset.loan_amount;

    if asset.is_sol == true {
        let user_token_account = spl_associated_token_account::get_associated_token_address(
            &payer.pubkey(), &liquidity_mint);

        let fee_receiver_ata = spl_associated_token_account::get_associated_token_address(
            &fee_receiver, &liquidity_mint);

        let mut instructions: Vec<Instruction> = Vec::new();

        instructions.push(create_associated_token_account_idempotent(
            &payer.pubkey(), &payer.pubkey(), &liquidity_mint, &spl_token::id()
        ));

        let extra_sol = 10_000_000; 

        instructions.push(
            system_instruction::transfer(
                &payer.pubkey(),
                &user_token_account,
                extra_sol,
            )
        );

        instructions.push(
            sync_native(
                &spl_token::id(),
                &user_token_account,
            )?
        );

        let mut borrow_data = Vec::with_capacity(9);
        borrow_data.push(19); // flash borrow opcode
        borrow_data.extend_from_slice(&loan_amount.to_le_bytes());
        let borrow_ix = Instruction {
            program_id,
            accounts: vec![
                solana_sdk::instruction::AccountMeta::new(reserve_liquidity_supply, false),
                solana_sdk::instruction::AccountMeta::new(user_token_account, false),
                solana_sdk::instruction::AccountMeta::new(reserve_account, false),
                solana_sdk::instruction::AccountMeta::new_readonly(lending_market, false),
                solana_sdk::instruction::AccountMeta::new_readonly(vault, false),
                solana_sdk::instruction::AccountMeta::new_readonly(solana_sdk::sysvar::instructions::id(), false),
                solana_sdk::instruction::AccountMeta::new_readonly(spl_token::id(), false),
            ],
            data: borrow_data,
        };
        instructions.push(borrow_ix);

        let mut repay_data = Vec::with_capacity(10);
        repay_data.push(20); 
        repay_data.extend_from_slice(&loan_amount.to_le_bytes());
        repay_data.push(3);  
        let repay_ix = Instruction {
            program_id,
            accounts: vec![
                solana_sdk::instruction::AccountMeta::new(user_token_account, false),
                solana_sdk::instruction::AccountMeta::new(reserve_liquidity_supply, false),
                solana_sdk::instruction::AccountMeta::new(fee_receiver_ata, false),
                solana_sdk::instruction::AccountMeta::new(user_token_account, false), 
                solana_sdk::instruction::AccountMeta::new(reserve_account, false),
                solana_sdk::instruction::AccountMeta::new_readonly(lending_market, false),
                solana_sdk::instruction::AccountMeta::new_readonly(payer.pubkey(), true),
                solana_sdk::instruction::AccountMeta::new_readonly(solana_sdk::sysvar::instructions::id(), false),
                solana_sdk::instruction::AccountMeta::new_readonly(spl_token::id(), false),
            ],
            data: repay_data,
        };
        instructions.push(repay_ix);


        // uncomment this if you wanna close the ata after usage
        /* 
        let close_ix = close_account(
            &spl_token::id(),
            &user_token_account,
            &payer.pubkey(),
            &payer.pubkey(),
            &[]
        )?;
        instructions.push(close_ix);
        */

        // build a transaction
        let recent_blockhash = client.get_latest_blockhash().await?;
        let transaction = Transaction::new_signed_with_payer(
            &instructions,
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        );

        println!("Simulating transaction...");
        let simulation = client.simulate_transaction(&transaction).await?;
        match simulation.value.err {
            Some(err) => {
                println!("Simulation failed with error: {:?}", err);
                match simulation.value.logs {
                    Some(ref logs) => {
                        println!("Program logs:");
                        for log in logs { println!("  {}", log); }
                    }
                    None => {}
                }
            }
            None => {
                println!("Simulation successful! Sending to network...");
                match client.send_and_confirm_transaction(&transaction).await {
                    Ok(sig) => println!("Success! Tx signature: https://solscan.io/tx/{}", sig),
                    Err(e)  => println!("Transaction failed: {:?}", e),
                }
            }
        }

        Ok(())
    } else {
        let user_token_account = get_associated_token_address(&payer.pubkey(), &liquidity_mint);

        let fee_receiver_ata = get_associated_token_address(&fee_receiver, &liquidity_mint);

        let mut borrow_data = Vec::with_capacity(9);
        borrow_data.push(19); 
        borrow_data.extend_from_slice(&loan_amount.to_le_bytes());

        let borrow_ix = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(reserve_liquidity_supply, false),
                AccountMeta::new(user_token_account, false),
                AccountMeta::new(reserve_account, false),
                AccountMeta::new_readonly(lending_market, false),
                AccountMeta::new_readonly(vault, false),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::id(), false),
                AccountMeta::new_readonly(spl_token::id(), false),
            ],
            data: borrow_data,
        };

        let mut repay_data = Vec::with_capacity(10);
        repay_data.push(20); 
        repay_data.extend_from_slice(&loan_amount.to_le_bytes()); 
        repay_data.push(0); 

        let repay_ix = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(user_token_account, false),
                AccountMeta::new(reserve_liquidity_supply, false),
                AccountMeta::new(fee_receiver_ata, false),
                AccountMeta::new(user_token_account, false), 
                AccountMeta::new(reserve_account, false),
                AccountMeta::new_readonly(lending_market, false),
                AccountMeta::new_readonly(payer.pubkey(), true),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::id(), false),
                AccountMeta::new_readonly(spl_token::id(), false),
            ],
            data: repay_data,
        };

        let instructions = vec![borrow_ix, repay_ix];
        let recent_blockhash = client.get_latest_blockhash().await?;
        let transaction = Transaction::new_signed_with_payer(
            &instructions,
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        );

        println!("simulating transaction...");
        let simulation = client.simulate_transaction(&transaction).await?;

        match simulation.value.err {
            Some(err) => {
                println!("simulation failed with error code: {:?}", err);
                if let Some(logs) = simulation.value.logs {
                    println!("Program logs:");
                    for log in logs {
                        println!("  {}", log);
                    }
                }
            }
            None => {
                println!("simulation successful! Sending to mainnet...");
                match client.send_and_confirm_transaction(&transaction).await {
                    Ok(signature) => println!("success! Tx hash: https://solscan.io/tx/{}", signature),
                    Err(err) => println!("sending error: {:?}", err),
                }
            }
        }
        Ok(())
    }
}
