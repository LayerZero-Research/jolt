use jolt_sdk::{postcard, serialize_and_print_size};
use std::time::Instant;

pub fn main() {
    solution1();
    // solution2();
}

pub fn solution1() {
    let save_to_disk = std::env::args().any(|arg| arg == "--save");

    let target_dir = "/tmp/jolt-guest-targets";
    let mut program = guest::compile_fib(target_dir);

    let prover_preprocessing = guest::preprocess_prover_fib(&mut program);
    let verifier_preprocessing =
        guest::verifier_preprocessing_from_prover_fib(&prover_preprocessing);

    if save_to_disk {
        serialize_and_print_size(
            "Verifier Preprocessing",
            "/tmp/jolt_verifier_preprocessing.dat",
            &verifier_preprocessing,
        )
        .expect("Could not serialize preprocessing.");
    }

    let prove_fib = guest::build_prover_fib(program, prover_preprocessing);
    let verify_fib = guest::build_verifier_fib(verifier_preprocessing);

    let program_summary = guest::analyze_fib(10);
    program_summary
        .write_to_file("fib_10.txt".into())
        .expect("should write");

    let trace_file = "/tmp/fib_trace.bin";
    guest::trace_fib_to_file(trace_file, 50);
    println!("Trace file written to: {trace_file}.");

    let now = Instant::now();
    let (output, proof, io_device) = prove_fib(50);
    println!("Prover runtime: {} s", now.elapsed().as_secs_f64());

    if save_to_disk {
        serialize_and_print_size("Proof", "/tmp/fib_proof.bin", &proof)
            .expect("Could not serialize proof.");
        serialize_and_print_size("io_device", "/tmp/fib_io_device.bin", &io_device)
            .expect("Could not serialize io_device.");
    }

    let is_valid = verify_fib(50, output, io_device.panic, proof);
    println!("output: {output}");
    println!("valid: {is_valid}");
}

pub fn solution2() {
    let save_to_disk = std::env::args().any(|arg| arg == "--save");

    let target_dir = "/tmp/jolt-guest-targets";
    let memory_config = guest::memory_config_fib();

    let mut builder = jolt_sdk::host::Program::new("fibonacci");
    builder.set_func("fib");
    builder.set_memory_config(memory_config);
    builder.build(target_dir);

    let elf_contents = builder.get_elf_contents().unwrap();

    let mut program = jolt_sdk::guest::program::Program::new(&elf_contents, &memory_config);
    let mut prover_preprocessing = jolt_sdk::guest::prover::preprocess(&program, 1024);
    let mut verifier_preprocessing = jolt_sdk::guest::verifier::preprocess(&program, 1024);

    let n: u32 = 10;
    let mut input_bytes: Vec<u8> = vec![memory_config.max_input_size as u8];
    input_bytes.append(&mut postcard::to_stdvec(&n).unwrap());

    let now = Instant::now();
    let mut output_bytes = vec![0; memory_config.max_output_size as usize];
    let (proof, io_device, _) = jolt_sdk::guest::prover::prove(
        &program,
        &input_bytes,
        &mut output_bytes,
        &prover_preprocessing,
    );
    println!("Prover runtime: {} s", now.elapsed().as_secs_f64());

    let is_valid = jolt_sdk::guest::verifier::verify(
        &input_bytes,
        &output_bytes,
        proof,
        &verifier_preprocessing,
    )
    .is_ok();

    let output = postcard::from_bytes::<u128>(&output_bytes).unwrap();
    println!("output: {output}");
    println!("valid: {is_valid}");
}
