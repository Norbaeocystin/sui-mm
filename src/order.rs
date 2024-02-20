

/*
/// Returns (base quantity filled, quote quantity filled, whether a maker order is being placed, order id of the maker order)
public fun place_limit_order<BaseAsset, QuoteAsset>(
        pool: &mut Pool<BaseAsset, QuoteAsset>,
        price: u64,
        quantity: u64,
        is_bid: bool,
        expire_timestamp: u64,
        restriction: u8,
        clock: &Clock,
        account_cap: &AccountCap,
        ctx: &mut TxContext
    ): (u64, u64, bool, u64)
 */
use std::str::FromStr;
use sui_types::base_types::{ObjectID, ObjectRef, SequenceNumber};
use sui_types::digests::ObjectDigest;
use sui_types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use sui_types::transaction::{CallArg, ObjectArg};
use sui_types::TypeTag;
use crate::constant::DEEPBOOK_PKG;

// all is in base asset ...
pub fn place_limit_order(mut tb: ProgrammableTransactionBuilder,
                         baseAsset: TypeTag,
                         quoteAsset: TypeTag,
                         pool_id: ObjectID,
                         pool_sequence_number: SequenceNumber,
                         client_order_id: u64,
                         price: u64,
                         quantity: u64,
                         is_bid: bool,
                         expire_timestamp: u64, // ms
                         restriction: u8,
                         account_cap: ObjectRef,
) -> ProgrammableTransactionBuilder{
    let pool = ObjectArg::SharedObject{
        id: pool_id,
        initial_shared_version: SequenceNumber::from_u64(32079148),
        mutable: true,
    };
    let account_cap = ObjectArg::ImmOrOwnedObject(account_cap);
    // 0: (account_cap, Default::default(), ObjectDigest::from_str("4KqUgNZCU3fsqeeeShLEHiHtu8bVkgYY7r7wfeq7U157").unwrap()) };
    let clock_object = ObjectArg::SharedObject {
        id: ObjectID::from_str("0x0000000000000000000000000000000000000000000000000000000000000006").unwrap(),
        initial_shared_version: SequenceNumber::from_u64(1),
        mutable: false
    };
    let bid: u8 = if is_bid {1} else {0};
    tb.move_call(
        DEEPBOOK_PKG.parse().unwrap(),
        "clob_v2".parse().unwrap(),
        "place_limit_order".parse().unwrap(),
        vec![baseAsset, quoteAsset],
        vec![
            CallArg::Object(pool), // 1
            CallArg::Pure(client_order_id.to_le_bytes().to_vec()), // 2
            CallArg::Pure(price.to_le_bytes().to_vec()), // 3
            CallArg::Pure(quantity.to_le_bytes().to_vec()), // 4
            CallArg::Pure(vec![0_u8]), // 5 self matching
            CallArg::Pure(vec![bid]), // 6
            CallArg::Pure(expire_timestamp.to_le_bytes().to_vec()), // 7
            CallArg::Pure(vec![restriction]), // 8
            CallArg::Object(clock_object), // 9
            CallArg::Object(account_cap), // 10
            // CallArg::Object(tx_context_object),
        ],
    );
    return tb
}

// returns Vec<u64>
pub fn list_open_orders(mut tb: ProgrammableTransactionBuilder, baseAsset: TypeTag, quoteAsset: TypeTag, pool_id: ObjectID, account_cap: ObjectID) -> ProgrammableTransactionBuilder{
    let pool_object = ObjectArg::SharedObject {
        id: pool_id,
        initial_shared_version: Default::default(),
        mutable: true,
    };
    let account_cap = ObjectArg::SharedObject {
        id: account_cap,
        initial_shared_version: Default::default(),
        mutable: true,
    };
    tb.move_call(
        DEEPBOOK_PKG.parse().unwrap(),
        "clob_v2".parse().unwrap(),
        "list_open_orders".parse().unwrap(),
        vec![baseAsset, quoteAsset],
        vec![
            CallArg::Object(pool_object),
            CallArg::Object(account_cap)
        ],
    );
    return tb
}

pub fn get_order_status(mut tb: ProgrammableTransactionBuilder, baseAsset: TypeTag, quoteAsset: TypeTag, pool_id: ObjectID,order_id: u64, account_cap: ObjectID) -> ProgrammableTransactionBuilder{
    let pool_object = ObjectArg::SharedObject {
        id: pool_id,
        initial_shared_version: Default::default(),
        mutable: true,
    };
    let account_cap = ObjectArg::SharedObject {
        id: account_cap,
        initial_shared_version: Default::default(),
        mutable: true,
    };
    tb.move_call(
        DEEPBOOK_PKG.parse().unwrap(),
        "clob_v2".parse().unwrap(),
        "get_order_status".parse().unwrap(),
        vec![baseAsset, quoteAsset],
        vec![
            CallArg::Object(pool_object),
            CallArg::Pure(order_id.to_le_bytes().to_vec()),
            CallArg::Object(account_cap)
        ],
    );
    return tb
}

/*
/// Parameters expected by this func
///
///   0. `[pool]` Object ID refers to the pool containing the trading pair
///   1. `[order_id]` order id of the order being queried
///   2. `[account_cap]` Object ID of the account_cap authorizing the
///       accessilility to the escrow account

/// Returns the order info of the order being queried
clob_v2
public fun get_order_status<BaseAsset, QuoteAsset>(
     pool: &Pool<BaseAsset, QuoteAsset>,
     order_id: u64,
     account_cap: &AccountCap
): &Order
 */