use crate::traits::Address;

pub enum W65Operand {
    Acc,
    Direct(Address),
    Abs(Address),
    Rel(Address),
    Long(Address),
    Imm(u16),
    ImmSym(Address),
    SrcDst { src: Address, dst: Address },
    Implied,
}

pub enum W65AddrMode {
    Imp,
    Acc,
    Abs,
    Direct,
    Long,
    Rel8,
    Rel16,
    IndirectLong,
}
