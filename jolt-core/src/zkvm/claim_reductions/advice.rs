//! Two-phase advice claim reduction (Stage 6 cycle → Stage 7 address)
//!
//! This module generalizes the previous single-phase `AdviceClaimReduction` so that trusted and
//! untrusted advice can be committed as an arbitrary Dory matrix `2^{nu_a} x 2^{sigma_a}` (balanced
//! by default), while still keeping a **single Stage 8 Dory opening** at the unified Dory point.
//!
//! For an advice matrix embedded as the **top-left block** `2^{nu_a} x 2^{sigma_a}`, the *native*
//! advice evaluation point (in Dory order, LSB-first) is:
//! - `advice_cols = col_coords[0..sigma_a]`
//! - `advice_rows = row_coords[0..nu_a]`
//! - `advice_point = [advice_cols || advice_rows]`
//!
//! In our current pipeline, `cycle` coordinates come from Stage 6 and `addr` coordinates come from
//! Stage 7.
//! - **Phase 1 (Stage 6)**: bind the cycle-derived advice coordinates and output an intermediate
//!   scalar claim `C_mid`.
//! - **Phase 2 (Stage 7)**: resume from `C_mid`, bind the address-derived advice coordinates, and
//!   cache the final advice opening `AdviceMLE(advice_point)` for batching into Stage 8.
//!
//! ## Dummy-gap scaling (within Stage 6)
//! With cycle-major order, there may be a gap during the cycle phase where the cycle variables
//! being bound in the batched sumcheck do not appear in the advice polynommial.
//!
//! We handle this without modifying the generic batched sumcheck by treating those intervening
//! rounds as **dummy internal rounds** (constant univariates), and maintaining a running scaling
//! factor `2^{-dummy_done}` so the per-round univariates remain consistent.
//!
//! Trusted and untrusted advice run as **separate** sumcheck instances (each may have different
//! dimensions).
//!

use std::cell::RefCell;

use crate::field::JoltField;
use crate::poly::commitment::dory::DoryGlobals;
use crate::poly::eq_poly::EqPolynomial;
use crate::poly::multilinear_polynomial::MultilinearPolynomial;
use crate::poly::opening_proof::{
    OpeningAccumulator, OpeningPoint, ProverOpeningAccumulator, SumcheckId,
    VerifierOpeningAccumulator, BIG_ENDIAN, LITTLE_ENDIAN,
};
use crate::poly::unipoly::UniPoly;
use crate::subprotocols::sumcheck_prover::SumcheckInstanceProver;
use crate::subprotocols::sumcheck_verifier::{SumcheckInstanceParams, SumcheckInstanceVerifier};
use crate::transcripts::Transcript;
use crate::utils::math::Math;
use crate::zkvm::claim_reductions::{
    build_permuted_precommitted_polys, precommitted_dummy_round_scale, PrecomittedParams,
    PrecomittedProver, PrecommittedClaimReduction, PrecommittedSchedulingReference,
    TWO_PHASE_DEGREE_BOUND,
};
use crate::zkvm::config::ReadWriteConfig;
use allocative::Allocative;
use rayon::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Allocative)]
pub enum PreCommitted {
    CycleVariables,
    AddressVariables,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Allocative)]
pub enum AdviceKind {
    Trusted,
    Untrusted,
}

#[derive(Clone, Allocative)]
pub struct AdviceClaimReductionParams<F: JoltField> {
    pub kind: AdviceKind,
    pub phase: PreCommitted,
    pub precommitted: PrecommittedClaimReduction<F>,
    pub gamma: F,
    pub single_opening: bool,
    pub log_t: usize,
    pub advice_col_vars: usize,
    pub advice_row_vars: usize,
    pub r_val_eval: OpeningPoint<BIG_ENDIAN, F>,
    pub r_val_final: Option<OpeningPoint<BIG_ENDIAN, F>>,
}

