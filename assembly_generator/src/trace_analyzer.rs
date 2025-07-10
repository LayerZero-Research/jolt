use jolt_core::host::Program;
use jolt_core::field::JoltField;
use ark_bn254::Fr;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <input_size>", args[0]);
        std::process::exit(1);
    }
    
    let input_size: usize = args[1].parse().unwrap_or(32);
    let input_data = vec![5u8; input_size];  // Use specified input size
    
    let mut program = Program::new("inline_sha256_benchmark");
    let inputs = postcard::to_stdvec(&input_data).unwrap();
    
    // Get trace and analyze
    let summary = program.trace_analyze::<Fr>(inputs.as_slice());
    
    println!("=== INLINE SHA256 TRACE ANALYSIS ===");
    println!("Input size: {} bytes", input_size);
    println!("Trace length: {}", summary.trace_len());
    println!("Bytecode size: {}", summary.bytecode.len());
    println!("Memory initialization entries: {}", summary.memory_init.len());
    
    // Count instruction types
    let instruction_counts = summary.analyze::<Fr>();
    println!("\n=== INSTRUCTION BREAKDOWN ===");
    println!("Top 15 most frequent instructions:");
    for (i, (instruction, count)) in instruction_counts.iter().take(15).enumerate() {
        println!("{:2}. {:<15} | {}", i+1, instruction, count);
    }
    
    // Look for inline SHA256 instructions specifically
    println!("\n=== INLINE SHA256 INSTRUCTIONS ===");
    let inline_count = instruction_counts.iter()
        .filter(|(name, _)| name.contains("sha256") || name.contains("SHA256"))
        .fold(0, |acc, (_, count)| acc + count);
    
    if inline_count > 0 {
        println!("Found {} inline SHA256 instructions", inline_count);
    } else {
        println!("No inline SHA256 instructions found (check if feature is enabled)");
    }
} 