use std::{
    fmt::Display,
    io::{ErrorKind, Read, Write},
    ops::Range,
    slice::{self, Iter},
    u16,
};

use either::Either;

use crate::traits::*;

mod registers;

pub use register::Register as Wc65c816Register;

#[derive(Clone)]
pub enum Wc65c816Address {
    Absolute(u32),
    BankLocal(u16),
    PCRelLong(i16),
    PCRelShort(i8),
    Symbol(String),
    LongSymbol(String),
    Stack(u8),
}

impl Display for Wc65c816Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Wc65c816Address::Absolute(i) => f.write_fmt(format_args!("{:x}", i)),
            Wc65c816Address::BankLocal(i) => f.write_fmt(format_args!("{:x}", i)),
            Wc65c816Address::PCRelLong(i) => f.write_fmt(format_args!("$+{:x}", i)),
            Wc65c816Address::PCRelShort(i) => f.write_fmt(format_args!("$+{:x}", i)),
            Wc65c816Address::Symbol(s) => s.fmt(f),
            Wc65c816Address::LongSymbol(s) => s.fmt(f),
            Wc65c816Address::Stack(i) => f.write_fmt(format_args!("{},S", i)),
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
            Wc65c816Address::Symbol(_) | Wc65c816Address::LongSymbol(_) => None,
            Wc65c816Address::Stack(_) => None,
        }
    }

    fn symbol_name(&self) -> Option<&str> {
        if let Self::Symbol(s) | Wc65c816Address::LongSymbol(s) = self {
            Some(s)
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
                Wc65c816Address::Symbol(_) | Wc65c816Address::LongSymbol(_) => None,
                Wc65c816Address::Stack(_) => None,
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
    ImmediateAddress(Wc65c816Address),
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
            Wc65c816Operand::Immediate(i) => f.write_fmt(format_args!("#{}", i)),
            Wc65c816Operand::ImmediateAddress(i) => f.write_fmt(format_args!("@{}", i)),
        }
    }
}

impl Operand for Wc65c816Operand {
    type Arch = Wc65c816;

    fn as_address(&self) -> Option<&Wc65c816Address> {
        if let Self::Address(addr, _) = self {
            Some(addr)
        } else if let Self::ImmediateAddress(addr) = self {
            Some(addr)
        } else {
            None
        }
    }

    fn as_indirect_address(&self) -> Option<&Wc65c816Address> {
        if let Self::Indirect(o, _) = self {
            if let Self::Address(a, _) = &**o {
                return Some(a);
            }
        }
        None
    }

    fn as_immediate(&self) -> Option<&u16> {
        if let Self::Immediate(i) = self {
            Some(i)
        } else {
            None
        }
    }

    fn as_address_fragment(&self) -> Option<&Wc65c816AddressPart> {
        if let Self::AddressPart(a) = self {
            Some(a)
        } else {
            None
        }
    }

    fn as_register(&self) -> Option<&Wc65c816Register> {
        if let Self::Register(r) = self {
            Some(r)
        } else {
            None
        }
    }

    fn is_implied(&self) -> bool {
        matches!(self, Self::Register(_))
    }
}

