
use std::str::FromStr;
use log::{debug, info, LevelFilter, warn};
use sui_keys::keystore::{AccountKeystore, InMemKeystore};
use sui_sdk::SuiClientBuilder;
use sui_types::base_types::{ObjectID, SuiAddress};
use sui_types::crypto::SignatureScheme;
use sui_types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use sui_types::transaction::{Transaction, TransactionData};
use std::env;
use std::time::SystemTime;
use sui_sdk::rpc_types::{SuiObjectDataOptions, SuiTransactionBlockResponseOptions, SuiTypeTag};
use sui_types::quorum_driver_types::ExecuteTransactionRequestType;
use shared_crypto::intent::Intent;
use sui_mm::order::OrderWrapper;
use sui_mm::user::{get_account_cap};

#[tokio::test]
async fn depth_test() {
    env_logger::builder().filter_level(LevelFilter::Debug).init();
    let mut keystore = InMemKeystore::default();
    let mnemonic = env::var("SUI_WALLET").expect("$SUI_WALLET is not set");
    let sui_rpc = env::var("SUI_RPC").expect("$SUI_RPC is not set");
    keystore.import_from_mnemonic(&*mnemonic,
                                  SignatureScheme::ED25519, None
    );
    let sender = keystore.addresses().first().unwrap().clone();
    let pool_id = ObjectID::from_str("0x4405b50d791fd3346754e8171aaab6bc2ed26c2c46efdd033c14b30ae507ac33").unwrap();
    let client = SuiClientBuilder::default()
        .build(sui_rpc)
        .await.unwrap();
    let response = get_account_cap(&client, &sender ).await.unwrap();
    let account_cap_id = response.data[0].data.clone().unwrap().object_id;
    let mut tb = ProgrammableTransactionBuilder::new();
    let order_wrapper = OrderWrapper::new(&client, pool_id, Some(account_cap_id), None ).await;
    let start = SystemTime::now();
    let (top_bids, top_asks) = order_wrapper.get_bid_ask().await;
    info!("bids: {:?}\n ", top_bids);
    info!("asks: {:?}\n", top_asks);
    info!("elapsed: {:?}\n", start.elapsed().unwrap().as_millis());
}