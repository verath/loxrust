pub type ErrorCallback = Fn(u64, &str) -> ();

pub mod expr;
pub mod print;
pub mod scanner;
pub mod token;
