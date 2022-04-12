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
                    $(#[$meta])*{
                        let fmt = Box::new(crate:: $($fmt)::* ::create_format());
                        map.insert(String::from(collect_dashed_idents!($($fmt)-*)),fmt);
                    }
                )*

                map
            };
        }

        lazy_static::lazy_static!{
            static ref BINARY_FORMATS: std::vec::Vec<&'static (dyn crate::fmt::Binfmt + Sync + Send)> = {
                let mut vec = std::vec::Vec::new();

                $(
                    $(#[$meta])* {
                        vec.push(&*BINARY_FORMATS_BY_NAME[collect_dashed_idents!($($fmt)-*)]);
                    }
                )*

                vec
            };
        }
    }
}

use std::ops::Deref;

use target_tuples::Target;

#[rustfmt::skip]
define_formats![
    #[cfg(all(feature = "elf32", feature = "w65"))]
    elf32-w65,
    #[cfg(all(feature = "elf32", feature = "x86"))]
    elf32-x86_64,
    #[cfg(all(feature = "elf64", feature = "x86"))]
    elf64-x86_64,
    #[cfg(all(feature = "elf64", feature = "clever"))]
    elf64-clever,
    binary
];

pub fn formats() -> impl Iterator<Item = &'static (dyn crate::fmt::Binfmt + Sync + Send + 'static)>
{
    BINARY_FORMATS.iter().copied()
}

pub fn format_by_name(
    name: &str,
) -> Option<&'static (dyn crate::fmt::Binfmt + Sync + Send + 'static)> {
    BINARY_FORMATS_BY_NAME.get(name).map(Deref::deref)
}

pub fn def_vec_for(targ: &Target) -> &'static (dyn crate::fmt::Binfmt + Sync + Send + 'static) {
    target_tuples::match_targets! {
        match (targ){
            w65-*-elf => &*BINARY_FORMATS_BY_NAME["elf32-w65"],
            w65-*-snes-elf => &*BINARY_FORMATS_BY_NAME["elf32-w65"],
            x86_64-*-elf => &*BINARY_FORMATS_BY_NAME["elf64-x86_64"],
            x86_64-*-*-elf => &*BINARY_FORMATS_BY_NAME["elf64-x86_64"],
            x86_64-*-*-gnu => &*BINARY_FORMATS_BY_NAME["elf64-x86_64"],
            // x86_64-*-*-gnux32 => &*BINARY_FORMATS_BY_NAME["elf32-x86_64"],
            // x86_64-*-*-musl => &*BINARY_FORMATS_BY_NAME["elf64-x86_64"],
            // x86_64-*-*-newlib => &*BINARY_FORMATS_BY_NAME["elf64-x86_64"],
            // x86_64-*-*-uclibc => &*BINARY_FORMATS_BY_NAME["elf64-x86_64"],
            clever-*-elf => &*BINARY_FORMATS_BY_NAME["elf64-clever"],
            clever-*-*-elf => &*BINARY_FORMATS_BY_NAME["elf64-clever"],
            * => panic!("Unknown Target")
        }
    }
}
