use allocative::Allocative;
use rayon::prelude::*;
use std::ops::Range;

use crate::field::{JoltField, MaybeAllocative};
use crate::poly::commitment::dory::{DoryGlobals, DoryLayout};
use crate::poly::multilinear_polynomial::{BindingOrder, MultilinearPolynomial, PolynomialBinding};
use crate::poly::opening_proof::{
    OpeningAccumulator, OpeningPoint, ProverOpeningAccumulator, BIG_ENDIAN, LITTLE_ENDIAN,
};
use crate::poly::unipoly::UniPoly;
use crate::subprotocols::sumcheck_prover::SumcheckInstanceProver;
use crate::subprotocols::sumcheck_verifier::SumcheckInstanceParams;
use crate::transcripts::Transcript;

pub(crate) mod sealed {
    pub trait Sealed {}
}

/// Unified two-phase marker for all pre-committed claim reductions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Allocative)]
pub enum PreCommitted {
    CycleVariables,
    AddressVariables,
}

/// Shared phase/cycle-challenge state for two-phase claim reductions over pre-committed
/// polynomials (e.g. advice and program image).
#[derive(Clone, Allocative)]
pub struct PreCommittedPolyClaimReductionState<F: JoltField, Phase: Copy + Eq + Allocative> {
    pub phase: Phase,
    /// Challenges bound during phase 1, stored in little-endian round order.
    pub cycle_var_challenges: Vec<F::Challenge>,
}

impl<F: JoltField, Phase: Copy + Eq + Allocative> PreCommittedPolyClaimReductionState<F, Phase> {
    pub fn new(initial_phase: Phase) -> Self {
        Self {
            phase: initial_phase,
            cycle_var_challenges: vec![],
        }
    }

    /// Shared round-offset policy:
    /// - cycle phase aligns to the start of the batched challenge vector
    /// - final phase is front-loaded (offset 0)
    pub fn round_offset(
        &self,
        cycle_phase: Phase,
        max_num_rounds: usize,
        cycle_alignment_rounds: usize,
    ) -> usize {
        if self.phase == cycle_phase {
            max_num_rounds.saturating_sub(cycle_alignment_rounds)
        } else {
            0
        }
    }

    pub fn set_cycle_challenges_from_opening_point(
        &mut self,
        opening_point: &OpeningPoint<BIG_ENDIAN, F>,
    ) {
        let opening_point_le: OpeningPoint<LITTLE_ENDIAN, F> = opening_point.match_endianness();
        self.cycle_var_challenges = opening_point_le.r;
    }

    pub fn set_cycle_challenges_from_sumcheck_slice(
        &mut self,
        sumcheck_challenges: &[F::Challenge],
    ) {
        self.cycle_var_challenges = sumcheck_challenges.to_vec();
    }
}

/// Number of internal dummy rounds between bound column and row ranges in phase 1.
#[inline]
pub fn internal_dummy_gap_len(
    cycle_phase_col_rounds: &Range<usize>,
    cycle_phase_row_rounds: &Range<usize>,
) -> usize {
    let cycle_phase_total_rounds = if !cycle_phase_row_rounds.is_empty() {
        cycle_phase_row_rounds.end - cycle_phase_col_rounds.start
    } else {
        cycle_phase_col_rounds.len()
    };
    cycle_phase_total_rounds - (cycle_phase_col_rounds.len() + cycle_phase_row_rounds.len())
}

