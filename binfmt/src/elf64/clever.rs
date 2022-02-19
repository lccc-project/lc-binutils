use crate::howto::HowTo;

use super::Elf64Format;

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Elf64CleverHowTo {
    None,
    Abs16,
    Abs32,
    Abs64,
    Rel16,
    Rel32,
    Rel64,
    Simm,
    SimmRel,
    RelaxLong,
    RelaxLongPcrel,
    RelaxShort,
    RelaxShortPcrel,
    Got,
    GotPcrel,
    Plt,
    PltRcrel,
    RelaxGot,
    RelaxGotPcrel,
    RelaxPlt,
    RelaxPltPcrel,
    Dynent,
}

static HOWTO: [Option<Elf64CleverHowTo>; 32] = [
    Some(Elf64CleverHowTo::None),
    Some(Elf64CleverHowTo::Abs16),
    Some(Elf64CleverHowTo::Abs32),
    Some(Elf64CleverHowTo::Abs64),
    None,
    Some(Elf64CleverHowTo::Rel16),
    Some(Elf64CleverHowTo::Rel32),
    Some(Elf64CleverHowTo::Rel64),
    Some(Elf64CleverHowTo::Simm),
    Some(Elf64CleverHowTo::SimmRel),
    Some(Elf64CleverHowTo::RelaxLong),
    Some(Elf64CleverHowTo::RelaxLongPcrel),
    Some(Elf64CleverHowTo::RelaxShort),
    Some(Elf64CleverHowTo::RelaxShortPcrel),
    None,
    None,
    Some(Elf64CleverHowTo::Got),
    Some(Elf64CleverHowTo::GotPcrel),
    Some(Elf64CleverHowTo::Plt),
    Some(Elf64CleverHowTo::PltRcrel),
    Some(Elf64CleverHowTo::RelaxGot),
    Some(Elf64CleverHowTo::RelaxGotPcrel),
    Some(Elf64CleverHowTo::RelaxPlt),
    Some(Elf64CleverHowTo::RelaxPltPcrel),
    Some(Elf64CleverHowTo::Dynent),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
];

impl HowTo for Elf64CleverHowTo {
    fn from_relnum<'a>(num: u32) -> Option<&'a Self>
    where
        Self: Sized + 'a,
    {
        HOWTO.get(num as usize).and_then(Option::as_ref)
    }

    fn from_reloc_code<'a>(code: crate::howto::RelocCode) -> Option<&'a Self>
    where
        Self: Sized + 'a,
    {
        match code {
            crate::howto::RelocCode::None => Self::from_relnum(0),
            crate::howto::RelocCode::Abs { addr_width: 16 } => Self::from_relnum(1),
            crate::howto::RelocCode::Abs { addr_width: 32 } => Self::from_relnum(2),
            crate::howto::RelocCode::Abs { addr_width: 64 } => Self::from_relnum(3),
            crate::howto::RelocCode::Rel { addr_width: 16 } => Self::from_relnum(5),
            crate::howto::RelocCode::Rel { addr_width: 32 } => Self::from_relnum(6),
            crate::howto::RelocCode::Rel { addr_width: 64 } => Self::from_relnum(7),
            crate::howto::RelocCode::Got { addr_width: 64 } => Self::from_relnum(16),
            crate::howto::RelocCode::RelGot { addr_wdith: 64 } => Self::from_relnum(17),
            crate::howto::RelocCode::Plt { addr_width: 64 } => Self::from_relnum(18),
            crate::howto::RelocCode::RelPlt { addr_width: 64 } => Self::from_relnum(19),
            crate::howto::RelocCode::DynSymEntry { width: 8 } => Self::from_relnum(25),
            _ => None,
        }
    }

    fn reloc_num(&self) -> u32 {
        match self {
            Elf64CleverHowTo::None => 0,
            Elf64CleverHowTo::Abs16 => todo!(),
            Elf64CleverHowTo::Abs32 => todo!(),
            Elf64CleverHowTo::Abs64 => todo!(),
            Elf64CleverHowTo::Rel16 => todo!(),
            Elf64CleverHowTo::Rel32 => todo!(),
            Elf64CleverHowTo::Rel64 => todo!(),
            Elf64CleverHowTo::Simm => todo!(),
            Elf64CleverHowTo::SimmRel => todo!(),
            Elf64CleverHowTo::RelaxLong => todo!(),
            Elf64CleverHowTo::RelaxLongPcrel => todo!(),
            Elf64CleverHowTo::RelaxShort => todo!(),
            Elf64CleverHowTo::RelaxShortPcrel => todo!(),
            Elf64CleverHowTo::Got => todo!(),
            Elf64CleverHowTo::GotPcrel => todo!(),
            Elf64CleverHowTo::Plt => todo!(),
            Elf64CleverHowTo::PltRcrel => todo!(),
            Elf64CleverHowTo::RelaxGot => todo!(),
            Elf64CleverHowTo::RelaxGotPcrel => todo!(),
            Elf64CleverHowTo::RelaxPlt => todo!(),
            Elf64CleverHowTo::RelaxPltPcrel => todo!(),
            Elf64CleverHowTo::Dynent => todo!(),
        }
    }

    fn name(&self) -> &'static str {
        todo!()
    }

    fn reloc_size(&self) -> usize {
        todo!()
    }

    fn pcrel(&self) -> bool {
        todo!()
    }

    fn is_relax(&self) -> bool {
        todo!()
    }

    fn relax_size(&self, _addr: u128, _at_addr: u128) -> Option<usize> {
        todo!()
    }

    fn apply(
        &self,
        _addr: u128,
        _at_addr: u128,
        _region: &mut [u8],
    ) -> Result<bool, crate::howto::HowToError> {
        todo!()
    }
}

pub fn create_format() -> Elf64Format<Elf64CleverHowTo> {
    super::Elf64Format::new(
        super::consts::EM_CLEVER,
        super::consts::ELFDATA2LSB,
        "elf64-clever",
        None,
    )
}
