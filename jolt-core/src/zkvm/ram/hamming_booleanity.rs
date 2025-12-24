use std::marker::PhantomData;

use crate::field::JoltField;
use crate::poly::eq_poly::EqPolynomial;
use crate::poly::multilinear_polynomial::{BindingOrder, MultilinearPolynomial, PolynomialBinding};
use crate::poly::opening_proof::{
    OpeningAccumulator, OpeningPoint, ProverOpeningAccumulator, SumcheckId,
    VerifierOpeningAccumulator, BIG_ENDIAN, LITTLE_ENDIAN,
};
use crate::poly::split_eq_poly::GruenSplitEqPolynomial;
use crate::poly::unipoly::UniPoly;
use crate::subprotocols::split_sumcheck_prover::{SplitSumcheckInstance, SplitSumcheckInstanceInner};
use crate::subprotocols::sumcheck_prover::SumcheckInstanceProver;
use crate::subprotocols::sumcheck_verifier::{SumcheckInstanceParams, SumcheckInstanceVerifier};
use crate::transcripts::Transcript;
use crate::zkvm::witness::VirtualPolynomial;
use allocative::Allocative;
#[cfg(feature = "allocative")]
use allocative::FlameGraphBuilder;
use rayon::prelude::*;
use tracer::instruction::Cycle;

/// Helper to extract evaluations from a MultilinearPolynomial as a Vec<F>
fn multilinear_to_evals<F: JoltField>(poly: &MultilinearPolynomial<F>) -> Vec<F> {
    (0..poly.len()).map(|i| poly.get_bound_coeff(i)).collect()
}

// RAM Hamming booleanity sumcheck
//
// Proves a zero-check of the form
//   0 = Σ_j eq(r_cycle, j) · (H(j)^2 − H(j))
// where:
// - r_cycle are the time/cycle variables bound in this sumcheck
// - H(j) is an indicator of whether a RAM access occurred at cycle j (1 if address != 0, 0 otherwise)

/// Degree bound of the sumcheck round polynomials in [`HammingBooleanitySumcheckVerifier`].
const DEGREE_BOUND: usize = 3;

#[derive(Allocative)]
pub struct HammingBooleanitySumcheckParams<F: JoltField> {
    pub r_cycle: OpeningPoint<BIG_ENDIAN, F>,
}

impl<F: JoltField> HammingBooleanitySumcheckParams<F> {
    pub fn new(opening_accumulator: &dyn OpeningAccumulator<F>) -> Self {
        let (r_cycle, _) = opening_accumulator.get_virtual_polynomial_opening(
            VirtualPolynomial::LookupOutput,
            SumcheckId::SpartanOuter,
        );

        Self { r_cycle }
    }
}

impl<F: JoltField> SumcheckInstanceParams<F> for HammingBooleanitySumcheckParams<F> {
    fn degree(&self) -> usize {
        DEGREE_BOUND
    }

    fn num_rounds(&self) -> usize {
        self.r_cycle.len()
    }

    fn input_claim(&self, _: &dyn OpeningAccumulator<F>) -> F {
        F::zero()
    }

    fn normalize_opening_point(
        &self,
        challenges: &[<F as JoltField>::Challenge],
    ) -> OpeningPoint<BIG_ENDIAN, F> {
        OpeningPoint::<LITTLE_ENDIAN, F>::new(challenges.to_vec()).match_endianness()
    }
}

#[derive(Allocative)]
pub struct HammingBooleanitySumcheckProver<F: JoltField, T: Transcript> {
    eq_r_cycle: GruenSplitEqPolynomial<F>,
    H: MultilinearPolynomial<F>,
    pub params: HammingBooleanitySumcheckParams<F>,
    _phantom: PhantomData<T>,
}

