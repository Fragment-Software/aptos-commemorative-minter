mod config;
mod constants;
mod menu;
mod minter;
mod parser;
mod utils;

use menu::menu;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    pretty_env_logger::formatted_timed_builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    if let Err(e) = menu().await {
        log::error!("Execution stopped with error: {e}");
    }

    Ok(())
}
