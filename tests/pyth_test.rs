
use log::{debug, LevelFilter};
use sui_mm::pyth::{get_sui_usdc_price, PythFeeder};

#[tokio::test]
async fn price_test() {
    env_logger::builder().filter_level(LevelFilter::Debug).init();
    let feeder = PythFeeder::new_suiusdc();
    let result = feeder.get_latest_price().await.unwrap();
    let price = get_sui_usdc_price(result);
    debug!("{:?}", price);
}