pub fn cycle_phase_round_schedule(
    log_t: usize,
    log_k_chunk: usize,
    main_col_vars: usize,
    poly_row_vars: usize,
    poly_col_vars: usize,
) -> (Range<usize>, Range<usize>) {
    match DoryGlobals::get_layout() {
        DoryLayout::CycleMajor => {
            let col_end = std::cmp::min(log_t, poly_col_vars);
            let col_binding_rounds = 0..col_end;
            // In CycleMajor, row-phase variables of the embedded balanced block must stay aligned
            // with the main matrix row split (`main_col_vars`) to match Stage 8 top-left embedding.
            let row_start = std::cmp::min(
                log_t,
                std::cmp::max(std::cmp::min(log_t, main_col_vars), col_end),
            );
            let row_end = std::cmp::min(log_t, row_start + poly_row_vars);
            let row_binding_rounds = row_start..row_end;
            (col_binding_rounds, row_binding_rounds)
        }
        DoryLayout::AddressMajor => {
            let col_end = std::cmp::min(log_t, poly_col_vars.saturating_sub(log_k_chunk));
            let col_binding_rounds = 0..col_end;
            let row_start_unclamped = main_col_vars.saturating_sub(log_k_chunk);
            let row_start = std::cmp::min(log_t, std::cmp::max(row_start_unclamped, col_end));
            let row_end = std::cmp::min(log_t, row_start + poly_row_vars);
            let row_binding_rounds = row_start..row_end;
            (col_binding_rounds, row_binding_rounds)
        }
    }
}

pub fn precommitted_num_rounds<Phase: Copy + Eq>(
    phase: Phase,
    cycle_phase: Phase,
    cycle_phase_col_rounds: &Range<usize>,
    cycle_phase_row_rounds: &Range<usize>,
    total_poly_vars: usize,
) -> usize {
    if phase == cycle_phase {
        if !cycle_phase_row_rounds.is_empty() {
            cycle_phase_row_rounds.end - cycle_phase_col_rounds.start
        } else {
            cycle_phase_col_rounds.len()
        }
    } else {
        let first_phase_rounds = cycle_phase_col_rounds.len() + cycle_phase_row_rounds.len();
        total_poly_vars - first_phase_rounds
    }
}

pub fn normalize_two_phase_opening_point<F: JoltField, Phase: Copy + Eq + Allocative>(
    reduction: &PreCommittedPolyClaimReductionState<F, Phase>,
    cycle_phase: Phase,
    cycle_phase_col_rounds: &Range<usize>,
    cycle_phase_row_rounds: &Range<usize>,
    challenges: &[F::Challenge],
) -> OpeningPoint<BIG_ENDIAN, F> {
    if reduction.phase == cycle_phase {
        let compact_offset = cycle_phase_col_rounds.start;
        let compact_col_rounds = 0..cycle_phase_col_rounds.len();
        let compact_row_rounds = cycle_phase_row_rounds.start.saturating_sub(compact_offset)
            ..cycle_phase_row_rounds.end.saturating_sub(compact_offset);
        let mut cycle_var_challenges: Vec<F::Challenge> =
            Vec::with_capacity(cycle_phase_col_rounds.len() + cycle_phase_row_rounds.len());
        cycle_var_challenges.extend_from_slice(&challenges[compact_col_rounds]);
        if !cycle_phase_row_rounds.is_empty() {
            cycle_var_challenges.extend_from_slice(&challenges[compact_row_rounds]);
        }
        return OpeningPoint::<LITTLE_ENDIAN, F>::new(cycle_var_challenges).match_endianness();
    }

    match DoryGlobals::get_layout() {
        DoryLayout::CycleMajor => OpeningPoint::<LITTLE_ENDIAN, F>::new(
            [reduction.cycle_var_challenges.as_slice(), challenges].concat(),
        )
        .match_endianness(),
        DoryLayout::AddressMajor => OpeningPoint::<LITTLE_ENDIAN, F>::new(
            [challenges, reduction.cycle_var_challenges.as_slice()].concat(),
        )
        .match_endianness(),
    }
}

/// Param access used by the shared pre-committed parent sumcheck.
pub trait PreCommittedClaimReductionParams<F: JoltField> {
    fn reduction(&self) -> &PreCommittedPolyClaimReductionState<F, PreCommitted>;
    fn reduction_mut(&mut self) -> &mut PreCommittedPolyClaimReductionState<F, PreCommitted>;
    fn cycle_phase_col_rounds(&self) -> &Range<usize>;
    fn cycle_phase_row_rounds(&self) -> &Range<usize>;
    fn total_poly_vars(&self) -> usize;
    fn cycle_alignment_rounds(&self) -> usize;
    fn address_alignment_rounds(&self) -> usize;

    fn is_cycle_phase(&self) -> bool {
        self.reduction().phase == PreCommitted::CycleVariables
    }

