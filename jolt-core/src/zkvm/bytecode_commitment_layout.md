# Bytecode Commitment Layouts (AddressMajor vs CycleMajor)

This note explains how bytecode chunk polynomials are laid out, committed, and embedded into
the Stage-8 Main matrix in committed mode.

## Symbols

- `K`: lanes per bytecode chunk (`k_chunk`)
- `T`: cycle length used by that polynomial (`bytecode_T` or Main `T`)
- `C`: number of matrix columns (`num_columns`)
- `R`: number of matrix rows (`R = (K * T) / C`)

Each bytecode chunk polynomial has `K * T` coefficients and is committed as a `R x C` matrix
in row-major order (row first, then column).

---

## 1) Layout-specific linear index

The same `(lane, cycle)` value maps differently by layout:

- `CycleMajor`: `index = lane * T + cycle`
- `AddressMajor`: `index = cycle * K + lane`

This mapping is implemented by `DoryLayout::address_cycle_to_index` in
`jolt-core/src/poly/commitment/dory/dory_globals.rs`.

---

## 2) How rows are formed from columns (`C`)

For both layouts, matrix coordinates are:

- `row = index / C`
- `col = index % C`

So yes: changing `C` changes row packing, therefore it changes commitments.

---

## 3) AddressMajor picture

Linear order is cycle-first:

```text
[t0:a0, t0:a1, ..., t0:a(K-1), t1:a0, t1:a1, ..., t1:a(K-1), ...]
```

If `C = m * K`, each matrix row packs `m` cycles.

```text
row0: t0 all lanes | t1 all lanes | ... | t(m-1) all lanes
row1: tm all lanes | t(m+1) all lanes | ...
...
```

So:

- `cycles_per_row = C / K`
- increasing `C` means more cycles packed per row
- decreasing `C` means fewer cycles per row

### Tiny example (`K=4`)

With `C=4` (1 cycle/row):

```text
row0: t0:a0 t0:a1 t0:a2 t0:a3
row1: t1:a0 t1:a1 t1:a2 t1:a3
row2: t2:a0 t2:a1 t2:a2 t2:a3
...
```

With `C=8` (2 cycles/row):

```text
row0: t0:a0 t0:a1 t0:a2 t0:a3 | t1:a0 t1:a1 t1:a2 t1:a3
row1: t2:a0 t2:a1 t2:a2 t2:a3 | t3:a0 t3:a1 t3:a2 t3:a3
...
```

---

## 4) CycleMajor picture

Linear order is lane-first:

```text
[a0:t0, a0:t1, ..., a0:t(T-1), a1:t0, a1:t1, ..., a1:t(T-1), ...]
```

If `T` changes, lane stride changes (`lane * T`), so row membership changes.

### Example (`K=2`, `C=4`)

`T=4`:

```text
row0: a0:t0 a0:t1 a0:t2 a0:t3
row1: a1:t0 a1:t1 a1:t2 a1:t3
```

`T=8`:

```text
row0: a0:t0 a0:t1 a0:t2 a0:t3
row1: a0:t4 a0:t5 a0:t6 a0:t7
row2: a1:t0 a1:t1 a1:t2 a1:t3
row3: a1:t4 a1:t5 a1:t6 a1:t7
```

This is why CycleMajor is sensitive to `T`: different `T` means different row commitments.

---

## 5) Committed-mode bytecode dimensions in current code

`committed_bytecode_dimensions` in `jolt-core/src/zkvm/program.rs`:

- `main_num_columns = main_num_columns(log_k_chunk, log_t)`
- `CycleMajor`: `(C, T) = (main_num_columns, max_trace_len)`
- `AddressMajor`: `(C, T) = (min(main_num_columns, K * bytecode_len), bytecode_len)`

So:

- AddressMajor keeps `T = bytecode_len`, but still depends on `C`.
- CycleMajor ties bytecode `T` to the committed preprocessing bound (`max_trace_len`).

---

## 6) How bytecode is built before commit

In `jolt-core/src/zkvm/program.rs`:

- AddressMajor path builds regular chunks (`build_bytecode_chunks_from_program`).
- CycleMajor path builds chunks with main-matrix `T` (`build_bytecode_chunks_for_main_matrix_from_program`),
  which writes bytecode values for `cycle < bytecode_len` and leaves the rest zero.

The low-level writer is in `jolt-core/src/zkvm/bytecode/chunks.rs`.

---

## 7) How Main matrix dimensions are forced in committed mode

In both prover and verifier Stage 8 (`jolt-core/src/zkvm/prover.rs`,
`jolt-core/src/zkvm/verifier.rs`), Main context is initialized with:

- `T = padded_trace_len` (runtime padded trace)
- `C = trusted.bytecode_num_columns` (from committed preprocessing)

So committed mode replays bytecode's agreed width `C` for Stage 8 batching.

Additionally, prover-side padding enforces:

- `padded_trace_len >= bytecode_size`
- `padded_trace_len >= trusted.bytecode_T`

which is required for CycleMajor row alignment.

---

## 8) Practical takeaway

A bytecode commitment is bound to:

- layout (`AddressMajor` / `CycleMajor`)
- `C` (columns)
- `T` (bytecode cycle domain used for indexing)

Changing any of these changes row commitments, so a different commitment is required.

