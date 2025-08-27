use std::{cell::RefCell, iter::once, rc::Rc};

use crate::{
    field::JoltField,
    poly::{
        commitment::commitment_scheme::CommitmentScheme,
        compact_polynomial::SmallScalar,
        eq_poly::EqPolynomial,
        identity_poly::IdentityPolynomial,
        multilinear_polynomial::{
            BindingOrder, MultilinearPolynomial, PolynomialBinding, PolynomialEvaluation,
        },
        opening_proof::{
            OpeningPoint, ProverOpeningAccumulator, SumcheckId, VerifierOpeningAccumulator,
            BIG_ENDIAN,
        },
    },
    subprotocols::sumcheck::SumcheckInstance,
    transcripts::Transcript,
    utils::{expanding_table::ExpandingTable, math::Math, thread::unsafe_allocate_zero_vec},
    zkvm::{
        dag::state_manager::StateManager,
        instruction::{
            CircuitFlags, InstructionFlags, InstructionLookup, InterleavedBitsMarker,
            NUM_CIRCUIT_FLAGS,
        },
        lookup_table::{LookupTables, NUM_LOOKUP_TABLES},
        witness::{CommittedPolynomial, VirtualPolynomial},
    },
};
use allocative::Allocative;
#[cfg(feature = "allocative")]
use allocative::FlameGraphBuilder;
use common::constants::{REGISTER_COUNT, XLEN};
use rayon::prelude::*;
use strum::{EnumCount, IntoEnumIterator};
use tracer::instruction::NormalizedInstruction;

/// Number of batched read-checking sumchecks bespokely
const STAGES: usize = 3;

#[derive(Allocative)]
struct ReadCheckingProverState<F: JoltField> {
    F: [MultilinearPolynomial<F>; STAGES],
    ra: Vec<MultilinearPolynomial<F>>,
    v: Vec<ExpandingTable<F>>,
    eq_polys: [MultilinearPolynomial<F>; STAGES],
    val_gamma: Option<[F; STAGES]>,
    pc: Vec<usize>,
}

#[derive(Allocative)]
pub struct ReadRafSumcheck<F: JoltField> {
    gamma: [F; STAGES],
    gamma_cub: F,
    gamma_sqr: F,
    rv_claim: F,
    log_K_chunk: usize,
    K_chunk: usize,
    log_K: usize,
    log_T: usize,
    d: usize,
    prover_state: Option<ReadCheckingProverState<F>>,
    val_polys: [MultilinearPolynomial<F>; STAGES],
    int_poly: IdentityPolynomial<F>,
    r_cycles: [Vec<F>; STAGES],

    unexpanded_pc_vals: Vec<F>,
    imm_vals: Vec<i128>,
    rd_vals: Vec<u64>,
    circuit_flags_val: [Vec<u8>; CircuitFlags::COUNT],

    gamma_powers_val1: Vec<F>,
}

#[derive(Debug, Clone, Copy)]
enum ReadCheckingValType {
    /// Spartan outer sumcheck
    Stage1,
    /// Registers read-write sumcheck
    Stage2,
    /// Registers val sumcheck wa, PCSumcheck, Instruction Lookups
    Stage3,
}

