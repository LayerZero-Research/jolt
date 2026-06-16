use crate::field::JoltField;
use crate::poly::opening_proof::{OpeningPoint, BIG_ENDIAN};
use crate::utils::errors::ProofVerifyError;

pub enum FinalOpeningPointParts<F: JoltField> {
    // A larger precommitted/advice opening already determines the final PCS point.
    DominantPrecommittedAnchor {
        point: OpeningPoint<BIG_ENDIAN, F>,
    },
    // Native openings still need PCS-specific ordering before they become a final point.
    Native {
        r_address_stage7: Vec<F::Challenge>,
        r_cycle_stage6: OpeningPoint<BIG_ENDIAN, F>,
        hamming_point: OpeningPoint<BIG_ENDIAN, F>,
        log_k_chunk: usize,
    },
}

impl<F: JoltField> FinalOpeningPointParts<F> {
    pub(crate) fn into_canonical(self) -> Result<OpeningPoint<BIG_ENDIAN, F>, ProofVerifyError> {
        match self {
            Self::DominantPrecommittedAnchor { point } => Ok(point),
            Self::Native {
                r_address_stage7,
                r_cycle_stage6,
                hamming_point,
                log_k_chunk,
            } => {
                let native_cycle = &hamming_point.r[log_k_chunk..];
                if r_cycle_stage6.r.len() < native_cycle.len() {
                    return Err(ProofVerifyError::DoryError(
                        "stage6 cycle challenges shorter than native cycle vars".to_string(),
                    ));
                }
                if r_cycle_stage6.r[..native_cycle.len()] != *native_cycle {
                    return Err(ProofVerifyError::DoryError(format!(
                        "canonical Stage-8 expects stage6 cycle prefix to equal native cycle vars \
                         (cycle_full_len={}, native_len={})",
                        r_cycle_stage6.r.len(),
                        native_cycle.len()
                    )));
                }
                let cycle_extra = &r_cycle_stage6.r[native_cycle.len()..];
                let cycle_extra_and_anchor =
                    [cycle_extra, r_address_stage7.as_slice(), native_cycle].concat();
                Ok(OpeningPoint::<BIG_ENDIAN, F>::new(cycle_extra_and_anchor))
            }
        }
    }
}
