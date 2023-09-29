use {
    super::{Instruction, Operands},
    crate::{
        holeybytes::{
            OpsA, OpsO, OpsP, OpsRRA, OpsRRAH, OpsRRO, OpsRROH, OpsRRP, OpsRRPH, Relative16,
            Relative32,
        },
        traits::InsnWrite,
    },
    delegate::delegate,
    std::io::Write,
};

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
                    $(Operands::$transty(op) => $this.write_all(&op.encode()),)*
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
                Operands::OpsRRA(OpsRRA(r0, r1, addr)) => {
                    self.write_all(&[r0.0, r1.0])?;
                    self.write_addr(64, addr, false)
                },
                Operands::OpsRRAH(OpsRRAH(r0, r1, addr, i0)) => {
                    self.write_all(&[r0.0, r1.0])?;
                    self.write_addr(64, addr, false)?;
                    self.write_all(&i0.to_le_bytes())
                },
                Operands::OpsRROH(OpsRROH(r0, r1, Relative32(addr), i0)) => {
                    self.write_all(&[r0.0, r1.0])?;
                    self.write_addr(32, addr, true)?;
                    self.write_all(&i0.to_le_bytes())
                },
                Operands::OpsRRPH(OpsRRPH(r0, r1, Relative16(addr), i0)) => {
                    self.write_all(&[r0.0, r1.0])?;
                    self.write_addr(16, addr, true)?;
                    self.write_all(&i0.to_le_bytes())
                },
                Operands::OpsRRO(OpsRRO(r0, r1, Relative32(addr))) => {
                    self.write_all(&[r0.0, r1.0])?;
                    self.write_addr(32, addr, true)
                },
                Operands::OpsRRP(OpsRRP(r0, r1, Relative16(addr))) => {
                    self.write_all(&[r0.0, r1.0])?;
                    self.write_addr(16, addr, true)
                },
                Operands::OpsA(OpsA(addr)) => {
                    self.write_addr(64, addr, false)
                },

                Operands::OpsO(OpsO(Relative32(addr))) => {
                    self.write_addr(32, addr, true)
                },
                Operands::OpsP(OpsP(Relative16(addr))) => {
                    self.write_addr(16, addr, true)
                },
            };
        }
    }
}
