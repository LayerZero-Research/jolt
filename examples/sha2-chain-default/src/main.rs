use std::time::Instant;

pub fn main() {
    let target_dir = "/tmp/jolt-guest-targets";
    let mut program = guest::compile_sha2_chain_default(target_dir);

    let prover_preprocessing = guest::preprocess_prover_sha2_chain_default(&mut program);
    let verifier_preprocessing =
        guest::verifier_preprocessing_from_prover_sha2_chain_default(&prover_preprocessing);

    let prove_sha2_chain_default = guest::build_prover_sha2_chain_default(program, prover_preprocessing);
    let verify_sha2_chain_default = guest::build_verifier_sha2_chain_default(verifier_preprocessing);

    let input = [5u8; 32];
    let iters = 1000;
    let native_output = guest::sha2_chain_default(input, iters);
    let now = Instant::now();
    let (output, proof) = prove_sha2_chain_default(input, iters);
    println!("Prover runtime: {} s", now.elapsed().as_secs_f64());
    let is_valid = verify_sha2_chain_default(input, iters, output, proof);

    assert_eq!(output, native_output, "output mismatch");
    println!("output: {}", hex::encode(output));
    println!("native_output: {}", hex::encode(native_output));
    println!("valid: {is_valid}");
}
