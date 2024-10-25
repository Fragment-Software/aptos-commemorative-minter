use aptos_sdk::types::account_address::AccountAddress;
use std::{str::FromStr, sync::LazyLock};

pub const SECRETS_FILE_PATH: &str = "data/secrets.txt";
pub const PROXIES_FILE_PATH: &str = "data/proxies.txt";

pub static COLLECTION_ID: LazyLock<AccountAddress> = LazyLock::new(|| {
    AccountAddress::from_str("0xd42cd397c41a62eaf03e83ad0324ff6822178a3e40aa596c4b9930561d4753e5")
        .unwrap()
});

pub static MINTER_CONTRACT_ADDRESS: LazyLock<AccountAddress> = LazyLock::new(|| {
    AccountAddress::from_str("0x96c192a4e3c529f0f6b3567f1281676012ce65ba4bb0a9b20b46dec4e371cccd")
        .unwrap()
});

pub const TX_TIMEOUT: u64 = 10;

pub const APTOS_EXPLORER_URL: &str = "https://explorer.aptoslabs.com";
