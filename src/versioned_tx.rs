use std::str::FromStr;

use base64::{engine::general_purpose, Engine as _};
#[allow(unused_imports)]
pub use borsh::{BorshDeserialize, BorshSerialize};
pub use solana_address_lookup_table_program::state::AddressLookupTable;
pub use solana_client::{
    rpc_client::RpcClient, rpc_config::RpcSendTransactionConfig, rpc_request::RpcRequest,
};
pub use solana_sdk::instruction::AccountMeta;
#[allow(unused_imports)]
pub use solana_sdk::{
    address_lookup_table::AddressLookupTableAccount,
    commitment_config::{CommitmentConfig, CommitmentLevel},
    instruction::Instruction,
    message::VersionedMessage,
    pubkey::Pubkey,
    signature::Signature,
    signature::{Keypair, Signer},
    signer::EncodableKey,
    transaction::{Transaction, VersionedTransaction},
};
pub use solana_transaction_status::UiTransactionEncoding;

use crate::legacy_tx::get_discriminant;

/// Sign and submit a legacy transaction.
///
/// This method fully signs a transaction with all required signers, which
/// must be present in the `keypairs` slice.
///
/// # Panics
///
/// Panics when signing or signature verification fails.
///
/// # Examples
///
/// This example uses the [`solana_program_client`] crate.
///
/// ```
/// use solana_program_client::versioned_tx::*;

/// #[derive(BorshSerialize, BorshDeserialize)]
/// #[borsh(crate = "borsh")]
/// pub struct UpdateBlob {
///     pub data: Vec<u8>,
/// }

/// fn call_with_lookup_table() {
///     let connection = RpcClient::new("https://api.devnet.solana.com");
///     let program_id = blob::ID;
///     let instruction_name = "update_blob";
///     let instruction_data = UpdateBlob {
///         data: "another data".as_bytes().to_vec(),
///     };
///     let payer: Keypair = Keypair::read_from_file("~/.config/solana/id.json").unwrap();
///     let signers = &[&payer];
///
///     // create lookup table
///     let latest_blockhash = connection
///         .get_latest_blockhash()
///         .expect("latest block hash");
///     let table_pk = create_lookup_table(&connection, &payer, latest_blockhash).unwrap();

///     // add accounts to lookup table
///     let new_accounts = vec![program_id, payer.pubkey()];
///     update_lookup_table(
///         &connection,
///         &payer,
///         latest_blockhash,
///         table_pk,
///         new_accounts,
///     )
///     .unwrap();

///     // set up accounts
///     let (blob_account, _) = Pubkey::find_program_address(&[&b"blob"[..]], &program_id);
///     let accounts = vec![
///         AccountMeta::new(blob_account, false),
///         AccountMeta::new(payer.pubkey(), true),
///     ];

///     // call program with lookup table
///     let _tx_signature = call_with_lookup_table(
///         connection,
///         program_id,
///         instruction_name,
///         instruction_data,
///         &table_pk,
///         &payer,
///         signers,
///         accounts,
///     )
///     .unwrap();
/// }
/// ```
pub fn call_with_lookup_table<T>(
    connection: &RpcClient,
    program_id: &Pubkey,
    instruction_name: &str,
    instruction_data: T,
    lookup_table_key: &Pubkey,
    payer: &Keypair,
    signers: &[&Keypair],
    accounts: Vec<AccountMeta>,
) -> Result<Signature, Box<dyn std::error::Error>>
where
    T: BorshSerialize,
{
    // get lookup table addresses from lookup table key
    let lookup_table_account = connection.get_account(lookup_table_key)?;
    let address_lookup_table = AddressLookupTable::deserialize(&lookup_table_account.data)?;
    let address_lookup_table_account = AddressLookupTableAccount {
        key: lookup_table_key.clone(),
        addresses: address_lookup_table.addresses.to_vec(),
    };

    // construct instruction
    let instruction_discriminant = get_discriminant("global", instruction_name);
    let ix = Instruction::new_with_borsh(
        program_id.clone(),
        &(instruction_discriminant, instruction_data),
        accounts,
    );

    // create versioned transaction with lookup table
    let blockhash = connection.get_latest_blockhash()?;
    let tx = VersionedTransaction::try_new(
        VersionedMessage::V0(solana_sdk::message::v0::Message::try_compile(
            &payer.pubkey(),
            &[ix],
            &[address_lookup_table_account],
            blockhash,
        )?),
        signers,
    )?;

    // serialize and encode transaction
    let serialized_tx = bincode::serialize(&tx)?;
    let serialized_encoded_tx = general_purpose::STANDARD.encode(serialized_tx);

    // construct transaction pre-execution configuration
    let config = RpcSendTransactionConfig {
        skip_preflight: false,
        preflight_commitment: Some(CommitmentLevel::Confirmed),
        encoding: Some(UiTransactionEncoding::Base64),
        ..RpcSendTransactionConfig::default()
    };

    // submit transaction and retrieve transaction signature
    let signature = connection.send::<String>(
        RpcRequest::SendTransaction,
        serde_json::json!([serialized_encoded_tx, config]),
    )?;

    // verify transaction execution
    connection.confirm_transaction_with_commitment(
        &Signature::from_str(signature.as_str())?,
        CommitmentConfig::finalized(),
    )?;

    Ok(Signature::from_str(&signature)?)
}

