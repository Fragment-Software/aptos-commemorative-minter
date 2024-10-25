use std::path::Path;

use aptos_sdk::types::LocalAccount;
use reqwest::Proxy;
use tokio::io::AsyncBufReadExt;

use crate::constants::{PROXIES_FILE_PATH, SECRETS_FILE_PATH};

async fn read_file_lines(path: impl AsRef<Path>) -> eyre::Result<Vec<String>> {
    let file = tokio::fs::read(path).await?;
    let mut lines = file.lines();

    let mut contents = vec![];
    while let Some(line) = lines.next_line().await? {
        contents.push(line);
    }

    Ok(contents)
}

pub async fn read_proxies() -> Vec<Proxy> {
    read_file_lines(PROXIES_FILE_PATH)
        .await
        .expect("Proxies file to be valid")
        .iter()
        .map(|proxy| Proxy::all(proxy).expect("Proxy format to be valid"))
        .collect()
}

pub async fn read_private_keys() -> Vec<LocalAccount> {
    read_file_lines(SECRETS_FILE_PATH)
        .await
        .expect("Secrets file to be valid")
        .iter()
        .map(|secret| {
            LocalAccount::from_private_key(secret, 0)
                .or_else(|_| LocalAccount::from_derive_path("m/44'/637'/0'/0'/0'", secret, 0))
                .unwrap_or_else(|_| panic!("Failed to create LocalAccount from private key or derivation path for key: {}",
                    secret))
        })
        .collect()
}
