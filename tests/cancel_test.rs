
use std::str::FromStr;
use log::{debug, LevelFilter, warn};
use sui_keys::keystore::{AccountKeystore, InMemKeystore};
use sui_sdk::SuiClientBuilder;
use sui_types::base_types::{ObjectID, SuiAddress};
use sui_types::crypto::SignatureScheme;
use sui_types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use sui_types::transaction::{Transaction, TransactionData};
use std::env;
use sui_sdk::rpc_types::{SuiObjectDataOptions, SuiTransactionBlockResponseOptions, SuiTypeTag};
use sui_types::quorum_driver_types::ExecuteTransactionRequestType;
use shared_crypto::intent::Intent;
use sui_mm::order::OrderWrapper;
use sui_mm::user::{get_account_cap};

#[tokio::test]
async fn cancel_test() {
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
    let account_cap_ref = order_wrapper.fetch_account_cap_object_ref().await;
    let tb = order_wrapper.cancel_all_orders(tb, account_cap_ref);
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
    let response = client.quorum_driver_api().execute_transaction_block(
        tx,
        SuiTransactionBlockResponseOptions::full_content(),
        Some(ExecuteTransactionRequestType::WaitForLocalExecution),

    ).await;
    if response.is_err() {
        warn!("got error:{:?}", response)
    } else {
        debug!("{:?}", response.unwrap().digest);
    }
}