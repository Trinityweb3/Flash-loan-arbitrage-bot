# Solana Arbitrage Bot

It's a mainnet-ready flash loan execution script written in Rust which use Save Finance, Meteora DLMM Programm and Orca Finance. Also using Jito Bundles

## TxID<br>
### https://solscan.io/tx/fyiBAa1hJbBHNh1A55BLoL79EU22nBB9C9NEG5Ds1tpStg9CobvGNYimso6ma9CS1VHydrXfufn2Le6gWvkHva6

## Setup

First, do:
```git clone https://github.com/Trinityweb3/Flash-loan-arbitrage-bot```

Second, create a .env file in the your project root.

RPC_URL=https://helius-rpc.com/xxxxxxx<br>
PRIVATE_KEY=xxxxx(base58)<br>
ALT_ADDRESS=xxx (Address lookup tables. First create it using solana-cli and command: <pre>solana address-lookup-table create</pre> )<br>

And compile & start a script using ```cargo run``` 

## Customising

All protocol addresses are taken from the official Save Finance Docs. 
They're there - https://docs.save.finance/architecture/addresses/mainnet/main-pools
  
## Returning 20% of fees

Actually, Save Finance charges a 0.05% total fee from flash loans. But the protocol architecture is designed to reward front-end integrators like a host fee mechanism
So, a total fee spliting on<br>
1 - 80% goes directly to the protocol treasury (fee_receiver_ata).<br>
2 - 20% goes to the integrator (host_fee_receiver).

## How this code exploits it:
In the repay_ix accounts vector, we pass our own wallet's ATA as the host_fee_receiver. Thus, by acting as our own host, we automatically save 20% of the loan costs.<br>

## WARNING!
The code doesn't take into the complex mathematics of Meteora DLMM bins calculating. So, code doesnt take into slippage influence for large volumes during swap, make sure that you understand this

Please, tips me if this repo was helpful<br>
HogXprfJCbNSWRXeU3vkt9mM2AcXHrqnLVc6WP1wpQrS
## Created by [@trinitycult](https://t.me/trinitycult)
