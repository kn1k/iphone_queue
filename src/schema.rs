//! Iphone queue database schema.
use crate::participant::Participant;
use exonum::crypto::{Hash, PublicKey};
use exonum_merkledb::{IndexAccess, ObjectHash, ProofListIndex, ProofMapIndex};

/// Pipe types table name
pub const PARTICIPANT_TYPES_TABLE: &str = "iphone_queue.participant";
/// Pipe type history table name
pub const PARTICIPANT_HISTORY_TABLE: &str = "iphone_queue.participant.history";

/// Database schema.
#[derive(Debug)]
pub struct Schema<T> {
    view: T,
}

impl<T> AsMut<T> for Schema<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.view
    }
}

impl<T> Schema<T>
where
    T: IndexAccess,
{
    /// Creates a new schema from the database view.
    pub fn new(view: T) -> Self {
        Schema { view }
    }

    /// Returns `ProofMapIndex` with pipe types.
    pub fn participants(&self) -> ProofMapIndex<T, PublicKey, Participant> {
        ProofMapIndex::new(PARTICIPANT_TYPES_TABLE, self.view.clone())
    }

    /// Returns history of the pipe type with the given public key.
    pub fn participant_history(&self, public_key: &PublicKey) -> ProofListIndex<T, Hash> {
        ProofListIndex::new_in_family(PARTICIPANT_HISTORY_TABLE, public_key, self.view.clone())
    }

    /// Returns pipe type for the given public key.
    pub fn participant(&self, pub_key: &PublicKey) -> Option<Participant> {
        self.participants().get(pub_key)
    }

    /// Returns the state hash of service.
    pub fn state_hash(&self) -> Vec<Hash> {
        vec![self.participants().object_hash()]
    }

    /// Create new participant and append first record to its history.
    pub fn add_participant(
        &mut self,
        key: &PublicKey,
        timestamp: u64,
        have_bought: bool,
        removed: bool,
        transaction: &Hash,
    ) {
        let created_participant = {
            let mut history = self.participant_history(key);
            history.push(*transaction);
            let history_hash = history.object_hash();

            Participant::new(
                key,
                timestamp,
                have_bought,
                removed,
                history.len(),
                &history_hash,
            )
        };
        self.participants().put(key, created_participant);
    }
}
