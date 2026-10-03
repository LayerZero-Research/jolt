use akita_algebra::{ring::WideCyclotomicRing, CyclotomicRing};
use akita_error::AkitaError;
use akita_pcs::custom_source::CommitInnerPlan;
use akita_types::{AkitaExpandedSetup, RingVec};
use jolt_field::Fp128x8i32;
use rayon::prelude::*;

use super::digit_windows::{flush_digit_accumulators, DigitWindows};
use super::source::TraceOneHotColumn;
use super::traversal::{
    flush_deferred_rank, flush_wide, row_is_committed, trace_block_task_schedule,
    validate_block_geometry, visit_segment_ring_range, visit_segment_ring_row_range,
    DeferredFp128Ring, TraceBlockTaskSchedule,
};
use super::{K256_ROW_BATCH, MAX_WIDE_ACCUMULATIONS, NO_SELECTED_ROW};
use crate::AkitaField;

pub(super) fn commit_columns<const D: usize>(
    expanded: &AkitaExpandedSetup<AkitaField>,
    source: &TraceOneHotColumn,
    plan: CommitInnerPlan,
) -> Result<Vec<RingVec<AkitaField>>, AkitaError> {
    let _span = tracing::info_span!(
        "TraceOneHotColumn::commit_inner",
        ring_dimension = D,
        one_hot_k = source.one_hot_k,
        rows = source.rows.num_rows(),
        columns = source.rows.num_columns(),
        n_a = plan.n_a,
        positions_per_block = plan.num_positions_per_block,
        inner_digits = plan.num_digits_inner,
    )
    .entered();
    let _prepare_span = tracing::info_span!("trace_onehot_commit_prepare").entered();
    let segment_rings = source.segment_ring_elems::<D>()?;
    let (_, num_blocks) = validate_block_geometry(
        segment_rings,
        source.num_columns,
        plan.num_positions_per_block,
    )?;
    if plan.num_live_blocks != num_blocks / source.num_columns {
        return Err(AkitaError::InvalidInput(
            "trace commitment live-block extent disagrees with its columns".into(),
        ));
    }
    let active_cols = plan
        .num_positions_per_block
        .checked_mul(plan.num_digits_inner)
        .ok_or_else(|| AkitaError::InvalidSetup("active A width overflow".to_string()))?;
    let a_view = expanded
        .shared_matrix()
        .ring_view::<D>(plan.n_a, active_cols)?;
    let a_rows = a_view.rows().collect::<Vec<_>>();
    let max_per_ring = (D / source.one_hot_k).max(1);
    drop(_prepare_span);

    let rows = if segment_rings >= plan.num_positions_per_block {
        let blocks_per_column = segment_rings / plan.num_positions_per_block;
        debug_assert_eq!(
            blocks_per_column * plan.num_positions_per_block,
            segment_rings
        );
        let schedule = trace_block_task_schedule::<D>(
            source.one_hot_k,
            plan.num_positions_per_block,
            blocks_per_column,
        );
        let num_columns = source.rows.num_columns();
        let _accumulate_span = tracing::info_span!(
            "trace_onehot_commit_accumulate",
            num_blocks,
            blocks_per_column,
            task_parts = schedule.parts,
            tasks = blocks_per_column * schedule.parts,
            active_columns = num_columns,
            rows_per_ring = D / source.one_hot_k,
        )
        .entered();
        let partials = if source.one_hot_k < D {
            source.commit_shared_tiles(&a_rows, plan, blocks_per_column, &schedule)?
        } else {
            (0..blocks_per_column * schedule.parts)
                .into_par_iter()
                .map(|task| {
                    let trace_block = task / schedule.parts;
                    let part = task % schedule.parts;
                    let mut reduced = vec![CyclotomicRing::zero(); num_columns * plan.n_a];
                    let block_ring_start = trace_block * plan.num_positions_per_block;
                    let (part_start, part_end) = schedule.part_range(part);
                    let ring_start = block_ring_start + part_start;
                    let ring_end = block_ring_start + part_end;
                    let rank_tiled_k256 = matches!(D, 64 | 128 | 256)
                        && source.one_hot_k == 256
                        && plan.num_positions_per_block >= source.one_hot_k / D
                        && num_columns <= u32::BITS as usize;
                    let mut wide = if rank_tiled_k256 {
                        Vec::new()
                    } else {
                        vec![WideCyclotomicRing::zero(); num_columns * plan.n_a]
                    };
                    let mut budget = 0usize;
                    if rank_tiled_k256 {
                        // Stream one A rank at a time so its destination accumulators fit in cache.
                        let rings_per_row = source.one_hot_k / D;
                        debug_assert!(matches!(rings_per_row, 1 | 2 | 4));
                        debug_assert_eq!(ring_start % rings_per_row, 0);
                        debug_assert_eq!(ring_end % rings_per_row, 0);
                        let row_start = ring_start / rings_per_row;
                        let row_end = ring_end / rings_per_row;
                        let mut selected_rows = vec![NO_SELECTED_ROW; num_columns];
                        let mut hot_values = vec![0u8; K256_ROW_BATCH * num_columns];
                        let mut ring_masks = vec![[0u32; 4]; K256_ROW_BATCH];
                        let mut rank_deferred = vec![DeferredFp128Ring::zero(); num_columns];
                        for tile_start in (row_start..row_end).step_by(K256_ROW_BATCH) {
                            let tile_len = (row_end - tile_start).min(K256_ROW_BATCH);
                            for row_offset in 0..tile_len {
                                let row = tile_start + row_offset;
                                source.rows.fill_row(row, &mut selected_rows);
                                let committed_zero_mask =
                                    source.rows.committed_digit_zero_mask(row);
                                let masks = &mut ring_masks[row_offset];
                                *masks = [0; 4];
                                for (column, &hot) in selected_rows.iter().enumerate() {
                                    if !row_is_committed(hot, committed_zero_mask, column) {
                                        continue;
                                    }
                                    if usize::from(hot) >= source.one_hot_k {
                                        return Err(AkitaError::InvalidInput(format!(
                                            "trace one-hot row {hot} is outside K={}",
                                            source.one_hot_k
                                        )));
                                    }
                                    hot_values[row_offset * num_columns + column] = hot;
                                    masks[usize::from(hot) / D] |= 1 << column;
                                }
                            }
                            for (a, a_row) in a_rows.iter().enumerate() {
                                for row_offset in 0..tile_len {
                                    let trace_row = tile_start + row_offset;
                                    for (ring_offset, &mask) in
                                        ring_masks[row_offset][..rings_per_row].iter().enumerate()
                                    {
                                        if mask == 0 {
                                            continue;
                                        }
                                        let ring = trace_row * rings_per_row + ring_offset;
                                        let position = ring - block_ring_start;
                                        let a_col = position * plan.num_digits_inner;
                                        let mut remaining = mask;
                                        while remaining != 0 {
                                            let column = remaining.trailing_zeros() as usize;
                                            remaining &= remaining - 1;
                                            let hot = hot_values[row_offset * num_columns + column]
                                                as usize;
                                            rank_deferred[column]
                                                .shift_accumulate(&a_row[a_col], hot % D);
                                        }
                                    }
                                }
                                flush_deferred_rank(&mut rank_deferred, &mut reduced, plan.n_a, a);
                            }
                        }
                    } else {
                        visit_segment_ring_range::<D>(
                            source,
                            ring_start,
                            ring_end,
                            |ring, contributions| {
                                if contributions.is_empty() {
                                    return;
                                }
                                let position = ring - block_ring_start;
                                let a_col = position * plan.num_digits_inner;
                                for (a, a_row) in a_rows.iter().enumerate() {
                                    let a_wide = WideCyclotomicRing::from_ring(&a_row[a_col]);
                                    for &(column, coefficient) in contributions {
                                        a_wide.shift_accumulate_into(
                                            &mut wide[column * plan.n_a + a],
                                            coefficient,
                                        );
                                    }
                                }
                                budget += max_per_ring;
                                if budget >= MAX_WIDE_ACCUMULATIONS {
                                    flush_wide(&mut wide, &mut reduced);
                                    budget = 0;
                                }
                            },
                        )?;
                    }
                    if budget != 0 {
                        flush_wide(&mut wide, &mut reduced);
                    }
                    Ok::<_, AkitaError>(reduced)
                })
                .collect::<Result<Vec<_>, _>>()?
        };
        drop(_accumulate_span);
        let _merge_span = tracing::info_span!(
            "trace_onehot_commit_merge_partials",
            num_blocks,
            blocks_per_column,
            task_parts = schedule.parts,
            active_columns = num_columns,
            n_a = plan.n_a,
        )
        .entered();
        let mut rows = vec![vec![CyclotomicRing::zero(); plan.n_a]; num_blocks];
        for (task, block_rows) in partials.into_iter().enumerate() {
            let trace_block = task / schedule.parts;
            let part = task % schedule.parts;
            for column in 0..num_columns {
                let dst = &mut rows[column * blocks_per_column + trace_block];
                let src = &block_rows[column * plan.n_a..(column + 1) * plan.n_a];
                if part == 0 {
                    dst.copy_from_slice(src);
                } else {
                    for (dst, src) in dst.iter_mut().zip(src) {
                        *dst += *src;
                    }
                }
            }
        }
        rows
    } else {
        let _accumulate_span = tracing::info_span!(
            "trace_onehot_commit_accumulate_flat",
            num_blocks,
            segment_rings,
            n_a = plan.n_a,
        )
        .entered();
        let mut wide = vec![WideCyclotomicRing::zero(); num_blocks * plan.n_a];
        let mut reduced = vec![CyclotomicRing::zero(); num_blocks * plan.n_a];
        let mut budget = 0usize;
        visit_segment_ring_range::<D>(source, 0, segment_rings, |ring, contributions| {
            for &(column, coefficient) in contributions {
                let block = column;
                let position = ring;
                let a_col = position * plan.num_digits_inner;
                for (a, a_row) in a_rows.iter().enumerate() {
                    let a_wide = WideCyclotomicRing::from_ring(&a_row[a_col]);
                    a_wide.shift_accumulate_into(&mut wide[block * plan.n_a + a], coefficient);
                }
            }
            budget += contributions.len();
            if budget >= MAX_WIDE_ACCUMULATIONS {
                flush_wide(&mut wide, &mut reduced);
                budget = 0;
            }
        })?;
        if budget != 0 {
            flush_wide(&mut wide, &mut reduced);
        }
        reduced
            .chunks_exact(plan.n_a)
            .map(<[CyclotomicRing<AkitaField, D>]>::to_vec)
            .collect()
    };

    rows.chunks_exact(num_blocks / source.num_columns)
        .map(|blocks| {
            let coefficients = blocks
                .iter()
                .flatten()
                .flat_map(|row| row.coefficients().iter().copied())
                .collect();
            RingVec::from_coeffs_with_ring_dim(coefficients, D)
        })
        .collect()
}

