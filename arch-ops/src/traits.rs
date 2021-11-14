use std::io::{Read, Write};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Address {
    Abs(u128),
    Disp(i64),
    Symbol { name: String, disp: i64 },
}

pub trait InsnRead: Read {
    fn read_addr(&mut self, size: usize) -> std::io::Result<Address>;
}

impl<R: InsnRead> InsnRead for R {
    fn read_addr(&mut self, size: usize) -> std::io::Result<Address> {
        <R as InsnRead>::read_addr(self, size)
    }
}

pub trait InsnWrite: Write {
    fn write_addr(&mut self, size: usize) -> std::io::Result<()>;
}

impl<W: InsnWrite> InsnWrite for W {
    fn write_addr(&mut self, size: usize) -> std::io::Result<()> {
        <W as InsnWrite>::write_addr(self, size)
    }
}
