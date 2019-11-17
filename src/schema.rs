//! Iphone queue database schema.
use crate::participant::Participant;
use exonum::crypto::{Hash, PublicKey};
use exonum_merkledb::{IndexAccess, ObjectHash, ProofListIndex, ProofMapIndex};
use std::cmp::Ordering;

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

    /// Returns `ProofMapIndex` with participants.
    pub fn participants(&self) -> ProofMapIndex<T, PublicKey, Participant> {
        ProofMapIndex::new(PARTICIPANT_TYPES_TABLE, self.view.clone())
    }

    /// Returns history of the participant with the given public key.
    pub fn participant_history(&self, public_key: &PublicKey) -> ProofListIndex<T, Hash> {
        ProofListIndex::new_in_family(PARTICIPANT_HISTORY_TABLE, public_key, self.view.clone())
    }

    /// Returns participant for the given public key.
    pub fn participant(&self, pub_key: &PublicKey) -> Option<Participant> {
        self.participants().get(pub_key)
    }

    /// Returns the state hash of service.
    pub fn state_hash(&self) -> Vec<Hash> {
        vec![self.participants().object_hash()]
    }

    fn order_decs(&self, d1: &Participant, d2: &Participant) -> Ordering
    {
        let sort_by_timestamp = d2.timestamp.cmp(&d1.timestamp);
        // if sort_by_timestamp != Ordering::Equal
        // {
        //     return sort_by_timestamp;
        // }
        sort_by_timestamp
    }

    /// Returns first participant.
    pub fn first_participant(&self) -> Option<Participant> {
        let participants = self.participants();
        let p = participants.iter()
            .map(|x| x.1)
            .filter(|x| !x.have_bought && !x.removed)
            .max_by(|x, y| self.order_decs(x, y))
            .unwrap();
       Some(p)
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

    /// Participant have bought a phone
    pub fn participant_have_bought(
        &mut self,
        participant: Participant,
        transaction: &Hash
    ) {
        let participant = {
            let mut history = self.participant_history(&participant.key);
            history.push(*transaction);
            let history_hash = history.object_hash();
            participant.buy(&history_hash)
        };
        self.participants().put(&participant.key, participant.clone());
    }

    /// Remove a participant.
    pub fn remove_participant(
        &mut self,
        participant: Participant,
        transaction: &Hash
    ) {
        let participant = {
            let mut history = self.participant_history(&participant.key);
            history.push(*transaction);
            let history_hash = history.object_hash();
            participant.remove(&history_hash)
        };
        self.participants().put(&participant.key, participant.clone());
    }
}
