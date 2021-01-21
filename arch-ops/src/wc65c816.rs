use std::{
    fmt::Display,
    io::{Read, Write},
    iter::FusedIterator,
    ops::Range,
};

use either::Either;

use crate::traits::*;

#[derive(Copy, Clone)]
pub enum Wc65c816Register {
    Acc,
    DBR,
    DirectPage,
    PBR,
    PC,
    SP,
    IdxX,
    IdxY,
    Status,
}

impl Display for Wc65c816Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Wc65c816Register::Acc => f.write_str("%A"),
            Wc65c816Register::DBR => f.write_str("%B"),
            Wc65c816Register::DirectPage => f.write_str("%D"),
            Wc65c816Register::PBR => f.write_str("%K"),
            Wc65c816Register::PC => f.write_str("%PC"),
            Wc65c816Register::SP => f.write_str("%S"),
            Wc65c816Register::IdxX => f.write_str("%X"),
            Wc65c816Register::IdxY => f.write_str("%Y"),
            Wc65c816Register::Status => f.write_str("%S"),
        }
    }
}

impl Register for Wc65c816Register {
    type Value = u16;

    fn known_size(&self) -> Option<u32> {
        match self {
            Self::Acc | Self::IdxX | Self::IdxY => None,
            Self::DBR | Self::PBR | Self::Status => Some(8),
            _ => Some(16),
        }
    }

    fn size_range(&self) -> (u32, u32) {
        match self {
            Self::Acc | Self::IdxX | Self::IdxY => (8, 16),
            Self::DBR | Self::PBR | Self::Status => (8, 8),
            _ => (16, 16),
        }
    }
}

#[derive(Clone)]
pub enum Wc65c816Address {
    Absolute(u32),
    BankLocal(u16),
    PCRelLong(i16),
    PCRelShort(i8),
    Symbol(String),
}

impl Display for Wc65c816Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Wc65c816Address::Absolute(i) => f.write_fmt(format_args!("{:x}", i)),
            Wc65c816Address::BankLocal(i) => f.write_fmt(format_args!("{:x}", i)),
            Wc65c816Address::PCRelLong(i) => f.write_fmt(format_args!("$+{:x}", i)),
            Wc65c816Address::PCRelShort(i) => f.write_fmt(format_args!("$+{:x}", i)),
            Wc65c816Address::Symbol(s) => s.fmt(f),
        }
    }
}

impl Address for Wc65c816Address {
    type Value = u32;

    fn is_symbol(&self) -> bool {
        matches!(self, Self::Symbol(_))
    }

    fn to_value(&self) -> Option<Self::Value> {
        match self {
            Wc65c816Address::Absolute(v) => Some(*v),
            Wc65c816Address::BankLocal(v) => Some(*v as u32),
            Wc65c816Address::PCRelLong(_) => None,
            Wc65c816Address::PCRelShort(_) => None,
            Wc65c816Address::Symbol(_) => None,
        }
    }

    fn symbol_name(&self) -> Option<&str> {
        if let Self::Symbol(s) = self {
            Some(&**s)
        } else {
            None
        }
    }

    fn is_absolute(&self) -> bool {
        matches!(self, Self::Absolute(_) | Self::BankLocal(_))
    }

    fn to_absolute(&self, base: Self::Value) -> Option<Self::Value> {
        if self.is_absolute() {
            self.to_value()
        } else {
            let part = base & 0xffff;
            match self {
                Wc65c816Address::Absolute(_) => unreachable!(),
                Wc65c816Address::BankLocal(_) => unreachable!(),
                Wc65c816Address::PCRelLong(off) => Some(
                    base & 0xff0000
                        | (((part as i32).wrapping_add(*off as i32)) as u32 & 0xffffu32),
                ),
                Wc65c816Address::PCRelShort(off) => Some(
                    base & 0xff0000
                        | (((part as i32).wrapping_add(*off as i32)) as u32 & 0xffffu32),
                ),
                Wc65c816Address::Symbol(_) => None,
            }
        }
    }
}

