use exonum::crypto::{Hash, PublicKey};
use super::proto;

/// Stores information about a pipe type
#[derive(Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::Participant", serde_pb_convert)]
pub struct Participant {
    /// key
    pub key: PublicKey,
    /// timestamp
    pub timestamp: u64,
    /// have bought
    pub have_bought: bool,
    /// Removed
    pub removed: bool,
    /// Length of the transactions history.
    pub history_len: u64,
    /// `Hash` of the transactions history.
    pub history_hash: Hash,
}

impl Participant {
    /// Creates new participant
    pub fn new(
        &key: &PublicKey,
        timestamp: u64,
        have_bought: bool, 
        removed: bool,
        history_len: u64,
        &history_hash: &Hash,
    ) -> Self {
        Self {
            key,
            timestamp, have_bought, removed, history_len, history_hash
        }
    }
    // TODO buy
    // TODO remove
}