/// create a lookup table with an authority account.
///
/// This method submit transaction that creates a
/// lookup table. Returns lookup table account public key.
///
/// # Panics
///
/// Panics when signature verification fails.
///
/// # Examples
///
/// This example uses the [`solana_program_client`] crate.
///
/// ```
/// use solana_program_client::versioned_tx::*;
///
/// fn create_lookup_table() {
///     let connection = RpcClient::new("https://api.devnet.solana.com");
///     let payer: Keypair =
///         Keypair::read_from_file("/Users/cenwadike/.config/solana/solfate-dev.json").unwrap();

///     let latest_blockhash = connection
///         .get_latest_blockhash()
///         .expect("latest block hash");

///     let lookup_table_pk = create_lookup_table(&connection, &payer, latest_blockhash);
/// }
/// ```
pub fn create_lookup_table(
    connection: &RpcClient,
    payer: &Keypair,
    latest_blockhash: solana_sdk::hash::Hash,
) -> Result<Pubkey, Box<dyn std::error::Error>> {
    let recent_slot = connection.get_slot()?;
    let (create_ix, table_pk) =
        solana_address_lookup_table_program::instruction::create_lookup_table(
            payer.pubkey(),
            payer.pubkey(),
            recent_slot,
        );

    connection.send_and_confirm_transaction(&Transaction::new_signed_with_payer(
        &[create_ix],
        Some(&payer.pubkey()),
        &[&payer],
        latest_blockhash,
    ))?;

    Ok(table_pk)
}

/// extend a lookup table.
///
/// This method submit transaction that extends a
/// lookup table. Returns lookup table account public key.
///
/// # Panics
///
/// Panics when signature verification fails.
///
/// # Examples
///
/// This example uses the [`solana_program_client`] crate.
///
/// ```
/// use solana_program_client::versioned_tx::*;
///
/// fn test_update_lookup_table() {
///     let connection = RpcClient::new("https://api.devnet.solana.com");
///     let payer: Keypair =
///         Keypair::read_from_file("/Users/cenwadike/.config/solana/solfate-dev.json").unwrap();

///     let latest_blockhash = connection
///         .get_latest_blockhash()
///         .expect("latest block hash");

///     let table_pk = create_lookup_table(&connection, &payer, latest_blockhash).unwrap();
///     let new_accounts = vec![Pubkey::new_unique()];
///     let res = extend_lookup_table(
///         &connection,
///         &payer,
///         latest_blockhash,
///         table_pk,
///         new_accounts,
///     )
///     .unwrap();
/// }  
/// ```
pub fn extend_lookup_table(
    connection: &RpcClient,
    payer: &Keypair,
    latest_blockhash: solana_sdk::hash::Hash,
    table_pk: Pubkey,
    new_accounts: Vec<Pubkey>,
) -> Result<bool, Box<dyn std::error::Error>> {
    // add accounts to look up table
    let extend_ix = solana_address_lookup_table_program::instruction::extend_lookup_table(
        table_pk,
        payer.pubkey(),
        Some(payer.pubkey()),
        new_accounts,
    );

    let signature =
        connection.send_and_confirm_transaction(&Transaction::new_signed_with_payer(
            &[extend_ix],
            Some(&payer.pubkey()),
            &[&payer],
            latest_blockhash,
        ))?;

    Ok(connection
        .confirm_transaction_with_spinner(
            &signature,
            &latest_blockhash,
            CommitmentConfig::confirmed(),
        )
        .is_ok())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_create_lookup_table() {
        let connection = RpcClient::new("https://api.devnet.solana.com");
        let payer: Keypair =
            Keypair::read_from_file("/Users/cenwadike/.config/solana/solfate-dev.json").unwrap();

        let latest_blockhash = connection
            .get_latest_blockhash()
            .expect("latest block hash");

        let lookup_table_pk = create_lookup_table(&connection, &payer, latest_blockhash);
        assert!(lookup_table_pk.is_ok())
    }

    #[test]
    fn test_update_lookup_table() {
        let connection = RpcClient::new("https://api.devnet.solana.com");
        let payer: Keypair =
            Keypair::read_from_file("/Users/cenwadike/.config/solana/solfate-dev.json").unwrap();

        let latest_blockhash = connection
            .get_latest_blockhash()
            .expect("latest block hash");

        let table_pk = create_lookup_table(&connection, &payer, latest_blockhash).unwrap();
        let new_accounts = vec![Pubkey::new_unique()];
        let res = extend_lookup_table(
            &connection,
            &payer,
            latest_blockhash,
            table_pk,
            new_accounts,
        )
        .unwrap();
        assert!(res);
    }

    #[derive(BorshSerialize, BorshDeserialize)]
    pub struct UpdateBlob {
        pub data: Vec<u8>,
    }

    #[test]
    fn test_call_with_lookup_table() {
        let connection = RpcClient::new("https://api.devnet.solana.com");
        let program_id = blob::ID;
        let instruction_name = "update_blob";
        let instruction_data = UpdateBlob {
            data: "another data".as_bytes().to_vec(),
        };
        let payer: Keypair =
            Keypair::read_from_file("/Users/cenwadike/.config/solana/solfate-dev.json").unwrap();

        let signers = &[&payer];
        // create lookup table
        let latest_blockhash = connection
            .get_latest_blockhash()
            .expect("latest block hash");
        let table_pk = create_lookup_table(&connection, &payer, latest_blockhash).unwrap();

        // add accounts to lookup table
        let new_accounts = vec![program_id, payer.pubkey()];
        extend_lookup_table(
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

        let res = call_with_lookup_table(
            &connection,
            &program_id,
            instruction_name,
            instruction_data,
            &table_pk,
            &payer,
            signers,
            accounts,
        );

        assert!(res.is_ok());
    }
}
