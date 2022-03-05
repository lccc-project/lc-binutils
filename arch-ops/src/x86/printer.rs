use super::{
    insn::{ModRM, X86Instruction, X86Mode},
    X86RegisterClass,
};
use core::ops::{Deref, DerefMut};

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Syntax {
    Intel,
    Att(bool),
}

pub struct InsnPrinter<W> {
    syntax: Syntax,
    mode: X86Mode,
    writer: W,
}

impl<W> InsnPrinter<W> {
    pub const fn new(writer: W, mode: X86Mode, syntax: Syntax) -> Self {
        Self {
            writer,
            mode,
            syntax,
        }
    }

    pub fn into_inner(self) -> W {
        self.writer
    }

    pub fn set_mode(&mut self, mode: X86Mode) {
        self.mode = mode;
    }

    pub fn mode(&self) -> X86Mode {
        self.mode
    }

    pub fn set_syntax(&mut self, syntax: Syntax) {
        self.syntax = syntax
    }

    pub fn syntax(&self) -> Syntax {
        self.syntax
    }
}

impl<W> Deref for InsnPrinter<W> {
    type Target = W;
    fn deref(&self) -> &W {
        &self.writer
    }
}

impl<W> DerefMut for InsnPrinter<W> {
    fn deref_mut(&mut self) -> &mut W {
        &mut self.writer
    }
}

impl<W: core::fmt::Write> InsnPrinter<W> {
    pub fn write_insn(&mut self, insn: &X86Instruction) -> core::fmt::Result {
        let mode = insn.mode_override().unwrap_or(self.mode);

        match self.syntax {
            Syntax::Intel => {
                let mut addr_mode = mode.largest_gpr();
                for op in insn.operands() {
                    match op {
                        super::insn::X86Operand::ModRM(modrm)
                        | super::insn::X86Operand::FarModRM { modrm, .. } => match modrm {
                            ModRM::Indirect { mode, .. }
                            | ModRM::IndirectDisp8 { mode, .. }
                            | ModRM::IndirectDisp32 { mode, .. } => match mode {
                                super::insn::ModRMRegOrSib::Reg(r) => addr_mode = r.class(),
                                super::insn::ModRMRegOrSib::Abs(_) => todo!(),
                                super::insn::ModRMRegOrSib::RipRel(_) => todo!(),
                                super::insn::ModRMRegOrSib::Sib { base, .. } => {
                                    addr_mode = base.class()
                                }
                                super::insn::ModRMRegOrSib::Index16 { .. } => {
                                    addr_mode = X86RegisterClass::Word
                                }
                            },
                            _ => {}
                        },
                        _ => {}
                    }
                }

                if addr_mode != mode.largest_gpr() {
                    self.writer
                        .write_fmt(format_args!("addr{} ", addr_mode.size(mode) * 8))?;
                }
                self.writer.write_fmt(format_args!("{} ", insn.opcode()))?;
            }
            Syntax::Att(_) => todo!("att_syntax"),
        }
        Ok(())
    }
}
