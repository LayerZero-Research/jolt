fn main() {
    println!("Target: {}", std::env::consts::ARCH);
    println!("OS: {}", std::env::consts::OS);
    println!("Family: {}", std::env::consts::FAMILY);
}