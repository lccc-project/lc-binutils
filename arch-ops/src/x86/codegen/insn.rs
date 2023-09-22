use super::{
    ModRMOptions, OpRegOptions, SsePrefix, X86Encoding, X86EncodingMode, X86EncodingPrefix,
    X86InstructionMap, X86Operand, X86OperandMode,
};
use crate::x86::{X86Mode, X86Register, X86RegisterClass};

use core::mem::MaybeUninit;

macro_rules! expand_opt {
    () => {
        None
    };
    ($expr:expr) => {
        Some($expr)
    };
}

macro_rules! expand_or_zero {
    () => {
        0
    };
    ($val:literal) => {
        $val
    };
}

macro_rules! x86_codegen_instructions {
    {
        $(insn $mnemonic:ident {
            $([$($oprs:pat),*] $(in $($modes:ident)|*)? => $($encoding_prefix:ident)|+ $opcode:literal $(imm $immsize:literal)? @ $encoding_mode:expr;)*
        })*
    } => {
        paste::paste!{
            #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
            pub enum X86CodegenOpcode{
                $([<$mnemonic:camel>]),*
            }

            impl X86CodegenOpcode{
                pub const fn mnenomic(&self) -> &'static str{
                    match self{
                        $(Self::[<$mnemonic:camel>] => ::core::stringify!($mnemonic)),*
                    }
                }

                #[allow(unsafe_code, unused_assignment, unused_mut)]
                pub fn allowed_prefixes<'a>(&self, oprs: &[X86Operand], buffer: &'a mut [MaybeUninit<X86EncodingPrefix>]) -> &'a mut [X86EncodingPrefix]{
                    let mut allowed_prefixes = [None;X86EncodingPrefix::PREFIX_COUNT];

                    use X86OperandMode::*;
                    use X86Register::*;
                    use X86RegisterClass::*;
                    use X86RegisterClass::{Cr,Dr};
                    match self{
                        $(Self::[<$mnemonic:camel>] => {
                            $({
                                if {
                                    let mut idx = 0;
                                    $(({
                                        (oprs.get({let i = idx; idx += 1; i}).map(|opr| opr.matches_mode(|md| matches!(md,$oprs))).unwrap_or(false))
                                    })&&)* (oprs.len()==idx)
                                } {
                                    for prefix in [$(X86EncodingPrefix::$encoding_prefix),*]{
                                        allowed_prefixes[prefix as usize] = Some(prefix);
                                    }
                                }
                            })*
                        }),*
                    }


                    let mut offset = 0;
                    for prefix in allowed_prefixes{
                        if let Some(prefix) = prefix{
                            buffer[offset] = MaybeUninit::new(prefix);
                            offset += 1;
                        }
                    }

                    unsafe{&mut *(&mut buffer[..offset] as *mut [MaybeUninit<X86EncodingPrefix>] as *mut [X86EncodingPrefix])}
                }

                #[allow(dead_code, unused_mut)]
                pub fn encoding_info(&self, oprs: &[X86Operand]) -> Option<X86Encoding>{
                    use X86EncodingMode::*;
                    use X86OperandMode::*;
                    use X86Register::*;
                    use X86RegisterClass::*;
                    use X86RegisterClass::{Cr,Dr};
                    match self{
                        $(Self::[<$mnemonic:camel>] => {
                            match oprs{
                                $(oprs if {
                                    let mut idx = 0;
                                    $(({
                                        (oprs.get({let i = idx; idx += 1; i}).map(|opr| opr.matches_mode(|md| matches!(md,$oprs))).unwrap_or(false))
                                    })&&)* (oprs.len()==idx)
                                } =>Some(X86Encoding{
                                    map: X86InstructionMap::from_opcode($opcode).unwrap(),
                                    base_opcode: ($opcode&0xFF) as u8,
                                    mode: $encoding_mode,
                                    allowed_modes: expand_opt!($(&[$(X86Mode::$modes),*])?),
                                    sse_prefix: SsePrefix::from_opcode($opcode),
                                    imm_size: expand_or_zero!($($immsize)?)
                                }),)*
                                _ => None
                            }
                        })*
                    }
                }
            }

            impl core::fmt::Display for X86CodegenOpcode{
                fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result{
                    match self{
                        $(Self::[<$mnemonic:camel>] => f.write_str(::core::stringify!($mnemonic))),*
                    }
                }
            }
        }
    };
}

