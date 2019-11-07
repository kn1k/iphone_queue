#![allow(bare_trait_objects)]

use exonum::{
    blockchain::{ExecutionError, ExecutionResult, Transaction, TransactionContext},
    crypto::{PublicKey, SecretKey},
    messages::{Message, RawTransaction, Signed},
};

use super::{ proto, schema::Schema, SERVICE_ID };

/// Error codes emitted by pipes transactions during execution.
#[derive(Debug, Fail)]
#[repr(u8)]
pub enum Error {
    /// Participant already exists.
    ///
    /// Can be emitted by `Add`.
    #[fail(display = "Participant already exists")]
    ParticipantAlreadyExists = 0,

    // TODO add some errors
}

impl From<Error> for ExecutionError {
    fn from(value: Error) -> ExecutionError {
        let description = format!("{}", value);
        ExecutionError::with_description(value as u8, description)
    }
}

/// Create pipe type.
#[derive(Serialize, Deserialize, Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::Add")]
pub struct Add {
    /// `PublicKey` of participant.
    pub key: PublicKey,
    /// timestamp
    pub timestamp: u64,
}

/// Transaction group.
#[derive(Serialize, Deserialize, Clone, Debug, TransactionSet)]
pub enum ParticipantTransactions {
    /// Add tx.
    Add(Add),
}

impl Add {
    #[doc(hidden)]
    pub fn sign(
        pk: &PublicKey,
        &key: &PublicKey,
        timestamp: u64,
        sk: &SecretKey) -> Signed<RawTransaction> {

        Message::sign_transaction(
            Self { key, timestamp },
            SERVICE_ID,
            *pk,
            sk,
        )
    }
}

impl Transaction for Add {
    fn execute(&self, context: TransactionContext) -> ExecutionResult {
        let hash = context.tx_hash();

        let mut schema = Schema::new(context.fork());
            
        let key = &self.key;

        if schema.participant(key).is_none() {
            let timestamp = self.timestamp;

            schema.add_participant(key, timestamp, false, false, &hash);
            
            Ok(())
        } else {
            Err(Error::ParticipantAlreadyExists)?
        }
    }
}