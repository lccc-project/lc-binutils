use crate::howto::{HowTo, HowToError};

use super::consts;

#[non_exhaustive]
pub enum Elf32W65HowTo {
    None,
    Abs24,
    Abs16,
    Rel8,
    Rel16,
    Bank,
    Abs8,
    Direct,
    RelaxJsl,
    RelaxJml,
    RelaxBrl,
    RelaxDirect,
    RelaxAbs,
    RelaxJmp,
}

mod howtos {
    use super::Elf32W65HowTo::{self, *};

    pub static RELOCS: [Option<Elf32W65HowTo>; 16] = [
        Some(Elf32W65HowTo::None),
        Some(Abs24),
        Some(Abs16),
        Some(Rel8),
        Some(Rel16),
        Some(Bank),
        Some(Abs8),
        Some(Direct),
        Option::None,
        Option::None,
        Some(RelaxJsl),
        Some(RelaxJml),
        Some(RelaxBrl),
        Some(RelaxDirect),
        Some(RelaxAbs),
        Some(RelaxJmp),
    ];

    #[allow(dead_code)]
    pub static RELAX_DIRECT_INSTS: &[(u8, u8)] = &[
        (0x6D, 0x65),
        (0x6F, 0x65),
        (0x7D, 0x75),
        (0x7F, 0x75),
        (0xED, 0xE5),
        (0xEF, 0xE5),
        (0xFD, 0xF5),
        (0xFF, 0xF5),
        (0xCD, 0xC5),
        (0xCF, 0xC5),
        (0xDD, 0xD5),
        (0xDF, 0xD5),
        (0xEC, 0xE4),
        (0xCC, 0xC4),
    ];
}

impl HowTo for Elf32W65HowTo {
    fn from_relnum<'a>(num: u32) -> Option<&'a Self>
    where
        Self: Sized + 'a,
    {
        howtos::RELOCS
            .get(num as usize)
            .map(|x| x.as_ref())
            .unwrap_or(None)
    }

    fn from_reloc_code<'a>(code: crate::howto::RelocCode) -> Option<&'a Self>
    where
        Self: Sized + 'a,
    {
        match code {
            crate::howto::RelocCode::None => Self::from_relnum(0),
            crate::howto::RelocCode::Abs { addr_width: 24 } => Self::from_relnum(1),
            crate::howto::RelocCode::Abs { addr_width: 16 } => Self::from_relnum(2),
            crate::howto::RelocCode::Rel { addr_width: 8 } => Self::from_relnum(3),
            crate::howto::RelocCode::Rel { addr_width: 16 } => Self::from_relnum(4),
            crate::howto::RelocCode::AbsShifted {
                addr_width: 3,
                shift: 8,
            } => Self::from_relnum(5),
            crate::howto::RelocCode::Abs { addr_width: 8 } => Self::from_relnum(6),
            crate::howto::RelocCode::W65Direct => Self::from_relnum(7),
            crate::howto::RelocCode::W65RelaxJsl => Self::from_relnum(10),
            crate::howto::RelocCode::W65RelaxJml => Self::from_relnum(11),
            crate::howto::RelocCode::W65RelaxBrl => Self::from_relnum(12),
            crate::howto::RelocCode::W65RelaxDirect => Self::from_relnum(13),
            crate::howto::RelocCode::W65RelaxAbs => Self::from_relnum(14),
            crate::howto::RelocCode::W65RelaxJmp => Self::from_relnum(15),
            _ => None,
        }
    }

    fn reloc_num(&self) -> u32 {
        match self {
            Elf32W65HowTo::None => 0,
            Elf32W65HowTo::Abs24 => 1,
            Elf32W65HowTo::Abs16 => 2,
            Elf32W65HowTo::Rel8 => 3,
            Elf32W65HowTo::Rel16 => 4,
            Elf32W65HowTo::Bank => 5,
            Elf32W65HowTo::Abs8 => 6,
            Elf32W65HowTo::Direct => 7,
            Elf32W65HowTo::RelaxJsl => 10,
            Elf32W65HowTo::RelaxJml => 11,
            Elf32W65HowTo::RelaxBrl => 12,
            Elf32W65HowTo::RelaxDirect => 13,
            Elf32W65HowTo::RelaxAbs => 14,
            Elf32W65HowTo::RelaxJmp => 15,
        }
    }

