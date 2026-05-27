/// Tiny fixture: a minimal Rust file for scanning
fn main() {
    println!("Hello from the fixture project");
}

pub fn compute(x: u32) -> u32 {
    x * 2
}

pub struct Receipt {
    pub id: String,
}

pub trait Capability {
    fn name(&self) -> &str;
}