#[non_exhaustive]
pub enum Wc65c816Instruction {
    Adc(Wc65c816Operand),
    Sbc(Wc65c816Operand),
    Cmp(Wc65c816Operand),
    Cpx(Wc65c816Operand),
    Cpy(Wc65c816Operand),
    Dec(Option<Wc65c816Operand>),
    Dex(Option<Wc65c816Operand>),
    Dey(Option<Wc65c816Operand>),
    Inc(Option<Wc65c816Operand>),
    Inx(Option<Wc65c816Operand>),
    Iny(Option<Wc65c816Operand>),
    And(Wc65c816Operand),
    Eor(Wc65c816Operand),
    Ora(Wc65c816Operand),
    Bit(Wc65c816Operand),
    Trb(Wc65c816Operand),
    Tsb(Wc65c816Operand),
    Asl(Wc65c816Operand),
    Lsr(Wc65c816Operand),
    Ror(Wc65c816Operand),
    Rol(Wc65c816Operand),
    Bcc(Wc65c816Operand),
    Bcs(Wc65c816Operand),
    Beq(Wc65c816Operand),
    Bne(Wc65c816Operand),
    Bmi(Wc65c816Operand),
    Bpl(Wc65c816Operand),
    Bra(Wc65c816Operand),
    Bvc(Wc65c816Operand),
    Bvs(Wc65c816Operand),
    Brl(Wc65c816Operand),
    Jmp(Wc65c816Operand),
    Jsr(Wc65c816Operand),
    Jsl(Wc65c816Operand),
    Rts,
    Rtl,
    Brk(Option<Wc65c816Operand>),
    Cop(Option<Wc65c816Operand>),
    Rti,
    Clc,
    Cld,
    Cli,
    Clv,
    Sec,
    Sed,
    Sei,
    Rep(Wc65c816Operand),
    Sep(Wc65c816Operand),
    Lda(Wc65c816Operand),
    Ldx(Wc65c816Operand),
    Ldy(Wc65c816Operand),
    Sta(Wc65c816Operand),
    Stx(Wc65c816Operand),
    Sty(Wc65c816Operand),
    Stz(Wc65c816Operand),
    Mvn([Wc65c816Operand; 2]),
    Mvp([Wc65c816Operand; 2]),
    Nop,
    Wdm(Option<Wc65c816Operand>),
    Pea(Wc65c816Operand),
    Pha,
    Phx,
    Phy,
    Pla,
    Plx,
    Ply,
    Phb,
    Phd,
    Phk,
    Php,
    Plb,
    Pld,
    Plp,
    Stp,
    Wai,
    Tcd,
    Tcs,
    Tdc,
    Tsc,
    Xba,
    Xce,
}

pub const A: Wc65c816Operand = Wc65c816Operand::Register(Wc65c816Register::Acc);
pub const X: Wc65c816Operand = Wc65c816Operand::Register(Wc65c816Register::IdxX);
pub const Y: Wc65c816Operand = Wc65c816Operand::Register(Wc65c816Register::IdxY);

