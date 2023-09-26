use super::Operand;

use {super::Instruction, crate::traits::InsnWrite, delegate::delegate, std::io::Write};

pub struct HbEncoder<W> {
    inner: W,
}

impl<W> HbEncoder<W> {
    pub const fn new(inner: W) -> Self {
        Self { inner }
    }

    pub fn into_inner(self) -> W {
        self.inner
    }

    pub fn inner(&self) -> &W {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut W {
        &mut self.inner
    }
}

impl<W: Write> Write for HbEncoder<W> {
    delegate!(to self.inner {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize>;
        fn flush(&mut self) -> std::io::Result<()>;
    });
}

impl<W: InsnWrite> InsnWrite for HbEncoder<W> {
    delegate!(to self.inner {
        fn write_addr(&mut self, size: usize, addr: crate::traits::Address, rel: bool) -> std::io::Result<()>;
        fn write_reloc(&mut self, reloc: crate::traits::Reloc) -> std::io::Result<()>;
        fn offset(&self) -> usize;
    });
}

impl<W: InsnWrite> HbEncoder<W> {
    pub fn write_instruction(&mut self, instruction: Instruction) -> std::io::Result<()> {
        self.write_all(&[instruction.opcode() as u8])?;
        match instruction.operand() {
            Operand::OpsRR(_) => todo!(),
            Operand::OpsRRR(_) => todo!(),
            Operand::OpsRRRR(_) => todo!(),
            Operand::OpsRRB(_) => todo!(),
            Operand::OpsRRH(_) => todo!(),
            Operand::OpsRRW(_) => todo!(),
            Operand::OpsRD(_) => todo!(),
            Operand::OpsRRD(_) => todo!(),
            Operand::OpsRRA(_) => todo!(),
            Operand::OpsRRAH(_) => todo!(),
            Operand::OpsRROH(_) => todo!(),
            Operand::OpsRRO(_) => todo!(),
            Operand::OpsRRP(_) => todo!(),
            Operand::OpsA(_) => todo!(),
            Operand::OpsO(_) => todo!(),
            Operand::OpsN(_) => todo!(),
        }
        
        Ok(())
    }
}
