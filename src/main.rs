pub mod user;
pub mod market;
pub mod utils;
pub mod constant;
pub mod order;
pub mod pyth;

use std::str::FromStr;
use log::{debug, LevelFilter};
use sui_keys::keystore::{AccountKeystore, InMemKeystore};
use sui_sdk::{SuiClient, SuiClientBuilder};
use sui_types::base_types::{ObjectID, SuiAddress};
use sui_types::crypto::SignatureScheme;
use sui_types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use sui_types::transaction::TransactionKind;
use sui_types::TypeTag;
use std::env;
use crate::constant::{SUI_PRICE_FEED, USDC_PRICE_FEED};
use crate::market::{get_fills, get_market_price};
use crate::pyth::PythFeeder;
use crate::user::{get_account_balance, get_account_cap, parse_result_account_balance};
use crate::utils::parse_result_u64;

#[tokio::main]
async fn main() {
    env_logger::builder().filter_level(LevelFilter::Debug).init();
    let mut keystore = InMemKeystore::default();
    let mnemonic = env::var("SUI_WALLET").expect("$SUI_WALLET is not set");
    let sui_rpc = env::var("SUI_RPC").expect("$SUI_RPC is not set");
    keystore.import_from_mnemonic(&*mnemonic,
                                  SignatureScheme::ED25519, None
    );
    let sender = keystore.addresses().first().unwrap().clone();
    let pool_id = ObjectID::from_str("0x4405b50d791fd3346754e8171aaab6bc2ed26c2c46efdd033c14b30ae507ac33").unwrap();
    let sui_tag = TypeTag::from_str("0x2::sui::SUI").unwrap();
    let usdc_tag = TypeTag::from_str("0x5d4b302506645c37ff133b98c4b50a5ae14841659738d6d733d59d0d217a93bf::coin::COIN").unwrap();
    let client = SuiClientBuilder::default()
        .build(sui_rpc)
        .await.unwrap();
    let response = get_account_cap(&client, &sender ).await.unwrap();
    let object_id = response.data[0].data.clone().unwrap().object_id;
    debug!("{:?}", object_id );
    let mut tb = ProgrammableTransactionBuilder::new();
    tb = get_account_balance(tb, sui_tag.clone(), usdc_tag.clone(), pool_id, object_id);
    tb = get_market_price(tb, sui_tag, usdc_tag, pool_id);
    let result = client.read_api().dev_inspect_transaction_block(SuiAddress::ZERO, TransactionKind::ProgrammableTransaction(tb.finish()), None, None, None).await;
    let execution_result =  result.unwrap().results.unwrap();
    let item = execution_result.first().unwrap();
    let results = parse_result_u64(item, 0);
    debug!("{:?}", results);
    debug!("{:?}", parse_result_u64(&execution_result[1], 1));
    // 181.94.248.117
    get_fills(&client).await;
    let feeder = PythFeeder::new(vec![SUI_PRICE_FEED.to_string(), USDC_PRICE_FEED.to_string()]);
    let result = feeder.get_latest_price().await;
    debug!("{:?}", result.unwrap());
}

