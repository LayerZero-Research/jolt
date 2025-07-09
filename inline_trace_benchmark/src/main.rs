use std::path::PathBuf;
use std::fs::File;
use std::io::Read;

use jolt_core::host;
use jolt_core::jolt::vm::rv32i_vm::RV32IJoltVM;
use jolt_core::jolt::vm::{Jolt, JoltProverPreprocessing, JoltVerifierPreprocessing};
use jolt_core::poly::commitment::dory::DoryCommitmentScheme as Dory;
use jolt_core::utils::transcript::KeccakTranscript;
use ark_bn254::Fr;

fn format_trace_data(result: &jolt_core::host::analyze::ProgramSummary, instruction_counts: &[(&'static str, usize)]) -> String {
    let mut output = String::new();
    
    // Header
    output.push_str("=== JOLT TRACE DATA ANALYSIS ===\n");
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    output.push_str(&format!("Generated at timestamp: {}\n", timestamp));
    output.push_str("\n");
    
    // Basic statistics
    output.push_str("=== BASIC STATISTICS ===\n");
    output.push_str(&format!("Trace length: {}\n", result.trace_len()));
    output.push_str(&format!("Bytecode instructions: {}\n", result.bytecode.len()));
    output.push_str(&format!("Memory init entries: {}\n", result.memory_init.len()));
    output.push_str("\n");
    
    // Instruction frequency analysis
    output.push_str("=== INSTRUCTION FREQUENCY ANALYSIS ===\n");
    output.push_str(&format!("{:<20} | {:>10} | {:>8}\n", "Instruction", "Count", "Percent"));
    output.push_str(&format!("{:-<20}-+-{:->10}-+-{:->8}\n", "", "", ""));
    
    for (instruction, count) in instruction_counts.iter().take(20) {
        let percentage = (*count as f64 / result.trace_len() as f64) * 100.0;
        output.push_str(&format!("{:<20} | {:>10} | {:>7.2}%\n", instruction, count, percentage));
    }
    output.push_str("\n");
    
    // Memory layout analysis
    output.push_str("=== MEMORY LAYOUT ANALYSIS ===\n");
    if !result.memory_init.is_empty() {
        let min_addr = result.memory_init.iter().map(|(addr, _)| *addr).min().unwrap();
        let max_addr = result.memory_init.iter().map(|(addr, _)| *addr).max().unwrap();
        output.push_str(&format!("Memory address range: 0x{:x} - 0x{:x}\n", min_addr, max_addr));
        output.push_str(&format!("Memory span: {} bytes\n", max_addr - min_addr + 1));
        output.push_str(&format!("Total memory entries: {}\n", result.memory_init.len()));
    } else {
        output.push_str("No memory initialization data\n");
    }
    output.push_str("\n");
    
    // Sample trace entries
    output.push_str("=== SAMPLE TRACE ENTRIES (first 10) ===\n");
    for (i, cycle) in result.trace.iter().take(10).enumerate() {
        output.push_str(&format!("{:4}: {:?}\n", i, cycle));
    }
    
    if result.trace.len() > 10 {
        output.push_str(&format!("... and {} more trace entries\n", result.trace.len() - 10));
    }
    output.push_str("\n");
    
    // Sample bytecode entries
    output.push_str("=== SAMPLE BYTECODE (first 10) ===\n");
    for (i, instruction) in result.bytecode.iter().take(10).enumerate() {
        output.push_str(&format!("{:4}: {:?}\n", i, instruction));
    }
    
    if result.bytecode.len() > 10 {
        output.push_str(&format!("... and {} more bytecode instructions\n", result.bytecode.len() - 10));
    }
    output.push_str("\n");
    
    // Sample memory initialization
    output.push_str("=== SAMPLE MEMORY INITIALIZATION (first 10) ===\n");
    for (i, (addr, value)) in result.memory_init.iter().take(10).enumerate() {
        output.push_str(&format!("{:4}: 0x{:x} -> 0x{:02x}\n", i, addr, value));
    }
    
    if result.memory_init.len() > 10 {
        output.push_str(&format!("... and {} more memory entries\n", result.memory_init.len() - 10));
    }
    
    output
}

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

/// Example: How to analyze saved trace data
#[allow(dead_code)]
fn analyze_saved_trace(filename: &str) {
    println!("To load saved trace data, you can use external tools or:");
    println!("1. Use the ProgramSummary::write_to_file() method saves in bincode format");
    println!("2. File contains: trace, bytecode, memory_init, and io_device data");
    println!("3. You can write a separate analysis tool to read: {}", filename);
}

fn main() {
    println!("Hello from main");
    let mut tasks = Vec::new();
    let mut program = host::Program::new("sha2-guest");
    let inputs = postcard::to_stdvec(&vec![5u8; 2048]).unwrap();

    let task = move || {
        let (trace, final_memory_state, io_device) = program.trace(&inputs);
        let (bytecode, init_memory_state) = program.decode();

        // Trace length: 95108
        // Bytecode size: 17120
        let result = program.trace_analyze::<Fr>(&vec![5u8; 2048]);
        
        // Save trace data to file
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let filename = format!("sha2_trace_data_{}.txt", timestamp);
        let filepath = PathBuf::from(&filename);
        
        // Also print analysis before saving
        println!("=== TRACE ANALYSIS ===");
        println!("Trace length: {}", result.trace_len());
        println!("Bytecode size: {}", result.bytecode.len());
        println!("Memory init entries: {}", result.memory_init.len());
        
        // Show top 10 most frequent instructions
        let instruction_counts = result.analyze::<Fr>();
        println!("\n=== TOP 10 INSTRUCTIONS ===");
        for (i, (instruction, count)) in instruction_counts.iter().take(10).enumerate() {
            println!("{:2}. {:<15} | {}", i+1, instruction, count);
        }
        
        // Write trace data to text file manually
        match std::fs::write(&filepath, format_trace_data(&result, &instruction_counts)) {
            Ok(_) => println!("\n✅ Saved complete trace data to: {}", filename),
            Err(e) => println!("\n❌ Failed to save trace data: {}", e),
        }
        
        // Continue with proof generation using the original data
        println!("\n=== STARTING PROOF GENERATION ===");

        let preprocessing: JoltProverPreprocessing<Fr, Dory<KeccakTranscript>, KeccakTranscript> =
            RV32IJoltVM::prover_preprocess(
                bytecode.clone(),
                io_device.memory_layout.clone(),
                init_memory_state,
                1 << 18,
                1 << 18,
                1 << 20,
            );

        let (jolt_proof, program_io, _) = <RV32IJoltVM as Jolt<32, Fr, Dory<KeccakTranscript>, KeccakTranscript>>::prove(
            io_device,
            trace,
            final_memory_state,
            preprocessing.clone(),
        );

        let verifier_preprocessing =
            JoltVerifierPreprocessing::<Fr, Dory<KeccakTranscript>, KeccakTranscript>::from(&preprocessing);

        println!("Proof sizing:");
        serialize_and_print_size("jolt_proof", &jolt_proof);
        serialize_and_print_size(" jolt_proof.commitments", &jolt_proof.commitments);
        serialize_and_print_size(" jolt_proof.r1cs", &jolt_proof.r1cs);
        serialize_and_print_size(" jolt_proof.bytecode", &jolt_proof.bytecode);
        serialize_and_print_size(" jolt_proof.ram", &jolt_proof.ram);
        serialize_and_print_size(" jolt_proof.registers", &jolt_proof.registers);
        serialize_and_print_size(
            " jolt_proof.instruction_lookups",
            &jolt_proof.instruction_lookups,
        );
        serialize_and_print_size(" jolt_proof.opening_proof", &jolt_proof.opening_proof);

        let verification_result =
            RV32IJoltVM::verify(verifier_preprocessing, jolt_proof, program_io, None);
        assert!(
            verification_result.is_ok(),
            "Verification failed with error: {:?}",
            verification_result.err()
        );
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