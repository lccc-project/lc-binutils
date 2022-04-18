use std::io::{Read, Write};

use crate::traits::{Address, InsnRead, InsnWrite, Reloc};

use super::{X86Register, X86RegisterClass};

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Prefix {
    Lock,
    Rep,
    Repnz,
    Repz,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum X86Mode {
    Real,
    Protected,
    Virtual8086,
    Compatibility,
    Long,
}

impl X86Mode {
    pub fn default_mode_for(target: &Target) -> Option<X86Mode> {
        match target.arch() {
            target_tuples::Architecture::I86 => Some(X86Mode::Real),
            target_tuples::Architecture::I8086 => Some(X86Mode::Real),
            target_tuples::Architecture::I086 => Some(X86Mode::Real),
            target_tuples::Architecture::I186 => Some(X86Mode::Real),
            target_tuples::Architecture::I286 => Some(X86Mode::Real),
            target_tuples::Architecture::I386 => Some(X86Mode::Protected),
            target_tuples::Architecture::I486 => Some(X86Mode::Protected),
            target_tuples::Architecture::I586 => Some(X86Mode::Protected),
            target_tuples::Architecture::I686 => Some(X86Mode::Protected),
            target_tuples::Architecture::X86_64 => Some(X86Mode::Long),
            _ => None,
        }
    }

    pub fn largest_gpr(&self) -> X86RegisterClass {
        match self {
            Self::Real | Self::Virtual8086 => X86RegisterClass::Word,
            Self::Protected | Self::Compatibility => X86RegisterClass::Double,
            Self::Long => X86RegisterClass::Quad,
        }
    }
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

    /// A memory operand that must be encoded in an SIB byte
    ModRMSib(X86RegisterClass),

    /// Either STi or a memory reference with a given size
    ModRMReal(X86RegisterClass),

    /// A register encoded in the r/m field of the ModRM byte
    ModRMReg(X86RegisterClass),

    // xmm/m32/64
    ModRMScalar(X86RegisterClass),

    // xmm/ymm/zmm
    VexReg,
    // xmm/ymm/zmm/m128/m256/m512
    VexModRM,
    // x/y/zmm/m32/64
    VexModRMScalar(X86RegisterClass),

    // Instruction uses VEX/EVEX encoding, even if no vector registers are present
    // Class indicates the Size field for the VEX/EVEX prefix (EVEX forced if [`X86RegisterClass::Zmm`])
    VexPrefix(X86RegisterClass),

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

macro_rules! define_x86_instructions {
    {
        $(($enum:ident, $mnemonic:literal, $opcode:literal, [$($operand:expr),*] $(, [$($mode:ident),*] $(, [$($feature:ident),*])?)?)),* $(,)?
    } => {
        #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
        #[non_exhaustive]
        pub enum X86Opcode {
            $($enum,)* __NoMoreOpcodes
        }

        impl X86Opcode{
            pub fn opcode(&self) -> u64 {
                match self {
                    $(Self::$enum => $opcode,)*
                    Self::__NoMoreOpcodes => unreachable!(),
                }
            }

            pub fn operands(&self) -> &'static [X86OperandType] {
                match self {
                    $(Self::$enum => &[$($operand),*],)*
                    Self::__NoMoreOpcodes => unreachable!(),
                }
            }

            pub fn mnemonic(&self) -> &'static str {
                match self {
                    $(Self:: $enum => $mnemonic,)*
                    Self::__NoMoreOpcodes => unreachable!(),
                }
            }

            pub fn valid_in_mode(&self, modes: &X86Mode) -> bool {
                #[allow(unreachable_code)] // it's not unreachable in all expansions, and we want the default expansion to return true
                match self{
                    $(Self:: $enum =>{ $(return match modes {
                        $(X86Mode:: $mode => true,)*
                        _ => false
                    };)? true}),*
                    Self::__NoMoreOpcodes => unreachable!(),
                }
            }

        }

        impl std::fmt::Display for X86Opcode {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                match self {
                    $(Self::$enum => f.write_str($mnemonic),)*
                    Self::__NoMoreOpcodes => unreachable!(),
                }
            }
        }

        pub const X86_OPCODES: [X86Opcode; X86Opcode::__NoMoreOpcodes as usize] = [ $(X86Opcode::$enum,)* ];
    }
}

