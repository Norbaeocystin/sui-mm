use log::{debug, info, warn};
use crate::constant::{SUI_DECIMALS, USDC_DECIMALS};
use crate::market::CalculationResult;

#[derive(Debug)]
pub struct Result {
    pub ask_price: u64,
    pub ask_quantity: u64,
    pub bid_price: u64,
    pub bid_quantity: u64,
    pub duration_ms: u64,
}

pub fn calculate_totals(inputs: &Vec<u64>, price: f64, base_decimals: Option<u64>, quote_decimals: Option<u64>,
                        calc: CalculationResult,
                        volatility: f64) -> Option<Result> {
    let b_dec = if base_decimals.is_some() {base_decimals.unwrap()} else {SUI_DECIMALS};
    let q_dec = if quote_decimals.is_some() {quote_decimals.unwrap()} else {USDC_DECIMALS};
    let base_amounts = ((inputs[0].clone() + inputs[1].clone()) as f64 * price) as u64 / b_dec;
    let quote_amounts = (inputs[2].clone() + inputs[3].clone()) / q_dec;
    let total = base_amounts + quote_amounts;
    info!("base amount in quote: {} quote: {} total: {}", base_amounts, quote_amounts, total);
    // if there are waiting orders do nothing
    if inputs[1] > 0 || inputs[3] > 0 {
        debug!("open orders");
        return None
    }
    // if the volatility is great, do nothing
    // TODO incorporate filled_total
    // because if filled_total is bigger - the influence on spread can be lower ...
    if volatility > 0.3 {
        debug!("volatility too high");
        return None;
    }
    let mut spread = 0.025_f64;
    let ratio = (calc.filled_total as f64)/(total as f64);
    // decrease influence of volatility on spread
    let decrease_vol = if calc.filled_per_s > total {ratio} else {1.0};
    let spread = spread * ((volatility/(0.012 * decrease_vol)) + 1.0).min(20.0);
    // TODO decrease size depending on volatility ...
    let mut result = Result{
        ask_price: 0,
        ask_quantity: 0,
        bid_price: 0,
        bid_quantity: 0,
        duration_ms: 0,
    };
    let ask_spread = if base_amounts >= quote_amounts {spread} else { spread * (((quote_amounts + 1)/(base_amounts +1)).min(4)) as f64};
    let ask_price = price + (price * (ask_spread/100.0));
    let bid_spread = if quote_amounts >= base_amounts {spread} else { spread * (((base_amounts + 1)/(quote_amounts + 1)).min(4)) as f64};
    let bid_price = price - (price * (bid_spread/100.0));
    // max time
    let maximum_duration: u64 = 90 * 60 * 1000; // 30 MIN;
    let base_quote_ratio = (base_amounts as f64/quote_amounts as f64);
    // duration of order is min: 10 min max: 60 min
    let duration_final = ((maximum_duration as f64/ ratio) as u64).max(16 * 60 * 1000).min(maximum_duration);
    let raw_ask_quantity = if inputs[0] > 0 {((inputs[0])/ 100_000_000) * 100_000_000} else {0};
    let raw_bid_quantity = if quote_amounts > 0 {((((quote_amounts as f64)/ ask_price) as u64 * b_dec)/100_000_000) * 100_000_000} else {0};
    result.duration_ms = duration_final;
    result.ask_price = ((ask_price * 1_000_000_f64) as u64/ 100) * 100;
    result.ask_quantity = if (base_amounts <= quote_amounts ||( base_quote_ratio < 1.2 &&  base_quote_ratio > 0.8)) {raw_ask_quantity} else {(((raw_ask_quantity - raw_bid_quantity)/2)/100_000_000) * 100_000_000 };
    result.bid_price = ((bid_price * 1_000_000_f64) as u64/ 100) * 100;
    result.bid_quantity = if (quote_amounts <= base_amounts  || ( base_quote_ratio < 1.2 &&  base_quote_ratio > 0.8)) {raw_bid_quantity} else {(((raw_bid_quantity - raw_ask_quantity)/2)/100_000_000) * 100_000_000};
    info!("{:?} {}", result, spread);
    if result.bid_quantity == 0 && result.ask_quantity == 0 {
        return None
    }
    return Some(result);
}