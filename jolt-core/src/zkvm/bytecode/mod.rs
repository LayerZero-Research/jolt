use std::collections::BTreeMap;

use crate::poly::compact_polynomial::SmallScalar;
use crate::poly::opening_proof::SumcheckId;
use crate::utils::math::Math;
#[cfg(feature = "allocative")]
use crate::utils::profiling::print_data_structure_heap_usage;
use crate::zkvm::bytecode::booleanity::BooleanitySumcheck;
use crate::zkvm::bytecode::hamming_weight::HammingWeightSumcheck;
use crate::zkvm::bytecode::read_raf_checking::ReadRafSumcheck;
use crate::zkvm::dag::stage::SumcheckStages;
use crate::zkvm::dag::state_manager::StateManager;
use crate::zkvm::instruction::{InstructionFlags, NUM_CIRCUIT_FLAGS};
use crate::zkvm::witness::{compute_d_parameter, VirtualPolynomial, DTH_ROOT_OF_K};
use crate::{
    field::JoltField,
    poly::{commitment::commitment_scheme::CommitmentScheme, eq_poly::EqPolynomial},
    subprotocols::sumcheck::SumcheckInstance,
    transcripts::Transcript,
    utils::thread::unsafe_allocate_zero_vec,
};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use common::constants::{ALIGNMENT_FACTOR_BYTECODE, RAM_START_ADDRESS};
use rayon::prelude::*;
use tracer::instruction::{NormalizedInstruction, RV32IMCycle, RV32IMInstruction};

pub mod booleanity;
pub mod hamming_weight;
pub mod read_raf_checking;

#[derive(Debug, Clone, CanonicalSerialize, CanonicalDeserialize)]
pub struct BytecodePreprocessing<F>
where F: JoltField {
    pub code_size: usize,
    pub bytecode: Vec<RV32IMInstruction>,
    /// Maps the memory address of each instruction in the bytecode to its "virtual" address.
    /// See Section 6.1 of the Jolt paper, "Reflecting the program counter". The virtual address
    /// is the one used to keep track of the next (potentially virtual) instruction to execute.
    /// Key: (ELF address, inline sequence index or 0)
    pub virtual_address_map: BTreeMap<(usize, u16), usize>,
    pub d: usize,
    pub val_1: Vec<F>,
    pub val_2: Vec<F>,
    pub val_3: Vec<F>,
}


// OMID"s change
/// Returns a vec of evaluations:
    // ///    Val(k) = unexpanded_pc(k) + gamma * imm(k)
    // ///             + gamma^2 * circuit_flags[0](k) + gamma^3 * circuit_flags[1](k) + ...
    // /// This particular Val virtualizes claims output by Spartan's "outer" sumcheck
    // fn compute_val_1(
    //     sm: &mut StateManager<F, impl Transcript, impl CommitmentScheme<Field = F>>,
    //     gamma_powers: &[F],
    // ) -> Vec<F> {
    //     sm.get_bytecode()
    //         .par_iter()
    //         .map(|instruction| {
    //             let NormalizedInstruction {
    //                 address: unexpanded_pc,
    //                 operands,
    //                 ..
    //             } = instruction.normalize();

    //             let mut linear_combination = F::zero();
    //             linear_combination += F::from_u64(unexpanded_pc as u64);
    //             linear_combination += operands.imm.field_mul(gamma_powers[1]);
    //             linear_combination += (operands.rd as u64).field_mul(gamma_powers[2]);
    //             for (flag, gamma_power) in instruction
    //                 .circuit_flags()
    //                 .iter()
    //                 .zip(gamma_powers[3..].iter())
    //             {
    //                 if *flag {
    //                     linear_combination += *gamma_power;
    //                 }
    //             }

    //             linear_combination
    //         })
    //         .collect()
    // }



impl<F: JoltField> BytecodePreprocessing<F> {
    #[tracing::instrument(skip_all, name = "BytecodePreprocessing::preprocess")]
    pub fn preprocess(mut bytecode: Vec<RV32IMInstruction>) -> Self {
        let mut virtual_address_map = BTreeMap::new();
        let mut virtual_address = 1; // Account for no-op instruction prepended to bytecode
        for instruction in bytecode.iter() {
            if instruction.normalize().address == 0 {
                virtual_address += 1;
                // ignore unimplemented instructions
                continue;
            }
            let instr = instruction.normalize();
            debug_assert!(instr.address >= RAM_START_ADDRESS as usize);
            debug_assert!(instr.address.is_multiple_of(ALIGNMENT_FACTOR_BYTECODE));
            assert_eq!(
                virtual_address_map.insert(
                    (instr.address, instr.inline_sequence_remaining.unwrap_or(0)),
                    virtual_address
                ),
                None,
                "Virtual address map already contains entry for address: {:#X}, inline sequence: {:?}. map size: {}",
                instr.address, instr.inline_sequence_remaining, virtual_address_map.len());
            virtual_address += 1;
        }

        // Bytecode: Prepend a single no-op instruction
        bytecode.insert(0, RV32IMInstruction::NoOp);
        assert_eq!(virtual_address_map.insert((0, 0), 0), None);

        let d = compute_d_parameter(bytecode.len().next_power_of_two());
        // Make log(code_size) a multiple of d
        let code_size = (bytecode.len().next_power_of_two().log_2().div_ceil(d) * d)
            .pow2()
            .max(DTH_ROOT_OF_K);

        // Bytecode: Pad to nearest power of 2
        bytecode.resize(code_size, RV32IMInstruction::NoOp);
        let val_1 = Self::compute_val_1(&bytecode, &Self::get_gamma_powers_deterministic(3 + NUM_CIRCUIT_FLAGS));

        Self {
            code_size,
            bytecode,
            virtual_address_map,
            d,
            val_1,
            val_2: Self::compute_val_2(),
            val_3: Self::compute_val_3(),
        }
    }

