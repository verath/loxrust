pub type ErrorCallback = Fn(u64, &str) -> ();

pub mod scanner;
pub mod token;
