pub type Elf64Format<Howto> = crate::elf::ElfFormat<crate::elf::Elf64, Howto>;

pub use crate::elf::{consts, Elf64};

#[cfg(feature = "x86")]
pub mod x86_64;

#[cfg(feature = "clever")]
pub mod clever;

#[cfg(feature = "holey-bytes")]
pub mod holeybytes;

pub mod genericle {
    pub fn create_format() -> super::Elf64Format<crate::elf::ElfHowToUnknown> {
        super::Elf64Format::new(
            super::consts::EM_NONE,
            super::consts::ELFDATA2LSB,
            "elf64-genericle",
            None,
            None,
        )
    }
}

pub mod genericbe {
    pub fn create_format() -> super::Elf64Format<crate::elf::ElfHowToUnknown> {
        super::Elf64Format::new(
            super::consts::EM_NONE,
            super::consts::ELFDATA2MSB,
            "elf64-genericbe",
            None,
            None,
        )
    }
}
