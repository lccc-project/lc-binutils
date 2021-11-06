#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum CleverExtension {
    Base,
    Float,
    Vector,

    #[doc(hidden)]
    __Nonexhaustive,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct CleverRegister(pub u8);

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum CleverOperand {
    Register {
        reg: CleverRegister,
        ss: u16,
    },
    VectorRegister {
        reg: CleverRegister,
        vss: u16,
    },
    IndirectRegister {
        base: CleverRegister,
        scale: u16,
        ss: u16,
        index: CleverIndex,
    },
    ShortImmediate {
        val: u16,
        pcrel: bool,
    },
    LongImmediate {
        val: u64,
        ss: u16,
        pcrel: bool,
        mref: bool,
        zz: u16,
    },
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum CleverIndex {
    Register(CleverRegister),
    Abs(u16),
}
