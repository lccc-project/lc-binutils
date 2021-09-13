#![deny(warnings)]

pub mod traits;

#[cfg(feature = "w65")]
pub mod w65;

#[cfg(feature = "x86")]
pub mod x86;