#[derive(Clone)]
pub enum Wc65c816AddressPart {
    Bank(Wc65c816Address),
    DirectPage(Wc65c816Address),
}

impl Display for Wc65c816AddressPart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Wc65c816AddressPart::Bank(a) => f.write_fmt(format_args!("!{}", a)),
            Wc65c816AddressPart::DirectPage(a) => f.write_fmt(format_args!("%D+{}", a)),
        }
    }
}

impl AddressPart for Wc65c816AddressPart {
    type Address = Wc65c816Address;

    type PositionRange = Range<u32>;

    fn get_bits(&self) -> Self::PositionRange {
        match self {
            Wc65c816AddressPart::Bank(_) => 16..24,
            Wc65c816AddressPart::DirectPage(_) => 0..8,
        }
    }

    fn mask(&self) -> <Self::Address as Address>::Value {
        match self {
            Wc65c816AddressPart::Bank(_) => 0xff0000,
            Wc65c816AddressPart::DirectPage(_) => 0xff,
        }
    }

    fn get_address(&self) -> &Self::Address {
        match self {
            Self::Bank(a) | Self::DirectPage(a) => a,
        }
    }
}

#[derive(Clone)]
pub enum Wc65c816Operand {
    Indirect(Box<Wc65c816Operand>, Option<Wc65c816Register>),
    IndirectLong(Wc65c816Address),
    Address(Wc65c816Address, Option<Wc65c816Register>),
    AddressPart(Wc65c816AddressPart),
    Register(Wc65c816Register),
    Immediate(u16),
}

impl Display for Wc65c816Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Wc65c816Operand::Indirect(a, r) => {
                f.write_str("(")?;
                a.fmt(f)?;
                f.write_str(")")?;
                if let Some(r) = r {
                    f.write_str(",")?;
                    r.fmt(f)?;
                }
                Ok(())
            }
            Wc65c816Operand::IndirectLong(a) => f.write_fmt(format_args!("[{}]", a)),
            Wc65c816Operand::Address(a, r) => {
                a.fmt(f)?;
                if let Some(r) = r {
                    f.write_str(",")?;
                    r.fmt(f)?;
                }
                Ok(())
            }
            Wc65c816Operand::AddressPart(p) => p.fmt(f),
            Wc65c816Operand::Register(r) => r.fmt(f),
            Wc65c816Operand::Immediate(i) => i.fmt(f),
        }
    }
}

impl Operand for Wc65c816Operand {
    type Arch = Wc65c816;

    fn as_address(&self) -> Option<&Wc65c816Address> {
        if let Self::Address(addr, _) = self {
            Some(addr)
        } else {
            None
        }
    }

    fn as_indirect_address(&self) -> Option<&Wc65c816Address> {
        todo!()
    }

    fn as_immediate(&self) -> Option<&u16> {
        todo!()
    }

    fn as_address_fragment(&self) -> Option<&Wc65c816AddressPart> {
        todo!()
    }

    fn as_register(&self) -> Option<&Wc65c816Register> {
        todo!()
    }

    fn is_implied(&self) -> bool {
        todo!()
    }
}

#[non_exhaustive]
pub enum Wc65c816Instruction {
    ADC(Wc65c816Operand),
}

impl Display for Wc65c816Instruction {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

pub struct Wc65c816InstructionIterator<'a> {
    immediate: <&'a [Wc65c816Operand] as IntoIterator>::IntoIter,
    last: Option<&'a Wc65c816Operand>,
}

impl<'a> Iterator for Wc65c816InstructionIterator<'a> {
    type Item = &'a Wc65c816Operand;

    fn next(&mut self) -> Option<Self::Item> {
        self.immediate.next().or_else(|| self.last.take())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (min, max) = self.immediate.size_hint();
        (
            min.saturating_add(if self.last.is_some() { 1 } else { 0 }),
            max.map(|v| v.checked_add(if self.last.is_some() { 1 } else { 0 }))
                .flatten(),
        )
    }
}