    fn num_rounds_for_current_phase(&self) -> usize {
        precommitted_num_rounds(
            self.reduction().phase,
            PreCommitted::CycleVariables,
            self.cycle_phase_col_rounds(),
            self.cycle_phase_row_rounds(),
            self.total_poly_vars(),
        )
    }

    fn round_offset(&self, max_num_rounds: usize) -> usize {
        self.reduction().round_offset(
            PreCommitted::CycleVariables,
            max_num_rounds,
            self.cycle_alignment_rounds(),
        )
    }

    fn transition_to_address_phase(&mut self) {
        self.reduction_mut().phase = PreCommitted::AddressVariables;
    }
}

/// Shared params glue for pre-committed claim-reduction sumchecks.
///
/// Implementers provide only instance-specific pieces; shared scheduling and opening-point
/// normalization are inherited from [`PreCommittedClaimReductionParams`].
pub trait PreCommittedSumcheckInstanceParams<F: JoltField>:
    PreCommittedClaimReductionParams<F> + sealed::Sealed
{
    fn precommitted_degree(&self) -> usize;

    fn precommitted_input_claim(&self, accumulator: &dyn OpeningAccumulator<F>) -> F;

    fn precommitted_normalize_opening_point(
        &self,
        challenges: &[F::Challenge],
    ) -> OpeningPoint<BIG_ENDIAN, F> {
        normalize_two_phase_opening_point(
            self.reduction(),
            PreCommitted::CycleVariables,
            self.cycle_phase_col_rounds(),
            self.cycle_phase_row_rounds(),
            challenges,
        )
    }
}

impl<F, P> SumcheckInstanceParams<F> for P
where
    F: JoltField,
    P: PreCommittedSumcheckInstanceParams<F>,
{
    fn degree(&self) -> usize {
        self.precommitted_degree()
    }

    fn num_rounds(&self) -> usize {
        self.num_rounds_for_current_phase()
    }

    fn input_claim(&self, accumulator: &dyn OpeningAccumulator<F>) -> F {
        self.precommitted_input_claim(accumulator)
    }

    fn normalize_opening_point(&self, challenges: &[F::Challenge]) -> OpeningPoint<BIG_ENDIAN, F> {
        self.precommitted_normalize_opening_point(challenges)
    }
}

/// Shared prover state for pre-committed claim reductions.
///
/// This removes boilerplate in child provers by keeping common `(params, value, eq, scale)`
/// fields in a single struct reused across all implementations.
#[derive(Allocative)]
pub struct PreCommittedPolyReductionCore<F: JoltField, P: PreCommittedClaimReductionParams<F>> {
    pub params: P,
    pub value_poly: MultilinearPolynomial<F>,
    pub eq_poly: MultilinearPolynomial<F>,
    pub scale: F,
}

impl<F: JoltField, P: PreCommittedClaimReductionParams<F>> PreCommittedPolyReductionCore<F, P> {
    pub fn new(params: P, value_poly: MultilinearPolynomial<F>, eq_poly: MultilinearPolynomial<F>) -> Self {
        Self {
            params,
            value_poly,
            eq_poly,
            scale: F::one(),
        }
    }
}

/// Parent sumcheck implementation (Rust trait-based inheritance) for pre-committed
/// polynomial reductions where prover messages are over `value_poly * eq_poly`.
pub trait PreCommittedPolyClaimReduction<F: JoltField> {
    type Params: PreCommittedClaimReductionParams<F>;

    fn precommitted_core(&self) -> &PreCommittedPolyReductionCore<F, Self::Params>;
    fn precommitted_core_mut(&mut self) -> &mut PreCommittedPolyReductionCore<F, Self::Params>;

    fn params(&self) -> &Self::Params {
        &self.precommitted_core().params
    }

    fn params_mut(&mut self) -> &mut Self::Params {
        &mut self.precommitted_core_mut().params
    }

    fn value_poly(&self) -> &MultilinearPolynomial<F> {
        &self.precommitted_core().value_poly
    }

    fn value_poly_mut(&mut self) -> &mut MultilinearPolynomial<F> {
        &mut self.precommitted_core_mut().value_poly
    }

