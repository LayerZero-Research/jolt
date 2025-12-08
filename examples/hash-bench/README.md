# Hash Benchmark

Performance benchmarks for optimized hash implementations in `jolt-inlines`.

## How to Run

```bash
cd examples/hash-bench
RUST_LOG=info cargo run --release
```

---

## Results (RV64IMAC Cycles)

### Blake2b

| Size | Aligned | Unaligned |
|------|---------|-----------|
| 64B | **346** | 431 |
| 128B | **195** | 347 |
| 256B | **330** | 633 |
| 512B | **593** | 1195 |

### Blake3 (max 64B)

| Size | Aligned | Unaligned |
|------|---------|-----------|
| 32B | **146** | 192 |
| 64B | **142** | 237 |

### Keccak256

| Size | Aligned | Unaligned |
|------|---------|-----------|
| 32B | **380** | 426 |
| 64B | **382** | 476 |
| 136B | **406** | 741 |
| 256B | **527** | 1007 |
| 512B | **711** | 1851 |

---

## Usage

```rust
// Blake2b
let hash = Blake2b::digest(&input);

// Blake3 (max 64 bytes)
let hash = Blake3::digest(&input);

// Keccak256
let hash = Keccak256::digest(&input);
```

---

## Notes

- **Aligned inputs** use fast direct memory access
- **Unaligned inputs** use safe byte-by-byte reads (slower but correct)
- Runtime alignment detection optimizes for the common case