impl Display for Wc65c816Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Wc65c816Instruction::Adc(o) => f.write_fmt(format_args!("adc {}", o)),
            Wc65c816Instruction::Sbc(o) => f.write_fmt(format_args!("sbc {}", o)),
            Wc65c816Instruction::Cmp(o) => f.write_fmt(format_args!("cmp {}", o)),
            Wc65c816Instruction::Cpx(o) => f.write_fmt(format_args!("cpx {}", o)),
            Wc65c816Instruction::Cpy(o) => f.write_fmt(format_args!("cpy {}", o)),
            Wc65c816Instruction::Dec(Some(o)) => f.write_fmt(format_args!("dec {}", o)),
            Wc65c816Instruction::Dex(Some(o)) => f.write_fmt(format_args!("dex {}", o)),
            Wc65c816Instruction::Dey(Some(o)) => f.write_fmt(format_args!("dey {}", o)),
            Wc65c816Instruction::Inc(Some(o)) => f.write_fmt(format_args!("inc {}", o)),
            Wc65c816Instruction::Inx(Some(o)) => f.write_fmt(format_args!("inx {}", o)),
            Wc65c816Instruction::Iny(Some(o)) => f.write_fmt(format_args!("iny {}", o)),
            Wc65c816Instruction::Dec(None) => f.write_fmt(format_args!("dec")),
            Wc65c816Instruction::Dex(None) => f.write_fmt(format_args!("dex")),
            Wc65c816Instruction::Dey(None) => f.write_fmt(format_args!("dey")),
            Wc65c816Instruction::Inc(None) => f.write_fmt(format_args!("inc")),
            Wc65c816Instruction::Inx(None) => f.write_fmt(format_args!("inx")),
            Wc65c816Instruction::Iny(None) => f.write_fmt(format_args!("iny")),
            Wc65c816Instruction::And(o) => f.write_fmt(format_args!("and {}", o)),
            Wc65c816Instruction::Eor(o) => f.write_fmt(format_args!("eor {}", o)),
            Wc65c816Instruction::Ora(o) => f.write_fmt(format_args!("ora {}", o)),
            Wc65c816Instruction::Bit(o) => f.write_fmt(format_args!("bit {}", o)),
            Wc65c816Instruction::Trb(o) => f.write_fmt(format_args!("trb {}", o)),
            Wc65c816Instruction::Tsb(o) => f.write_fmt(format_args!("tsb {}", o)),
            Wc65c816Instruction::Asl(o) => f.write_fmt(format_args!("asl {}", o)),
            Wc65c816Instruction::Lsr(o) => f.write_fmt(format_args!("lsr {}", o)),
            Wc65c816Instruction::Ror(o) => f.write_fmt(format_args!("ror {}", o)),
            Wc65c816Instruction::Rol(o) => f.write_fmt(format_args!("rol {}", o)),
            Wc65c816Instruction::Bcc(o) => f.write_fmt(format_args!("bcc {}", o)),
            Wc65c816Instruction::Bcs(o) => f.write_fmt(format_args!("bcs {}", o)),
            Wc65c816Instruction::Beq(o) => f.write_fmt(format_args!("beq {}", o)),
            Wc65c816Instruction::Bne(o) => f.write_fmt(format_args!("bne {}", o)),
            Wc65c816Instruction::Bmi(o) => f.write_fmt(format_args!("bmi {}", o)),
            Wc65c816Instruction::Bpl(o) => f.write_fmt(format_args!("bpl {}", o)),
            Wc65c816Instruction::Bra(o) => f.write_fmt(format_args!("bra {}", o)),
            Wc65c816Instruction::Bvc(o) => f.write_fmt(format_args!("bvc {}", o)),
            Wc65c816Instruction::Bvs(o) => f.write_fmt(format_args!("bvs {}", o)),
            Wc65c816Instruction::Brl(o) => f.write_fmt(format_args!("brl {}", o)),
            Wc65c816Instruction::Jmp(o) => f.write_fmt(format_args!("jmp {}", o)),
            Wc65c816Instruction::Jsr(o) => f.write_fmt(format_args!("jsr {}", o)),
            Wc65c816Instruction::Jsl(o) => f.write_fmt(format_args!("jsl {}", o)),
            Wc65c816Instruction::Rts => f.write_fmt(format_args!("rts")),
            Wc65c816Instruction::Rtl => f.write_fmt(format_args!("rtl")),
            Wc65c816Instruction::Brk(Some(o)) => f.write_fmt(format_args!("brk {}", o)),
            Wc65c816Instruction::Cop(Some(o)) => f.write_fmt(format_args!("cop {}", o)),
            Wc65c816Instruction::Brk(None) => f.write_fmt(format_args!("brk")),
            Wc65c816Instruction::Cop(None) => f.write_fmt(format_args!("cop")),
            Wc65c816Instruction::Rti => f.write_fmt(format_args!("rti")),
            Wc65c816Instruction::Clc => f.write_fmt(format_args!("clc")),
            Wc65c816Instruction::Cld => f.write_fmt(format_args!("cld")),
            Wc65c816Instruction::Cli => f.write_fmt(format_args!("cli")),
            Wc65c816Instruction::Clv => f.write_fmt(format_args!("clv")),
            Wc65c816Instruction::Sec => f.write_fmt(format_args!("sec")),
            Wc65c816Instruction::Sed => f.write_fmt(format_args!("sed")),
            Wc65c816Instruction::Sei => f.write_fmt(format_args!("sei")),
            Wc65c816Instruction::Rep(o) => f.write_fmt(format_args!("rep {}", o)),
            Wc65c816Instruction::Sep(o) => f.write_fmt(format_args!("sep {}", o)),
            Wc65c816Instruction::Lda(o) => f.write_fmt(format_args!("lda {}", o)),
            Wc65c816Instruction::Ldx(o) => f.write_fmt(format_args!("ldx {}", o)),
            Wc65c816Instruction::Ldy(o) => f.write_fmt(format_args!("ldy {}", o)),
            Wc65c816Instruction::Sta(o) => f.write_fmt(format_args!("sta {}", o)),
            Wc65c816Instruction::Stx(o) => f.write_fmt(format_args!("stx {}", o)),
            Wc65c816Instruction::Sty(o) => f.write_fmt(format_args!("sty {}", o)),
            Wc65c816Instruction::Stz(o) => f.write_fmt(format_args!("stz {}", o)),
            Wc65c816Instruction::Mvn([a, b]) => f.write_fmt(format_args!("mvn {} {}", a, b)),
            Wc65c816Instruction::Mvp([a, b]) => f.write_fmt(format_args!("adc {} {}", a, b)),
            Wc65c816Instruction::Nop => f.write_fmt(format_args!("nop")),
            Wc65c816Instruction::Wdm(Some(o)) => f.write_fmt(format_args!("wdm {}", o)),
            Wc65c816Instruction::Wdm(None) => f.write_fmt(format_args!("wdm")),
            Wc65c816Instruction::Pea(o) => f.write_fmt(format_args!("pea {}", o)),
            Wc65c816Instruction::Pha => f.write_fmt(format_args!("pha")),
            Wc65c816Instruction::Phx => f.write_fmt(format_args!("phx")),
            Wc65c816Instruction::Phy => f.write_fmt(format_args!("phy")),
            Wc65c816Instruction::Pla => f.write_fmt(format_args!("pla")),
            Wc65c816Instruction::Plx => f.write_fmt(format_args!("plx")),
            Wc65c816Instruction::Ply => f.write_fmt(format_args!("ply")),
            Wc65c816Instruction::Phb => f.write_fmt(format_args!("phb")),
            Wc65c816Instruction::Phd => f.write_fmt(format_args!("phd")),
            Wc65c816Instruction::Phk => f.write_fmt(format_args!("phk")),
            Wc65c816Instruction::Php => f.write_fmt(format_args!("php")),
            Wc65c816Instruction::Plb => f.write_fmt(format_args!("plb")),
            Wc65c816Instruction::Pld => f.write_fmt(format_args!("pld")),
            Wc65c816Instruction::Plp => f.write_fmt(format_args!("plp")),
            Wc65c816Instruction::Stp => f.write_fmt(format_args!("stp")),
            Wc65c816Instruction::Wai => f.write_fmt(format_args!("wai")),
            Wc65c816Instruction::Tcd => f.write_fmt(format_args!("tcd")),
            Wc65c816Instruction::Tcs => f.write_fmt(format_args!("tcs")),
            Wc65c816Instruction::Tdc => f.write_fmt(format_args!("tdc")),
            Wc65c816Instruction::Tsc => f.write_fmt(format_args!("tsc")),
            Wc65c816Instruction::Xba => f.write_fmt(format_args!("xba")),
            Wc65c816Instruction::Xce => f.write_fmt(format_args!("xce")),
        }
    }
}

