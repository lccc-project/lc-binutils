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

pub use arch_ops::traits::{Reloc, RelocCode};