    fn name(&self) -> &'static str {
        match self {
            Elf32W65HowTo::None => "R_WC65C816_NONE",
            Elf32W65HowTo::Abs24 => "R_WC65C816_ABS24",
            Elf32W65HowTo::Abs16 => "R_WC65C816_ABS16",
            Elf32W65HowTo::Rel8 => "R_WC65C816_REL8",
            Elf32W65HowTo::Rel16 => "R_WC65C816_REL16",
            Elf32W65HowTo::Bank => "R_WC65C816_BANK",
            Elf32W65HowTo::Abs8 => "R_WC65C816_ABS8",
            Elf32W65HowTo::Direct => "R_WC65C816_DIRECT",
            Elf32W65HowTo::RelaxJsl => "R_WC65C816_RELAX_JSL",
            Elf32W65HowTo::RelaxJml => "R_WC65C816_RELAX_JML",
            Elf32W65HowTo::RelaxBrl => "R_WC65C816_RELAX_BRL",
            Elf32W65HowTo::RelaxDirect => "R_WC65C816_RELAX_DIRECT",
            Elf32W65HowTo::RelaxAbs => "R_WC65C816_RELAX_ABS",
            Elf32W65HowTo::RelaxJmp => "R_WC65C816_RELAX_JMP",
        }
    }

    fn reloc_size(&self) -> usize {
        match self {
            Elf32W65HowTo::None => 0,
            Elf32W65HowTo::Abs24 => 3,
            Elf32W65HowTo::Abs16 => 2,
            Elf32W65HowTo::Rel8 => 1,
            Elf32W65HowTo::Rel16 => 2,
            Elf32W65HowTo::Bank => 1,
            Elf32W65HowTo::Abs8 => 1,
            Elf32W65HowTo::Direct => 2,
            Elf32W65HowTo::RelaxJsl => 4,
            Elf32W65HowTo::RelaxJml => 4,
            Elf32W65HowTo::RelaxBrl => 3,
            Elf32W65HowTo::RelaxDirect => 4,
            Elf32W65HowTo::RelaxAbs => 4,
            Elf32W65HowTo::RelaxJmp => 3,
        }
    }

    fn pcrel(&self) -> bool {
        match self {
            Elf32W65HowTo::Rel8
            | Elf32W65HowTo::Rel16
            | Elf32W65HowTo::RelaxJmp
            | Elf32W65HowTo::RelaxBrl => true,
            Elf32W65HowTo::None => false,
            Elf32W65HowTo::Abs24 => false,
            Elf32W65HowTo::Abs16 => false,
            Elf32W65HowTo::Bank => false,
            Elf32W65HowTo::Abs8 => false,
            Elf32W65HowTo::Direct => false,
            Elf32W65HowTo::RelaxJsl => false,
            Elf32W65HowTo::RelaxJml => false,
            Elf32W65HowTo::RelaxDirect => false,
            Elf32W65HowTo::RelaxAbs => false,
        }
    }

    fn is_relax(&self) -> bool {
        match self {
            Elf32W65HowTo::None => false,
            Elf32W65HowTo::Abs24 => false,
            Elf32W65HowTo::Abs16 => false,
            Elf32W65HowTo::Rel8 => false,
            Elf32W65HowTo::Rel16 => false,
            Elf32W65HowTo::Bank => false,
            Elf32W65HowTo::Abs8 => false,
            Elf32W65HowTo::Direct => false,
            Elf32W65HowTo::RelaxJsl => true,
            Elf32W65HowTo::RelaxJml => true,
            Elf32W65HowTo::RelaxBrl => true,
            Elf32W65HowTo::RelaxDirect => true,
            Elf32W65HowTo::RelaxAbs => true,
            Elf32W65HowTo::RelaxJmp => true,
        }
    }

    fn relax_size(&self, _addr: u128, _at_addr: u128) -> Option<usize> {
        None
    }

    fn apply<'a>(
        &self,
        addr: u128,
        at_addr: u128,
        region: &'a mut [u8],
    ) -> Result<&'a mut [u8], crate::howto::HowToError> {
        match self {
            Elf32W65HowTo::None => Ok(region),
            Elf32W65HowTo::Abs24 => {
                if addr > 16777216 {
                    Err(HowToError::UnsignedOverflow)
                } else {
                    let bytes = addr.to_le_bytes();
                    region.copy_from_slice(&bytes[..3]);
                    Ok(region)
                }
            }
            Elf32W65HowTo::Abs16 => {
                let bytes = addr.to_le_bytes();
                region.copy_from_slice(&bytes[..2]);
                Ok(region)
            }
            Elf32W65HowTo::Rel8 => {
                let val = (addr as i128) - (at_addr as i128);
                if let Ok(x) = i8::try_from(val) {
                    let bytes = x.to_le_bytes();
                    region.copy_from_slice(&bytes);
                    Ok(region)
                } else {
                    Err(HowToError::SignedOverflow)
                }
            }
            Elf32W65HowTo::Rel16 => {
                let val = (addr as i128) - (at_addr as i128);
                if let Ok(x) = i16::try_from(val) {
                    let bytes = x.to_le_bytes();
                    region.copy_from_slice(&bytes);
                    Ok(region)
                } else {
                    Err(HowToError::SignedOverflow)
                }
            }
            Elf32W65HowTo::Bank => {
                let bytes = addr.to_le_bytes();
                region.copy_from_slice(&bytes[3..4]);
                Ok(region)
            }
            Elf32W65HowTo::Abs8 => {
                let bytes = addr.to_le_bytes();
                region.copy_from_slice(&bytes[..1]);
                Ok(region)
            }
            Elf32W65HowTo::Direct => {
                let bytes = (addr & !0xff).to_le_bytes();
                region.copy_from_slice(&bytes[..2]);
                Ok(region)
            }
            Elf32W65HowTo::RelaxJsl => unimplemented!(),
            Elf32W65HowTo::RelaxJml => unimplemented!(),
            Elf32W65HowTo::RelaxBrl => unimplemented!(),
            Elf32W65HowTo::RelaxDirect => unimplemented!(),
            Elf32W65HowTo::RelaxAbs => unimplemented!(),
            Elf32W65HowTo::RelaxJmp => unimplemented!(),
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

pub fn create_format() -> super::Elf32Format<Elf32W65HowTo> {
    super::Elf32Format::new(
        super::consts::EM_WC65C816,
        consts::ELFDATA2LSB,
        "elf32-w65",
        None,
        None,
    )
}