use target_tuples::Target;
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
    (PushImm8,"push",0x6A, [Imm(8)]),
    (IMul8,   "imul",0x6B, [RegGeneral, ModRMGeneral, Imm(8)]),
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
    (AddImm8, "add", 0x80, [RControlBits(0), ModRM(Byte), Imm(8)]),
    (OrImm8,  "or",  0x80, [RControlBits(1), ModRM(Byte), Imm(8)]),
    (AdcImm8, "adc", 0x80, [RControlBits(2), ModRM(Byte), Imm(8)]),
    (SbbImm8, "sbb", 0x80, [RControlBits(3), ModRM(Byte), Imm(8)]),
    (AndImm8, "and", 0x80, [RControlBits(4), ModRM(Byte), Imm(8)]),
    (SubImm8, "sub", 0x80, [RControlBits(5), ModRM(Byte), Imm(8)]),
    (XorImm8, "xor", 0x80, [RControlBits(6), ModRM(Byte), Imm(8)]),
    (CmpImm8, "cmp", 0x80, [RControlBits(6), ModRM(Byte), Imm(8)]),
    (AddImm,  "add", 0x81, [RControlBits(0), ModRMGeneral, ImmGeneral]),
    (OrImm,   "or",  0x81, [RControlBits(1), ModRMGeneral, ImmGeneral]),
    (AdcImm,  "adc", 0x81, [RControlBits(2), ModRMGeneral, ImmGeneral]),
    (SbbImm,  "sbb", 0x81, [RControlBits(3), ModRMGeneral, ImmGeneral]),
    (AndImm,  "and", 0x81, [RControlBits(4), ModRMGeneral, ImmGeneral]),
    (SubImm,  "sub", 0x81, [RControlBits(5), ModRMGeneral, ImmGeneral]),
    (XorImm,  "xor", 0x81, [RControlBits(6), ModRMGeneral, ImmGeneral]),
    (CmpImm,  "cmp", 0x81, [RControlBits(7), ModRMGeneral, ImmGeneral]),
    (AddGImm8,"add", 0x83, [RControlBits(0), ModRMGeneral, Imm(8)]),
    (OrGImm8, "or",  0x83, [RControlBits(1), ModRMGeneral, Imm(8)]),
    (AdcGImm8,"adc", 0x83, [RControlBits(2), ModRMGeneral, Imm(8)]),
    (SbbGImm8,"sbb", 0x83, [RControlBits(3), ModRMGeneral, Imm(8)]),
    (AndGImm8,"and", 0x83, [RControlBits(4), ModRMGeneral, Imm(8)]),
    (SubGImm8,"sub", 0x83, [RControlBits(5), ModRMGeneral, Imm(8)]),
    (XorGImm8,"xor", 0x83, [RControlBits(6), ModRMGeneral, Imm(8)]),
    (CmpGImm8,"cmp", 0x83, [RControlBits(7), ModRMGeneral, Imm(8)]),
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
    (MovUpsRM, "movups", 0x0F10, [Reg(Xmm),ModRM(Xmm)]),
    (MovSsRM, "movss", 0xF30F10, [Reg(Xmm), ModRMScalar(Double)]),
    (MovUpdRM, "movupd", 0x660F10, [Reg(Xmm), ModRM(Xmm)]),
    (MovSdRM, "movsd", 0xF20F10, [Reg(Xmm), ModRMScalar(Quad)]),
    (VMovUpsRM, "vmovups", 0x0F10, [VexReg,VexModRM]),
    (VMovSsRM, "vmovss", 0xF30F10, [VexReg, VexModRMScalar(Double)]),
    (VMovUpdRM, "vmovupd", 0x660F10, [VexReg, VexModRM]),
    (VMovSdRM, "vmovsd", 0xF20F10, [VexReg, VexModRMScalar(Quad)]),
    (MovUpsMR, "movups", 0x0F11, [ModRM(Xmm),Reg(Xmm)]),
    (MovSsMR, "movss", 0xF30F11, [ModRMScalar(Double),Reg(Xmm)]),
    (MovUpdMR, "movupd", 0x660F11, [ModRM(Xmm),Reg(Xmm)]),
    (MovSdMR, "movsd", 0xF20F11, [ModRMScalar(Quad), Reg(Xmm)]),
    (VMovUpsMR, "vmovups", 0x0F11, [VexModRM,VexReg]),
    (VMovSsMR, "vmovss", 0xF30F11, [VexModRMScalar(Double),VexReg]),
    (VMovUpdMR, "vmovupd", 0x660F11, [VexModRM,VexReg]),
    (VMovSdMR, "vmovsd", 0xF20F11, [VexModRMScalar(Quad),VexReg]),
    (MovDRM, "movd", 0x0F6E, [Reg(Mmx),ModRM(Double)]),
    (MovQRM, "movq", 0x0F6F, [Reg(Mmx),ModRM(Quad)]),
    (MovDMR, "movd", 0x0F7E, [Reg(Mmx),ModRM(Double)]),
    (MovQMR, "movq", 0x0F7F, [ModRM(Quad),Reg(Mmx)]),
    (MFence, "vmfence", 0x0FAE, []),

    (TileLoadD, "tileloadd", 0xF20F384B,[VexPrefix(Xmm), Reg(Tmm), ModRMSib(Tmm)]),
    (TileLoadDT1, "tileloaddt1", 0x660F384B,[VexPrefix(Xmm), Reg(Tmm), ModRMSib(Tmm)]),
    (TileStoreD, "tilestored", 0xF30F384B, [VexPrefix(Xmm),ModRMSib(Tmm),Reg(Tmm)]),

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
        size: X86RegisterClass,
        mode: ModRMRegOrSib,
    },
    IndirectDisp8 {
        size: X86RegisterClass,
        mode: ModRMRegOrSib,
        disp8: i8,
    },
    IndirectDisp32 {
        size: X86RegisterClass,
        mode: ModRMRegOrSib,
        disp32: i32,
    },
    Direct(X86Register),
}

impl ModRM {
    pub fn with_disp32(self, disp: i32) -> Option<Self> {
        match self {
            ModRM::Indirect {
                mode: ModRMRegOrSib::RipRel(_),
                ..
            }
            | ModRM::Indirect {
                mode: ModRMRegOrSib::Abs(_),
                ..
            } => todo!("[disp32]/[rIP+disp32]"),
            ModRM::Indirect { size, mode } => Some(Self::IndirectDisp32 {
                size,
                mode,
                disp32: disp,
            }),
            ModRM::IndirectDisp8 { size, mode, disp8 } => Some(ModRM::IndirectDisp32 {
                size,
                mode,
                disp32: (disp8 as i32) + disp,
            }),
            ModRM::IndirectDisp32 { size, mode, disp32 } => Some(ModRM::IndirectDisp32 {
                size,
                mode,
                disp32: disp32 + disp,
            }),
            ModRM::Direct(_) => None,
        }
    }

