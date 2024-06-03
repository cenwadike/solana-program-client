extern crate solana_program_client;

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

    let signers = &[&payer];
    // set up accounts
    let accounts = vec![
        AccountMeta::new(blob_account, false),
        AccountMeta::new(payer.pubkey(), true),
    ];

    let _tx_signature = signed_call(
        &connection,
        &program_id,
        &payer,
        signers,
        instruction_name,
        instruction_data,
        accounts,
    ).unwrap();
}
