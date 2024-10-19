use std::sync::Arc;

use aptos_sdk::rest_client::Client;
use config::Config;
use minter::process_accounts;
use utils::read_private_keys;

mod config;
mod constants;
mod minter;
mod utils;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    pretty_env_logger::formatted_timed_builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let config = Config::read_default().await;

    let provider = Arc::new(Client::new(config.rpc_url.parse()?));
    let accounts = read_private_keys().await;

    process_accounts(accounts, config, provider).await;

    Ok(())
}