    pub fn get_pc(&self, cycle: &RV32IMCycle) -> usize {
        if matches!(cycle, tracer::instruction::RV32IMCycle::NoOp) {
            return 0;
        }
        let instr = cycle.instruction().normalize();
        *self
            .virtual_address_map
            .get(&(instr.address, instr.inline_sequence_remaining.unwrap_or(0)))
            .unwrap()
    }

    fn compute_val_1(bytecode: &[RV32IMInstruction], gamma_powers: &[F]) -> Vec<F> {
        bytecode
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

    fn compute_val_2() -> Vec<F> {
        Vec::new()
    }

    fn compute_val_3() -> Vec<F> {
        Vec::new()
    }

    pub fn get_gamma_powers_deterministic(amount: usize) -> Vec<F> {
        let mut gamma_powers = vec![F::one()];
        for _ in 1..amount {
            gamma_powers.push(F::from_u128(1000) * gamma_powers.last().unwrap());
        }
        gamma_powers
    }
}

#[derive(Default)]
pub struct BytecodeDag {}

impl<F: JoltField, PCS: CommitmentScheme<Field = F>, T: Transcript> SumcheckStages<F, T, PCS>
    for BytecodeDag
{
    fn stage4_prover_instances(
        &mut self,
        sm: &mut StateManager<'_, F, T, PCS>,
    ) -> Vec<Box<dyn SumcheckInstance<F>>> {
        let (preprocessing, trace, _, _) = sm.get_prover_data();
        let bytecode_preprocessing = &preprocessing.shared.bytecode;

        let r_cycle: Vec<F> = sm
            .get_virtual_polynomial_opening(
                VirtualPolynomial::UnexpandedPC,
                SumcheckId::SpartanOuter,
            )
            .0
            .r;
        let E_1: Vec<F> = EqPolynomial::evals(&r_cycle);

        let F_1 = compute_ra_evals(bytecode_preprocessing, trace, &E_1);

        let read_raf = ReadRafSumcheck::new_prover(sm);
        let booleanity = BooleanitySumcheck::new_prover(sm, E_1, F_1.clone());
        let hamming_weight = HammingWeightSumcheck::new_prover(sm, F_1);
        
        // Store _val_commitments for later verification
        // TODO: Pass these commitments to the verifier when needed

        #[cfg(feature = "allocative")]
        {
            print_data_structure_heap_usage("Bytecode ReadRafSumcheck", &read_raf);
            print_data_structure_heap_usage("Bytecode BooleanitySumcheck", &booleanity);
            print_data_structure_heap_usage("Bytecode HammingWeightSumcheck", &hamming_weight);
        }

        vec![
            Box::new(read_raf),
            Box::new(booleanity),
            Box::new(hamming_weight),
        ]
    }

    fn stage4_verifier_instances(
        &mut self,
        sm: &mut StateManager<'_, F, T, PCS>,
    ) -> Vec<Box<dyn SumcheckInstance<F>>> {        
        let read_checking = ReadRafSumcheck::new_verifier(sm);
        let booleanity = BooleanitySumcheck::new_verifier(sm);
        let hamming_weight = HammingWeightSumcheck::new_verifier(sm);

        vec![
            Box::new(read_checking),
            Box::new(booleanity),
            Box::new(hamming_weight),
        ]
    }
}

#[inline(always)]
#[tracing::instrument(skip_all, name = "Bytecode::compute_ra_evals")]
fn compute_ra_evals<F: JoltField>(
    preprocessing: &BytecodePreprocessing<F>,
    trace: &[RV32IMCycle],
    eq_r_cycle: &[F],
) -> Vec<Vec<F>> {
    let T = trace.len();
    let num_chunks = rayon::current_num_threads().next_power_of_two().min(T);
    let chunk_size = (T / num_chunks).max(1);
    let log_K = preprocessing.code_size.log_2();
    let d = preprocessing.d;
    let log_K_chunk = log_K.div_ceil(d);
    let K_chunk = log_K_chunk.pow2();

    trace
        .par_chunks(chunk_size)
        .enumerate()
        .map(|(chunk_index, trace_chunk)| {
            let mut result: Vec<Vec<F>> =
                (0..d).map(|_| unsafe_allocate_zero_vec(K_chunk)).collect();
            let mut j = chunk_index * chunk_size;
            for cycle in trace_chunk {
                let mut pc = preprocessing.get_pc(cycle);
                for i in (0..d).rev() {
                    let k = pc % K_chunk;
                    result[i][k] += eq_r_cycle[j];
                    pc >>= log_K_chunk;
                }
                j += 1;
            }
            result
        })
        .reduce(
            || {
                (0..d)
                    .map(|_| unsafe_allocate_zero_vec(K_chunk))
                    .collect::<Vec<_>>()
            },
            |mut running, new| {
                running
                    .par_iter_mut()
                    .zip(new.into_par_iter())
                    .for_each(|(x, y)| {
                        x.par_iter_mut()
                            .zip(y.into_par_iter())
                            .for_each(|(x, y)| *x += y)
                    });
                running
            },
        )
}
