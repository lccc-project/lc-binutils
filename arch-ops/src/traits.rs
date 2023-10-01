use std::io::{Read, Write};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Address {
    Abs(u128),
    Disp(i64),
    Symbol { name: String, disp: i64 },
    PltSym { name: String },
}

impl core::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Abs(n) => f.write_fmt(format_args!("{:#x}", n)),
            Self::Disp(n) => f.write_fmt(format_args!("{:+}", n)),
            Self::Symbol { name, disp: 0 } => f.write_str(name),
            Self::Symbol { name, disp } => f.write_fmt(format_args!("{}{:+}", name, disp)),
            Self::PltSym { name } => f.write_fmt(format_args!("{}@plt", name)),
        }
    }
}

pub trait InsnRead: Read {
    fn read_addr(&mut self, size: usize, rel: bool) -> std::io::Result<Address>;
    /// Reads a relocation
    /// If `offset` is `None`, reads from the head of the stream. Otherwise, reads from the given offset from the head.
    ///
    /// If `offset` is not-None, the function may error if it is not between -size and size in bytes, rounded away from zero
    fn read_reloc(
        &mut self,
        size: usize,
        rel: bool,
        offset: Option<isize>,
    ) -> std::io::Result<Option<Address>>;
}

impl<R: InsnRead + ?Sized> InsnRead for &mut R {
    fn read_addr(&mut self, size: usize, rel: bool) -> std::io::Result<Address> {
        <R as InsnRead>::read_addr(self, size, rel)
    }

    fn read_reloc(
        &mut self,
        size: usize,
        rel: bool,
        offset: Option<isize>,
    ) -> std::io::Result<Option<Address>> {
        <R as InsnRead>::read_reloc(self, size, rel, offset)
    }
}

impl<R: InsnRead + ?Sized> InsnRead for Box<R> {
    fn read_addr(&mut self, size: usize, rel: bool) -> std::io::Result<Address> {
        <R as InsnRead>::read_addr(self, size, rel)
    }

    fn read_reloc(
        &mut self,
        size: usize,
        rel: bool,
        offset: Option<isize>,
    ) -> std::io::Result<Option<Address>> {
        <R as InsnRead>::read_reloc(self, size, rel, offset)
    }
}

/// Writes `count` zeroes to `out` using the default block, in blocks of 1024.
pub fn default_write_zeroes<W: Write>(mut out: W, mut count: usize) -> std::io::Result<()> {
    let val: [u8; 1024] = [0; 1024];
    while count >= 1024 {
        out.write_all(&val)?;
        count -= 1024;
    }
    out.write_all(&val[..count])
}

pub trait InsnWrite: Write {
    /// Writes an address of `size` (in bits) to the writer. This will typically generate a relocation
    fn write_addr(&mut self, size: usize, addr: Address, rel: bool) -> std::io::Result<()>;
    fn write_reloc(&mut self, reloc: Reloc) -> std::io::Result<()>;
    fn offset(&self) -> usize;
    /// Writes `count` zero bytes. This is provided as some writer types (such as sections) may implement this more efficiently then the default
    fn write_zeroes(&mut self, count: usize) -> std::io::Result<()> {
        default_write_zeroes(self, count)
    }
}

impl<W: InsnWrite + ?Sized> InsnWrite for &mut W {
    fn write_addr(&mut self, size: usize, addr: Address, rel: bool) -> std::io::Result<()> {
        <W as InsnWrite>::write_addr(self, size, addr, rel)
    }
    fn offset(&self) -> usize {
        <W as InsnWrite>::offset(self)
    }

    fn write_reloc(&mut self, reloc: Reloc) -> std::io::Result<()> {
        <W as InsnWrite>::write_reloc(self, reloc)
    }
    fn write_zeroes(&mut self, count: usize) -> std::io::Result<()> {
        <W as InsnWrite>::write_zeroes(self, count)
    }
}

impl<W: InsnWrite + ?Sized> InsnWrite for Box<W> {
    fn write_addr(&mut self, size: usize, addr: Address, rel: bool) -> std::io::Result<()> {
        <W as InsnWrite>::write_addr(self, size, addr, rel)
    }
    fn offset(&self) -> usize {
        <W as InsnWrite>::offset(self)
    }

    fn write_reloc(&mut self, reloc: Reloc) -> std::io::Result<()> {
        <W as InsnWrite>::write_reloc(self, reloc)
    }
    fn write_zeroes(&mut self, count: usize) -> std::io::Result<()> {
        <W as InsnWrite>::write_zeroes(self, count)
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
    HbRelaxedRel,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Reloc {
    pub code: RelocCode,
    pub symbol: String,
    pub addend: Option<i64>,
    pub offset: u64,
}
