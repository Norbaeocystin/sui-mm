pub mod user;
pub mod market;
pub mod utils;
pub mod constant;
pub mod order;
pub mod pyth;
pub mod pool;
mod volatility;

use std::str::FromStr;
use log::{debug, LevelFilter};
use sui_keys::keystore::{AccountKeystore, InMemKeystore};
use sui_sdk::{SuiClient, SuiClientBuilder};
use sui_types::base_types::{ObjectID, SequenceNumber, SuiAddress};
use sui_types::crypto::SignatureScheme;
use sui_types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use sui_types::transaction::{ObjectArg, Transaction, TransactionData, TransactionKind};
use sui_types::{DEEPBOOK_PACKAGE_ID, TypeTag};
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};
use serde_json::Value;
use sui_sdk::rpc_types::{SuiMoveValue, SuiObjectDataOptions, SuiTransactionBlockResponseOptions, SuiTypeTag};
use sui_types::quorum_driver_types::ExecuteTransactionRequestType;
use shared_crypto::intent::Intent;
use sui_sdk::json::SuiJsonValue;
use sui_sdk::rpc_types::SuiMoveValue::Number;
use sui_types::object::Owner;
use sui_types::object::Owner::Shared;
use sui_types::TypeTag::U64;
use crate::constant::{LIMIT_ORDER_NO_RESTRICTION, SUI_PRICE_FEED, USDC_PRICE_FEED};
use crate::market::{get_fills, get_market_price};
use crate::order::{list_open_orders, OrderWrapper, place_limit_order};
use crate::pyth::{get_sui_usdc_price, PythFeeder};
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
    let account_cap_id = response.data[0].data.clone().unwrap().object_id;
    debug!("{:?}", account_cap_id );
    let account_cap_raw = client.read_api().get_object_with_options(account_cap_id, SuiObjectDataOptions::new()).await.unwrap();
    let object_account_cap = account_cap_raw.object().unwrap().object_ref();
    let mut tb = ProgrammableTransactionBuilder::new();
    tb = get_account_balance(tb, sui_tag.clone(), usdc_tag.clone(), pool_id, account_cap_id);
    tb = get_market_price(tb, sui_tag.clone(), usdc_tag.clone(), pool_id);
    let result = client.read_api().dev_inspect_transaction_block(SuiAddress::ZERO, TransactionKind::ProgrammableTransaction(tb.finish()), None, None, None).await;
    let execution_result =  result.unwrap().results.unwrap();
    let item = execution_result.first().unwrap();
    let results = parse_result_u64(item, 0);
    debug!("{:?}", results);
    debug!("{:?}", parse_result_u64(&execution_result[1], 1));
    // 181.94.248.117
    get_fills(&client).await;
    let feeder = PythFeeder::new_suiusdc();
    let result = feeder.get_latest_price().await.unwrap();
    let price = get_sui_usdc_price(result);
    debug!("{:?}", price);
    let mut tb = ProgrammableTransactionBuilder::new();
    let order_wrapper = OrderWrapper::new(&client, pool_id, Some(account_cap_id), None ).await;
    let account_cap_ref = order_wrapper.fetch_account_cap_object_ref().await;
    let tb = order_wrapper.place_limit_order(tb,
        1_500_000,
                                    100_000_000,
                                        true,
        LIMIT_ORDER_NO_RESTRICTION,
        None,
        account_cap_ref,
    );
    let coins = client
        .coin_read_api()
        .get_coins(sender, None, None, None)
        .await.unwrap();
    let coin = coins.data.into_iter().next().unwrap();
    let ptxn = tb.finish();
    let gas_budget = 50_000_000;
    let gas_price = client.read_api().get_reference_gas_price().await.unwrap();
    let tx_data = TransactionData::new_programmable(
        sender,
        vec![coin.object_ref()],
        ptxn,
        gas_budget,
        gas_price,
    );
    // let tx_data = plo;
    let signature = keystore.sign_secure(&sender,
                                         &tx_data,
                                         Intent::sui_transaction()).unwrap();
    let tx = Transaction::from_data(tx_data,
                                    vec![signature],
    );
    let t = tx.transaction_data();
    println!("{:?}", t);
    let response = client.quorum_driver_api().execute_transaction_block(
        tx,
        SuiTransactionBlockResponseOptions::full_content(),
        Some(ExecuteTransactionRequestType::WaitForLocalExecution),

    ).await;
    if response.is_err() {
        println!("got error:{:?}", response)
    } else {
        println!("{:?}", response.unwrap().digest);
    }
}

