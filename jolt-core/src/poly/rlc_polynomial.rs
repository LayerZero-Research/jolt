use crate::field::{BarrettReduce, FMAdd, JoltField};
use crate::poly::commitment::dory::{DoryGlobals, DoryLayout};
use crate::poly::multilinear_polynomial::MultilinearPolynomial;
use crate::poly::one_hot_polynomial::OneHotPolynomial;
use crate::utils::accumulation::Acc6S;
use crate::utils::math::{s64_from_diff_u64s, Math};
use crate::utils::thread::unsafe_allocate_zero_vec;
use crate::zkvm::bytecode::chunks::{
    committed_bytecode_chunk_cycle_len, committed_lanes, for_each_active_lane_value,
    ActiveLaneValue,
};
use crate::zkvm::config::OneHotParams;
use crate::zkvm::instruction::LookupQuery;
use crate::zkvm::program::ProgramPreprocessing;
use crate::zkvm::ram::remap_address;
use crate::zkvm::witness::CommittedPolynomial;
use allocative::Allocative;
use common::constants::XLEN;
use common::jolt_device::MemoryLayout;
use itertools::Itertools;
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use tracer::ChunksIterator;
use tracer::{instruction::Cycle, LazyTraceIterator};

#[derive(Clone, Debug)]
pub struct RLCStreamingData {
    pub program: Arc<ProgramPreprocessing>,
    pub memory_layout: MemoryLayout,
}

/// Computes the committed bytecode polynomial contribution to a vector-matrix product.
///
/// This is a standalone version of the bytecode VMP computation that can be used
/// by external callers (e.g., GPU prover) without needing a full `StreamingRLCContext`.
///
/// # Arguments
/// * `result` - Output buffer to accumulate contributions into
/// * `left_vec` - Left vector for the vector-matrix product (length >= num_rows)
/// * `num_columns` - Number of columns in the Main Dory matrix
/// * `bytecode_coeff` - RLC coefficient for the committed bytecode polynomial
/// * `program` - Program preprocessing data
/// * `bytecode_T` - Stored bytecode cycle domain (expected to equal `bytecode_len`)
/// * `bytecode_chunk_count` - Cycle chunk count used during bytecode commitment preprocessing
pub fn compute_bytecode_vmp_contribution<F: JoltField>(
    result: &mut [F],
    left_vec: &[F],
    num_columns: usize,
    bytecode_coeff: F,
    program: &ProgramPreprocessing,
    bytecode_T: usize,
    bytecode_chunk_count: usize,
) {
    if bytecode_coeff.is_zero() {
        return;
    }

    let bytecode_len = program.bytecode_len();
    let layout = DoryGlobals::get_layout();
    let lane_capacity = committed_lanes();
    let chunk_cycle_len = committed_bytecode_chunk_cycle_len(bytecode_len, bytecode_chunk_count);
    let total_vars = lane_capacity.log_2() + chunk_cycle_len.log_2();
    let (sigma_bytecode, _) = DoryGlobals::balanced_sigma_nu(total_vars);
    let bytecode_cols = 1usize << sigma_bytecode;
    debug_assert!(
        bytecode_cols <= num_columns,
        "bytecode columns ({bytecode_cols}) must fit in main num_columns ({num_columns})"
    );
    debug_assert!(
        bytecode_cols.is_power_of_two(),
        "Dory num_columns must be power-of-two (got {bytecode_cols})"
    );
    let col_shift = bytecode_cols.trailing_zeros();
    let col_mask = bytecode_cols - 1;

    // Committed bytecode uses top-left embedding with bytecode's own cycle domain.
    // Keep the parameter for backward compatibility and guard against stale commitments.
    let index_T = chunk_cycle_len;
    debug_assert_eq!(
        bytecode_T, index_T,
        "bytecode_T mismatch: expected chunk_cycle_len={index_T}, got {bytecode_T}"
    );

    // Bytecode is embedded as a top-left block of Main:
    // - rows [0 .. bytecode_rows)
    // - cols [0 .. bytecode_cols)
    // where bytecode_rows = (K_bytecode * bytecode_len) / bytecode_cols.

    // Parallelize over cycles with thread-local accumulation.
    let bytecode_contrib: Vec<F> = program.instructions[..bytecode_len]
        .par_iter()
        .enumerate()
        .fold(
            || unsafe_allocate_zero_vec(num_columns),
            |mut acc, (cycle, instr)| {
                debug_assert!(cycle < bytecode_len);
                let chunk_cycle = cycle % chunk_cycle_len;
                for_each_active_lane_value::<F>(instr, |global_lane, lane_val| {
                    let global_index = layout.address_cycle_to_index(
                        global_lane,
                        chunk_cycle,
                        lane_capacity,
                        index_T,
                    );
                    let row_index = global_index >> col_shift;
                    if row_index >= left_vec.len() {
                        return;
                    }
                    let left = left_vec[row_index];
                    if left.is_zero() {
                        return;
                    }
                    let col_index = global_index & col_mask;

                    let base = left * bytecode_coeff;
                    match lane_val {
                        ActiveLaneValue::One => {
                            acc[col_index] += base;
                        }
                        ActiveLaneValue::Scalar(v) => {
                            acc[col_index] += base * v;
                        }
                    }
                });
                acc
            },
        )
        .reduce(
            || unsafe_allocate_zero_vec(num_columns),
            |mut a, b| {
                a.iter_mut().zip(b.iter()).for_each(|(x, y)| *x += *y);
                a
            },
        );

    result
        .par_iter_mut()
        .zip(bytecode_contrib.par_iter())
        .for_each(|(r, c)| *r += *c);
}

/// Source of trace data for streaming VMV computation.
#[derive(Clone, Debug)]
pub enum TraceSource {
    /// Pre-materialized trace in memory (default, efficient single pass)
    Materialized(Arc<Vec<Cycle>>),
    /// Lazy trace iterator (experimental, re-runs tracer)
    /// Boxed to avoid large enum size difference (LazyTraceIterator is ~34KB)
    Lazy(Box<LazyTraceIterator>),
}

impl TraceSource {
    pub fn len(&self) -> usize {
        match self {
            TraceSource::Materialized(trace) => trace.len(),
            // Lazy trace length is not known upfront (would require full iteration)
            TraceSource::Lazy(_) => panic!("Cannot get length of lazy trace"),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            TraceSource::Materialized(trace) => trace.is_empty(),
            TraceSource::Lazy(_) => panic!("Cannot check emptiness of lazy trace"),
        }
    }
}

