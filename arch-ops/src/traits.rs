use std::io::{Read, Write};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Address {
    Abs(u128),
    Disp(i64),
    Symbol { name: String, disp: i64 },
    PltSym { name: String },
}

pub trait InsnRead: Read {
    fn read_addr(&mut self, size: usize, rel: bool) -> std::io::Result<Address>;
}

impl<R: InsnRead> InsnRead for &mut R {
    fn read_addr(&mut self, size: usize, rel: bool) -> std::io::Result<Address> {
        <R as InsnRead>::read_addr(self, size, rel)
    }
}

pub trait InsnWrite: Write {
    fn write_addr(&mut self, size: usize, addr: Address, rel: bool) -> std::io::Result<()>;
    fn write_reloc(&mut self, reloc: Reloc) -> std::io::Result<()>;
    fn offset(&self) -> usize;
}

impl<W: InsnWrite> InsnWrite for &mut W {
    fn write_addr(&mut self, size: usize, addr: Address, rel: bool) -> std::io::Result<()> {
        <W as InsnWrite>::write_addr(self, size, addr, rel)
    }
    fn offset(&self) -> usize {
        <W as InsnWrite>::offset(self)
    }

    fn write_reloc(&mut self, reloc: Reloc) -> std::io::Result<()> {
        <W as InsnWrite>::write_reloc(self, reloc)
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum RelocCode {
    None,
    Abs { addr_width: usize },
    BaseRel { addr_width: usize },
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
    CleverShort,
    CleverShortPcrel,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Reloc {
    pub code: RelocCode,
    pub symbol: String,
    pub addend: Option<i64>,
    pub offset: u64,
}
