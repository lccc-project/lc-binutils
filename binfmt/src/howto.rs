pub enum HowToError {
    SignedOverflow,
    UnsignedOverflow,
    InvalidReloc,
}

pub trait HowTo {
    // Helpers for generic binary formats (like Elf/)
    fn from_relnum<'a>(num: u32) -> Option<&'a Self>
    where
        Self: Sized + 'a;
    fn from_reloc_code<'a>(code: RelocCode) -> Option<&'a Self>
    where
        Self: Sized + 'a;
    fn reloc_num(&self) -> u32;
    fn name(&self) -> &'static str;
    fn reloc_size(&self) -> usize;
    fn pcrel(&self) -> bool;
    fn is_relax(&self) -> bool;
    fn relax_size(&self, addr: u128, at_addr: u128) -> Option<usize>;
    fn apply(&self, addr: u128, at_addr: u128, region: &mut [u8]) -> Result<bool, HowToError>;
}

pub enum RelocCode {
    None,
    Abs { addr_width: usize },
    Rel { addr_width: usize },
    AbsShifted { addr_width: usize, shift: usize },
    RelShifted { addr_width: usize, shift: usize },
    Got { addr_width: usize },
    RelGot { addr_wdith: usize },
    Plt { addr_width: usize },
    RelPlt { addr_width: usize },
    DynSymEntry { width: usize },
    W65Direct,
    W65RelaxJsl,
    W65RelaxJml,
    W65RelaxBrl,
    W65RelaxDirect,
    W65RelaxAbs,
    W65RelaxJmp,
}

pub struct Reloc {
    pub code: RelocCode,
    pub symbol: String,
    pub addend: Option<u64>,
    pub segno: Option<u32>,
    pub offset: u64,
}