impl<F: JoltField, T: Transcript> HammingBooleanitySumcheckProver<F, T> {
    #[tracing::instrument(skip_all, name = "RamHammingBooleanitySumcheckProver::initialize")]
    pub fn initialize(params: HammingBooleanitySumcheckParams<F>, trace: &[Cycle]) -> Self {
        let H = trace
            .par_iter()
            .map(|cycle| cycle.ram_access().address() != 0)
            .collect::<Vec<bool>>();
        let H = MultilinearPolynomial::from(H);

        let eq_r_cycle = GruenSplitEqPolynomial::new(&params.r_cycle.r, BindingOrder::LowToHigh);

        Self {
            eq_r_cycle,
            H,
            params,
            _phantom: PhantomData,
        }
    }
}

impl<F: JoltField, T: Transcript> SumcheckInstanceProver<F, T>
    for HammingBooleanitySumcheckProver<F, T>
{
    fn get_params(&self) -> &dyn SumcheckInstanceParams<F> {
        &self.params
    }

    #[tracing::instrument(skip_all, name = "RamHammingBooleanitySumcheckProver::compute_message")]
    fn compute_message(&mut self, _round: usize, previous_claim: F) -> UniPoly<F> {
        let eq = &self.eq_r_cycle;
        let H = &self.H;

        // Accumulate constant (c0) and quadratic (e) coefficients via generic split-eq fold.
        let [c0, e] = eq.par_fold_out_in_unreduced::<9, 2>(&|g| {
            let h0 = H.get_bound_coeff(2 * g);
            let h1 = H.get_bound_coeff(2 * g + 1);
            let delta = h1 - h0;
            [h0.square() - h0, delta.square()]
        });
        eq.gruen_poly_deg_3(c0, e, previous_claim)
    }

    #[tracing::instrument(
        skip_all,
        name = "RamHammingBooleanitySumcheckProver::ingest_challenge"
    )]
    fn ingest_challenge(&mut self, r_j: F::Challenge, _round: usize) {
        self.eq_r_cycle.bind(r_j);
        self.H.bind_parallel(r_j, BindingOrder::LowToHigh);
    }

    fn cache_openings(
        &self,
        accumulator: &mut ProverOpeningAccumulator<F>,
        transcript: &mut T,
        sumcheck_challenges: &[F::Challenge],
    ) {
        // Delegate to cache_openings_with_claims with the claim from self.H
        let h_claim = self.H.final_sumcheck_claim();
        self.cache_openings_impl(accumulator, transcript, sumcheck_challenges, h_claim);
    }
}

impl<F: JoltField, T: Transcript> HammingBooleanitySumcheckProver<F, T> {
    /// Shared implementation for cache_openings that takes the H claim as a parameter.
    fn cache_openings_impl(
        &self,
        accumulator: &mut ProverOpeningAccumulator<F>,
        transcript: &mut T,
        sumcheck_challenges: &[F::Challenge],
        h_claim: F,
    ) {
        accumulator.append_virtual(
            transcript,
            VirtualPolynomial::RamHammingWeight,
            SumcheckId::RamHammingBooleanity,
            self.params.normalize_opening_point(sumcheck_challenges),
            h_claim,
        );
    }
}

