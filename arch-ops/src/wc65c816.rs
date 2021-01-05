use std::{fmt::Display, ops::Range};

use crate::traits::{Address, AddressPart, Register};

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
                        | (((part as i32).wrapping_add(*off as i32)) as u32 & 0xffff as u32),
                ),
                Wc65c816Address::PCRelShort(off) => Some(
                    base & 0xff0000
                        | (((part as i32).wrapping_add(*off as i32)) as u32 & 0xffff as u32),
                ),
                Wc65c816Address::Symbol(_) => None,
            }
        }
    }
}

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
