use std::time::Instant;

pub fn main() {
    let target_dir = "/tmp/jolt-guest-targets";
    let mut program = guest::compile_ec_bench(target_dir);

    let prover_preprocessing = guest::preprocess_prover_ec_bench(&mut program);
    let verifier_preprocessing =
        guest::verifier_preprocessing_from_prover_ec_bench(&prover_preprocessing);

    let prove_ec_bench = guest::build_prover_ec_bench(program, prover_preprocessing);
    let verify_ec_bench = guest::build_verifier_ec_bench(verifier_preprocessing);

    let input = 1;
    let now = Instant::now();
    let (output, proof, program_io) = prove_ec_bench(input);
    println!("Prover runtime: {} s", now.elapsed().as_secs_f64());
    let is_valid = verify_ec_bench(input, output, program_io.panic, proof);

    println!("output: {}", output);
    println!("valid: {is_valid}");
}