/// Streaming context for RLC evaluation
#[derive(Clone, Debug)]
pub struct StreamingRLCContext<F: JoltField> {
    pub dense_polys: Vec<(CommittedPolynomial, F)>,
    pub onehot_polys: Vec<(CommittedPolynomial, F)>,
    /// RLC coefficient for the committed bytecode polynomial.
    pub bytecode_coeff: F,
    /// The T value used for bytecode coefficient indexing (from TrustedProgramCommitments).
    /// In committed mode this is fixed to `bytecode_len` (top-left embedding).
    pub bytecode_T: usize,
    /// Cycle chunk count used during committed bytecode preprocessing.
    pub bytecode_chunk_count: usize,
    /// Pre-committed polynomials with their RLC coefficients and IDs.
    /// These are NOT streamed from trace - they're passed in directly.
    /// Format: (poly_id, coeff, polynomial) - ID is needed to determine
    /// commitment dimensions.
    pub pre_committed_polys: Vec<(CommittedPolynomial, F, MultilinearPolynomial<F>)>,
    pub trace_source: TraceSource,
    pub preprocessing: Arc<RLCStreamingData>,
    pub one_hot_params: OneHotParams,
}

/// `RLCPolynomial` represents a multilinear polynomial comprised of a
/// random linear combination of multiple polynomials, potentially with
/// different sizes.
#[derive(Default, Clone, Debug, Allocative)]
pub struct RLCPolynomial<F: JoltField> {
    /// Random linear combination of dense (i.e. length T) polynomials.
    /// Empty if using streaming mode.
    pub dense_rlc: Vec<F>,
    /// Random linear combination of one-hot polynomials (length T x K
    /// for some K). Instead of pre-emptively combining these polynomials,
    /// as we do for `dense_rlc`, we store a vector of (coefficient, polynomial)
    /// pairs and lazily handle the linear combination in `commit_rows`
    /// and `vector_matrix_product`.
    pub one_hot_rlc: Vec<(F, Arc<MultilinearPolynomial<F>>)>,
    /// When present, dense_rlc and one_hot_rlc are not materialized.
    #[allocative(skip)]
    pub streaming_context: Option<Arc<StreamingRLCContext<F>>>,
}

impl<F: JoltField> PartialEq for RLCPolynomial<F> {
    fn eq(&self, other: &Self) -> bool {
        // Compare materialized data only (streaming context is ephemeral)
        self.dense_rlc == other.dense_rlc && self.one_hot_rlc == other.one_hot_rlc
    }
}

impl<F: JoltField> RLCPolynomial<F> {
    pub fn new() -> Self {
        Self {
            dense_rlc: unsafe_allocate_zero_vec(DoryGlobals::get_T()),
            one_hot_rlc: vec![],
            streaming_context: None,
        }
    }

    /// Constructs an `RLCPolynomial` as a linear combination of `polynomials` with the provided
    /// `coefficients`.
    ///
    /// This is a legacy helper (used by some commitment backends) that eagerly combines dense
    /// polynomials into `dense_rlc` and stores one-hot polynomials lazily in `one_hot_rlc`.
    pub fn linear_combination(
        poly_ids: Vec<CommittedPolynomial>,
        polynomials: Vec<Arc<MultilinearPolynomial<F>>>,
        coefficients: &[F],
        streaming_context: Option<Arc<StreamingRLCContext<F>>>,
    ) -> Self {
        debug_assert_eq!(polynomials.len(), coefficients.len());
        debug_assert_eq!(polynomials.len(), poly_ids.len());

        // Partition into dense and one-hot polynomials
        let (dense, one_hot): (Vec<_>, Vec<_>) = polynomials
            .iter()
            .zip(coefficients.iter())
            .partition(|(p, _)| !matches!(p.as_ref(), MultilinearPolynomial::OneHot(_)));

        // Eagerly materialize the dense linear combination (if any).
        let dense_rlc = if dense.is_empty() {
            vec![]
        } else {
            let max_len = dense
                .iter()
                .map(|(p, _)| p.as_ref().original_len())
                .max()
                .unwrap();

            (0..max_len)
                .into_par_iter()
                .map(|idx| {
                    let mut acc = F::zero();
                    for (poly, coeff) in &dense {
                        if idx < poly.as_ref().original_len() {
                            acc += poly.as_ref().get_scaled_coeff(idx, **coeff);
                        }
                    }
                    acc
                })
                .collect()
        };

        // Store one-hot polynomials lazily.
        let one_hot_rlc: Vec<_> = one_hot
            .into_iter()
            .map(|(poly, coeff)| (*coeff, poly.clone()))
            .collect();

        Self {
            dense_rlc,
            one_hot_rlc,
            streaming_context,
        }
    }

    /// Creates a streaming RLC polynomial from polynomial IDs and coefficients.
    /// O(sqrt(T)) space - streams directly from trace without materializing polynomials.
    ///
    /// # Arguments
    /// * `one_hot_params` - Parameters for one-hot polynomial chunking
    /// * `preprocessing` - Bytecode and memory layout for address computation
    /// * `trace_source` - Either materialized trace (default) or lazy trace (experimental)
    /// * `poly_ids` - List of polynomial identifiers
    /// * `coefficients` - RLC coefficients for each polynomial
    /// * `pre_committed_poly_map` - Map of pre-committed polynomial IDs to their actual polynomials
    /// * `bytecode_T` - Stored bytecode cycle domain (expected to equal `bytecode_len`)
    /// * `bytecode_chunk_count` - Chunk count used for committed bytecode commitment
    #[tracing::instrument(skip_all)]
    pub fn new_streaming(
        one_hot_params: OneHotParams,
        preprocessing: Arc<RLCStreamingData>,
        trace_source: TraceSource,
        poly_ids: Vec<CommittedPolynomial>,
        coefficients: &[F],
        mut pre_committed_poly_map: HashMap<CommittedPolynomial, MultilinearPolynomial<F>>,
        bytecode_T: usize,
        bytecode_chunk_count: usize,
    ) -> Self {
        debug_assert_eq!(poly_ids.len(), coefficients.len());

        let mut dense_polys = Vec::new();
        let mut onehot_polys = Vec::new();
        let bytecode_coeff = F::zero();
        let mut pre_committed_polys = Vec::new();

        for (poly_id, coeff) in poly_ids.iter().zip(coefficients.iter()) {
            match poly_id {
                CommittedPolynomial::RdInc | CommittedPolynomial::RamInc => {
                    dense_polys.push((*poly_id, *coeff));
                }
                CommittedPolynomial::InstructionRa(_)
                | CommittedPolynomial::BytecodeRa(_)
                | CommittedPolynomial::RamRa(_) => {
                    onehot_polys.push((*poly_id, *coeff));
                }
                CommittedPolynomial::BytecodeChunk(_)
                | CommittedPolynomial::TrustedAdvice
                | CommittedPolynomial::UntrustedAdvice
                | CommittedPolynomial::ProgramImageInit => {
                    // "Extra" polynomials are passed in directly (not streamed from trace).
                    // Today this includes pre-committed polynomials such as advice, bytecode
                    // chunks, and the program-image polynomial.
                    if pre_committed_poly_map.contains_key(poly_id) {
                        pre_committed_polys.push((
                            *poly_id,
                            *coeff,
                            pre_committed_poly_map.remove(poly_id).unwrap(),
                        ));
                    }
                }
            }
        }

        Self {
            dense_rlc: vec![],   // Not materialized in streaming mode
            one_hot_rlc: vec![], // Not materialized in streaming mode
            streaming_context: Some(Arc::new(StreamingRLCContext {
                dense_polys,
                onehot_polys,
                bytecode_coeff,
                bytecode_T,
                bytecode_chunk_count,
                pre_committed_polys,
                trace_source,
                preprocessing,
                one_hot_params,
            })),
        }
    }

