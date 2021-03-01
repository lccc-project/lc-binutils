#![deny(warnings)]
#![allow(clippy::clippy::wrong_self_convention)]

pub mod traits;

pub mod debug;

pub mod fmt;

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

#[cfg(feature = "xir")]
pub mod xir;

#[cfg(feature = "ar")]
pub mod ar;

pub mod binary;

extern crate lazy_static;

use std::{collections::HashMap, io::Read};

use traits::BinaryFile;

pub type BinfmtConstructor =
    Box<(dyn Fn(&mut (dyn Read + '_)) -> std::io::Result<Box<dyn BinaryFile>> + Sync)>;

lazy_static::lazy_static! {
    static ref BINFMTS: HashMap<&'static str,BinfmtConstructor> = {
        #[allow(unused_mut)]
        let mut hm = HashMap::<&'static str,BinfmtConstructor>::new();
        hm.insert("raw", Box::new(|r|Ok(binary::RawBinaryFile::read(r)?)));

        hm
    };

    static ref SELECT: Vec<BinfmtConstructor> = vec![Box::new(|r|Ok(binary::RawBinaryFile::read(r)?))];
}