    pub fn resize(self, size: X86RegisterClass) -> Option<ModRM> {
        match self {
            ModRM::Indirect { mode, .. } => Some(ModRM::Indirect { size, mode }),
            ModRM::IndirectDisp8 { disp8, mode, .. } => {
                Some(ModRM::IndirectDisp8 { size, mode, disp8 })
            }
            ModRM::IndirectDisp32 { mode, disp32, .. } => {
                Some(ModRM::IndirectDisp32 { size, mode, disp32 })
            }
            ModRM::Direct(_) => None,
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum X86Operand {
    Register(X86Register),
    ModRM(ModRM),
    Immediate(u64),
    RelAddr(Address),
    AbsAddr(Address),
    FarAddr { sreg: X86Register, addr: Address },
    FarRelAddr { sreg: X86Register, addr: Address },
    FarModRM { sreg: X86Register, modrm: ModRM },
}

#[derive(Clone, Debug)]
pub struct X86Instruction {
    opc: X86Opcode,
    operands: Vec<X86Operand>,
    mode_override: Option<X86Mode>,
}

impl X86Instruction {
    pub const fn new(opc: X86Opcode, operands: Vec<X86Operand>) -> Self {
        Self {
            opc,
            operands,
            mode_override: None,
        }
    }

    pub const fn new_in_mode(opc: X86Opcode, operands: Vec<X86Operand>, mode: X86Mode) -> Self {
        Self {
            opc,
            operands,
            mode_override: Some(mode),
        }
    }

    pub const fn opcode(&self) -> X86Opcode {
        self.opc
    }

    pub fn operands(&self) -> &[X86Operand] {
        &self.operands
    }

    pub fn mode_override(&self) -> Option<X86Mode> {
        self.mode_override
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
    Int3,
    MFence
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct X86Decoder<R> {
    reader: R,
    mode: X86Mode,
}

impl<R> X86Decoder<R> {
    pub const fn new(reader: R, defmode: X86Mode) -> Self {
        Self {
            reader,
            mode: defmode,
        }
    }

    pub fn into_inner(self) -> R {
        self.reader
    }

    pub fn reader_mut(&mut self) -> &mut R {
        &mut self.reader
    }

    pub fn mode(&self) -> X86Mode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: X86Mode) {
        self.mode = mode;
    }
}

impl<R: Read> Read for X86Decoder<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.reader.read(buf)
    }
}

impl<R: InsnRead> InsnRead for X86Decoder<R> {
    fn read_addr(&mut self, size: usize, rel: bool) -> std::io::Result<Address> {
        self.reader.read_addr(size, rel)
    }

    fn read_reloc(
        &mut self,
        size: usize,
        rel: bool,
        offset: Option<isize>,
    ) -> std::io::Result<Option<Address>> {
        self.reader.read_reloc(size, rel, offset)
    }
}

impl<R: InsnRead> X86Decoder<R> {
    pub fn read_insn(&mut self) -> std::io::Result<X86Instruction> {
        todo!()
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
        self.mode = mode;
    }
}

impl<W: Write> Write for X86Encoder<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.writer.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

impl<W: InsnWrite> InsnWrite for X86Encoder<W> {
    fn write_addr(&mut self, size: usize, addr: Address, rel: bool) -> std::io::Result<()> {
        self.writer.write_addr(size, addr, rel)
    }

    fn offset(&self) -> usize {
        self.writer.offset()
    }

    fn write_reloc(&mut self, reloc: Reloc) -> std::io::Result<()> {
        self.writer.write_reloc(reloc)
    }
}

pub struct ModRMAndPrefixes {
    pub rex: Option<u8>,
    pub size_override: Option<u8>,
    pub addr_override: Option<u8>,
    pub modrm: u8,
    pub sib: Option<u8>,
    pub disp: Option<(usize, i64)>,
    pub addr: Option<(usize, Address, bool)>,
}

pub fn encode_modrm(modrm: ModRM, r: u8, mode: X86Mode) -> ModRMAndPrefixes {
    let mut output = ModRMAndPrefixes {
        rex: None,
        size_override: None,
        addr_override: None,
        modrm: 0,
        sib: None,
        disp: None,
        addr: None,
    };
    if r > 8 {
        *output.rex.get_or_insert(0x40) |= 0x04;
    }

    // Compute Size
    // Gets REX.W and 66h prefix
    let size = match modrm {
        ModRM::Indirect { size, .. }
        | ModRM::IndirectDisp8 { size, .. }
        | ModRM::IndirectDisp32 { size, .. } => size,
        ModRM::Direct(r) => r.class(),
    };

    // Compute REX.XB and Addr size
    let addr_size = match modrm {
        ModRM::Direct(_) => {
            mode.largest_gpr() // doesn't matter, don't insert anything extra
        }
        ModRM::Indirect {
            mode: ModRMRegOrSib::Reg(r),
            ..
        }
        | ModRM::IndirectDisp8 {
            mode: ModRMRegOrSib::Reg(r),
            ..
        }
        | ModRM::IndirectDisp32 {
            mode: ModRMRegOrSib::Reg(r),
            ..
        } => {
            if r.regnum() >= 8 {
                *output.rex.get_or_insert(0x40) |= 0x01;
            }
            r.class()
        }
        ModRM::Indirect {
            mode: ModRMRegOrSib::Sib { index, base, .. },
            ..
        }
        | ModRM::IndirectDisp8 {
            mode: ModRMRegOrSib::Sib { index, base, .. },
            ..
        }
        | ModRM::IndirectDisp32 {
            mode: ModRMRegOrSib::Sib { index, base, .. },
            ..
        } => {
            if base.regnum() >= 8 {
                *output.rex.get_or_insert(0x40) |= 0x01;
            }
            if index.regnum() >= 8 {
                *output.rex.get_or_insert(0x40) |= 0x02;
            }
            assert!(base.class() == index.class());
            base.class()
        }
        ModRM::Indirect {
            mode: ModRMRegOrSib::Index16 { .. },
            ..
        }
        | ModRM::IndirectDisp8 {
            mode: ModRMRegOrSib::Index16 { .. },
            ..
        }
        | ModRM::IndirectDisp32 {
            mode: ModRMRegOrSib::Index16 { .. },
            ..
        } => X86RegisterClass::Word,
        ModRM::Indirect {
            mode: ModRMRegOrSib::Abs(_),
            ..
        }
        | ModRM::Indirect {
            mode: ModRMRegOrSib::RipRel(_),
            ..
        } => mode.largest_gpr(),
        mode => panic!("Invalid ModRM byte {:?}", mode),
    };

    // Compute 66h REX.W
    match (size, mode) {
        (X86RegisterClass::Byte, _) => {}
        (X86RegisterClass::ByteRex, X86Mode::Long) => {}
        (X86RegisterClass::Word, X86Mode::Real | X86Mode::Virtual8086) => {}
        (X86RegisterClass::Double, X86Mode::Real | X86Mode::Virtual8086)
        | (X86RegisterClass::Word, _) => {
            output.size_override = Some(0x66);
        }
        (X86RegisterClass::Double, _) => {}
        (X86RegisterClass::Quad, X86Mode::Long) => {
            *output.rex.get_or_insert(0x40) |= 0x08;
        }
        (
            X86RegisterClass::Xmm
            | X86RegisterClass::Ymm
            | X86RegisterClass::Zmm
            | X86RegisterClass::St
            | X86RegisterClass::Mmx,
            _,
        )
        | (X86RegisterClass::Tmm, X86Mode::Long) => {}
        (size, mode) => panic!("Operand size {:?} not supported in mode {:?}", size, mode),
    };

    match (addr_size, mode) {
        (X86RegisterClass::Word, X86Mode::Real | X86Mode::Virtual8086)
        | (X86RegisterClass::Double, X86Mode::Protected | X86Mode::Compatibility)
        | (X86RegisterClass::Quad, X86Mode::Long) => {}
        (X86RegisterClass::Double, X86Mode::Real | X86Mode::Virtual8086 | X86Mode::Long)
        | (X86RegisterClass::Word, X86Mode::Protected | X86Mode::Compatibility) => {
            output.addr_override = Some(0x67)
        }
        (size, mode) => panic!("Operand size {:?} not supported in mode {:?}", size, mode),
    }

    // Finally, construct the ModRM (and possibly SIB) byte(s) + displacement
    match modrm {
        ModRM::Direct(reg) => {
            if reg.class() == X86RegisterClass::Byte && reg.regnum() > 3 && output.rex != None {
                panic!("Cannot encode register {:?} with a REX prefix", reg);
            } else if reg.class() == X86RegisterClass::ByteRex {
                output.rex.get_or_insert(0x40);
            }
            output.modrm = 0xC0 | ((r & 0x7) << 3) | (reg.regnum() & 0x7);
        }
        ModRM::Indirect {
            mode: ModRMRegOrSib::Index16 { base, index },
            ..
        } => {
            output.modrm = match (base, index) {
                (X86Register::Bx, X86Register::Si) => ((r & 0x7) << 3),
                (X86Register::Bx, X86Register::Di) => 0x01 | ((r & 0x7) << 3),
                (X86Register::Bp, X86Register::Si) => 0x02 | ((r & 0x7) << 3),
                (X86Register::Bp, X86Register::Di) => 0x03 | ((r & 0x7) << 3),
                (base, index) => panic!("16-bit addresses cannot encode [{}+{}]", base, index),
            };
        }
        ModRM::IndirectDisp8 {
            mode: ModRMRegOrSib::Index16 { base, index },
            disp8,
            ..
        } => {
            output.disp = Some((1, disp8.into()));
            output.modrm = match (base, index) {
                (X86Register::Bx, X86Register::Si) => 0x40 | ((r & 0x7) << 3),
                (X86Register::Bx, X86Register::Di) => 0x41 | ((r & 0x7) << 3),
                (X86Register::Bp, X86Register::Si) => 0x42 | ((r & 0x7) << 3),
                (X86Register::Bp, X86Register::Di) => 0x43 | ((r & 0x7) << 3),
                (base, index) => panic!("16-bit addresses cannot encode [{}+{}]", base, index),
            };
        }
        ModRM::IndirectDisp32 {
            mode: ModRMRegOrSib::Index16 { base, index },
            disp32,
            ..
        } => {
            output.disp = Some((2, disp32.into()));
            output.modrm = match (base, index) {
                (X86Register::Bx, X86Register::Si) => 0x80 | ((r & 0x7) << 3),
                (X86Register::Bx, X86Register::Di) => 0x81 | ((r & 0x7) << 3),
                (X86Register::Bp, X86Register::Si) => 0x82 | ((r & 0x7) << 3),
                (X86Register::Bp, X86Register::Di) => 0x83 | ((r & 0x7) << 3),
                (base, index) => panic!("16-bit addresses cannot encode [{}+{}]", base, index),
            };
        }
        ModRM::Indirect {
            mode: ModRMRegOrSib::Reg(reg),
            ..
        } if addr_size == X86RegisterClass::Word => {
            output.modrm = match reg {
                X86Register::Si => 0x04 | ((r & 0x7) << 3),
                X86Register::Di => 0x05 | ((r & 0x7) << 3),
                X86Register::Bx => 0x07 | ((r & 0x7) << 3),
                reg => panic!("16-bit addresses cannot encode [{}]", reg),
            }
        }
        ModRM::IndirectDisp8 {
            mode: ModRMRegOrSib::Reg(reg),
            disp8,
            ..
        } if addr_size == X86RegisterClass::Word => {
            output.disp = Some((1, disp8.into()));
            output.modrm = match reg {
                X86Register::Si => 0x44 | ((r & 0x7) << 3),
                X86Register::Di => 0x45 | ((r & 0x7) << 3),
                X86Register::Bp => 0x46 | ((r & 0x7) << 3),
                X86Register::Bx => 0x47 | ((r & 0x7) << 3),
                reg => panic!("16-bit addresses cannot encode [{}]", reg),
            }
        }
        ModRM::IndirectDisp32 {
            mode: ModRMRegOrSib::Reg(reg),
            disp32,
            ..
        } if addr_size == X86RegisterClass::Word => {
            output.disp = Some((2, disp32.into()));
            output.modrm = match reg {
                X86Register::Si => 0x84 | ((r & 0x7) << 3),
                X86Register::Di => 0x85 | ((r & 0x7) << 3),
                X86Register::Bp => 0x86 | ((r & 0x7) << 3),
                X86Register::Bx => 0x87 | ((r & 0x7) << 3),
                reg => panic!("16-bit addresses cannot encode [{}]", reg),
            }
        }
        ModRM::Indirect {
            mode: ModRMRegOrSib::Reg(reg),
            ..
        } => {
            match reg.regnum() & 0x7 {
                4 => {
                    // r/m encoding 0x04 is always an SIB byte
                    output.modrm = 0x04 | ((r & 0x7) << 3);
                    output.sib = Some(0x24)
                }
                5 => {
                    // r/m encoding 0x05 in mod 00
                    output.modrm = 0x45 | ((r & 0x7) << 3);
                    output.disp = Some((1, 0));
                }
                n => output.modrm = ((r & 0x7) << 3) | n,
            }
        }
        ModRM::Indirect {
            mode: ModRMRegOrSib::RipRel(addr),
            ..
        } => {
            if mode == X86Mode::Long {
                output.modrm = 0x05 | ((r & 0x7) << 3);
                output.addr = Some((4, addr, true));
            } else {
                panic!("[rip+{:?}] is not supported in this mode", addr)
            }
        }
        ModRM::Indirect {
            mode: ModRMRegOrSib::Abs(addr),
            ..
        } => {
            if mode == X86Mode::Long {
                // ModR/M can only encode [rip+disp32] in 64-bit mode
                // Use an SIB byte instead
                output.modrm = 0x04 | ((r & 0x7) << 3);
                output.sib = Some(0x25);
                output.addr = Some((4, addr, false));
            } else if addr_size == X86RegisterClass::Word {
                output.modrm = 0x06 | ((r & 0x7) << 3);
                output.addr = Some((2, addr, false));
            } else {
                output.modrm = 0x05 | ((r & 0x7) << 3);
                output.addr = Some((4, addr, false))
            }
        }
        ModRM::Indirect {
            mode: ModRMRegOrSib::Sib { scale, index, base },
            ..
        } => {
            output.modrm = 0x04 | ((r & 0x7) << 3);
            if index.regnum() == 4 {
                panic!("Cannot encode SIB byte with {:?} as index", index);
            }
            if base.regnum() & 0x7 == 5 {
                output.modrm |= 0x40;
                output.sib = Some(
                    ((scale.trailing_zeros() as u8 + 1) << 6)
                        | ((index.regnum() & 0x7) << 3)
                        | (0x05),
                );
                output.disp = Some((1, 0));
            } else {
                output.sib = Some(
                    (((scale.trailing_zeros() as u8 + 1) as u8) << 6)
                        | ((index.regnum() & 0x7) << 3)
                        | (base.regnum()),
                )
            }
        }
        ModRM::IndirectDisp8 {
            mode: ModRMRegOrSib::Sib { scale, index, base },
            disp8,
            ..
        } => {
            output.modrm = 0x44 | ((r & 0x7) << 3);
            output.disp = Some((1, disp8.into()));
            if index.regnum() == 4 {
                panic!("Cannot encode SIB byte with {:?} as index", index);
            }
            output.sib = Some(
                ((scale.trailing_zeros() as u8 + 1) << 6)
                    | ((index.regnum() & 0x7) << 3)
                    | (base.regnum()),
            )
        }
        ModRM::IndirectDisp32 {
            mode: ModRMRegOrSib::Sib { scale, index, base },
            disp32,
            ..
        } => {
            output.modrm = 0x84 | ((r & 0x7) << 3);
            output.disp = Some((4, disp32.into()));
            if index.regnum() == 4 {
                panic!("Cannot encode SIB byte with {:?} as index", index);
            }
            output.sib = Some(
                ((scale.trailing_zeros() as u8 + 1) << 6)
                    | ((index.regnum() & 0x7) << 3)
                    | (base.regnum()),
            )
        }
        ModRM::IndirectDisp8 {
            mode: ModRMRegOrSib::Reg(reg),
            disp8,
            ..
        } => {
            if reg.regnum() & 0x7 == 4 {
                // r/m encoding 0x04 is always an SIB byte
                output.modrm = 0x44 | ((r & 0x7) << 3);
                output.sib = Some(0x24)
            } else {
                output.modrm = 0x40 | (reg.regnum() & 0x7) | ((r & 0x7) << 3)
            }
            output.disp = Some((1, disp8.into()))
        }
        ModRM::IndirectDisp32 {
            mode: ModRMRegOrSib::Reg(reg),
            disp32,
            ..
        } => {
            if reg.regnum() & 0x7 == 4 {
                // r/m encoding 0x04 is always an SIB byte
                output.modrm = 0x84 | ((r & 0x7) << 3);
                output.sib = Some(0x24)
            } else {
                output.modrm = 0x80 | (reg.regnum() & 0x7) | ((r & 0x7) << 3)
            }
            output.disp = Some((4, disp32.into()))
        }
        modrm => todo!("{:?}", modrm),
    }

    output
}

impl<W: InsnWrite> X86Encoder<W> {
    pub fn write_insn(&mut self, insn: X86Instruction) -> std::io::Result<()> {
        let mode = insn.mode_override().unwrap_or(self.mode);
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
            [Imm(n)] => {
                assert_eq!(insn.operands.len(), 1);

                self.writer.write_all(&opcode)?;

                match &insn.operands[0] {
                    X86Operand::Immediate(imm) => self.writer.write_all(&imm.to_ne_bytes()[..*n]),
                    X86Operand::AbsAddr(addr) => {
                        self.writer.write_addr(*n / 8, addr.clone(), false)
                    }
                    op => panic!("Invalid operand {:?} for Immediate", op),
                }
            }
            [Rel(n)] => {
                let n = *n;
                assert_eq!(insn.operands.len(), 1);

                self.writer.write_all(&opcode)?;

                match &insn.operands[0] {
                    X86Operand::Immediate(imm) => self.writer.write_all(&imm.to_ne_bytes()[..n]),
                    X86Operand::AbsAddr(addr) => self.writer.write_addr(n, addr.clone(), true),
                    op => panic!("Invalid operand {:?} for Immediate", op),
                }
            }
            [Insn] => todo!(),
            [OpRegMode] => {
                let reg = match &insn.operands[0] {
                    X86Operand::Register(r) => r,
                    op => panic!("Invalid operand {:?} for OpRegMode", op),
                };

                match (reg.class(), mode) {
                    (X86RegisterClass::Word, X86Mode::Real | X86Mode::Virtual8086)
                    | (X86RegisterClass::Double, X86Mode::Protected | X86Mode::Compatibility)
                    | (X86RegisterClass::Quad, X86Mode::Long) => {}
                    (X86RegisterClass::Word, _)
                    | (X86RegisterClass::Double, X86Mode::Real | X86Mode::Virtual8086) => {
                        self.writer.write_all(&[0x66])?;
                    }
                    (_, mode) => panic!("Register {} is not valid in mode {:?}", reg, mode),
                }

                *opcode.last_mut().unwrap() += reg.regnum() & 0x7;
                let mut opcode = &opcode[..];
                let sse_prefix = opcode[0];

                match sse_prefix {
                    0x66 | 0xF2 | 0xF3 => {
                        self.writer.write_all(core::slice::from_ref(&sse_prefix))?;
                        opcode = &opcode[1..]
                    }
                    _ => {}
                }

                if reg.regnum() > 8 {
                    self.writer.write_all(&[0x41])?;
                }

                self.writer.write_all(opcode)
            }
            [ModRM(cl1), Reg(cl2)] if cl1 == cl2 => {
                let reg = match &insn.operands[1] {
                    X86Operand::Register(r) => *r,
                    op => panic!("Invalid operand {:?} for Reg({:?})", op, cl2),
                };

                let modrm = match &insn.operands[0] {
                    X86Operand::Register(r) => ModRM::Direct(*r),
                    X86Operand::ModRM(modrm) => modrm.clone(),
                    X86Operand::RelAddr(addr) => ModRM::Indirect {
                        size: *cl1,
                        mode: ModRMRegOrSib::RipRel(addr.clone()),
                    },
                    X86Operand::AbsAddr(addr) => ModRM::Indirect {
                        size: *cl1,
                        mode: ModRMRegOrSib::Abs(addr.clone()),
                    },
                    op => panic!("Invalid operand {:?} for ModRM({:?})", op, cl1),
                };

                let mut encoding = encode_modrm(modrm, reg.regnum(), mode);

                if reg.class() == X86RegisterClass::Byte
                    && encoding.rex.is_some()
                    && reg.regnum() > 3
                {
                    panic!("Cannot encode {} with a rex prefix", reg);
                } else if reg.class() == X86RegisterClass::ByteRex {
                    encoding.rex.get_or_insert(0x40); // Put in a null REX prefix if we have spl/bpl/sil/dil
                }
                let mut opcode = &opcode[..];
                let sse_prefix = opcode[0];

                self.writer.write_all(
                    encoding
                        .size_override
                        .as_ref()
                        .map(core::slice::from_ref)
                        .unwrap_or(&[]),
                )?;
                self.writer.write_all(
                    encoding
                        .addr_override
                        .as_ref()
                        .map(core::slice::from_ref)
                        .unwrap_or(&[]),
                )?;

                match sse_prefix {
                    0x66 | 0xF2 | 0xF3 => {
                        self.writer.write_all(core::slice::from_ref(&sse_prefix))?;
                        opcode = &opcode[1..]
                    }
                    _ => {}
                }

                self.writer.write_all(
                    encoding
                        .rex
                        .as_ref()
                        .map(core::slice::from_ref)
                        .unwrap_or(&[]),
                )?;

                self.writer.write_all(opcode)?;
                self.writer
                    .write_all(core::slice::from_ref(&encoding.modrm))?;
                self.writer.write_all(
                    encoding
                        .sib
                        .as_ref()
                        .map(core::slice::from_ref)
                        .unwrap_or(&[]),
                )?;
                match (encoding.disp, encoding.addr) {
                    (Some((size, disp)), None) => {
                        self.writer.write_all(&disp.to_ne_bytes()[..size])?
                    }
                    (None, Some((size, addr, pcrel))) => {
                        self.writer.write_addr(size * 8, addr, pcrel)?
                    }
                    (None, None) => {}
                    (Some(_), Some(_)) => {
                        panic!("Cannot encode both an address and a displacement")
                    }
                }
                Ok(())
            }
            [Reg(cl2), ModRM(cl1)] if cl1 == cl2 => {
                let reg = match &insn.operands[0] {
                    X86Operand::Register(r) => *r,
                    op => panic!("Invalid operand {:?} for Reg({:?})", op, cl2),
                };

                let modrm = match &insn.operands[1] {
                    X86Operand::Register(r) => ModRM::Direct(*r),
                    X86Operand::ModRM(modrm) => modrm.clone(),
                    X86Operand::RelAddr(addr) => ModRM::Indirect {
                        size: *cl1,
                        mode: ModRMRegOrSib::RipRel(addr.clone()),
                    },
                    X86Operand::AbsAddr(addr) => ModRM::Indirect {
                        size: *cl1,
                        mode: ModRMRegOrSib::Abs(addr.clone()),
                    },
                    op => panic!("Invalid operand {:?} for ModRM({:?})", op, cl1),
                };

                let mut encoding = encode_modrm(modrm, reg.regnum(), mode);

                if reg.class() == X86RegisterClass::Byte
                    && encoding.rex.is_some()
                    && reg.regnum() > 3
                {
                    panic!("Cannot encode {} with a rex prefix", reg);
                } else if reg.class() == X86RegisterClass::ByteRex {
                    encoding.rex.get_or_insert(0x40); // Put in a null REX prefix if we have spl/bpl/sil/dil
                }
                let mut opcode = &opcode[..];
                let sse_prefix = opcode[0];

                self.writer.write_all(
                    encoding
                        .size_override
                        .as_ref()
                        .map(core::slice::from_ref)
                        .unwrap_or(&[]),
                )?;
                self.writer.write_all(
                    encoding
                        .addr_override
                        .as_ref()
                        .map(core::slice::from_ref)
                        .unwrap_or(&[]),
                )?;

                match sse_prefix {
                    0x66 | 0xF2 | 0xF3 => {
                        self.writer.write_all(core::slice::from_ref(&sse_prefix))?;
                        opcode = &opcode[1..]
                    }
                    _ => {}
                }

                self.writer.write_all(
                    encoding
                        .rex
                        .as_ref()
                        .map(core::slice::from_ref)
                        .unwrap_or(&[]),
                )?;

                self.writer.write_all(opcode)?;
                self.writer
                    .write_all(core::slice::from_ref(&encoding.modrm))?;
                self.writer.write_all(
                    encoding
                        .sib
                        .as_ref()
                        .map(core::slice::from_ref)
                        .unwrap_or(&[]),
                )?;
                match (encoding.disp, encoding.addr) {
                    (Some((size, disp)), None) => {
                        self.writer.write_all(&disp.to_ne_bytes()[..size])?
                    }
                    (None, Some((size, addr, pcrel))) => {
                        self.writer.write_addr(size, addr, pcrel)?
                    }
                    (None, None) => {}
                    (Some(_), Some(_)) => {
                        panic!("Cannot encode both an address and a displacement")
                    }
                }
                Ok(())
            }
            [ModRMGeneral, RegGeneral] => {
                let reg = match &insn.operands[1] {
                    X86Operand::Register(r) => *r,
                    op => panic!("Invalid operand {:?} for RegGeneral", op),
                };

                let modrm = match &insn.operands[0] {
                    X86Operand::Register(r) => ModRM::Direct(*r),
                    X86Operand::ModRM(modrm) => modrm.clone(),
                    X86Operand::RelAddr(addr) => ModRM::Indirect {
                        size: reg.class(),
                        mode: ModRMRegOrSib::RipRel(addr.clone()),
                    },
                    X86Operand::AbsAddr(addr) => ModRM::Indirect {
                        size: reg.class(),
                        mode: ModRMRegOrSib::Abs(addr.clone()),
                    },
                    op => panic!("Invalid operand {:?} for ModRMGeneral", op),
                };

                let encoding = encode_modrm(modrm, reg.regnum(), mode);

                let mut opcode = &opcode[..];
                let sse_prefix = opcode[0];

                self.writer.write_all(
                    encoding
                        .size_override
                        .as_ref()
                        .map(core::slice::from_ref)
                        .unwrap_or(&[]),
                )?;
                self.writer.write_all(
                    encoding
                        .addr_override
                        .as_ref()
                        .map(core::slice::from_ref)
                        .unwrap_or(&[]),
                )?;

                match sse_prefix {
                    0x66 | 0xF2 | 0xF3 => {
                        self.writer.write_all(core::slice::from_ref(&sse_prefix))?;
                        opcode = &opcode[1..]
                    }
                    _ => {}
                }

                self.writer.write_all(
                    encoding
                        .rex
                        .as_ref()
                        .map(core::slice::from_ref)
                        .unwrap_or(&[]),
                )?;

                self.writer.write_all(opcode)?;
                self.writer
                    .write_all(core::slice::from_ref(&encoding.modrm))?;
                self.writer.write_all(
                    encoding
                        .sib
                        .as_ref()
                        .map(core::slice::from_ref)
                        .unwrap_or(&[]),
                )?;
                match (encoding.disp, encoding.addr) {
                    (Some((size, disp)), None) => {
                        self.writer.write_all(&disp.to_ne_bytes()[..size])?
                    }
                    (None, Some((size, addr, pcrel))) => {
                        self.writer.write_addr(size, addr, pcrel)?
                    }
                    (None, None) => {}
                    (Some(_), Some(_)) => {
                        panic!("Cannot encode both an address and a displacement")
                    }
                }
                Ok(())
            }
            [RegGeneral, ModRMGeneral] => {
                let reg = match &insn.operands[0] {
                    X86Operand::Register(r) => *r,
                    op => panic!("Invalid operand {:?} for RegGeneral", op),
                };

                let modrm = match &insn.operands[1] {
                    X86Operand::Register(r) => ModRM::Direct(*r),
                    X86Operand::ModRM(modrm) => modrm.clone(),
                    X86Operand::RelAddr(addr) => ModRM::Indirect {
                        size: reg.class(),
                        mode: ModRMRegOrSib::RipRel(addr.clone()),
                    },
                    X86Operand::AbsAddr(addr) => ModRM::Indirect {
                        size: reg.class(),
                        mode: ModRMRegOrSib::Abs(addr.clone()),
                    },
                    op => panic!("Invalid operand {:?} for ModRMGeneral", op),
                };

                let encoding = encode_modrm(modrm, reg.regnum(), mode);

                let mut opcode = &opcode[..];
                let sse_prefix = opcode[0];

                self.writer.write_all(
                    encoding
                        .size_override
                        .as_ref()
                        .map(core::slice::from_ref)
                        .unwrap_or(&[]),
                )?;
                self.writer.write_all(
                    encoding
                        .addr_override
                        .as_ref()
                        .map(core::slice::from_ref)
                        .unwrap_or(&[]),
                )?;

                match sse_prefix {
                    0x66 | 0xF2 | 0xF3 => {
                        self.writer.write_all(core::slice::from_ref(&sse_prefix))?;
                        opcode = &opcode[1..]
                    }
                    _ => {}
                }

                self.writer.write_all(
                    encoding
                        .rex
                        .as_ref()
                        .map(core::slice::from_ref)
                        .unwrap_or(&[]),
                )?;

                self.writer.write_all(opcode)?;
                self.writer
                    .write_all(core::slice::from_ref(&encoding.modrm))?;
                self.writer.write_all(
                    encoding
                        .sib
                        .as_ref()
                        .map(core::slice::from_ref)
                        .unwrap_or(&[]),
                )?;
                match (encoding.disp, encoding.addr) {
                    (Some((size, disp)), None) => {
                        self.writer.write_all(&disp.to_ne_bytes()[..size])?
                    }
                    (None, Some((size, addr, pcrel))) => {
                        self.writer.write_addr(size * 8, addr, pcrel)?
                    }
                    (None, None) => {}
                    (Some(_), Some(_)) => {
                        panic!("Cannot encode both an address and a displacement")
                    }
                }
                Ok(())
            }
            [RControlBits(b), ModRMGeneral, ImmGeneral] => {
                let modrm = match &insn.operands[0] {
                    X86Operand::Register(r) => ModRM::Direct(*r),
                    X86Operand::ModRM(modrm) => modrm.clone(),
                    X86Operand::RelAddr(addr) => ModRM::Indirect {
                        size: mode.largest_gpr(),
                        mode: ModRMRegOrSib::RipRel(addr.clone()),
                    },
                    X86Operand::AbsAddr(addr) => ModRM::Indirect {
                        size: mode.largest_gpr(),
                        mode: ModRMRegOrSib::Abs(addr.clone()),
                    },
                    op => panic!("Invalid operand {:?} for ModRMGeneral", op),
                };

                let size = match modrm {
                    ModRM::Indirect { size, .. } => size,
                    ModRM::IndirectDisp8 { size, .. } => size,
                    ModRM::IndirectDisp32 { size, .. } => size,
                    ModRM::Direct(r) => r.class(),
                };

                let encoding = encode_modrm(modrm, *b, mode);

                let mut opcode = &opcode[..];
                let sse_prefix = opcode[0];

                self.writer.write_all(
                    encoding
                        .size_override
                        .as_ref()
                        .map(core::slice::from_ref)
                        .unwrap_or(&[]),
                )?;
                self.writer.write_all(
                    encoding
                        .addr_override
                        .as_ref()
                        .map(core::slice::from_ref)
                        .unwrap_or(&[]),
                )?;

                match sse_prefix {
                    0x66 | 0xF2 | 0xF3 => {
                        self.writer.write_all(core::slice::from_ref(&sse_prefix))?;
                        opcode = &opcode[1..]
                    }
                    _ => {}
                }

                self.writer.write_all(
                    encoding
                        .rex
                        .as_ref()
                        .map(core::slice::from_ref)
                        .unwrap_or(&[]),
                )?;

                self.writer.write_all(opcode)?;
                self.writer
                    .write_all(core::slice::from_ref(&encoding.modrm))?;
                self.writer.write_all(
                    encoding
                        .sib
                        .as_ref()
                        .map(core::slice::from_ref)
                        .unwrap_or(&[]),
                )?;

                match (encoding.disp, encoding.addr) {
                    (Some((size, disp)), None) => {
                        self.writer.write_all(&disp.to_ne_bytes()[..size])?;
                    }
                    (None, Some((size, addr, pcrel))) => {
                        self.writer.write_addr(size * 8, addr, pcrel)?;
                    }
                    (None, None) => {}
                    (Some(_), Some(_)) => {
                        panic!("Cannot encode both an address and a displacement")
                    }
                }

                let n = match size {
                    X86RegisterClass::Word => 2,
                    _ => 4,
                };

                match &insn.operands[1] {
                    X86Operand::Immediate(imm) => self.writer.write_all(&imm.to_ne_bytes()[..n]),
                    X86Operand::AbsAddr(addr) => self.writer.write_addr(n * 8, addr.clone(), false),
                    op => panic!("Invalid operand {:?} for Immediate", op),
                }
            }
            [RControlBits(b), ModRM(Byte), Imm(8)] => {
                let modrm = match &insn.operands[0] {
                    X86Operand::Register(r) => ModRM::Direct(*r),
                    X86Operand::ModRM(modrm) => modrm.clone(),
                    X86Operand::RelAddr(addr) => ModRM::Indirect {
                        size: mode.largest_gpr(),
                        mode: ModRMRegOrSib::RipRel(addr.clone()),
                    },
                    X86Operand::AbsAddr(addr) => ModRM::Indirect {
                        size: mode.largest_gpr(),
                        mode: ModRMRegOrSib::Abs(addr.clone()),
                    },
                    op => panic!("Invalid operand {:?} for ModRMGeneral", op),
                };

                let encoding = encode_modrm(modrm, *b, mode);

                let mut opcode = &opcode[..];
                let sse_prefix = opcode[0];

                self.writer.write_all(
                    encoding
                        .size_override
                        .as_ref()
                        .map(core::slice::from_ref)
                        .unwrap_or(&[]),
                )?;
                self.writer.write_all(
                    encoding
                        .addr_override
                        .as_ref()
                        .map(core::slice::from_ref)
                        .unwrap_or(&[]),
                )?;

                match sse_prefix {
                    0x66 | 0xF2 | 0xF3 => {
                        self.writer.write_all(core::slice::from_ref(&sse_prefix))?;
                        opcode = &opcode[1..]
                    }
                    _ => {}
                }

                self.writer.write_all(
                    encoding
                        .rex
                        .as_ref()
                        .map(core::slice::from_ref)
                        .unwrap_or(&[]),
                )?;

                self.writer.write_all(opcode)?;
                self.writer
                    .write_all(core::slice::from_ref(&encoding.modrm))?;
                self.writer.write_all(
                    encoding
                        .sib
                        .as_ref()
                        .map(core::slice::from_ref)
                        .unwrap_or(&[]),
                )?;

                match (encoding.disp, encoding.addr) {
                    (Some((size, disp)), None) => {
                        self.writer.write_all(&disp.to_ne_bytes()[..size])?;
                    }
                    (None, Some((size, addr, pcrel))) => {
                        self.writer.write_addr(size * 8, addr, pcrel)?;
                    }
                    (None, None) => {}
                    (Some(_), Some(_)) => {
                        panic!("Cannot encode both an address and a displacement")
                    }
                }

                match &insn.operands[1] {
                    X86Operand::Immediate(imm) => self.writer.write_all(&imm.to_ne_bytes()[..1]),
                    X86Operand::AbsAddr(addr) => self.writer.write_addr(8, addr.clone(), false),
                    op => panic!("Invalid operand {:?} for Immediate", op),
                }
            }
            [RelGeneral] => {
                let size = match mode {
                    X86Mode::Virtual8086 | X86Mode::Real => 2,
                    _ => 4,
                };

                let mut opcode = &opcode[..];
                let sse_prefix = opcode[0];

                match sse_prefix {
                    0x66 | 0xF2 | 0xF3 => {
                        self.writer.write_all(core::slice::from_ref(&sse_prefix))?;
                        opcode = &opcode[1..]
                    }
                    _ => {}
                }

                self.writer.write_all(opcode)?;

                match &insn.operands[0] {
                    X86Operand::RelAddr(addr) => {
                        self.writer.write_addr(size * 8, addr.clone(), true)
                    }
                    op => panic!("Invalid Operand for RelGeneral {:?}", op),
                }
            }
            [OpRegGeneral, ImmGeneralWide] => {
                let reg = match &insn.operands[0] {
                    X86Operand::Register(r) => r,
                    op => panic!("Invalid Operand for {:?}", op),
                };

                let mut rex = None;
                if reg.regnum() > 7 {
                    *rex.get_or_insert(0x40) |= 0x1;
                }

                let opsize = match (reg.class(), mode) {
                    (X86RegisterClass::Word, X86Mode::Real | X86Mode::Virtual8086)
                    | (
                        X86RegisterClass::Double,
                        X86Mode::Protected | X86Mode::Compatibility | X86Mode::Long,
                    ) => None,
                    (X86RegisterClass::Word, _)
                    | (X86RegisterClass::Double, X86Mode::Real | X86Mode::Virtual8086) => {
                        Some(0x66)
                    }
                    (X86RegisterClass::Quad, X86Mode::Long) => None,
                    (class, mode) => {
                        panic!("Unsupported register class {:?} in mode {:?}", class, mode)
                    }
                };

                let immsize = reg.class().size(mode);

                self.writer
                    .write_all(opsize.as_ref().map(core::slice::from_ref).unwrap_or(&[]))?;

                *opcode.last_mut().unwrap() += reg.regnum() & 0x7;
                let mut opcode = &opcode[..];
                let sse_prefix = opcode[0];

                match sse_prefix {
                    0x66 | 0xF2 | 0xF3 => {
                        self.writer.write_all(core::slice::from_ref(&sse_prefix))?;
                        opcode = &opcode[1..]
                    }
                    _ => {}
                }

                self.writer
                    .write_all(rex.as_ref().map(core::slice::from_ref).unwrap_or(&[]))?;

                self.writer.write_all(opcode)?;
                match &insn.operands[1] {
                    X86Operand::Immediate(i) => self.writer.write_all(&i.to_le_bytes()[0..immsize]),
                    X86Operand::AbsAddr(addr) => {
                        self.writer.write_addr(immsize * 8, addr.clone(), false)
                    }
                    op => panic!("Invalid operand {:?} for ImmGeneralWide", op),
                }
            }
            m => panic!("Unsupported Addressing Mode {:?}", m),
        }
    }
}

#[cfg(test)]
mod test {

