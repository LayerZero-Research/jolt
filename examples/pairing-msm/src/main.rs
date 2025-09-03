use std::time::Instant;

pub fn main() {
    let target_dir = "/tmp/jolt-guest-targets";
    let mut program = guest::compile_pairing_msm(target_dir);

    let prover_preprocessing = guest::preprocess_prover_pairing_msm(&mut program);
    let verifier_preprocessing =
        guest::verifier_preprocessing_from_prover_pairing_msm(&prover_preprocessing);

    let prove_pairing_msm = guest::build_prover_pairing_msm(program, prover_preprocessing);
    let verify_pairing_msm = guest::build_verifier_pairing_msm(verifier_preprocessing);

    let input: u32 = 30;
    let now = Instant::now();
    let (output, proof, program_io) = prove_pairing_msm(input);
    println!("Prover runtime: {} s", now.elapsed().as_secs_f64());
    let is_valid = verify_pairing_msm(input, output, program_io.panic, proof);

    println!("output: {output}");
    println!("valid: {is_valid}");
}