impl<F: JoltField, T: Transcript> SplitSumcheckInstanceInner<F, T, HammingBooleanitySumcheckParams<F>>
    for HammingBooleanitySumcheckProver<F, T>
{

    /// Inverse of `create_remainder`: reconstructs `self.eq_r_cycle` and `self.H`
    /// from the remainder polynomials so we can continue the sumcheck from `round_number`.
    ///
    /// `create_remainder` produces:
    ///   - `remainder[0]` = `self.eq_r_cycle.merge().Z`
    ///   - `remainder[1]` = `multilinear_to_evals(&self.H)`
    ///
    /// This function reconstructs the internal state from those evaluations.
    fn initialize_lower_rounds(params: HammingBooleanitySumcheckParams<F>, remainder: Vec<Vec<F>>, round_number: usize) -> Self {
        assert_eq!(remainder.len(), 2, "Expected 2 polynomials: eq and H");

        // Take ownership of remainder vectors without cloning
        let mut iter = remainder.into_iter();
        let eq_evals = iter.next().unwrap();
        let h_evals = iter.next().unwrap();

        assert_eq!(
            eq_evals.len(),
            h_evals.len(),
            "eq and H must have same length"
        );

        let remaining_vars = params.r_cycle.len() - round_number;

        // Inverse of `self.eq_r_cycle.merge().Z`:
        // The sum of eq evaluations equals current_scalar (since unscaled eq sums to 1)
        let current_scalar: F = eq_evals.iter().sum();

        // Recreate GruenSplitEqPolynomial using only the first `remaining_vars` challenges
        // from params.r_cycle.r (which correspond to the remaining unbound variables)
        let w = &params.r_cycle.r[..remaining_vars];
        let eq_r_cycle = GruenSplitEqPolynomial::new_with_scaling(
            w,
            BindingOrder::LowToHigh,
            Some(current_scalar),
        );

        // Inverse of `multilinear_to_evals(&self.H)`:
        // Create a DensePolynomial (LargeScalars variant) from the evaluations
        let H = MultilinearPolynomial::from(h_evals);

        HammingBooleanitySumcheckProver {
            eq_r_cycle,
            H,
            params,
            _phantom: PhantomData,
        }
    }

    fn create_remainder(&self) -> Vec<Vec<F>> {
        vec![
            self.eq_r_cycle.merge().Z,
            multilinear_to_evals(&self.H),
        ]
    }

    fn create_expr(&self) -> Box<dyn Fn(&[F]) -> F + Send + Sync> {
        Box::new(|vals: &[F]| {
            let eq = vals[0];
            let h = vals[1];
            eq * (h.square() - h)
        })
    }

    fn cache_openings_with_claims(
        &self,
        accumulator: &mut ProverOpeningAccumulator<F>,
        transcript: &mut T,
        sumcheck_challenges: &[F::Challenge],
        poly_claims: &[F],
    ) {
        // poly_claims[0] is eq claim (not needed for openings)
        // poly_claims[1] is H claim
        let h_claim = poly_claims[1];
        self.cache_openings_impl(accumulator, transcript, sumcheck_challenges, h_claim);
    }
}

pub struct HammingBooleanitySumcheckVerifier<F: JoltField> {
    params: HammingBooleanitySumcheckParams<F>,
}

impl<F: JoltField> HammingBooleanitySumcheckVerifier<F> {
    pub fn new(opening_accumulator: &dyn OpeningAccumulator<F>) -> Self {
        Self {
            params: HammingBooleanitySumcheckParams::new(opening_accumulator),
        }
    }
}

impl<F: JoltField, T: Transcript> SumcheckInstanceVerifier<F, T>
    for HammingBooleanitySumcheckVerifier<F>
{
    fn get_params(&self) -> &dyn SumcheckInstanceParams<F> {
        &self.params
    }

    fn expected_output_claim(
        &self,
        accumulator: &VerifierOpeningAccumulator<F>,
        sumcheck_challenges: &[F::Challenge],
    ) -> F {
        let H_claim = accumulator
            .get_virtual_polynomial_opening(
                VirtualPolynomial::RamHammingWeight,
                SumcheckId::RamHammingBooleanity,
            )
            .1;

        let (r_cycle, _) = accumulator.get_virtual_polynomial_opening(
            VirtualPolynomial::LookupOutput,
            SumcheckId::SpartanOuter,
        );

        let eq = EqPolynomial::<F>::mle(
            sumcheck_challenges,
            &r_cycle
                .r
                .iter()
                .cloned()
                .rev()
                .collect::<Vec<F::Challenge>>(),
        );

        (H_claim.square() - H_claim) * eq
    }

    fn cache_openings(
        &self,
        accumulator: &mut VerifierOpeningAccumulator<F>,
        transcript: &mut T,
        sumcheck_challenges: &[F::Challenge],
    ) {
        accumulator.append_virtual(
            transcript,
            VirtualPolynomial::RamHammingWeight,
            SumcheckId::RamHammingBooleanity,
            self.params.normalize_opening_point(sumcheck_challenges),
        );
    }
}
