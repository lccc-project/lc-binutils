use std::ops::{Range, RangeFrom, RangeFull, RangeTo};

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
#[non_exhaustive]
pub enum CleverExtension {
    Base,
    Float,
    Vector,
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

trait HBits {
    fn from_bits(bits: u16) -> Self;
    fn to_hbits(self) -> u16;
}

impl HBits for bool {
    fn from_bits(bits: u16) -> Self {
        bits != 0
    }

    fn to_hbits(self) -> u16 {
        self as u16
    }
}

impl HBits for u8 {
    fn from_bits(bits: u16) -> Self {
        bits as u8
    }
    fn to_hbits(self) -> u16 {
        self as u16
    }
}

// Because ss is typically represented as `u16` for reasons
impl HBits for u16 {
    fn from_bits(bits: u16) -> Self {
        bits
    }
    fn to_hbits(self) -> u16 {
        self
    }
}

impl HBits for CleverRegister {
    fn from_bits(bits: u16) -> Self {
        Self(bits as u8)
    }

    fn to_hbits(self) -> u16 {
        self.0 as u16
    }
}

trait HBitRange {
    fn shift(&self) -> u32;
    fn mask(&self) -> u16;
}

impl HBitRange for u32 {
    fn shift(&self) -> u32 {
        *self
    }

    fn mask(&self) -> u16 {
        1u16 << (*self)
    }
}

impl HBitRange for Range<u32> {
    fn shift(&self) -> u32 {
        self.start
    }

    fn mask(&self) -> u16 {
        ((1u16 << ((self.end - 1) - self.start)) - 1) << self.start
    }
}

impl HBitRange for RangeFrom<u32> {
    fn shift(&self) -> u32 {
        self.start
    }

    fn mask(&self) -> u16 {
        ((1u16 << (4 - self.start)) - 1) << self.start
    }
}

impl HBitRange for RangeTo<u32> {
    fn shift(&self) -> u32 {
        0
    }

    fn mask(&self) -> u16 {
        (1u16 << (self.end - 1)) - 1
    }
}

impl HBitRange for RangeFull {
    fn shift(&self) -> u32 {
        0
    }

    fn mask(&self) -> u16 {
        0xf
    }
}

macro_rules! clever_instructions{
    {
        $([$enum:ident, $insn:literal, $opcode:literal, $operands:literal $(, { $($hfield:ident @ $range:expr => $ty:ty ),* $(,)?})? ]),* $(,)?
    } => {

        #[derive(Copy,Clone,Debug,Hash,PartialEq, Eq)]
        pub enum CleverOpcode{
            $($enum $({$($hfield: $ty),*})?),*
        }

        impl CleverOpcode{
            pub fn name(&self) -> &'static str{
                match self{
                    $(Self:: $enum $({ $($hfield: _),*})? => $insn),*
                }
            }

            pub fn opcode(&self) -> u16{
                match self{
                    $(Self:: $enum $({$($hfield),*})? => {
                        let base: u16 = $opcode;
                        #[allow(unused_mut)] // mut may be unused if the instruction doesn't have any h bits
                        let mut opc = base<<4;
                        $($({
                            let range = $range;

                            let bits = (HBits::to_hbits(*$hfield)<< HBitRange::shift(&range))&HBitRange::mask(&range);
                            opc |= bits;
                        })*)?
                        opc
                    })*
                }
            }
        }
    }
}

clever_instructions! {
    [Ud0, "ud", 0x000, 0],
    [Add, "add", 0x001, 2, {lock @ 3 => bool, flags @ 0 => bool}],
    [Sub, "sub", 0x002, 2, {lock @ 3 => bool, flags @ 0 => bool}],
    [And, "and", 0x003, 2, {lock @ 3 => bool, flags @ 0 => bool}],
    [Or , "or" , 0x004, 2, {lock @ 3 => bool, flags @ 0 => bool}],
    [Xor, "xor", 0x005, 2, {lock @ 3 => bool, flags @ 0 => bool}],

    [Mul, "mul", 0x006, 0, {ss @ 2..4 => u16, flags @ 0 => bool}],
    [Div, "div", 0x007, 0, {ss @ 2..4 => u16, wide @ 1 => bool, flags @ 0 => bool}],

    [Mov, "mov", 0x008, 2],
    [Lea, "lea", 0x009, 2],

    [MovRD, "mov", 0x00A, 1, {r @ .. => CleverRegister}],
    [MovRS, "mov", 0x00B, 1, {r @ .. => CleverRegister}],
    [LeaRD, "lea", 0x00C, 1, {r @ .. => CleverRegister}],

    [Nop10, "nop", 0x010, 0],
    [Nop11, "nop", 0x011, 1],
    [Nop12, "nop", 0x012, 2],

    [Push, "push", 0x014, 1],
    [Pop , "pop" , 0x015, 1],

    [PushR, "push", 0x016, 0, {r @ .. => CleverRegister}],
    [PopR , "pop" , 0x017, 0, {r @ .. => CleverRegister}],

    [Stogpr , "stogpr" , 0x018, 1],
    [Stoar  , "stoar"  , 0x019, 1],
    [Rstogpr, "rstogpr", 0x01A, 1],
    [Rstoar , "rstoar" , 0x01B, 1],
    [Pushgpr, "pushgpr", 0x01C, 0],
    [Pushar , "pushar" , 0x01D, 0],
    [Popgpr , "popgpr" , 0x01E, 0],
    [Popar  , "popar"  , 0x01F, 0],

    [Movsx, "movsx", 0x020, 2],

}
