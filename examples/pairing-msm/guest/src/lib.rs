#![cfg_attr(feature = "guest", no_std)]


use jolt::{test_pairing_output_msm};


#[jolt::provable(memory_size = 102400, max_trace_length = 131072)]
fn pairing_msm(num_elements: u32) -> u32 {  
    test_pairing_output_msm(num_elements as usize);

    return 10;

    
}