impl<F: JoltField> ReadRafSumcheck<F> {
    #[tracing::instrument(skip_all, name = "BytecodeReadRafSumcheck::new_prover")]
    pub fn new_prover<PCS: CommitmentScheme<Field = F>>(
        sm: &mut StateManager<F, impl Transcript, PCS>,
    ) -> Self {
        let K = sm.get_prover_data().0.shared.bytecode.code_size;
        let log_K = K.log_2();
        let d = sm.get_prover_data().0.shared.bytecode.d;
        let log_T = sm.get_prover_data().1.len().log_2();
        let log_K_chunk = log_K.div_ceil(d);
        let K_chunk = 1 << log_K_chunk;
        let gamma: F = sm.transcript.borrow_mut().challenge_scalar();
        let gamma_sqr = gamma.square();
        let gamma_cub = gamma_sqr * gamma;
        let gamma_four = gamma_sqr.square();
        let (val_1, rv_claim_1, r_cycle_1, gamma_powers1) = Self::compute_val_rv(sm, ReadCheckingValType::Stage1);
        let (val_2, rv_claim_2, r_cycle_2, gamma_powers2) = Self::compute_val_rv(sm, ReadCheckingValType::Stage2);
        let (val_3, rv_claim_3, r_cycle_3, gamma_powers3) = Self::compute_val_rv(sm, ReadCheckingValType::Stage3);
        let (_, raf_claim) =
            sm.get_virtual_polynomial_opening(VirtualPolynomial::PC, SumcheckId::SpartanOuter);
        let (_, raf_shift_claim) =
            sm.get_virtual_polynomial_opening(VirtualPolynomial::PC, SumcheckId::SpartanShift);
        let int_poly = IdentityPolynomial::<F>::new(log_K);
        let (preprocessing, trace, _, _) = sm.get_prover_data();

        let rv_claim = rv_claim_1
            + gamma * rv_claim_2
            + gamma_sqr * rv_claim_3
            + gamma_cub * raf_claim
            + gamma_four * raf_shift_claim;

        let eq_evals = [
            EqPolynomial::evals(&r_cycle_1),
            EqPolynomial::evals(&r_cycle_2),
            EqPolynomial::evals(&r_cycle_3),
        ];

        let T = trace.len();
        let num_chunks = rayon::current_num_threads().next_power_of_two().min(T);
        let chunk_size = (T / num_chunks).max(1);

        let F = trace
            .par_chunks(chunk_size)
            .enumerate()
            .map(|(chunk_index, trace_chunk)| {
                let mut result_1: Vec<F> = unsafe_allocate_zero_vec(K);
                let mut result_2: Vec<F> = unsafe_allocate_zero_vec(K);
                let mut result_3: Vec<F> = unsafe_allocate_zero_vec(K);
                let mut j = chunk_index * chunk_size;
                for cycle in trace_chunk {
                    let pc = preprocessing.shared.bytecode.get_pc(cycle);
                    result_1[pc] += eq_evals[0][j];
                    result_2[pc] += eq_evals[1][j];
                    result_3[pc] += eq_evals[2][j];
                    j += 1;
                }
                (result_1, result_2, result_3)
            })
            .reduce(
                || {
                    (
                        unsafe_allocate_zero_vec(K),
                        unsafe_allocate_zero_vec(K),
                        unsafe_allocate_zero_vec(K),
                    )
                },
                |(mut running_1, mut running_2, mut running_3), (new_1, new_2, new_3)| {
                    running_1
                        .par_iter_mut()
                        .zip(new_1.into_par_iter())
                        .for_each(|(x, y)| *x += y);
                    running_2
                        .par_iter_mut()
                        .zip(new_2.into_par_iter())
                        .for_each(|(x, y)| *x += y);
                    running_3
                        .par_iter_mut()
                        .zip(new_3.into_par_iter())
                        .for_each(|(x, y)| *x += y);
                    (running_1, running_2, running_3)
                },
            );

        let eq_polys = eq_evals
            .into_iter()
            .map(MultilinearPolynomial::from)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        let F = [
            MultilinearPolynomial::from(F.0),
            MultilinearPolynomial::from(F.1),
            MultilinearPolynomial::from(F.2),
        ];

        let mut v = (0..d)
            .map(|_| ExpandingTable::new(K_chunk))
            .collect::<Vec<_>>();
        v.par_iter_mut().for_each(|v| v.reset(F::one()));

        let pc = trace
            .par_iter()
            .map(|cycle| preprocessing.shared.bytecode.get_pc(cycle))
            .collect();

        // Create the val_polys - use precomputed ones from preprocessing for consistency
        let val_polys = [
            MultilinearPolynomial::from(val_1),
            MultilinearPolynomial::from(val_2),
            MultilinearPolynomial::from(val_3),
        ];

        let sumcheck = Self {
            rv_claim,
            log_K,
            log_K_chunk,
            K_chunk,
            d,
            log_T,
            prover_state: Some(ReadCheckingProverState {
                F,
                ra: Vec::with_capacity(d),
                v,
                eq_polys,
                val_gamma: None,
                pc,
            }),
            val_polys,
            int_poly,
            r_cycles: [r_cycle_1, r_cycle_2, r_cycle_3],
            gamma: [F::one(), gamma, gamma_sqr],
            gamma_sqr,
            gamma_cub,
            unexpanded_pc_vals: sm.prover_state.as_ref().unwrap().preprocessing.shared.bytecode.get_unexpanded_pc_vec(),
            imm_vals: sm.prover_state.as_ref().unwrap().preprocessing.shared.bytecode.get_imm_vec(),
            rd_vals: sm.prover_state.as_ref().unwrap().preprocessing.shared.bytecode.get_rd_vec(),
            circuit_flags_val: std::array::from_fn(|i| sm.prover_state.as_ref().unwrap().preprocessing.shared.bytecode.get_circuit_flags_vec(i)),
            gamma_powers_val1: gamma_powers1,
        };

        sumcheck
    }

