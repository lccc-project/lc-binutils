use crate::traits::{Address, InsnWrite};

use super::{X86Register, X86RegisterClass};

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Prefix {
    Lock,
    Rep,
    Repnz,
    Repz,
    OpSizeOverride,
    AddrSizeOverride,
    Rex(u8),
    EsOverride,
    CsOverride,
    SsOverride,
    DsOverride,
    FsOverride,
    GsOverride,
    Vex {
        rex: u8,
        next: u16,
        len: bool,
        size_prefix: u8,
        src2: u8,
    },
    DoublePrecision,
    SinglePrecision,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum X86Mode {
    Real,
    Protected,
    Virtual8086,
    Compatibility,
    Long,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum X86OperandType {
    /// The Mod and R/M portions of a ModR/M byte (potentially with a trailing SIB byte)
    /// If an 1-byte register is used, use the class Byte. REX prefix will correctly shift it to ByteRex
    ModRM(X86RegisterClass),
    /// The R portion of a ModR/M byte
    /// If an 1-byte register is used, use the class Byte. REX prefix will correctly shift it to ByteRex
    Reg(X86RegisterClass),
    /// A Register Number embedded in the Opcode
    /// If an 1-byte register is used, use the class Byte. REX prefix will correctly shift it to ByteRex
    OpReg(X86RegisterClass),

    /// m/r16/32/64 depending on mode and prefixes
    ModRMGeneral,

    /// m/r16/32/64 depending on mode and 66h prefix
    ModRMMode,

    /// r16/32/64 depending on mode and prefixes
    RegGeneral,

    /// A register Number embedded in the opcode, with size depending on mode and prefix
    OpRegGeneral,

    /// A register number emebedded in the opcode, with size depending on the mode and 66h prefix
    OpRegMode,

    /// AL/rAX depending on class
    /// Note: Suprising results may occur if a class other than Byte, Word, Double, or Quad is Used
    AReg(X86RegisterClass),

    /// rAX depending on mode and prefixes
    ARegGeneral,

    /// rAX depening on the mode only
    ARegMode,

    DReg(X86RegisterClass),

    DRegGeneral,

    /// A trailing Instruciton (prefix opcode other than 0x0F or VEX)
    Insn,
    /// Vector Instruction (VEX prefix)
    VecInsn,
    /// Immediate value
    Imm(usize),

    /// Immediate value depending on prefix and mode (no REX)
    ImmGeneral,

    /// A relative Word with a given Instruction Instruction size
    Rel(usize),

    /// Sets bits in the R field of the ModR/M byte.
    RControlBits(u8),

    /// xmm/ymm/zmm depending on prefixes and control bits in prefixes
    AvxReg,

    /// m128/256/512 depending on prefixes and control bits in prefixes
    AvxMem,
}

macro_rules! define_x86_instructions{
    {
        $(($enum:ident, $mneomic:literal, $opcode:literal, [$($operand:expr),*] $(, [$($mode:ident),*] $(, [$($feature:ident),*])?)?)),* $(,)?
    } => {
        #[derive(Copy,Clone,Debug,Hash,PartialEq,Eq)]
        pub enum X86Opcode{
            $($enum,)*

            #[doc(hidden)]
            __Nonexhaustive
        }

        impl X86Opcode{
            pub fn opcode(&self) -> u64{
                match self{
                    $(Self::$enum => $opcode,)*
                    Self::__Nonexhaustive => unreachable!(),
                }
            }

            pub fn operands(&self) -> &'static [X86OperandType]{
                match self{
                    $(Self::$enum => &[$($operand),*],)*

                    Self::__Nonexhaustive => unreachable!()
                }
            }

        }
    }
}

use X86OperandType::*;
use X86RegisterClass::*;

define_x86_instructions! {
    (AddMR8,  "add", 0x00, [ModRM(Byte), Reg(Byte)]),
    (AddMR,   "add", 0x01, [ModRMGeneral, RegGeneral]),
    (AddRM8,  "add", 0x02, [Reg(Byte), ModRM(Byte)]),
    (AddRM,   "add", 0x03, [RegGeneral, ModRMGeneral]),
    (AddAImm8,"add", 0x04, [AReg(Byte), Imm(1)]),
    (AddAImm, "add", 0x05, [ARegGeneral, ImmGeneral]),
    (OrMR8,   "or",  0x08, [ModRM(Byte), Reg(Byte)]),
    (OrMR,    "or",  0x09, [ModRMGeneral, RegGeneral]),
    (OrRM8,   "or",  0x0A, [Reg(Byte), ModRM(Byte)]),
    (OrRM,    "or",  0x0B, [RegGeneral, ModRMGeneral]),
    (OrAImm8, "or",  0x0C, [AReg(Byte), Imm(1)]),
    (OArImm,  "or",  0x0D, [ARegGeneral, ImmGeneral]),
    (AdcMR8,  "adc", 0x10, [ModRM(Byte), Reg(Byte)]),
    (AdcMR,   "adc", 0x11, [ModRMGeneral, RegGeneral]),
    (AdcRM8,  "adc", 0x12, [Reg(Byte), ModRM(Byte)]),
    (AdcRM,   "adc", 0x13, [RegGeneral, ModRMGeneral]),
    (AdcAImm8,"adc", 0x14, [AReg(Byte), Imm(1)]),
    (AdcAImm, "adc", 0x15, [ARegGeneral, ImmGeneral]),
    (SbbMR8,  "sbb", 0x18, [ModRM(Byte), Reg(Byte)]),
    (SbbMR,   "sbb", 0x19, [ModRMGeneral, RegGeneral]),
    (SbbRM8,  "sbb", 0x1A, [Reg(Byte), ModRM(Byte)]),
    (SbbRM,   "sbb", 0x1B, [RegGeneral, ModRMGeneral]),
    (SbbAImm8,"sbb", 0x1C, [AReg(Byte), Imm(1)]),
    (SbbaImm, "sbb", 0x1D, [ARegGeneral, ImmGeneral]),
    (AndMR8,  "and", 0x20, [ModRM(Byte), Reg(Byte)]),
    (AndMR,   "and", 0x21, [ModRMGeneral, RegGeneral]),
    (AndRM8,  "and", 0x22, [Reg(Byte), ModRM(Byte)]),
    (AndRM,   "and", 0x23, [RegGeneral, ModRMGeneral]),
    (AndAImm8,"and", 0x24, [AReg(Byte), Imm(1)]),
    (AndAImm, "and", 0x25, [ARegGeneral, ImmGeneral]),
    (SubMR8,  "sub", 0x28, [ModRM(Byte), Reg(Byte)]),
    (SubMR,   "sub", 0x29, [ModRMGeneral, RegGeneral]),
    (SubRM8,  "sub", 0x2A, [Reg(Byte), ModRM(Byte)]),
    (SubRM,   "sub", 0x2B, [RegGeneral, ModRMGeneral]),
    (SubAImm8,"sub", 0x2C, [AReg(Byte), Imm(1)]),
    (SubAImm, "sub", 0x2D, [ARegGeneral, ImmGeneral]),
    (XorMR8,  "xor", 0x30, [ModRM(Byte), Reg(Byte)]),
    (XorMR,   "xor", 0x31, [ModRMGeneral, RegGeneral]),
    (XorRM8,  "xor", 0x32, [Reg(Byte), ModRM(Byte)]),
    (XorRM,   "xor", 0x33, [RegGeneral, ModRMGeneral]),
    (XorAImm8,"xor", 0x34, [AReg(Byte), Imm(1)]),
    (XorAImm, "xor", 0x35, [ARegGeneral, ImmGeneral]),
    (CmpMR8,  "cmp", 0x38, [ModRM(Byte), Reg(Byte)]),
    (CmpMR,   "cmp", 0x39, [ModRMGeneral, RegGeneral]),
    (CmpRM8,  "cmp", 0x3A, [Reg(Byte), ModRM(Byte)]),
    (CmpRM,   "cmp", 0x3B, [RegGeneral, ModRMGeneral]),
    (CmpAImm8,"cmp", 0x3C, [AReg(Byte), Imm(1)]),
    (CmpAImm, "cmp", 0x3D, [ARegGeneral, ImmGeneral]),
    (Rex,     "rex", 0x40, [Insn], [Long]),
    (RexB,    "rex.b",0x41, [Insn], [Long]),
    (RexX,    "rex.x",0x42, [Insn], [Long]),
    (RexXB,   "rex.xb",0x43, [Insn], [Long]),
    (RexR,    "rex.r", 0x44, [Insn], [Long]),
    (RexRB,   "rex.rb",0x45, [Insn], [Long]),
    (RexRX,   "rex.rx",0x46, [Insn], [Long]),
    (RexRXB,  "rex.rxb", 0x47, [Insn], [Long]),
    (RexW,    "rex.w", 0x48, [Insn], [Long]),
    (RexWB,   "rex.wb",0x49, [Insn], [Long]),
    (RexWX,   "rex.wx",0x4A, [Insn], [Long]),
    (RexWXB,  "rex.wxb",0x4B, [Insn], [Long]),
    (RexWR,   "rex.wr", 0x4C, [Insn], [Long]),
    (RexWRB,  "rex.wrb",0x4D, [Insn], [Long]),
    (RexWRX,  "rex.wrx",0x4E, [Insn], [Long]),
    (RexWRXB, "rex.wrxb", 0x4F, [Insn], [Long]),
    (Push,    "push",0x50, [OpRegMode]),
    (Pop,     "pop", 0x58, [OpRegMode]),
    (Movsxd,  "movsxd",0x63, [RegGeneral, ModRM(Double)]),
    (FsSeg,   "fs",  0x64, [Insn]),
    (GsSeg,   "gs",  0x65, [Insn]),
    (OpSize,  "66h", 0x66, [Insn]),
    (AddrSize,"67h", 0x67, [Insn]),
    (PushImm, "push",0x68, [ImmGeneral]),
    (IMul,    "imul",0x69, [RegGeneral, ModRMGeneral, ImmGeneral]),
    (PushImm8,"push",0x6A, [Imm(1)]),
    (IMul8,   "imul",0x6B, [RegGeneral, ModRMGeneral, Imm(1)]),
    (Insb,    "ins", 0x6C, [ModRM(Byte), DReg(Byte)]),
    (Ins,     "ins", 0x6D, [ModRMGeneral, DRegGeneral]),
    (Outsb,   "outs",0x6E, [DReg(Byte), ModRM(Byte)]),
    (Outs,    "outs",0x6F, [DRegGeneral, ModRMGeneral]),
    (Jo,      "jo",  0x70, [Rel(8)]),
    (Jno,     "jno", 0x71, [Rel(8)]),
    (Jb,      "jb",  0x72, [Rel(8)]),
    (Jae,     "jae", 0x73, [Rel(8)]),
    (Jz,      "jz",  0x74, [Rel(8)]),
    (Jnz,     "jnz", 0x75, [Rel(8)]),
    (Jbe,     "jbe", 0x76, [Rel(8)]),
    (Ja,      "ja",  0x77, [Rel(8)]),
    (Js,      "js",  0x78, [Rel(8)]),
    (Jns,     "jns", 0x79, [Rel(8)]),
    (Jp,      "jp",  0x7A, [Rel(8)]),
    (Jnp,     "jnp", 0x7B, [Rel(8)]),
    (Jl,      "jl",  0x7C, [Rel(8)]),
    (Jge,     "jge", 0x7D, [Rel(8)]),
    (Jle,     "jle", 0x7E, [Rel(8)]),
    (Jg,      "jg",  0x7F, [Rel(8)]),
    (AddImm8, "add", 0x80, [RControlBits(1), ModRM(Byte), Imm(8)]),
    (OrImm8,  "or",  0x80, [RControlBits(2), ModRM(Byte), Imm(8)]),
    (AdcImm8, "adc", 0x80, [RControlBits(3), ModRM(Byte), Imm(8)]),
    (SbbImm8, "sbb", 0x80, [RControlBits(4), ModRM(Byte), Imm(8)]),
    (AndImm8, "and", 0x80, [RControlBits(5), ModRM(Byte), Imm(8)]),
    (SubImm8, "sub", 0x80, [RControlBits(6), ModRM(Byte), Imm(8)]),
    (XorImm8, "xor", 0x80, [RControlBits(7), ModRM(Byte), Imm(8)]),
    (AddImm,  "add", 0x81, [RControlBits(1), ModRMGeneral, ImmGeneral]),
    (OrImm,   "or",  0x81, [RControlBits(2), ModRMGeneral, ImmGeneral]),
    (AdcImm,  "adc", 0x81, [RControlBits(3), ModRMGeneral, ImmGeneral]),
    (SbbImm,  "sbb", 0x81, [RControlBits(4), ModRMGeneral, ImmGeneral]),
    (AndImm,  "and", 0x81, [RControlBits(5), ModRMGeneral, ImmGeneral]),
    (SubImm,  "sub", 0x81, [RControlBits(6), ModRMGeneral, ImmGeneral]),
    (XorImm,  "xor", 0x81, [RControlBits(7), ModRMGeneral, ImmGeneral]),
    (AddGImm8,"add", 0x83, [RControlBits(1), ModRMGeneral, Imm(8)]),
    (OrGImm8, "or",  0x83, [RControlBits(2), ModRMGeneral, Imm(8)]),
    (AdcGImm8,"adc", 0x83, [RControlBits(3), ModRMGeneral, Imm(8)]),
    (SbbGImm8,"sbb", 0x83, [RControlBits(4), ModRMGeneral, Imm(8)]),
    (AndGImm8,"and", 0x83, [RControlBits(5), ModRMGeneral, Imm(8)]),
    (SubGImm8,"sub", 0x83, [RControlBits(6), ModRMGeneral, Imm(8)]),
    (XorGImm8,"xor", 0x83, [RControlBits(7), ModRMGeneral, Imm(8)]),
    (Test8,   "test",0x84, [ModRM(Byte),Reg(Byte)]),
    (Test,    "test",0x85, [ModRMGeneral, RegGeneral]),
    (Xchg8,   "xchg",0x86, [Reg(Byte), ModRM(Byte)]),
    (Xchg,    "xchg",0x87, [RegGeneral, ModRMGeneral]),
    (MovMR8,  "mov", 0x88, [ModRM(Byte),Reg(Byte)]),
    (MovMR,   "mov", 0x89, [ModRMGeneral, RegGeneral]),
    (MovRM8,  "mov", 0x8A, [Reg(Byte), ModRM(Byte)]),
    (MovRM,   "mov", 0x8B, [RegGeneral, ModRMGeneral]),
    (MovRMS,  "mov", 0x8C, [ModRMGeneral, Reg(Sreg)]),
    (Lea,     "lea", 0x8D, [RegGeneral, ModRMGeneral]),
    (MovSRM,  "mov", 0x8E, [Reg(Sreg), ModRM(Word)]),
    (PopMR,   "pop", 0x8F, [ModRMMode]),
    (Nop90,   "nop", 0x90, []),
    (XchgReg, "xchg",0x90, [OpRegGeneral,ARegGeneral]),
    (Cbw, "cbw", 0x98, [AReg(Word), AReg(Byte)]),
    (Cwde, "cwde", 0x98, [AReg(Double), AReg(Word)]),
    (Cdqe, "cdqe", 0x98, [AReg(Quad), AReg(Word)]),
    (Cwd, "cwd", 0x99, [DReg(Word), AReg(Word)]),
    (Cdq, "cdq", 0x99, [DReg(Double), DReg(Double)]),
    (Cdo, "cqo", 0x99, [DReg(Quad),DReg(Quad)]),
    (Fwait, "fwait", 0x9B, [])
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum ModRMRegOrSib {
    Reg(X86Register),
    Abs(Address),
    RipRel(Address),
    Sib {
        scale: u32,
        index: X86Register,
        base: X86Register,
    },
    Index16 {
        base: X86Register,
        index: X86Register,
    },
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum ModRM {
    Indirect {
        reg: u8,
        mode: ModRMRegOrSib,
    },
    IndirectDisp8 {
        reg: u8,
        mode: ModRMRegOrSib,
        disp8: i8,
    },
    IndirectDisp32 {
        reg: u8,
        mode: ModRMRegOrSib,
        disp32: i32,
    },
    Direct(X86Register),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum X86Operand {
    Register(X86Register),
    ModRM(ModRM),
    Immediate(u64),
}

#[derive(Clone, Debug)]
pub struct X86Instruction {
    opc: X86Opcode,
    operands: Vec<X86Operand>,
}

impl X86Instruction {
    pub fn new(opc: X86Opcode, operands: Vec<X86Operand>) -> Self {
        Self { opc, operands }
    }

    pub fn opcode(&self) -> X86Opcode {
        self.opc
    }

    pub fn operands(&self) -> &[X86Operand] {
        &self.operands
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct X86Encoder<W> {
    writer: W,
    mode: X86Mode,
}

impl<W> X86Encoder<W> {
    pub const fn new(writer: W, defmode: X86Mode) -> Self {
        Self {
            writer,
            mode: defmode,
        }
    }

    pub fn into_inner(self) -> W {
        self.writer
    }

    pub fn writer_mut(&mut self) -> &mut W {
        &mut self.writer
    }

    pub fn mode(&self) -> X86Mode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: X86Mode) {
        self.mode = mode
    }
}

impl<W: InsnWrite> X86Encoder<W> {
    pub fn write_insn(&mut self, insn: X86Instruction) -> std::io::Result<()> {
        let opcode = insn.opc;
        let _opval = opcode.opcode();
        todo!()
    }
}
