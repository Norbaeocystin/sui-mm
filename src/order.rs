

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
use sui_types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use sui_types::TypeTag;
use crate::constant::DEEPBOOK_PKG;

// all is in base asset ...
pub fn place_limit_order(mut tb: ProgrammableTransactionBuilder, baseAsset: TypeTag, quoteAsset: TypeTag) -> ProgrammableTransactionBuilder{
    tb.move_call(
        DEEPBOOK_PKG.parse().unwrap(),
        "clob_v2".parse().unwrap(),
        "place_limit_order".parse().unwrap(),
        vec![baseAsset, quoteAsset],
        vec![],
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