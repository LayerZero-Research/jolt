use std::fs;
use std::path::Path;

fn main() {
    let provable_macro_path = Path::new("src/provable_macro.rs");

    if !provable_macro_path.exists() {
        let fallback_content = r#"macro_rules! provable_with_config {
    ($item: item) => {
        #[jolt::provable(
            max_input_size = 1048576,
            max_output_size = 4096,
            max_untrusted_advice_size = 0,
            max_trusted_advice_size = 0,
            heap_size = 268435456,
            stack_size = 33554432,
            max_trace_length = 134217728
        )]
        $item
    };
}"#;
        fs::write(provable_macro_path, fallback_content).unwrap();
        println!("cargo:warning=Created fallback provable_macro.rs with default macro");
    } else {
        println!("cargo:rerun-if-changed=src/provable_macro.rs");
    }
}
