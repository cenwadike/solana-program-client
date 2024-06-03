extern crate solana_program_client;

use solana_program_client::versioned_tx::*;

#[derive(BorshSerialize, BorshDeserialize)]
#[borsh(crate = "borsh")]
pub struct UpdateBlob {
    pub data: Vec<u8>,
}

fn main() {
    let connection = RpcClient::new("https://api.devnet.solana.com");
    let program_id = blob::ID;
    let instruction_name = "update_blob";
    let instruction_data = UpdateBlob {
        data: "another data".as_bytes().to_vec(),
    };
    let payer: Keypair = Keypair::read_from_file("~/.config/solana/id.json").unwrap();

    // create lookup table
    let latest_blockhash = connection
        .get_latest_blockhash()
        .expect("latest block hash");
    let table_pk = create_lookup_table(&connection, &payer, latest_blockhash).unwrap();

    // add accounts to lookup table
    let new_accounts = vec![program_id, payer.pubkey()];
    update_lookup_table(
        &connection,
        &payer,
        latest_blockhash,
        table_pk,
        new_accounts,
    )
    .unwrap();

    // set up accounts
    let (blob_account, _) = Pubkey::find_program_address(&[&b"blob"[..]], &program_id);
    let accounts = vec![
        AccountMeta::new(blob_account, false),
        AccountMeta::new(payer.pubkey(), true),
    ];

    // call program with lookup table
    let _tx_signature = call_with_lookup_table(
        connection,
        program_id,
        instruction_name,
        instruction_data,
        &table_pk,
        &payer,
        accounts,
    ).unwrap();
}
