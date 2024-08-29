#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
pub enum HowToError {
    SignedOverflow,
    UnsignedOverflow,
    InvalidReloc,
}

#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
pub enum RelocOutput {
    FixedExec,
    RelocExec,
    RelocExecRelro,
    RelocDyn,
    RelocDynRelro,
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
    fn apply<'a>(
        &self,
        addr: u128,
        at_addr: u128,
        region: &'a mut [u8],
    ) -> Result<&'a mut [u8], HowToError>;
    fn valid_in(&self, output_ty: RelocOutput, sym_vis: &Symbol) -> bool;
}

pub use arch_ops::traits::{Reloc, RelocCode};

use crate::sym::Symbol;
