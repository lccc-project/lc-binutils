use {
    super::Elf64Format,
    crate::howto::{HowTo, HowToError},
    arch_ops::traits::RelocCode,
};

macro_rules! gen {
    (count: $count:expr; $($variant:ident),* $(,)?) => {
        #[derive(Copy, Clone, Debug, PartialEq, Eq)]
        #[repr(u8)]
        #[non_exhaustive]
        pub enum Elf64HoleyBytesHowTo
            { $($variant),* }

        static TABLE: [Elf64HoleyBytesHowTo; $count] =
            [$(Elf64HoleyBytesHowTo::$variant),*];
    };
}

gen! {
    count: 15;
    None,
    Abs64,
    Rel16,
    Rel32,
    RelaxRel,
    GotPcrel,
    PltPcrel,
    GotPcrelRelax,
    PltRelax,
    Dynent,
    JumpSlot,
    GlobData,
    TpOff,
    DynTpOff,
    GotDynTpOff,
}

impl HowTo for Elf64HoleyBytesHowTo {
    #[inline]
    fn from_relnum<'a>(num: u32) -> Option<&'a Self>
    where
        Self: Sized + 'a,
    {
        TABLE.get(num as usize)
    }

    fn from_reloc_code<'a>(code: RelocCode) -> Option<&'a Self>
    where
        Self: Sized + 'a,
    {
        Some(match code {
            RelocCode::None => &TABLE[Self::None as usize],
            RelocCode::Abs { addr_width: 64 } => &TABLE[Self::Abs64 as usize],
            RelocCode::Rel { addr_width: 16 } => &TABLE[Self::Rel16 as usize],
            RelocCode::Rel { addr_width: 32 } => &TABLE[Self::Rel32 as usize],
            RelocCode::RelGot { addr_wdith: 32 } => &TABLE[Self::GotPcrel as usize],
            RelocCode::RelPlt { addr_width: 32 } => &TABLE[Self::PltPcrel as usize],
            RelocCode::DynSymEntry { width: 8 } => &TABLE[Self::Dynent as usize],
            _ => return None,
        })
    }

    fn reloc_num(&self) -> u32 {
        *self as u32
    }

    fn name(&self) -> &'static str {
        use Elf64HoleyBytesHowTo::*;
        match self {
            None => "R_HOLEYBYTES_NONE",
            Abs64 => "R_HOLEYBYTES_ABS64",
            Rel16 => "R_HOLEYBYTES_REL16",
            Rel32 => "R_HOLEYBYTES_REL32",
            RelaxRel => "R_HOLEYBYTES_RELAXREL",
            GotPcrel => "R_HOLEYBYTES_GOTPCREL",
            PltPcrel => "R_HOLEYBYTES_PLT",
            GotPcrelRelax => "R_HOLEYBYTES_GOTPCRELRELAX",
            PltRelax => "R_HOLEYBYTES_PLTRELAX",
            Dynent => "R_HOLEYBYTES_DYNENT",
            JumpSlot => "R_HOLEYBYTES_JUMPSLOT",
            GlobData => "R_HOLEYBYTES_GLOBDATA",
            TpOff => "R_HOLEYBYTES_TPOFF",
            DynTpOff => "R_HOLEYBYTES_DYNTPOFF",
            GotDynTpOff => "R_HOLEYBYTES_GOTDYNTPOFF",
        }
    }

    fn reloc_size(&self) -> usize {
        use Elf64HoleyBytesHowTo::*;
        match self {
            None => 0,
            Rel16 => 2,
            Rel32 | GotPcrel | PltPcrel | GotDynTpOff => 4,
            RelaxRel | GotPcrelRelax | PltRelax => 5,
            Abs64 | Dynent | JumpSlot | GlobData | TpOff | DynTpOff => 8,
        }
    }

    fn pcrel(&self) -> bool {
        use Elf64HoleyBytesHowTo::*;
        match self {
            None | Abs64 | Dynent | JumpSlot | GlobData | TpOff | DynTpOff => false,
            Rel16 | Rel32 | RelaxRel | GotPcrel | PltPcrel | GotPcrelRelax | PltRelax | GotDynTpOff => {
                true
            }
        }
    }

    fn is_relax(&self) -> bool {
        use Elf64HoleyBytesHowTo::*;
        matches!(self, RelaxRel | GotPcrelRelax | PltRelax)
    }

    fn relax_size(&self, _addr: u128, _at_addr: u128) -> Option<usize> {
        unimplemented!("Will be removed soon™?")
    }

    fn apply(&self, _addr: u128, _at_addr: u128, _region: &mut [u8]) -> Result<bool, HowToError> {
        todo!()
    }
}

pub fn create_format() -> Elf64Format<Elf64HoleyBytesHowTo> {
    Elf64Format::new(
        super::consts::EM_HOLEYBYTES,
        super::consts::ELFDATA2LSB,
        "elf64-holeybytes",
        None,
        None,
    )
}
