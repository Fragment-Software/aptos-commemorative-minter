use std::sync::Arc;

use crate::{
    config::Config, minter::process_accounts, parser::scrape_nft_balances,
    utils::files::read_private_keys,
};

use aptos_sdk::rest_client::Client;
use dialoguer::{theme::ColorfulTheme, Select};

const LOGO: &str = r#"
    ___                                                  __
  /'___\                                                /\ \__
 /\ \__/  _ __    __       __     ___ ___      __    ___\ \ ,_\
 \ \ ,__\/\`'__\/'__`\   /'_ `\ /' __` __`\  /'__`\/' _ `\ \ \/
  \ \ \_/\ \ \//\ \L\.\_/\ \L\ \/\ \/\ \/\ \/\  __//\ \/\ \ \ \_
   \ \_\  \ \_\\ \__/.\_\ \____ \ \_\ \_\ \_\ \____\ \_\ \_\ \__\
    \/_/   \/_/ \/__/\/_/\/___L\ \/_/\/_/\/_/\/____/\/_/\/_/\/__/
                  ___  __  /\____/
                /'___\/\ \_\_/__/
   ____    ___ /\ \__/\ \ ,_\ __  __  __     __    _ __    __
  /',__\  / __`\ \ ,__\\ \ \//\ \/\ \/\ \  /'__`\ /\`'__\/'__`\
 /\__, `\/\ \L\ \ \ \_/ \ \ \\ \ \_/ \_/ \/\ \L\.\\ \ \//\  __/
 \/\____/\ \____/\ \_\   \ \__\ \___x___/'\ \__/.\_\ \_\\ \____\
  \/___/  \/___/  \/_/    \/__/\/__//__/   \/__/\/_/\/_/ \/____/

                     t.me/fragment_software
"#;

pub async fn menu() -> eyre::Result<()> {
    println!("{LOGO}");

    loop {
        let config = Config::read_default().await;

        let provider = Arc::new(Client::new(config.rpc_url.parse()?));
        let accounts = read_private_keys().await;

        let options = vec![
            "Minter",
            "Check Aptos TWO Mainnet Anniversary 2024 NFT balance",
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Choice:")
            .items(&options)
            .default(0)
            .interact()
            .unwrap();

        match selection {
            0 => {
                process_accounts(accounts, config, provider).await;
            }
            1 => scrape_nft_balances(accounts).await,
            2 => {
                return Ok(());
            }
            _ => log::error!("Invalid selection"),
        }
    }
}
