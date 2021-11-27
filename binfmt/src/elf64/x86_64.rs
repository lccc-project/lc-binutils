use crate::howto::{HowTo, HowToError};

use super::consts;

pub enum Elf64X86_64HowTo {
    None,
    Abs64,
    Pc32,
    Got32,
    Plt32,
    Copy,
    GlobDat,
    JumpSlot,
    Rel64,
    GotPcRel,
    Abs32,
    Abs32S,
    Abs16,
    Pc16,
    Abs8,
    Pc8,
    DptMod64,
    DtpOff64,
    TpOff64,
    TlsGd,
    TlsLd,
    DtpOff32,
    GotTpOff,
    TpOff32,
    Pc64,
    GotOff64,
    GotPc32,
}

mod howtos {
    use super::Elf64X86_64HowTo::{self, *};

    pub static RELOCS: [Option<Elf64X86_64HowTo>; 27] = [
        Some(Elf64X86_64HowTo::None),
        Some(Abs64),
        Some(Pc32),
        Some(Got32),
        Some(Plt32),
        Some(Copy),
        Some(GlobDat),
        Some(JumpSlot),
        Some(Rel64),
        Some(GotPcRel),
        Some(Abs32),
        Some(Abs32S),
        Some(Abs16),
        Some(Pc16),
        Some(Abs8),
        Some(Pc8),
        Some(DptMod64),
        Some(DtpOff64),
        Some(TpOff64),
        Some(TlsGd),
        Some(TlsLd),
        Some(DtpOff32),
        Some(GotTpOff),
        Some(TpOff32),
        Some(Pc64),
        Some(GotOff64),
        Some(GotPc32),
    ];
}

impl HowTo for Elf64X86_64HowTo {
    fn from_relnum<'a>(num: u32) -> Option<&'a Self>
    where
        Self: Sized + 'a,
    {
        howtos::RELOCS
            .get(num as usize)
            .map(|x| x.as_ref())
            .unwrap_or(None)
    }
}

pub fn create_format() -> super::Elf64Format<Elf64X86_64HowTo> {
    super::Elf64Format::new(
        super::consts::EM_X86_64,
        consts::ELFDATA2LSB,
        "elf32-w65",
        None,
    )
}