    pub fn new_verifier<PCS: CommitmentScheme<Field = F>>(
        sm: &mut StateManager<F, impl Transcript, PCS>
    ) -> Self {
        let K = sm.get_verifier_data().0.shared.bytecode.code_size;
        let log_K = K.log_2();
        let d = sm.get_verifier_data().0.shared.bytecode.d;
        let log_T = sm.get_verifier_data().2.log_2();
        let log_K_chunk = log_K.div_ceil(d);
        let gamma: F = sm.transcript.borrow_mut().challenge_scalar();
        let gamma_sqr = gamma.square();
        let gamma_cub = gamma_sqr * gamma;
        let gamma_four = gamma_sqr.square();
        let (val_1, rv_claim_1, r_cycle_1, gamma_powers1) = Self::compute_val_rv(sm, ReadCheckingValType::Stage1);
        let (val_2, rv_claim_2, r_cycle_2, gamma_powers2) = Self::compute_val_rv(sm, ReadCheckingValType::Stage2);
        let (val_3, rv_claim_3, r_cycle_3, gamma_powers3) = Self::compute_val_rv(sm, ReadCheckingValType::Stage3);
        let int_poly = IdentityPolynomial::new(log_K);

        assert_eq!(r_cycle_1.len(), r_cycle_2.len());
        assert_eq!(r_cycle_1.len(), r_cycle_3.len());

        let val_polys = [
            MultilinearPolynomial::from(val_1),
            MultilinearPolynomial::from(val_2),
            MultilinearPolynomial::from(val_3),
        ];

        let (_, raf_claim) =
            sm.get_virtual_polynomial_opening(VirtualPolynomial::PC, SumcheckId::SpartanOuter);
        let (_, raf_shift_claim) =
            sm.get_virtual_polynomial_opening(VirtualPolynomial::PC, SumcheckId::SpartanShift);
        let rv_claim = rv_claim_1
            + gamma * rv_claim_2
            + gamma_sqr * rv_claim_3
            + gamma_cub * raf_claim
            + gamma_four * raf_shift_claim;

        Self {
            gamma: [F::one(), gamma, gamma_sqr],
            gamma_sqr,
            gamma_cub,
            rv_claim,
            log_K,
            log_K_chunk,
            K_chunk: 1 << log_K_chunk,
            d,
            log_T,
            prover_state: None,
            val_polys,
            int_poly,
            r_cycles: [r_cycle_1, r_cycle_2, r_cycle_3],
            unexpanded_pc_vals: sm.verifier_state.as_ref().unwrap().preprocessing.shared.bytecode.get_unexpanded_pc_vec(),
            imm_vals: sm.verifier_state.as_ref().unwrap().preprocessing.shared.bytecode.get_imm_vec(),
            rd_vals: sm.verifier_state.as_ref().unwrap().preprocessing.shared.bytecode.get_rd_vec(),
            circuit_flags_val: std::array::from_fn(|i| sm.verifier_state.as_ref().unwrap().preprocessing.shared.bytecode.get_circuit_flags_vec(i)),
            gamma_powers_val1: gamma_powers1,
        }
    }

    pub fn get_gamma_powers_deterministic(amount: usize) -> Vec<F> {
        let mut gamma_powers = vec![F::one()];
        for _ in 1..amount {
            gamma_powers.push(F::from_u128(1000) * gamma_powers.last().unwrap());
        }
        gamma_powers
    }

    fn compute_val_rv(
        sm: &mut StateManager<F, impl Transcript, impl CommitmentScheme<Field = F>>,
        val_type: ReadCheckingValType,
    ) -> (Vec<F>, F, Vec<F>, Vec<F>) {
        match val_type {
            ReadCheckingValType::Stage1 => {
                let gamma_powers = get_gamma_powers(
                    &mut *sm.get_transcript().borrow_mut(),
                    3 + NUM_CIRCUIT_FLAGS,
                );
                let (r_cycle, _) = sm.get_virtual_polynomial_opening(
                    VirtualPolynomial::Imm,
                    SumcheckId::SpartanOuter,
                );
                (
                    Self::compute_val_1(sm, &gamma_powers),
                    Self::compute_rv_claim_1(sm, &gamma_powers),
                    r_cycle.r,
                    gamma_powers,
                )
            }
            ReadCheckingValType::Stage2 => {
                let gamma_powers = get_gamma_powers(&mut *sm.get_transcript().borrow_mut(), 3);
                let (r, _) = sm.get_virtual_polynomial_opening(
                    VirtualPolynomial::Rs1Ra,
                    SumcheckId::RegistersReadWriteChecking,
                );
                let (_, r_cycle) = r.split_at((REGISTER_COUNT as usize).log_2());
                (
                    Self::compute_val_2(sm, &gamma_powers),
                    Self::compute_rv_claim_2(sm, &gamma_powers),
                    r_cycle.r,
                    gamma_powers,
                )
            }
            ReadCheckingValType::Stage3 => {
                let gamma_powers = get_gamma_powers(
                    &mut *sm.get_transcript().borrow_mut(),
                    4 + NUM_LOOKUP_TABLES,
                );
                let (r, _) = sm.get_virtual_polynomial_opening(
                    VirtualPolynomial::RdWa,
                    SumcheckId::RegistersValEvaluation,
                );
                let (_, r_cycle) = r.split_at((REGISTER_COUNT as usize).log_2());
                (
                    Self::compute_val_3(sm, &gamma_powers),
                    Self::compute_rv_claim_3(sm, &gamma_powers),
                    r_cycle.r,
                    gamma_powers,
                )
            }
        }
    }

    /// Returns a vec of evaluations:
    ///    Val(k) = unexpanded_pc(k) + gamma * imm(k)
    ///             + gamma^2 * circuit_flags[0](k) + gamma^3 * circuit_flags[1](k) + ...
    /// This particular Val virtualizes claims output by Spartan's "outer" sumcheck
    fn compute_val_1(
        sm: &mut StateManager<F, impl Transcript, impl CommitmentScheme<Field = F>>,
        gamma_powers: &[F],
    ) -> Vec<F> {
        if sm.prover_state.is_none() {
            return vec![F::zero(); sm.get_bytecode().len()]
        }
        sm.get_bytecode()
            .par_iter()
            .map(|instruction| {
                let NormalizedInstruction {
                    address: unexpanded_pc,
                    operands,
                    ..
                } = instruction.normalize();

                let mut linear_combination = F::zero();
                linear_combination += F::from_u64(unexpanded_pc as u64);
                linear_combination += operands.imm.field_mul(gamma_powers[1]);
                linear_combination += (operands.rd as u64).field_mul(gamma_powers[2]);
                for (flag, gamma_power) in instruction
                    .circuit_flags()
                    .iter()
                    .zip(gamma_powers[3..].iter())
                {
                    if *flag {
                        linear_combination += *gamma_power;
                    }
                }

                linear_combination
            })
            .collect()
    }