    fn eq_poly(&self) -> &MultilinearPolynomial<F> {
        &self.precommitted_core().eq_poly
    }

    fn eq_poly_mut(&mut self) -> &mut MultilinearPolynomial<F> {
        &mut self.precommitted_core_mut().eq_poly
    }

    fn scale(&self) -> &F {
        &self.precommitted_core().scale
    }

    fn scale_mut(&mut self) -> &mut F {
        &mut self.precommitted_core_mut().scale
    }

    fn bind_aux_polys(&mut self, _r_j: F::Challenge) {}

    fn transition_to_address_phase(&mut self) {
        self.params_mut().transition_to_address_phase();
    }

    fn round_offset(&self, max_num_rounds: usize) -> usize {
        self.params().round_offset(max_num_rounds)
    }

    fn compute_message(&self, round: usize, previous_claim: F) -> UniPoly<F> {
        let params = self.params();
        if params.is_cycle_phase()
            && !params.cycle_phase_col_rounds().contains(&round)
            && !params.cycle_phase_row_rounds().contains(&round)
        {
            return UniPoly::from_coeff(vec![previous_claim * F::from_u64(2).inverse().unwrap()]);
        }

        let trailing_cap = if params.is_cycle_phase() {
            params.cycle_alignment_rounds()
        } else {
            params.address_alignment_rounds()
        };
        let num_trailing_variables =
            trailing_cap.saturating_sub(params.num_rounds_for_current_phase());
        let scaling_factor = *self.scale() * F::one().mul_pow_2(num_trailing_variables);
        let prev_unscaled = previous_claim * scaling_factor.inverse().unwrap();
        let poly_unscaled = self.compute_message_unscaled(prev_unscaled);
        poly_unscaled * scaling_factor
    }

    fn ingest_challenge(&mut self, r_j: F::Challenge, round: usize) {
        let is_cycle_phase = self.params().is_cycle_phase();
        if is_cycle_phase {
            let is_dummy_round = {
                let params = self.params();
                !params.cycle_phase_col_rounds().contains(&round)
                    && !params.cycle_phase_row_rounds().contains(&round)
            };
            if is_dummy_round {
                *self.scale_mut() *= F::from_u64(2).inverse().unwrap();
            } else {
                self.value_poly_mut()
                    .bind_parallel(r_j, BindingOrder::LowToHigh);
                self.eq_poly_mut()
                    .bind_parallel(r_j, BindingOrder::LowToHigh);
                self.bind_aux_polys(r_j);
                self.params_mut()
                    .reduction_mut()
                    .cycle_var_challenges
                    .push(r_j);
            }
            return;
        }

        self.value_poly_mut()
            .bind_parallel(r_j, BindingOrder::LowToHigh);
        self.eq_poly_mut()
            .bind_parallel(r_j, BindingOrder::LowToHigh);
        self.bind_aux_polys(r_j);
    }

    fn cycle_intermediate_claim(&self) -> F {
        let value_poly = self.value_poly();
        let eq_poly = self.eq_poly();
        let len = value_poly.len();
        debug_assert_eq!(len, eq_poly.len());

        let mut sum = F::zero();
        for i in 0..len {
            sum += value_poly.get_bound_coeff(i) * eq_poly.get_bound_coeff(i);
        }
        sum * *self.scale()
    }

    fn final_claim_if_ready(&self) -> Option<F> {
        let value_poly = self.value_poly();
        if value_poly.len() == 1 {
            Some(value_poly.final_sumcheck_claim())
        } else {
            None
        }
    }