    /// Materializes a streaming RLC polynomial for testing purposes.
    #[cfg(test)]
    pub fn materialize(
        &self,
        _poly_ids: &[CommittedPolynomial],
        polynomials: &[Arc<MultilinearPolynomial<F>>],
        coefficients: &[F],
    ) -> Self {
        if self.streaming_context.is_none() {
            return self.clone();
        }

        let mut result = RLCPolynomial::<F>::new();
        let dense_indices: Vec<usize> = polynomials
            .iter()
            .enumerate()
            .filter(|(_, p)| !matches!(p.as_ref(), MultilinearPolynomial::OneHot(_)))
            .map(|(i, _)| i)
            .collect();

        if !dense_indices.is_empty() {
            let dense_len = result.dense_rlc.len();

            result.dense_rlc = (0..dense_len)
                .into_par_iter()
                .map(|i| {
                    let mut acc = F::zero();
                    for &poly_idx in &dense_indices {
                        let poly = polynomials[poly_idx].as_ref();
                        let coeff = coefficients[poly_idx];

                        if i < poly.original_len() {
                            acc += poly.get_scaled_coeff(i, coeff);
                        }
                    }
                    acc
                })
                .collect();
        }

        for (i, poly) in polynomials.iter().enumerate() {
            if matches!(poly.as_ref(), MultilinearPolynomial::OneHot(_)) {
                result.one_hot_rlc.push((coefficients[i], poly.clone()));
            }
        }

        result
    }

    /// Computes a vector-matrix product, viewing the coefficients of the
    /// polynomial as a matrix (used in Dory).
    /// We do so by computing the vector-matrix product for the individual
    /// polynomials comprising the linear combination, and taking the
    /// linear combination of the resulting products.
    #[tracing::instrument(skip_all, name = "RLCPolynomial::vector_matrix_product")]
    pub fn vector_matrix_product(&self, left_vec: &[F]) -> Vec<F> {
        let num_columns = DoryGlobals::get_num_columns();

        // Compute the vector-matrix product for dense submatrix
        let mut result: Vec<F> = if let Some(ctx) = &self.streaming_context {
            // Streaming mode: generate rows on-demand from trace
            self.streaming_vector_matrix_product(left_vec, num_columns, Arc::clone(ctx))
        } else {
            let mut dense_result = vec![F::zero(); num_columns];
            match DoryGlobals::get_layout() {
                DoryLayout::CycleMajor => {
                    dense_result
                        .par_iter_mut()
                        .enumerate()
                        .for_each(|(col_idx, dest)| {
                            *dest = self
                                .dense_rlc
                                .iter()
                                .skip(col_idx)
                                .step_by(num_columns)
                                .zip(left_vec.iter())
                                .map(|(&a, &b)| a * b)
                                .sum();
                        });
                }
                DoryLayout::AddressMajor => {
                    let dense_stride = DoryGlobals::address_major_dense_stride();
                    dense_result = self
                        .dense_rlc
                        .par_iter()
                        .enumerate()
                        .fold(
                            || unsafe_allocate_zero_vec(num_columns),
                            |mut acc, (cycle, coeff)| {
                                let scaled_index = cycle.saturating_mul(dense_stride);
                                let row_index = scaled_index / num_columns;
                                if row_index >= left_vec.len() {
                                    return acc;
                                }
                                let col_index = scaled_index % num_columns;
                                acc[col_index] += *coeff * left_vec[row_index];
                                acc
                            },
                        )
                        .reduce(
                            || unsafe_allocate_zero_vec(num_columns),
                            |mut a, b| {
                                a.iter_mut().zip(b.iter()).for_each(|(x, y)| *x += *y);
                                a
                            },
                        );
                }
            }
            dense_result
        };

        let one_hot_column_stride = DoryGlobals::address_major_one_hot_stride();
        // Compute the **linear space** vector-matrix product for one-hot polynomials
        for (coeff, poly) in self.one_hot_rlc.iter() {
            match poly.as_ref() {
                MultilinearPolynomial::OneHot(one_hot) => {
                    Self::one_hot_vector_matrix_product_with_stride(
                        one_hot,
                        left_vec,
                        *coeff,
                        &mut result,
                        one_hot_column_stride,
                    );
                }
                _ => panic!("Expected OneHot polynomial in one_hot_rlc"),
            }
        }

        result
    }

    fn one_hot_vector_matrix_product_with_stride(
        one_hot: &OneHotPolynomial<F>,
        left_vec: &[F],
        coeff: F,
        result: &mut [F],
        column_stride: usize,
    ) {
        // CycleMajor one-hot polys stay in the canonical flattened (k, t) prefix domain.
        // Extra Joint-only variables must remain zero for one-hot contributions.
        if DoryGlobals::get_layout() == DoryLayout::CycleMajor {
            one_hot.vector_matrix_product(left_vec, coeff, result);
            return;
        }

        debug_assert_eq!(DoryGlobals::get_layout(), DoryLayout::AddressMajor);
        let num_columns = DoryGlobals::get_num_columns();
        debug_assert_eq!(result.len(), num_columns);
        let dense_stride = DoryGlobals::address_major_dense_stride();

        let onehot_contrib: Vec<F> = one_hot
            .nonzero_indices
            .par_iter()
            .enumerate()
            .fold(
                || unsafe_allocate_zero_vec(num_columns),
                |mut acc, (cycle, k_opt)| {
                    let Some(k) = k_opt else {
                        return acc;
                    };
                    let scaled_index =
                        cycle.saturating_mul(dense_stride) + (*k as usize) * column_stride;
                    let row_index = scaled_index / num_columns;
                    if row_index >= left_vec.len() {
                        return acc;
                    }
                    let col_index = scaled_index % num_columns;
                    acc[col_index] += coeff * left_vec[row_index];
                    acc
                },
            )
            .reduce(
                || unsafe_allocate_zero_vec(num_columns),
                |mut a, b| {
                    a.iter_mut().zip(b.iter()).for_each(|(x, y)| *x += *y);
                    a
                },
            );

        result
            .par_iter_mut()
            .zip(onehot_contrib.par_iter())
            .for_each(|(r, c)| *r += *c);
    }

