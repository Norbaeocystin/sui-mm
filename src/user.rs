/*
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "suix_getOwnedObjects",
  "params": [
    "0xac9d68e280e2dc576e004169fe31e73e3e6897eb2a7b5681f5af34b070ce8447",
    {
      "filter": {
        "StructType": "0x000000000000000000000000000000000000000000000000000000000000dee9::custodian_v2::AccountCap"
      },
      "options": {
        "showBcs": false,
        "showContent": true,
        "showDisplay": false,
        "showOwner": false,
        "showPreviousTransaction": false,
        "showStorageRebate": false,
        "showType": false
      }
    },
    null,
    null
  ]
}
 */

use std::str::FromStr;
use sui_sdk::error::SuiRpcResult;
use sui_sdk::rpc_types::{ObjectsPage, SuiExecutionResult, SuiObjectDataFilter, SuiObjectResponseQuery};
use sui_sdk::SuiClient;
use sui_types::base_types::{ObjectID, SequenceNumber, SuiAddress};
use sui_types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use sui_types::transaction::{CallArg, ObjectArg};
use sui_types::TypeTag;

const ACCOUNT_CAP_TAG: &str = "0x000000000000000000000000000000000000000000000000000000000000dee9::custodian_v2::AccountCap";

pub async fn get_account_cap(client: &SuiClient, address: &SuiAddress) -> SuiRpcResult<ObjectsPage>{
    let response = client.read_api().get_owned_objects(*address, Some(SuiObjectResponseQuery {
        filter: Some(SuiObjectDataFilter::StructType(
            ACCOUNT_CAP_TAG.parse().unwrap())),
        options: None
    }), None, None).await;
    return response
}

// (base_avail, base_locked, quote_avail, quote_locked)
pub fn parse_result_account_balance(sui_execution_result: &SuiExecutionResult) -> Vec<u64>{
    let mut results = vec![];
    for (bytes, _) in sui_execution_result.return_values.iter() {
        let r = u64::from_le_bytes(bytes.as_slice().try_into().unwrap());
        results.push(r);
    }
    return results;
}


// response (base_avail, base_locked, quote_avail, quote_locked)
pub fn get_account_balance(mut tb: ProgrammableTransactionBuilder, baseAsset: TypeTag, quoteAsset: TypeTag, pool_id: ObjectID, account_cap: ObjectID)
-> ProgrammableTransactionBuilder {
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
    tb.move_call("0x000000000000000000000000000000000000000000000000000000000000dee9".parse().unwrap(),
                 "clob_v2".parse().unwrap(),
                 "account_balance".parse().unwrap(),
                 vec![baseAsset, quoteAsset],
                 vec![CallArg::Object(pool_object,
                 ),
                 CallArg::Object(account_cap),
                 ]
                 );
    return tb;
}