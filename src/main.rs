mod flashloan;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    
    let rpc_url: String = std::env::var("RPC_URL").expect("RPC_URL does not determine in .env");
    let private_key: String = std::env::var("PRIVATE_KEY").expect("PRIVATE_KEY does not determine in .env");
    let loan_amount: u64 = 10_000_000; 

    match flashloan::execute_flash_loan(&rpc_url, &private_key, loan_amount).await {
        Ok(_) => {},
        Err(e) => eprintln!("process crashed with error: {:?}", e),
    }
    Ok(())
}