impl<F: JoltField> AdviceClaimReductionParams<F> {
    pub fn new(
        kind: AdviceKind,
        advice_size_bytes: usize,
        rw_config: &ReadWriteConfig,
        scheduling_reference: PrecommittedSchedulingReference,
        accumulator: &dyn OpeningAccumulator<F>,
        transcript: &mut impl Transcript,
    ) -> Self {
        let log_t = DoryGlobals::main_t().log_2();
        let single_opening = rw_config.needs_single_advice_opening(log_t);

        let r_val_eval = accumulator
            .get_advice_opening(kind, SumcheckId::RamValEvaluation)
            .map(|(p, _)| p)
            .unwrap();
        let r_val_final = if single_opening {
            None
        } else {
            accumulator
                .get_advice_opening(kind, SumcheckId::RamValFinalEvaluation)
                .map(|(p, _)| p)
        };

        let gamma: F = transcript.challenge_scalar();

        let (advice_col_vars, advice_row_vars) =
            DoryGlobals::advice_sigma_nu_from_max_bytes(advice_size_bytes);
        let total_vars = advice_row_vars + advice_col_vars;
        let precommitted = PrecommittedClaimReduction::new(
            total_vars,
            advice_row_vars,
            advice_col_vars,
            scheduling_reference,
        );

        Self {
            kind,
            phase: PreCommitted::CycleVariables,
            precommitted,
            gamma,
            advice_col_vars,
            advice_row_vars,
            single_opening,
            log_t,
            r_val_eval,
            r_val_final,
        }
    }

    /// (Total # advice variables) - (# variables bound during cycle phase)
    pub fn num_address_phase_rounds(&self) -> usize {
        self.precommitted.num_address_phase_rounds()
    }
}

impl<F: JoltField> AdviceClaimReductionParams<F> {
    fn is_cycle_phase(&self) -> bool {
        self.phase == PreCommitted::CycleVariables
    }

    pub fn transition_to_address_phase(&mut self) {
        self.phase = PreCommitted::AddressVariables;
    }

    pub fn round_offset(&self, max_num_rounds: usize) -> usize {
        self.precommitted
            .round_offset(self.is_cycle_phase(), max_num_rounds)
    }
}

impl<F: JoltField> SumcheckInstanceParams<F> for AdviceClaimReductionParams<F> {
    fn input_claim(&self, accumulator: &dyn OpeningAccumulator<F>) -> F {
        match self.phase {
            PreCommitted::CycleVariables => {
                let mut claim = F::zero();
                if let Some((_, eval)) =
                    accumulator.get_advice_opening(self.kind, SumcheckId::RamValEvaluation)
                {
                    claim += eval;
                }
                if !self.single_opening {
                    if let Some((_, final_eval)) =
                        accumulator.get_advice_opening(self.kind, SumcheckId::RamValFinalEvaluation)
                    {
                        claim += self.gamma * final_eval;
                    }
                }
                claim
            }
            PreCommitted::AddressVariables => {
                // Address phase starts from the cycle phase intermediate claim.
                accumulator
                    .get_advice_opening(self.kind, SumcheckId::AdviceClaimReductionCyclePhase)
                    .expect("Cycle phase intermediate claim not found")
                    .1
            }
        }
    }

    fn degree(&self) -> usize {
        TWO_PHASE_DEGREE_BOUND
    }

    fn num_rounds(&self) -> usize {
        self.precommitted.num_rounds_for_phase(self.is_cycle_phase())
    }

    fn normalize_opening_point(&self, challenges: &[F::Challenge]) -> OpeningPoint<BIG_ENDIAN, F> {
        self.precommitted.normalize_opening_point(
            self.is_cycle_phase(),
            challenges,
            self.log_t,
        )
    }
}

impl<F: JoltField> PrecomittedParams<F> for AdviceClaimReductionParams<F> {
    fn is_cycle_phase(&self) -> bool {
        self.phase == PreCommitted::CycleVariables
    }

    fn is_cycle_phase_round(&self, round: usize) -> bool {
        self.precommitted.is_cycle_phase_round(round)
    }

    fn is_address_phase_round(&self, round: usize) -> bool {
        self.precommitted.is_address_phase_round(round)
    }

    fn cycle_alignment_rounds(&self) -> usize {
        self.precommitted.cycle_alignment_rounds()
    }

    fn address_alignment_rounds(&self) -> usize {
        self.precommitted.address_alignment_rounds()
    }

    fn record_cycle_challenge(&mut self, challenge: F::Challenge) {
        self.precommitted.record_cycle_challenge(challenge);
    }
}

