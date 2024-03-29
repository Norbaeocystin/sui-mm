use std::str::FromStr;
use log::{debug, warn};
use reqwest::Client;
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderValue};
use serde_derive::Deserialize;
use serde_derive::Serialize;
use crate::constant::{HERMES_LATES_PRICE_FEEDS, SUI_PRICE_FEED, SUI_USDC_DECIMALS, USDC_PRICE_FEED};


pub struct PythFeeder {
    client: Client,
    price_feeds: Vec<String>,
}

pub fn get_sui_usdc_price(feeds: LatestPriceFeeds) -> f64 {
    let sui_expo = f64::from(feeds[0].price.expo as i8);
    let sui_exponent: f64 = f64::powf(10.0,sui_expo);
    let usdc_expo = f64::from(feeds[1].price.expo as i8);
    let usdc_exponent: f64 = f64::powf(10.0,usdc_expo);
    let sui_price_raw = f64::from_str(&*feeds[0].price.price).unwrap();
    let usdc_price_raw = f64::from_str(&*feeds[1].price.price).unwrap();
    let sui_price = sui_exponent * sui_price_raw;
    let usdc_price = usdc_exponent * usdc_price_raw;
    return sui_price/usdc_price
}

impl PythFeeder {
    pub fn new(feeds: Vec<String>) -> PythFeeder {
        return PythFeeder{ client: reqwest::Client::new(), price_feeds: feeds }
    }

    pub fn new_suiusdc() -> PythFeeder {
        return PythFeeder{ client: reqwest::Client::new(), price_feeds: vec![SUI_PRICE_FEED.to_string(), USDC_PRICE_FEED.to_string()] }
    }

    pub async fn get_latest_price(&self) -> Option<LatestPriceFeeds> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        let response = self.client
            .get(HERMES_LATES_PRICE_FEEDS)
            .query(&self.price_feeds.clone().iter().map(|x| ("ids[]", x.as_str())).collect::<Vec<(&str,&str)>>()) // &("ids[]", &feed.clone()))
            .headers(headers)
            .send()
            .await
            .unwrap();
        match response.status() {
            reqwest::StatusCode::OK => {
                // on success, parse our JSON to an APIResponse
                match response.json::<LatestPriceFeeds>().await {
                    Ok(parsed) => {
                        debug!("Success!");
                        return Some(parsed);
                    },
                    Err(_) => warn!("Hm, the response didn't match the shape we expected."),
                };
            }
            other => {
                warn!("Uh oh! Something unexpected happened: {:?}", other);
            }
        };
        return None;
    }
}

pub type LatestPriceFeeds = Vec<PriceFeed>;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceFeed {
    pub id: String,
    pub price: Price,
    #[serde(rename = "ema_price")]
    pub ema_price: EmaPrice,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Price {
    pub price: String,
    pub conf: String,
    pub expo: i64,
    #[serde(rename = "publish_time")]
    pub publish_time: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmaPrice {
    pub price: String,
    pub conf: String,
    pub expo: i64,
    #[serde(rename = "publish_time")]
    pub publish_time: i64,
}
