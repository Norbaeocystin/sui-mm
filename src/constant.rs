
pub const TIMESTAMP_INF: u64 = ((1u128 << 64 - 1) as u64);
pub const SUI_USDC_DECIMALS:u64 = 1_000_000;
pub const USDC_DECIMALS:u64 = 1_000_000;
pub const SUI_DECIMALS:u64 = 1_000_000_000;
// limit orders;
pub const LIMIT_ORDER_NO_RESTRICTION: u8 = 0;
pub const LIMIT_ORDER_IMMEDIATE_OR_CANCEL: u8 = 1;
pub const LIMIT_ORDER_FILL_OR_KILL: u8 = 2;
pub const LIMIT_ORDER_POST_OR_ABORT: u8 = 3;
pub const DEEPBOOK_PKG: &str = "0x000000000000000000000000000000000000000000000000000000000000dee9";
pub const HERMES_LATES_PRICE_FEEDS: &str = "https://hermes.pyth.network/api/latest_price_feeds"; // ?ids[]=0x23d7315113f5b1d3ba7a83604c44b94d79f4fd69af77f804fc7f920a6dc65744
pub const SUI_PRICE_FEED: &str = "0x23d7315113f5b1d3ba7a83604c44b94d79f4fd69af77f804fc7f920a6dc65744";
pub const USDC_PRICE_FEED: &str = "0xeaa020c61cc479712813461ce153894a96a6c00b21ed0cfc2798d1f9a9e9c94a";

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