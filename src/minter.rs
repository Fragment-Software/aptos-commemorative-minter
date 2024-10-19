use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use aptos_sdk::{
    bcs,
    move_types::{ident_str, language_storage::ModuleId},
    rest_client::Client,
    transaction_builder::TransactionBuilder,
    types::{
        chain_id::ChainId,
        transaction::{EntryFunction, SignedTransaction, TransactionPayload},
        LocalAccount,
    },
};
use rand::{thread_rng, Rng};

use crate::{
    config::Config,
    constants::{APTOS_EXPLORER_URL, COLLECTION_ID, MINTER_CONTRACT_ADDRESS, TX_TIMEOUT},
    utils::{pretty_sleep, random_in_range},
};

async fn get_account_seq_number(
    account: &LocalAccount,
    provider: Arc<Client>,
) -> eyre::Result<u64> {
    let account = provider.get_account(account.address()).await?;
    Ok(account.inner().sequence_number)
}

fn assemble_and_sign_mint_tx(
    account: &LocalAccount,
    seq_number: u64,
    quantity: u64,
) -> SignedTransaction {
    let args = vec![
        bcs::to_bytes(&*COLLECTION_ID).unwrap(),
        bcs::to_bytes(&Some(quantity)).unwrap(),
    ];

    let payload = TransactionPayload::EntryFunction(EntryFunction::new(
        ModuleId::new(
            *MINTER_CONTRACT_ADDRESS,
            ident_str!("unmanaged_launchpad").to_owned(),
        ),
        ident_str!("mint").to_owned(),
        vec![],
        args,
    ));

    let timeout = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + TX_TIMEOUT;

    let raw_transaction = TransactionBuilder::new(payload, timeout, ChainId::mainnet())
        .sender(account.address())
        .sequence_number(seq_number)
        .max_gas_amount(5000)
        .gas_unit_price(100)
        .build();

    account.sign_transaction(raw_transaction)
}

pub async fn process_accounts(
    mut accounts: Vec<LocalAccount>,
    config: Config,
    provider: Arc<Client>,
) {
    let mut rng = thread_rng();

    while !accounts.is_empty() {
        let index = rng.gen_range(0..accounts.len());
        let account = &accounts[index];

        let seq_number = get_account_seq_number(account, provider.clone()).await;

        if let Ok(seq_number) = seq_number {
            let quantity = random_in_range(config.mint_quantity_range);
            let signed_transaction = assemble_and_sign_mint_tx(account, seq_number, quantity);

            log::info!("Account: {}. Quantity: {}", account.address(), quantity);

            match provider.submit(&signed_transaction).await {
                Ok(receipt) => {
                    accounts.remove(index);

                    let tx_hash = receipt.inner().hash;
                    log::info!("Transaction sent: {}/txn/{}", APTOS_EXPLORER_URL, tx_hash);

                    pretty_sleep(config.wallet_delay_range).await;
                }
                Err(e) => log::error!("Failed to send trasnaction: {e}"),
            }
        }
    }
}
