#![deny(warnings)]
#![allow(unknown_lints, renamed_and_removed_lints)]

pub mod traits;


pub mod disasm;

#[cfg(feature = "w65")]
pub mod w65;

#[cfg(feature = "m6502")]
pub mod m6502;

#[cfg(feature = "x86")]
pub mod x86;

#[cfg(feature = "clever")]
pub mod clever;

#[cfg(test)]
pub(crate) mod test;

