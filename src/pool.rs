use sui_sdk::rpc_types::{EventFilter, EventPage};
use sui_sdk::SuiClient;

pub async fn get_pools_created(client: &SuiClient) -> EventPage {
    let response = client.event_api().query_events(
        EventFilter::MoveEventType("0xdee9::clob_v2::PoolCreated".parse().unwrap()),
        None,
        None,
        false
    ).await.unwrap();
    // does not return anything
    // let response2 = client.event_api().query_events(
    //     EventFilter::MoveEventType("0xdee9::clob::PoolCreated".parse().unwrap()),
    //     None,
    //     None,
    //     false
    // ).await.unwrap();
    return response
}