    use crate::x86::X86Register;

    use super::{ModRM, X86Encoder, X86Instruction, X86Mode, X86Opcode, X86Operand};

    use crate::test::TestWriter;

    #[test]
    fn test_encoder_simple() {
        let mut enc = X86Encoder::new(TestWriter { inner: Vec::new() }, X86Mode::Protected);
        enc.write_insn(X86Instruction::Retn).unwrap();
        assert_eq!(&*enc.writer_mut().inner, &[0xC3]);
    }

    #[test]
    fn test_encoder_modrm_reg32() {
        let mut enc = X86Encoder::new(TestWriter { inner: Vec::new() }, X86Mode::Protected);
        enc.write_insn(X86Instruction::new(
            X86Opcode::XorMR,
            vec![
                X86Operand::ModRM(ModRM::Direct(X86Register::Eax)),
                X86Operand::Register(X86Register::Eax),
            ],
        ))
        .unwrap();

        assert_eq!(&*enc.writer_mut().inner, &[0x31, 0xC0]);
    }

    #[test]
    fn test_encoder_modrm_reg32_long() {
        let mut enc = X86Encoder::new(TestWriter { inner: Vec::new() }, X86Mode::Long);
        enc.write_insn(X86Instruction::new(
            X86Opcode::XorMR,
            vec![
                X86Operand::ModRM(ModRM::Direct(X86Register::Eax)),
                X86Operand::Register(X86Register::Eax),
            ],
        ))
        .unwrap();

        assert_eq!(&*enc.writer_mut().inner, &[0x31, 0xC0]);
    }

