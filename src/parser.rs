use std::collections::HashMap;

use aptos_sdk::types::LocalAccount;

use rand::{rngs::ThreadRng, thread_rng, Rng};
use reqwest::{Method, Proxy};
use serde::{Deserialize, Serialize};
use tabled::{settings::Style, Table, Tabled};
use tokio::task::JoinSet;

use crate::{
    constants::COLLECTION_ID,
    utils::{
        fetch::{send_http_request, RequestParams},
        files::read_proxies,
    },
};

#[derive(Serialize, Debug)]
struct IndexerRequestBody<'a> {
    query: &'a str,
    variables: Variables<'a>,
}

#[derive(Serialize, Debug)]
struct Variables<'a> {
    where_condition: WhereCondition<'a>,
    offset: i32,
    limit: i32,
    order_by: Vec<HashMap<&'a str, &'a str>>,
}

#[derive(Serialize, Debug)]
struct WhereCondition<'a> {
    owner_address: OwnerAddressCondition<'a>,
    amount: AmountCondition,
}

#[derive(Serialize, Debug)]
struct OwnerAddressCondition<'a> {
    #[serde(rename = "_eq")]
    eq: &'a str,
}

#[derive(Serialize, Debug)]
struct AmountCondition {
    #[serde(rename = "_gt")]
    gt: i32,
}

#[derive(Deserialize, Debug)]
struct IndexerResponseBody<T> {
    data: T,
}

#[derive(Deserialize, Debug)]
struct TokenOwnershipData {
    current_token_ownerships_v2: Vec<TokenData>,
}

#[allow(unused)]
#[derive(Deserialize, Debug)]
struct TokenData {
    token_data_id: String,
    current_token_data: CurrentTokenData,
}

#[derive(Deserialize, Debug)]
struct CurrentTokenData {
    collection_id: String,
}

impl<'a> IndexerRequestBody<'a> {
    const fn get_owned_tokens_query() -> &'static str {
        r#"query getOwnedTokens($where_condition: current_token_ownerships_v2_bool_exp!, $offset: Int, $limit: Int, $order_by: [current_token_ownerships_v2_order_by!]) {
  current_token_ownerships_v2(
    where: $where_condition
    offset: $offset
    limit: $limit
    order_by: $order_by
  ) {
    ...CurrentTokenOwnershipFields
  }
}

fragment CurrentTokenOwnershipFields on current_token_ownerships_v2 {
  token_standard
  token_properties_mutated_v1
  token_data_id
  table_type_v1
  storage_id
  property_version_v1
  owner_address
  last_transaction_version
  last_transaction_timestamp
  is_soulbound_v2
  is_fungible_v2
  amount
  current_token_data {
    collection_id
    description
    is_fungible_v2
    largest_property_version_v1
    last_transaction_timestamp
    last_transaction_version
    maximum
    supply
    token_data_id
    token_name
    token_properties
    token_standard
    token_uri
    current_collection {
      collection_id
      collection_name
      creator_address
      current_supply
      description
      last_transaction_timestamp
      last_transaction_version
      max_supply
      mutable_description
      mutable_uri
      table_handle_v1
      token_standard
      total_minted_v2
      uri
    }
  }
}"#
    }

    fn get_owned_tokens_body(address: &str) -> IndexerRequestBody {
        let where_condition = WhereCondition {
            owner_address: OwnerAddressCondition { eq: address },
            amount: AmountCondition { gt: 0 },
        };

        let vars = Variables {
            where_condition,
            offset: 0,
            limit: 100,
            order_by: vec![
                [("last_transaction_version", "desc")].into_iter().collect(),
                [("token_data_id", "desc")].into_iter().collect(),
            ],
        };

        IndexerRequestBody {
            query: Self::get_owned_tokens_query(),
            variables: vars,
        }
    }
}

type OwnedTokensData = IndexerResponseBody<TokenOwnershipData>;

async fn get_owned_tokens(address: &str, proxy: Option<Proxy>) -> eyre::Result<OwnedTokensData> {
    let body = IndexerRequestBody::get_owned_tokens_body(address);

    let request_params = RequestParams {
        url: "https://api.mainnet.aptoslabs.com/v1/graphql/",
        method: Method::POST,
        body: Some(body),
        query_args: None,
        proxy,
        headers: None,
    };

    let response_body = send_http_request::<OwnedTokensData>(request_params).await?;

    Ok(response_body)
}

async fn get_nft_balance(address: &str, proxy: Option<Proxy>) -> eyre::Result<usize> {
    let response = get_owned_tokens(address, proxy).await?;

    let nft_count = response
        .data
        .current_token_ownerships_v2
        .iter()
        .filter(|data| data.current_token_data.collection_id == COLLECTION_ID.to_string())
        .count();

    Ok(nft_count)
}

pub async fn scrape_nft_balances(accounts: Vec<LocalAccount>) {
    let mut rng = thread_rng();

    let addresses = accounts
        .iter()
        .map(|account| account.address().to_string())
        .collect::<Vec<_>>();

    let proxies = read_proxies().await;

    let spawn_task = |handles: &mut JoinSet<_>, proxy: Option<Proxy>, address: String| {
        handles.spawn(async move {
            let balance = get_nft_balance(&address, proxy).await;
            (balance, address)
        })
    };

    let get_random_proxy = |proxies: &[Proxy], rng: &mut ThreadRng| {
        if proxies.is_empty() {
            return None;
        }

        let proxy_index = rng.gen_range(0..proxies.len());
        Some(proxies[proxy_index].clone())
    };

    let mut handles = JoinSet::new();

    for address in addresses {
        let proxy = get_random_proxy(&proxies, &mut rng);

        spawn_task(&mut handles, proxy, address);
    }

    let mut entries = vec![];

    while let Some(res) = handles.join_next().await {
        let (balance, address) = res.unwrap();
        match balance {
            Ok(balance) => {
                entries.push(Entry { address, balance });
            }
            Err(e) => {
                log::error!("Failed to get NFT balance for wallet `{address}`: {e}");
                let proxy = get_random_proxy(&proxies, &mut rng);
                spawn_task(&mut handles, proxy, address);
            }
        }
    }

    let mut table = Table::new(&entries);
    let table = table.with(Style::modern_rounded());

    println!("{table}");
}

#[derive(Tabled, Debug)]
struct Entry {
    #[tabled(rename = "Address")]
    address: String,
    #[tabled(rename = "NFT balance")]
    balance: usize,
}
