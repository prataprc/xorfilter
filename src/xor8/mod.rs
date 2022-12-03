mod builder;
mod filter;

pub use builder::Xor8Builder;
pub use filter::Xor8;

#[cfg(test)]
#[path = "xor8_test.rs"]
mod xor8_test;
