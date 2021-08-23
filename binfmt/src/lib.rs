#![deny(warnings)]
#![allow(clippy::wrong_self_convention)]

pub mod traits;

pub mod debug;

pub mod fmt;
pub mod howto;
pub mod sym;

#[cfg(feature = "elf")]
pub mod elf;

#[cfg(feature = "elf32")]
pub mod elf32;

#[cfg(feature = "elf64")]
pub mod elf64;

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

#[cfg(feature = "xo65")]
pub mod xo65;

#[cfg(feature = "o65")]
pub mod o65;

pub mod binary;

#[macro_use]
extern crate bytemuck;

extern crate lazy_static;

macro_rules! collect_dashed_idents{
    ($($ident:ident)-+) => {
        ::core::concat!("" $(, ::core::stringify!($ident), )"-"* )
    }
}

macro_rules! define_formats{
    [$($(#[$meta:meta])* $($fmt:ident)-*),* $(,)?] => {
        lazy_static::lazy_static!{
            static ref BINARY_FORMATS_BY_NAME: std::collections::HashMap<String,Box<(dyn crate::fmt::Binfmt + Sync + Send)>> = {
                let mut map = std::collections::HashMap::<String,Box<(dyn crate::fmt::Binfmt + Sync + Send)>>::new();
                $(
                    $($meta)*{
                        let fmt = Box::new(crate:: $($fmt)::* ::create_format());
                        map.insert(String::from(collect_dashed_idents!($($fmt)-*)),fmt);
                    }
                )*

                map
            };
        }
    }
}

define_formats![binary];