    /// Adds pre-committed polynomial contributions to the vector-matrix-vector product result.
    ///
    /// In Dory's batch opening, pre-committed polynomials are embedded as top-left blocks of the
    /// main matrix. This function computes their contribution to the VMV product:
    /// ```text
    /// result[col] += left_vec[row] * (coeff * poly[row, col])
    /// ```
    /// for rows and columns within each pre-committed block.
    ///
    /// Each balanced block occupies:
    /// - `sigma_a = ceil(poly_vars/2)`, `nu_a = poly_vars - sigma_a`
    /// - rows `[0 .. 2^{nu_a})` and cols `[0 .. 2^{sigma_a})`
    ///
    /// # Complexity
    /// It uses O(m + a) space where m is the number of rows
    /// and a is the pre-committed polynomial size, so even though it is linear it is negl space overall.
    fn vmp_pre_committed_contribution(
        result: &mut [F],
        left_vec: &[F],
        num_columns: usize,
        ctx: &StreamingRLCContext<F>,
    ) {
        // Dispatch by polynomial type so each pre-committed polynomial class can evolve
        // independently while sharing one integration point in VMV.
        for (poly_id, coeff, poly) in ctx.pre_committed_polys.iter() {
            if poly.original_len() == 0 {
                continue;
            }
            match poly_id {
                CommittedPolynomial::ProgramImageInit => {
                    Self::vmp_program_image_contribution(
                        result,
                        left_vec,
                        num_columns,
                        *coeff,
                        poly,
                        ctx.preprocessing.program.program_image_words.len(),
                    );
                }
                CommittedPolynomial::TrustedAdvice | CommittedPolynomial::UntrustedAdvice => {
                    Self::vmp_balanced_top_left_contribution(
                        result,
                        left_vec,
                        num_columns,
                        *coeff,
                        poly,
                        "Advice",
                    );
                }
                CommittedPolynomial::BytecodeChunk(_) => {
                    Self::vmp_balanced_top_left_contribution(
                        result,
                        left_vec,
                        num_columns,
                        *coeff,
                        poly,
                        "Bytecode chunk",
                    );
                }
                _ => {
                    debug_assert!(
                        false,
                        "unexpected pre-committed polynomial in VMV: {poly_id:?}"
                    );
                }
            }
        }
    }

    fn vmp_program_image_contribution(
        result: &mut [F],
        left_vec: &[F],
        num_columns: usize,
        coeff: F,
        poly: &MultilinearPolynomial<F>,
        nonzero_prefix_len: usize,
    ) {
        let poly_len = poly.original_len();
        let poly_vars = poly_len.log_2();
        let (sigma_a, nu_a) = DoryGlobals::balanced_sigma_nu(poly_vars);
        let poly_cols = 1usize << sigma_a;
        let poly_rows = 1usize << nu_a;

        debug_assert!(
            poly_cols <= num_columns,
            "Program image columns ({poly_cols}) must fit in main num_columns={num_columns}; \
guardrail in gen_from_trace should ensure sigma_main >= sigma_a."
        );

        let effective_rows = poly_rows.min(left_vec.len());
        let len = nonzero_prefix_len.min(poly_len);

        let MultilinearPolynomial::U64Scalars(program_image_poly) = poly else {
            unreachable!("ProgramImageInit polynomial must be U64Scalars");
        };
        let active_len = len.min(effective_rows.saturating_mul(poly_cols));
        let column_contributions: Vec<F> = program_image_poly.coeffs[..active_len]
            .par_chunks(poly_cols)
            .enumerate()
            .fold(
                || unsafe_allocate_zero_vec(poly_cols),
                |mut acc, (row_idx, row)| {
                    let left = left_vec[row_idx];
                    if left.is_zero() {
                        return acc;
                    }
                    let row_coeff = left * coeff;
                    for (col_idx, &word) in row.iter().enumerate() {
                        if word != 0 {
                            acc[col_idx] += row_coeff * F::from_u64(word);
                        }
                    }
                    acc
                },
            )
            .reduce(
                || unsafe_allocate_zero_vec(poly_cols),
                |mut a, b| {
                    a.iter_mut().zip(b.iter()).for_each(|(x, y)| *x += *y);
                    a
                },
            );

        result[..poly_cols]
            .par_iter_mut()
            .zip(column_contributions.par_iter())
            .for_each(|(res, &contrib)| {
                *res += contrib;
            });
    }

    fn vmp_balanced_top_left_contribution(
        result: &mut [F],
        left_vec: &[F],
        num_columns: usize,
        coeff: F,
        poly: &MultilinearPolynomial<F>,
        label: &str,
    ) {
        let poly_len = poly.original_len();
        let poly_vars = poly_len.log_2();
        let (sigma_a, nu_a) = DoryGlobals::balanced_sigma_nu(poly_vars);
        let poly_cols = 1usize << sigma_a;
        let poly_rows = 1usize << nu_a;

        debug_assert!(
            poly_cols <= num_columns,
            "{label} columns ({poly_cols}) must fit in main num_columns={num_columns}; \
guardrail in gen_from_trace should ensure sigma_main >= sigma_a."
        );

        let effective_rows = poly_rows.min(left_vec.len());
        // Row-wise accumulation avoids repeatedly scanning `left_vec` for each column.
        let column_contributions: Vec<F> = (0..effective_rows)
            .into_par_iter()
            .fold(
                || unsafe_allocate_zero_vec(poly_cols),
                |mut acc, row_idx| {
                    let left = left_vec[row_idx];
                    if left.is_zero() {
                        return acc;
                    }
                    let left_coeff = left * coeff;
                    let row_base = row_idx * poly_cols;
                    for col_idx in 0..poly_cols {
                        let coeff_idx = row_base + col_idx;
                        let poly_val = poly.get_coeff(coeff_idx);
                        if !poly_val.is_zero() {
                            acc[col_idx] += left_coeff * poly_val;
                        }
                    }
                    acc
                },
            )
            .reduce(
                || unsafe_allocate_zero_vec(poly_cols),
                |mut a, b| {
                    a.iter_mut().zip(b.iter()).for_each(|(x, y)| *x += *y);
                    a
                },
            );

        result[..poly_cols]
            .par_iter_mut()
            .zip(column_contributions.par_iter())
            .for_each(|(res, &contrib)| {
                *res += contrib;
            });
    }