#[derive(Allocative)]
pub struct AdviceClaimReductionProver<F: JoltField> {
    core: PrecomittedProver<F, AdviceClaimReductionParams<F>>,
}

impl<F: JoltField> AdviceClaimReductionProver<F> {
    pub fn params(&self) -> &AdviceClaimReductionParams<F> {
        self.core.params()
    }

    pub fn transition_to_address_phase(&mut self) {
        self.core.params_mut().transition_to_address_phase();
    }

    pub fn initialize(
        params: AdviceClaimReductionParams<F>,
        advice_poly: MultilinearPolynomial<F>,
    ) -> Self {
        let eq_evals = if params.single_opening {
            EqPolynomial::evals(&params.r_val_eval.r)
        } else {
            let evals = EqPolynomial::evals(&params.r_val_eval.r);
            let r_final = params
                .r_val_final
                .as_ref()
                .expect("r_val_final must exist when !single_opening");
            let eq_final = EqPolynomial::evals_with_scaling(&r_final.r, Some(params.gamma));
            evals
                .par_iter()
                .zip(eq_final.par_iter())
                .map(|(e1, e2)| *e1 + e2)
                .collect()
        };
        let (advice_poly, eq_poly): (MultilinearPolynomial<F>, MultilinearPolynomial<F>) = {
            let MultilinearPolynomial::U64Scalars(poly) = advice_poly else {
                panic!("Advice should have u64 coefficients");
            };
            build_permuted_precommitted_polys(
                poly.coeffs,
                eq_evals,
                params.precommitted.embedding_mode,
                params.advice_row_vars,
                params.advice_col_vars,
                &params.precommitted.scheduling_reference,
                params.log_t,
            )
        };

        Self {
            core: PrecomittedProver::new(params, advice_poly, eq_poly),
        }
    }
}

impl<F: JoltField, T: Transcript> SumcheckInstanceProver<F, T> for AdviceClaimReductionProver<F> {
    fn get_params(&self) -> &dyn SumcheckInstanceParams<F> {
        self.core.params()
    }

    fn round_offset(&self, max_num_rounds: usize) -> usize {
        self.core.params().round_offset(max_num_rounds)
    }

    fn compute_message(&mut self, round: usize, previous_claim: F) -> UniPoly<F> {
        self.core.compute_message(round, previous_claim)
    }

    fn ingest_challenge(&mut self, r_j: F::Challenge, round: usize) {
        self.core.ingest_challenge(r_j, round);
    }

    fn cache_openings(
        &self,
        accumulator: &mut ProverOpeningAccumulator<F>,
        transcript: &mut T,
        sumcheck_challenges: &[F::Challenge],
    ) {
        let params = self.core.params();
        let opening_point = params.normalize_opening_point(sumcheck_challenges);
        if params.phase == PreCommitted::CycleVariables {
            let c_mid = self.core.cycle_intermediate_claim();

            match params.kind {
                AdviceKind::Trusted => accumulator.append_trusted_advice(
                    transcript,
                    SumcheckId::AdviceClaimReductionCyclePhase,
                    opening_point.clone(),
                    c_mid,
                ),
                AdviceKind::Untrusted => accumulator.append_untrusted_advice(
                    transcript,
                    SumcheckId::AdviceClaimReductionCyclePhase,
                    opening_point.clone(),
                    c_mid,
                ),
            }
        }

        if let Some(advice_claim) = self.core.final_claim_if_ready() {
            match params.kind {
                AdviceKind::Trusted => accumulator.append_trusted_advice(
                    transcript,
                    SumcheckId::AdviceClaimReduction,
                    opening_point,
                    advice_claim,
                ),
                AdviceKind::Untrusted => accumulator.append_untrusted_advice(
                    transcript,
                    SumcheckId::AdviceClaimReduction,
                    opening_point,
                    advice_claim,
                ),
            }
        }
    }

    #[cfg(feature = "allocative")]
    fn update_flamegraph(&self, flamegraph: &mut allocative::FlameGraphBuilder) {
        flamegraph.visit_root(self);
    }
}

pub struct AdviceClaimReductionVerifier<F: JoltField> {
    pub params: RefCell<AdviceClaimReductionParams<F>>,
}

