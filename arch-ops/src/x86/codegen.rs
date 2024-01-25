use crate::traits::{Address, InsnWrite};
use std::{
    io::{self, Write},
    marker::PhantomData,
    mem::MaybeUninit,
};

use super::{X86Mode, X86Register, X86RegisterClass};

macro_rules! options{
    {
        $(#[$meta:meta])*
        $vis:vis struct $options_ty:ident $(:$(#[$meta3:meta])* $base_ty:ident)?{
            $($(#[$meta2:meta])* $vis2:vis $field:ident),* $(,)?
        }
    } => {
        $(#[$meta])*
        #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Default)]
        $vis struct $options_ty{
            #[doc(hidden)]
            $vis __nonexhausive: (),
            $($(#[$meta3])* #[doc(hidden)] $vis __base: $base_ty,)?
            $($(#[$meta2])* $vis2 $field: bool),*
        }

        impl $options_ty{
            pub const NONE: Self = Self::new();

            paste::paste!{
                $($vis2 const [<$field:upper>]: Self = Self{$field: true, ..Self::new()};)*
            }

            $(pub const fn base(x: $base_ty) -> Self{
                Self{__nonexhausive: (), __base: x, ..Self::new()}
            })?

            pub const fn new() -> Self{
                Self{__nonexhausive: (), $(__base: $base_ty :: NONE,)? $($field: false),*}
            }
        }

        impl core::ops::BitOr for $options_ty{
            type Output = Self;
            fn bitor(mut self, rhs: Self) -> Self{
                self |= rhs;
                self
            }
        }

        impl core::ops::BitOrAssign for $options_ty{
            fn bitor_assign(&mut self, rhs: Self){
                $(self. $field |= rhs.$field;)*
            }
        }
        $(
            impl<B> core::ops::BitOr<B> for $options_ty where $base_ty: core::ops::BitOrAssign<B>{
                type Output = Self;
                fn bitor(mut self, rhs: B) -> Self{
                    self |= rhs;
                    self
                }
            }

            impl core::ops::BitOr<$options_ty> for $base_ty{
                type Output = $options_ty;
                fn bitor(self, mut rhs: $options_ty) -> $options_ty{
                    rhs.__base |= self;

                    rhs
                }
            }

            impl<B> core::ops::BitOrAssign<B> for $options_ty where $base_ty: core::ops::BitOrAssign<B>{
                fn bitor_assign(&mut self, rhs: B){
                    self.__base |= rhs
                }
            }
        )?
    }
}

options! {
    pub struct ModRMOptions{
        pub no_rex_w,
        pub ignore_size_mismatch,
        pub no_escape,
    }
}

options! {
    pub struct AvxModRMOptions : ModRMOptions{
        pub vex_extended_vex_only,
    }
}

options! {
    pub struct OpRegOptions{
        pub no_rex_w,
    }
}

options! {
    pub struct LegacyEvexEncodingOptions{
        pub no_rex_w,
        pub ndd,
        pub allow_nf,
    }
}

options! {
    pub struct EvexModRMOptions: ModRMOptions{
        pub allow_size_256,
        pub allow_size_512,
        pub use_mask,
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum X86PrefixGroup {
    LegacyControl,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
#[non_exhaustive]
pub enum X86EncodingMode {
    Prefix(X86PrefixGroup),
    OpcodeOnly,
    ModRM(ModRMOptions),
    ModRMControl(u8, ModRMOptions),
    ModRMOp2(u8),
    OpReg(OpRegOptions),
    AvxModRM(AvxModRMOptions),
    EvexModRM(EvexModRMOptions),
    LegacyEvexModRM(LegacyEvexEncodingOptions),
    OpcodeWithSize(X86RegisterClass),
    OffsetImm(usize),
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
#[non_exhaustive]
pub enum X86InstructionMap {
    /// No escape prefix
    Legacy,
    /// Escape prefix 0F
    Extended,
    /// Escape prefix 0F 38
    Extended2,
    /// Escape prefix 0F 3A
    Extended3,
}

impl X86InstructionMap {
    pub const fn from_opcode(opc: u32) -> Option<X86InstructionMap> {
        match (opc & 0xFFFFFF) >> 8 {
            0 => Some(Self::Legacy),
            0x0F => Some(Self::Extended),
            0x0F38 => Some(Self::Extended2),
            0x0F3A => Some(Self::Extended3),
            _ => None,
        }
    }

    pub const fn from_vex_map(map: u8) -> Option<X86InstructionMap> {
        match map {
            4 => Some(Self::Legacy),
            1 => Some(Self::Extended),
            2 => Some(Self::Extended2),
            3 => Some(Self::Extended3),
            _ => None,
        }
    }

    pub const fn escape_prefix_bytes(&self) -> &'static [u8] {
        match self {
            Self::Legacy => &[],
            Self::Extended => &[0x0F],
            Self::Extended2 => &[0x0F, 0x38],
            Self::Extended3 => &[0x0F, 0x3A],
        }
    }

    pub const fn vex_map_value(&self) -> u8 {
        match self {
            Self::Legacy => 4,
            Self::Extended => 1,
            Self::Extended2 => 2,
            Self::Extended3 => 3,
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
#[non_exhaustive]
#[allow(clippy::manual_non_exhaustive)] // See above
pub enum X86EncodingPrefix {
    NoPrefix,
    Rex,
    Rex2,
    Vex,
    Evex,

    #[doc(hidden)]
    __PrefixCount,
}

impl X86EncodingPrefix {
    pub const PREFIX_COUNT: usize = Self::__PrefixCount as usize;
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum SsePrefix {
    PackedDouble,
    ScalarDouble,
    ScalarSingle,
}

impl SsePrefix {
    pub const fn from_opcode(opc: u32) -> Option<Self> {
        match opc >> 24 {
            0x66 => Some(SsePrefix::PackedDouble),
            0xF2 => Some(SsePrefix::ScalarDouble),
            0xF3 => Some(SsePrefix::ScalarSingle),
            _ => None,
        }
    }

    pub const fn encoding(&self) -> &'static [u8] {
        match self {
            Self::PackedDouble => &[0x66],
            Self::ScalarDouble => &[0xF2],
            Self::ScalarSingle => &[0xF3],
        }
    }

    pub const fn vex_prefix_value(&self) -> u8 {
        match self {
            Self::PackedDouble => 1,
            Self::ScalarSingle => 2,
            Self::ScalarDouble => 3,
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct X86Encoding {
    pub map: X86InstructionMap,
    pub base_opcode: u8,
    pub mode: X86EncodingMode,
    pub allowed_modes: Option<&'static [X86Mode]>,
    pub sse_prefix: Option<SsePrefix>,
    pub imm_size: usize,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
#[repr(u8)]
pub enum Segment {
    Es = 0x26,
    Cs = 0x2E,
    Ss = 0x36,
    Ds = 0x3E,
    Fs = 0x64,
    Gs = 0x65,
}

impl core::fmt::Display for Segment {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Es => f.write_str("es"),
            Self::Cs => f.write_str("cs"),
            Self::Ss => f.write_str("ss"),
            Self::Ds => f.write_str("ds"),
            Self::Fs => f.write_str("fs"),
            Self::Gs => f.write_str("gs"),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct MaskSpec {
    zero: bool,
    mask_reg: X86Register,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
#[non_exhaustive]
pub enum X86Operand {
    Mask(MaskSpec),
    Register(X86Register),
    Memory(X86RegisterClass, Option<Segment>, X86MemoryOperand),
    Immediate(i64),
    RelOffset(Address),
    AbsOffset(Address),
}

impl core::fmt::Display for X86Operand {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            X86Operand::Mask(mask) => {
                f.write_fmt(format_args!("{{{}}}", mask.mask_reg))?;

                if mask.zero {
                    f.write_str("{z}")?;
                }

                Ok(())
            }
            X86Operand::Register(reg) => reg.fmt(f),
            X86Operand::Memory(size, seg, mem) => {
                match size {
                    X86RegisterClass::Byte => f.write_str("byte ptr ")?,
                    X86RegisterClass::Word => f.write_str("word ptr ")?,
                    X86RegisterClass::Double => f.write_str("dword ptr ")?,
                    X86RegisterClass::Quad => f.write_str("qword ptr ")?,
                    X86RegisterClass::St => f.write_str("tbyte ptr ")?,
                    X86RegisterClass::Xmm => f.write_str("xmmword ptr ")?,
                    X86RegisterClass::Ymm => f.write_str("ymmword ptr ")?,
                    X86RegisterClass::Zmm => f.write_str("zmmword ptr ")?,
                    X86RegisterClass::Tmm => f.write_str("tmmword ptr ")?,
                    r => panic!("Bad size specifier for x86 memory operand {:?}", r),
                }

                if let Some(seg) = seg {
                    f.write_fmt(format_args!("{}:", seg))?;
                }

                f.write_fmt(format_args!("[{}]", mem))
            }
            X86Operand::Immediate(imm) => imm.fmt(f),
            X86Operand::AbsOffset(addr) => f.write_fmt(format_args!("offset {}", addr)),
            X86Operand::RelOffset(addr) => f.write_fmt(format_args!("disp {}", addr)),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum X86Displacement {
    Offset(i32),
    Addr(Address),
}

impl core::fmt::Display for X86Displacement {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::Offset(val) => val.fmt(f),
            Self::Addr(addr) => addr.fmt(f),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
#[non_exhaustive]
pub enum X86MemoryOperand {
    Indirect {
        reg: X86Register,
        disp: Option<X86Displacement>,
    },
    IndirectSib {
        scale: u8,
        index: X86Register,
        base: Option<X86Register>,
        disp: Option<X86Displacement>,
    },
    RelAddr(Address),
    AbsAddr(Address),
}

impl core::fmt::Display for X86MemoryOperand {
    #[allow(unused_assignments)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::AbsAddr(addr) => addr.fmt(f),
            Self::RelAddr(addr) => f.write_fmt(format_args!("{}+rip", addr)),
            Self::Indirect { reg, disp } => {
                reg.fmt(f)?;
                if let Some(disp) = disp {
                    f.write_str(" + ")?;
                    disp.fmt(f)?
                }
                Ok(())
            }
            Self::IndirectSib {
                base,
                scale,
                index,
                disp,
            } => {
                let mut sep = "";
                if *scale > 1 {
                    f.write_fmt(format_args!("{}*", scale))?;
                }
                index.fmt(f)?;
                sep = " + ";

                if let Some(base) = base {
                    f.write_str(sep)?;
                    sep = " + ";
                    base.fmt(f)?;
                }

                if let Some(disp) = disp {
                    f.write_str(sep)?;
                    disp.fmt(f)?;
                }

                Ok(())
            }
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
#[non_exhaustive]
pub enum X86OperandMode {
    TargetRegister(X86Register),
    Reg(X86RegisterClass),
    RegRM(X86RegisterClass),
    MemOrReg(X86RegisterClass),
    MemoryOnly(X86RegisterClass),
    MemOffset(X86RegisterClass),
    Immediate(X86RegisterClass),
    ImmediateVal(i64),
    RelAddr(u8),
    AbsAddr(u8),
    StringAddr(X86RegisterClass, X86Register),
    Mask,
}

impl X86Operand {
    pub fn matches_mode<F: Fn(X86OperandMode) -> bool>(&self, accepter: F) -> bool {
        match self {
            X86Operand::Mask(_) => accepter(X86OperandMode::Mask),
            X86Operand::Register(reg) if reg.class() == X86RegisterClass::ByteRex => [
                X86OperandMode::TargetRegister(*reg),
                X86OperandMode::Reg(X86RegisterClass::Byte),
                X86OperandMode::MemOrReg(X86RegisterClass::Byte),
                X86OperandMode::RegRM(X86RegisterClass::Byte),
            ]
            .iter()
            .copied()
            .any(accepter),
            X86Operand::Register(reg) => [
                X86OperandMode::TargetRegister(*reg),
                X86OperandMode::Reg(reg.class()),
                X86OperandMode::MemOrReg(reg.class()),
                X86OperandMode::RegRM(reg.class()),
            ]
            .iter()
            .copied()
            .any(accepter),
            X86Operand::Memory(
                cl,
                None | Some(Segment::Es),
                X86MemoryOperand::Indirect {
                    reg: reg @ (X86Register::Di | X86Register::Edi | X86Register::Rdi),
                    disp: None,
                },
            ) => [
                X86OperandMode::MemoryOnly(*cl),
                X86OperandMode::MemOrReg(*cl),
                X86OperandMode::StringAddr(*cl, *reg),
            ]
            .iter()
            .copied()
            .any(accepter),
            X86Operand::Memory(cl, _, X86MemoryOperand::Indirect { reg, disp: None }) => [
                X86OperandMode::MemoryOnly(*cl),
                X86OperandMode::MemOrReg(*cl),
                X86OperandMode::StringAddr(*cl, *reg),
            ]
            .iter()
            .copied()
            .any(accepter),
            X86Operand::Memory(cl, _, X86MemoryOperand::RelAddr(_)) => [
                X86OperandMode::MemOffset(*cl),
                X86OperandMode::MemoryOnly(*cl),
                X86OperandMode::MemOrReg(*cl),
            ]
            .iter()
            .copied()
            .any(accepter),
            X86Operand::Memory(cl, _, _) => [
                X86OperandMode::MemoryOnly(*cl),
                X86OperandMode::MemOrReg(*cl),
            ]
            .iter()
            .copied()
            .any(accepter),
            X86Operand::Immediate(x) if -128 <= *x && *x < 128 => [
                X86OperandMode::ImmediateVal(*x),
                X86OperandMode::Immediate(X86RegisterClass::Byte),
                X86OperandMode::Immediate(X86RegisterClass::Word),
                X86OperandMode::Immediate(X86RegisterClass::Double),
                X86OperandMode::Immediate(X86RegisterClass::Quad),
            ]
            .iter()
            .copied()
            .any(accepter),
            X86Operand::Immediate(x) if -32768 <= *x && *x < 32768 => [
                X86OperandMode::ImmediateVal(*x),
                X86OperandMode::Immediate(X86RegisterClass::Word),
                X86OperandMode::Immediate(X86RegisterClass::Double),
                X86OperandMode::Immediate(X86RegisterClass::Quad),
            ]
            .iter()
            .copied()
            .any(accepter),
            X86Operand::Immediate(x) if -2147483648 <= *x && *x < 2147483648 => [
                X86OperandMode::ImmediateVal(*x),
                X86OperandMode::Immediate(X86RegisterClass::Double),
                X86OperandMode::Immediate(X86RegisterClass::Quad),
            ]
            .iter()
            .copied()
            .any(accepter),
            X86Operand::Immediate(x) => [
                X86OperandMode::ImmediateVal(*x),
                X86OperandMode::Immediate(X86RegisterClass::Quad),
            ]
            .iter()
            .copied()
            .any(accepter),
            X86Operand::RelOffset(Address::Disp(x)) if -128 <= *x && *x < 128 => [
                X86OperandMode::RelAddr(8),
                X86OperandMode::RelAddr(16),
                X86OperandMode::RelAddr(32),
                X86OperandMode::Immediate(X86RegisterClass::Byte),
                X86OperandMode::Immediate(X86RegisterClass::Word),
                X86OperandMode::Immediate(X86RegisterClass::Double),
                X86OperandMode::Immediate(X86RegisterClass::Quad),
            ]
            .iter()
            .copied()
            .any(accepter),
            X86Operand::RelOffset(Address::Disp(x)) if -32768 <= *x && *x < 32768 => [
                X86OperandMode::RelAddr(16),
                X86OperandMode::RelAddr(32),
                X86OperandMode::Immediate(X86RegisterClass::Word),
                X86OperandMode::Immediate(X86RegisterClass::Double),
                X86OperandMode::Immediate(X86RegisterClass::Quad),
            ]
            .iter()
            .copied()
            .any(accepter),
            X86Operand::RelOffset(Address::Disp(x)) if -2147483648 <= *x && *x < 2147483648 => [
                X86OperandMode::RelAddr(32),
                X86OperandMode::Immediate(X86RegisterClass::Double),
                X86OperandMode::Immediate(X86RegisterClass::Quad),
            ]
            .iter()
            .copied()
            .any(accepter),
            X86Operand::RelOffset(_) => [
                X86OperandMode::RelAddr(32),
                X86OperandMode::Immediate(X86RegisterClass::Quad),
            ]
            .iter()
            .copied()
            .any(accepter),
            X86Operand::AbsOffset(Address::Abs(x)) if *x < 256 => [
                X86OperandMode::AbsAddr(8),
                X86OperandMode::AbsAddr(16),
                X86OperandMode::AbsAddr(32),
                X86OperandMode::Immediate(X86RegisterClass::Byte),
                X86OperandMode::Immediate(X86RegisterClass::Word),
                X86OperandMode::Immediate(X86RegisterClass::Double),
                X86OperandMode::Immediate(X86RegisterClass::Quad),
            ]
            .iter()
            .copied()
            .any(accepter),
            X86Operand::AbsOffset(Address::Abs(x)) if *x < 65536 => [
                X86OperandMode::AbsAddr(16),
                X86OperandMode::AbsAddr(32),
                X86OperandMode::Immediate(X86RegisterClass::Word),
                X86OperandMode::Immediate(X86RegisterClass::Double),
                X86OperandMode::Immediate(X86RegisterClass::Quad),
            ]
            .iter()
            .copied()
            .any(accepter),
            X86Operand::AbsOffset(Address::Abs(x)) if *x < 4294967296 => [
                X86OperandMode::AbsAddr(32),
                X86OperandMode::Immediate(X86RegisterClass::Double),
                X86OperandMode::Immediate(X86RegisterClass::Quad),
            ]
            .iter()
            .copied()
            .any(accepter),
            X86Operand::AbsOffset(_) => [
                X86OperandMode::AbsAddr(32),
                X86OperandMode::Immediate(X86RegisterClass::Quad),
            ]
            .iter()
            .copied()
            .any(accepter),
        }
    }
}

mod insn;

pub use insn::X86CodegenOpcode;

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum X86Prefix {
    Lock,
    Repnz,
    Repz,
    Rep,
}

impl X86Prefix {
    pub fn mnemonic(&self) -> &'static str {
        match self {
            Self::Lock => "lock",
            Self::Repnz => "repnz",
            Self::Repz => "repz",
            Self::Rep => "rep",
        }
    }

    pub fn opcode(&self) -> u8 {
        match self {
            Self::Lock => 0xF0,
            Self::Repnz => 0xF2,
            Self::Repz => 0xF3,
            Self::Rep => 0xF3,
        }
    }
}

impl core::fmt::Display for X86Prefix {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.write_str(self.mnemonic())
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct X86Instruction {
    prefix: Option<X86Prefix>,
    opc: X86CodegenOpcode,
    oprs: Vec<X86Operand>,
    mode_override: Option<X86Mode>,
}

macro_rules! nop_instructions{
    [
        $($mnemonic:ident),* $(,)?
    ] => {
        paste::paste!{
            #[allow(non_upper_case_globals)]
            impl X86Instruction{
                $(pub const [<$mnemonic:camel>]: Self = Self::new(X86CodegenOpcode::[<$mnemonic:camel>],vec![]);)*
            }
        }
    }
}

nop_instructions![
    ud2, int3, into, iret, iretq, ret, retf, leave, pushf, pushfd, pushfq, lahf, sahf, nop,
];

impl X86Instruction {
    pub const fn new(opc: X86CodegenOpcode, oprs: Vec<X86Operand>) -> Self {
        Self {
            prefix: None,
            opc,
            oprs,
            mode_override: None,
        }
    }
    pub const fn with_prefix(mut self, prefix: X86Prefix) -> Self {
        self.prefix = Some(prefix);
        self
    }
    pub const fn with_mode(mut self, mode: X86Mode) -> Self {
        self.mode_override = Some(mode);
        self
    }

    pub const fn opcode(&self) -> X86CodegenOpcode {
        self.opc
    }

    pub fn operands(&self) -> &[X86Operand] {
        &self.oprs
    }

    pub const fn prefix(&self) -> Option<X86Prefix> {
        self.prefix
    }

    pub const fn mode_override(&self) -> Option<X86Mode> {
        self.mode_override
    }
}

impl core::fmt::Display for X86Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(prefix) = &self.prefix {
            prefix.fmt(f)?;
            f.write_str(" ")?;
        }

        self.opc.fmt(f)?;

        let mut sep = " ";

        for opr in &self.oprs {
            f.write_str(sep)?;
            sep = ", ";
            opr.fmt(f)?;
        }

        if let Some(x) = self.mode_override {
            f.write_fmt(format_args!("; in mode {:?}", x))?;
        }

        Ok(())
    }
}

pub struct X86Encoder<W> {
    writer: W,
    mode: X86Mode,
}

impl<W> X86Encoder<W> {
    pub const fn new(writer: W, mode: X86Mode) -> Self {
        Self { writer, mode }
    }

    pub fn into_inner(self) -> W {
        self.writer
    }

    pub fn inner(&self) -> &W {
        &self.writer
    }

    pub fn inner_mut(&mut self) -> &mut W {
        &mut self.writer
    }
    pub fn mode(&self) -> X86Mode {
        self.mode
    }
    pub fn set_mode(&mut self, mode: X86Mode) {
        self.mode = mode;
    }
}

impl<W: io::Write> io::Write for X86Encoder<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.writer.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.writer.write_all(buf)
    }
}

impl<W: InsnWrite> InsnWrite for X86Encoder<W> {
    fn offset(&self) -> usize {
        self.writer.offset()
    }
    fn write_addr(&mut self, size: usize, addr: Address, rel: bool) -> std::io::Result<()> {
        self.writer.write_addr(size, addr, rel)
    }
    fn write_reloc(&mut self, reloc: crate::traits::Reloc) -> std::io::Result<()> {
        self.writer.write_reloc(reloc)
    }
    fn write_zeroes(&mut self, count: usize) -> std::io::Result<()> {
        self.writer.write_zeroes(count)
    }
}

pub struct NoRM;
pub struct NoReg;
pub struct NoReg3;
pub struct NoMask;

pub struct NoOpcode;

pub struct RM;

pub struct Reg;

pub struct Reg3;

pub struct Mask;

pub struct Opcode;

pub struct ModRMBuilder<HasRM, HasReg, HasReg3, HasMask, HasOpcode> {
    use_nf: bool,
    sib_byte: Option<u8>,
    /// [0]=>b,[1]=>x,[2]=>r
    reg_top_bits: [u8; 3],
    modrm_byte: u8,
    disp: Option<X86Displacement>,
    size: Option<X86RegisterClass>,
    reg3: Option<u8>,
    mode: X86Mode,
    group: Option<X86InstructionMap>,
    sse_prefix: Option<SsePrefix>,
    mask: Option<MaskSpec>,
    rel_addr: bool,
    addr_size: Option<X86RegisterClass>,
    sreg: Option<Segment>,
    _status: PhantomData<(HasRM, HasReg, HasReg3, HasMask, HasOpcode)>,
}

impl ModRMBuilder<NoRM, NoReg, NoReg3, NoMask, NoOpcode> {
    pub const fn new(mode: X86Mode) -> Self {
        Self {
            use_nf: false,
            sib_byte: None,
            reg_top_bits: [0u8; 3],
            modrm_byte: 0,
            disp: None,
            size: None,
            reg3: None,
            mode,
            group: None,
            sse_prefix: None,
            mask: None,
            rel_addr: false,
            addr_size: None,
            sreg: None,
            _status: PhantomData,
        }
    }
}

impl<HasRM, HasReg, HasReg3, HasMask, HasOpcode>
    ModRMBuilder<HasRM, HasReg, HasReg3, HasMask, HasOpcode>
{
    pub fn prefix_allowed(&self, prefix: &X86EncodingPrefix, opts: ModRMOptions) -> bool {
        match prefix {
            X86EncodingPrefix::NoPrefix => {
                match self.reg_top_bits {
                    [0, 0, 0] => {}
                    _ => return false,
                }

                if self.use_nf {
                    return false; // Need EVEX
                }

                match self.size {
                    Some(X86RegisterClass::ByteRex) => return false, // always need a REX prefix
                    Some(X86RegisterClass::Quad) if !opts.no_rex_w => return false, // need REX.W
                    Some(X86RegisterClass::Ymm) => return false,     // Need VEX
                    Some(X86RegisterClass::Zmm) => return false,     // Need EVEX
                    _ => {}
                }

                if self.reg3.is_some() {
                    return false; // need VEX or EVEX
                }

                if self.mask.is_some() {
                    return false; // need EVEX
                }

                true
            }
            X86EncodingPrefix::Rex => {
                if self.mode != X86Mode::Long {
                    return false;
                }

                if self.use_nf {
                    return false; // Need EVEX
                }

                match self.reg_top_bits {
                    [0 | 1, 0 | 1, 0 | 1] => {}
                    _ => return false, // 2 top bits needs REX2 or EVEX
                }

                match self.size {
                    Some(X86RegisterClass::Ymm) => return false, // Need VEX
                    Some(X86RegisterClass::Zmm) => return false, // Need EVEX
                    _ => {} // Note No prefix is generated when a non-REX compatible byte register is used, no need to filter here
                }

                if self.reg3.is_some() {
                    return false; // need VEX or EVEX
                }

                if self.mask.is_some() {
                    return false; // need EVEX
                }

                true
            }
            X86EncodingPrefix::Rex2 => {
                if self.mode != X86Mode::Long {
                    return false;
                }

                if self.use_nf {
                    return false; // Need EVEX
                }

                match self.size {
                    Some(X86RegisterClass::Ymm) => return false, // Need VEX
                    Some(X86RegisterClass::Zmm) => return false, // Need EVEX
                    _ => {} // Note No prefix is generated when a non-REX compatible byte register is used, no need to filter here
                }

                if self.reg3.is_some() {
                    return false; // need VEX or EVEX
                }

                self.mask.is_none()
            }
            X86EncodingPrefix::Vex => {
                if self.use_nf {
                    return false; // Need EVEX
                }

                match self.reg_top_bits {
                    [0 | 1, 0 | 1, 0 | 1] => {}
                    _ => return false, // 2 top bits needs REX2 or EVEX
                }

                // Match is easier to extend
                #[allow(clippy::single_match)]
                match self.size {
                    Some(X86RegisterClass::Zmm) => return false, // Need EVEX
                    _ => {}
                }

                match self.reg3 {
                    Some(0..=15) => {}
                    Some(_) => return false,
                    None => {}
                }

                if self.mask.is_some() {
                    return false; // need EVEX
                }

                true // Legacy instructions cannot be enoded using VEX
            }
            X86EncodingPrefix::Evex => {
                self.mode != X86Mode::Real && self.mode != X86Mode::Virtual8086
            }
            prefix => panic!("Unknown prefix: {:?}", prefix),
        }
    }
}

impl<HasRM, HasReg, HasReg3, HasMask> ModRMBuilder<HasRM, HasReg, HasReg3, HasMask, NoOpcode> {
    pub fn with_group_and_sse_prefix(
        self,
        group: X86InstructionMap,
        sse_prefix: Option<SsePrefix>,
    ) -> io::Result<ModRMBuilder<HasRM, HasReg, HasReg3, HasMask, Opcode>> {
        let Self {
            use_nf,
            sib_byte,
            reg_top_bits,
            modrm_byte,
            disp,
            size,
            reg3,
            mode,
            group: _,
            sse_prefix: _,
            mask,
            rel_addr,
            addr_size,
            sreg,
            _status,
        } = self;
        Ok(ModRMBuilder {
            use_nf,
            sib_byte,
            reg_top_bits,
            modrm_byte,
            disp,
            size,
            reg3,
            mode,
            group: Some(group),
            sse_prefix,
            mask,
            rel_addr,
            addr_size,
            sreg,
            _status: PhantomData,
        })
    }
}

impl<HasRM, HasReg, HasReg3, HasMask> ModRMBuilder<HasRM, HasReg, HasReg3, HasMask, Opcode> {
    pub fn encode_modrm<W: InsnWrite>(&self, mut writer: W) -> io::Result<()> {
        writer.write_all(core::slice::from_ref(&self.modrm_byte))?;

        if let Some(sib) = &self.sib_byte {
            writer.write_all(core::slice::from_ref(sib))?;
        }

        if let Some(addr) = &self.disp {
            let size = if (self.modrm_byte & 0o300) == 0o100 {
                8
            } else if self.mode == X86Mode::Real || self.mode == X86Mode::Virtual8086 {
                16
            } else {
                32
            };

            match addr {
                X86Displacement::Addr(addr) => {
                    writer.write_addr(size, addr.clone(), self.rel_addr)?
                }
                X86Displacement::Offset(val) => {
                    let val = val.to_le_bytes();
                    writer.write_all(&val[..(size >> 3)])?;
                }
            }
        }

        Ok(())
    }
    pub fn encode_prefix<W: InsnWrite>(
        &self,
        mut writer: W,
        prefix: X86EncodingPrefix,
        opts: ModRMOptions,
    ) -> io::Result<()> {
        let escape_size = match self.mode {
            X86Mode::Virtual8086 | X86Mode::Real => X86RegisterClass::Double,
            _ => X86RegisterClass::Word,
        };

        match (self.addr_size, self.mode) {
            (None, _)
            | (Some(X86RegisterClass::Word), X86Mode::Real | X86Mode::Virtual8086)
            | (Some(X86RegisterClass::Double), X86Mode::Protected | X86Mode::Compatibility)
            | (Some(X86RegisterClass::Quad), X86Mode::Long) => {}
            (
                Some(X86RegisterClass::Double),
                X86Mode::Real | X86Mode::Virtual8086 | X86Mode::Long,
            )
            | (Some(X86RegisterClass::Word), X86Mode::Protected | X86Mode::Compatibility) => {
                writer.write_all(&[0x67])?
            }
            (Some(size), mode) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("{:?} addresses are not supported in {:?}", size, mode),
                ))
            }
        }

        let size_escape = &core::slice::from_ref(&0x66u8)
            [(opts.no_escape || self.size != Some(escape_size)) as usize..];

        writer.write_all(size_escape)?;

        #[allow(unsafe_code)]
        writer.write_all(
            self.sreg
                .as_ref()
                .map(|s| unsafe { &*(s as *const Segment as *const u8) })
                .map(core::slice::from_ref)
                .unwrap_or(&[]),
        )?;

        match prefix {
            X86EncodingPrefix::NoPrefix => {
                writer.write_all(
                    self.sse_prefix
                        .as_ref()
                        .map(|s| s.encoding())
                        .unwrap_or(&[]),
                )?;

                writer.write_all(self.group.unwrap().escape_prefix_bytes())
            }
            X86EncodingPrefix::Rex => {
                let mut rex_byte = 0x40u8;

                for (b, n) in self.reg_top_bits.iter().zip(0u8..) {
                    rex_byte |= *b << n
                }

                if !opts.no_rex_w && self.size == Some(X86RegisterClass::Quad) {
                    rex_byte |= 8;
                }

                writer.write_all(
                    self.sse_prefix
                        .as_ref()
                        .map(|s| s.encoding())
                        .unwrap_or(&[]),
                )?;

                writer.write_all(core::slice::from_ref(&rex_byte))?;

                writer.write_all(self.group.unwrap().escape_prefix_bytes())
            }
            X86EncodingPrefix::Rex2 => {
                let mut rex2_bytes = [0xD5, 0x00];

                for (b, n) in self.reg_top_bits.iter().zip(0u8..) {
                    rex2_bytes[1] |= (*b & 0x1) << n;
                    rex2_bytes[1] |= (*b & 0x2) << (n + 3);
                }

                if !opts.no_rex_w && self.size == Some(X86RegisterClass::Quad) {
                    rex2_bytes[1] |= 8;
                }

                match self.group.unwrap() {
                    X86InstructionMap::Legacy => {}
                    X86InstructionMap::Extended => rex2_bytes[1] |= 0x80,
                    _ => panic!("Improper prefix for instruction"),
                }

                writer.write_all(
                    self.sse_prefix
                        .as_ref()
                        .map(|s| s.encoding())
                        .unwrap_or(&[]),
                )?;

                writer.write_all(&rex2_bytes)
            }
            X86EncodingPrefix::Vex => todo!(),
            X86EncodingPrefix::Evex => todo!(),
            _ => panic!("bad prefix"),
        }
    }
}

impl<HasReg, HasReg3, HasMask, HasOpcode> ModRMBuilder<NoRM, HasReg, HasReg3, HasMask, HasOpcode> {
    // Sets the r/m field and related fields
    /// Note: Unexpected results may occur if called multiple times with different operands
    pub fn with_rm(
        mut self,
        op: X86Operand,
    ) -> io::Result<ModRMBuilder<RM, HasReg, HasReg3, HasMask, HasOpcode>> {
        match op {
            X86Operand::Mask(_) => panic!("Cannot specify a mask specifier in modr/m"),
            X86Operand::Register(r) => {
                self.size = Some(r.class());
                let rnum = r.regnum();

                self.modrm_byte = (self.modrm_byte & 0o070) | 0o300 | (rnum & 0o007);
                self.reg_top_bits[0] = rnum >> 3;
                self.addr_size = None; // Not an address
            }
            X86Operand::Memory(size, sreg, op) => {
                self.size = Some(size);
                self.sreg = sreg;

                match op {
                    X86MemoryOperand::Indirect { reg, disp } => {
                        let class = reg.class();
                        self.addr_size = Some(class);
                        if class == X86RegisterClass::Word {
                            let rm = match reg {
                                X86Register::Si => 0o4,
                                X86Register::Di => 0o5,
                                X86Register::Bp => 0o6,
                                X86Register::Bx => 0o7,
                                reg => {
                                    return Err(io::Error::new(
                                        io::ErrorKind::InvalidInput,
                                        format!("Indirect to register {} cannot be encoded", reg),
                                    ))
                                }
                            };
                            let (mode, addr) = match (rm, disp) {
                                (0o006, None) => (0o100, Some(X86Displacement::Offset(0))),
                                (_, None) => (0o000, None),
                                (_, Some(X86Displacement::Offset(x))) if x < 256 && x >= 0 => {
                                    (0o100, Some(X86Displacement::Offset(x)))
                                }
                                (_, Some(X86Displacement::Addr(Address::Abs(x)))) if x < 256 => {
                                    (0o100, Some(X86Displacement::Addr(Address::Abs(x))))
                                }
                                (_, Some(addr)) => (0o200, Some(addr)),
                            };

                            self.modrm_byte = (self.modrm_byte & 0o070) | mode | rm;
                            self.disp = addr;
                        } else {
                            let regno = reg.regnum();
                            let rm = regno & 0o7;
                            self.reg_top_bits[0] = regno >> 3;
                            let (mode, addr) = match (rm, disp) {
                                (0o005, None) => (0o100, Some(X86Displacement::Offset(0))),
                                (_, None) => (0o000, None),
                                (_, Some(X86Displacement::Offset(x))) if x < 256 && x >= 0 => {
                                    (0o100, Some(X86Displacement::Offset(x)))
                                }
                                (_, Some(X86Displacement::Addr(Address::Abs(x)))) if x < 256 => {
                                    (0o100, Some(X86Displacement::Addr(Address::Abs(x))))
                                }
                                (_, Some(addr)) => (0o200, Some(addr)),
                            };

                            if rm == 0o004 {
                                self.sib_byte = Some(0o044);
                            }

                            self.modrm_byte = (self.modrm_byte & 0o070) | mode | rm;
                            self.disp = addr;
                        }
                    }
                    X86MemoryOperand::IndirectSib {
                        scale,
                        index,
                        base,
                        disp,
                    } => {
                        let addr_size = index.class();

                        if addr_size == X86RegisterClass::Word {
                            if scale != 1 {
                                return Err(io::Error::new(
                                    io::ErrorKind::InvalidInput,
                                    format!("16-bit modr/m cannot encode scale {}", scale),
                                ));
                            }

                            let (mode, disp) = match (index, base, disp) {
                                (X86Register::Bp, None, None) => {
                                    (0o100, Some(X86Displacement::Offset(0)))
                                }
                                (_, _, None) => (0o000, None),
                                (_, _, Some(X86Displacement::Offset(x))) if x < 256 => {
                                    (0o100, Some(X86Displacement::Offset(x)))
                                }
                                (_, _, Some(X86Displacement::Addr(Address::Abs(x)))) if x < 256 => {
                                    (0o100, Some(X86Displacement::Addr(Address::Abs(x))))
                                }
                                (_, _, Some(addr)) => (0o200, Some(addr)),
                            };

                            let rm = match (index, base) {
                                (X86Register::Si, Some(X86Register::Bx)) => 0o000,
                                (X86Register::Di, Some(X86Register::Bx)) => 0o001,
                                (X86Register::Si, Some(X86Register::Bp)) => 0o002,
                                (X86Register::Di, Some(X86Register::Bp)) => 0o003,
                                (X86Register::Bx, Some(X86Register::Si)) => 0o000,
                                (X86Register::Bx, Some(X86Register::Di)) => 0o001,
                                (X86Register::Bp, Some(X86Register::Si)) => 0o002,
                                (X86Register::Bp, Some(X86Register::Di)) => 0o003,
                                (X86Register::Si, None) => 0o004,
                                (X86Register::Di, None) => 0o005,
                                (X86Register::Bp, None) => 0o006,
                                (X86Register::Bx, None) => 0o007,
                                (reg, None) => {
                                    return Err(io::Error::new(
                                        io::ErrorKind::InvalidInput,
                                        format!("16-bit modr/m cannot encode register {}", reg),
                                    ))
                                }
                                (reg, Some(base)) => {
                                    return Err(io::Error::new(
                                        io::ErrorKind::InvalidInput,
                                        format!("16-bit modr/m cannot encode {}+{}", reg, base),
                                    ))
                                }
                            };

                            self.modrm_byte = (self.modrm_byte & 0o070) | mode | rm;
                            self.disp = disp;
                        } else {
                            let scale = match scale {
                                1 => 0o000,
                                2 => 0o100,
                                4 => 0o200,
                                8 => 0o300,
                                scale => {
                                    return Err(io::Error::new(
                                        io::ErrorKind::InvalidInput,
                                        format!("SIB cannot encode scale {}", scale),
                                    ))
                                }
                            };

                            let idx = index.regnum() & 0o007 << 3;
                            if idx == 4 {
                                return Err(io::Error::new(
                                    io::ErrorKind::InvalidInput,
                                    format!("Cannot index by {}", idx),
                                ));
                            }
                            self.reg_top_bits[1] = idx >> 3;
                            self.reg_top_bits[0] = base.map(X86Register::regnum).unwrap_or(0) >> 3;
                            let base = base.map(X86Register::regnum).unwrap_or(0o005) & 0o007;

                            let (mode, disp) = match (base, disp) {
                                (5, None) => (0o100, Some(X86Displacement::Offset(0))),
                                (_, None) => (0o000, None),
                                (_, Some(X86Displacement::Offset(x))) if x < 256 => {
                                    (0o100, Some(X86Displacement::Offset(x)))
                                }
                                (_, Some(X86Displacement::Addr(Address::Abs(x)))) if x < 256 => {
                                    (0o100, Some(X86Displacement::Addr(Address::Abs(x))))
                                }
                                (_, Some(addr)) => (0o0200, Some(addr)),
                            };

                            let rm = 0o004;

                            self.sib_byte = Some(scale | idx | base);
                            self.modrm_byte = (self.modrm_byte & 0o070) | mode | rm;
                            self.disp = disp;
                        }
                    }
                    X86MemoryOperand::RelAddr(addr) => {
                        self.rel_addr = true;
                        self.modrm_byte = (self.modrm_byte & 0o070) | 0o005;
                        self.disp = Some(X86Displacement::Addr(addr));
                    }
                    X86MemoryOperand::AbsAddr(_) => todo!(),
                }
            }
            op => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Cannot encode {} in a modr/m byte", op),
                ))
            }
        }
        let Self {
            use_nf,
            sib_byte,
            reg_top_bits,
            modrm_byte,
            disp,
            size,
            reg3,
            mode,
            group,
            sse_prefix,
            mask,
            rel_addr,
            addr_size,
            sreg,
            _status,
        } = self;
        Ok(ModRMBuilder {
            use_nf,
            sib_byte,
            reg_top_bits,
            modrm_byte,
            disp,
            size,
            reg3,
            mode,
            group,
            sse_prefix,
            mask,
            rel_addr,
            addr_size,
            sreg,
            _status: PhantomData,
        })
    }
    pub fn with_op_size(
        self,
        size: X86RegisterClass,
    ) -> io::Result<ModRMBuilder<RM, HasReg, HasReg3, HasMask, HasOpcode>> {
        let Self {
            use_nf,
            sib_byte,
            reg_top_bits,
            modrm_byte,
            disp,
            size: _,
            reg3,
            mode,
            group,
            sse_prefix,
            mask,
            rel_addr,
            addr_size,
            sreg,
            _status,
        } = self;
        Ok(ModRMBuilder {
            use_nf,
            sib_byte,
            reg_top_bits,
            modrm_byte,
            disp,
            size: Some(size),
            reg3,
            mode,
            group,
            sse_prefix,
            mask,
            rel_addr,
            addr_size,
            sreg,
            _status: PhantomData,
        })
    }
}

impl<HasRM, HasReg3, HasMask, HasOpcode> ModRMBuilder<HasRM, NoReg, HasReg3, HasMask, HasOpcode> {
    pub fn with_reg(
        mut self,
        op: X86Operand,
    ) -> io::Result<ModRMBuilder<HasRM, Reg, HasReg3, HasMask, HasOpcode>> {
        match op {
            X86Operand::Register(reg) => {
                let regno = reg.regnum();
                self.size = Some(reg.class());
                self.modrm_byte = (self.modrm_byte & 0o307) | (regno & 0o007) << 3;
                self.reg_top_bits[2] = regno >> 3;
                let Self {
                    use_nf,
                    sib_byte,
                    reg_top_bits,
                    modrm_byte,
                    disp,
                    size,
                    reg3,
                    mode,
                    group,
                    sse_prefix,
                    mask,
                    rel_addr,
                    addr_size,
                    sreg,
                    _status,
                } = self;
                Ok(ModRMBuilder {
                    use_nf,
                    sib_byte,
                    reg_top_bits,
                    modrm_byte,
                    disp,
                    size,
                    reg3,
                    mode,
                    group,
                    sse_prefix,
                    mask,
                    rel_addr,
                    addr_size,
                    sreg,
                    _status: PhantomData,
                })
            }
            op => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Cannot encode {} in /r field", op),
            )),
        }
    }

    pub fn with_control_bits(
        mut self,
        ctrl: u8,
    ) -> io::Result<ModRMBuilder<HasRM, Reg, HasReg3, HasMask, HasOpcode>> {
        self.modrm_byte = (self.modrm_byte & 0o307) | (ctrl & 0o007) << 3;
        let Self {
            use_nf,
            sib_byte,
            reg_top_bits,
            modrm_byte,
            disp,
            size,
            reg3,
            mode,
            group,
            sse_prefix,
            mask,
            rel_addr,
            addr_size,
            sreg,
            _status,
        } = self;
        Ok(ModRMBuilder {
            use_nf,
            sib_byte,
            reg_top_bits,
            modrm_byte,
            disp,
            size,
            reg3,
            mode,
            group,
            sse_prefix,
            mask,
            rel_addr,
            addr_size,
            sreg,
            _status: PhantomData,
        })
    }
}

impl<HasRM, HasReg, HasMask, HasOpcode> ModRMBuilder<HasRM, HasReg, NoReg3, HasMask, HasOpcode> {
    pub fn with_reg3(
        mut self,
        op: X86Operand,
    ) -> io::Result<ModRMBuilder<HasRM, HasReg, Reg3, HasMask, HasOpcode>> {
        match op {
            X86Operand::Register(reg) => {
                self.reg3 = Some(reg.regnum());
                let Self {
                    use_nf,
                    sib_byte,
                    reg_top_bits,
                    modrm_byte,
                    disp,
                    size,
                    reg3,
                    mode,
                    group,
                    sse_prefix,
                    mask,
                    rel_addr,
                    addr_size,
                    sreg,
                    _status,
                } = self;
                Ok(ModRMBuilder {
                    use_nf,
                    sib_byte,
                    reg_top_bits,
                    modrm_byte,
                    disp,
                    size,
                    reg3,
                    mode,
                    group,
                    sse_prefix,
                    mask,
                    rel_addr,
                    addr_size,
                    sreg,
                    _status: PhantomData,
                })
            }
            op => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Cannot encode {} in /r field", op),
            )),
        }
    }
}

impl<HasRM, HasReg, HasReg3, HasOpcode> ModRMBuilder<HasRM, HasReg, HasReg3, NoMask, HasOpcode> {
    pub fn with_mask(
        mut self,
        mask: X86Operand,
    ) -> io::Result<ModRMBuilder<HasRM, HasReg, HasReg3, Mask, HasOpcode>> {
        match mask {
            X86Operand::Mask(m) => {
                self.mask = Some(m);
                let Self {
                    use_nf,
                    sib_byte,
                    reg_top_bits,
                    modrm_byte,
                    disp,
                    size,
                    reg3,
                    mode,
                    group,
                    sse_prefix,
                    mask,
                    rel_addr,
                    addr_size,
                    sreg,
                    _status,
                } = self;
                Ok(ModRMBuilder {
                    use_nf,
                    sib_byte,
                    reg_top_bits,
                    modrm_byte,
                    disp,
                    size,
                    reg3,
                    mode,
                    group,
                    sse_prefix,
                    mask,
                    rel_addr,
                    addr_size,
                    sreg,
                    _status: PhantomData,
                })
            }
            op => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Cannot encode {} in mask", op),
            )),
        }
    }
}

impl<W: InsnWrite> X86Encoder<W> {
    pub fn write_single(&mut self, b: u8) -> io::Result<()> {
        self.write_all(core::slice::from_ref(&b))
    }
    #[allow(unused_variables)]
    pub fn write_insn(&mut self, insn: X86Instruction) -> io::Result<()> {
        let mode = insn.mode_override().unwrap_or(self.mode);

        let opc = insn.opcode();
        let oprs = insn.operands();

        let mut prefix_space = [MaybeUninit::uninit(); X86EncodingPrefix::PREFIX_COUNT];
        let valid_prefixes = opc.allowed_prefixes(oprs, &mut prefix_space);

        let encoding = opc.encoding_info(oprs).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Cannot encode instruction {}", insn),
            )
        })?;

        if let Some(prefix) = insn.prefix() {
            self.write_single(prefix.opcode())?;
        }

        match encoding.mode {
            X86EncodingMode::Prefix(_) => todo!("prefix"),
            X86EncodingMode::OpcodeOnly => {
                let builder = ModRMBuilder::new(mode)
                    .with_group_and_sse_prefix(encoding.map, encoding.sse_prefix)?;
                let option = ModRMOptions::NONE;
                for prefix in valid_prefixes {
                    if builder.prefix_allowed(prefix, option) {
                        builder.encode_prefix(&mut *self, *prefix, option)?;
                        self.write_single(encoding.base_opcode)?;
                        return Ok(());
                    }
                }
                Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!(
                        "Cannot encode instruction {} (no valid encoding prefix found)",
                        insn
                    ),
                ))
            }
            X86EncodingMode::ModRM(option) => match oprs {
                [op1 @ (X86Operand::Register(_) | X86Operand::Memory(_, _, _)), op2 @ X86Operand::Register(_)] =>
                {
                    let builder = ModRMBuilder::new(mode)
                        .with_group_and_sse_prefix(encoding.map, encoding.sse_prefix)?
                        .with_reg(op2.clone())?
                        .with_rm(op1.clone())?;
                    for prefix in valid_prefixes {
                        if builder.prefix_allowed(prefix, option) {
                            builder.encode_prefix(&mut *self, *prefix, option)?;
                            self.write_single(encoding.base_opcode)?;
                            builder.encode_modrm(&mut *self)?;
                            return Ok(());
                        }
                    }
                    Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!(
                            "Cannot encode instruction {} (no valid encoding prefix found)",
                            insn
                        ),
                    ))
                }
                [op2 @ X86Operand::Register(_), op1 @ X86Operand::Memory(_, _, _)] => {
                    let builder = ModRMBuilder::new(mode)
                        .with_group_and_sse_prefix(encoding.map, encoding.sse_prefix)?
                        .with_reg(op2.clone())?
                        .with_rm(op1.clone())?;
                    for prefix in valid_prefixes {
                        if builder.prefix_allowed(prefix, option) {
                            builder.encode_prefix(&mut *self, *prefix, option)?;
                            self.write_single(encoding.base_opcode)?;
                            builder.encode_modrm(&mut *self)?;
                            return Ok(());
                        }
                    }
                    Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!(
                            "Cannot encode instruction {} (no valid encoding prefix found)",
                            insn
                        ),
                    ))
                }
                oprs => todo!("modr/m /r {:?}", oprs),
            },
            X86EncodingMode::ModRMControl(bits, option) => match oprs {
                [op @ (X86Operand::Register(_) | X86Operand::Memory(_, _, _)), X86Operand::Immediate(imm)] =>
                {
                    let imm_bytes = imm.to_le_bytes();
                    let builder = ModRMBuilder::new(mode)
                        .with_group_and_sse_prefix(encoding.map, encoding.sse_prefix)?
                        .with_control_bits(bits)?
                        .with_rm(op.clone())?;
                    for prefix in valid_prefixes {
                        if builder.prefix_allowed(prefix, option) {
                            builder.encode_prefix(&mut *self, *prefix, option)?;
                            self.write_single(encoding.base_opcode)?;
                            builder.encode_modrm(&mut *self)?;
                            return self.write_all(&imm_bytes[..encoding.imm_size]);
                        }
                    }
                    Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!(
                            "Cannot encode instruction {} (no valid encoding prefix found)",
                            insn
                        ),
                    ))
                }
                oprs => todo!("{:?}", oprs),
            },
            X86EncodingMode::ModRMOp2(op) => todo!("{}", op),
            X86EncodingMode::OpReg(_) => todo!("+r"),
            X86EncodingMode::AvxModRM(_) => todo!("avx modr/m"),
            X86EncodingMode::EvexModRM(_) => todo!("evex modr/m"),
            X86EncodingMode::LegacyEvexModRM(_) => todo!("APX modr/m"),
            X86EncodingMode::OpcodeWithSize(_) => todo!("opcode with size"),
            X86EncodingMode::OffsetImm(bits) => {
                let size = match bits {
                    8 => X86RegisterClass::Byte,
                    16 => X86RegisterClass::Word,
                    32 => X86RegisterClass::Double,
                    64 => X86RegisterClass::Quad,
                    x => panic!("Invalid immediate size {}", x),
                };

                let builder = ModRMBuilder::new(mode)
                    .with_group_and_sse_prefix(encoding.map, encoding.sse_prefix)?
                    .with_op_size(size)?;
                let option = ModRMOptions::NONE;
                for prefix in valid_prefixes {
                    if builder.prefix_allowed(prefix, option) {
                        builder.encode_prefix(&mut *self, *prefix, option)?;
                        self.write_single(encoding.base_opcode)?;
                        match oprs {
                            [X86Operand::Immediate(n)] => {
                                let imm_bytes = n.to_le_bytes();

                                self.write_all(&imm_bytes[..(bits / 8)])?;
                            }
                            [X86Operand::RelOffset(off)] => {
                                self.write_addr(bits, off.clone(), true)?;
                            }
                            [X86Operand::AbsOffset(off)] => {
                                self.write_addr(bits, off.clone(), false)?;
                            }
                            _ => panic!("Invalid operand "),
                        }
                        return Ok(());
                    }
                }
                Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!(
                        "Cannot encode instruction {} (no valid encoding prefix found)",
                        insn
                    ),
                ))
            }
        }
    }
}
