use super::Operands;

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
        /// Write operands
        /// 
        /// # Syntax
        /// - `match <expr>`              – match on operand
        /// - `this: <expr>`              – `self`
        /// - `transmute: [<type>, …]`    – types to soundly bytecast and write
        /// - `else: { <typical match> }` – match on different types to manually write
        macro_rules! opwrite {
            (
                match $match_on:expr;
                this:      $this:expr;
                transmute: [$($transty:ident),* $(,)?];
                else:      {$($pat:pat => $expr:expr),* $(,)?};
            ) => {
                match $match_on {
                    $(Operands::$transty(op) => $this.write_all(&op.encode())?,)*
                    $($pat => $expr),*
                }
            };
        }

        let (opcode, operands) = instruction.into_pair();
        self.write_all(&[opcode as u8])?;

        opwrite! {
            match operands;
            this: self;
            transmute: [OpsRR, OpsRRR, OpsRRRR, OpsRRB, OpsRRH, OpsRRW, OpsRD, OpsRRD, OpsN];
            else: {
                Operands::OpsRRA(_) => todo!(),
                Operands::OpsRRAH(_) => todo!(),
                Operands::OpsRROH(_) => todo!(),
                Operands::OpsRRO(_) => todo!(),
                Operands::OpsRRP(_) => todo!(),
                Operands::OpsA(_) => todo!(),
                Operands::OpsO(_) => todo!(),
            };
        };

        Ok(())
    }
}