impl TraceOneHotColumn {
    fn commit_shared_tiles<const D: usize>(
        &self,
        a_rows: &[&[CyclotomicRing<AkitaField, D>]],
        plan: CommitInnerPlan,
        blocks_per_column: usize,
        schedule: &TraceBlockTaskSchedule,
    ) -> Result<Vec<Vec<CyclotomicRing<AkitaField, D>>>, AkitaError> {
        let num_columns = self.rows.num_columns();
        let rows_per_ring = D / self.one_hot_k;
        // Bound all simultaneously prepared parts and ranks to 8 MiB.
        let tile_len =
            (8 * 1024 * 1024 / schedule.parts / plan.n_a / (2 * D * size_of::<[i32; 8]>())).max(1);
        let mut partials = (0..blocks_per_column * schedule.parts)
            .map(|_| {
                (
                    vec![[Fp128x8i32([0; 8]); D]; num_columns * plan.n_a],
                    vec![CyclotomicRing::zero(); num_columns * plan.n_a],
                    0usize,
                )
            })
            .collect::<Vec<_>>();
        let part_len = (0..schedule.parts)
            .map(|part| {
                let (start, end) = schedule.part_range(part);
                end - start
            })
            .max()
            .unwrap_or(0);
        for offset in (0..part_len).step_by(tile_len) {
            let tiles = (0..schedule.parts)
                .into_par_iter()
                .map(|part| {
                    let (start, end) = schedule.part_range(part);
                    let start = (start + offset).min(end);
                    let end = (start + tile_len).min(end);
                    let windows = (start..end)
                        .flat_map(|position| {
                            a_rows.iter().map(move |row| {
                                let mut windows = DigitWindows::<D>::new();
                                windows.load(&row[position * plan.num_digits_inner]);
                                windows
                            })
                        })
                        .collect::<Vec<_>>();
                    (start, windows)
                })
                .collect::<Vec<_>>();
            partials.par_iter_mut().enumerate().try_for_each(
                |(task, (accumulators, reduced, budget))| {
                    let trace_block = task / schedule.parts;
                    let part = task % schedule.parts;
                    let (start, windows) = &tiles[part];
                    let block_start = trace_block * plan.num_positions_per_block;
                    let mut shifts = vec![0usize; rows_per_ring];
                    visit_segment_ring_row_range::<D>(
                        self,
                        block_start + start,
                        block_start + start + windows.len() / plan.n_a,
                        |ring, selected_rows, committed_zero_masks| {
                            if *budget + rows_per_ring > MAX_WIDE_ACCUMULATIONS {
                                flush_digit_accumulators(accumulators, reduced);
                                *budget = 0;
                            }
                            let position = ring - block_start - start;
                            for column in 0..num_columns {
                                let mut len = 0;
                                for (row_offset, (row_indices, &mask)) in selected_rows
                                    .chunks_exact(num_columns)
                                    .zip(committed_zero_masks)
                                    .enumerate()
                                {
                                    let hot = row_indices[column];
                                    shifts[len] = row_offset * self.one_hot_k + usize::from(hot);
                                    len += usize::from(row_is_committed(hot, mask, column));
                                }
                                for a in 0..plan.n_a {
                                    windows[position * plan.n_a + a].accumulate(
                                        &mut accumulators[column * plan.n_a + a],
                                        &shifts[..len],
                                    );
                                }
                            }
                            *budget += rows_per_ring;
                        },
                    )
                },
            )?;
        }
        Ok(partials
            .into_par_iter()
            .map(|(mut accumulators, mut reduced, _)| {
                flush_digit_accumulators(&mut accumulators, &mut reduced);
                reduced
            })
            .collect::<Vec<_>>())
    }
}
