# Solana Flash Loan

It's a mainnet-ready flash loan execution script written in Rust.

## Proof of work (TxID)
https://solscan.io/tx/3TeBp5d6FkY3fvWCiG46CTtNrY9cWXNYEdqS56JmLcJhq714e6t3gvR6XdX8pGp4oitW2LvffSQ8jZeoa3Pp33SJ

## Setup

First, do:
```git clone https://github.com/Trinityweb3/Flash-Loan-using-Save-formerly-Solend-```

Second, create a .env file in the your project root.

RPC_URL=https://helius-rpc.com/xxxxxxx<br>
PRIVATE_KEY=xxxxx(base58)

And compile & start a script using ```cargo run``` 

## Customising

All protocol addresses are taken from the official Save Finance Docs. 
They're there - https://docs.save.finance/architecture/addresses/mainnet/main-pools

If you want to switch from USDC to SOL or USDT, you just need to swap the hardcoded Pubkeys constants in src/main.rs

!!! Notice: If you borrowing SOL you must remember that $SOL has 9 decimals instead of USDC's 6!!!
  
## Returning 20% of fees

Actually, Save Finance charges a 0.05% total fee from flash loans. But the protocol architecture is designed to reward front-end integrators like a host fee mechanism
So, a total fee spliting on
1 -  80% goes directly to the protocol treasury (fee_receiver_ata).
2 - 20% goes to the integrator (host_fee_receiver).

## How this code exploits it:
In the repay_ix accounts vector, we pass our own wallet's ATA as the host_fee_receiver:

By acting as our own host, we automatically save 20% of the loan costs. Thus 20% of it is ref back to our wallet

Please, tips me =)
DdKQPiHdhgCACMvPe1UKH9aJ8KY415YW9eNC7ah6nzS7

## Created by [@trinitycult](https://t.me/trinitycult)
