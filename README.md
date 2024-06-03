# Solana program client 

`solana_program_client` is a lightweight crate to call Solana programs. It focuses on abstracting away relevant crates used for interacting with Solana programs.

# Overview

`solana_program_client` is a crate for developing clients for Solana programs and client-side web applications using Rust. The primary goal behind this project is to provide an ergonomic interface for interacting with Solana programs.

This in turn allows developers to use call program functions using simple RPC requests.

An example is available here:

```rust
use solana_program_client::legacy_tx::*;

#[derive(BorshSerialize, BorshDeserialize)]
#[borsh(crate = "borsh")]
pub struct UpdateBlob {
    pub data: Vec<u8>,
}

fn main() {
    // create a Rpc client connection
    let connection = RpcClient::new("https://api.devnet.solana.com");
    let program_id = blob::ID;

    // get blob PDA
    let (blob_account, _) = Pubkey::find_program_address(&[&b"blob"[..]], &program_id);

    let payer = Keypair::read_from_file("~/.config/solana/id.json").unwrap();

    let instruction_name = "update_blob";

    //  construct instruction data
    let instruction_data = UpdateBlob {
        data: "data".as_bytes().to_vec(),
    };

    // set up accounts
    let accounts = vec![
        AccountMeta::new(blob_account, false),
        AccountMeta::new(payer.pubkey(), true),
    ];

    // call signed call
    let _tx_signature = signed_call(
        connection,
        program_id,
        payer,
        instruction_name,
        instruction_data,
        accounts,
    ).unwrap();
}
```

# Features

- Submit signed call to Solana program
- Create a lookup table
- Extend a lookup table
- Submit versioned transaction


# Motivation

- Inability to submit transactions through Rust Solana client without knowing the function discriminant.
- Repeated code duplication when building Rust Solana client.

# Development status

The crate is currently under development and should be considered in the alpha stage. Additional work is needed on further extensions.
# TODO

- unsigned call
- event subscription

# Tip:

solana: Fj72ApTUaYEwC3RKCKQ7iX3s8i8CVAnZW1f9PAXSKtbY