impl<'a> InstructionLifetime<'a> for Wc65c816Instruction {
    type Arch = Wc65c816;

    type Operands = Iter<'a, Wc65c816Operand>;
}

impl Instruction for Wc65c816Instruction {
    fn name(&self) -> &str {
        match self {
            Wc65c816Instruction::Adc(_) => "adc",
            Wc65c816Instruction::Sbc(_) => "sbc",
            Wc65c816Instruction::Cmp(_) => "cmp",
            Wc65c816Instruction::Cpx(_) => "cpx",
            Wc65c816Instruction::Cpy(_) => "cpy",
            Wc65c816Instruction::Dec(_) => "dec",
            Wc65c816Instruction::Dex(_) => "dex",
            Wc65c816Instruction::Dey(_) => "dey",
            Wc65c816Instruction::Inc(_) => "inc",
            Wc65c816Instruction::Inx(_) => "inx",
            Wc65c816Instruction::Iny(_) => "iny",
            Wc65c816Instruction::And(_) => "and",
            Wc65c816Instruction::Eor(_) => "eor",
            Wc65c816Instruction::Ora(_) => "ora",
            Wc65c816Instruction::Bit(_) => "bit",
            Wc65c816Instruction::Trb(_) => "trb",
            Wc65c816Instruction::Tsb(_) => "tsb",
            Wc65c816Instruction::Asl(_) => "asl",
            Wc65c816Instruction::Lsr(_) => "lsr",
            Wc65c816Instruction::Ror(_) => "ror",
            Wc65c816Instruction::Rol(_) => "rol",
            Wc65c816Instruction::Bcc(_) => "bcc",
            Wc65c816Instruction::Bcs(_) => "bcs",
            Wc65c816Instruction::Beq(_) => "beq",
            Wc65c816Instruction::Bne(_) => "bne",
            Wc65c816Instruction::Bmi(_) => "bmi",
            Wc65c816Instruction::Bpl(_) => "bpl",
            Wc65c816Instruction::Bra(_) => "bra",
            Wc65c816Instruction::Bvc(_) => "bvc",
            Wc65c816Instruction::Bvs(_) => "bvs",
            Wc65c816Instruction::Brl(_) => "brl",
            Wc65c816Instruction::Jmp(_) => "jmp",
            Wc65c816Instruction::Jsr(_) => "jsr",
            Wc65c816Instruction::Jsl(_) => "jsl",
            Wc65c816Instruction::Rts => "rts",
            Wc65c816Instruction::Rtl => "rtl",
            Wc65c816Instruction::Brk(_) => "brk",
            Wc65c816Instruction::Cop(_) => "cop",
            Wc65c816Instruction::Rti => "rti",
            Wc65c816Instruction::Clc => "clc",
            Wc65c816Instruction::Cld => "cld",
            Wc65c816Instruction::Cli => "cli",
            Wc65c816Instruction::Clv => "clv",
            Wc65c816Instruction::Sec => "sec",
            Wc65c816Instruction::Sed => "sed",
            Wc65c816Instruction::Sei => "sei",
            Wc65c816Instruction::Rep(_) => "rep",
            Wc65c816Instruction::Sep(_) => "sep",
            Wc65c816Instruction::Lda(_) => "lda",
            Wc65c816Instruction::Ldx(_) => "ldx",
            Wc65c816Instruction::Ldy(_) => "ldy",
            Wc65c816Instruction::Sta(_) => "sta",
            Wc65c816Instruction::Stx(_) => "stx",
            Wc65c816Instruction::Sty(_) => "sty",
            Wc65c816Instruction::Stz(_) => "stz",
            Wc65c816Instruction::Mvn(_) => "mvn",
            Wc65c816Instruction::Mvp(_) => "mvp",
            Wc65c816Instruction::Nop => "nop",
            Wc65c816Instruction::Wdm(_) => "wdm",
            Wc65c816Instruction::Pea(_) => "pea",
            Wc65c816Instruction::Pha => "pha",
            Wc65c816Instruction::Phx => "phx",
            Wc65c816Instruction::Phy => "phy",
            Wc65c816Instruction::Pla => "pla",
            Wc65c816Instruction::Plx => "plx",
            Wc65c816Instruction::Ply => "ply",
            Wc65c816Instruction::Phb => "phb",
            Wc65c816Instruction::Phd => "phd",
            Wc65c816Instruction::Phk => "phk",
            Wc65c816Instruction::Php => "php",
            Wc65c816Instruction::Plb => "plb",
            Wc65c816Instruction::Pld => "pld",
            Wc65c816Instruction::Plp => "plp",
            Wc65c816Instruction::Stp => "stp",
            Wc65c816Instruction::Wai => "wai",
            Wc65c816Instruction::Tcd => "tcd",
            Wc65c816Instruction::Tcs => "tcs",
            Wc65c816Instruction::Tdc => "tcd",
            Wc65c816Instruction::Tsc => "tsc",
            Wc65c816Instruction::Xba => "xba",
            Wc65c816Instruction::Xce => "xce",
        }
    }