    #[test]
    fn test_encoder_modrm_reg32_real() {
        let mut enc = X86Encoder::new(TestWriter { inner: Vec::new() }, X86Mode::Real);
        enc.write_insn(X86Instruction::new(
            X86Opcode::XorMR,
            vec![
                X86Operand::ModRM(ModRM::Direct(X86Register::Eax)),
                X86Operand::Register(X86Register::Eax),
            ],
        ))
        .unwrap();

        assert_eq!(&*enc.writer_mut().inner, &[0x66, 0x31, 0xC0]);
    }

    #[test]
    fn test_encoder_modrm_reg16_real() {
        let mut enc = X86Encoder::new(TestWriter { inner: Vec::new() }, X86Mode::Real);
        enc.write_insn(X86Instruction::new(
            X86Opcode::XorMR,
            vec![
                X86Operand::ModRM(ModRM::Direct(X86Register::Ax)),
                X86Operand::Register(X86Register::Ax),
            ],
        ))
        .unwrap();

        assert_eq!(&*enc.writer_mut().inner, &[0x31, 0xC0]);
    }

    #[test]
    fn test_encoder_modrm_reg16_protected() {
        let mut enc = X86Encoder::new(TestWriter { inner: Vec::new() }, X86Mode::Protected);
        enc.write_insn(X86Instruction::new(
            X86Opcode::XorMR,
            vec![
                X86Operand::ModRM(ModRM::Direct(X86Register::Ax)),
                X86Operand::Register(X86Register::Ax),
            ],
        ))
        .unwrap();

        assert_eq!(&*enc.writer_mut().inner, &[0x66, 0x31, 0xC0]);
    }

