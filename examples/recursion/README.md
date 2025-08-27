
## Generate proof binary
```bash
cargo run --release -p recursion generate --example fibonacci        
```

## Verify proof binary
Use `--embed` to embed the proof bytes directly into the recursion-guest program binary, otherwise the proof bytes are passed as input to the recursion-guest program at runtime.
```bash
cargo run --release -p recursion verify --example fibonacci --embed
```


<!-- feat/rv64 result -->
"deserialize preprocessing": 8641971 RV32IM cycles, 0 virtual cycles
"deserialize count of proofs": 108 RV32IM cycles, 0 virtual cycles
"deserialize proof": 17379798 RV32IM cycles, 0 virtual cycles
"deserialize device": 2834 RV32IM cycles, 0 virtual cycles
Warning: sys_rand is a deterministic PRNG, not a cryptographically secure RNG. Use with caution.
"verification": 1411334354 RV32IM cycles, 0 virtual cycles
1437370144 raw RISC-V instructions + 63015770 virtual instructions = 1500385914 total cycles
bytecode size: 2097152

<!-- Committed polynomials result -->
"deserialize preprocessing": 8655217 RV32IM cycles, 0 virtual cycles
"deserialize count of proofs": 109 RV32IM cycles, 0 virtual cycles
"deserialize proof": 18117135 RV32IM cycles, 0 virtual cycles
"deserialize device": 2825 RV32IM cycles, 0 virtual cycles
Warning: sys_rand is a deterministic PRNG, not a cryptographically secure RNG. Use with caution.
"verification": 1653496086 RV32IM cycles, 0 virtual cycles
1680282451 raw RISC-V instructions + 74288002 virtual instructions = 1754570453 total cycles