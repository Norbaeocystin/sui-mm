
use std::str::FromStr;
use log::{debug, info, LevelFilter};
use sui_keys::keystore::{AccountKeystore, InMemKeystore};
use sui_sdk::SuiClientBuilder;
use sui_types::base_types::{ObjectID, SuiAddress};
use sui_types::crypto::SignatureScheme;
use sui_types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use sui_types::transaction::{Transaction, TransactionData, TransactionKind};
use sui_types::TypeTag;
use std::env;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use sui_sdk::rpc_types::{SuiObjectDataOptions, SuiTransactionBlockResponseOptions, SuiTypeTag};
use sui_types::quorum_driver_types::ExecuteTransactionRequestType;
use shared_crypto::intent::Intent;
use sui_mm::constant::{LIMIT_ORDER_NO_RESTRICTION, LIMIT_ORDER_POST_OR_ABORT};
use sui_mm::market::{CalculationResult, get_fills, get_market_price};
use sui_mm::order::{Order, OrderWrapper};
use sui_mm::pyth::{get_sui_usdc_price, PythFeeder};
use sui_mm::user::{get_account_balance, get_account_cap, parse_result_account_balance};
use sui_mm::utils::{parse_result_u64, parse_result_u64_from_vec};
use bcs::from_bytes;
use tokio::sync::Mutex;
use tokio::time::sleep;
use sui_mm::transaction::TransactionWrapper;
use sui_mm::volatility::Volatility;
use clap::Parser;
use clap::ArgAction;
use statistical::mean;
use sui_mm::calculations::calculate_totals;


#[derive(Parser)]
struct Cli {
    #[arg(short,long, default_value_t = 1)]
    price: u64,
    #[arg(short,long, default_value_t = 30)]
    calculations: u64,
    #[arg(short,long, default_value = "0.0.0")]
    version: String,
    #[arg(short, long, action)]
    debug: bool,
}