    #[test]
    fn test_encoder_modrm_reg16_long() {
        let mut enc = X86Encoder::new(TestWriter { inner: Vec::new() }, X86Mode::Long);
        enc.write_insn(X86Instruction::new(
            X86Opcode::XorMR,
            vec![
                X86Operand::ModRM(ModRM::Direct(X86Register::Ax)),
                X86Operand::Register(X86Register::Ax),
            ],
        ))
        .unwrap();

        assert_eq!(&*enc.writer_mut().inner, &[0x66, 0x31, 0xC0]);
    }

    #[test]
    fn test_encoder_modrm_reg64_long() {
        let mut enc = X86Encoder::new(TestWriter { inner: Vec::new() }, X86Mode::Long);
        enc.write_insn(X86Instruction::new(
            X86Opcode::XorMR,
            vec![
                X86Operand::ModRM(ModRM::Direct(X86Register::Rax)),
                X86Operand::Register(X86Register::Rax),
            ],
        ))
        .unwrap();

        assert_eq!(&*enc.writer_mut().inner, &[0x48, 0x31, 0xC0]);
    }

    #[test]
    fn test_encoder_opreg_mode() {
        let mut enc = X86Encoder::new(TestWriter { inner: Vec::new() }, X86Mode::Protected);
        enc.write_insn(X86Instruction::new(
            X86Opcode::Push,
            vec![X86Operand::Register(X86Register::Eax)],
        ))
        .unwrap();
        assert_eq!(&*enc.writer_mut().inner, &[0x50]);
    }

