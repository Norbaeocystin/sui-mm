use std::env;
use log::{debug, LevelFilter};
use sui_sdk::SuiClientBuilder;
use sui_mm::pool::get_pools_created;

#[tokio::test]
async fn pool_test() {
    env_logger::builder().filter_level(LevelFilter::Debug).init();
    let sui_rpc = env::var("SUI_RPC").expect("$SUI_RPC is not set");
    let client = SuiClientBuilder::default()
        .build(sui_rpc)
        .await.unwrap();
    let r = get_pools_created(&client).await;
    debug!("{:?}", r);
    debug!("{}", r.data.len());
}