    /// Streaming VMP implementation that generates rows on-demand from trace.
    /// Achieves O(sqrt(n)) space complexity by lazily generating the witness.
    /// Single pass through trace for both dense and one-hot polynomials.
    /// AddressMajor also uses a streaming path specialized for strided embedding.
    fn streaming_vector_matrix_product(
        &self,
        left_vec: &[F],
        num_columns: usize,
        ctx: Arc<StreamingRLCContext<F>>,
    ) -> Vec<F> {
        // AddressMajor uses a dedicated streaming path.
        match DoryGlobals::get_layout() {
            DoryLayout::AddressMajor => self.address_major_vector_matrix_product(left_vec, num_columns, &ctx),
            DoryLayout::CycleMajor => {
                let matrix_t = DoryGlobals::get_matrix_t();
                match &ctx.trace_source {
                    TraceSource::Materialized(trace) => self.materialized_vector_matrix_product(
                        left_vec,
                        num_columns,
                        trace,
                        &ctx,
                        matrix_t,
                    ),
                    TraceSource::Lazy(lazy_trace) => self.lazy_vector_matrix_product(
                        left_vec,
                        num_columns,
                        (**lazy_trace).clone(),
                        &ctx,
                        matrix_t,
                    ),
                }
            }
        }
    }

    /// AddressMajor VMP: stream dense + one-hot terms directly from trace.
    #[tracing::instrument(skip_all, name = "RLCPolynomial::address_major_vmp")]
    fn address_major_vector_matrix_product(
        &self,
        left_vec: &[F],
        num_columns: usize,
        ctx: &StreamingRLCContext<F>,
    ) -> Vec<F> {
        let trace = match &ctx.trace_source {
            TraceSource::Materialized(trace) => trace,
            TraceSource::Lazy(_) => panic!("AddressMajor VMP requires materialized trace"),
        };

        let dense_stride = DoryGlobals::address_major_dense_stride();
        let one_hot_column_stride = DoryGlobals::address_major_one_hot_stride();

        let mut rd_inc_coeff = F::zero();
        let mut ram_inc_coeff = F::zero();
        for (poly_id, coeff) in ctx.dense_polys.iter() {
            match poly_id {
                CommittedPolynomial::RdInc => rd_inc_coeff += *coeff,
                CommittedPolynomial::RamInc => ram_inc_coeff += *coeff,
                _ => unreachable!("one-hot polynomial found in dense_polys"),
            }
        }

        let mut instruction_coeffs: Vec<F> =
            unsafe_allocate_zero_vec(ctx.one_hot_params.instruction_d);
        let mut bytecode_coeffs: Vec<F> = unsafe_allocate_zero_vec(ctx.one_hot_params.bytecode_d);
        let mut ram_coeffs: Vec<F> = unsafe_allocate_zero_vec(ctx.one_hot_params.ram_d);
        for (poly_id, coeff) in ctx.onehot_polys.iter() {
            if coeff.is_zero() {
                continue;
            }
            match poly_id {
                CommittedPolynomial::InstructionRa(idx) => {
                    debug_assert!(*idx < instruction_coeffs.len());
                    instruction_coeffs[*idx] += *coeff;
                }
                CommittedPolynomial::BytecodeRa(idx) => {
                    debug_assert!(*idx < bytecode_coeffs.len());
                    bytecode_coeffs[*idx] += *coeff;
                }
                CommittedPolynomial::RamRa(idx) => {
                    debug_assert!(*idx < ram_coeffs.len());
                    ram_coeffs[*idx] += *coeff;
                }
                _ => unreachable!("dense polynomial found in onehot_polys"),
            }
        }
        let instruction_terms: Vec<(usize, F)> = instruction_coeffs
            .into_iter()
            .enumerate()
            .filter_map(|(idx, coeff)| (!coeff.is_zero()).then_some((idx, coeff)))
            .collect();
        let bytecode_terms: Vec<(usize, F)> = bytecode_coeffs
            .into_iter()
            .enumerate()
            .filter_map(|(idx, coeff)| (!coeff.is_zero()).then_some((idx, coeff)))
            .collect();
        let ram_terms: Vec<(usize, F)> = ram_coeffs
            .into_iter()
            .enumerate()
            .filter_map(|(idx, coeff)| (!coeff.is_zero()).then_some((idx, coeff)))
            .collect();
        let has_onehot_terms =
            !instruction_terms.is_empty() || !bytecode_terms.is_empty() || !ram_terms.is_empty();

        let num_threads = rayon::current_num_threads().max(1);
        let cycles_per_thread = trace.len().div_ceil(num_threads).max(1);

        let (dense_accs, onehot_accs) = trace
            .par_chunks(cycles_per_thread)
            .enumerate()
            .map(|(chunk_idx, cycles)| {
                let (mut dense_accs, mut onehot_accs) =
                    VmvSetup::<F>::create_accumulators(num_columns);
                let cycle_start = chunk_idx * cycles_per_thread;

                for (offset, cycle) in cycles.iter().enumerate() {
                    let cycle_idx = cycle_start + offset;
                    let scaled_cycle_index = cycle_idx.saturating_mul(dense_stride);
                    let row_index = scaled_cycle_index / num_columns;
                    if row_index >= left_vec.len() {
                        continue;
                    }
                    let left = left_vec[row_index];
                    if left.is_zero() {
                        continue;
                    }

                    let col_index = scaled_cycle_index % num_columns;
                    let scaled_rd_inc = left * rd_inc_coeff;
                    if !scaled_rd_inc.is_zero() {
                        let (_, pre_value, post_value) = cycle.rd_write().unwrap_or_default();
                        let diff = s64_from_diff_u64s(post_value, pre_value);
                        dense_accs[col_index].fmadd(&scaled_rd_inc, &diff);
                    }
                    let scaled_ram_inc = left * ram_inc_coeff;
                    if !scaled_ram_inc.is_zero() {
                        if let tracer::instruction::RAMAccess::Write(write) = cycle.ram_access() {
                            let diff = s64_from_diff_u64s(write.post_value, write.pre_value);
                            dense_accs[col_index].fmadd(&scaled_ram_inc, &diff);
                        }
                    }

                    if !has_onehot_terms {
                        continue;
                    }

                    let lookup_index = LookupQuery::<XLEN>::to_lookup_index(cycle);
                    for (idx, coeff) in instruction_terms.iter() {
                        let k = ctx.one_hot_params.lookup_index_chunk(lookup_index, *idx) as usize;
                        let onehot_col =
                            (scaled_cycle_index + k * one_hot_column_stride) % num_columns;
                        onehot_accs[onehot_col] += left.mul_unreduced::<9>(*coeff);
                    }

                    let pc = ctx.preprocessing.program.get_pc(cycle);
                    for (idx, coeff) in bytecode_terms.iter() {
                        let k = ctx.one_hot_params.bytecode_pc_chunk(pc, *idx) as usize;
                        let onehot_col =
                            (scaled_cycle_index + k * one_hot_column_stride) % num_columns;
                        onehot_accs[onehot_col] += left.mul_unreduced::<9>(*coeff);
                    }

                    let remapped_address = remap_address(
                        cycle.ram_access().address() as u64,
                        &ctx.preprocessing.memory_layout,
                    );
                    if let Some(remapped_address) = remapped_address {
                        for (idx, coeff) in ram_terms.iter() {
                            let k = ctx.one_hot_params.ram_address_chunk(remapped_address, *idx)
                                as usize;
                            let onehot_col =
                                (scaled_cycle_index + k * one_hot_column_stride) % num_columns;
                            onehot_accs[onehot_col] += left.mul_unreduced::<9>(*coeff);
                        }
                    }
                }

                (dense_accs, onehot_accs)
            })
            .reduce(
                || VmvSetup::<F>::create_accumulators(num_columns),
                VmvSetup::<F>::merge_accumulators,
            );
        let mut result = VmvSetup::<F>::finalize(dense_accs, onehot_accs, num_columns);

        Self::vmp_pre_committed_contribution(&mut result, left_vec, num_columns, ctx);

        result
    }

