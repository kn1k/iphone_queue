#[macro_use]
extern crate serde_json;

use exonum::{
    api::node::public::explorer::{TransactionQuery, TransactionResponse},
    crypto::{self, Hash, PublicKey, SecretKey, PUBLIC_KEY_LENGTH},
    messages::{self, RawTransaction, Signed},
};
use exonum_testkit::{ApiKind, TestKit, TestKitApi, TestKitBuilder};

// Import data types used in tests from the crate where the service is defined.
use iphone_queue::{
    api::{ParticipantQuery, ParticipantInfo/*, GetFirstQuery*/},
    transactions::{Add},
    participant::Participant,
    Service,
};

/// add participant test
#[test]
fn test_add_participant() {
    let (mut testkit, api) = create_testkit();
    let (pk, _) = crypto::gen_keypair();
    // Create and send a transaction via API
    let (tx, _) = api.add_participant(&pk, 100);
    testkit.create_block();
    api.assert_tx_status(tx.hash(), &json!({ "type": "success" }));

    // Check that the user indeed is persisted by the service.
    let p = api.get_participant(pk).unwrap();
    assert_eq!(p.key, pk);
    assert_eq!(p.timestamp, 100);

    // let first = api.get_first().unwrap();
    // assert_eq!(first, pk);
}

/// Wrapper for the cryptocurrency service API allowing to easily use it
/// (compared to `TestKitApi` calls).
struct ParticipantsApi {
    pub inner: TestKitApi,
}

impl ParticipantsApi {
    /// Generates a wallet creation transaction with a random key pair, sends it over HTTP,
    /// and checks the synchronous result (i.e., the hash of the transaction returned
    /// within the response).
    /// Note that the transaction is not immediately added to the blockchain, but rather is put
    /// to the pool of unconfirmed transactions.
    fn add_participant(&self, pk: &PublicKey, timestamp: u64) -> (Signed<RawTransaction>, SecretKey) {
        let (pubkey, key) = crypto::gen_keypair();
        // Create a pre-signed transaction
        let tx = Add::sign(&pubkey, pk, timestamp, &key);

        let data = messages::to_hex_string(&tx);
        let tx_info: TransactionResponse = self
            .inner
            .public(ApiKind::Explorer)
            .query(&json!({ "tx_body": data }))
            .post("v1/transactions")
            .unwrap();
        assert_eq!(tx_info.tx_hash, tx.hash());
        (tx, key)
    }

    fn get_participant(&self, pub_key: PublicKey) -> Option<Participant> {
        let participant_info = self
            .inner
            .public(ApiKind::Service("iphone_queue"))
            .query(&ParticipantQuery { pub_key })
            .get::<ParticipantInfo>("v1/iphone_queue/info")
            .unwrap();

        let to_participant = participant_info.participant_proof.to_participant.check().unwrap();
        println!("{:?}", to_participant);
        let (_, participant) = to_participant.all_entries().find(|(&key, _)| key == pub_key)?;
        participant.cloned()
    }

    // fn get_first(&self) -> Option<PublicKey> {
    //     let get_first = self
    //         .inner
    //         .public(ApiKind::Service("pipe_marking"))
    //         .query(&GetFirstQuery { })
    //         .get::<Vec<Participant>>("v1/participants/first")
    //         .unwrap();

    //     Some(get_first.first().unwrap().key)
    // }
/*
    /// Sends a transfer transaction over HTTP and checks the synchronous result.
    fn transfer(&self, tx: &Signed<RawTransaction>) {
        let data = messages::to_hex_string(&tx);
        let tx_info: TransactionResponse = self
            .inner
            .public(ApiKind::Explorer)
            .query(&json!({ "tx_body": data }))
            .post("v1/transactions")
            .unwrap();
        assert_eq!(tx_info.tx_hash, tx.hash());
    }

    /// Asserts that a wallet with the specified public key is not known to the blockchain.
    fn assert_no_wallet(&self, pub_key: PublicKey) {
        let wallet_info: WalletInfo = self
            .inner
            .public(ApiKind::Service("cryptocurrency"))
            .query(&WalletQuery { pub_key })
            .get("v1/wallets/info")
            .unwrap();

        let to_wallet = wallet_info.wallet_proof.to_wallet.check().unwrap();
        assert!(to_wallet.missing_keys().any(|&key| key == pub_key))
    }*/

    /// Asserts that the transaction with the given hash has a specified status.
    fn assert_tx_status(&self, tx_hash: Hash, expected_status: &serde_json::Value) {
        let info: serde_json::Value = self
            .inner
            .public(ApiKind::Explorer)
            .query(&TransactionQuery::new(tx_hash))
            .get("v1/transactions")
            .unwrap();

        if let serde_json::Value::Object(mut info) = info {
            let tx_status = info.remove("status").unwrap();
            assert_eq!(tx_status, *expected_status);
        } else {
            panic!("Invalid transaction info format, object expected");
        }
    }
}

/// Creates a testkit together with the API wrapper defined above.
fn create_testkit() -> (TestKit, ParticipantsApi) {
    let testkit = TestKitBuilder::validator().with_service(Service).create();
    let api = ParticipantsApi {
        inner: testkit.api(),
    };
    (testkit, api)
}
