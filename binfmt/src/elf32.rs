pub type Elf32Format<Howto> = crate::elf::ElfFormat<crate::elf::Elf32, Howto>;

pub use crate::elf::{consts, Elf32};

#[cfg(feature = "w65")]
pub mod w65;

#[cfg(feature = "x86")]
pub mod x86_64;

pub mod genericle {
    pub fn create_format() -> super::Elf32Format<crate::elf::ElfHowToUnknown> {
        super::Elf32Format::new(
            super::consts::EM_NONE,
            super::consts::ELFDATA2LSB,
            "elf32-genericle",
            None,
            None,
        )
    }
}

pub mod genericbe {
    pub fn create_format() -> super::Elf32Format<crate::elf::ElfHowToUnknown> {
        super::Elf32Format::new(
            super::consts::EM_NONE,
            super::consts::ELFDATA2MSB,
            "elf32-genericbe",
            None,
            None,
        )
    }
}
