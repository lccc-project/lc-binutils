use std::{
    fmt::{Debug},
    io::{Read, Write},
};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Address{
    Symbol(String),
    Abs(u128),
    Offset(String,i64)
}


pub trait InsnWrite : Write{
    fn write_addr(&mut self, addr: Address, size: usize, pcrel: bool) -> std::io::Result<()>;
}

impl<I: InsnWrite> InsnWrite for &mut I{
    fn write_addr(&mut self, addr: Address, size: usize, pcrel: bool) -> std::io::Result<()>{
        <I as InsnWrite>::write_addr(self,addr,size,pcrel)
    }
}

pub trait InsnRead : Read{
    fn read_addr(&mut self, size: usize, pcrel: bool) -> std::io::Result<Address>;
}

impl<I: InsnRead> InsnRead for &mut I{
    fn read_addr(&mut self,size: usize, pcrel: bool) -> std::io::Result<Address>{
        <I as InsnRead>::read_addr(self,size,pcrel)
    }
}