    fn compute_rv_claim_1(
        sm: &mut StateManager<F, impl Transcript, impl CommitmentScheme<Field = F>>,
        gamma_powers: &[F],
    ) -> F {
        let (_, unexpanded_pc_claim) = sm.get_virtual_polynomial_opening(
            VirtualPolynomial::UnexpandedPC,
            SumcheckId::SpartanOuter,
        );
        let (_, imm_claim) =
            sm.get_virtual_polynomial_opening(VirtualPolynomial::Imm, SumcheckId::SpartanOuter);
        let (_, rd_claim) =
            sm.get_virtual_polynomial_opening(VirtualPolynomial::Rd, SumcheckId::SpartanOuter);
        once(unexpanded_pc_claim)
            .chain(once(imm_claim))
            .chain(once(rd_claim))
            .chain(CircuitFlags::iter().map(|flag| {
                sm.get_virtual_polynomial_opening(
                    VirtualPolynomial::OpFlags(flag),
                    SumcheckId::SpartanOuter,
                )
                .1
            }))
            .zip(gamma_powers)
            .map(|(claim, gamma)| claim * gamma)
            .sum()
    }

    /// Returns a vec of evaluations:
    ///    Val(k) = rd(k, r_register) + gamma * rs1(k, r_register) + gamma^2 * rs2(k, r_register)
    /// where rd(k, k') = 1 if the k'th instruction in the bytecode has rd = k'
    /// and analogously for rs1(k, k') and rs2(k, k').
    /// This particular Val virtualizes claims output by the registers read/write checking sumcheck.
    fn compute_val_2(
        sm: &mut StateManager<F, impl Transcript, impl CommitmentScheme<Field = F>>,
        gamma_powers: &[F],
    ) -> Vec<F> {
        let r_register = sm
            .get_virtual_polynomial_opening(
                VirtualPolynomial::RdWa,
                SumcheckId::RegistersReadWriteChecking,
            )
            .0
            .r;
        let r_register = &r_register[..(REGISTER_COUNT as usize).log_2()];
        let eq_r_register = EqPolynomial::evals(r_register);
        debug_assert_eq!(eq_r_register.len(), REGISTER_COUNT as usize);
        sm.get_bytecode()
            .par_iter()
            .map(|instruction| {
                let instr = instruction.normalize();

                std::iter::empty()
                    .chain(once(instr.operands.rd))
                    .chain(once(instr.operands.rs1))
                    .chain(once(instr.operands.rs2))
                    .map(|r| eq_r_register[r as usize])
                    .zip(gamma_powers)
                    .map(|(claim, gamma)| claim * gamma)
                    .sum::<F>()
            })
            .collect()
    }

    fn compute_rv_claim_2(
        sm: &mut StateManager<F, impl Transcript, impl CommitmentScheme<Field = F>>,
        gamma_powers: &[F],
    ) -> F {
        std::iter::empty()
            .chain(once(VirtualPolynomial::RdWa))
            .chain(once(VirtualPolynomial::Rs1Ra))
            .chain(once(VirtualPolynomial::Rs2Ra))
            .map(|vp| {
                sm.get_virtual_polynomial_opening(vp, SumcheckId::RegistersReadWriteChecking)
                    .1
            })
            .zip(gamma_powers)
            .map(|(claim, gamma)| claim * gamma)
            .sum()
    }

    /// Returns a vec of evaluations:
    ///    Val(k) = rd(k, r_register) + gamma * unexpanded_pc(k) + gamma^2 * instr_raf_flag(k)
    ///             + gamma^3 * lookup_table_flag[0](k)
    ///             + gamma^4 * lookup_table_flag[1](k) + ...
    /// where rd(k, k') = 1 if the k'th instruction in the bytecode has rd = k'
    /// This particular Val virtualizes claims output by the PCSumcheck,
    /// the registers val-evaluation sumcheck, and the instruction lookups sumcheck.
    fn compute_val_3(
        sm: &mut StateManager<F, impl Transcript, impl CommitmentScheme<Field = F>>,
        gamma_powers: &[F],
    ) -> Vec<F> {
        let r_register = sm
            .get_virtual_polynomial_opening(
                VirtualPolynomial::RdWa,
                SumcheckId::RegistersValEvaluation,
            )
            .0
            .r;
        let r_register: Vec<_> = r_register[..(REGISTER_COUNT as usize).log_2()].to_vec();
        let eq_r_register = EqPolynomial::evals(&r_register);
        debug_assert_eq!(eq_r_register.len(), REGISTER_COUNT as usize);
        sm.get_bytecode()
            .par_iter()
            .map(|instruction| {
                let instr = instruction.normalize();
                let flags = instruction.circuit_flags();
                let unexpanded_pc = instr.address;

                let mut linear_combination: F = F::zero();

                linear_combination += eq_r_register[instr.operands.rd as usize];
                linear_combination += gamma_powers[1].mul_u64(unexpanded_pc as u64);
                if flags[CircuitFlags::IsNoop] {
                    linear_combination += gamma_powers[2];
                }
                if !flags.is_interleaved_operands() {
                    linear_combination += gamma_powers[3];
                }

                if let Some(table) = instruction.lookup_table() {
                    let table_index = LookupTables::enum_index(&table);
                    linear_combination += gamma_powers[4 + table_index];
                }

                linear_combination
            })
            .collect()
    }