    fn operands(&self) -> Iter<Wc65c816Operand> {
        match self {
            Wc65c816Instruction::Adc(o) => slice::from_ref(o),
            Wc65c816Instruction::Sbc(o) => slice::from_ref(o),
            Wc65c816Instruction::Cmp(o) => slice::from_ref(o),
            Wc65c816Instruction::Cpx(o) => slice::from_ref(o),
            Wc65c816Instruction::Cpy(o) => slice::from_ref(o),
            Wc65c816Instruction::Dec(o) => {
                if let Some(o) = o {
                    slice::from_ref(o)
                } else {
                    &[]
                }
            }
            Wc65c816Instruction::Dex(o) => {
                if let Some(o) = o {
                    slice::from_ref(o)
                } else {
                    &[]
                }
            }
            Wc65c816Instruction::Dey(o) => {
                if let Some(o) = o {
                    slice::from_ref(o)
                } else {
                    &[]
                }
            }
            Wc65c816Instruction::Inc(o) => {
                if let Some(o) = o {
                    slice::from_ref(o)
                } else {
                    &[]
                }
            }
            Wc65c816Instruction::Inx(o) => {
                if let Some(o) = o {
                    slice::from_ref(o)
                } else {
                    &[]
                }
            }
            Wc65c816Instruction::Iny(o) => {
                if let Some(o) = o {
                    slice::from_ref(o)
                } else {
                    &[]
                }
            }
            Wc65c816Instruction::And(o) => slice::from_ref(o),
            Wc65c816Instruction::Eor(o) => slice::from_ref(o),
            Wc65c816Instruction::Ora(o) => slice::from_ref(o),
            Wc65c816Instruction::Bit(o) => slice::from_ref(o),
            Wc65c816Instruction::Trb(o) => slice::from_ref(o),
            Wc65c816Instruction::Tsb(o) => slice::from_ref(o),
            Wc65c816Instruction::Asl(o) => slice::from_ref(o),
            Wc65c816Instruction::Lsr(o) => slice::from_ref(o),
            Wc65c816Instruction::Ror(o) => slice::from_ref(o),
            Wc65c816Instruction::Rol(o) => slice::from_ref(o),
            Wc65c816Instruction::Bcc(o) => slice::from_ref(o),
            Wc65c816Instruction::Bcs(o) => slice::from_ref(o),
            Wc65c816Instruction::Beq(o) => slice::from_ref(o),
            Wc65c816Instruction::Bne(o) => slice::from_ref(o),
            Wc65c816Instruction::Bmi(o) => slice::from_ref(o),
            Wc65c816Instruction::Bpl(o) => slice::from_ref(o),
            Wc65c816Instruction::Bra(o) => slice::from_ref(o),
            Wc65c816Instruction::Bvc(o) => slice::from_ref(o),
            Wc65c816Instruction::Bvs(o) => slice::from_ref(o),
            Wc65c816Instruction::Brl(o) => slice::from_ref(o),
            Wc65c816Instruction::Jmp(o) => slice::from_ref(o),
            Wc65c816Instruction::Jsr(o) => slice::from_ref(o),
            Wc65c816Instruction::Jsl(o) => slice::from_ref(o),
            Wc65c816Instruction::Rts => &[],
            Wc65c816Instruction::Rtl => &[],
            Wc65c816Instruction::Brk(o) => {
                if let Some(o) = o {
                    slice::from_ref(o)
                } else {
                    &[]
                }
            }
            Wc65c816Instruction::Cop(o) => {
                if let Some(o) = o {
                    slice::from_ref(o)
                } else {
                    &[]
                }
            }
            Wc65c816Instruction::Rti => &[],
            Wc65c816Instruction::Clc => &[],
            Wc65c816Instruction::Cld => &[],
            Wc65c816Instruction::Cli => &[],
            Wc65c816Instruction::Clv => &[],
            Wc65c816Instruction::Sec => &[],
            Wc65c816Instruction::Sed => &[],
            Wc65c816Instruction::Sei => &[],
            Wc65c816Instruction::Rep(o) => slice::from_ref(o),
            Wc65c816Instruction::Sep(o) => slice::from_ref(o),
            Wc65c816Instruction::Lda(o) => slice::from_ref(o),
            Wc65c816Instruction::Ldx(o) => slice::from_ref(o),
            Wc65c816Instruction::Ldy(o) => slice::from_ref(o),
            Wc65c816Instruction::Sta(o) => slice::from_ref(o),
            Wc65c816Instruction::Stx(o) => slice::from_ref(o),
            Wc65c816Instruction::Sty(o) => slice::from_ref(o),
            Wc65c816Instruction::Stz(o) => slice::from_ref(o),
            Wc65c816Instruction::Mvn(a) => a,
            Wc65c816Instruction::Mvp(a) => a,
            Wc65c816Instruction::Nop => &[],
            Wc65c816Instruction::Wdm(o) => {
                if let Some(o) = o {
                    slice::from_ref(o)
                } else {
                    &[]
                }
            }
            Wc65c816Instruction::Pea(o) => slice::from_ref(o),
            Wc65c816Instruction::Pha => &[],
            Wc65c816Instruction::Phx => &[],
            Wc65c816Instruction::Phy => &[],
            Wc65c816Instruction::Pla => &[],
            Wc65c816Instruction::Plx => &[],
            Wc65c816Instruction::Ply => &[],
            Wc65c816Instruction::Phb => &[],
            Wc65c816Instruction::Phd => &[],
            Wc65c816Instruction::Phk => &[],
            Wc65c816Instruction::Php => &[],
            Wc65c816Instruction::Plb => &[],
            Wc65c816Instruction::Pld => &[],
            Wc65c816Instruction::Plp => &[],
            Wc65c816Instruction::Stp => &[],
            Wc65c816Instruction::Wai => &[],
            Wc65c816Instruction::Tcd => &[],
            Wc65c816Instruction::Tcs => &[],
            Wc65c816Instruction::Tdc => &[],
            Wc65c816Instruction::Tsc => &[],
            Wc65c816Instruction::Xba => &[],
            Wc65c816Instruction::Xce => &[],
        }
        .iter()
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
    relocs: &'a mut dyn RelocationWriter<Relocation = Wc65c816Relocation>,
    writer: &'a mut (dyn Write + 'a),
    arch: &'a Wc65c816,
    acc8: bool,
    _idx8: bool,
}

