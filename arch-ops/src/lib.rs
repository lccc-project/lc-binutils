#![deny(warnings)]

pub mod traits;

#[cfg(feature = "wc65c816")]
pub mod wc65c816;

#[cfg(feature = "x86")]
pub mod x86;

pub mod generators;

#[macro_use]
extern crate tablegen;