    fn compute_rv_claim_3(
        sm: &mut StateManager<F, impl Transcript, impl CommitmentScheme<Field = F>>,
        gamma_powers: &[F],
    ) -> F {
        let (_, rd_wa_claim) = sm.get_virtual_polynomial_opening(
            VirtualPolynomial::RdWa,
            SumcheckId::RegistersValEvaluation,
        );
        let (_, unexpanded_pc_claim) = sm.get_virtual_polynomial_opening(
            VirtualPolynomial::UnexpandedPC,
            SumcheckId::SpartanShift,
        );
        let (_, is_noop_claim) = sm.get_virtual_polynomial_opening(
            VirtualPolynomial::OpFlags(CircuitFlags::IsNoop),
            SumcheckId::SpartanShift,
        );
        let (_, raf_flag_claim) = sm.get_virtual_polynomial_opening(
            VirtualPolynomial::InstructionRafFlag,
            SumcheckId::InstructionReadRaf,
        );
        std::iter::empty()
            .chain(once(rd_wa_claim))
            .chain(once(unexpanded_pc_claim))
            .chain(once(is_noop_claim))
            .chain(once(raf_flag_claim))
            .chain((0..LookupTables::<XLEN>::COUNT).map(|i| {
                sm.get_virtual_polynomial_opening(
                    VirtualPolynomial::LookupTableFlag(i),
                    SumcheckId::InstructionReadRaf,
                )
                .1
            }))
            .zip(gamma_powers)
            .map(|(claim, gamma)| claim * gamma)
            .sum()
    }
}

impl<F: JoltField> SumcheckInstance<F> for ReadRafSumcheck<F> {
    fn degree(&self) -> usize {
        self.d + 1
    }

    fn num_rounds(&self) -> usize {
        self.log_K + self.log_T
    }

    fn input_claim(&self) -> F {
        self.rv_claim
    }

    #[tracing::instrument(skip_all, name = "BytecodeReadRafSumcheck::compute_prover_message")]
    fn compute_prover_message(&mut self, round: usize, _previous_claim: F) -> Vec<F> {
        let ps = self
            .prover_state
            .as_ref()
            .expect("Prover state not initialized");
        if round < self.log_K {
            const DEGREE: usize = 2;

            let univariate_poly_evals: [F; DEGREE] = (0..self.val_polys[0].len() / 2)
                .into_par_iter()
                .map(|i| {
                    let ra_evals = ps.F.iter().map(|poly| {
                        poly.sumcheck_evals_array::<DEGREE>(i, BindingOrder::HighToLow)
                    });
                    let int_evals =
                        self.int_poly
                            .sumcheck_evals(i, DEGREE, BindingOrder::HighToLow);
                    // We have a separate Val polynomial for each stage
                    // Additionally, for stages 1 and 3 we have an Int polynomial for RAF
                    // So we would have:
                    // Stage 1: gamma^0 * (Val_1 + gamma^3 * Int)
                    // Stage 2: gamma^1 * (Val_2)
                    // Stage 3: gamma^2 * (Val_3 + gamma^2 * Int)
                    // Which matches with the input claim:
                    // rv_1 + gamma * rv_2 + gamma^2 * rv_3 + gamma^3 * raf_1 + gamma^4 * raf_3
                    let val_evals = self
                        .val_polys
                        .iter()
                        // Val polynomials
                        .map(|val| val.sumcheck_evals_array::<DEGREE>(i, BindingOrder::HighToLow))
                        // Here are the RAF polynomials and their powers
                        .zip([Some(&int_evals), None, Some(&int_evals)])
                        .zip([Some(self.gamma_cub), None, Some(self.gamma_sqr)])
                        .map(|((val_evals, int_evals), gamma)| {
                            std::array::from_fn::<F, DEGREE, _>(|j| {
                                val_evals[j]
                                    + int_evals.map_or(F::zero(), |int_evals| {
                                        int_evals[j] * gamma.unwrap()
                                    })
                            })
                        });

                    // Compute ra * val * gamma, and sum together
                    ra_evals
                        .zip(val_evals)
                        .zip(self.gamma.iter())
                        .map(|((ra_evals, val_evals), gamma)| {
                            std::array::from_fn(|j| ra_evals[j] * val_evals[j] * gamma)
                        })
                        .fold([F::zero(); DEGREE], |mut running, new: [F; DEGREE]| {
                            for i in 0..DEGREE {
                                running[i] += new[i];
                            }
                            running
                        })
                })
                .reduce(
                    || [F::zero(); DEGREE],
                    |mut running, new| {
                        for i in 0..DEGREE {
                            running[i] += new[i];
                        }
                        running
                    },
                );

            univariate_poly_evals.to_vec()
        } else {
            let degree = self.degree();
            (0..ps.ra[0].len() / 2)
                .into_par_iter()
                .map(|i| {
                    let eq_evals = ps
                        .eq_polys
                        .iter()
                        .map(|eq| eq.sumcheck_evals(i, degree, BindingOrder::LowToHigh));
                    let ra_evals = ps
                        .ra
                        .iter()
                        .map(|ra| ra.sumcheck_evals(i, degree, BindingOrder::LowToHigh));
                    let eq_times_val = eq_evals
                        .zip(ps.val_gamma.as_ref().unwrap().iter())
                        .map(|(eq_evals, val_evals)| {
                            eq_evals
                                .into_iter()
                                .map(|eq_eval| eq_eval * val_evals)
                                .collect()
                        })
                        .fold(
                            vec![F::zero(); degree],
                            |mut running: Vec<F>, new: Vec<F>| {
                                for i in 0..degree {
                                    running[i] += new[i];
                                }
                                running
                            },
                        );

                    ra_evals.fold(eq_times_val, |mut running: Vec<F>, new: Vec<F>| {
                        for i in 0..degree {
                            running[i] *= new[i];
                        }
                        running
                    })
                })
                .reduce(
                    || vec![F::zero(); degree],
                    |mut running, new| {
                        for i in 0..degree {
                            running[i] += new[i];
                        }
                        running
                    },
                )
        }
    }