x86_codegen_instructions! {
    insn add{
        [TargetRegister(Al), MemOrReg(Byte)] =>  NoPrefix | Rex | Rex2 0x04 @ ModRM(ModRMOptions::NONE);
        [TargetRegister(Ax), MemOrReg(Word)] => NoPrefix | Rex | Rex2 0x05 @ ModRM(ModRMOptions::NONE);
        [TargetRegister(Eax), MemOrReg(Double)] => NoPrefix | Rex | Rex2 0x05 @ ModRM(ModRMOptions::NONE);
        [TargetRegister(Rax), MemOrReg(Double)] => Rex | Rex2 0x05 @ ModRM(ModRMOptions::NONE);
        [MemOrReg(Byte), Reg(Byte)] => NoPrefix | Rex | Rex2 0x00 @ ModRM(ModRMOptions::NONE);
        [MemOrReg(Word | Double | Quad), Reg(Word | Double | Quad)] => NoPrefix | Rex | Rex2  0x01 @ ModRM(ModRMOptions::NONE);
        [Reg(Byte), MemOrReg(Byte)] => NoPrefix | Rex | Rex2  0x02 @ ModRM(ModRMOptions::NONE);
        [Reg(Word | Double | Quad), MemOrReg(Word | Double | Quad)] => NoPrefix | Rex | Rex2 0x03 @ ModRM(ModRMOptions::NONE);
        [MemOrReg(Byte), Immediate(Byte)] => NoPrefix | Rex | Rex2 0x80 imm 1 @ ModRMControl(0,ModRMOptions::NONE);
        [MemOrReg(Word), Immediate(Word)] => NoPrefix | Rex | Rex2 0x81 imm 2 @ ModRMControl(0,ModRMOptions::NONE);
        [MemOrReg(Double | Quad), Immediate(Double)] => NoPrefix | Rex | Rex2 0x81 imm 4 @ ModRMControl(0,ModRMOptions::NONE);
        [MemOrReg(Word | Double | Quad), Immediate(Byte)] => NoPrefix | Rex | Rex2 0x83 imm 1 @ ModRMControl(0,ModRMOptions::NONE);
    }
    insn or{
        [TargetRegister(Al), MemOrReg(Byte)] =>  NoPrefix | Rex | Rex2 0x0C @ ModRM(ModRMOptions::NONE);
        [TargetRegister(Ax), MemOrReg(Word)] => NoPrefix | Rex | Rex2 0x0D @ ModRM(ModRMOptions::NONE);
        [TargetRegister(Eax), MemOrReg(Double)] => NoPrefix | Rex | Rex2 0x0D @ ModRM(ModRMOptions::NONE);
        [TargetRegister(Rax), MemOrReg(Double)] => Rex | Rex2 0x0D @ ModRM(ModRMOptions::NONE);
        [MemOrReg(Byte), Reg(Byte)] => NoPrefix | Rex | Rex2 0x08 @ ModRM(ModRMOptions::NONE);
        [MemOrReg(Word | Double | Quad), Reg(Word | Double | Quad)] => NoPrefix | Rex | Rex2  0x09 @ ModRM(ModRMOptions::NONE);
        [Reg(Byte), MemOrReg(Byte)] => NoPrefix | Rex | Rex2  0x0A @ ModRM(ModRMOptions::NONE);
        [Reg(Word | Double | Quad), MemOrReg(Word | Double | Quad)] => NoPrefix | Rex | Rex2 0x0B @ ModRM(ModRMOptions::NONE);
        [MemOrReg(Byte), Immediate(Byte)] => NoPrefix | Rex | Rex2 0x80 imm 1 @ ModRMControl(1,ModRMOptions::NONE);
        [MemOrReg(Word), Immediate(Word)] => NoPrefix | Rex | Rex2 0x81 imm 2 @ ModRMControl(1,ModRMOptions::NONE);
        [MemOrReg(Double | Quad), Immediate(Double)] => NoPrefix | Rex | Rex2 0x81 imm 4 @ ModRMControl(1,ModRMOptions::NONE);
        [MemOrReg(Word | Double | Quad), Immediate(Byte)] => NoPrefix | Rex | Rex2 0x83 imm 1 @ ModRMControl(1,ModRMOptions::NONE);
    }
    insn adc{
        [TargetRegister(Al), MemOrReg(Byte)] =>  NoPrefix | Rex | Rex2 0x14 @ ModRM(ModRMOptions::NONE);
        [TargetRegister(Ax), MemOrReg(Word)] => NoPrefix | Rex | Rex2 0x15 @ ModRM(ModRMOptions::NONE);
        [TargetRegister(Eax), MemOrReg(Double)] => NoPrefix | Rex | Rex2 0x15 @ ModRM(ModRMOptions::NONE);
        [TargetRegister(Rax), MemOrReg(Double)] => Rex | Rex2 0x15 @ ModRM(ModRMOptions::NONE);
        [MemOrReg(Byte), Reg(Byte)] => NoPrefix | Rex | Rex2 0x10 @ ModRM(ModRMOptions::NONE);
        [MemOrReg(Word | Double | Quad), Reg(Word | Double | Quad)] => NoPrefix | Rex | Rex2  0x11 @ ModRM(ModRMOptions::NONE);
        [Reg(Byte), MemOrReg(Byte)] => NoPrefix | Rex | Rex2  0x12 @ ModRM(ModRMOptions::NONE);
        [Reg(Word | Double | Quad), MemOrReg(Word | Double | Quad)] => NoPrefix | Rex | Rex2 0x13 @ ModRM(ModRMOptions::NONE);
        [MemOrReg(Byte), Immediate(Byte)] => NoPrefix | Rex | Rex2 0x80 imm 1 @ ModRMControl(2,ModRMOptions::NONE);
        [MemOrReg(Word), Immediate(Word)] => NoPrefix | Rex | Rex2 0x81 imm 2 @ ModRMControl(2,ModRMOptions::NONE);
        [MemOrReg(Double | Quad), Immediate(Double)] => NoPrefix | Rex | Rex2 0x81 imm 4 @ ModRMControl(2,ModRMOptions::NONE);
        [MemOrReg(Word | Double | Quad), Immediate(Byte)] => NoPrefix | Rex | Rex2 0x83 imm 1 @ ModRMControl(2,ModRMOptions::NONE);
    }
    insn sbb{
        [TargetRegister(Al), MemOrReg(Byte)] =>  NoPrefix | Rex | Rex2 0x1C @ ModRM(ModRMOptions::NONE);
        [TargetRegister(Ax), MemOrReg(Word)] => NoPrefix | Rex | Rex2 0x1D @ ModRM(ModRMOptions::NONE);
        [TargetRegister(Eax), MemOrReg(Double)] => NoPrefix | Rex | Rex2 0x1D @ ModRM(ModRMOptions::NONE);
        [TargetRegister(Rax), MemOrReg(Double)] => Rex | Rex2 0x1D @ ModRM(ModRMOptions::NONE);
        [MemOrReg(Byte), Reg(Byte)] => NoPrefix | Rex | Rex2 0x18 @ ModRM(ModRMOptions::NONE);
        [MemOrReg(Word | Double | Quad), Reg(Word | Double | Quad)] => NoPrefix | Rex | Rex2  0x19 @ ModRM(ModRMOptions::NONE);
        [Reg(Byte), MemOrReg(Byte)] => NoPrefix | Rex | Rex2  0x1A @ ModRM(ModRMOptions::NONE);
        [Reg(Word | Double | Quad), MemOrReg(Word | Double | Quad)] => NoPrefix | Rex | Rex2 0x1B @ ModRM(ModRMOptions::NONE);
        [MemOrReg(Byte), Immediate(Byte)] => NoPrefix | Rex | Rex2 0x80 imm 1 @ ModRMControl(3,ModRMOptions::NONE);
        [MemOrReg(Word), Immediate(Word)] => NoPrefix | Rex | Rex2 0x81 imm 2 @ ModRMControl(3,ModRMOptions::NONE);
        [MemOrReg(Double | Quad), Immediate(Double)] => NoPrefix | Rex | Rex2 0x81 imm 4 @ ModRMControl(3,ModRMOptions::NONE);
        [MemOrReg(Word | Double | Quad), Immediate(Byte)] => NoPrefix | Rex | Rex2 0x83 imm 1 @ ModRMControl(3,ModRMOptions::NONE);
    }
    insn and{
        [TargetRegister(Al), MemOrReg(Byte)] =>  NoPrefix | Rex | Rex2 0x24 @ ModRM(ModRMOptions::NONE);
        [TargetRegister(Ax), MemOrReg(Word)] => NoPrefix | Rex | Rex2 0x25 @ ModRM(ModRMOptions::NONE);
        [TargetRegister(Eax), MemOrReg(Double)] => NoPrefix | Rex | Rex2 0x25 @ ModRM(ModRMOptions::NONE);
        [TargetRegister(Rax), MemOrReg(Double)] => Rex | Rex2 0x25 @ ModRM(ModRMOptions::NONE);
        [MemOrReg(Byte), Reg(Byte)] => NoPrefix | Rex | Rex2 0x20 @ ModRM(ModRMOptions::NONE);
        [MemOrReg(Word | Double | Quad), Reg(Word | Double | Quad)] => NoPrefix | Rex | Rex2  0x21 @ ModRM(ModRMOptions::NONE);
        [Reg(Byte), MemOrReg(Byte)] => NoPrefix | Rex | Rex2  0x22 @ ModRM(ModRMOptions::NONE);
        [Reg(Word | Double | Quad), MemOrReg(Word | Double | Quad)] => NoPrefix | Rex | Rex2 0x23 @ ModRM(ModRMOptions::NONE);
        [MemOrReg(Byte), Immediate(Byte)] => NoPrefix | Rex | Rex2 0x80 imm 1 @ ModRMControl(4,ModRMOptions::NONE);
        [MemOrReg(Word), Immediate(Word)] => NoPrefix | Rex | Rex2 0x81 imm 2 @ ModRMControl(4,ModRMOptions::NONE);
        [MemOrReg(Double | Quad), Immediate(Double)] => NoPrefix | Rex | Rex2 0x81 imm 4 @ ModRMControl(4,ModRMOptions::NONE);
        [MemOrReg(Word | Double | Quad), Immediate(Byte)] => NoPrefix | Rex | Rex2 0x83 imm 1 @ ModRMControl(4,ModRMOptions::NONE);

    }
    insn sub{
        [TargetRegister(Al), MemOrReg(Byte)] =>  NoPrefix | Rex | Rex2 0x2C @ ModRM(ModRMOptions::NONE);
        [TargetRegister(Ax), MemOrReg(Word)] => NoPrefix | Rex | Rex2 0x2D @ ModRM(ModRMOptions::NONE);
        [TargetRegister(Eax), MemOrReg(Double)] => NoPrefix | Rex | Rex2 0x2D @ ModRM(ModRMOptions::NONE);
        [TargetRegister(Rax), MemOrReg(Double)] => Rex | Rex2 0x2D @ ModRM(ModRMOptions::NONE);
        [MemOrReg(Byte), Reg(Byte)] => NoPrefix | Rex | Rex2 0x28 @ ModRM(ModRMOptions::NONE);
        [MemOrReg(Word | Double | Quad), Reg(Word | Double | Quad)] => NoPrefix | Rex | Rex2  0x29 @ ModRM(ModRMOptions::NONE);
        [Reg(Byte), MemOrReg(Byte)] => NoPrefix | Rex | Rex2  0x2A @ ModRM(ModRMOptions::NONE);
        [Reg(Word | Double | Quad), MemOrReg(Word | Double | Quad)] => NoPrefix | Rex | Rex2 0x2B @ ModRM(ModRMOptions::NONE);
        [MemOrReg(Byte), Immediate(Byte)] => NoPrefix | Rex | Rex2 0x80 imm 1 @ ModRMControl(5,ModRMOptions::NONE);
        [MemOrReg(Word), Immediate(Word)] => NoPrefix | Rex | Rex2 0x81 imm 2 @ ModRMControl(5,ModRMOptions::NONE);
        [MemOrReg(Double | Quad), Immediate(Double)] => NoPrefix | Rex | Rex2 0x81 imm 4 @ ModRMControl(5,ModRMOptions::NONE);
        [MemOrReg(Word | Double | Quad), Immediate(Byte)] => NoPrefix | Rex | Rex2 0x83 imm 1 @ ModRMControl(5,ModRMOptions::NONE);
    }
    insn xor{
        [TargetRegister(Al), MemOrReg(Byte)] =>  NoPrefix | Rex | Rex2 0x34 @ ModRM(ModRMOptions::NONE);
        [TargetRegister(Ax), MemOrReg(Word)] => NoPrefix | Rex | Rex2 0x35 @ ModRM(ModRMOptions::NONE);
        [TargetRegister(Eax), MemOrReg(Double)] => NoPrefix | Rex | Rex2 0x35 @ ModRM(ModRMOptions::NONE);
        [TargetRegister(Rax), MemOrReg(Double)] => Rex | Rex2 0x35 @ ModRM(ModRMOptions::NONE);
        [MemOrReg(Byte), Reg(Byte)] => NoPrefix | Rex | Rex2 0x30 @ ModRM(ModRMOptions::NONE);
        [MemOrReg(Word | Double | Quad), Reg(Word | Double | Quad)] => NoPrefix | Rex | Rex2  0x31 @ ModRM(ModRMOptions::NONE);
        [Reg(Byte), MemOrReg(Byte)] => NoPrefix | Rex | Rex2  0x32 @ ModRM(ModRMOptions::NONE);
        [Reg(Word | Double | Quad), MemOrReg(Word | Double | Quad)] => NoPrefix | Rex | Rex2 0x33 @ ModRM(ModRMOptions::NONE);
        [MemOrReg(Byte), Immediate(Byte)] => NoPrefix | Rex | Rex2 0x80 imm 1 @ ModRMControl(6,ModRMOptions::NONE);
        [MemOrReg(Word), Immediate(Word)] => NoPrefix | Rex | Rex2 0x81 imm 2 @ ModRMControl(6,ModRMOptions::NONE);
        [MemOrReg(Double | Quad), Immediate(Double)] => NoPrefix | Rex | Rex2 0x81 imm 4 @ ModRMControl(6,ModRMOptions::NONE);
        [MemOrReg(Word | Double | Quad), Immediate(Byte)] => NoPrefix | Rex | Rex2 0x83 imm 1 @ ModRMControl(6,ModRMOptions::NONE);

    }
    insn cmp{
        [TargetRegister(Al), MemOrReg(Byte)] =>  NoPrefix | Rex | Rex2 0x3C @ ModRM(ModRMOptions::NONE);
        [TargetRegister(Ax), MemOrReg(Word)] => NoPrefix | Rex | Rex2 0x3D @ ModRM(ModRMOptions::NONE);
        [TargetRegister(Eax), MemOrReg(Double)] => NoPrefix | Rex | Rex2 0x3D @ ModRM(ModRMOptions::NONE);
        [TargetRegister(Rax), MemOrReg(Double)] => Rex | Rex2 0x3D @ ModRM(ModRMOptions::NONE);
        [MemOrReg(Byte), Reg(Byte)] => NoPrefix | Rex | Rex2 0x38 @ ModRM(ModRMOptions::NONE);
        [MemOrReg(Word | Double | Quad), Reg(Word | Double | Quad)] => NoPrefix | Rex | Rex2  0x39 @ ModRM(ModRMOptions::NONE);
        [Reg(Byte), MemOrReg(Byte)] => NoPrefix | Rex | Rex2  0x3A @ ModRM(ModRMOptions::NONE);
        [Reg(Word | Double | Quad), MemOrReg(Word | Double | Quad)] => NoPrefix | Rex | Rex2 0x3B @ ModRM(ModRMOptions::NONE);
        [MemOrReg(Byte), Immediate(Byte)] => NoPrefix | Rex | Rex2 0x80 imm 1 @ ModRMControl(7,ModRMOptions::NONE);
        [MemOrReg(Word), Immediate(Word)] => NoPrefix | Rex | Rex2 0x81 imm 2 @ ModRMControl(7,ModRMOptions::NONE);
        [MemOrReg(Double | Quad), Immediate(Double)] => NoPrefix | Rex | Rex2 0x81 imm 4 @ ModRMControl(7,ModRMOptions::NONE);
        [MemOrReg(Word | Double | Quad), Immediate(Byte)] => NoPrefix | Rex | Rex2 0x83 imm 1 @ ModRMControl(7,ModRMOptions::NONE);
    }
    insn push{
        [TargetRegister(Es)] in Real | Protected => NoPrefix 0x06 @ OpcodeOnly;
        [TargetRegister(Cs)] in Real | Protected => NoPrefix 0x0E @ OpcodeOnly;
        [TargetRegister(Ss)] in Real | Protected => NoPrefix 0x16 @ OpcodeOnly;
        [TargetRegister(Ds)] in Real | Protected => NoPrefix 0x1E @ OpcodeOnly;
        [TargetRegister(Fs)] => NoPrefix 0x0FA0 @ OpcodeOnly;
        [TargetRegister(Gs)] => NoPrefix 0x0FA8 @ OpcodeOnly;
        [Reg(Word | Double | Quad)] => NoPrefix | Rex | Rex2 0x50 @ OpReg(OpRegOptions::NO_REX_W);
        [Immediate(Word | Double)] => NoPrefix 0x68 @ OpcodeOnly;
        [Immediate(Byte)] => NoPrefix 0x6A @ OpcodeOnly;
        [MemOrReg(Word | Double | Quad)] => NoPrefix | Rex | Rex2 0xFF @ ModRMControl(6,ModRMOptions::NO_REX_W);
    }
    insn pop{
        [TargetRegister(Es)] in Real | Protected => NoPrefix 0x07 @ OpcodeOnly;
        [TargetRegister(Ss)] in Real | Protected => NoPrefix 0x17 @ OpcodeOnly;
        [TargetRegister(Ds)] in Real | Protected => NoPrefix 0x1F @ OpcodeOnly;
        [TargetRegister(Fs)] => NoPrefix 0x0FA1 @ OpcodeOnly;
        [TargetRegister(Gs)] => NoPrefix 0x0FA9 @ OpcodeOnly;
        [Reg(Word | Double | Quad)] => NoPrefix | Rex | Rex2 0x58 @ OpReg(OpRegOptions::NO_REX_W);
    }
    insn pusha{
        [] in Real | Protected => NoPrefix 0x60 @ OpcodeOnly;
    }
    insn popa{
        [] in Real | Protected => NoPrefix 0x61 @ OpcodeOnly;
    }
    insn bound{
        [Reg(Word | Double), MemoryOnly(Word | Double)] in Real | Protected => NoPrefix 0x62 @ ModRM(ModRMOptions::NONE);
    }
    insn arpl{
        [MemOrReg(Word), Reg(Word)] in Real | Protected => NoPrefix 0x63 @ ModRM(ModRMOptions::NO_ESCAPE);
    }
    insn movsx{
        [Reg(Double | Quad), MemOrReg(Double)] in Long => NoPrefix 0x63 @ ModRM(ModRMOptions::IGNORE_SIZE_MISMATCH);
    }
    insn daa{
        [TargetRegister(Al)] in Real | Protected => NoPrefix 0x27 @ OpcodeOnly;
        [] in Real | Protected => NoPrefix 0x27 @ OpcodeOnly;
    }
    insn das{
        [TargetRegister(Al)] in Real | Protected => NoPrefix 0x2F @ OpcodeOnly;
        [] in Real | Protected => NoPrefix 0x2F @ OpcodeOnly;
    }
    insn aaa{
        [TargetRegister(Al)] in Real | Protected => NoPrefix 0x37 @ OpcodeOnly;
        [] in Real | Protected => NoPrefix 0x37 @ OpcodeOnly;
    }
    insn aas{
        [TargetRegister(Al)] in Real | Protected => NoPrefix 0x3F @ OpcodeOnly;
        [] in Real | Protected => NoPrefix 0x3F @ OpcodeOnly;
    }
    insn inc{
        [Reg(Word | Double)] in Real | Protected => NoPrefix 0x40 @ OpReg(OpRegOptions::NONE);
        [MemOrReg(Byte)] => NoPrefix | Rex | Rex2 0xFE @ ModRMControl(0,ModRMOptions::NONE);
        [MemOrReg(Word | Double | Quad)] => NoPrefix | Rex | Rex2 0xFF @ ModRMControl(0, ModRMOptions::NONE);
    }
    insn dec{
        [Reg(Word | Double)] in Real | Protected => NoPrefix 0x48 @ OpReg(OpRegOptions::NONE);
        [MemOrReg(Byte)] => NoPrefix | Rex | Rex2 0xFE @ ModRMControl(1,ModRMOptions::NONE);
        [MemOrReg(Word | Double | Quad)] => NoPrefix | Rex | Rex2 0xFF @ ModRMControl(1, ModRMOptions::NONE);
    }
    insn imul{
        [MemOrReg(Word), Reg(Word), Immediate(Word)] => NoPrefix | Rex | Rex2 0x69 @ ModRM(ModRMOptions::NONE);
        [MemOrReg(Double | Quad), Reg(Double | Quad), Immediate(Double) ] => NoPrefix | Rex | Rex2 0x69 @ ModRM(ModRMOptions::NONE);
        [MemOrReg(Word | Double | Quad), Reg(Word | Double | Quad), Immediate(Byte)] => NoPrefix | Rex | Rex2 0x6B @ ModRM(ModRMOptions::NONE);
        [MemOrReg(Byte)] => NoPrefix | Rex | Rex2 0xF6 @ ModRMControl(5,ModRMOptions::NONE);
        [MemOrReg(Word | Double | Quad)] => NoPrefix | Rex | Rex2 0xF7 @ ModRMControl(5,ModRMOptions::NONE);
    }
    insn ins{
        [StringAddr(Byte, Di | Edi | Rdi)] => NoPrefix 0x6C @ OpcodeOnly;
        [StringAddr(Word | Double, Di | Edi | Rdi)] => NoPrefix | Rex | Rex2 0x6D @ OpcodeOnly;
    }
    insn outs{
        [StringAddr(Byte, Si | Esi | Edi)] => NoPrefix 0x6E @ OpcodeOnly;
        [StringAddr(Word | Double, Si | Esi | Edi)] => NoPrefix | Rex | Rex2 0x6F @ OpcodeOnly;
    }
    insn jo {
        [RelAddr(8)] => NoPrefix 0x70 @ OpcodeOnly;
    }
    insn jno{
        [RelAddr(8)] => NoPrefix 0x71 @ OpcodeOnly;
    }
    insn jb{
        [RelAddr(8)] => NoPrefix 0x72 @ OpcodeOnly;
    }
    insn jnb{
        [RelAddr(8)] => NoPrefix 0x73 @ OpcodeOnly;
    }
    insn jz {
        [RelAddr(8)] => NoPrefix 0x74 @ OpcodeOnly;
    }
    insn jnz{
        [RelAddr(8)] => NoPrefix 0x75 @ OpcodeOnly;
    }
    insn jbe{
        [RelAddr(8)] => NoPrefix 0x76 @ OpcodeOnly;
    }
    insn jnbe{
        [RelAddr(8)] => NoPrefix 0x77 @ OpcodeOnly;
    }
    insn js{
        [RelAddr(8)] => NoPrefix 0x78 @ OpcodeOnly;
    }
    insn jns{
        [RelAddr(8)] => NoPrefix 0x79 @ OpcodeOnly;
    }
    insn jp{
        [RelAddr(8)] => NoPrefix 0x7A @ OpcodeOnly;
    }
    insn jnp{
        [RelAddr(8)] => NoPrefix 0x7B @ OpcodeOnly;
    }
    insn jl{
        [RelAddr(8)] => NoPrefix 0x7C @ OpcodeOnly;
    }
    insn jnl{
        [RelAddr(8)] => NoPrefix 0x7D @ OpcodeOnly;
    }
    insn jle{
        [RelAddr(8)] => NoPrefix 0x7E @ OpcodeOnly;
    }
    insn jnle{
        [RelAddr(8)] => NoPrefix 0x7F @ OpcodeOnly;
    }
    insn test{
        [MemOrReg(Byte),Reg(Byte)] => NoPrefix | Rex | Rex2 0x84 @ ModRM(ModRMOptions::NONE);
        [MemOrReg(Word | Double | Quad), Reg(Word | Double | Quad)] => NoPrefix | Rex | Rex2 0x85 @ ModRM(ModRMOptions::NONE);
    }
    insn nop{
        [] => NoPrefix | Rex | Rex2 0x90 @ OpcodeOnly;
    }
    insn xchg{
        [Reg(Byte), MemOrReg(Byte)] => NoPrefix | Rex | Rex2 0x86 @ ModRM(ModRMOptions::NONE);
        [Reg(Word), TargetRegister(Ax)] => NoPrefix | Rex | Rex2 0x90 @ OpReg(OpRegOptions::NONE);
        [Reg(Double), TargetRegister(Eax)] => NoPrefix | Rex | Rex2 0x90 @ OpReg(OpRegOptions::NONE);
        [Reg(Quad), TargetRegister(Rax)] => Rex | Rex2 0x90 @ OpReg(OpRegOptions::NONE);
    }
    insn fwait{
        [] => NoPrefix 0x9B @ OpcodeOnly;
    }
    insn pushf{
        [] => NoPrefix 0x9C @ OpcodeWithSize(Word);
    }
    insn pushfd{
        [] in Real | Protected => NoPrefix 0x9C @ OpcodeWithSize(Double);
    }
    insn pushfq{
        [] in Long => NoPrefix 0x9C @ OpcodeOnly;
    }
    insn popf{
        [] => NoPrefix 0x9D @ OpcodeWithSize(Word);
    }
    insn popfd{
        [] in Real | Protected => NoPrefix 0x9D @ OpcodeWithSize(Double);
    }
    insn popfq{
        [] in Long => NoPrefix 0x9D @ OpcodeOnly;
    }
    insn sahf{
        [] => NoPrefix 0x9E @ OpcodeOnly;
    }
    insn lahf{
        [] => NoPrefix 0x9F @ OpcodeOnly;
    }
    insn mov{
        [TargetRegister(Al), MemOffset(Byte)] => NoPrefix 0xA0 @ OffsetImm(32);
        [TargetRegister(Ax), MemOffset(Word)] => NoPrefix 0xA1 @ OffsetImm(32);
        [TargetRegister(Eax), MemOffset(Double)] => NoPrefix 0xA1 @ OffsetImm(32);
        [TargetRegister(Rax), MemOffset(Quad)] => Rex | Rex2 0xA1 @ OffsetImm(32);
        [MemOffset(Byte), TargetRegister(Al)] => NoPrefix 0xA2 @ OffsetImm(32);
        [MemOffset(Word), TargetRegister(Ax)] => NoPrefix 0xA3 @ OffsetImm(32);
        [MemOffset(Double), TargetRegister(Eax)] => NoPrefix 0xA3 @ OffsetImm(32);
        [MemOffset(Quad), TargetRegister(Rax)] => Rex | Rex2 0xA3 @ OffsetImm(32);
        [MemOrReg(Byte), Reg(Byte)] => NoPrefix | Rex | Rex2 0x88 @ ModRM(ModRMOptions::NONE);
        [MemOrReg(Word | Double | Quad), Reg(Word | Double | Quad)] => NoPrefix | Rex | Rex2 0x89 @ ModRM(ModRMOptions::NONE);
        [Reg(Byte), MemOrReg(Byte)] => NoPrefix | Rex | Rex2 0x8A @ ModRM(ModRMOptions::NONE);
        [Reg(Word | Double | Quad), MemOrReg(Word | Double | Quad)] => NoPrefix | Rex | Rex2 0x8B @ ModRM(ModRMOptions::NONE);
        [Reg(Byte), Immediate(Byte)] => NoPrefix | Rex | Rex2 0xB0 imm 1 @ OpReg(OpRegOptions::NONE);
        [Reg(Word), Immediate(Word)] => NoPrefix | Rex | Rex2 0xB8 imm 2 @ OpReg(OpRegOptions::NONE);
        [Reg(Double), Immediate(Double)] => NoPrefix | Rex | Rex2 0xB8 imm 4 @ OpReg(OpRegOptions::NONE);
        [MemOrReg(Byte), Immediate(Byte)] => NoPrefix | Rex | Rex2 0xC6 imm 1 @ ModRMControl(0,ModRMOptions::NONE);
        [MemOrReg(Word), Immediate(Word)] => NoPrefix | Rex | Rex2 0xC7 imm 2 @ ModRMControl(0,ModRMOptions::NONE);
        [MemOrReg(Double | Quad), Immediate(Double)] => NoPrefix | Rex | Rex2 0xC7 imm 4 @ ModRMControl(0,ModRMOptions::NONE);
        [Reg(Quad), Immediate(Quad)] => NoPrefix | Rex | Rex2 0xB8 @ OpReg(OpRegOptions::NONE);
        [RegRM(Word | Double | Quad), Reg(Sreg)] => NoPrefix | Rex | Rex2 0x8C @ ModRM(ModRMOptions::NONE);
        [MemoryOnly(Word), Reg(Sreg)] => NoPrefix | Rex | Rex2 0x8C @ ModRM(ModRMOptions::NO_ESCAPE);
        [Reg(Sreg), RegRM(Word | Double | Quad)] => NoPrefix | Rex | Rex2 0x8E @ ModRM(ModRMOptions::NONE);
        [Reg(Sreg), MemoryOnly(Word)] => NoPrefix | Rex | Rex2 0x8E @ ModRM(ModRMOptions::NO_ESCAPE);
        [RegRM(Double | Quad), Reg(Cr)] => NoPrefix | Rex | Rex2 0x0F20 @ ModRM(ModRMOptions::NO_REX_W);
        [RegRM(Double | Quad), Reg(Dr)] => NoPrefix | Rex | Rex2 0x0F21 @ ModRM(ModRMOptions::NO_REX_W);
        [Reg(Cr), RegRM(Double | Quad)] => NoPrefix | Rex | Rex2 0x0F22 @ ModRM(ModRMOptions::NO_REX_W);
        [Reg(Dr), RegRM(Double | Quad)] => NoPrefix | Rex | Rex2 0x0F23 @ ModRM(ModRMOptions::NO_REX_W);
    }
    insn enter{
        [Immediate(Word), Immediate(Byte)] => NoPrefix 0xC8 @ OpcodeOnly;
    }
    insn leave{
        [] => NoPrefix 0xC9 @ OpcodeOnly;
    }
    insn ret{
        [Immediate(Word)] => NoPrefix 0xC2 imm 2 @ OpcodeOnly;
        [] => NoPrefix 0xC3 @ OpcodeOnly;
    }
    insn retf{
        [Immediate(Word)] => NoPrefix 0xCA @ OpcodeOnly;
        [] => NoPrefix 0xCB @ OpcodeOnly;
    }
    insn int3{
        [] => NoPrefix 0xCC @ OpcodeOnly;
    }
    insn int{
        [Immediate(Byte)] => NoPrefix 0xCD @ OpcodeOnly;
    }
    insn into{
        [] => NoPrefix 0xCE @ OpcodeOnly;
    }
    insn iret{
        [] => NoPrefix 0xCF @ OpcodeOnly;
    }
    insn iretq{
        [] => Rex | Rex2 0xCF @ OpcodeWithSize(Quad);
    }
    insn ud2{
        [] => NoPrefix 0x0F0B @ OpcodeOnly;
    }
    insn rol{
        [MemOrReg(Byte)] => NoPrefix | Rex | Rex2 0xD0 @ ModRMControl(0,ModRMOptions::NONE);
        [MemOrReg(Word | Double | Quad)] => NoPrefix | Rex | Rex2 0xD1 @ ModRMControl(0,ModRMOptions::NONE);
        [MemOrReg(Byte), Immediate(Byte)] => NoPrefix | Rex | Rex2 0xC0 @ ModRMControl(0,ModRMOptions::NONE);
        [MemOrReg(Word | Double | Quad), Immediate(Byte)] => NoPrefix | Rex | Rex2 0xC1 @ ModRMControl(1,ModRMOptions::NONE);

    }
    insn call{
        [RelAddr(16)] => NoPrefix 0xE8 @ OffsetImm(16);
        [RelAddr(32)] => NoPrefix 0xE8 @ OffsetImm(32);
        [MemOrReg(Word | Double | Quad)] => NoPrefix | Rex | Rex2 0xFF @ ModRMControl(2,ModRMOptions::NONE);
    }
    insn lea{
        [Reg(Word | Double | Quad), MemoryOnly(_)] => NoPrefix | Rex | Rex2 0x8D @ ModRM(ModRMOptions::IGNORE_SIZE_MISMATCH);
    }
    insn jmp{
        [RelAddr(8)] => NoPrefix 0xEA @ OffsetImm(8);
        [RelAddr(16)] => NoPrefix 0xE9 @ OffsetImm(16);
        [RelAddr(32)] => NoPrefix 0xE9 @ OffsetImm(32);
        [MemOrReg(Word | Double | Quad)] => NoPrefix | Rex | Rex2 0xFF @ ModRMControl(4,ModRMOptions::NONE);
    }
}
