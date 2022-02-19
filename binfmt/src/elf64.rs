pub type Elf64Format<Howto> = crate::elf::ElfFormat<crate::elf::Elf64, Howto>;

pub use crate::elf::{consts, Elf64};

#[cfg(feature = "x86")]
pub mod x86_64;

#[cfg(feature = "clever")]
pub mod clever;