    #[tracing::instrument(skip_all, name = "BytecodeReadRafSumcheck::bind")]
    fn bind(&mut self, r_j: F, round: usize) {
        let ps = self
            .prover_state
            .as_mut()
            .expect("Prover state not initialized");

        if round < self.log_K {
            rayon::scope(|s| {
                s.spawn(|_| {
                    self.val_polys
                        .par_iter_mut()
                        .for_each(|poly| poly.bind_parallel(r_j, BindingOrder::HighToLow))
                });
                s.spawn(|_| {
                    self.int_poly.bind_parallel(r_j, BindingOrder::HighToLow);
                });
                s.spawn(|_| {
                    ps.F.par_iter_mut()
                        .for_each(|poly| poly.bind_parallel(r_j, BindingOrder::HighToLow));
                });
                s.spawn(|_| {
                    ps.v[round / self.log_K_chunk].update(r_j);
                });
            });
            if round == self.log_K - 1 {
                self.init_log_t_rounds();
            }
        } else {
            ps.ra
                .par_iter_mut()
                .chain(ps.eq_polys.par_iter_mut())
                .for_each(|poly| poly.bind_parallel(r_j, BindingOrder::LowToHigh));
        }
    }

    fn expected_output_claim(
        &self,
        accumulator: Option<Rc<RefCell<VerifierOpeningAccumulator<F>>>>,
        r: &[F],
    ) -> F {
        let (r_address_prime, r_cycle_prime) = r.split_at(self.log_K);
        // r_cycle is bound LowToHigh, so reverse
        let r_cycle_prime = r_cycle_prime.iter().rev().copied().collect::<Vec<F>>();

        let int_poly = self.int_poly.evaluate(r_address_prime);

        let acc = accumulator.as_ref().unwrap();
        let ra_claims = (0..self.d).map(|i| {
            acc.borrow()
                .get_committed_polynomial_opening(
                    CommittedPolynomial::BytecodeRa(i),
                    SumcheckId::BytecodeReadRaf,
                )
                .1
        });

        let unexpanded_pc_claim =
            acc.borrow()
                .get_committed_polynomial_opening(
        CommittedPolynomial::UnexpandedPC,
        SumcheckId::BytecodeReadRaf,
                )
                .1;
        let imm_claim =
            acc.borrow()
                .get_committed_polynomial_opening(
        CommittedPolynomial::Imm,
        SumcheckId::BytecodeReadRaf,
                )
                .1;
        let rd_claim =
            acc.borrow()
                .get_committed_polynomial_opening(
        CommittedPolynomial::Rd,
        SumcheckId::BytecodeReadRaf,
                )
                .1;
        let circuit_claim = (0..CircuitFlags::COUNT).map(|i| {
                acc.borrow()
                    .get_committed_polynomial_opening(
                        CommittedPolynomial::CircuitFlags(i),
                        SumcheckId::BytecodeReadRaf,
                    )
                    .1
            });

        let mut linear_combination = F::zero();
        linear_combination += unexpanded_pc_claim;
        linear_combination += imm_claim.mul(self.gamma_powers_val1[1]);
        linear_combination += rd_claim.mul(self.gamma_powers_val1[2]);
        for (flag, gamma_power) in circuit_claim
            .zip(self.gamma_powers_val1[3..].iter())
        {
                linear_combination += flag.mul(gamma_power);
        }
        println!("Random Linear Combination: {}", linear_combination);
        // println!("Random Linear Combination Original: {}", val_stage_claims[0]);
        // We have a separate Val polynomial for each stage
        // Additionally, for stages 1 and 3 we have an Int polynomial for RAF
        // So we would have:
        // Stage 1: gamma^0 * (Val_1 + gamma^3 * Int)
        // Stage 2: gamma^1 * (Val_2)
        // Stage 3: gamma^2 * (Val_3 + gamma^2 * Int)
        // Which matches with the input claim:
        // rv_1 + gamma * rv_2 + gamma^2 * rv_3 + gamma^3 * raf_1 + gamma^4 * raf_3
        // Use linear_combination for Stage 1 (val_polys[0]), keep current logic for the rest
        let stage1_term = (linear_combination + int_poly * self.gamma_cub)
            * EqPolynomial::mle(&self.r_cycles[0], &r_cycle_prime)
            * self.gamma[0];

        let val_rest: F = self
            .val_polys[1..]
            .iter()
            .zip(self.r_cycles[1..].iter())
            .zip(self.gamma[1..].iter())
            .zip([
                F::zero(),                 // Stage 2 RAF term
                int_poly * self.gamma_sqr, // Stage 3 RAF term
            ])
            .map(|(((val, r_cycle), gamma), int_term)| {
                (val.evaluate(r_address_prime) + int_term)
                    * EqPolynomial::mle(r_cycle, &r_cycle_prime)
                    * gamma
            })
            .sum::<F>();

        let val = stage1_term + val_rest;
        println!("val.evaluate(r_address: {}", self.val_polys[0].evaluate(r_address_prime));
        // println!("Openning Point: {}", val_stage_claims[0]);

        ra_claims.fold(val, |running, ra_claim| running * ra_claim)
    }

