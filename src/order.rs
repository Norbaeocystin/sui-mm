use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};
use log::debug;
use sui_sdk::rpc_types::SuiObjectDataOptions;
use sui_sdk::SuiClient;
use sui_types::base_types::{ObjectID, ObjectRef, SequenceNumber, SuiAddress};
use sui_types::object::Owner::Shared;
use sui_types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use sui_types::transaction::{CallArg, ObjectArg};
use sui_types::TypeTag;
use crate::constant::DEEPBOOK_PKG;
use serde::{Serialize,Deserialize};


#[derive(Serialize,Deserialize,Debug)]
pub struct Order {
    order_id: u64,
    client_order_id: u64,
    price: u64,
    original_quantity: u64,
    quantity: u64,
    is_bid: bool,
    owner: SuiAddress,
    expire_timestamp: u64,
    self_matching_prevention: u8
}


pub struct OrderWrapper<'a> {
    client: &'a SuiClient,
    pool_id: ObjectID,
    pool_initial_shared_sequence: SequenceNumber,
    cap_id: ObjectID,
    max_min: u64,
    base_asset: TypeTag,
    quote_asset:TypeTag,
}

impl  OrderWrapper<'_> {
    // creates OrderWrapper, will fetch initial shared version of pool_id, if max_min not provided - 1 hour will be used
    pub async fn new(client: &SuiClient, pool_id: ObjectID, cap_id: Option<ObjectID>, max_min: Option<u64>) -> OrderWrapper{
        let mut account_cap_id = ObjectID::random();
        if cap_id.is_some() {
            account_cap_id = cap_id.unwrap();
        } else {
            // TODO fetch if not exists create ...
        }
        let result = client.read_api().get_object_with_options(pool_id, SuiObjectDataOptions{
            show_type: true,
            show_owner: true,
            show_previous_transaction: false,
            show_display: false,
            show_content: false,
            show_bcs: true,
            show_storage_rebate: false,
        }).await.unwrap();
        let unwrapped = result.data.unwrap();
        let content = unwrapped.type_.unwrap();
        let raw_type = content.to_string().clone();
        let assets = raw_type.split_once("<").unwrap().1.replace(">","").clone();
        let (base_str_raw, quote_str_raw) = assets.split_once(", ").unwrap();
        let (base_str, quote_str) = (base_str_raw.clone(), quote_str_raw.clone());
        let owner = unwrapped.owner.unwrap();
        let mut pool_isv = SequenceNumber::new();
        match owner {
            Shared {initial_shared_version} => {
                // debug!("initial shared version: {}", initial_shared_version);
                pool_isv = initial_shared_version;
            }

            _ => {}
        }
        debug!("base: {} quote: {} pool initial shared version {}", base_str, quote_str, pool_isv);
        return OrderWrapper{
            client,
            pool_id,
            pool_initial_shared_sequence: pool_isv,
            cap_id: account_cap_id,
            max_min: if max_min.is_some() {max_min.unwrap()} else {( 1000 * 60 * 60)},
            base_asset: TypeTag::from_str(base_str).unwrap(),
            quote_asset: TypeTag::from_str(quote_str).unwrap(),
        }
    }

    pub async fn fetch_account_cap_object_ref(&self) -> ObjectRef {
        let account_cap_raw = self.client.read_api().get_object_with_options(self.cap_id, SuiObjectDataOptions::new()).await.unwrap();
        return account_cap_raw.object().unwrap().object_ref();
    }

    // place limit order, if client id not provided it will use timestamp ...
    pub fn place_limit_order(&self, mut tb: ProgrammableTransactionBuilder,
                             price: u64,
                             quantity: u64,
                             is_bid: bool,
                             restriction: u8,
                             client_id: Option<u64>,
        account_cap: ObjectRef,
    ) -> ProgrammableTransactionBuilder {
            let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
            return place_limit_order(tb,
            self.base_asset.clone(),
                                     self.quote_asset.clone(),
                self.pool_id,
                self.pool_initial_shared_sequence,
                t,
                price,
                quantity,
                is_bid,
                if client_id.is_some() { client_id.unwrap()} else {t + self.max_min},
                restriction,
                account_cap,
            );
    }

    pub fn list_open_orders(self, mut tb: ProgrammableTransactionBuilder) -> ProgrammableTransactionBuilder {
        return list_open_orders(tb,
        self.base_asset.clone(),
            self.quote_asset.clone(),
            self.pool_id,
            self.cap_id,
        )
    }

    pub fn get_order_status(self, mut tb: ProgrammableTransactionBuilder, order_id: u64) -> ProgrammableTransactionBuilder {
        return get_order_status(tb,
        self.base_asset.clone(),
            self.quote_asset.clone(),
            self.pool_id,
            order_id,
            self.cap_id,
        );
    }

    pub fn cancel_all_orders(self, mut tb: ProgrammableTransactionBuilder, account_cap_ref: ObjectRef) -> ProgrammableTransactionBuilder {
        cancel_all_orders(tb, self.base_asset, self.quote_asset, self.pool_id, self.pool_initial_shared_sequence, account_cap_ref)
    }

}
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
        initial_shared_version: pool_sequence_number, // SequenceNumber::from_u64(32079148),
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

pub fn cancel_all_orders(mut tb: ProgrammableTransactionBuilder, baseAsset: TypeTag, quoteAsset: TypeTag, pool_id: ObjectID, pool_sequence_order: SequenceNumber, account_cap: ObjectRef) -> ProgrammableTransactionBuilder{
    let pool_object = ObjectArg::SharedObject {
        id: pool_id,
        initial_shared_version: pool_sequence_order, // initial
        mutable: true,
    };
    let account_cap = ObjectArg::ImmOrOwnedObject(account_cap);
    tb.move_call(
        DEEPBOOK_PKG.parse().unwrap(),
        "clob_v2".parse().unwrap(),
        "cancel_all_orders".parse().unwrap(),
        vec![baseAsset, quoteAsset],
        vec![
            CallArg::Object(pool_object),
            CallArg::Object(account_cap)
        ],
    );
    return tb
}

pub fn cancel_order(mut tb: ProgrammableTransactionBuilder, baseAsset: TypeTag, quoteAsset: TypeTag, pool_id: ObjectID, pool_sequence_order: SequenceNumber, order_id: u64, account_cap: ObjectRef) -> ProgrammableTransactionBuilder{
    let pool_object = ObjectArg::SharedObject {
        id: pool_id,
        initial_shared_version: pool_sequence_order, // initial
        mutable: true,
    };
    let account_cap = ObjectArg::ImmOrOwnedObject(account_cap);
    tb.move_call(
        DEEPBOOK_PKG.parse().unwrap(),
        "clob_v2".parse().unwrap(),
        "cancel_all_orders".parse().unwrap(),
        vec![baseAsset, quoteAsset],
        vec![
            CallArg::Object(pool_object),
            CallArg::Pure(order_id.to_le_bytes().to_vec()),
            CallArg::Object(account_cap)
        ],
    );
    return tb
}