    /// Single-pass VMV over materialized trace. Parallelizes by dividing rows evenly across threads.
    #[tracing::instrument(skip_all)]
    fn materialized_vector_matrix_product(
        &self,
        left_vec: &[F],
        num_columns: usize,
        trace: &[Cycle],
        ctx: &StreamingRLCContext<F>,
        T: usize,
    ) -> Vec<F> {
        let num_rows = T / num_columns;
        let trace_len = trace.len();
        let has_onehot = !ctx.onehot_polys.is_empty();

        // In CycleMajor with expanded matrix cycle-domain (trace_len < T), one-hot coefficients
        // are embedded into the leading linear-space prefix (effective_t = trace_len), not spread
        // across the full matrix cycle-domain (T). Use exact one-hot accumulation in this case.
        let exact_onehot_prefix_mode =
            DoryGlobals::get_layout() == DoryLayout::CycleMajor && has_onehot && trace_len < T;

        // Setup: precompute coefficients, row factors, and folded one-hot tables.
        // For one-hot polys we fold over the canonical trace domain rows, not the
        // potentially larger matrix T.
        let onehot_rows_per_k = trace_len.div_ceil(num_columns).min(num_rows);
        let setup = VmvSetup::new(ctx, left_vec, num_rows, onehot_rows_per_k);

        // Divide rows evenly among threads using par_chunks on left_vec
        // Only use first num_rows elements (left_vec may be longer due to padding)
        let num_threads = rayon::current_num_threads();
        let rows_per_thread = num_rows.div_ceil(num_threads);

        let (dense_accs, onehot_accs) = left_vec[..num_rows]
            .par_chunks(rows_per_thread)
            .enumerate()
            .map(|(chunk_idx, row_weights)| {
                let (mut dense_accs, mut onehot_accs) =
                    VmvSetup::<F>::create_accumulators(num_columns);

                let row_start = chunk_idx * rows_per_thread;
                for (local_idx, &row_weight) in row_weights.iter().enumerate() {
                    let row_idx = row_start + local_idx;
                    let chunk_start = row_idx * num_columns;

                    // Row-scaled dense coefficients.
                    let scaled_rd_inc = row_weight * setup.rd_inc_coeff;
                    let scaled_ram_inc = row_weight * setup.ram_inc_coeff;
                    // Split into valid trace range vs padding range.
                    let valid_end = std::cmp::min(chunk_start + num_columns, trace_len);
                    let row_cycles = if chunk_start < valid_end {
                        &trace[chunk_start..valid_end]
                    } else {
                        &trace[0..0] // Fully padded row
                    };

                    // Process valid trace elements.
                    for (col_idx, cycle) in row_cycles.iter().enumerate() {
                        if exact_onehot_prefix_mode {
                            setup.process_cycle_dense(
                                cycle,
                                scaled_rd_inc,
                                scaled_ram_inc,
                                &mut dense_accs[col_idx],
                            );
                            setup.process_cycle_onehot_prefix_exact(
                                cycle,
                                chunk_start + col_idx,
                                trace_len,
                                num_columns,
                                left_vec,
                                &ctx.onehot_polys,
                                &mut onehot_accs,
                            );
                        } else {
                            let row_factor = setup.row_factors[row_idx];
                            setup.process_cycle(
                                cycle,
                                scaled_rd_inc,
                                scaled_ram_inc,
                                row_factor,
                                &mut dense_accs[col_idx],
                                &mut onehot_accs[col_idx],
                            );
                        }
                    }
                }

                (dense_accs, onehot_accs)
            })
            .reduce(
                || VmvSetup::<F>::create_accumulators(num_columns),
                VmvSetup::<F>::merge_accumulators,
            );

        let mut result = VmvSetup::<F>::finalize(dense_accs, onehot_accs, num_columns);

        // Pre-committed contribution is small and independent of the trace; add it after the streamed pass.
        Self::vmp_pre_committed_contribution(&mut result, left_vec, num_columns, ctx);
        result
    }

