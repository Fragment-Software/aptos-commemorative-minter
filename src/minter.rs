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
        transaction::{EntryFunction, ExecutionStatus, SignedTransaction, TransactionPayload},
        LocalAccount,
    },
};
use log::error;
use rand::{thread_rng, Rng};

use crate::{
    config::Config,
    constants::{APTOS_EXPLORER_URL, COLLECTION_ID, MINTER_CONTRACT_ADDRESS, TX_TIMEOUT},
    utils::pretty_sleep,
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

async fn mint_nft(account: &LocalAccount, provider: Arc<Client>) -> eyre::Result<bool> {
    let seq_number = get_account_seq_number(account, provider.clone()).await?;

    let signed_transaction = assemble_and_sign_mint_tx(account, seq_number, 1);

    let simulation_result = provider
        .simulate_bcs_with_gas_estimation(&signed_transaction, true, true)
        .await?;

    if let ExecutionStatus::MoveAbort { info, .. } = simulation_result.inner().info.status() {
        match info {
            Some(ref info) if info.reason_name == "EINSUFFICIENT_MAX_PER_USER_BALANCE" => {
                log::warn!("{} has already minted the NFT", account.address());
                return Ok(true);
            }
            _ => {}
        }
    }

    log::info!("Account: {}. Minting an NFT", account.address());

    let pending_transaction = provider.submit(&signed_transaction).await?;

    let confirmed_transaction = provider
        .wait_for_transaction(pending_transaction.inner())
        .await?;

    if confirmed_transaction.inner().success() {
        let tx_hash = pending_transaction.inner().hash;
        log::info!(
            "Transaction confirmed: {}/txn/{}",
            APTOS_EXPLORER_URL,
            tx_hash
        );
        return Ok(true);
    }

    Ok(false)
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

        match mint_nft(account, provider.clone()).await {
            Ok(res) => {
                if res {
                    accounts.remove(index);
                    pretty_sleep(config.wallet_delay_range).await;
                }
            }
            Err(e) => {
                error!("`{}` mint failed: {e}", account.address())
            }
        }
    }
}
