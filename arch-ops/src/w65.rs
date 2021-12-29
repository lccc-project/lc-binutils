use crate::traits::Address;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum W65Address {
    Absolute(Address),
    Direct(Address),
    Long(Address),
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
    Rel8,
    Rel16,
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
}

macro_rules! w65_opcodes{
    {$(($enum:ident, $insn:literal, $opcode:literal, $addr:expr $(,)?)),* $(,)?} => {
        #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
        pub enum W65Opcode{
            $($enum),*
        }

        impl W65Opcode {
            pub fn opcode(&self) -> u8 {
                match self{
                    $(Self:: $enum => $opcode),*
                }
            }

            pub fn insn(&self) -> &'static str {
                match self{
                    $(Self:: $enum => $insn),*
                }
            }

            pub fn addr_mode(&self) -> W65AddrMode{
                match self{
                    $(Self:: $enum => $addr),*
                }
            }
        }
    }
}

use W65AddrMode::*;

w65_opcodes! {
    (AdcDirX, "adc", 0x61, DirectIndirectX),
    (AdcStk, "adc", 0x63, Stack),
    (AdcDir, "adc", 0x65, Direct),

}