    fn normalize_opening_point(&self, opening_point: &[F]) -> OpeningPoint<BIG_ENDIAN, F> {
        let mut r = opening_point.to_vec();
        r[self.log_K..].reverse();
        OpeningPoint::new(r)
    }

    fn cache_openings_prover(
        &self,
        accumulator: Rc<RefCell<ProverOpeningAccumulator<F>>>,
        opening_point: OpeningPoint<BIG_ENDIAN, F>,
    ) {
        let ps = self
            .prover_state
            .as_ref()
            .expect("Prover state not initialized");
        let (r_address, r_cycle) = opening_point.clone().split_at(self.log_K);

        // Cache openings for ra polynomials
        for i in 0..self.d {
            let r_address = &r_address.r[self.log_K_chunk * i..self.log_K_chunk * (i + 1)];
            accumulator.borrow_mut().append_sparse(
                vec![CommittedPolynomial::BytecodeRa(i)],
                SumcheckId::BytecodeReadRaf,
                r_address.to_vec(),
                r_cycle.clone().into(),
                vec![ps.ra[i].final_sumcheck_claim()],
            );
        }

        // Append dense openings for val polynomials at r_address
        // Use the bound polynomial's final claim to avoid recomputation mismatches
        // for (i, val_poly) in self.val_polys.iter().enumerate() {
        //     let val_claim = val_poly.final_sumcheck_claim();
        //     accumulator.borrow_mut().append_dense(
        //         vec![CommittedPolynomial::BytecodeVal(i)],
        //         SumcheckId::BytecodeReadRaf,
        //         r_address.r.clone(),
        //         &[val_claim],
        //     );
        //     break;
        // }

        let unexpanded_pc_poly = MultilinearPolynomial::<F>::from(self.unexpanded_pc_vals.clone());
        let unexpanded_pc_eval = unexpanded_pc_poly.evaluate(&r_address.r);
        accumulator.borrow_mut().append_dense(
            vec![CommittedPolynomial::UnexpandedPC],
            SumcheckId::BytecodeReadRaf,
            r_address.r.clone(),
            &[unexpanded_pc_eval],
        );
        let imm_poly = MultilinearPolynomial::<F>::from(self.imm_vals.clone());
        let imm_eval = imm_poly.evaluate(&r_address.r);
        accumulator.borrow_mut().append_dense(
            vec![CommittedPolynomial::Imm],
            SumcheckId::BytecodeReadRaf,
            r_address.r.clone(),
            &[imm_eval],
        );
        let rd_poly = MultilinearPolynomial::<F>::from(self.rd_vals.clone());
        let rd_eval = rd_poly.evaluate(&r_address.r);
        accumulator.borrow_mut().append_dense(
            vec![CommittedPolynomial::Rd],
            SumcheckId::BytecodeReadRaf,
            r_address.r.clone(),
            &[rd_eval],
        );
        for i in 0..CircuitFlags::COUNT {
            let circut_poly = MultilinearPolynomial::<F>::from(self.circuit_flags_val[i].clone());
            let circuit_eval = circut_poly.evaluate(&r_address.r);
            accumulator.borrow_mut().append_dense(
                vec![CommittedPolynomial::CircuitFlags(i)],
                SumcheckId::BytecodeReadRaf,
                r_address.r.clone(),
                &[circuit_eval],
            );
        }

        // Batch open UnexpandedPC, Imm, Rd, and all CircuitFlags at r_address
        {
            let mut polynomials = Vec::with_capacity(3 + CircuitFlags::COUNT);
            polynomials.push(CommittedPolynomial::UnexpandedPC);
            polynomials.push(CommittedPolynomial::Imm);
            polynomials.push(CommittedPolynomial::Rd);
            for i in 0..CircuitFlags::COUNT {
                polynomials.push(CommittedPolynomial::CircuitFlags(i));
            }

            // Compute claims at the same opening point
            let mut claims: Vec<F> = Vec::with_capacity(polynomials.len());
            let unexpanded_pc_eval = MultilinearPolynomial::<F>::from(self.unexpanded_pc_vals.clone())
                .evaluate(&r_address.r);
            let imm_eval = MultilinearPolynomial::<F>::from(self.imm_vals.clone())
                .evaluate(&r_address.r);
            let rd_eval = MultilinearPolynomial::<F>::from(self.rd_vals.clone())
                .evaluate(&r_address.r);
            claims.push(unexpanded_pc_eval);
            claims.push(imm_eval);
            claims.push(rd_eval);
            for i in 0..CircuitFlags::COUNT {
                let circuit_eval = MultilinearPolynomial::<F>::from(self.circuit_flags_val[i].clone())
                    .evaluate(&r_address.r);
                claims.push(circuit_eval);
            }

            accumulator.borrow_mut().append_dense(
                polynomials,
                SumcheckId::BytecodeReadRaf,
                r_address.r.clone(),
                &claims,
            );
        }
    }