    #[test]
    fn test_encoder_opreg_mode_long() {
        let mut enc = X86Encoder::new(TestWriter { inner: Vec::new() }, X86Mode::Long);
        enc.write_insn(X86Instruction::new(
            X86Opcode::Push,
            vec![X86Operand::Register(X86Register::Rax)],
        ))
        .unwrap();
        assert_eq!(&*enc.writer_mut().inner, &[0x50]);
    }

    #[test]
    fn test_encoder_opreg_mode_protected_r16() {
        let mut enc = X86Encoder::new(TestWriter { inner: Vec::new() }, X86Mode::Protected);
        enc.write_insn(X86Instruction::new(
            X86Opcode::Push,
            vec![X86Operand::Register(X86Register::Ax)],
        ))
        .unwrap();
        assert_eq!(&*enc.writer_mut().inner, &[0x66, 0x50]);
    }

    #[test]
    fn test_encoder_opreg_mode_long_r16() {
        let mut enc = X86Encoder::new(TestWriter { inner: Vec::new() }, X86Mode::Long);
        enc.write_insn(X86Instruction::new(
            X86Opcode::Push,
            vec![X86Operand::Register(X86Register::Ax)],
        ))
        .unwrap();
        assert_eq!(&*enc.writer_mut().inner, &[0x66, 0x50]);
    }

    #[test]
    fn test_encoder_opreg_mode_real_r32() {
        let mut enc = X86Encoder::new(TestWriter { inner: Vec::new() }, X86Mode::Real);
        enc.write_insn(X86Instruction::new(
            X86Opcode::Push,
            vec![X86Operand::Register(X86Register::Eax)],
        ))
        .unwrap();
        assert_eq!(&*enc.writer_mut().inner, &[0x66, 0x50]);
    }

    #[test]
    #[should_panic]
    fn test_encoder_opreg_mode_long_r32() {
        let mut enc = X86Encoder::new(TestWriter { inner: Vec::new() }, X86Mode::Long);
        let _ = enc.write_insn(X86Instruction::new(
            X86Opcode::Push,
            vec![X86Operand::Register(X86Register::Eax)],
        ));
    }
}
