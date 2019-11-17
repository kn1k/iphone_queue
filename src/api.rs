use exonum::{
    api::{self, ServiceApiBuilder, ServiceApiState},
    blockchain::{self, BlockProof, TransactionMessage},
    crypto::{Hash, PublicKey},
    explorer::BlockchainExplorer,
    helpers::Height,
};
use exonum_merkledb::{ListProof, MapProof};

use super::{schema::Schema, SERVICE_ID};
use crate::participant::Participant;

/// Get first participant key
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct GetFirstQuery {}

/// Describes the query parameters for the `get_participant` endpoint.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct ParticipantQuery {
    /// Public key of the queried participant.
    pub pub_key: PublicKey,
}

/// Proof of existence for specific participant.
#[derive(Debug, Serialize, Deserialize)]
pub struct ParticipantProof {
    /// Proof of the whole database table.
    pub to_table: MapProof<Hash, Hash>,
    /// Proof of the specific participant in this table.
    pub to_participant: MapProof<PublicKey, Participant>,
}

/// Participant history.
#[derive(Debug, Serialize, Deserialize)]
pub struct ParticipantHistory {
    /// Proof of the list of transaction hashes.
    pub proof: ListProof<Hash>,
    /// List of above transactions.
    pub transactions: Vec<TransactionMessage>,
}

/// Participant information.
#[derive(Debug, Serialize, Deserialize)]
pub struct ParticipantInfo {
    /// Proof of the last block.
    pub block_proof: BlockProof,
    /// Proof of the appropriate participant.
    pub participant_proof: ParticipantProof,
    /// History of the appropriate participant.
    pub participant_history: Option<ParticipantHistory>,
}

/// Public service API description.
#[derive(Debug, Clone, Copy)]
pub struct PublicApi;

impl PublicApi {
    /// Endpoint for getting a single participant.
    fn participant_info(
        state: &ServiceApiState,
        query: ParticipantQuery,
    ) -> api::Result<ParticipantInfo> {
        let snapshot = state.snapshot();
        let general_schema = blockchain::Schema::new(&snapshot);
        let currency_schema = Schema::new(&snapshot);

        let max_height = general_schema.block_hashes_by_height().len() - 1;

        let block_proof = general_schema
            .block_and_precommits(Height(max_height))
            .unwrap();

        let to_table: MapProof<Hash, Hash> =
            general_schema.get_proof_to_service_table(SERVICE_ID, 0);

        let to_participant: MapProof<PublicKey, Participant> =
            currency_schema.participants().get_proof(query.pub_key);

        let participant_proof = ParticipantProof {
            to_table,
            to_participant,
        };

        let participant = currency_schema.participant(&query.pub_key);

        let explorer = BlockchainExplorer::new(state.blockchain());

        let participant_history = participant.map(|_| {
            let history = currency_schema.participant_history(&query.pub_key);
            let proof = history.get_range_proof(0..history.len());

            let transactions = history
                .iter()
                .map(|record| explorer.transaction_without_proof(&record).unwrap())
                .collect::<Vec<_>>();

            ParticipantHistory {
                proof,
                transactions,
            }
        });

        Ok(ParticipantInfo {
            block_proof,
            participant_proof,
            participant_history,
        })
    }

    fn get_first(state: &ServiceApiState, _: GetFirstQuery) -> api::Result<String> {
        let snapshot = state.snapshot();
        let schema = Schema::new(&snapshot);
        let first = schema.first_participant().unwrap();

        Ok(first.key.to_hex())
    }

    /// Wires the above endpoint to public scope of the given `ServiceApiBuilder`.
    pub fn wire(builder: &mut ServiceApiBuilder) {
        builder
            .public_scope()
            .endpoint("v1/iphone_queue/info", Self::participant_info)
            .endpoint("v1/iphone_queue/get_first", Self::get_first);
    }
}