impl<'a> InstructionWriter for Wc65c816InstructionWriter<'a> {
    type Arch = Wc65c816;

    type Instruction = Wc65c816Instruction;

    type Relocation = Wc65c816Relocation;

    type Error = std::io::Error;

    fn get_architecture(&self) -> &Self::Arch {
        &self.arch
    }

    fn write_instruction(&mut self, ins: Self::Instruction) -> Result<(), Self::Error> {
        match ins {
            Wc65c816Instruction::Adc(Wc65c816Operand::Immediate(v)) => {
                let insbytes = [0x69, (v & 0xff) as u8, (v >> 8) as u8];
                if self.acc8 {
                    self.write_bytes(&insbytes[0..2])?;
                } else {
                    self.write_bytes(&insbytes)?;
                }
            }
            Wc65c816Instruction::Adc(Wc65c816Operand::Address(v, None)) => match v {
                Wc65c816Address::Absolute(v) => {
                    let insbytes = ((v << 8) | 0x6F).to_le_bytes();
                    self.write_bytes(&insbytes)?;
                }
                Wc65c816Address::BankLocal(v) => {
                    let insbytes = [0x6D, v as u8, (v >> 8) as u8];
                    self.write_bytes(&insbytes)?;
                }
                i @ Wc65c816Address::PCRelLong(_)
                | i @ Wc65c816Address::PCRelShort(_)
                | i @ Wc65c816Address::Symbol(_) => {
                    let insbytes = [0x6D, 0, 0];
                    self.write_bytes(&insbytes)?;
                    let rel = Wc65c816Relocation {
                        rel_type: Wc65c816RelocationType::Short,
                        symbol: Either::Left(i),
                    };
                    self.relocs.write_relocation(rel)
                }
                i @ Wc65c816Address::LongSymbol(_) => {
                    let insbytes = [0x6F, 0, 0, 0];
                    self.write_bytes(&insbytes)?;
                    let rel = Wc65c816Relocation {
                        rel_type: Wc65c816RelocationType::Long,
                        symbol: Either::Left(i),
                    };
                    self.relocs.write_relocation(rel)
                }
                Wc65c816Address::Stack(off) => {
                    let insbytes = [0x63, off];
                    self.write_bytes(&insbytes)?;
                }
            },
            Wc65c816Instruction::Adc(Wc65c816Operand::Address(v, Some(Wc65c816Register::IdxX))) => {
                match v {
                    Wc65c816Address::Absolute(v) => {
                        let insbytes = ((v << 8) | 0x7F).to_le_bytes();
                        self.write_bytes(&insbytes)?;
                    }
                    Wc65c816Address::BankLocal(v) => {
                        let insbytes = [0x7D, v as u8, (v >> 8) as u8];
                        self.write_bytes(&insbytes)?;
                    }
                    i @ Wc65c816Address::PCRelLong(_)
                    | i @ Wc65c816Address::PCRelShort(_)
                    | i @ Wc65c816Address::Symbol(_) => {
                        let insbytes = [0x7D, 0, 0];
                        self.write_bytes(&insbytes)?;
                        let rel = Wc65c816Relocation {
                            rel_type: Wc65c816RelocationType::Short,
                            symbol: Either::Left(i),
                        };
                        self.relocs.write_relocation(rel)
                    }
                    i @ Wc65c816Address::LongSymbol(_) => {
                        let insbytes = [0x7F, 0, 0, 0];
                        self.write_bytes(&insbytes)?;
                        let rel = Wc65c816Relocation {
                            rel_type: Wc65c816RelocationType::Long,
                            symbol: Either::Left(i),
                        };
                        self.relocs.write_relocation(rel)
                    }
                    o => {
                        return Err(std::io::Error::new(
                            ErrorKind::Other,
                            format!("Invalid instruction: adc {},X", o),
                        ))
                    }
                }
            }
            Wc65c816Instruction::Sbc(_) => {}
            Wc65c816Instruction::Cmp(_) => {}
            Wc65c816Instruction::Cpx(_) => {}
            Wc65c816Instruction::Cpy(_) => {}
            Wc65c816Instruction::Dec(_) => {}
            Wc65c816Instruction::Dex(_) => {}
            Wc65c816Instruction::Dey(_) => {}
            Wc65c816Instruction::Inc(_) => {}
            Wc65c816Instruction::Inx(_) => {}
            Wc65c816Instruction::Iny(_) => {}
            Wc65c816Instruction::And(_) => {}
            Wc65c816Instruction::Eor(_) => {}
            Wc65c816Instruction::Ora(_) => {}
            Wc65c816Instruction::Bit(_) => {}
            Wc65c816Instruction::Trb(_) => {}
            Wc65c816Instruction::Tsb(_) => {}
            Wc65c816Instruction::Asl(_) => {}
            Wc65c816Instruction::Lsr(_) => {}
            Wc65c816Instruction::Ror(_) => {}
            Wc65c816Instruction::Rol(_) => {}
            Wc65c816Instruction::Bcc(_) => {}
            Wc65c816Instruction::Bcs(_) => {}
            Wc65c816Instruction::Beq(_) => {}
            Wc65c816Instruction::Bne(_) => {}
            Wc65c816Instruction::Bmi(_) => {}
            Wc65c816Instruction::Bpl(_) => {}
            Wc65c816Instruction::Bra(_) => {}
            Wc65c816Instruction::Bvc(_) => {}
            Wc65c816Instruction::Bvs(_) => {}
            Wc65c816Instruction::Brl(_) => {}
            Wc65c816Instruction::Jmp(_) => {}
            Wc65c816Instruction::Jsr(_) => {}
            Wc65c816Instruction::Jsl(_) => {}
            Wc65c816Instruction::Rts => {}
            Wc65c816Instruction::Rtl => {}
            Wc65c816Instruction::Brk(_) => {}
            Wc65c816Instruction::Cop(_) => {}
            Wc65c816Instruction::Rti => {}
            Wc65c816Instruction::Clc => {}
            Wc65c816Instruction::Cld => {}
            Wc65c816Instruction::Cli => {}
            Wc65c816Instruction::Clv => {}
            Wc65c816Instruction::Sec => {}
            Wc65c816Instruction::Sed => {}
            Wc65c816Instruction::Sei => {}
            Wc65c816Instruction::Rep(_) => {}
            Wc65c816Instruction::Sep(_) => {}
            Wc65c816Instruction::Lda(_) => {}
            Wc65c816Instruction::Ldx(_) => {}
            Wc65c816Instruction::Ldy(_) => {}
            Wc65c816Instruction::Sta(_) => {}
            Wc65c816Instruction::Stx(_) => {}
            Wc65c816Instruction::Sty(_) => {}
            Wc65c816Instruction::Stz(_) => {}
            Wc65c816Instruction::Mvn(_) => {}
            Wc65c816Instruction::Mvp(_) => {}
            Wc65c816Instruction::Nop => {}
            Wc65c816Instruction::Wdm(_) => {}
            Wc65c816Instruction::Pea(_) => {}
            Wc65c816Instruction::Pha => {}
            Wc65c816Instruction::Phx => {}
            Wc65c816Instruction::Phy => {}
            Wc65c816Instruction::Pla => {}
            Wc65c816Instruction::Plx => {}
            Wc65c816Instruction::Ply => {}
            Wc65c816Instruction::Phb => {}
            Wc65c816Instruction::Phd => {}
            Wc65c816Instruction::Phk => {}
            Wc65c816Instruction::Php => {}
            Wc65c816Instruction::Plb => {}
            Wc65c816Instruction::Pld => {}
            Wc65c816Instruction::Plp => {}
            Wc65c816Instruction::Stp => {}
            Wc65c816Instruction::Wai => {}
            Wc65c816Instruction::Tcd => {}
            Wc65c816Instruction::Tcs => {}
            Wc65c816Instruction::Tdc => {}
            Wc65c816Instruction::Tsc => {}
            Wc65c816Instruction::Xba => {}
            Wc65c816Instruction::Xce => {}
            i => {
                return Err(std::io::Error::new(
                    ErrorKind::InvalidData,
                    format!("invalid instruction {}", i),
                ))
            }
        }

        Ok(())
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
