use anchor_client;
#[allow(unused_imports)]
pub use borsh::{BorshDeserialize, BorshSerialize};
pub use solana_client::rpc_client::RpcClient;
#[allow(unused_imports)]
pub use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::{AccountMeta, Instruction},
    message::Message,
    pubkey::Pubkey,
    signature::Signature,
    signature::{Keypair, Signer},
    signer::EncodableKey,
    system_program,
    transaction::Transaction,
};

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
/// use solana_program_client::legacy_tx::*;
/// #[derive(BorshSerialize, BorshDeserialize)]
/// #[borsh(crate = "borsh")]
/// pub struct UpdateBlob {
///     pub data: Vec<u8>,
/// }
///
/// fn main() {
///     // create a Rpc client connection
///     let connection = RpcClient::new("https://api.devnet.solana.com");
///     let program_id = blob::ID;

///     // get blob PDA
///     let (blob_account, _) = Pubkey::find_program_address(&[&b"blob"[..]], &program_id);

///     let payer = Keypair::read_from_file("~/.config/solana/id.json").unwrap();

///     let instruction_name = "update_blob";

///     //  construct instruction data
///     let instruction_data = UpdateBlob {
///         data: "data".as_bytes().to_vec(),
///     };

///     // setup signers
///     let signers = &[&payer];
///
///     // set up accounts
///     let accounts = vec![
///         AccountMeta::new(blob_account, false),
///         AccountMeta::new(payer.pubkey(), true),
///     ];

///     // call signed call
///     let _tx_signature = signed_call(
///         &connection,
///         &program_id,
///         &payer,
///         signers,
///         instruction_name,
///         instruction_data,
///         accounts,
///     )
///     .unwrap();
/// }
/// ```
pub fn signed_call<T>(
    connection: &RpcClient,
    program_id: &Pubkey,
    payer: &Keypair,
    signers: &[&Keypair],
    instruction_name: &str,
    instruction_data: T,
    accounts: Vec<AccountMeta>,
) -> Result<Signature, Box<dyn std::error::Error>>
where
    T: BorshSerialize,
{
    // get discriminant
    let instruction_discriminant = get_discriminant("global", instruction_name);

    // construct instruction
    let ix = Instruction::new_with_borsh(
        program_id.clone(),
        &(instruction_discriminant, instruction_data),
        accounts.clone(),
    );

    // get latest block hash
    let blockhash = connection.get_latest_blockhash()?;

    // construct message
    let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);

    //construct transaction
    let mut tx = Transaction::new_unsigned(msg);

    // sign transaction
    tx.sign(signers, tx.message.recent_blockhash);

    // send and confirm transaction
    let tx_signature = connection.send_and_confirm_transaction(&tx)?;

    Ok(tx_signature)
}

/// returns function signature
///
/// accepts name space and name function
pub fn get_discriminant(namespace: &str, name: &str) -> [u8; 8] {
    let preimage = format!("{}:{}", namespace, name);

    let mut sighash = [0u8; 8];
    sighash.copy_from_slice(
        &anchor_client::anchor_lang::solana_program::hash::hash(preimage.as_bytes()).to_bytes()
            [..8],
    );
    sighash
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(BorshSerialize, BorshDeserialize)]
    #[borsh(crate = "borsh")]
    pub struct UpdateBlob {
        pub data: Vec<u8>,
    }

    #[test]
    fn test_signed_call() {
        let connection = RpcClient::new("https://api.devnet.solana.com");
        let program_id = blob::ID;

        let (blob_account, _) = Pubkey::find_program_address(&[&b"blob"[..]], &program_id);

        let payer =
            Keypair::read_from_file("/Users/cenwadike/.config/solana/solfate-dev.json").unwrap();

        let instruction_name = "update_blob";

        //  construct instruction data
        let instruction_data = UpdateBlob {
            data: "another data".as_bytes().to_vec(),
        };

        let signers = &[&payer];
        // set up accounts
        let accounts = vec![
            AccountMeta::new(blob_account, false),
            AccountMeta::new(payer.pubkey(), true),
        ];

        let tx_signature = signed_call(
            &connection,
            &program_id,
            &payer,
            signers,
            instruction_name,
            instruction_data,
            accounts,
        );

        assert!(tx_signature.is_ok());
    }
}