#[tokio::main]
async fn main() {
    let cli: Cli = Cli::parse();
    if cli.debug {
        env_logger::builder().filter_level(LevelFilter::Debug).init();
    } else {
        env_logger::builder().filter_level(LevelFilter::Info).init();
    }
    let price_interval_sec = cli.price.clone();
    let calculations_interval_sec = cli.calculations.clone();
    let sui_rpc = env::var("SUI_RPC").expect("$SUI_RPC is not set");
    let client = SuiClientBuilder::default()
        .build(sui_rpc.clone())
        .await.unwrap();
    let pool_id = ObjectID::from_str("0x4405b50d791fd3346754e8171aaab6bc2ed26c2c46efdd033c14b30ae507ac33").unwrap();
    let transaction_wrapper = TransactionWrapper::new(&client);
    let response = get_account_cap(&client, &transaction_wrapper.signer ).await.unwrap();
    let account_cap_id = response.data[0].data.clone().unwrap().object_id;
    let order_wrapper = OrderWrapper::new(&client, pool_id, Some(account_cap_id), None).await;
    let (base_asset, quote_asset) = (order_wrapper.base_asset.clone(), order_wrapper.quote_asset.clone());
    let pyth_feeder = PythFeeder::new_suiusdc();
    let result = pyth_feeder.get_latest_price().await.unwrap();
    let price = get_sui_usdc_price(result);
    let sui_usdc_price = Arc::new(Mutex::new(price));
    let sui_usdc_price_clone = Arc::clone(&sui_usdc_price);
    let volatility_cal: Option<f64> = None;
    let vol_mutex = Arc::new(Mutex::new(volatility_cal));
    let vol_mutex_clone = Arc::clone(&vol_mutex);
    let calculations = get_fills(&client, base_asset.to_string(), quote_asset.to_string()).await;
    let calc_mutex = Arc::new(Mutex::new(calculations));
    let calc_mutex_clone = Arc::clone(&calc_mutex);
    tokio::spawn(
        async move {
            let client = SuiClientBuilder::default()
                .build(sui_rpc.clone())
                .await.unwrap();
            let ba = base_asset.to_string();
            let qa = quote_asset.to_string();
            loop {
                sleep(Duration::from_secs(calculations_interval_sec)).await;
                let calc = get_fills(&client, ba.clone(), qa.clone()).await;
                info!("calculation on events for pool: {:?}", calc);
                let mut guard = calc_mutex_clone.lock().await;
                *guard = calc;
            }

        }
    );
    tokio::spawn(
        async move {
            let mut volatility = Volatility{ prices: vec![], length: 300 };
            loop {
                sleep(Duration::from_secs(price_interval_sec)).await;
                let result_raw = pyth_feeder.get_latest_price().await;
                if result_raw.is_none() {
                    continue;
                }
                let price = get_sui_usdc_price(result_raw.unwrap());
                let mut price_guard = sui_usdc_price_clone.lock().await;
                volatility.insert(price);
                if volatility.prices.len() == volatility.length {
                    let result = volatility.clone().volatility();
                        let vol_calc = result.unwrap();
                        let mut vol_guard = vol_mutex_clone.lock().await;
                        *vol_guard = Some(vol_calc);
                }
                *price_guard = price;
            }
        }
    );
    loop {
        sleep(Duration::from_millis(400)).await;
        let price: f64 = sui_usdc_price.lock().await.clone();
        let mut vol = vol_mutex.lock().await.clone();
        let calc = calc_mutex.lock().await.clone();
        if vol.is_some() {
            let (balance_data, bid_ask_data, open_orders) = order_wrapper.get_data().await;
            // TODO implement logic when there are already opened orders ...
            // If size of existing order is less ...
            // todo if open order is
            if open_orders.len() > 0 {
                let order = open_orders.first().unwrap();
                let diff = (order.price as f64 - (((bid_ask_data[0] + bid_ask_data[1])/2) as f64)).abs()/order.price as f64;
                let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
                // if the price difference is greater than 1 percent
                if diff > 0.01 || (order.expire_timestamp - t) < 60 * 1000 {
                    // TODO cancel
                    let account_cap_ref = order_wrapper.clone().fetch_account_cap_object_ref().await;
                    let mut tb = ProgrammableTransactionBuilder::new();
                    let tb = order_wrapper.clone().cancel_all_orders(tb, account_cap_ref);
                    let result = transaction_wrapper.process_ptx(tb.finish(), None, None, None).await;
                    info!("cancel {:?} {} {}", result, diff,  (order.expire_timestamp - t) < 60 * 1000);
                } else {
                    continue;
                }
                debug!("Orders opened: {:?}", open_orders);
                continue;
            };
            let vol_number = vol.unwrap();
            // TODO - check if prices bid and ask and pyth price is are too different ...
            let orders_to_do = calculate_totals(&balance_data, price, None, None, calc, vol_number);
            info!("Price: {} {:?} {:?} {:?}", price, vol, calc, bid_ask_data );
            let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
            if orders_to_do.is_some() {
                let orders_calc = orders_to_do.unwrap();
                let mut tb = ProgrammableTransactionBuilder::new();
                let account_cap_ref = order_wrapper.fetch_account_cap_object_ref().await;
                let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
                if orders_calc.bid_quantity > 0 {
                    tb = order_wrapper.place_limit_order(tb,
                            orders_calc.bid_price,
                        orders_calc.bid_quantity,
                        true,
                        LIMIT_ORDER_POST_OR_ABORT,
                        None,
                        account_cap_ref,
                        Some(t + orders_calc.duration_ms),
                    );
                }
                if orders_calc.ask_quantity > 0 {
                    tb = order_wrapper.place_limit_order(tb,
                                                    orders_calc.ask_price,
                                                    orders_calc.ask_quantity,
                                                    false,
                                                    LIMIT_ORDER_POST_OR_ABORT,
                                                    None,
                                                    account_cap_ref,
                                                    Some(t + orders_calc.duration_ms),
                    );
                }
                let result = transaction_wrapper.process_ptx(tb.finish(), None, None, None).await;
                info!("{:?}", result);
            }
        }
    }
}