    fn compute_message_unscaled(&self, previous_claim_unscaled: F) -> UniPoly<F> {
        const DEGREE_BOUND: usize = 2;
        let value_poly = self.value_poly();
        let eq_poly = self.eq_poly();
        let half = value_poly.len() / 2;
        let evals: [F; DEGREE_BOUND] = (0..half)
            .into_par_iter()
            .map(|j| {
                let value_evals =
                    value_poly.sumcheck_evals_array::<DEGREE_BOUND>(j, BindingOrder::LowToHigh);
                let eq_evals =
                    eq_poly.sumcheck_evals_array::<DEGREE_BOUND>(j, BindingOrder::LowToHigh);
                let mut out = [F::zero(); DEGREE_BOUND];
                for i in 0..DEGREE_BOUND {
                    out[i] = value_evals[i] * eq_evals[i];
                }
                out
            })
            .reduce(
                || [F::zero(); DEGREE_BOUND],
                |mut acc, arr| {
                    acc.iter_mut().zip(arr.iter()).for_each(|(a, b)| *a += *b);
                    acc
                },
            );
        UniPoly::from_evals_and_hint(previous_claim_unscaled, &evals)
    }
}

/// Shared prover glue for pre-committed claim-reduction sumchecks.
///
/// Implementers provide only claim-reduction-specific hooks, while round scheduling,
/// message computation, and challenge ingestion are inherited from
/// [`PreCommittedPolyClaimReduction`].
pub trait PreCommittedSumcheckInstanceProver<F: JoltField, T: Transcript>:
    PreCommittedPolyClaimReduction<F> + sealed::Sealed + Send + Sync + MaybeAllocative
where
    Self::Params: PreCommittedSumcheckInstanceParams<F>,
{
    fn precommitted_params(&self) -> &dyn SumcheckInstanceParams<F> {
        self.params()
    }

    fn precommitted_round_offset(&self, max_num_rounds: usize) -> usize {
        <Self as PreCommittedPolyClaimReduction<F>>::round_offset(self, max_num_rounds)
    }

    fn precommitted_compute_message(&mut self, round: usize, previous_claim: F) -> UniPoly<F> {
        <Self as PreCommittedPolyClaimReduction<F>>::compute_message(self, round, previous_claim)
    }

    fn precommitted_ingest_challenge(&mut self, r_j: F::Challenge, round: usize) {
        <Self as PreCommittedPolyClaimReduction<F>>::ingest_challenge(self, r_j, round);
    }

    fn precommitted_cache_openings(
        &self,
        accumulator: &mut ProverOpeningAccumulator<F>,
        transcript: &mut T,
        sumcheck_challenges: &[F::Challenge],
    );

    #[cfg(feature = "allocative")]
    fn precommitted_update_flamegraph(&self, flamegraph: &mut allocative::FlameGraphBuilder);
}

impl<F, T, P> SumcheckInstanceProver<F, T> for P
where
    F: JoltField,
    T: Transcript,
    P: PreCommittedSumcheckInstanceProver<F, T>,
    <P as PreCommittedPolyClaimReduction<F>>::Params: PreCommittedSumcheckInstanceParams<F>,
{
    fn get_params(&self) -> &dyn SumcheckInstanceParams<F> {
        <P as PreCommittedSumcheckInstanceProver<F, T>>::precommitted_params(self)
    }

    fn round_offset(&self, max_num_rounds: usize) -> usize {
        <P as PreCommittedSumcheckInstanceProver<F, T>>::precommitted_round_offset(
            self,
            max_num_rounds,
        )
    }

    fn compute_message(&mut self, round: usize, previous_claim: F) -> UniPoly<F> {
        <P as PreCommittedSumcheckInstanceProver<F, T>>::precommitted_compute_message(
            self,
            round,
            previous_claim,
        )
    }

    fn ingest_challenge(&mut self, r_j: F::Challenge, round: usize) {
        <P as PreCommittedSumcheckInstanceProver<F, T>>::precommitted_ingest_challenge(
            self, r_j, round,
        );
    }

    fn cache_openings(
        &self,
        accumulator: &mut ProverOpeningAccumulator<F>,
        transcript: &mut T,
        sumcheck_challenges: &[F::Challenge],
    ) {
        <P as PreCommittedSumcheckInstanceProver<F, T>>::precommitted_cache_openings(
            self,
            accumulator,
            transcript,
            sumcheck_challenges,
        )
    }

    #[cfg(feature = "allocative")]
    fn update_flamegraph(&self, flamegraph: &mut allocative::FlameGraphBuilder) {
        <P as PreCommittedSumcheckInstanceProver<F, T>>::precommitted_update_flamegraph(
            self, flamegraph,
        );
    }
}
