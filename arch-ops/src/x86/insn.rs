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
    EVex {
        rex: u8,
        next: u16,
        len: u8,
        size_prefix: u8,
        src2: u8,
        mask: u8,
    },
    PackedDouble,
    ScalarSingle,
    ScalarDouble,
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
    /// The Mod and R/M portions of a ModR/M Byte, except that no registers cannot be used and size checking should not be performed
    ModRMMem,
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

    /// Either STi or a memory reference with a given size
    ModRMReal(X86RegisterClass),

    /// A register encoded in the r/m field of the ModRM byte
    ModRMReg(X86RegisterClass),

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

    CReg(X86RegisterClass),

    CRegGeneral,

    Flags(X86RegisterClass),

    FlagsMode,

    /// A trailing Instruciton (prefix opcode other than 0x0F or VEX)
    Insn,
    /// Vector Instruction (VEX prefix)
    VecInsn,
    /// Immediate value
    Imm(usize),

    /// Immediate value depending on prefix and mode (no REX.W)
    ImmGeneral,

    /// Immediate value depending on prefix and mode (respects REX.W in 64-bit mode)
    ImmGeneralWide,

    /// A relative Word with a given Instruction Address size
    Rel(usize),

    /// A relative address with size given by mode and 66h prefix
    RelGeneral,

    /// Sets bits in the R field of the ModR/M byte.
    RControlBits(u8),

    // 16-bit segment value or GDT offset
    Seg,

    /// xmm/ymm/zmm depending on prefixes and control bits in prefixes
    AvxReg,

    /// m128/256/512 depending on prefixes and control bits in prefixes
    AvxMem,

    Moff(X86RegisterClass),
    MoffGeneral,

    /// Memory Reference referring to the destination address, which is always es:eDI or rDI depending on mode.
    /// In legacy mode, es is not controlled by any segment override prefix
    MemDest(X86RegisterClass),
    /// m16/32/64, which is always es:eDI or rDI depending on mode. In legacy mode, es is not controlled by any segment override prefix
    MemDestGeneral,
    /// Memory Reference referring to the source address, which is always ds:eSI or rSI depending on mode.
    /// In legacy mode, ds is controlled by the segment override prefix.
    MemSrc(X86RegisterClass),
    // m16/32/64, which is always ds:eSI or rSI depending on mode. In legacy mode, ds is controlled by the segment override prefix
    MemSrcGeneral,
}

