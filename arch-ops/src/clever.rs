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
    cpuidlo => 136,
    cpuidhi => 137,
    cpuex2 => 138,
    cpuex3 => 139,
    cpuex4 => 140,
    cpuex5 => 141,
    cpuex6 => 142,
    mscpuex => 143,
    msr0 => 148,
    msr1 => 149,
    msr2 => 150,
    msr3 => 151,
    msr4 => 152,
    msr5 => 153,
    msr6 => 154
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

impl HBits for i8 {
    fn from_bits(bits: u16) -> Self {
        (bits as i8) | ((bits & 0x8).wrapping_neg() as i8)
    }
    fn to_hbits(self) -> u16 {
        (self & 0xf) as u16
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

pub enum CleverOperandKind {
    Normal(u32),
    AbsAddr,
    RelAddr,
    // Prefix
    Insn,
}

macro_rules! clever_instructions{
    {
        $([$enum:ident, $insn:literal, $opcode:literal, $operands:expr $(, { $($hfield:ident @ $range:expr => $ty:ty ),* $(,)?})? ]),* $(,)?
    } => {

        #[derive(Copy,Clone,Debug,Hash,PartialEq, Eq)]
        pub enum CleverOpcode{
            $($enum $({$($hfield: $ty),*})?),*
        }

        impl CleverOpcode{

            pub fn from_opcode(opc: u16) -> Option<CleverOpcode>{
                match opc>>4{
                    $(#[allow(unreachable_patterns)] $opcode => {
                        Some(Self:: $enum $({$($hfield: HBits::from_bits(
                            (opc>>HBitRange::shift(&$range))&HBitRange::mask(&$range)
                        )),*})?)
                    },)*
                    _ => None
                }
            }

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

            pub fn operands(&self) -> CleverOperandKind{
                match self{
                    $(Self:: $enum {..} => $operands),*
                }
            }
        }
    }
}

clever_instructions! {
    // Undefined Instruction 0
    [Und0, "und", 0x000, CleverOperandKind::Normal(0)],

    // Arithmetic Instructions
    [Add, "add", 0x001, CleverOperandKind::Normal(2), {lock @ 3 => bool, flags @ 0 => bool}],
    [Sub, "sub", 0x002, CleverOperandKind::Normal(2), {lock @ 3 => bool, flags @ 0 => bool}],
    [And, "and", 0x003, CleverOperandKind::Normal(2), {lock @ 3 => bool, flags @ 0 => bool}],
    [Or , "or" , 0x004, CleverOperandKind::Normal(2), {lock @ 3 => bool, flags @ 0 => bool}],
    [Xor, "xor", 0x005, CleverOperandKind::Normal(2), {lock @ 3 => bool, flags @ 0 => bool}],

    // Division and multiplication Instructions
    [Mul, "mul", 0x006, CleverOperandKind::Normal(0), {ss @ 2..4 => u16, flags @ 0 => bool}],
    [Div, "div", 0x007, CleverOperandKind::Normal(0), {ss @ 2..4 => u16, wide @ 1 => bool, flags @ 0 => bool}],

    // Register Manipulation Instructions
    [Mov, "mov", 0x008, CleverOperandKind::Normal(2)],
    [Lea, "lea", 0x009, CleverOperandKind::Normal(2)],
    [MovRD, "mov", 0x00A, CleverOperandKind::Normal(1), {r @ .. => CleverRegister}],
    [MovRS, "mov", 0x00B, CleverOperandKind::Normal(1), {r @ .. => CleverRegister}],
    [LeaRD, "lea", 0x00C, CleverOperandKind::Normal(1), {r @ .. => CleverRegister}],

    // Nops
    [Nop10, "nop", 0x010, CleverOperandKind::Normal(0)],
    [Nop11, "nop", 0x011, CleverOperandKind::Normal(1)],
    [Nop12, "nop", 0x012, CleverOperandKind::Normal(2)],

    // Stack Manipulation
    [Push, "push", 0x014, CleverOperandKind::Normal(1)],
    [Pop , "pop" , 0x015, CleverOperandKind::Normal(1)],
    [PushR, "push", 0x016, CleverOperandKind::Normal(0), {r @ .. => CleverRegister}],
    [PopR , "pop" , 0x017, CleverOperandKind::Normal(0), {r @ .. => CleverRegister}],

    // Mass Register Storage
    [Stogpr , "stogpr" , 0x018, CleverOperandKind::Normal(1)],
    [Stoar  , "stoar"  , 0x019, CleverOperandKind::Normal(1)],
    [Rstogpr, "rstogpr", 0x01A, CleverOperandKind::Normal(1)],
    [Rstoar , "rstoar" , 0x01B, CleverOperandKind::Normal(1)],
    [Pushgpr, "pushgpr", 0x01C, CleverOperandKind::Normal(0)],
    [Pushar , "pushar" , 0x01D, CleverOperandKind::Normal(0)],
    [Popgpr , "popgpr" , 0x01E, CleverOperandKind::Normal(0)],
    [Popar  , "popar"  , 0x01F, CleverOperandKind::Normal(0)],

    // Converting Moves
    [Movsx, "movsx", 0x020, CleverOperandKind::Normal(2)],
    [Bswap, "bswap", 0x021, CleverOperandKind::Normal(2)],
    [Movsif, "movsif", 0x022, CleverOperandKind::Normal(2), {flags @ 0 => bool}],
    [Movxf, "movxf", 0x023, CleverOperandKind::Normal(2), {flags @0 => bool}],
    [Movfsi, "movfsi", 0x024, CleverOperandKind::Normal(2), {flags @ 0 => bool}],
    [Movfx, "movfx", 0x025, CleverOperandKind::Normal(2), {flags @ 0 => bool}],
    [Cvtf, "cvtf", 0x026, CleverOperandKind::Normal(2), {flags @ 0 => bool}],

    // Block Instructions
    [Bcpy, "bcpy", 0x02a, CleverOperandKind::Normal(0), {ss @ 0..2 => u16}],
    [Bsto, "bsto", 0x02b, CleverOperandKind::Normal(0), {ss @ 0..2 => u16}],
    [Bsca, "bsca", 0x02c, CleverOperandKind::Normal(0), {ss @ 0..2 => u16}],
    [Bcmp, "bcmp", 0x02d, CleverOperandKind::Normal(0), {ss @ 0..2 => u16}],
    [Btst, "btst", 0x02e, CleverOperandKind::Normal(0), {ss @ 0..2 => u16}],

    // Integer Shifts
    [Lsh, "lsh", 0x030, CleverOperandKind::Normal(2), {l @ 3 => bool, f @ 0 => bool}],
    [Rsh, "rsh", 0x031, CleverOperandKind::Normal(2), {l @ 3 => bool, f @ 0 => bool}],
    [Arsh, "arsh", 0x032, CleverOperandKind::Normal(2), {l @ 3 => bool, f @ 0 => bool}],
    [Lshc, "lshc", 0x033, CleverOperandKind::Normal(2), {l @ 3 => bool, f @ 0 => bool}],
    [Rshc, "rshc", 0x034, CleverOperandKind::Normal(2), {l @ 3 => bool, f @ 0 => bool}],
    [Lrot, "lrot", 0x035, CleverOperandKind::Normal(2), {l @ 3 => bool, f @ 0 => bool}],
    [Rrot, "rrot", 0x036, CleverOperandKind::Normal(2), {l @ 3 => bool, f @ 0 => bool}],
    [LshR, "lsh", 0x038, CleverOperandKind::Normal(2), {r @ 0..4 => CleverRegister}],
    [RshR, "rsh", 0x039, CleverOperandKind::Normal(2), {r @ 0..4 => CleverRegister}],
    [ArshR, "arsh", 0x03A, CleverOperandKind::Normal(2), {r @ 0..4 => CleverRegister}],
    [LshcR, "lshc", 0x03B, CleverOperandKind::Normal(2), {r @ 0..4 => CleverRegister}],
    [RshcR, "rshc", 0x03C, CleverOperandKind::Normal(2), {r @ 0..4 => CleverRegister}],
    [LrotR, "lrot", 0x03D, CleverOperandKind::Normal(2), {r @ 0..4 => CleverRegister}],
    [RrotR, "rrot", 0x03E, CleverOperandKind::Normal(2), {r @ 0..4 => CleverRegister}],

    // Arithmetic/Logic GPR Specifications
    // Unary Operations
    // Signed Multiplication/Division
    [Imul, "imul", 0x040, CleverOperandKind::Normal(0), {ss @ 2..4 => u16, flags @ 0 => bool}],
    [AddRD, "add", 0x041, CleverOperandKind::Normal(1), {r @ 0..4 => CleverRegister}],
    [SubRD, "sub", 0x042, CleverOperandKind::Normal(1), {r @ 0..4 => CleverRegister}],
    [AndRD, "and", 0x043, CleverOperandKind::Normal(1), {r @ 0..4 => CleverRegister}],
    [OrRD, "or", 0x044, CleverOperandKind::Normal(1), {r @ 0..4 => CleverRegister}],
    [XorRD, "xor", 0x045, CleverOperandKind::Normal(1), {r @ 0..4 => CleverRegister}],
    [BNot, "bnot", 0x046, CleverOperandKind::Normal(1), {l @ 3 => bool, f @ 0 => bool}],
    [Neg, "neg", 0x047, CleverOperandKind::Normal(1), {l @ 3 => bool, f @ 0 => bool}],
    [Idiv, "idiv", 0x048, CleverOperandKind::Normal(1), {ss @ 2..4 => u16, wide @ 1 => bool, flags @ 0 => bool}],
    [AddRS, "add", 0x049, CleverOperandKind::Normal(1), {r @ 0..4 => CleverRegister}],
    [SubRS, "sub", 0x04A, CleverOperandKind::Normal(1), {r @ 0..4 => CleverRegister}],
    [AndRS, "and", 0x04B, CleverOperandKind::Normal(1), {r @ 0..4 => CleverRegister}],
    [OrRS, "or", 0x04C, CleverOperandKind::Normal(1), {r @ 0..4 => CleverRegister}],
    [XorRS, "xor", 0x04D, CleverOperandKind::Normal(1), {r @ 0..4 => CleverRegister}],
    [BNotR, "bnot", 0x046, CleverOperandKind::Normal(1), {r @ 0..4 => CleverRegister}],
    [NegR, "neg", 0x047, CleverOperandKind::Normal(1), {r @ 0..4 => CleverRegister}],

    // Comparison operations
    [Cmp, "cmp", 0x06C, CleverOperandKind::Normal(2), {}],
    [Test, "test", 0x06D, CleverOperandKind::Normal(2), {}],
    [CmpR, "cmp", 0x06C, CleverOperandKind::Normal(1), {r @ 0..4 => CleverRegister}],
    [TestR, "test", 0x06D, CleverOperandKind::Normal(1), {r @ 0..4 => CleverRegister}],

    // Floating-Point Operations
    [Round, "round", 0x100, CleverOperandKind::Normal(1), {f @ 0 => bool}],
    [Ceil, "ceil", 0x101, CleverOperandKind::Normal(1), {f @ 0 => bool}],
    [Floor, "floor", 0x102, CleverOperandKind::Normal(1), {f @ 0 => bool}],
    [FAbs, "fabs", 0x103, CleverOperandKind::Normal(1), {f @ 0 => bool}],
    [FNeg, "fneg", 0x104, CleverOperandKind::Normal(1), {f @ 0 => bool}],
    [FInv, "finv",0x105, CleverOperandKind::Normal(1), {f @ 0 => bool}],
    [FAdd, "fadd", 0x106, CleverOperandKind::Normal(2), {f @ 0 => bool}],
    [FSub, "fsub", 0x107, CleverOperandKind::Normal(2), {f @ 0 => bool}],
    [FMul, "fmul", 0x108, CleverOperandKind::Normal(2), {f @ 0 => bool}],
    [FDiv, "fdiv", 0x109, CleverOperandKind::Normal(2), {f @ 0 => bool}],
    [FRem, "frem", 0x10A, CleverOperandKind::Normal(2), {f @ 0 => bool}],
    [FFma, "ffma", 0x10B, CleverOperandKind::Normal(3), {f @ 0 => bool}],

    // Floating-point comparions
    [FCmpz, "fcmpz", 0x118, CleverOperandKind::Normal(1), {}],
    [FCmp, "fcmp", 0x119, CleverOperandKind::Normal(2), {}],

    // Floating-point extra instructions
    [Exp, "exp", 0x120, CleverOperandKind::Normal(1), {}],
    [Ln, "ln", 0x121, CleverOperandKind::Normal(1), {}],
    [Lg, "lg", 0x122, CleverOperandKind::Normal(1), {}],
    [Sin, "sin", 0x123, CleverOperandKind::Normal(1), {}],
    [Cos, "cos", 0x124, CleverOperandKind::Normal(1), {}],
    [Tan, "tan", 0x125, CleverOperandKind::Normal(1), {}],
    [Asin, "asin", 0x126, CleverOperandKind::Normal(1), {}],
    [Acos, "acos", 0x127, CleverOperandKind::Normal(1), {}],
    [Atan, "atan", 0x128, CleverOperandKind::Normal(1), {}],
    [Exp2,"exp2", 0x129, CleverOperandKind::Normal(1),{}],
    [Log10, "log10", 0x12A, CleverOperandKind::Normal(1), {}],
    [Lnp1, "lnp1", 0x12B, CleverOperandKind::Normal(1), {}],
    [Expm1, "expm1", 0x12C, CleverOperandKind::Normal(1), {}],
    [Sqrt, "sqrt", 0x12D, CleverOperandKind::Normal(1), {}],

    // Floating-point exception control
    [FRaiseExcept, "fraiseexcept", 0x130, CleverOperandKind::Normal(0), {}],
    [FTriggerExcept, "ftriggerexcept", 0x130, CleverOperandKind::Normal(0), {}],

    // Atomic Operations
    [Xchg, "xchg", 0x200, CleverOperandKind::Normal(2), {}],
    [Cmpxchg, "cmpxchg", 0x201, CleverOperandKind::Normal(3), {}],
    [Wcmpxchg, "wcmpxchg", 0x202, CleverOperandKind::Normal(3), {}],
    [Fence, "fence", 0x203, CleverOperandKind::Normal(0), {}],

    // Vector Instructions
    [Vmov, "vmov",0x401, CleverOperandKind::Normal(2), {}],

    // conditional Branches
    [CBP0A , "jp" , 0x700, CleverOperandKind::AbsAddr, {w @ 0..4 => i8}],
    [CBC0A , "jc" , 0x701, CleverOperandKind::AbsAddr, {w @ 0..4 => i8}],
    [CBV0A , "jo" , 0x702, CleverOperandKind::AbsAddr, {w @ 0..4 => i8}],
    [CBZ0A , "jz" , 0x703, CleverOperandKind::AbsAddr, {w @ 0..4 => i8}],
    [CBL0A , "jlt", 0x704, CleverOperandKind::AbsAddr, {w @ 0..4 => i8}],
    [CBLE0A, "jle", 0x705, CleverOperandKind::AbsAddr, {w @ 0..4 => i8}],
    [CBBE0A, "jbe", 0x706, CleverOperandKind::AbsAddr, {w @ 0..4 => i8}],
    [CBM0A , "jmi", 0x707, CleverOperandKind::AbsAddr, {w @ 0..4 => i8}],
    [CBPS0A, "jps", 0x708, CleverOperandKind::AbsAddr, {w @ 0..4 => i8}],
    [CBA0A , "ja" , 0x709, CleverOperandKind::AbsAddr, {w @ 0..4 => i8}],
    [CBG0A , "jgt", 0x70A, CleverOperandKind::AbsAddr, {w @ 0..4 => i8}],
    [CBGE0A, "jge", 0x70B, CleverOperandKind::AbsAddr, {w @ 0..4 => i8}],
    [CBNZ0A, "jnz", 0x70C, CleverOperandKind::AbsAddr, {w @ 0..4 => i8}],
    [CBNV0A, "jno", 0x70D, CleverOperandKind::AbsAddr, {w @ 0..4 => i8}],
    [CBNC0A, "jnc", 0x70E, CleverOperandKind::AbsAddr, {w @ 0..4 => i8}],
    [CBNP0A, "jnp", 0x70F, CleverOperandKind::AbsAddr, {w @ 0..4 => i8}],

    [CBP0R , "jp" , 0x710, CleverOperandKind::RelAddr, {w @ 0..4 => i8}],
    [CBC0R , "jc" , 0x711, CleverOperandKind::RelAddr, {w @ 0..4 => i8}],
    [CBV0R , "jo" , 0x712, CleverOperandKind::RelAddr, {w @ 0..4 => i8}],
    [CBZ0R , "jz" , 0x713, CleverOperandKind::RelAddr, {w @ 0..4 => i8}],
    [CBL0R , "jlt", 0x714, CleverOperandKind::RelAddr, {w @ 0..4 => i8}],
    [CBLE0R, "jle", 0x715, CleverOperandKind::RelAddr, {w @ 0..4 => i8}],
    [CBBE0R, "jbe", 0x716, CleverOperandKind::RelAddr, {w @ 0..4 => i8}],
    [CBM0R , "jmi", 0x717, CleverOperandKind::RelAddr, {w @ 0..4 => i8}],
    [CBPS0R, "jps", 0x718, CleverOperandKind::RelAddr, {w @ 0..4 => i8}],
    [CBA0R , "ja" , 0x719, CleverOperandKind::RelAddr, {w @ 0..4 => i8}],
    [CBG0R , "jgt", 0x71A, CleverOperandKind::RelAddr, {w @ 0..4 => i8}],
    [CBGE0R, "jge", 0x71B, CleverOperandKind::RelAddr, {w @ 0..4 => i8}],
    [CBNZ0R, "jnz", 0x71C, CleverOperandKind::RelAddr, {w @ 0..4 => i8}],
    [CBNV0R, "jno", 0x71D, CleverOperandKind::RelAddr, {w @ 0..4 => i8}],
    [CBNC0R, "jnc", 0x71E, CleverOperandKind::RelAddr, {w @ 0..4 => i8}],
    [CBNP0R, "jnp", 0x71F, CleverOperandKind::RelAddr, {w @ 0..4 => i8}],

    [CBP1A , "jp" , 0x740, CleverOperandKind::AbsAddr, {w @ 0..4 => i8}],
    [CBC1A , "jc" , 0x741, CleverOperandKind::AbsAddr, {w @ 0..4 => i8}],
    [CBV1A , "jo" , 0x742, CleverOperandKind::AbsAddr, {w @ 0..4 => i8}],
    [CBZ1A , "jz" , 0x743, CleverOperandKind::AbsAddr, {w @ 0..4 => i8}],
    [CBL1A , "jlt", 0x744, CleverOperandKind::AbsAddr, {w @ 0..4 => i8}],
    [CBLE1A, "jle", 0x745, CleverOperandKind::AbsAddr, {w @ 0..4 => i8}],
    [CBBE1A, "jbe", 0x746, CleverOperandKind::AbsAddr, {w @ 0..4 => i8}],
    [CBM1A , "jmi", 0x747, CleverOperandKind::AbsAddr, {w @ 0..4 => i8}],
    [CBPS1A, "jps", 0x748, CleverOperandKind::AbsAddr, {w @ 0..4 => i8}],
    [CBA1A , "ja" , 0x749, CleverOperandKind::AbsAddr, {w @ 0..4 => i8}],
    [CBG1A , "jgt", 0x74A, CleverOperandKind::AbsAddr, {w @ 0..4 => i8}],
    [CBGE1A, "jge", 0x74B, CleverOperandKind::AbsAddr, {w @ 0..4 => i8}],
    [CBNZ1A, "jnz", 0x74C, CleverOperandKind::AbsAddr, {w @ 0..4 => i8}],
    [CBNV1A, "jno", 0x74D, CleverOperandKind::AbsAddr, {w @ 0..4 => i8}],
    [CBNC1A, "jnc", 0x74E, CleverOperandKind::AbsAddr, {w @ 0..4 => i8}],
    [CBNP1A, "jnp", 0x74F, CleverOperandKind::AbsAddr, {w @ 0..4 => i8}],

    [CBP1R , "jp" , 0x750, CleverOperandKind::RelAddr, {w @ 0..4 => i8}],
    [CBC1R , "jc" , 0x751, CleverOperandKind::RelAddr, {w @ 0..4 => i8}],
    [CBV1R , "jo" , 0x752, CleverOperandKind::RelAddr, {w @ 0..4 => i8}],
    [CBZ1R , "jz" , 0x753, CleverOperandKind::RelAddr, {w @ 0..4 => i8}],
    [CBL1R , "jlt", 0x754, CleverOperandKind::RelAddr, {w @ 0..4 => i8}],
    [CBLE1R, "jle", 0x755, CleverOperandKind::RelAddr, {w @ 0..4 => i8}],
    [CBBE1R, "jbe", 0x756, CleverOperandKind::RelAddr, {w @ 0..4 => i8}],
    [CBM1R , "jmi", 0x757, CleverOperandKind::RelAddr, {w @ 0..4 => i8}],
    [CBPS1R, "jps", 0x758, CleverOperandKind::RelAddr, {w @ 0..4 => i8}],
    [CBA1R , "ja" , 0x759, CleverOperandKind::RelAddr, {w @ 0..4 => i8}],
    [CBG1R , "jgt", 0x75A, CleverOperandKind::RelAddr, {w @ 0..4 => i8}],
    [CBGE1R, "jge", 0x75B, CleverOperandKind::RelAddr, {w @ 0..4 => i8}],
    [CBNZ1R, "jnz", 0x75C, CleverOperandKind::RelAddr, {w @ 0..4 => i8}],
    [CBNV1R, "jno", 0x75D, CleverOperandKind::RelAddr, {w @ 0..4 => i8}],
    [CBNC1R, "jnc", 0x75E, CleverOperandKind::RelAddr, {w @ 0..4 => i8}],
    [CBNP1R, "jnp", 0x75F, CleverOperandKind::RelAddr, {w @ 0..4 => i8}],

    [CBP2A , "jp" , 0x780, CleverOperandKind::AbsAddr, {w @ 0..4 => i8}],
    [CBC2A , "jc" , 0x781, CleverOperandKind::AbsAddr, {w @ 0..4 => i8}],
    [CBV2A , "jo" , 0x782, CleverOperandKind::AbsAddr, {w @ 0..4 => i8}],
    [CBZ2A , "jz" , 0x783, CleverOperandKind::AbsAddr, {w @ 0..4 => i8}],
    [CBL2A , "jlt", 0x784, CleverOperandKind::AbsAddr, {w @ 0..4 => i8}],
    [CBLE2A, "jle", 0x785, CleverOperandKind::AbsAddr, {w @ 0..4 => i8}],
    [CBBE2A, "jbe", 0x786, CleverOperandKind::AbsAddr, {w @ 0..4 => i8}],
    [CBM2A , "jmi", 0x787, CleverOperandKind::AbsAddr, {w @ 0..4 => i8}],
    [CBPS2A, "jps", 0x788, CleverOperandKind::AbsAddr, {w @ 0..4 => i8}],
    [CBA2A , "ja" , 0x789, CleverOperandKind::AbsAddr, {w @ 0..4 => i8}],
    [CBG2A , "jgt", 0x78A, CleverOperandKind::AbsAddr, {w @ 0..4 => i8}],
    [CBGE2A, "jge", 0x78B, CleverOperandKind::AbsAddr, {w @ 0..4 => i8}],
    [CBNZ2A, "jnz", 0x78C, CleverOperandKind::AbsAddr, {w @ 0..4 => i8}],
    [CBNV2A, "jno", 0x78D, CleverOperandKind::AbsAddr, {w @ 0..4 => i8}],
    [CBNC2A, "jnc", 0x78E, CleverOperandKind::AbsAddr, {w @ 0..4 => i8}],
    [CBNP2A, "jnp", 0x78F, CleverOperandKind::AbsAddr, {w @ 0..4 => i8}],

    [CBP2R , "jp" , 0x790, CleverOperandKind::RelAddr, {w @ 0..4 => i8}],
    [CBC2R , "jc" , 0x791, CleverOperandKind::RelAddr, {w @ 0..4 => i8}],
    [CBV2R , "jo" , 0x792, CleverOperandKind::RelAddr, {w @ 0..4 => i8}],
    [CBZ2R , "jz" , 0x793, CleverOperandKind::RelAddr, {w @ 0..4 => i8}],
    [CBL2R , "jlt", 0x794, CleverOperandKind::RelAddr, {w @ 0..4 => i8}],
    [CBLE2R, "jle", 0x795, CleverOperandKind::RelAddr, {w @ 0..4 => i8}],
    [CBBE2R, "jbe", 0x796, CleverOperandKind::RelAddr, {w @ 0..4 => i8}],
    [CBM2R , "jmi", 0x797, CleverOperandKind::RelAddr, {w @ 0..4 => i8}],
    [CBPS2R, "jps", 0x798, CleverOperandKind::RelAddr, {w @ 0..4 => i8}],
    [CBA2R , "ja" , 0x799, CleverOperandKind::RelAddr, {w @ 0..4 => i8}],
    [CBG2R , "jgt", 0x79A, CleverOperandKind::RelAddr, {w @ 0..4 => i8}],
    [CBGE2R, "jge", 0x79B, CleverOperandKind::RelAddr, {w @ 0..4 => i8}],
    [CBNZ2R, "jnz", 0x79C, CleverOperandKind::RelAddr, {w @ 0..4 => i8}],
    [CBNV2R, "jno", 0x79D, CleverOperandKind::RelAddr, {w @ 0..4 => i8}],
    [CBNC2R, "jnc", 0x79E, CleverOperandKind::RelAddr, {w @ 0..4 => i8}],
    [CBNP2R, "jnp", 0x79F, CleverOperandKind::RelAddr, {w @ 0..4 => i8}],

    // Unconditional Branches/Calls
    [JmpA, "jmp", 0x7C0, CleverOperandKind::AbsAddr, {ss @ 0..2 => u16}],
    [CallA, "call", 0x7C1, CleverOperandKind::AbsAddr, {ss @ 0..2 => u16}],
    [FcallA, "fcall", 0x7C2, CleverOperandKind::AbsAddr, {ss @ 0..2 => u16}],
    [Ret, "ret", 0x7C3, CleverOperandKind::Normal(0), {}],
    [Scall, "scall", 0x7C4, CleverOperandKind::Normal(0), {}],
    [Int, "int", 0x7C5, CleverOperandKind::Normal(0), {i @ 0..4 => u16}],
    [IjmpA, "ijmp", 0x7C8, CleverOperandKind::Normal(0), {r @ 0..4 => CleverRegister}],
    [IcallA, "icall", 0x7C9, CleverOperandKind::Normal(0), {r @ 0..4 => CleverRegister}],
    [IfcallA, "ifcall", 0x7CA, CleverOperandKind::Normal(0), {}],
    [JmpR, "jmp", 0x7D0, CleverOperandKind::AbsAddr, {ss @ 0..2 => u16}],
    [CallR, "call", 0x7D1, CleverOperandKind::AbsAddr, {ss @ 0..2 => u16}],
    [FcallR, "fcall", 0x7D2, CleverOperandKind::AbsAddr, {ss @ 0..2 => u16}],
    [IjmpR, "ijmp", 0x7D8, CleverOperandKind::Normal(0), {r @ 0..4 => CleverRegister}],
    [IcallR, "icall", 0x7D9, CleverOperandKind::Normal(0), {r @ 0..4 => CleverRegister}],
    [IfcallR, "ifcall", 0x7DA, CleverOperandKind::Normal(0), {}],

    // Halt
    [Halt, "halt", 0x801, CleverOperandKind::Normal(0), {}],

    // Cache Control
    [Pcfl, "pcfl", 0x802, CleverOperandKind::Normal(0), {}],
    [FlAll, "flall", 0x803, CleverOperandKind::Normal(0), {}],
    [Dflush, "dflush", 0x804, CleverOperandKind::Normal(1), {}],
    [Iflush, "iflush", 0x805, CleverOperandKind::Normal(1), {}],

    // I/O Transfers
    [In, "in", 0x806, CleverOperandKind::Normal(0), {ss @ 0..2 => u16}],
    [Out, "out", 0x807, CleverOperandKind::Normal(0), {ss @ 0..2 => u16}],

    // Mass Register Storage
    [StoRegF, "storegf", 0x808, CleverOperandKind::Normal(1), {}],
    [RstRegF, "rstregf", 0x809, CleverOperandKind::Normal(1), {}],

    // Supervisor Branches
    [Scret, "scret", 0xFC6, CleverOperandKind::Normal(0), {}],
    [Iret, "iret", 0xFC6, CleverOperandKind::Normal(0), {}],
    [Hcall, "hcall", 0xFCB, CleverOperandKind::Normal(0), {}],
    [Hret, "hret", 0xFD6, CleverOperandKind::Normal(0), {}],
    [Hresume, "hresume", 0xFD7, CleverOperandKind::Normal(0), {}],

    // VM Creation/Disposal
    [VMCreate, "vmcreate", 0xFDA, CleverOperandKind::Normal(1), {}],
    [VMDestroy,"vmdestroy",0xFDB, CleverOperandKind::Normal(0), {}]
}

impl CleverOpcode {
    pub fn is_branch(&self) -> bool {
        (self.opcode() < 0xFD80) && (self.opcode() & 0x7200 == 0x7000)
    }

