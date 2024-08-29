use arch_ops::clever::CleverPrinter;

use crate::howto::{HowTo, HowToError};

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
    PltPcrel,
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
    Some(Elf64CleverHowTo::PltPcrel),
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
            crate::howto::RelocCode::CleverShort => Self::from_relnum(9),
            crate::howto::RelocCode::CleverShortPcrel => Self::from_relnum(10),
            _ => None,
        }
    }

    fn reloc_num(&self) -> u32 {
        match self {
            Elf64CleverHowTo::None => 0,
            Elf64CleverHowTo::Abs16 => 1,
            Elf64CleverHowTo::Abs32 => 2,
            Elf64CleverHowTo::Abs64 => 3,
            Elf64CleverHowTo::Rel16 => 5,
            Elf64CleverHowTo::Rel32 => 6,
            Elf64CleverHowTo::Rel64 => 7,
            Elf64CleverHowTo::Simm => 8,
            Elf64CleverHowTo::SimmRel => 9,
            Elf64CleverHowTo::RelaxLong => 10,
            Elf64CleverHowTo::RelaxLongPcrel => 11,
            Elf64CleverHowTo::RelaxShort => 12,
            Elf64CleverHowTo::RelaxShortPcrel => 13,
            Elf64CleverHowTo::Got => 16,
            Elf64CleverHowTo::GotPcrel => 17,
            Elf64CleverHowTo::Plt => 18,
            Elf64CleverHowTo::PltPcrel => 19,
            Elf64CleverHowTo::RelaxGot => 20,
            Elf64CleverHowTo::RelaxGotPcrel => 21,
            Elf64CleverHowTo::RelaxPlt => 22,
            Elf64CleverHowTo::RelaxPltPcrel => 23,
            Elf64CleverHowTo::Dynent => 24,
        }
    }

    fn name(&self) -> &'static str {
        match self {
            Elf64CleverHowTo::None => "R_CLEVER_NONE",
            Elf64CleverHowTo::Abs16 => "R_CLEVER_ABS16",
            Elf64CleverHowTo::Abs32 => "R_CLEVER_ABS32",
            Elf64CleverHowTo::Abs64 => "R_CLEVER_ABS64",
            Elf64CleverHowTo::Rel16 => "R_CLEVER_REL16",
            Elf64CleverHowTo::Rel32 => "R_CLEVER_REL32",
            Elf64CleverHowTo::Rel64 => "R_CLEVER_REL64",
            Elf64CleverHowTo::Simm => "R_CLEVER_SIMM",
            Elf64CleverHowTo::SimmRel => "R_CLEVER_SIMMREL",
            Elf64CleverHowTo::RelaxLong => "R_CLEVER_RELAX_LONG",
            Elf64CleverHowTo::RelaxLongPcrel => "R_CLEVER_RELAX_LONG_PCREL",
            Elf64CleverHowTo::RelaxShort => "R_CLEVER_RELAX_SHORT",
            Elf64CleverHowTo::RelaxShortPcrel => "R_CLEVER_RELAX_SHORT_PCREL",
            Elf64CleverHowTo::Got => "R_CLEVER_GOT",
            Elf64CleverHowTo::GotPcrel => "R_CLEVER_GOTPCREL",
            Elf64CleverHowTo::Plt => "R_CLEVER_PLT",
            Elf64CleverHowTo::PltPcrel => "R_CLEVER_PLTPCREL",
            Elf64CleverHowTo::RelaxGot => "R_CLEVER_RELAX_GOT",
            Elf64CleverHowTo::RelaxGotPcrel => "R_CLEVER_RELAX_GOTPCREL",
            Elf64CleverHowTo::RelaxPlt => "R_CLEVER_RELAX_PLT",
            Elf64CleverHowTo::RelaxPltPcrel => "R_CLEVER_RELAX_PLT_PCREL",
            Elf64CleverHowTo::Dynent => "R_CLEVER_DYNENT",
        }
    }

    fn reloc_size(&self) -> usize {
        match self {
            Elf64CleverHowTo::None => 0,
            Elf64CleverHowTo::Abs16 => 2,
            Elf64CleverHowTo::Abs32 => 4,
            Elf64CleverHowTo::Abs64 => 8,
            Elf64CleverHowTo::Rel16 => 2,
            Elf64CleverHowTo::Rel32 => 4,
            Elf64CleverHowTo::Rel64 => 8,
            Elf64CleverHowTo::Simm => 2,
            Elf64CleverHowTo::SimmRel => 2,
            Elf64CleverHowTo::RelaxLong => 10, // 8 byte immediate + Prefix
            Elf64CleverHowTo::RelaxLongPcrel => 10,
            Elf64CleverHowTo::RelaxShort => 10,
            Elf64CleverHowTo::RelaxShortPcrel => 10,
            Elf64CleverHowTo::Got => 8,
            Elf64CleverHowTo::GotPcrel => 8,
            Elf64CleverHowTo::Plt => 8,
            Elf64CleverHowTo::PltPcrel => 8,
            Elf64CleverHowTo::RelaxGot => 10,
            Elf64CleverHowTo::RelaxGotPcrel => 10,
            Elf64CleverHowTo::RelaxPlt => 10,
            Elf64CleverHowTo::RelaxPltPcrel => 10,
            Elf64CleverHowTo::Dynent => 4,
        }
    }

    fn pcrel(&self) -> bool {
        matches!(
            self,
            Self::Rel16
                | Self::Rel32
                | Self::Rel64
                | Self::SimmRel
                | Self::RelaxLongPcrel
                | Self::RelaxShortPcrel
                | Self::GotPcrel
                | Self::PltPcrel
                | Self::RelaxGotPcrel
                | Self::RelaxPltPcrel
        )
    }

    fn is_relax(&self) -> bool {
        matches!(
            self,
            Self::RelaxLong
                | Self::RelaxLongPcrel
                | Self::RelaxShort
                | Self::RelaxShortPcrel
                | Self::RelaxGot
                | Self::RelaxGotPcrel
                | Self::RelaxPlt
                | Self::RelaxPltPcrel
        )
    }

    fn relax_size(&self, _addr: u128, _at_addr: u128) -> Option<usize> {
        None // Don't support relaxations yet
    }

    fn apply<'a>(
        &self,
        addr: u128,
        at_addr: u128,
        region: &'a mut [u8],
    ) -> Result<&'a mut [u8], crate::howto::HowToError> {
        match self {
            Elf64CleverHowTo::None => Ok(region),
            Elf64CleverHowTo::Abs16 | Elf64CleverHowTo::Abs32 | Elf64CleverHowTo::Abs64 => {
                let size = self.reloc_size();
                let size_bits = (size as u32) << 3;

                let max = (1u128 << size_bits) - 1;

                if addr > max {
                    Err(HowToError::UnsignedOverflow)
                } else {
                    let bytes = addr.to_le_bytes();

                    region.copy_from_slice(&bytes[..size]);
                    Ok(region)
                }
            }
            Elf64CleverHowTo::Rel16 | Elf64CleverHowTo::Rel32 | Elf64CleverHowTo::Rel64 => {
                let val = (at_addr as i128) - (addr as i128);

                let size = self.reloc_size();
                let size_bits = (size as u32) << 3;

                let max = (1i128 << (size_bits - 1)) - 1;
                let min = max.wrapping_add(1);

                if !(min..=max).contains(&val) {
                    Err(HowToError::SignedOverflow)
                } else {
                    let bytes = val.to_le_bytes();

                    region.copy_from_slice(&bytes[..size]);
                    Ok(region)
                }
            }
            Elf64CleverHowTo::Simm => {
                if addr > 0xFFF {
                    Err(HowToError::UnsignedOverflow)
                } else {
                    region[0] = (region[0] & 0xF0) | (addr >> 8) as u8;
                    region[1] = addr as u8;

                    Ok(region)
                }
            }
            Elf64CleverHowTo::SimmRel => {
                let val = (at_addr as i128) - (addr as i128);
                if !(-0x800..=0x7ff).contains(&val) {
                    Err(HowToError::UnsignedOverflow)
                } else {
                    region[0] = (region[0] & 0xF0) | (val >> 8) as u8;
                    region[1] = val as u8;

                    Ok(region)
                }
            }
            Elf64CleverHowTo::RelaxLong => todo!(),
            Elf64CleverHowTo::RelaxLongPcrel => todo!(),
            Elf64CleverHowTo::RelaxShort => todo!(),
            Elf64CleverHowTo::RelaxShortPcrel => todo!(),
            Elf64CleverHowTo::Got => todo!(),
            Elf64CleverHowTo::GotPcrel => todo!(),
            Elf64CleverHowTo::Plt => todo!(),
            Elf64CleverHowTo::PltPcrel => todo!(),
            Elf64CleverHowTo::RelaxGot => todo!(),
            Elf64CleverHowTo::RelaxGotPcrel => todo!(),
            Elf64CleverHowTo::RelaxPlt => todo!(),
            Elf64CleverHowTo::RelaxPltPcrel => todo!(),
            Elf64CleverHowTo::Dynent => todo!(),
        }
    }

    fn valid_in(
        &self,
        _output_ty: crate::howto::RelocOutput,
        _sym_vis: &crate::sym::Symbol,
    ) -> bool {
        todo!()
    }
}

pub fn create_format() -> Elf64Format<Elf64CleverHowTo> {
    super::Elf64Format::new(
        super::consts::EM_CLEVER,
        super::consts::ELFDATA2LSB,
        "elf64-clever",
        None,
        Some(Box::new(CleverPrinter::new())),
    )
}
