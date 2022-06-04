use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

use crate::traits::Address;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum W65Address {
    Absolute(Address),
    Direct(Address),
    Long(Address),
    Rel8(Address),
    Rel16(Address),
    IndexedX(Box<W65Address>),
    IndexedY(Box<W65Address>),
    Indirect(Box<W65Address>),
    IndirectLong(Box<W65Address>),
    Stack { off: i8 },
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum W65Register {
    A,
    X,
    Y,
    D,
    Dbr,
    K,
    S,
    P,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum W65Operand {
    Address(W65Address),
    Immediate(u16),
    Register(W65Register),
    RegPair(W65Register, W65Register),
    Implied,
    SrcDest { src: Address, dest: Address },
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum W65AddrMode {
    Imp,
    Acc,
    Abs,
    AbsX,
    AbsY,
    Direct,
    DirectX,
    DirectY,
    Long,
    LongX,
    Rel8,
    Rel16,
    Rel,
    Indirect,
    IndirectLong,
    DirectIndirectLong,
    DirectIndirectLongY,
    DirectIndirect,
    DirectIndirectX,
    DirectIndirectExX,
    IndirectX,
    IndirectExX,
    DirectIndirectY,
    IndirectY,
    Imm8,
    Imm16,
    ImmA,
    ImmX,
    ImmY,
    SrcDest,
    Stack,
    StackIndirectY,
    IndX,
    IndY,
}

macro_rules! w65_opcodes{
    {$({$enum:ident, $insn:literal, [$($addr:ident $(| $aux_addr:pat)* => $opcode:literal),* $(,)?] $(,)?}),* $(,)?} => {
        #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
        pub enum W65Opcode{
            $($enum),*
        }

        impl W65Opcode {
            pub fn from_opcode(opc: u8) -> Option<(W65Opcode,W65AddrMode)>{
                match opc  {
                    $($(#[allow(unreachable_patterns)] $opcode => Some((Self:: $enum, $addr)),)*)*
                    _ => None
                }
            }

            pub fn accepts_addr_mode(&self, mode: W65AddrMode) -> bool{
                match self{
                    $(Self:: $enum => match mode{
                        $($addr $(| $aux_addr)* => true,)*
                        _ => false
                    })*
                }
            }

            pub fn is_rel_addr(&self) -> bool{
                match self{
                    $(Self:: $enum => matches!((W65AddrMode::Rel8,W65AddrMode::Rel16),(W65AddrMode::Imp,W65AddrMode::Imp) $(|($addr $(| $aux_addr)*,_)|(_,$addr $(| $aux_addr)*))*),)*
                }
            }
            pub fn insn(&self) -> &'static str {
                match self{
                    $(Self:: $enum => $insn),*
                }
            }

            pub fn opcode(&self, addr: W65AddrMode) -> Option<u8>{
                match self{
                    $(Self:: $enum => {
                        match addr{
                            $($addr $(|$aux_addr)* => Some($opcode),)*
                            #[allow(unreachable_patterns)] _ => None
                        }
                    }),*
                }
            }

            pub fn immediate_size(&self) -> Option<W65AddrMode>{
                match self{
                    $(Self:: $enum => {
                        match W65AddrMode::ImmA{
                            $($addr $(|$aux_addr)* => return Some(W65AddrMode::ImmA),)*
                            #[allow(unreachable_patterns)] _ => {}
                        }
                        match W65AddrMode::ImmX{
                            $($addr $(|$aux_addr)* => return Some(W65AddrMode::ImmX),)*
                            #[allow(unreachable_patterns)] _ => {}
                        }
                        match W65AddrMode::ImmY{
                            $($addr $(|$aux_addr)* => return Some(W65AddrMode::ImmA),)*
                            #[allow(unreachable_patterns)] _ => {}
                        }
                        match W65AddrMode::Imm8{
                            $($addr $(|$aux_addr)* => return Some(W65AddrMode::Imm8),)*
                            #[allow(unreachable_patterns)] _ => {}
                        }
                        match W65AddrMode::Imm16{
                            $($addr $(|$aux_addr)* => Some(W65AddrMode::Imm16),)*
                            #[allow(unreachable_patterns)] _ => None
                        }
                    })*
                }
            }
        }
    }
}

use W65AddrMode::*;

w65_opcodes! {
    {
        Adc, "adc", [
            DirectIndirectX => 0x61,
            Stack => 0x63,
            Direct => 0x65,
            DirectIndirectLong => 0x67,
            ImmA | Imm8 | Imm16 => 0x69,
            Abs => 0x6D,
            Long => 0x6F,
            DirectIndirectY => 0x71,
            DirectIndirect => 0x72,
            StackIndirectY => 0x73,
            DirectX => 0x75,
            DirectIndirectLongY => 0x77,
            AbsY => 0x79,
            AbsX => 0x7D,
            LongX => 0x7F
        ]
    },
    {
        Sbc, "sbc", [
            DirectIndirectX => 0xE1,
            Stack => 0xE3,
            Direct => 0xE5,
            DirectIndirectLong => 0xE7,
            ImmA | Imm8 | Imm16 => 0xE9,
            Abs => 0xED,
            Long => 0xEF,
            DirectIndirectY => 0xF1,
            DirectIndirect => 0xF2,
            StackIndirectY => 0xF3,
            DirectX => 0xF5,
            DirectIndirectLongY => 0xF7,
            AbsY => 0xF9,
            AbsX => 0xFD,
            LongX => 0xFF
        ]
    },
    {
        Cmp, "cmp", [
            DirectIndirectX => 0xD1,
            Stack => 0xD3,
            Direct => 0xD5,
            DirectIndirectLong => 0xD7,
            ImmA | Imm8 | Imm16 => 0xD9,
            Abs => 0xDD,
            Long => 0xDF,
            DirectIndirectY => 0xE1,
            DirectIndirect => 0xE2,
            StackIndirectY => 0xE3,
            DirectX => 0xE5,
            DirectIndirectLongY => 0xE7,
            AbsY => 0xE9,
            AbsX => 0xED,
            LongX => 0xEF
        ]
    },
    {
        Cpx, "cpx", [
            ImmY | Imm8 | Imm16 => 0xE0,
            Direct => 0xE4,
            Abs => 0xEC,
        ]
    },
    {
        Cpy, "cpy", [
            ImmY | Imm8 | Imm16 => 0xC0,
            Direct => 0xC4,
            Abs => 0xCC,
        ]
    },
    {
        Dec, "dec", [
            Acc => 0x3A,
            Direct => 0xC6,
            Abs => 0xCE,
            DirectX => 0xD6,
            AbsX => 0xDE
        ]
    },
    {
        Inc, "inc", [
            Acc => 0x1A,
            Direct => 0xE6,
            Abs => 0xEE,
            DirectX => 0xF6,
            AbsX => 0xFE
        ]
    },
    {
        Dex, "dex", [
            Imp => 0xCA
        ]
    },
    {
        Dey, "dey", [
            Imp => 0x88
        ]
    },
    {
        Inx, "dex", [
            Imp => 0xE8
        ]
    },
    {
        Iny, "iny", [
            Imp => 0xC8
        ]
    },
    {
        And, "and", [
            DirectIndirectX => 0x21,
            Stack => 0x23,
            Direct => 0x25,
            DirectIndirectLong => 0x27,
            ImmA | Imm8 | Imm16 => 0x29,
            Abs => 0x2D,
            Long => 0x2F,
            DirectIndirectY => 0x31,
            DirectIndirect => 0x32,
            StackIndirectY => 0x33,
            DirectX => 0x35,
            DirectIndirectLongY => 0x37,
            AbsY => 0x39,
            AbsX => 0x3D,
            LongX => 0x3F
        ]
    },
    {
        Eor, "eor", [
            DirectIndirectX => 0x41,
            Stack => 0x43,
            Direct => 0x45,
            DirectIndirectLong => 0x47,
            ImmA | Imm8 | Imm16 => 0x49,
            Abs => 0x4D,
            Long => 0x4F,
            DirectIndirectY => 0x51,
            DirectIndirect => 0x52,
            StackIndirectY => 0x53,
            DirectX => 0x55,
            DirectIndirectLongY => 0x57,
            AbsY => 0x59,
            AbsX => 0x5D,
            LongX => 0x5F
        ]
    },
    {
        Ora, "ora", [
            DirectIndirectX => 0x01,
            Stack => 0x03,
            Direct => 0x05,
            DirectIndirectLong => 0x07,
            ImmA | Imm8 | Imm16 => 0x09,
            Abs => 0x0D,
            Long => 0x0F,
            DirectIndirectY => 0x11,
            DirectIndirect => 0x12,
            StackIndirectY => 0x13,
            DirectX => 0x15,
            DirectIndirectLongY => 0x17,
            AbsY => 0x19,
            AbsX => 0x1D,
            LongX => 0x1F
        ]
    },
    {
        Bit, "bit", [
            Direct => 0x24,
            Abs => 0x2C,
            DirectX => 0x34,
            AbsX => 0x3C,
            ImmA | Imm8 | Imm16 => 0x89
        ]
    },
    {
        Trb, "trb", [
            Direct => 0x14,
            Abs => 0x1C
        ]
    },
    {
        Tsb, "tsb", [
            Direct => 0x04,
            Abs => 0x0C
        ]
    },
    {
        Asl, "asl", [
            Direct => 0x06,
            Acc => 0x0A,
            Abs => 0x0E,
            DirectX => 0x16,
            AbsX => 0x1E
        ]
    },
    {
        Lsr, "lsr", [
            Direct => 0x46,
            Acc => 0x4A,
            Abs => 0x4E,
            DirectX => 0x56,
            AbsX => 0x5E
        ]
    },
    {
        Rol, "rol", [
            Direct => 0x26,
            Acc => 0x2A,
            Abs => 0x2E,
            DirectX => 0x36,
            AbsX => 0x3E
        ]
    },
    {
        Ror, "ror", [
            Direct => 0x66,
            Acc => 0x6A,
            Abs => 0x6E,
            DirectX => 0x76,
            AbsX => 0x7E
        ]
    },
    {
        Bcc, "bcc", [
            Rel8 => 0x90
        ]
    },
    {
        Bcs, "bcs", [
            Rel8 => 0xB0
        ]
    },
    {
        Beq, "beq", [
            Rel8 => 0xF0
        ]
    },
    {
        Bmi, "bmi", [
            Rel8 => 0x30
        ]
    },
    {
        Bne, "bne", [
            Rel8 => 0xD0
        ]
    },
    {
        Bpl, "bpl", [
            Rel8 => 0x10
        ]
    },
    {
        Bra, "bra", [
            Rel8 => 0x80,
            Rel16 => 0x82,
        ]
    },
    {
        Brl, "", []
    },
    {
        Bvc, "bvc", [
            Rel8 => 0x50
        ]
    },
    {
        Bvs, "bvs", [
            Rel8 => 0x70
        ]
    },
    {
        Jmp, "jmp", [
            Abs => 0x4C,
            Long => 0x5C,
            Indirect => 0x6C,
            IndirectX => 0x7C,
            IndirectLong => 0xDc
        ]
    },
    {
        Jsr, "jsr", [
            Long => 0x22,
            Abs => 0x20,
            AbsX => 0xFc
        ]
    },
    {
        Rtl, "rtl", [
            Imp => 0x6B
        ]
    },
    {
        Rts, "rts", [
            Imp => 0x60
        ]
    },
    {
        Brk, "brk", [
            Imm8 => 0x00
        ]
    },
    {
        Cop, "cop", [
            Imm8 => 0x02
        ]
    },
    {
        Rti, "rti", [
            Imp => 0x40
        ]
    },
    {
        Clc, "clc", [
            Imp => 0x18
        ]
    },
    {
        Cld, "cld", [
            Imp => 0xD8
        ]
    },
    {
        Cli, "cli", [
            Imp => 0x58
        ]
    },
    {
        Clv, "clv", [
            Imp => 0xB8
        ]
    },
    {
        Sec, "sec", [
            Imp => 0x38
        ]
    },
    {
        Sed, "sed", [
            Imp => 0xF8
        ]
    },
    {
        Sei, "sei", [
            Imp => 0x78
        ]
    },
    {
        Rep, "rep", [
            Imm8 => 0xC2
        ]
    },
    {
        Sep, "sep", [
            Imm8 => 0xE2,
        ]
    },
    {
        Lda, "lda", [
            DirectIndirectX => 0xA1,
            Stack => 0xA3,
            Direct => 0xA5,
            DirectIndirectLong => 0xA7,
            ImmA | Imm8 | Imm16 => 0xA9,
            Abs => 0xAD,
            Long => 0xAF,
            DirectIndirectY => 0xB1,
            DirectIndirect => 0xB2,
            StackIndirectY => 0xB3,
            DirectX => 0xB5,
            DirectIndirectLongY => 0xB7,
            AbsY => 0xB9,
            AbsX => 0xBD,
            LongX => 0xBF
        ]
    },
    {
        Sta, "sta", [
            DirectIndirectX => 0x81,
            Stack => 0x83,
            Direct => 0x85,
            DirectIndirectLong => 0x87,
            Abs => 0x8D,
            Long => 0x8F,
            DirectIndirectY => 0x91,
            DirectIndirect => 0x92,
            StackIndirectY => 0x93,
            DirectX => 0x95,
            DirectIndirectLongY => 0x97,
            AbsY => 0x99,
            AbsX => 0x9D,
            LongX => 0x9F
        ]
    },
    {
        Ldx, "ldx", [
            ImmX => 0xA2,
            Direct => 0xA6,
            Abs => 0xAE,
            DirectY => 0xB6,
            AbsY => 0xBE
        ]
    },
    {
        Ldy, "ldy", [
            ImmY => 0xA0,
            Direct => 0xA4,
            Abs => 0xAC,
            DirectX => 0xB4,
            AbsX => 0xBC
        ]
    },
    {
        Stx, "stx", [
            Direct => 0x86,
            Abs => 0x8E,
            DirectY => 0x96,
            AbsY => 0x9E
        ]
    },
    {
        Sty, "sty", [
            Direct => 0x84,
            Abs => 0x8C,
            DirectX => 0x94,
            AbsX => 0x9C
        ]
    },
    {
        Stz, "stz", [
            Direct => 0x64,
            DirectX => 0x74,
            Abs => 0x9C,
            AbsX => 0x9E,
        ]
    },
    {
        Mvn, "mvn", [
            SrcDest => 0x54
        ]
    },
    {
        Mvp, "mvp", [
            SrcDest => 0x44
        ]
    },
    {
        Nop, "nop", [
            Imp => 0xEA
        ]
    },
    {
        Wdm, "wdm", [
            Imm8 => 0x42
        ]
    },
    {
        Pea, "pea", [
            Abs => 0xF4,
            DirectIndirect => 0xD4,
            Rel16 => 0x62
        ]
    },
    {
        Ph, "ph", []
    },
    {
        Pha, "pha", [
            Imp => 0x48
        ]
    },
    {
        Phx, "phx", [
            Imp => 0xDA
        ]
    },
    {
        Phy, "phy", [
            Imp => 0x5A
        ]
    },
    {
        Pl, "pl", []
    },
    {
        Pla, "pla", [
            Imp => 0x68
        ]
    },
    {
        Plx, "plx", [
            Imp => 0xFA
        ]
    },
    {
        Ply, "ply", [
            Imp => 0x7A
        ]
    },
    {
        Phb, "phb", [
            Imp => 0x8B
        ]
    },
    {
        Phd, "phd", [
            Imp => 0x0B
        ]
    },
    {
        Phk, "phk", [
            Imp => 0x4B
        ]
    },
    {
        Php, "php", [
            Imp => 0x08
        ]
    },
    {
        Plb, "plb", [
            Imp => 0xAB
        ]
    },
    {
        Pld, "pld", [
            Imp => 0x2B
        ]
    },
    {
        Plp, "plp", [
            Imp => 0x28
        ]
    },
    {
        Stp, "stp", [
            Imp => 0xDB
        ]
    },
    {
        Wai, "wai", [
            Imp => 0xCB
        ]
    },
    {
        Tr, "tr", []
    },
    {
        Tax, "tax", [
            Imp => 0xAA
        ]
    },
    {
        Tay, "tay", [
            Imp => 0xA8
        ]
    },
    {
        Tsx, "tsx", [
            Imp => 0xBA
        ]
    },
    {
        Txa, "txa", [
            Imp => 0x8A
        ]
    },
    {
        Txs, "txs", [
            Imp => 0x9A
        ]
    },
    {
        Txy, "txy", [
            Imp => 0x9B
        ]
    },
    {
        Tya, "tya", [
            Imp => 0x98
        ]
    },
    {
        Tyx, "tyx", [
            Imp => 0xBB
        ]
    },
    {
        Tad, "Tad", [
            Imp => 0x5B
        ]
    },
    {
        Tas, "tas", [
            Imp => 0x1B
        ]
    },
    {
        Tda, "tda", [
            Imp => 0x7B
        ]
    },
    {
        Tsa, "tsa", [
            Imp => 0x3B
        ]
    },
    {
        Xba, "xba", [
            Imp => 0xEB
        ]
    },
    {
        Xce, "xce", [
            Imp => 0xFB
        ]
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct W65Instruction {
    mode: Option<W65Mode>,
    opc: W65Opcode,
    opr: W65Operand,
}

impl W65Instruction {
    pub fn new(opc: W65Opcode, opr: W65Operand) -> Self {
        Self {
            mode: None,
            opc,
            opr,
        }
    }

    pub fn new_in_mode(mode: W65Mode, opc: W65Opcode, opr: W65Operand) -> Self {
        Self {
            mode: Some(mode),
            opc,
            opr,
        }
    }

    pub fn addr_mode(&self) -> Option<W65AddrMode> {
        match &self.opr {
            W65Operand::Address(addr) => match addr {
                W65Address::Absolute(_) => Some(W65AddrMode::Abs),
                W65Address::Direct(_) => Some(W65AddrMode::Direct),
                W65Address::Long(_) => Some(W65AddrMode::Long),
                W65Address::Rel8(_) => Some(W65AddrMode::Rel8),
                W65Address::Rel16(_) => Some(W65AddrMode::Rel8),
                W65Address::IndexedX(inner) => match &**inner {
                    W65Address::Absolute(_) => Some(W65AddrMode::AbsX),
                    W65Address::Direct(_) => Some(W65AddrMode::DirectX),
                    W65Address::Indirect(addr) => match &**addr {
                        W65Address::Direct(_) => Some(W65AddrMode::DirectIndirectX),
                        W65Address::Absolute(_) => Some(W65AddrMode::IndirectX),
                        _ => None,
                    },
                    _ => None,
                },
                W65Address::IndexedY(inner) => match &**inner {
                    W65Address::Absolute(_) => Some(W65AddrMode::AbsY),
                    W65Address::Direct(_) => Some(W65AddrMode::DirectY),
                    W65Address::IndirectLong(inner) => match &**inner {
                        W65Address::Direct(_) => Some(W65AddrMode::DirectIndirectLongY),
                        _ => None,
                    },
                    W65Address::Indirect(addr) => match &**addr {
                        W65Address::Direct(_) => Some(W65AddrMode::DirectIndirectY),
                        W65Address::Absolute(_) => Some(W65AddrMode::IndirectY),
                        W65Address::Stack { .. } => Some(W65AddrMode::StackIndirectY),
                        _ => None,
                    },
                    _ => None,
                },
                W65Address::Indirect(inner) => match &**inner {
                    W65Address::Absolute(_) => Some(W65AddrMode::Indirect),
                    W65Address::Direct(_) => Some(W65AddrMode::Direct),
                    W65Address::IndexedX(inner) => match &**inner {
                        W65Address::Absolute(_) => Some(W65AddrMode::IndirectX),
                        W65Address::Direct(_) => Some(W65AddrMode::DirectIndirectX),
                        _ => None,
                    },
                    _ => None,
                },
                W65Address::IndirectLong(_) => todo!(),
                W65Address::Stack { .. } => Some(W65AddrMode::Stack),
            },
            W65Operand::Immediate(_) => {
                if let Some(mode) = self.opc.immediate_size() {
                    Some(mode)
                } else if self.opc.is_rel_addr() {
                    if self.opc.accepts_addr_mode(W65AddrMode::Rel16) {
                        Some(W65AddrMode::Rel16)
                    } else {
                        Some(W65AddrMode::Rel8)
                    }
                } else {
                    None
                }
            }
            W65Operand::Implied => Some(W65AddrMode::Imp),
            W65Operand::SrcDest { .. } => Some(W65AddrMode::SrcDest),
            W65Operand::Register(W65Register::A) => Some(W65AddrMode::Acc),
            W65Operand::Register(W65Register::X) => Some(W65AddrMode::IndX),
            W65Operand::Register(W65Register::Y) => Some(W65AddrMode::IndY),
            _ => None,
        }
    }
}

macro_rules! w65_synthetic_instructions{
    ($([$base_opc:pat, $base_operand:pat => $actual_opc:expr, $actual_operand:expr])*) => {
        impl W65Instruction{
            pub fn into_real(self) -> Self{
                let mode = self.mode;
                match (self.opc,self.opr){
                    $(($base_opc, $base_operand) => Self{opc: $actual_opc, opr: $actual_operand,mode},)*
                    #[allow(unreachable_patterns)] (opc,opr) => Self{opc,opr,mode}
                }
            }
        }
    }
}

w65_synthetic_instructions! {
    [W65Opcode::Lda, W65Operand::Register(W65Register::X) => W65Opcode::Txa, W65Operand::Implied]
    [W65Opcode::Lda, W65Operand::Register(W65Register::Y) => W65Opcode::Tya, W65Operand::Implied]
    [W65Opcode::Lda, W65Operand::Register(W65Register::A) => W65Opcode::Nop, W65Operand::Implied]
    [W65Opcode::Lda, W65Operand::Register(W65Register::S) => W65Opcode::Tsa, W65Operand::Implied]
    [W65Opcode::Lda, W65Operand::Register(W65Register::D) => W65Opcode::Tda, W65Operand::Implied]
    [W65Opcode::Ldx, W65Operand::Register(W65Register::A) => W65Opcode::Tax, W65Operand::Implied]
    [W65Opcode::Ldx, W65Operand::Register(W65Register::Y) => W65Opcode::Tyx, W65Operand::Implied]
    [W65Opcode::Ldx, W65Operand::Register(W65Register::X) => W65Opcode::Nop, W65Operand::Implied]
    [W65Opcode::Ldx, W65Operand::Register(W65Register::S) => W65Opcode::Tsx, W65Operand::Implied]
    [W65Opcode::Ldy, W65Operand::Register(W65Register::A) => W65Opcode::Tay, W65Operand::Implied]
    [W65Opcode::Ldy, W65Operand::Register(W65Register::X) => W65Opcode::Txy, W65Operand::Implied]
    [W65Opcode::Ldy, W65Operand::Register(W65Register::Y) => W65Opcode::Nop, W65Operand::Implied]
    [W65Opcode::Tr, W65Operand::RegPair(W65Register::A,W65Register::X) => W65Opcode::Txa, W65Operand::Implied]
    [W65Opcode::Tr, W65Operand::RegPair(W65Register::A,W65Register::Y) => W65Opcode::Tya, W65Operand::Implied]
    [W65Opcode::Tr, W65Operand::RegPair(W65Register::A,W65Register::A) => W65Opcode::Nop, W65Operand::Implied]
    [W65Opcode::Tr, W65Operand::RegPair(W65Register::A,W65Register::S) => W65Opcode::Tsa, W65Operand::Implied]
    [W65Opcode::Tr, W65Operand::RegPair(W65Register::A,W65Register::D) => W65Opcode::Tda, W65Operand::Implied]
    [W65Opcode::Tr, W65Operand::RegPair(W65Register::X,W65Register::A) => W65Opcode::Tax, W65Operand::Implied]
    [W65Opcode::Tr, W65Operand::RegPair(W65Register::X,W65Register::Y) => W65Opcode::Tyx, W65Operand::Implied]
    [W65Opcode::Tr, W65Operand::RegPair(W65Register::X,W65Register::X) => W65Opcode::Nop, W65Operand::Implied]
    [W65Opcode::Tr, W65Operand::RegPair(W65Register::X,W65Register::S) => W65Opcode::Tsx, W65Operand::Implied]
    [W65Opcode::Tr, W65Operand::RegPair(W65Register::Y,W65Register::A) => W65Opcode::Tay, W65Operand::Implied]
    [W65Opcode::Tr, W65Operand::RegPair(W65Register::Y,W65Register::X) => W65Opcode::Txy, W65Operand::Implied]
    [W65Opcode::Tr, W65Operand::RegPair(W65Register::Y,W65Register::Y) => W65Opcode::Nop, W65Operand::Implied]
    [W65Opcode::Sta, W65Operand::Register(W65Register::X) => W65Opcode::Tax, W65Operand::Implied]
    [W65Opcode::Sta, W65Operand::Register(W65Register::Y) => W65Opcode::Tay, W65Operand::Implied]
    [W65Opcode::Sta, W65Operand::Register(W65Register::A) => W65Opcode::Nop, W65Operand::Implied]
    [W65Opcode::Sta, W65Operand::Register(W65Register::S) => W65Opcode::Tas, W65Operand::Implied]
    [W65Opcode::Sta, W65Operand::Register(W65Register::D) => W65Opcode::Tad, W65Operand::Implied]
    [W65Opcode::Stx, W65Operand::Register(W65Register::A) => W65Opcode::Txa, W65Operand::Implied]
    [W65Opcode::Stx, W65Operand::Register(W65Register::Y) => W65Opcode::Txy, W65Operand::Implied]
    [W65Opcode::Stx, W65Operand::Register(W65Register::X) => W65Opcode::Nop, W65Operand::Implied]
    [W65Opcode::Stx, W65Operand::Register(W65Register::S) => W65Opcode::Txs, W65Operand::Implied]
    [W65Opcode::Sty, W65Operand::Register(W65Register::A) => W65Opcode::Tya, W65Operand::Implied]
    [W65Opcode::Sty, W65Operand::Register(W65Register::X) => W65Opcode::Tyx, W65Operand::Implied]
    [W65Opcode::Sty, W65Operand::Register(W65Register::Y) => W65Opcode::Nop, W65Operand::Implied]
    [W65Opcode::Tr, W65Operand::RegPair(W65Register::S,W65Register::A) => W65Opcode::Tsa, W65Operand::Implied]
    [W65Opcode::Tr, W65Operand::RegPair(W65Register::D, W65Register::A) => W65Opcode::Tda, W65Operand::Implied]
    [W65Opcode::Tr, W65Operand::RegPair(W65Register::S, W65Register::S) => W65Opcode::Tsx, W65Operand::Implied]
    [W65Opcode::Brk, W65Operand::Implied => W65Opcode::Brk, W65Operand::Immediate(0)]
    [W65Opcode::Brl, W65Operand::Address(addr) => W65Opcode::Bra, W65Operand::Address(addr)]
    [W65Opcode::Ph, W65Operand::Register(W65Register::A) => W65Opcode::Pha, W65Operand::Implied]
    [W65Opcode::Ph, W65Operand::Register(W65Register::X) => W65Opcode::Phx, W65Operand::Implied]
    [W65Opcode::Ph, W65Operand::Register(W65Register::Y) => W65Opcode::Phy, W65Operand::Implied]
    [W65Opcode::Ph, W65Operand::Register(W65Register::D) => W65Opcode::Phd, W65Operand::Implied]
    [W65Opcode::Ph, W65Operand::Register(W65Register::Dbr) => W65Opcode::Phb, W65Operand::Implied]
    [W65Opcode::Ph, W65Operand::Register(W65Register::K) => W65Opcode::Phk, W65Operand::Implied]
    [W65Opcode::Ph, W65Operand::Register(W65Register::P) => W65Opcode::Php, W65Operand::Implied]
    [W65Opcode::Pl, W65Operand::Register(W65Register::A) => W65Opcode::Pla, W65Operand::Implied]
    [W65Opcode::Pl, W65Operand::Register(W65Register::X) => W65Opcode::Plx, W65Operand::Implied]
    [W65Opcode::Pl, W65Operand::Register(W65Register::Y) => W65Opcode::Ply, W65Operand::Implied]
    [W65Opcode::Pl, W65Operand::Register(W65Register::D) => W65Opcode::Pld, W65Operand::Implied]
    [W65Opcode::Pl, W65Operand::Register(W65Register::Dbr) => W65Opcode::Plb, W65Operand::Implied]
    [W65Opcode::Pl, W65Operand::Register(W65Register::P) => W65Opcode::Plp, W65Operand::Implied]
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct W65Mode {
    bits: u16,
}

impl W65Mode {
    pub const M: W65Mode = W65Mode { bits: 1 };
    pub const X: W65Mode = W65Mode { bits: 2 };
    pub const E: W65Mode = W65Mode { bits: 4 };

    pub fn is_acc16(self) -> bool {
        !self.is_emu() && ((self.bits & 1) == 0)
    }

    pub fn is_idx16(self) -> bool {
        !self.is_emu() && ((self.bits & 2) == 0)
    }

    pub fn is_emu(self) -> bool {
        self.bits & 4 != 0
    }

    pub fn get_immediate_size(self, imm: W65AddrMode) -> Option<usize> {
        match imm {
            W65AddrMode::Imm8 => Some(1),
            W65AddrMode::Imm16 => Some(2),
            W65AddrMode::ImmA => Some(1 + (self.is_acc16() as usize)),
            W65AddrMode::ImmX | W65AddrMode::ImmY => Some(1 + (self.is_idx16() as usize)),
            _ => None,
        }
    }
}

impl Not for W65Mode {
    type Output = Self;
    fn not(self) -> Self {
        Self {
            bits: (!self.bits) & 0x7,
        }
    }
}

impl BitOrAssign for W65Mode {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

impl BitAndAssign for W65Mode {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs;
    }
}

impl BitXorAssign for W65Mode {
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = *self ^ rhs;
    }
}

impl BitOr for W65Mode {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self {
            bits: self.bits | rhs.bits,
        }
    }
}

impl BitAnd for W65Mode {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Self {
            bits: self.bits | rhs.bits,
        }
    }
}

impl BitXor for W65Mode {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self {
        Self {
            bits: self.bits | rhs.bits,
        }
    }
}
