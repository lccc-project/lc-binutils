use std::ops::{Range, RangeFrom, RangeFull, RangeTo};

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
#[non_exhaustive]
pub enum CleverExtension {
    Base,
    Float,
    Vector,
}

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct CleverRegister(pub u8);

pub struct RegisterFromStrError;

macro_rules! clever_registers{
    {
        $($name:ident $(| $altnames:ident)* => $val:expr),* $(,)?
    } => {
        impl ::core::str::FromStr for CleverRegister{
            type Err = RegisterFromStrError;
            fn from_str(st: &str) -> Result<Self,Self::Err>{
                match st{
                    $(
                        ::core::stringify!($name) $(| ::core::stringify!($altnames))* => Ok(Self($val)),
                    )*
                    _ => Err(RegisterFromStrError)
                }
            }
        }
        impl ::core::fmt::Display for CleverRegister{
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result{
                match self{
                    $(CleverRegister($val) => f.write_str(::core::stringify!($name)),)*
                    CleverRegister(val) => f.write_fmt(::core::format_args!("r{}",val))
                }
            }
        }

        impl ::core::fmt::Debug for CleverRegister{
            fn fmt(&self, f: &mut core::fmt::Formatter) -> ::core::fmt::Result{
                struct DontEscape(&'static str);
                impl ::core::fmt::Debug for DontEscape{
                    fn fmt(&self, f: &mut core::fmt::Formatter) -> ::core::fmt::Result{
                        f.write_str(self.0)
                    }
                }

                match self{
                    $(CleverRegister($val) => {
                        f.debug_tuple("CleverRegister")
                            .field(&DontEscape(::core::stringify!($name))).finish()
                    })*
                    CleverRegister(val) => f.debug_tuple("CleverRegister").field(&val).finish()
                }
            }
        }
    }
}

clever_registers! {
    r0 | racc => 0,
    r1 | rsrc => 1,
    r2 | rdst => 2,
    r3 | rcnt => 3,
    r4 => 4,
    r5 => 5,
    r6 | fbase => 6,
    r7 | sptr => 7,
    r8 => 8,
    r9 => 9,
    r10 => 10,
    r11 => 11,
    r12 => 12,
    r13 => 13,
    r14 => 14,
    r15 | link => 15,
    ip => 16,
    flags => 17,
    fpcw => 19,
    f0 => 24,
    f1 => 25,
    f2 => 26,
    f3 => 27,
    f4 => 28,
    f5 => 29,
    f6 => 30,
    f7 => 31,
    v0l => 64,
    v0h => 65,
    v1l => 66,
    v1h => 67,
    v2l => 68,
    v2h => 69,
    v3l => 70,
    v3h => 71,
    v4l => 72,
    v4h => 73,
    v5l => 74,
    v5h => 75,
    v6l => 76,
    v6h => 77,
    v7l => 78,
    v7h => 79,
    v8l => 80,
    v8h => 81,
    v9l => 82,
    v9h => 83,
    v10l => 84,
    v10h => 85,
    v11l => 86,
    v11h => 87,
    v12l => 88,
    v12h => 89,
    v13l => 90,
    v13h => 91,
    v14l => 92,
    v14h => 93,
    v15l => 94,
    v15h => 95,
    v16l => 96,
    v16h => 97,
    v17l => 98,
    v17h => 99,
    v18l => 100,
    v18h => 101,
    v19l => 102,
    v19h => 103,
    v20l => 104,
    v20h => 105,
    v21l => 106,
    v21h => 107,
    v22l => 108,
    v22h => 109,
    v23l => 110,
    v23h => 111,
    v24l => 112,
    v24h => 113,
    v25l => 114,
    v25h => 115,
    v26l => 116,
    v26h => 117,
    v27l => 118,
    v27h => 119,
    v28l => 120,
    v28h => 121,
    v29l => 122,
    v29h => 123,
    v30l => 124,
    v30h => 125,
    v31l => 126,
    v31h => 127,
    cr0 => 128,
    page | cr1 => 129,
    flprotected | cr2 => 130,
    scdp | cr3 => 131,
    scsp | cr4 => 132,
    sccr | cr5 => 133,
    itabp | cr6 => 134,
    ciread | cr7 => 135,
}

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
#[repr(u8)]
pub enum ConditionCode {
    Parity,
    Carry,
    Overflow,
    Zero,
    LessThan,
    LessEq,
    BelowEq,
    Minus,
    Plus,
    Above,
    Greater,
    GreaterEq,
    NotZero,
    NoOverflow,
    NoCarry,
    NoParity,
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

impl HBits for ConditionCode {
    fn from_bits(bits: u16) -> Self {
        match bits {
            0 => ConditionCode::Parity,
            1 => ConditionCode::Carry,
            2 => ConditionCode::Overflow,
            3 => ConditionCode::Zero,
            4 => ConditionCode::LessThan,
            5 => ConditionCode::LessEq,
            6 => ConditionCode::BelowEq,
            7 => ConditionCode::Minus,
            8 => ConditionCode::Plus,
            9 => ConditionCode::Above,
            10 => ConditionCode::Greater,
            11 => ConditionCode::GreaterEq,
            12 => ConditionCode::NotZero,
            13 => ConditionCode::NoOverflow,
            14 => ConditionCode::NoCarry,
            15 => ConditionCode::NoParity,
            _ => unreachable!(),
        }
    }

    fn to_hbits(self) -> u16 {
        self as u8 as u16
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
    // Undefined Instruction 0
    [Und0, "und", 0x000, 0],

    // Arithmetic Instructions
    [Add, "add", 0x001, 2, {lock @ 3 => bool, flags @ 0 => bool}],
    [Sub, "sub", 0x002, 2, {lock @ 3 => bool, flags @ 0 => bool}],
    [And, "and", 0x003, 2, {lock @ 3 => bool, flags @ 0 => bool}],
    [Or , "or" , 0x004, 2, {lock @ 3 => bool, flags @ 0 => bool}],
    [Xor, "xor", 0x005, 2, {lock @ 3 => bool, flags @ 0 => bool}],

    // Division and multiplication Instructions
    [Mul, "mul", 0x006, 0, {ss @ 2..4 => u16, flags @ 0 => bool}],
    [Div, "div", 0x007, 0, {ss @ 2..4 => u16, wide @ 1 => bool, flags @ 0 => bool}],

    // Register Manipulation Instructions
    [Mov, "mov", 0x008, 2],
    [Lea, "lea", 0x009, 2],
    [MovRD, "mov", 0x00A, 1, {r @ .. => CleverRegister}],
    [MovRS, "mov", 0x00B, 1, {r @ .. => CleverRegister}],
    [LeaRD, "lea", 0x00C, 1, {r @ .. => CleverRegister}],

    // Nops
    [Nop10, "nop", 0x010, 0],
    [Nop11, "nop", 0x011, 1],
    [Nop12, "nop", 0x012, 2],

    // Stack Manipulation
    [Push, "push", 0x014, 1],
    [Pop , "pop" , 0x015, 1],
    [PushR, "push", 0x016, 0, {r @ .. => CleverRegister}],
    [PopR , "pop" , 0x017, 0, {r @ .. => CleverRegister}],

    // Mass Register Storage
    [Stogpr , "stogpr" , 0x018, 1],
    [Stoar  , "stoar"  , 0x019, 1],
    [Rstogpr, "rstogpr", 0x01A, 1],
    [Rstoar , "rstoar" , 0x01B, 1],
    [Pushgpr, "pushgpr", 0x01C, 0],
    [Pushar , "pushar" , 0x01D, 0],
    [Popgpr , "popgpr" , 0x01E, 0],
    [Popar  , "popar"  , 0x01F, 0],

    // Converting Moves
    [Movsx, "movsx", 0x020, 2],
    [Bswap, "bswap", 0x021, 2],
    [Movsif, "movsif", 0x022, 2, {flags @ 0 => bool}],
    [Movxf, "movxf", 0x023, 2, {flags @0 => bool}],
    [Movfsi, "movfsi", 0x024, 2, {flags @ 0 => bool}],
    [Movfx, "movfx", 0x025, 2, {flags @ 0 => bool}],
    [Cvtf, "cvtf", 0x026, 2, {flags @ 0 => bool}],

    // Block Instructions
    [Repbi, "repbi", 0x028, 0],
    [Repbc, "repbc", 0x029, 0, {cc @ .. => ConditionCode}],
    [Bcpy, "bcpy", 0x02a, 0, {ss @ 0..2 => u16}],
    [Bsto, "bsto", 0x02b, 0, {ss @ 0..2 => u16}],
    [Bsca, "bsca", 0x02c, 0, {ss @ 0..2 => u16}],
    [Bcmp, "bcmp", 0x02d, 0, {ss @ 0..2 => u16}],
    [Btst, "btst", 0x02e, 0, {ss @ 0..2 => u16}],

    // Integer Shifts
    [Lsh, "lsh", 0x030, 2, {l @ 3 => bool, f @ 0 => bool}],
    [Rsh, "rsh", 0x031, 2, {l @ 3 => bool, f @ 0 => bool}],
    [Arsh, "arsh", 0x032, 2, {l @ 3 => bool, f @ 0 => bool}],
    [Lshc, "lshc", 0x033, 2, {l @ 3 => bool, f @ 0 => bool}],
    [Rshc, "rshc", 0x034, 2, {l @ 3 => bool, f @ 0 => bool}],
    [Lrot, "lrot", 0x035, 2, {l @ 3 => bool, f @ 0 => bool}],
    [Rrot, "rrot", 0x036, 2, {l @ 3 => bool, f @ 0 => bool}],
    [LshR, "lsh", 0x038, 2, {r @ 0..4 => CleverRegister}],
    [RshR, "rsh", 0x039, 2, {r @ 0..4 => CleverRegister}],
    [ArshR, "arsh", 0x03A, 2, {r @ 0..4 => CleverRegister}],
    [LshcR, "lshc", 0x03B, 2, {r @ 0..4 => CleverRegister}],
    [RshcR, "rshc", 0x03C, 2, {r @ 0..4 => CleverRegister}],
    [LrotR, "lrot", 0x03D, 2, {r @ 0..4 => CleverRegister}],
    [RrotR, "rrot", 0x03E, 2, {r @ 0..4 => CleverRegister}],

    // Arithmetic/Logic GPR Specifications
    // Unary Operations
    // Signed Multiplication/Division
    [Imul, "imul", 0x040, 0, {ss @ 2..4 => u16, flags @ 0 => bool}],
    [AddRD, "add", 0x041, 1, {r @ 0..4 => CleverRegister}],
    [SubRD, "sub", 0x042, 1, {r @ 0..4 => CleverRegister}],
    [AndRD, "and", 0x043, 1, {r @ 0..4 => CleverRegister}],
    [OrRD, "or", 0x044, 1, {r @ 0..4 => CleverRegister}],
    [XorRD, "xor", 0x045, 1, {r @ 0..4 => CleverRegister}],
    [BNot, "bnot", 0x046, 1, {l @ 3 => bool, f @ 0 => bool}],
    [Neg, "neg", 0x047, 1, {l @ 3 => bool, f @ 0 => bool}],
    [Idiv, "idiv", 0x048, 0, {ss @ 2..4 => u16, wide @ 1 => bool, flags @ 0 => bool}],
    [AddRS, "add", 0x049, 1, {r @ 0..4 => CleverRegister}],
    [SubRS, "sub", 0x04A, 1, {r @ 0..4 => CleverRegister}],
    [AndRS, "and", 0x04B, 1, {r @ 0..4 => CleverRegister}],
    [OrRS, "or", 0x04C, 1, {r @ 0..4 => CleverRegister}],
    [XorRS, "xor", 0x04D, 1, {r @ 0..4 => CleverRegister}],
    [BNotR, "bnot", 0x046, 1, {r @ 0..4 => CleverRegister}],
    [NegR, "neg", 0x047, 1, {r @ 0..4 => CleverRegister}],

    // Floating-Point Operations
    [Round, "round", 0x100, 1, {f @ 0 => bool}],
    [Ceil, "ceil", 0x101, 1, {f @ 0 => bool}],
    [Floor, "floor", 0x102, 1, {f @ 0 => bool}],
    [FAbs, "fabs", 0x103, 1, {f @ 0 => bool}],
    [FNeg, "fneg", 0x104, 1, {f @ 0 => bool}],
    [FInv, "finv",0x105, 1, {f @ 0 => bool}],
}