impl<'a> FusedIterator for Wc65c816InstructionIterator<'a> {}

impl<'a> ExactSizeIterator for Wc65c816InstructionIterator<'a> {}

impl<'a> InstructionLifetime<'a> for Wc65c816Instruction {
    type Arch = Wc65c816;

    type Operands = Wc65c816InstructionIterator<'a>;
}

impl Instruction for Wc65c816Instruction {
    fn name(&self) -> &str {
        todo!()
    }

    fn operands(&self) -> Wc65c816InstructionIterator {
        todo!()
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub enum Wc65c816RelocationType {
    None,
    Long,
    Short,
    Rel8,
    Rel16,
    Bank,
    DirectPage,
}

pub struct Wc65c816Relocation {
    rel_type: Wc65c816RelocationType,
    symbol: Either<Wc65c816Address, Wc65c816AddressPart>,
}

impl Relocation for Wc65c816Relocation {
    type Address = Wc65c816Address;

    type AddressPart = Wc65c816AddressPart;

    type RelocationType = Wc65c816RelocationType;

    fn get_type(&self) -> Self::RelocationType {
        self.rel_type
    }

    fn get_address(&self) -> Option<&Self::Address> {
        match &self.symbol {
            Either::Left(a) => Some(a),
            Either::Right(a) => Some(a.get_address()),
        }
    }

    fn get_part(&self) -> Option<&Self::AddressPart> {
        self.symbol.as_ref().right()
    }
}
pub struct Wc65c816InstructionWriter<'a> {
    _relocs: &'a mut dyn RelocationWriter<Relocation = Wc65c816Relocation>,
    writer: &'a mut (dyn Write + 'a),
    arch: &'a Wc65c816,
}

impl<'a> InstructionWriter for Wc65c816InstructionWriter<'a> {
    type Arch = Wc65c816;

    type Instruction = Wc65c816Instruction;

    type Relocation = Wc65c816Relocation;

    type Error = std::io::Error;

    fn get_architecture(&self) -> &Self::Arch {
        &self.arch
    }

    fn write_instruction(&mut self, _ins: Self::Instruction) -> Result<(), Self::Error> {
        todo!()
    }

    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
        self.writer.write_all(bytes)
    }
}

pub struct Wc65c816InstructionReader<'a> {
    read: &'a mut (dyn Read + 'a),
    arch: &'a Wc65c816,
}

impl<'a> InstructionReader for Wc65c816InstructionReader<'a> {
    type Arch = Wc65c816;

    type Instruction = Wc65c816Instruction;

    type Error = std::io::Error;

    fn get_architecture(&self) -> &Self::Arch {
        &self.arch
    }

    fn read_instruction(&mut self) -> Result<Self::Instruction, Self::Error> {
        todo!()
    }

    fn read_bytes(&mut self, bytes: &mut [u8]) -> Result<(), Self::Error> {
        self.read.read_exact(bytes)
    }
}

pub struct Wc65c816 {}

impl<'a> Instructions<'a> for Wc65c816 {
    type InstructionWriter = Wc65c816InstructionWriter<'a>;

    type InstructionReader = Wc65c816InstructionReader<'a>;
}

impl Architecture for Wc65c816 {
    type Operand = Wc65c816Operand;
    type Address = Wc65c816Address;
    type Immediate = u16;
    type AddressPart = Wc65c816AddressPart;
    type Register = Wc65c816Register;

    type Instruction = Wc65c816Instruction;

    type Relocation = Wc65c816Relocation;

    fn registers(&self) -> &[Self::Register] {
        todo!()
    }

    fn new_writer<'a>(
        &'a self,
        _relocs: &'a mut (dyn crate::traits::RelocationWriter<Relocation = Self::Relocation> + 'a),
        _w: &'a mut (dyn Write + 'a),
    ) -> Wc65c816InstructionWriter<'a> {
        todo!()
    }

    fn new_reader<'a>(&'a self, _r: &'a mut (dyn Read + 'a)) -> Wc65c816InstructionReader {
        todo!()
    }
}