impl<F: JoltField> AdviceClaimReductionVerifier<F> {
    pub fn new(
        kind: AdviceKind,
        advice_size_bytes: usize,
        rw_config: &ReadWriteConfig,
        scheduling_reference: PrecommittedSchedulingReference,
        accumulator: &VerifierOpeningAccumulator<F>,
        transcript: &mut impl Transcript,
    ) -> Self {
        let params = AdviceClaimReductionParams::new(
            kind,
            advice_size_bytes,
            rw_config,
            scheduling_reference,
            accumulator,
            transcript,
        );

        Self {
            params: RefCell::new(params),
        }
    }
}

fn evaluate_advice_eq_combined<F: JoltField>(
    params: &AdviceClaimReductionParams<F>,
    opening_point_be: &OpeningPoint<BIG_ENDIAN, F>,
) -> F {
    let eq_eval = EqPolynomial::mle(&opening_point_be.r, &params.r_val_eval.r);
    if params.single_opening {
        eq_eval
    } else {
        let r_final = params
            .r_val_final
            .as_ref()
            .expect("r_val_final must exist when !single_opening");
        let eq_final = EqPolynomial::mle(&opening_point_be.r, &r_final.r);
        eq_eval + params.gamma * eq_final
    }
}

impl<F: JoltField, T: Transcript> SumcheckInstanceVerifier<F, T>
    for AdviceClaimReductionVerifier<F>
{
    fn get_params(&self) -> &dyn SumcheckInstanceParams<F> {
        unsafe { &*self.params.as_ptr() }
    }

    fn expected_output_claim(
        &self,
        accumulator: &VerifierOpeningAccumulator<F>,
        sumcheck_challenges: &[F::Challenge],
    ) -> F {
        let params = self.params.borrow();
        match params.phase {
            PreCommitted::CycleVariables => {
                accumulator
                    .get_advice_opening(params.kind, SumcheckId::AdviceClaimReductionCyclePhase)
                    .unwrap_or_else(|| panic!("Cycle phase intermediate claim not found",))
                    .1
            }
            PreCommitted::AddressVariables => {
                let opening_point = params.normalize_opening_point(sumcheck_challenges);
                let advice_claim = accumulator
                    .get_advice_opening(params.kind, SumcheckId::AdviceClaimReduction)
                    .expect("Final advice claim not found")
                    .1;
                let eq_combined = evaluate_advice_eq_combined(&params, &opening_point);

                // Account for Phase 1's internal dummy-gap traversal via constant scaling.
                let scale: F = precommitted_dummy_round_scale(&params.precommitted);
                advice_claim * eq_combined * scale
            }
        }
    }

    fn cache_openings(
        &self,
        accumulator: &mut VerifierOpeningAccumulator<F>,
        transcript: &mut T,
        sumcheck_challenges: &[F::Challenge],
    ) {
        let mut params = self.params.borrow_mut();
        if params.phase == PreCommitted::CycleVariables {
            let opening_point = params.normalize_opening_point(sumcheck_challenges);
            match params.kind {
                AdviceKind::Trusted => accumulator.append_trusted_advice(
                    transcript,
                    SumcheckId::AdviceClaimReductionCyclePhase,
                    opening_point.clone(),
                ),
                AdviceKind::Untrusted => accumulator.append_untrusted_advice(
                    transcript,
                    SumcheckId::AdviceClaimReductionCyclePhase,
                    opening_point.clone(),
                ),
            }
            let opening_point_le: OpeningPoint<LITTLE_ENDIAN, F> = opening_point.match_endianness();
            params
                .precommitted
                .set_cycle_var_challenges(opening_point_le.r);
        }

        if params.num_address_phase_rounds() == 0 || params.phase == PreCommitted::AddressVariables
        {
            let opening_point = params.normalize_opening_point(sumcheck_challenges);
            match params.kind {
                AdviceKind::Trusted => accumulator.append_trusted_advice(
                    transcript,
                    SumcheckId::AdviceClaimReduction,
                    opening_point,
                ),
                AdviceKind::Untrusted => accumulator.append_untrusted_advice(
                    transcript,
                    SumcheckId::AdviceClaimReduction,
                    opening_point,
                ),
            }
        }
    }

    fn round_offset(&self, max_num_rounds: usize) -> usize {
        let params = self.params.borrow();
        params.round_offset(max_num_rounds)
    }
}
