//! Utilities for streaming evaluations and values used by the Jolt VM.
//!
//! This module defines the `FunctionStream` trait together with several
//! concrete stream implementations.  A stream presents data **one element at a
//! time**, allowing algorithms such as space-efficient sum-checks to work with
//! large sequences without materialising them in memory all at once.
//!
//! In the simplest case a stream wraps a `Vec<T>` and yields its entries
//! sequentially, but more sophisticated streams (e.g. `LagrangeBasisStream`,
//! `ValStream`, and `RaStream`) compute their outputs on the fly.

use rayon::{
    iter::{
        IndexedParallelIterator, IntoParallelIterator, IntoParallelRefMutIterator, ParallelIterator,
    },
    slice::ParallelSlice,
};
use tracer::instruction::{NormalizedInstruction, RV32IMCycle, RV32IMInstruction};

use crate::{
    field::JoltField,
    jolt::vm::bytecode::{BytecodePreprocessing},
    poly::{compact_polynomial::SmallScalar, eq_poly::EqPolynomial},
    utils::thread::unsafe_allocate_zero_vec,
};

#[tracing::instrument(skip_all)]
pub fn bytecode_to_val<F: JoltField>(instruction: &RV32IMInstruction, gamma: F) -> F {
    let mut gamma_powers = vec![F::one()];
    for _ in 0..5 {
        gamma_powers.push(gamma * gamma_powers.last().unwrap());
    }

    let NormalizedInstruction {
        address,
        operands,
        virtual_sequence_remaining: _,
    } = instruction.normalize();
    let mut linear_combination = F::zero();
    linear_combination += (address as u64).field_mul(gamma_powers[0]);
    linear_combination += (operands.rd as u64).field_mul(gamma_powers[1]);
    linear_combination += (operands.rs1 as u64).field_mul(gamma_powers[2]);
    linear_combination += (operands.rs2 as u64).field_mul(gamma_powers[3]);
    linear_combination += operands.imm.field_mul(gamma_powers[4]);
    // TODO(moodlezoup): Circuit and lookup flags
    linear_combination
}


/// A generic stream of field elements.
///
/// The trait abstracts over any sequence that can be consumed **one element at
/// a time** while keeping only minimal state.  A stream can be restarted via
/// [`reset`] or queried incrementally via [`next`].
pub trait FunctionStream<F: JoltField> {
    /// Rewinds the stream back to its initial state.
    fn reset(&mut self);
    /// Produces the next element of the stream, or `None` if the stream is
    /// exhausted.
    fn next(&mut self) -> Option<F>;
    /// Consumes the stream and collects all remaining elements into a vector.
    fn into_vec(self) -> Vec<F>;
}

/// Streams the evaluations χ_{<v>}(r) for all binary vectors `v` in order.
///
/// Given a point `r ∈ Fⁿ`, the stream outputs χ_{<0>}(r), χ_{<1>}(r), …, χ_{<2ⁿ−1>}(r)
/// on successive calls to [`next`].
pub struct LagrangeBasisStream<'a, F: JoltField> {
    input: &'a [F],
    state: F,
    index: usize,
    index_range: usize,
}

impl<'a, F: JoltField> LagrangeBasisStream<'a, F> {
    pub fn new(input: &'a [F]) -> Self {
        // Compute the initial state χ_{<0>}(r) = ∏ (1 - r_i)
        let initial_state = Self::initial_state(input);
        Self {
            input,
            state: initial_state,
            index: 0,
            index_range: 1usize << input.len(),
        }
    }
    /// Computes χ_{<0>}(r).
    fn initial_state(input: &[F]) -> F {
        input
            .iter()
            .fold(F::one(), |acc, r_i| acc * (F::one() - *r_i))
    }
}

impl<'a, F: JoltField> FunctionStream<F> for LagrangeBasisStream<'a, F> {
    fn reset(&mut self) {
        self.index = 0;
        self.state = LagrangeBasisStream::initial_state(&self.input);
    }

    fn next(&mut self) -> Option<F> {
        // The stream is exhausted when we have produced 2^n values.
        if self.index >= self.index_range {
            return None;
        }
        // For the first call (v=0), the initial state is already computed.
        // We return it and advance the index.
        if self.index == 0 {
            self.index += 1;
            return Some(self.state);
        }

        // For all subsequent calls (v > 0), we compute the new state from the previous one.
        // self.state currently holds χ_{v-1}(r). We will now compute χ_{v}(r).
        // v is self.index.
        let v_minus_1 = self.index - 1;

        // j is the index of the least significant bit of ⟨v⟩ that is equal to 0
        let j = v_minus_1.trailing_ones() as usize;
        // Ensure `j` is within the bounds of the input vector `r`. This is a safety check.
        // This case should not be reachable if the stream length check is correct.
        debug_assert!(j < self.input.len());
        
        // Compute the update factor based on j.
        // factor = (r_j / (1 - r_j)) * ∏_{i<j} ((1 - r_i) / r_i)
        let r_j = self.input[self.input.len() - 1 - j];
        let mut factor = r_j / (F::one() - r_j);

        for i in 0..j {
            let r_i = self.input[self.input.len() - 1 - i];
            factor *= (F::one() - r_i) / r_i;
        }
        self.state *= factor;
        self.index += 1;
        Some(self.state)
    }

    fn into_vec(mut self) -> Vec<F> {
        (0..(1 << self.input.len()))
            .map(|_| self.next().unwrap())
            .collect()
    }
}