    fn cache_openings_verifier(
        &self,
        accumulator: Rc<RefCell<VerifierOpeningAccumulator<F>>>,
        opening_point: OpeningPoint<BIG_ENDIAN, F>,
    ) {
        let (r_address, r_cycle) = opening_point.split_at(self.log_K);

        // Cache openings for ra polynomials
        (0..self.d).for_each(|i| {
            let r_address = &r_address.r[self.log_K_chunk * i..self.log_K_chunk * (i + 1)];
            accumulator.borrow_mut().append_sparse(
                vec![CommittedPolynomial::BytecodeRa(i)],
                SumcheckId::BytecodeReadRaf,
                [r_address, &r_cycle.r].concat(),
            );
        });
        accumulator.borrow_mut().append_dense(
            vec![CommittedPolynomial::UnexpandedPC],
            SumcheckId::BytecodeReadRaf,
            r_address.r.clone(),
        );
        accumulator.borrow_mut().append_dense(
            vec![CommittedPolynomial::Imm],
            SumcheckId::BytecodeReadRaf,
            r_address.r.clone(),
        );
        accumulator.borrow_mut().append_dense(
            vec![CommittedPolynomial::Rd],
            SumcheckId::BytecodeReadRaf,
            r_address.r.clone(),
        );
        for i in 0..CircuitFlags::COUNT {
            accumulator.borrow_mut().append_dense(
                vec![CommittedPolynomial::CircuitFlags(i)],
                SumcheckId::BytecodeReadRaf,
                r_address.r.clone(),
            );
        }

        // Batch open UnexpandedPC, Imm, Rd, and all CircuitFlags at r_address
        {
            let mut polynomials = Vec::with_capacity(3 + CircuitFlags::COUNT);
            polynomials.push(CommittedPolynomial::UnexpandedPC);
            polynomials.push(CommittedPolynomial::Imm);
            polynomials.push(CommittedPolynomial::Rd);
            for i in 0..CircuitFlags::COUNT {
                polynomials.push(CommittedPolynomial::CircuitFlags(i));
            }
            accumulator.borrow_mut().append_dense(
                polynomials,
                SumcheckId::BytecodeReadRaf,
                r_address.r.clone(),
            );
        }
    }

    #[cfg(feature = "allocative")]
    fn update_flamegraph(&self, flamegraph: &mut FlameGraphBuilder) {
        flamegraph.visit_root(self);
    }
}

impl<F: JoltField> ReadRafSumcheck<F> {
    fn init_log_t_rounds(&mut self) {
        let ps = self
            .prover_state
            .as_mut()
            .expect("Prover state not initialized");
        let int_poly = self.int_poly.final_sumcheck_claim();

        // We have a separate Val polynomial for each stage
        // Additionally, for stages 1 and 3 we have an Int polynomial for RAF
        // So we would have:
        // Stage 1: gamma^0 * (Val_1 + gamma^3 * Int)
        // Stage 2: gamma^1 * (Val_2)
        // Stage 3: gamma^2 * (Val_3 + gamma^2 * Int)
        // Which matches with the input claim:
        // rv_1 + gamma * rv_2 + gamma^2 * rv_3 + gamma^3 * raf_1 + gamma^4 * raf_3
        ps.val_gamma = Some(
            self.val_polys
                .iter()
                .zip(self.gamma.iter())
                .zip([
                    int_poly * self.gamma_cub,
                    F::zero(),
                    int_poly * self.gamma_sqr,
                ])
                .map(|((poly, gamma), int_poly)| (poly.final_sumcheck_claim() + int_poly) * gamma)
                .collect::<Vec<F>>()
                .try_into()
                .unwrap(),
        );

        ps.v.par_iter()
            .enumerate()
            .map(|(i, v)| {
                let ra_i: Vec<F> = ps
                    .pc
                    .par_iter()
                    .map(|k| {
                        let k = (k >> (self.log_K_chunk * (self.d - i - 1))) % self.K_chunk;
                        v[k]
                    })
                    .collect();
                MultilinearPolynomial::from(ra_i)
            })
            .collect::<Vec<_>>()
            .into_iter()
            .for_each(|ra| {
                ps.ra.push(ra);
            });
    }
}

fn get_gamma_powers<F: JoltField>(transcript: &mut impl Transcript, amount: usize) -> Vec<F> {
    let mut gamma_powers = vec![F::one()];
    let gamma: F = transcript.challenge_scalar();
    for _ in 1..amount {
        gamma_powers.push(gamma * gamma_powers.last().unwrap());
    }
    gamma_powers
}