    /// Lazy VMV over lazy trace iterator (experimental, re-runs tracer).
    #[tracing::instrument(skip_all)]
    fn lazy_vector_matrix_product(
        &self,
        left_vec: &[F],
        num_columns: usize,
        lazy_trace: LazyTraceIterator,
        ctx: &StreamingRLCContext<F>,
        T: usize,
    ) -> Vec<F> {
        let num_rows = T / num_columns;

        // Setup: precompute coefficients, row factors, and folded one-hot tables.
        // Lazy trace is padded to matrix T, but one-hot support is only on the
        // canonical trace prefix.
        let onehot_rows_per_k = ctx.trace_source.len().div_ceil(num_columns).min(num_rows);
        let setup = VmvSetup::new(ctx, left_vec, num_rows, onehot_rows_per_k);

        let (dense_accs, onehot_accs) = lazy_trace
            .pad_using(T, |_| Cycle::NoOp)
            .iter_chunks(num_columns)
            .enumerate()
            .par_bridge()
            .fold(
                || VmvSetup::<F>::create_accumulators(num_columns),
                |(mut dense_accs, mut onehot_accs), (row_idx, chunk)| {
                    let row_weight = left_vec[row_idx];
                    let scaled_rd_inc = row_weight * setup.rd_inc_coeff;
                    let scaled_ram_inc = row_weight * setup.ram_inc_coeff;
                    let row_factor = setup.row_factors[row_idx];

                    // Process columns within chunk sequentially.
                    for (col_idx, cycle) in chunk.iter().enumerate() {
                        setup.process_cycle(
                            cycle,
                            scaled_rd_inc,
                            scaled_ram_inc,
                            row_factor,
                            &mut dense_accs[col_idx],
                            &mut onehot_accs[col_idx],
                        );
                    }

                    (dense_accs, onehot_accs)
                },
            )
            .reduce(
                || VmvSetup::<F>::create_accumulators(num_columns),
                VmvSetup::<F>::merge_accumulators,
            );
        let mut result = VmvSetup::<F>::finalize(dense_accs, onehot_accs, num_columns);

        // Pre-committed contribution is small and independent of the trace; add it after the streamed pass.
        Self::vmp_pre_committed_contribution(&mut result, left_vec, num_columns, ctx);
        result
    }
}

/// Precomputed tables for the one-hot VMV fast path.
/// Each polynomial type has its own Vec<F> of length k_chunk.
struct FoldedOneHotTables<F: JoltField> {
    /// Tables for InstructionRa polynomials, indexed by [poly_idx][k]
    instruction: Vec<Vec<F>>,
    /// Tables for BytecodeRa polynomials, indexed by [poly_idx][k]
    bytecode: Vec<Vec<F>>,
    /// Tables for RamRa polynomials, indexed by [poly_idx][k]
    ram: Vec<Vec<F>>,
}

/// Precomputed VMV setup shared between materialized and lazy paths.
struct VmvSetup<'a, F: JoltField> {
    /// Coefficient for RdInc dense polynomial
    rd_inc_coeff: F,
    /// Coefficient for RamInc dense polynomial
    ram_inc_coeff: F,
    /// Row factors from left vector decomposition
    row_factors: Vec<F>,
    /// Folded one-hot tables (coeff * eq_k pre-multiplied)
    folded_tables: FoldedOneHotTables<F>,
    /// Reference to program preprocessing data
    program: &'a ProgramPreprocessing,
    memory_layout: &'a MemoryLayout,
    /// Reference to one-hot parameters
    one_hot_params: &'a OneHotParams,
}

impl<'a, F: JoltField> VmvSetup<'a, F> {
    fn new(
        ctx: &'a StreamingRLCContext<F>,
        left_vec: &[F],
        matrix_rows_per_k: usize,
        active_onehot_rows_per_k: usize,
    ) -> Self {
        let one_hot_params = &ctx.one_hot_params;
        let k_chunk = one_hot_params.k_chunk;

        debug_assert!(
            left_vec.len() >= k_chunk * matrix_rows_per_k,
            "left_vec too short for one-hot VMV: len={} need_at_least={}",
            left_vec.len(),
            k_chunk * matrix_rows_per_k
        );
        debug_assert!(
            active_onehot_rows_per_k <= matrix_rows_per_k,
            "active_onehot_rows_per_k={} cannot exceed matrix_rows_per_k={}",
            active_onehot_rows_per_k,
            matrix_rows_per_k
        );

        // Compute row_factors and eq_k from left vector
        let (row_factors, eq_k) = Self::compute_row_factors_and_eq_k(
            left_vec,
            matrix_rows_per_k,
            active_onehot_rows_per_k,
            k_chunk,
        );

        // Extract dense coefficients
        let mut rd_inc_coeff = F::zero();
        let mut ram_inc_coeff = F::zero();
        for (poly_id, coeff) in ctx.dense_polys.iter() {
            match poly_id {
                CommittedPolynomial::RdInc => rd_inc_coeff = *coeff,
                CommittedPolynomial::RamInc => ram_inc_coeff = *coeff,
                _ => unreachable!("one-hot polynomial found in dense_polys"),
            }
        }

        // Build folded one-hot tables (non-flattened)
        let folded_tables =
            Self::build_folded_tables(&ctx.onehot_polys, one_hot_params, &eq_k, k_chunk);

        Self {
            rd_inc_coeff,
            ram_inc_coeff,
            row_factors,
            folded_tables,
            program: &ctx.preprocessing.program,
            memory_layout: &ctx.preprocessing.memory_layout,
            one_hot_params,
        }
    }

    /// Compute row_factors and eq_k from the Dory left vector.
    #[inline]
    fn compute_row_factors_and_eq_k(
        left_vec: &[F],
        matrix_rows_per_k: usize,
        active_onehot_rows_per_k: usize,
        k_chunk: usize,
    ) -> (Vec<F>, Vec<F>) {
        let mut row_factors: Vec<F> = unsafe_allocate_zero_vec(matrix_rows_per_k);
        let mut eq_k: Vec<F> = unsafe_allocate_zero_vec(k_chunk);

        for k in 0..k_chunk {
            // Left vector is laid out in full matrix rows-per-K blocks.
            // When exec_t < matrix_t, only the first active_onehot_rows_per_k rows per block
            // contribute to one-hot support; remaining rows are padding.
            let base = k * matrix_rows_per_k;
            let mut sum_k = F::zero();
            for row in 0..active_onehot_rows_per_k {
                let v = left_vec[base + row];
                sum_k += v;
                row_factors[row] += v;
            }
            eq_k[k] = sum_k;
        }

        (row_factors, eq_k)
    }

    /// Build per-polynomial folded one-hot tables (non-flattened).
    fn build_folded_tables(
        onehot_polys: &[(CommittedPolynomial, F)],
        one_hot_params: &OneHotParams,
        eq_k: &[F],
        k_chunk: usize,
    ) -> FoldedOneHotTables<F> {
        let instruction_d = one_hot_params.instruction_d;
        let bytecode_d = one_hot_params.bytecode_d;
        let ram_d = one_hot_params.ram_d;

        // Initialize tables with zeros
        let mut instruction: Vec<Vec<F>> = (0..instruction_d)
            .map(|_| unsafe_allocate_zero_vec(k_chunk))
            .collect();
        let mut bytecode: Vec<Vec<F>> = (0..bytecode_d)
            .map(|_| unsafe_allocate_zero_vec(k_chunk))
            .collect();
        let mut ram: Vec<Vec<F>> = (0..ram_d)
            .map(|_| unsafe_allocate_zero_vec(k_chunk))
            .collect();

        // Fill tables with coeff * eq_k[k]
        for (poly_id, coeff) in onehot_polys.iter() {
            if coeff.is_zero() {
                continue;
            }
            match poly_id {
                CommittedPolynomial::InstructionRa(idx) => {
                    for k in 0..k_chunk {
                        instruction[*idx][k] = *coeff * eq_k[k];
                    }
                }
                CommittedPolynomial::BytecodeRa(idx) => {
                    for k in 0..k_chunk {
                        bytecode[*idx][k] = *coeff * eq_k[k];
                    }
                }
                CommittedPolynomial::RamRa(idx) => {
                    for k in 0..k_chunk {
                        ram[*idx][k] = *coeff * eq_k[k];
                    }
                }
                _ => unreachable!("dense polynomial found in onehot_polys"),
            }
        }

        FoldedOneHotTables {
            instruction,
            bytecode,
            ram,
        }
    }