/// Streams the `val` linear-combination used in the lookup argument.
///
/// Each element corresponds to one instruction in the pre-processed bytecode
/// and is computed lazily via [`bytecode_to_val`].
pub struct ValStream<F: JoltField> {
    bytecode_preprocessing: BytecodePreprocessing,
    gamma: F,
    index: usize,
}

impl<F: JoltField> ValStream<F> {
    pub fn new(bytecode_preprocessing: BytecodePreprocessing, gamma: F) -> Self {
        Self {
            bytecode_preprocessing,
            gamma,
            index: 0,
        }
    }
}

impl<F: JoltField> FunctionStream<F> for ValStream<F> {
    fn reset(&mut self) {
        self.index = 0;
    }

    fn next(&mut self) -> Option<F> {
        if self.index < self.bytecode_preprocessing.get_bytecode().len() {
            let bytecode = self.bytecode_preprocessing.get_bytecode_index(self.index);
            let result = bytecode_to_val(bytecode, self.gamma);
            self.index += 1;
            return Some(result);
        } else {
            return None;
        }
    }

    fn into_vec(mut self) -> Vec<F> {
        self.reset();
        let mut result = Vec::new();
        loop {
            match self.next() {
                Some(value) => result.push(value),
                None => break,
            }
        }
        self.reset();
        return result;
    }
}

/// Streams the `ra` (row address) values used in the lookup argument.
///
/// Internally pre-computes auxiliary vectors `E` and `F` to allow constant-time
/// access during streaming.
pub struct RaStream<F: JoltField> {
    bytecode_preprocessing: BytecodePreprocessing,
    F: Vec<F>,
    index: usize,
}

impl<F: JoltField> RaStream<F> {
    pub fn new(
        bytecode_preprocessing: BytecodePreprocessing,
        trace: &[RV32IMCycle],
        r_cycle: Vec<F>,
        r_shift: Vec<F>,
        _z: F,
    ) -> (Self, Vec<F>, Vec<F>) {
        let K = bytecode_preprocessing
            .get_bytecode()
            .len()
            .next_power_of_two();

        let E: Vec<F> = EqPolynomial::evals(&r_cycle);
        let E_shift: Vec<F> = EqPolynomial::evals(&r_shift);

        let span = tracing::span!(tracing::Level::INFO, "compute F");
        let _guard = span.enter();

        let num_chunks = rayon::current_num_threads()
            .next_power_of_two()
            .min(trace.len());
        let chunk_size = (trace.len() / num_chunks).max(1);
        let (F, F_shift): (Vec<_>, Vec<_>) = trace
            .par_chunks(chunk_size)
            .enumerate()
            .map(|(chunk_index, trace_chunk)| {
                let mut result: Vec<F> = unsafe_allocate_zero_vec(K);
                let mut result_shift: Vec<F> = unsafe_allocate_zero_vec(K);
                let mut j = chunk_index * chunk_size;
                for cycle in trace_chunk {
                    let k = bytecode_preprocessing.get_pc(cycle, j == trace.len() - 1);
                    result[k] += E[j];
                    result_shift[k] += E_shift[j];
                    j += 1;
                }
                (result, result_shift)
            })
            .reduce(
                || (unsafe_allocate_zero_vec(K), unsafe_allocate_zero_vec(K)),
                |(mut running, mut running_shift), (new, new_shift)| {
                    running
                        .par_iter_mut()
                        .zip(new.into_par_iter())
                        .for_each(|(x, y)| *x += y);
                    running_shift
                        .par_iter_mut()
                        .zip(new_shift.into_par_iter())
                        .for_each(|(x, y)| *x += y);
                    (running, running_shift)
                },
            );
        drop(_guard);
        drop(span);
        (
            Self {
                bytecode_preprocessing,
                F,
                index: 0,
            },
            F_shift,
            E,
        )
    }

    pub fn get_vector(&mut self) -> Vec<F> {
        self.reset();
        let mut result = Vec::new();
        loop {
            match self.next() {
                Some(value) => result.push(value),
                None => break,
            }
        }
        self.reset();
        result
    }
}

impl<F: JoltField> FunctionStream<F> for RaStream<F> {
    fn reset(&mut self) {
        self.index = 0;
    }

    fn next(&mut self) -> Option<F> {
        if self.index < self.F.len() {
            let result = self.F[self.index];
            self.index += 1;
            return Some(result);
        } else {
            return None;
        }
    }

    fn into_vec(mut self) -> Vec<F> {
        self.reset();
        let mut result = Vec::new();
        loop {
            match self.next() {
                Some(value) => result.push(value),
                None => break,
            }
        }
        self.reset();
        return result;
    }
}

#[cfg(test)]
mod test_lagrange_basis_streaming {
    use crate::{jolt::vm::rv32i_vm::ProofTranscript, utils::transcript::Transcript};
    use super::*;
    #[test]
    fn test_correctness() {
        use ark_bn254::Fr;
        const LENGTH: usize = 20;
        let input: Vec<Fr> = ProofTranscript::new(&[0]).challenge_vector(LENGTH);
        let expected_basis = EqPolynomial::evals(&input);
        let mut basis_stream = LagrangeBasisStream::new(&input);
        for i in 0..LENGTH {
            assert_eq!(
                expected_basis[i],
                basis_stream.next().unwrap(),
                "round: {}",
                i
            );
        }
        basis_stream.reset();
        let computed_basis = basis_stream.into_vec();
        for i in 0..LENGTH {
            assert_eq!(expected_basis[i], computed_basis[i], "round: {}", i);
        }
    }
}
