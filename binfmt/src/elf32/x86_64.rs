use crate::elf64::x86_64::Elf64X86_64HowTo;

use super::consts;

pub fn create_format() -> super::Elf32Format<Elf64X86_64HowTo> {
    super::Elf32Format::new(
        super::consts::EM_X86_64,
        consts::ELFDATA2LSB,
        "elf32-x86_64",
        None,
    )
}