    pub fn is_cbranch(&self) -> bool {
        matches!(self.opcode() & 0xFE00, 0x7000 | 0x7400 | 0x7800)
    }

    pub fn branch_pcrel(&self) -> Option<bool> {
        if self.is_branch() {
            Some((self.opcode() & 0x100) != 0)
        } else {
            None
        }
    }

    pub fn branch_condition(&self) -> Option<ConditionCode> {
        match self.opcode() & 0xFE00 {
            0x7000 | 0x7400 | 0x7800 => Some(ConditionCode::from_bits((self.opcode() & 0xf0) >> 4)),
            _ => None,
        }
    }

    pub fn branch_weight(&self) -> Option<i8> {
        match self.opcode() & 0xFE00 {
            0x7000 | 0x7400 | 0x7800 => Some(i8::from_bits(self.opcode() & 0xf)),
            _ => None,
        }
    }

    pub fn branch_width(&self) -> Option<u16> {
        match self.opcode() & 0xFE00 {
            0x7000 | 0x7400 | 0x7800 => Some(((self.opcode() & 0xC00) >> 10) + 1),
            _ => None,
        }
    }

    pub fn cbranch(cc: ConditionCode, width: u16, pcrel: bool, weight: i8) -> Self {
        assert!(width - 1 < 3);
        let (width, pcrel, cc, weight) = (
            (width - 1) << 10,
            (pcrel as u16) << 8,
            cc.to_hbits() << 4,
            (weight & 0xf) as u16,
        );

        Self::from_opcode(0x7000 | width | pcrel | cc | weight).unwrap()
    }
}
pub enum CleverOperand {
    Register {
        size: u16,
        reg: CleverRegister,
    },
    Indirect {
        size: u16,
        base: CleverRegister,
        index: CleverIndex,
    },
    AbsImmediate {
        size: u32,
        val: u64,
    },
    VecImmediate {
        val: u128,
    },
    RelImmediate {
        size: u32,
        val: i64,
    },
}
