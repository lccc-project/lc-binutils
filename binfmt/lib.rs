#![deny(warnings)]

pub mod traits;

#[cfg(feature = "elf")]
pub mod elf;

#[cfg(feature = "coff")]
pub mod coff;

#[cfg(feature = "pe")]
pub mod pe;

#[cfg(feature = "macho")]
pub mod macho;

#[cfg(feature = "aout")]
pub mod aout;
