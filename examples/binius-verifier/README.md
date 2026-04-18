# Binius Guest Verifier

This example traces a Binius proof verification inside a Jolt guest.
It consumes serialized Binius verifier artifacts instead of building the Binius circuit inside this repository.
The host CLI can now generate those artifacts for supported Binius examples by shelling out to the sibling `../binius64` checkout.

## Artifact Layout

The Jolt trace driver expects an artifact directory with these files:

- `constraint_system.bin`
- `public.bin`
- `proof.bin`
- optional `log_inv_rate.txt`

You will usually also have a `non_public.bin` file because the Binius prover needs it.
The Jolt trace driver ignores that file once the proof has been created.

## Generate Artifacts With The Host CLI

The new `generate` subcommand drives Binius directly.
By default it targets the `keccak` example with a 1 MiB random message:

```bash
ARTIFACT_DIR=/tmp/binius-jolt-keccak-1m

cargo run -p binius-verifier-example -- generate \
  --artifacts-dir "$ARTIFACT_DIR"
```

That command runs:

1. the Binius example `save` flow,
2. the standalone Binius `prover`,
3. the standalone Binius `verifier`,
4. and writes `log_inv_rate.txt`.

You can override the example and message size:

```bash
cargo run -p binius-verifier-example -- generate \
  --artifacts-dir /tmp/binius-jolt-keccak-4k \
  --example keccak \
  --message-len 4096 \
  --max-len-bytes 4096
```

`sha256` is also supported:

```bash
cargo run -p binius-verifier-example -- generate \
  --artifacts-dir /tmp/binius-jolt-sha256 \
  --example sha256 \
  --message-len 1024 \
  --max-len-bytes 1024
```

## Manual Artifact Workflow

If you want to drive Binius manually, the original workflow still works.
Here is the small `sha256` version:

```bash
ARTIFACT_DIR=/tmp/binius-jolt-sha256
mkdir -p "$ARTIFACT_DIR"

cargo run --manifest-path ../binius64/Cargo.toml -p binius-examples --example sha256 -- \
  save \
  --message-string abc \
  --cs-path "$ARTIFACT_DIR/constraint_system.bin" \
  --pub-witness-path "$ARTIFACT_DIR/public.bin" \
  --non-pub-data-path "$ARTIFACT_DIR/non_public.bin"

cargo run --manifest-path ../binius64/Cargo.toml -p binius-examples --bin prover -- \
  --cs-path "$ARTIFACT_DIR/constraint_system.bin" \
  --pub-witness-path "$ARTIFACT_DIR/public.bin" \
  --non-pub-data-path "$ARTIFACT_DIR/non_public.bin" \
  --proof-path "$ARTIFACT_DIR/proof.bin" \
  --log-inv-rate 1
```

If you want the Jolt trace command to infer the inverse-rate log automatically, write it once:

```bash
printf '1\n' > "$ARTIFACT_DIR/log_inv_rate.txt"
```

Optional sanity check on the Binius side:

```bash
cargo run --manifest-path ../binius64/Cargo.toml -p binius-examples --bin verifier -- \
  --cs-path "$ARTIFACT_DIR/constraint_system.bin" \
  --pub-witness-path "$ARTIFACT_DIR/public.bin" \
  --proof-path "$ARTIFACT_DIR/proof.bin" \
  --log-inv-rate 1
```

## Trace Verification In Jolt

Without `--disk`, the command now runs in measurement-only mode.
It executes the guest, emits cycle-marker logs, and avoids materializing a full trace.

```bash
cargo run -p binius-verifier-example -- trace \
  --artifacts-dir "$ARTIFACT_DIR"
```

Use `--disk` only when you really want the full cycle trace on disk:

```bash
cargo run -p binius-verifier-example -- trace \
  --artifacts-dir "$ARTIFACT_DIR" \
  --disk \
  --trace-file "$ARTIFACT_DIR/guest.trace"
```

## What The Guest Measures

The guest emits cycle markers for three phases:

- `deserialize`
- `setup`
- `verify`

In measurement-only mode, the tracer prints the total cycle count after the guest finishes.
When tracing to disk, it also writes the full trace to the path you provide.

## Keccak Notes

The default 1 MiB `keccak` artifact set is valid, but it is extremely large in the current artifact-driven design.
On the validated run:

- `constraint_system.bin` was about `548 MiB`
- `public.bin` was about `4.0 MiB`
- `proof.bin` was about `430 KiB`

That means guest-side deserialization dominates the runtime.
The 1 MiB `keccak` proof was generated and verified successfully on the Binius side, but the guest-side measurement did not finish in a practical amount of time.

For a smaller `keccak` artifact set with `message_len = 4096`, the guest-side measurement completed with:

- `deserialize`: `73,372,959` cycles
- `setup`: `6,235,988` cycles
- `verify`: `295,336,457` cycles
- total trace length: `388,177,433` cycles
