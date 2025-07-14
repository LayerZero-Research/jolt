use ark_bn254::Fr;
use jolt_core::host;

#[allow(dead_code)]
fn serialize_and_print_size(name: &str, item: &impl ark_serialize::CanonicalSerialize) {
    use std::fs::File;
    let mut file = File::create("temp_file").unwrap();
    item.serialize_compressed(&mut file).unwrap();
    let file_size_bytes = file.metadata().unwrap().len();
    let file_size_kb = file_size_bytes as f64 / 1024.0;
    let file_size_mb = file_size_kb / 1024.0;
    println!("{name:<30} : {file_size_mb:.3} MB");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <guest_name>", args[0]);
        println!("Example: {} sha3-chain-guest", args[0]);
        return;
    }
    
    let guest_name = &args[1];
    
    let mut tasks = Vec::new();
    let mut program = host::Program::new(guest_name);
    let mut inputs = vec![];
    inputs.append(&mut postcard::to_stdvec(&[5u8; 32]).unwrap());
    inputs.append(&mut postcard::to_stdvec(&100u32).unwrap());

    let task = move || {
        let (trace, final_memory_state, io_device) = program.trace(&inputs);
        let (bytecode, init_memory_state) = program.decode();
        let result = program.trace_analyze::<Fr>(&inputs);

        let filename = format!("{}_RV64_trace.txt", guest_name);
        // Write trace analysis to file
        match result.write_trace_analysis::<Fr>(&filename) {
            Ok(_) => println!("✅ Saved complete trace analysis to: {}", filename),
            Err(e) => println!("❌ Failed to save trace analysis: {}", e),
        }

        // Continue with proof generation using the original data
        // println!("\n=== STARTING PROOF GENERATION ===");

        // let preprocessing: JoltProverPreprocessing<Fr, Dory<KeccakTranscript>, KeccakTranscript> =
        //     RV32IJoltVM::prover_preprocess(
        //         bytecode.clone(),
        //         io_device.memory_layout.clone(),
        //         init_memory_state,
        //         1 << 18,
        //         1 << 18,
        //         1 << 20,
        //     );

        // let (jolt_proof, program_io, _) =
        //     <RV32IJoltVM as Jolt<32, Fr, Dory<KeccakTranscript>, KeccakTranscript>>::prove(
        //         io_device,
        //         trace,
        //         final_memory_state,
        //         preprocessing.clone(),
        //     );

        // let verifier_preprocessing =
        //     JoltVerifierPreprocessing::<Fr, Dory<KeccakTranscript>, KeccakTranscript>::from(
        //         &preprocessing,
        //     );

        // println!("Proof sizing:");
        // serialize_and_print_size("jolt_proof", &jolt_proof);
        // serialize_and_print_size(" jolt_proof.commitments", &jolt_proof.commitments);
        // serialize_and_print_size(" jolt_proof.r1cs", &jolt_proof.r1cs);
        // serialize_and_print_size(" jolt_proof.bytecode", &jolt_proof.bytecode);
        // serialize_and_print_size(" jolt_proof.ram", &jolt_proof.ram);
        // serialize_and_print_size(" jolt_proof.registers", &jolt_proof.registers);
        // serialize_and_print_size(
        //     " jolt_proof.instruction_lookups",
        //     &jolt_proof.instruction_lookups,
        // );
        // serialize_and_print_size(" jolt_proof.opening_proof", &jolt_proof.opening_proof);

        // let verification_result =
        //     RV32IJoltVM::verify(verifier_preprocessing, jolt_proof, program_io, None);
        // assert!(
        //     verification_result.is_ok(),
        //     "Verification failed with error: {:?}",
        //     verification_result.err()
        // );
    };

    tasks.push((
        tracing::info_span!("Example_E2E"),
        Box::new(task) as Box<dyn FnOnce()>,
    ));

    // Execute the tasks
    for (span, task) in tasks {
        span.in_scope(|| {
            task();
        });
    }
}