    /// Process a single cycle.
    #[inline(always)]
    fn process_cycle_dense(
        &self,
        cycle: &Cycle,
        scaled_rd_inc: F,
        scaled_ram_inc: F,
        dense_acc: &mut Acc6S<F>,
    ) {
        // Dense polynomials: accumulate scaled_coeff * (post - pre)
        let (_, pre_value, post_value) = cycle.rd_write().unwrap_or_default();
        let diff = s64_from_diff_u64s(post_value, pre_value);
        dense_acc.fmadd(&scaled_rd_inc, &diff);

        if let tracer::instruction::RAMAccess::Write(write) = cycle.ram_access() {
            let diff = s64_from_diff_u64s(write.post_value, write.pre_value);
            dense_acc.fmadd(&scaled_ram_inc, &diff);
        }
    }

    /// Process one-hot terms with exact prefix embedding (effective_t = trace_len).
    #[inline(always)]
    fn process_cycle_onehot_prefix_exact(
        &self,
        cycle: &Cycle,
        cycle_idx: usize,
        trace_len: usize,
        num_columns: usize,
        left_vec: &[F],
        onehot_polys: &[(CommittedPolynomial, F)],
        onehot_accs: &mut [F::Unreduced<9>],
    ) {
        let lookup_index = LookupQuery::<XLEN>::to_lookup_index(cycle);
        let pc = self.program.get_pc(cycle);
        let remapped_address =
            remap_address(cycle.ram_access().address() as u64, self.memory_layout);

        for (poly_id, coeff) in onehot_polys.iter() {
            if coeff.is_zero() {
                continue;
            }

            let k = match poly_id {
                CommittedPolynomial::InstructionRa(idx) => {
                    self.one_hot_params.lookup_index_chunk(lookup_index, *idx) as usize
                }
                CommittedPolynomial::BytecodeRa(idx) => {
                    self.one_hot_params.bytecode_pc_chunk(pc, *idx) as usize
                }
                CommittedPolynomial::RamRa(idx) => {
                    let Some(addr) = remapped_address else {
                        continue;
                    };
                    self.one_hot_params.ram_address_chunk(addr, *idx) as usize
                }
                _ => unreachable!("dense polynomial found in onehot_polys"),
            };

            let global_index = k * trace_len + cycle_idx;
            let row_index = global_index / num_columns;
            let col_index = global_index % num_columns;
            if row_index < left_vec.len() && col_index < onehot_accs.len() {
                onehot_accs[col_index] += left_vec[row_index].mul_unreduced::<9>(*coeff);
            }
        }
    }

    /// Process a single cycle.
    #[inline(always)]
    fn process_cycle(
        &self,
        cycle: &Cycle,
        scaled_rd_inc: F,
        scaled_ram_inc: F,
        row_factor: F,
        dense_acc: &mut Acc6S<F>,
        onehot_acc: &mut F::Unreduced<9>,
    ) {
        self.process_cycle_dense(cycle, scaled_rd_inc, scaled_ram_inc, dense_acc);

        // One-hot polynomials: accumulate using pre-folded K tables (unreduced)
        let mut inner_sum = F::Unreduced::<5>::default();

        // Instruction RA chunks
        let lookup_index = LookupQuery::<XLEN>::to_lookup_index(cycle);
        for (i, table) in self.folded_tables.instruction.iter().enumerate() {
            let k = self.one_hot_params.lookup_index_chunk(lookup_index, i) as usize;
            inner_sum += *table[k].as_unreduced_ref();
        }

        // Bytecode RA chunks
        let pc = self.program.get_pc(cycle);
        for (i, table) in self.folded_tables.bytecode.iter().enumerate() {
            let k = self.one_hot_params.bytecode_pc_chunk(pc, i) as usize;
            inner_sum += *table[k].as_unreduced_ref();
        }

        // RAM RA chunks
        let address = cycle.ram_access().address() as u64;
        if let Some(remapped) = remap_address(address, self.memory_layout) {
            for (i, table) in self.folded_tables.ram.iter().enumerate() {
                let k = self.one_hot_params.ram_address_chunk(remapped, i) as usize;
                inner_sum += *table[k].as_unreduced_ref();
            }
        }

        // Reduce inner_sum before multiplying with row_factor
        let inner_sum_reduced = F::from_barrett_reduce::<5>(inner_sum);
        *onehot_acc += row_factor.mul_unreduced::<9>(inner_sum_reduced);
    }

    #[inline]
    fn create_accumulators(num_columns: usize) -> (Vec<Acc6S<F>>, Vec<F::Unreduced<9>>) {
        (
            unsafe_allocate_zero_vec(num_columns),
            unsafe_allocate_zero_vec(num_columns),
        )
    }

    #[inline]
    fn merge_accumulators(
        (mut dense_a, mut onehot_a): (Vec<Acc6S<F>>, Vec<F::Unreduced<9>>),
        (dense_b, onehot_b): (Vec<Acc6S<F>>, Vec<F::Unreduced<9>>),
    ) -> (Vec<Acc6S<F>>, Vec<F::Unreduced<9>>) {
        for (a, b) in dense_a.iter_mut().zip(dense_b.iter()) {
            *a = *a + *b;
        }
        for (a, b) in onehot_a.iter_mut().zip(onehot_b.iter()) {
            *a += *b;
        }
        (dense_a, onehot_a)
    }

    fn finalize(
        dense_accs: Vec<Acc6S<F>>,
        onehot_accs: Vec<F::Unreduced<9>>,
        num_columns: usize,
    ) -> Vec<F> {
        (0..num_columns)
            .into_par_iter()
            .map(|col_idx| {
                dense_accs[col_idx].barrett_reduce()
                    + F::from_montgomery_reduce::<9>(onehot_accs[col_idx])
            })
            .collect()
    }
}