macro_rules! define_x86_instructions{
    {
        $(($enum:ident, $mneomic:literal, $opcode:literal, [$($operand:expr),*] $(, [$($mode:ident),*] $(, [$($feature:ident),*])?)?)),* $(,)?
    } => {
        #[derive(Copy,Clone,Debug,Hash,PartialEq,Eq)]
        #[non_exhaustive]
        pub enum X86Opcode{
            $($enum,)*
        }

        impl X86Opcode{
            pub fn opcode(&self) -> u64{
                match self{
                    $(Self::$enum => $opcode,)*
                }
            }

            pub fn operands(&self) -> &'static [X86OperandType]{
                match self{
                    $(Self::$enum => &[$($operand),*],)*
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
    (OrAImm,  "or",  0x0D, [ARegGeneral, ImmGeneral]),
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
    (Fwait, "fwait", 0x9B, []),
    (Pushf, "pushf", 0x9C, [FlagsMode]),
    (Popf, "popf", 0x9D, [FlagsMode]),
    (Sahf,"sahf",0x9E, []),
    (Lahf, "lahf", 0x9F, []),
    (MovAlM, "mov", 0xA0, [AReg(Byte), Moff(Byte)]),
    (MovAregM, "mov", 0xA1, [ARegGeneral, MoffGeneral]),
    (MovMAl, "mov", 0xA2, [Moff(Byte), AReg(Byte)]),
    (MovMAreg, "mov", 0xA3, [MoffGeneral, ARegGeneral]),
    (Movsb,"movs", 0xA4, [MemDest(Byte),MemSrc(Byte)]),
    (Movs, "movs", 0xA5, [MemDestGeneral, MemSrcGeneral]),
    (Cmpsb, "cmps", 0xA6, [MemDest(Byte),MemSrc(Byte)]),
    (Cmps, "cmps", 0xA7, [MemDestGeneral, MemSrcGeneral]),
    (TestAreg8, "test", 0xA8, [AReg(Byte),Imm(8)]),
    (TestAreg, "test", 0xA9, [ARegGeneral,ImmGeneral]),
    (Stosb, "stos", 0xAA, [MemDest(Byte),AReg(Byte)]),
    (Stos, "stos", 0xAB,[MemDestGeneral,ARegGeneral]),
    (Lodsb, "lods",0xAC, [AReg(Byte),MemSrc(Byte)]),
    (Lods, "lods", 0xAD, [ARegGeneral, MemSrcGeneral]),
    (Scasb, "scas", 0xAE, [MemDest(Byte), AReg(Byte)]),
    (Scas, "scas", 0xAF, [MemDestGeneral, ARegGeneral]),
    (MovImm8, "mov", 0xB0, [OpReg(Byte),Imm(8)]),
    (MovImm, "mov", 0xB8, [OpRegGeneral, ImmGeneralWide]),
    (RolImm8, "rol", 0xC0, [RControlBits(0),ModRM(Byte),Imm(8)]),
    (RorImm8, "ror", 0xC0, [RControlBits(1),ModRM(Byte),Imm(8)]),
    (RclImm8, "rcl", 0xC0, [RControlBits(2),ModRM(Byte),Imm(8)]),
    (RcrImm8, "rcr", 0xC0, [RControlBits(3),ModRM(Byte), Imm(8)]),
    (ShlImm8, "shl", 0xC0, [RControlBits(4), ModRM(Byte), Imm(8)]),
    (ShrImm8, "shr", 0xC0, [RControlBits(5),ModRM(Byte),Imm(8)]),
    (Shl2Imm8, "shl", 0xC0, [RControlBits(6), ModRM(Byte), Imm(8)]),
    (SarImm8, "sar", 0xC0, [RControlBits(7), ModRM(Byte), Imm(8)]),
    (RolImm, "rol", 0xC1, [RControlBits(0), ModRMGeneral,Imm(8)]),
    (RorImm, "ror", 0xC1, [RControlBits(1), ModRMGeneral,Imm(8)]),
    (RclImm, "rcl", 0xC1, [RControlBits(2), ModRMGeneral,Imm(8)]),
    (RcrImm, "rcr", 0xC1, [RControlBits(3), ModRMGeneral, Imm(8)]),
    (ShlImm, "shl", 0xC1, [RControlBits(4), ModRMGeneral, Imm(8)]),
    (ShrImm, "shr", 0xC1, [RControlBits(5), ModRMGeneral,Imm(8)]),
    (Shl2Imm, "shl", 0xC1, [RControlBits(6), ModRMGeneral, Imm(8)]),
    (SarImm, "sar", 0xC1, [RControlBits(7), ModRMGeneral, Imm(8)]),
    (RetnPop, "retn", 0xC2, [Imm(16)]),
    (Retn, "retn", 0xC3, []),
    (MovImmM8, "mov", 0xC6, [RControlBits(0),ModRM(Byte), Imm(8)]),
    (MovImmM, "mov", 0xC7, [RControlBits(0),ModRMGeneral, ImmGeneral]),
    (Enter, "enter", 0xC8, [Imm(16),Imm(8)]),
    (Leave,"leave", 0xC9, []),
    (RetfPop, "retf", 0xCA, [Imm(16)]),
    (Retf, "retf", 0xCB, []),
    (Int3, "int3", 0xCC, []),
    (Int,"int", 0xCD, [Imm(8)]),
    (IntO, "into",0xCE,[]),
    (IRet,"iret",0xCF, []),
    (Rol8, "rol", 0xD0, [RControlBits(0),ModRM(Byte)]),
    (Ror8, "ror", 0xD0, [RControlBits(1),ModRM(Byte)]),
    (Rcl8, "rcl", 0xD0, [RControlBits(2),ModRM(Byte)]),
    (Rcr8, "rcr", 0xD0, [RControlBits(3),ModRM(Byte)]),
    (Shl8, "shl", 0xD0, [RControlBits(4), ModRM(Byte)]),
    (Shr8, "shr", 0xD0, [RControlBits(5),ModRM(Byte)]),
    (Shl2_8, "shl", 0xD0, [RControlBits(6), ModRM(Byte)]),
    (Sar8, "sar", 0xD0, [RControlBits(7), ModRM(Byte)]),
    (Rol, "rol", 0xD1, [RControlBits(0), ModRMGeneral]),
    (Ror, "ror", 0xD1, [RControlBits(1), ModRMGeneral]),
    (Rcl, "rcl", 0xD1, [RControlBits(2), ModRMGeneral]),
    (Rcr, "rcr", 0xD1, [RControlBits(3), ModRMGeneral]),
    (Shl, "shl", 0xD1, [RControlBits(4), ModRMGeneral]),
    (Shr, "shr", 0xD1, [RControlBits(5), ModRMGeneral]),
    (Shl2, "shl", 0xD1, [RControlBits(6), ModRMGeneral]),
    (Sar, "sar", 0xD1, [RControlBits(7), ModRMGeneral]),
    (RolCL8, "rol", 0xD2, [RControlBits(0),  ModRM(Byte) , CReg(Byte)]),
    (RorCL8, "ror", 0xD2, [RControlBits(1),  ModRM(Byte) , CReg(Byte)]),
    (RclCL8, "rcl", 0xD2, [RControlBits(2),  ModRM(Byte) , CReg(Byte)]),
    (RcrCL8, "rcr", 0xD2, [RControlBits(3),  ModRM(Byte) , CReg(Byte)]),
    (ShlCL8, "shl", 0xD2, [RControlBits(4),  ModRM(Byte) , CReg(Byte)]),
    (ShrCL8, "shr", 0xD2, [RControlBits(5),  ModRM(Byte) , CReg(Byte)]),
    (Shl2CL8, "shl", 0xD2, [RControlBits(6), ModRM(Byte) , CReg(Byte)]),
    (SarCL8, "sar", 0xD2, [RControlBits(7),  ModRM(Byte) , CReg(Byte)]),
    (RolCL, "rol", 0xD3, [RControlBits(0),   ModRMGeneral, CReg(Byte)]),
    (RorCL, "ror", 0xD3, [RControlBits(1),   ModRMGeneral, CReg(Byte)]),
    (RclCL, "rcl", 0xD3, [RControlBits(2),   ModRMGeneral, CReg(Byte)]),
    (RcrCL, "rcr", 0xD3, [RControlBits(3),   ModRMGeneral, CReg(Byte)]),
    (ShlCL, "shl", 0xD3, [RControlBits(4),   ModRMGeneral, CReg(Byte)]),
    (ShrCL, "shr", 0xD3, [RControlBits(5),   ModRMGeneral, CReg(Byte)]),
    (Shl2CL, "shl", 0xD3, [RControlBits(6),  ModRMGeneral, CReg(Byte)]),
    (SarCL, "sar", 0xD3, [RControlBits(7),   ModRMGeneral, CReg(Byte)]),
    (Xlat, "xlat", 0xD7, [ModRM(Byte)]),
    // Note: Double here is still X86RegisterClass::Double (dword), as in a 32-byte memory reference (m32real), not a double precision floating-point value
    (Fadd, "fadd", 0xD8, [RControlBits(0), ModRMReal(Double)]),
    (Fmul, "fmul", 0xD8, [RControlBits(1), ModRMReal(Double)]),
    (Fcom, "fcom", 0xD8, [RControlBits(2),ModRMReal(Double)]),
    (Fcomp, "fcomp",0xD8, [RControlBits(3), ModRMReal(Double)]),
    (Fsub, "fsub", 0xD8, [RControlBits(4), ModRMReal(Double)]),
    (Fsubr, "fsubr", 0xD8, [RControlBits(5), ModRMReal(Double)]),
    (Fdiv, "fdiv", 0xD8, [RControlBits(6), ModRMReal(Double)]),
    (Fdivr, "fdivr", 0xD8, [RControlBits(7), ModRMReal(Double)]),
    (Fld, "fld", 0xD9, [RControlBits(0), ModRMReal(Double)]),
    (Fxch, "fxch", 0xD9, [RControlBits(1),ModRM(St)]),
    (Fst, "fst",0xD9, [RControlBits(2),ModRMReal(Double)]),
    (Fstp, "fstp", 0xD9, [RControlBits(3), ModRMReal(Double)]),
    (Fldenv, "fldenv",0xD9, [RControlBits(4),ModRMMem]),
    (Fchs, "fchs", 0xD9E0, []),
    (Fabs, "fabs", 0xD9E1, []),
    (Ftst, "ftst", 0xD9E4, []),
    (Fxam, "fxam", 0xD9E5, []),
    (Fldcw, "fldcw", 0xD9, [RControlBits(5),ModRMMem]),
    (Fld1, "fld1", 0xD9E8,[]),
    (FldL2T, "fldl2t", 0xD9E9, []),
    (FldL2E, "fldl2e", 0xD9EA, []),
    (FldPi, "fldpi", 0xD9EB, []),
    (Fldlg2, "fldlg2", 0xD9EC, []),
    (Fldln2, "fldln2", 0xD9ED, []),
    (Fldz, "fldz", 0xD9EE, []),
    (Fnstenv, "fnstenv", 0xD9, [RControlBits(6), ModRMMem]),
    (Fstenv, "fstenv", 0x9BD9, [RControlBits(6), ModRMMem]),
    (F2Xm1, "f2xm1", 0xD9F0, []),
    (FYl2x, "fyl2x", 0xD9F1, []),
    (Fptan, "fptan", 0xD9F2, []),
    (Fpatan, "fpatan", 0xD9F3, []),
    (Fxtract, "fxtract", 0xD9F4, []),
    (Fprem1, "fprem1", 0xD9F5, []),
    (FDecStp, "fdecstp", 0xD9F6, []),
    (FIncStp, "fincstp", 0xD9F7, []),
    (Fnstcw, "fnstcw", 0xD9, [RControlBits(6), ModRMMem]),
    (Fstcw, "fstcw", 0x9BD9, [RControlBits(6), ModRMMem]),
    (Fprem, "fprem", 0xD9F8, []),
    (FYl2Xp1, "fyl2xp1", 0xD9F9, []),
    (Fsqrt, "fsqrt", 0xD9FA, []),
    (Fsincos, "fsincos", 0xD9FB, []),
    (Frndint, "frndint", 0xD9FC, []),
    (Fscale, "fscale", 0xD9FD, []),
    (Fsin, "fsin", 0xD9FE, []),
    (Fcos, "fcos", 0xD9FF, []),
    (FCmovB, "fcmovb", 0xDA, [RControlBits(0),ModRMReg(St)]),
    (FIadd, "fiadd", 0xDA, [RControlBits(0),ModRMReal(Double)]),
    (FCmovE, "fcmove", 0xDA, [RControlBits(1), ModRMReg(St)]),
    (FImul, "fimul", 0xDA, [RControlBits(1), ModRMReal(Double)]),
    (FCmovBE, "fcmovbe", 0xDA, [RControlBits(2),ModRMReg(St)]),
    (FIcom, "ficom", 0xDA, [RControlBits(2), ModRMReal(Double)]),
    (FCmovU, "fcmovu", 0xDA, [RControlBits(3), ModRMReg(St)]),
    (FIcomp, "ficomp", 0xDA, [RControlBits(3), ModRMReal(Double)]),
    (FISub, "fisub", 0xDA, [RControlBits(4), ModRMReal(Double)]),
    (FUcompp, "fucompp", 0xDA, [RControlBits(5), ModRMReg(St)]),
    (FISubr, "fisubr", 0xDA, [RControlBits(5), ModRMReal(Double)]),
    (FIdiv, "fidiv", 0xDA, [RControlBits(6), ModRMReal(Double)]),
    (FIdivr, "fidivr", 0xDA, [RControlBits(7), ModRMReal(Double)]),
    (FCmovNb, "fcmovnb", 0xDA, [RControlBits(0), ModRMReg(St)]),
    (FIld, "fild", 0xDB, [RControlBits(0), ModRMReal(Double)]),
    (FCmovNe, "fcmovne", 0xDB, [RControlBits(1), ModRMReg(St)]),
    (FISttp, "fisttp", 0xDB, [RControlBits(1), ModRMReal(Double)]),
    (FCmovNbe, "fcmovnbe", 0xDB, [RControlBits(2), ModRMReg(St)]),
    (FIst, "fist", 0xDB, [RControlBits(2), ModRMReal(Double)]),
    (FCMovNu, "fcmovnu", 0xDB, [RControlBits(3), ModRMReg(St)]),
    (FIstp, "fistp", 0xDB, [RControlBits(3), ModRMReal(Double)]),
    (Fneni, "fneni", 0xDBE0, []),
    (Fndisi, "fndisi", 0xDBE1, []),
    (Fnclex, "fnclex", 0xDBE2, []),
    (Fclex, "fclex", 0x9BDBE2, []),
    (Fninit, "fninit", 0xDBE3, []),
    (Finit, "finit", 0x9BDBE3, []),
    (Fnsetpm, "fnsetpm", 0xDBE4, []),
    (Fld80, "fld", 0xDB, [RControlBits(5), ModRMReal(St)]),
    (Fucmpi, "fucmpi", 0xDB, [RControlBits(6), ModRMReg(St)]),
    (Fcmpi, "fcmpi", 0xDB, [RControlBits(7), ModRMReg(St)]),
    (Fadd64, "fadd", 0xDC, [RControlBits(0), ModRMReal(Quad)]),
    (Fmul64, "fmul", 0xDC, [RControlBits(1), ModRMReal(Quad)]),
    (Fcom64, "fcom", 0xDC, [RControlBits(2), ModRMReal(Quad)]),
    (Fcomp64, "fcomp", 0xDC, [RControlBits(3), ModRMReal(Quad)]),
    (Fsub64, "fsub", 0xDC, [RControlBits(4), ModRMReal(Quad)]),
    (Fsubr64, "fsubr", 0xDC, [RControlBits(5), ModRMReal(Quad)]),
    (Fdiv64, "fdiv", 0xDC, [RControlBits(6), ModRMReal(Quad)]),
    (Fdivr64, "fdivr",0xDC, [RControlBits(7), ModRMReal(Quad)]),
    (LoopNz, "loopnz", 0xE0, [CRegGeneral, Rel(8)]),
    (LoopZ, "loopz", 0xE1, [CRegGeneral, Rel(8)]),
    (Loop, "loop", 0xE2, [CRegGeneral, Rel(8)]),
    (Jcxz, "jcxz", 0xE3, [CRegGeneral, Rel(8)]),
    (InImm8, "in", 0xE4, [AReg(Byte), Imm(8)]),
    (InImm, "in", 0xE5, [ARegGeneral, Imm(8)]),
    (OutImm8, "out", 0xE6, [AReg(Byte),Imm(8)]),
    (OutImm, "out", 0xE7, [ARegGeneral, Imm(8)]),
    (Call, "call", 0xE8, [RelGeneral]),
    (Jmp, "jmp", 0xE9, [RelGeneral]),
    (Jmpf, "jmpf", 0xEA, [Seg, RelGeneral]),
    (Jmp8, "jmp", 0xEB, [Rel(8)]),
    (In8, "in", 0xEC, [AReg(Byte), DReg(Word)]),
    (In, "in", 0xED, [ARegGeneral, DReg(Word)]),
    (Out8, "out", 0xEE, [AReg(Byte), DReg(Word)]),
    (Out, "out", 0xEF, [ARegGeneral, DReg(Word)]),
    (Int1, "int1", 0xF1, []),
    (Hlt, "hlt", 0xF4, []),
    (Cmc, "cmc", 0xF5, []),
    (TestImm8, "test", 0xF6, [RControlBits(0), ModRM(Byte), Imm(8)]),
    (TestImm82, "test", 0xF6, [RControlBits(0), ModRM(Byte), Imm(8)]),
    (Not8, "not", 0xF6, [RControlBits(2), ModRM(Byte)]),
    (Neg8, "neg", 0xF6, [RControlBits(3), ModRM(Byte)]),
    (MulAl8, "mul",0xF6, [RControlBits(4), ModRM(Byte)]),
    (IMulAl8, "imul", 0xF6, [RControlBits(5), ModRM(Byte)]),
    (DivAx8, "div", 0xF6, [RControlBits(6), ModRM(Byte)]),
    (IDivAx8, "idiv", 0xF6, [RControlBits(7), ModRM(Byte)]),
    (TestImm, "test", 0xF7, [RControlBits(0), ModRMGeneral, ImmGeneral]),
    (TestImm2, "test", 0xF7, [RControlBits(0), ModRMGeneral, ImmGeneral]),
    (Not, "not", 0xF7, [RControlBits(2), ModRMGeneral]),
    (Neg, "neg", 0xF7, [RControlBits(3), ModRMGeneral]),
    (MulAreg, "mul",0xF7, [RControlBits(4), ARegGeneral, DRegGeneral, ModRMGeneral]),
    (IMulAreg, "imul", 0xF7, [RControlBits(5), ARegGeneral, DRegGeneral, ModRMGeneral]),
    (Div, "div", 0xF7, [RControlBits(6), ARegGeneral, DRegGeneral, ModRMGeneral]),
    (IDiv, "idiv", 0xF7, [RControlBits(7), ARegGeneral, DRegGeneral, ModRMGeneral]),
    (Clc, "clc", 0xF8, []),
    (Stc, "stc", 0xF9, []),
    (Cli, "cli", 0xFA, []),
    (Sti, "sti", 0xFB, []),
    (Cld, "cld", 0xFC, []),
    (Std, "std", 0xFD, []),
    (Inc8, "inc", 0xFE, [RControlBits(0), ModRM(Byte)]),
    (Dec8, "dec", 0xFE, [RControlBits(1), ModRM(Byte)]),
    (Inc, "inc", 0xFF, [RControlBits(0), ModRMGeneral]),
    (Dec, "dec", 0xFF, [RControlBits(1), ModRMGeneral]),
    (CallInd, "call", 0xFF, [RControlBits(2), ModRMGeneral]),
    (Callf, "callf", 0xFF, [RControlBits(3), ModRMMem]),
    (JmpInd, "jmp", 0xFF, [RControlBits(4), ModRMGeneral]),
    (JmpfInd, "jmp", 0xFF, [RControlBits(5), ModRMMem]),
    (PushRM, "push", 0xFF, [RControlBits(6), ModRMMode]),
    (Ud2, "ud2", 0x0F0B, []),

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
    RelAddr(Address),
    AbsAddr(Address),
}

#[derive(Clone, Debug)]
pub struct X86Instruction {
    opc: X86Opcode,
    operands: Vec<X86Operand>,
}

impl X86Instruction {
    pub const fn new(opc: X86Opcode, operands: Vec<X86Operand>) -> Self {
        Self { opc, operands }
    }

    pub const fn opcode(&self) -> X86Opcode {
        self.opc
    }

    pub fn operands(&self) -> &[X86Operand] {
        &self.operands
    }
}

macro_rules! zop_insns{
    [$($name:ident),* $(,)?] => {
        #[allow(non_upper_case_globals)]
        impl X86Instruction{
            $(pub const $name: X86Instruction = X86Instruction::new(X86Opcode:: $name, Vec::new());)*
        }
    }
}

zop_insns! {
    Nop90,
    Retn,
    Leave,
    Retf,
    IRet,
    Fneni,
    Fndisi,
    Hlt,
    Cmc,
    Clc,
    Stc,
    Cli,
    Sti,
    Cld,
    Std,
    Ud2,
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
        let opcode_long = insn.opcode().opcode();
        let mut opcode = if opcode_long < 0x100 {
            vec![opcode_long as u8]
        } else if opcode_long < 0x10000 {
            vec![(opcode_long >> 8) as u8, opcode_long as u8]
        } else if opcode_long < 0x1000000 {
            vec![
                (opcode_long >> 16) as u8,
                (opcode_long >> 8) as u8,
                opcode_long as u8,
            ]
        } else {
            vec![
                (opcode_long >> 24) as u8,
                (opcode_long >> 16) as u8,
                (opcode_long >> 8) as u8,
                opcode_long as u8,
            ]
        };
        match insn.opcode().operands() {
            [] => {
                assert!(insn.operands.is_empty());
                self.writer.write_all(&opcode)
            }
            [Imm(1)] | [Rel(8)] => {
                assert_eq!(insn.operands.len(), 1);
                self.writer.write_all(&opcode)?;
                self.writer.write_all(&[match insn.operands()[0] {
                    X86Operand::Immediate(x) => x as u8,
                    _ => panic!(),
                }])
            }
            [Insn] => todo!(),
            [ModRM(Byte), Reg(Byte)] => {
                assert_eq!(insn.operands.len(), 2);
                let mut rex = false;
                let b = match insn.operands()[0] {
                    X86Operand::ModRM(ModRM::Direct(reg)) => {
                        if matches!(reg.class(), X86RegisterClass::ByteRex) {
                            rex = true;
                        }
                        reg.regnum() >= 8
                    }
                    _ => todo!(),
                };
                if b {
                    rex = true;
                }
                let (r, reg) = match insn.operands()[1] {
                    X86Operand::Register(reg) => {
                        if matches!(reg.class(), X86RegisterClass::ByteRex) {
                            rex = true;
                        }
                        (reg.regnum() >= 8, reg.regnum() % 7)
                    }
                    _ => todo!(),
                };
                if r {
                    rex = true;
                }
                if rex {
                    let rex = 0x40 | if b { 0x01 } else { 0x00 } | if r { 0x04 } else { 0x00 };
                    self.writer.write_all(&[rex])?;
                }
                self.writer.write_all(&opcode)?;
                match insn.operands()[0] {
                    X86Operand::ModRM(ModRM::Direct(regrm)) => {
                        self.writer.write_all(&[0xC0 + (reg << 3) + regrm.regnum()])
                    }
                    _ => todo!(),
                }
            }
            [ModRMGeneral, RegGeneral] => {
                assert_eq!(insn.operands.len(), 2);
                let mut short = false;
                let (b, w) = match insn.operands()[0] {
                    X86Operand::ModRM(ModRM::Direct(reg)) => {
                        if matches!(reg.class(), X86RegisterClass::Word) {
                            short = true;
                        }
                        (
                            reg.regnum() >= 8,
                            matches!(reg.class(), X86RegisterClass::Quad),
                        )
                    }
                    _ => todo!(),
                };
                let mut rex = b || w;
                let (r, reg) = match insn.operands()[1] {
                    X86Operand::Register(reg) => (reg.regnum() >= 8, reg.regnum() % 7),
                    _ => todo!(),
                };
                if r {
                    rex = true;
                }
                if short {
                    self.writer.write_all(&[0x66])?;
                }
                if rex {
                    let rex = 0x40
                        | if b { 0x01 } else { 0x00 }
                        | if r { 0x04 } else { 0x00 }
                        | if w { 0x08 } else { 0x00 };
                    self.writer.write_all(&[rex])?;
                }
                self.writer.write_all(&opcode)?;
                match insn.operands()[0] {
                    X86Operand::ModRM(ModRM::Direct(regrm)) => {
                        self.writer.write_all(&[0xC0 + (reg << 3) + regrm.regnum()])
                    }
                    _ => todo!(),
                }
            }
            [OpRegMode] => {
                assert_eq!(insn.operands.len(), 1);
                let reg = match insn.operands()[0] {
                    X86Operand::Register(reg) => reg,
                    _ => panic!(),
                };
                if matches!(reg.class(), X86RegisterClass::Word) {
                    self.writer.write_all(&[0x66])?; // 16-bit operand override
                }
                if reg.regnum() >= 8 {
                    self.writer.write_all(&[0x41])?; // REX.B
                }
                let len = opcode.len();
                opcode[len - 1] += reg.regnum() % 7;
                self.writer.write_all(&opcode)
            }
            [OpRegGeneral, ImmGeneral] => {
                assert_eq!(insn.operands.len(), 2);
                let reg = match insn.operands()[0] {
                    X86Operand::Register(reg) => reg,
                    _ => panic!(),
                };

                let imm = match insn.operands()[0] {
                    X86Operand::Immediate(val) => val,
                    _ => panic!(), // TODO: We should support addresses here as well
                };

                let mut rex = None;
                let mut immsz = 4;
                if matches!(reg.class(), X86RegisterClass::Word) {
                    immsz = 2;
                    if matches!(reg.class(), X86RegisterClass::Word) {
                        self.writer.write_all(&[0x66])?; // 16-bit operand override
                    }
                } else if matches!(reg.class(), X86RegisterClass::Quad) {
                    rex = Some(0x48); // rex.w
                }
                if reg.regnum() >= 8 {
                    rex = if let Some(rex) = rex {
                        Some(rex | 0x01)
                    } else {
                        Some(0x41)
                    }
                }
                if let Some(rex) = rex {
                    self.writer.write_all(&[rex])?;
                }
                let len = opcode.len();
                opcode[len - 1] += reg.regnum() % 7;
                self.writer.write_all(&opcode)?;
                let bytes = imm.to_le_bytes();
                self.writer.write_all(&bytes[..immsz])
            }
            [OpRegGeneral, ImmGeneralWide] => {
                assert_eq!(insn.operands.len(), 2);
                let reg = match insn.operands()[0] {
                    X86Operand::Register(reg) => reg,
                    _ => panic!(),
                };

                let imm = match insn.operands()[1] {
                    X86Operand::Immediate(val) => val,
                    _ => panic!(), // TODO: We should support addresses here as well
                };

                let mut rex = None;
                let mut immsz = 4;
                if matches!(reg.class(), X86RegisterClass::Word) {
                    immsz = 2;
                    if matches!(reg.class(), X86RegisterClass::Word) {
                        self.writer.write_all(&[0x66])?; // 16-bit operand override
                    }
                } else if matches!(reg.class(), X86RegisterClass::Quad) {
                    immsz = 8;
                    rex = Some(0x48); // rex.w
                }
                if reg.regnum() >= 8 {
                    rex = if let Some(rex) = rex {
                        Some(rex | 0x01)
                    } else {
                        Some(0x41)
                    }
                }
                if let Some(rex) = rex {
                    self.writer.write_all(&[rex])?;
                }
                let len = opcode.len();
                opcode[len - 1] += reg.regnum() % 7;
                self.writer.write_all(&opcode)?;
                let bytes = imm.to_le_bytes();
                self.writer.write_all(&bytes[..immsz])
            }
            m => panic!("Unsupported Addressing Mode {:?}", m),
        }
    }
}
