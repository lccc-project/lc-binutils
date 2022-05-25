use crate::traits::Address;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum W65Address {
    Absolute(Address),
    Direct(Address),
    Long(Address),
    Rel(Address),
    IndexedX(Box<W65Address>),
    IndexedY(Box<W65Address>),
    Indirect(Box<W65Address>),
    IndirectLong(Box<W65Address>),
    Stack { off: i8 },
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum W65Operand {
    Address(W65Address),
    Immediate(u16),
    Accumulator,
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
    IndirectX,
    DirectIndirectY,
    IndirectHY,
    Imm8,
    Imm16,
    ImmA,
    ImmX,
    SrcDest,
    Stack,
    StackIndirectY,
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
                    $($($opcode => Some((Self:: $enum, $addr)),)*)*
                    _ => None
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
    }
}
