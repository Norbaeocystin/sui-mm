use std::env;
use log::{debug, warn};
use shared_crypto::intent::Intent;
use sui_keys::keystore::{AccountKeystore, InMemKeystore};
use sui_sdk::rpc_types::SuiTransactionBlockResponseOptions;
use sui_sdk::SuiClient;
use sui_types::base_types::{ObjectRef, SuiAddress};
use sui_types::crypto::SignatureScheme;
use sui_types::digests::{Digest, TransactionDigest};
use sui_types::quorum_driver_types::ExecuteTransactionRequestType;
use sui_types::transaction::{ProgrammableTransaction, Transaction, TransactionData};

pub struct TransactionWrapper<'a> {
    client: &'a SuiClient,
    keystore: InMemKeystore,
    pub signer: SuiAddress,
}

impl TransactionWrapper<'_> {

    pub fn new(client: &SuiClient) -> TransactionWrapper {
        let mut keystore = InMemKeystore::default();
        let mnemonic = env::var("SUI_WALLET").expect("$SUI_WALLET is not set");
        keystore.import_from_mnemonic(&*mnemonic,
                                      SignatureScheme::ED25519, None
        );
        let sender = keystore.addresses().first().unwrap().clone();
        return TransactionWrapper{ client, keystore: keystore, signer: sender }
    }

    pub async fn process_ptx(&self, ptx: ProgrammableTransaction,
                             gascoin_object_ref: Option<ObjectRef>,
                             gasprice: Option<u64>,
                             gasbudget: Option<u64>,
    ) -> Option<TransactionDigest> {
        let tx_data = TransactionData::new_programmable(
            self.signer,
            vec![if gascoin_object_ref.is_some() {gascoin_object_ref.unwrap()} else { self.client
                .coin_read_api()
                .get_coins(self.signer, None, None, None)
                .await.unwrap().data.into_iter().filter(|x| x.balance > 10 * 50_000_000).next().unwrap().object_ref() }],
            ptx,
            if gasbudget.is_some() {gasbudget.unwrap()} else {50_000_000},
            if gasprice.is_some() {gasprice.unwrap()} else {self.client.read_api().get_reference_gas_price().await.unwrap()},
        );
        // let tx_data = plo;
        let signature = self.keystore.sign_secure(&self.signer,
                                             &tx_data,
                                             Intent::sui_transaction()).unwrap();
        let tx = Transaction::from_data(tx_data,
                                        vec![signature],
        );
        let response = self.client.quorum_driver_api().execute_transaction_block(
            tx,
            SuiTransactionBlockResponseOptions::full_content(),
            Some(ExecuteTransactionRequestType::WaitForLocalExecution),

        ).await;
        if response.is_err() {
            warn!("got error:{:?}", response);
        } else {
            let digest = response.unwrap().digest;
            debug!("{:?}", digest);
            return Some(digest);
        }
        return None;
    }
}