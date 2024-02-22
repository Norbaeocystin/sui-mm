use std::ops::Sub;
use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use log::{debug, info};
use serde_derive::{Deserialize, Serialize};
use sui_sdk::rpc_types::{EventFilter, SuiEvent};
use sui_sdk::SuiClient;
use sui_types::base_types::ObjectID;
use sui_types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use sui_types::transaction::{CallArg, ObjectArg};
use sui_types::TypeTag;
use crate::constant::{DEEPBOOK_PKG, SUI_DECIMALS, SUI_USDC_DECIMALS};

/// returns (best_bid_price, best_ask_price)
pub fn get_market_price(mut tb: ProgrammableTransactionBuilder,
                        baseAsset: TypeTag,
                        quoteAsset: TypeTag,
                        pool_id: ObjectID, ) -> ProgrammableTransactionBuilder {
    let pool_object = ObjectArg::SharedObject {
        id: pool_id,
        initial_shared_version: Default::default(),
        mutable: true,
    };
    tb.move_call(DEEPBOOK_PKG.parse().unwrap(),
                 "clob_v2".parse().unwrap(),
                 "get_market_price".parse().unwrap(),
                 vec![baseAsset, quoteAsset],
                 vec![CallArg::Object(pool_object,
                 ), ]
    );
    return tb;
}

/*
{ id: EventID { tx_digest: TransactionDigest(GrBu1rVaoyW6HLZAVHtx3SyZ1CiC34T7FCz38y2U4Tir), event_seq: 0 }, package_id: 0xa2ce75c54b8ee30b15b235faf8c6c01407bf90cf3fbcec5d84b04ec25400131a, transaction_module: Identifier("jk"), sender: 0x7fac4740148563dbebb980f4161ef7e7f7fdc0f7b6311227fafc7ef60899f096, type_: StructTag { address: 000000000000000000000000000000000000000000000000000000000000dee9, module: Identifier("clob_v2"), name: Identifier("OrderFilled"), type_params: [Struct(StructTag { address: 0000000000000000000000000000000000000000000000000000000000000002, module: Identifier("sui"), name: Identifier("SUI"), type_params: [] }), Struct(StructTag { address: 5d4b302506645c37ff133b98c4b50a5ae14841659738d6d733d59d0d217a93bf, module: Identifier("coin"), name: Identifier("COIN"), type_params: [] })] }, parsed_json: Object {"base_asset_quantity_filled": String("700000000000"), "base_asset_quantity_remaining": String("2700000000000"), "is_bid": Bool(true), "maker_address": String("0xf995d6df20e18421928ff0648bd583ccdf384ab05791d8be21d32977a37dacfc"), "maker_client_order_id": String("1708380292894207686"), "maker_rebates": String("249718"), "order_id": String("5925000"), "original_quantity": String("5000000000000"), "pool_id": String("0x4405b50d791fd3346754e8171aaab6bc2ed26c2c46efdd033c14b30ae507ac33"), "price": String("1783700"), "taker_address": String("0x11f8f568d871ff0cf829aca81e51a06a6869d12abe0b3351b914a4673ea3d857"), "taker_client_order_id": String("4399"), "taker_commission": String("249718")}
 */

pub async fn get_fills(client: &SuiClient, base_asset: String, quote_asset: String) -> CalculationResult {
    let query = format!("0xdee9::clob_v2::OrderFilled<{base_asset}, {quote_asset}>");
    // let query = "0xdee9::clob_v2::OrderFilled<0x2::sui::SUI, 0x5d4b302506645c37ff133b98c4b50a5ae14841659738d6d733d59d0d217a93bf::coin::COIN>";
    let events_response = client.event_api().query_events(EventFilter::MoveEventType(query.parse().unwrap()),
                                                          None, Some(100),
                                                          true).await;
    let unwrapped = events_response.unwrap().data;
    // debug!("{:?}", unwrapped);
    return calculate(&unwrapped);
}

#[derive(Serialize,Deserialize,Debug, Copy,Clone)]
pub struct CalculationResult {
    pub duration: u64,
    pub filled_total: u64,
    pub unfilled_total: u64,
    pub filled_per_s: u64,
    pub n: u64
}

pub fn calculate(events: &Vec<SuiEvent>) -> CalculationResult{
    let length = events.len();
    let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    let max_time = u128::from(events.last().clone().unwrap().timestamp_ms.unwrap());
    debug!("{} {}", max_time, t);
    let duration = (t - max_time)/ 1000;
    let mut filled_total: u128 = 0;
    let mut unfilled_total: u128 = 0;
    for item in events.iter(){
        let price = u128::from_str(item.parsed_json.get("price").unwrap().as_str().unwrap()).unwrap();
        let base_asset_quantity_filled = u128::from_str(item.parsed_json.get("base_asset_quantity_filled").unwrap().as_str().unwrap()).unwrap();
        filled_total += (base_asset_quantity_filled * price)/(u128::from(SUI_USDC_DECIMALS) * u128::from(SUI_DECIMALS));
        let base_asset_quantity_remaining = u128::from_str(item.parsed_json.get("base_asset_quantity_remaining").unwrap().as_str().unwrap()).unwrap();
        unfilled_total += (base_asset_quantity_remaining * price)/(u128::from(SUI_USDC_DECIMALS) * u128::from(SUI_DECIMALS));
        // println!("{} {} {}", item.timestamp_ms.unwrap(), price, base_asset_quantity_filled);
    }
    // Filled per second is amount of dollars exchanged per second ...
    debug!("Duration: {}, Filled total: {}, Unfilled total: {}, Filled per second: {}", duration, filled_total, unfilled_total, filled_total/duration);
    return CalculationResult{
        duration: duration as u64,
        filled_total: filled_total as u64,
        unfilled_total: unfilled_total as u64,
        filled_per_s: (filled_total / duration ) as u64, // HOW MUCH QUOTED IN QUOTE WAS EXCHANGED PER SECOND ...
        n: length as u64,
    }
}