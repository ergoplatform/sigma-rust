use crate::box_id::BoxId;
use crate::prover_result::ProverResult;

pub struct Input {
    pub box_id: BoxId,
    pub spending_proof: ProverResult,
}
