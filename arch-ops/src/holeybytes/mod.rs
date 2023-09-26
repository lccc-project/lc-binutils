use std::convert::TryFrom;

pub mod codec;

with_builtin_macros::with_builtin! {
    let $spec = include_from_root!("src/holeybytes/instructions.in") in {
        macro_rules! invoke_with_def {
            ($macro:path) => {
                $macro! { $spec }
            };
        }
    }
}

macro_rules! ignore_const_one {
    ($_:ident) => {
        1
    };
}

/// Create opcode enum from definition
macro_rules! opcodes {
    ($($opcode:expr, $mnemonic:ident, $_ty:ident, $doc:literal;)*) => {
        paste::paste! {
            #[derive(Clone, Copy, Debug, PartialEq, Eq)]
            #[repr(u8)]
            pub enum Opcode {
                $(
                    #[doc = $doc]
                    [<$mnemonic:camel>] = $opcode
                ),*
            }

            impl TryFrom<u8> for Opcode {
                type Error = ();
            
                fn try_from(value: u8) -> Result<Self, Self::Error> {
                    const INST_HIGH: u8 = 0 $(+ ignore_const_one!($mnemonic))*;
                    if value < INST_HIGH {
                        #[allow(unsafe_code)]
                        Ok(unsafe { std::mem::transmute(value) })
                    } else {
                        Err(())
                    }
                }
            }
        }
    };
}

invoke_with_def!(opcodes);

macro_rules! operands {
    ($($name:ident $inner:tt),* $(,)?) => {
        $(
            #[derive(Clone, Copy, Debug, PartialEq, Eq)]
            #[repr(transparent)]
            pub struct $name $inner;
        )*
    };
}

/// Verify if operands defined in spec do exist
macro_rules! verify_ops {
    ($($_o:expr, $mnemonic:ident, $ty:ident, $_d:literal;)*) => {
        mod __verify_ops {
            #![allow(
                clippy::upper_case_acronyms,
                unused,
            )]
            paste::paste!($(type $mnemonic = super::[<Ops $ty>];)*);
        }
    };
}

operands!(
    Register(pub u8),
    Address(pub u64),
    Relative32(pub u32),
    Relative16(pub u16),
);

macro_rules! define_operands {
    ($($name:ident ($($item:ident),* $(,)?)),* $(,)?) => {
        $(
            #[derive(Clone, Copy, Debug, PartialEq, Eq)]
            #[repr(packed)]
            pub struct $name($(pub $item),*);
        )*

        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum Operand {
            $($name($name)),*
        }
    };
}

define_operands! {
    OpsRR   (Register  , Register                      ),
    OpsRRR  (Register  , Register, Register            ),
    OpsRRRR (Register  , Register, Register  , Register),
    OpsRRB  (Register  , Register, u8                  ),
    OpsRRH  (Register  , Register, u16                 ),
    OpsRRW  (Register  , Register, u32                 ),
    OpsRD   (Register  , u64                           ),
    OpsRRD  (Register  , Register, u64                 ),
    OpsRRA  (Register  , Register, Address             ),
    OpsRRAH (Register  , Register, Address   , u16     ),
    OpsRROH (Register  , Register, Relative32, u16     ),
    OpsRRO  (Register  , Register, Relative32          ),
    OpsRRP  (Register  , Register, Relative16          ),
    OpsA    (Address                                   ),
    OpsO    (Relative32                                ),
    OpsN    (                                          ),
}

invoke_with_def!(verify_ops);

/// Validate if passed [`Operands`] is correct for passed [`Opcode`]
pub const fn validate_ops_for_opcode(opcode: Opcode, operand: &Operand) -> bool {
    macro_rules! generate {
        ($($_o:expr, $mnemonic:ident, $ty:ident, $_d:literal;)*) => {
            paste::paste! {
                match opcode {
                    $(Opcode::[<$mnemonic:camel>] =>
                        matches!(operand, Operand::[<Ops $ty>] { .. })),*
                }
            }
        };
    }

    invoke_with_def! { generate }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Instruction {
    opcode: Opcode,
    operand: Operand,
}

impl Instruction {
    pub const fn new(opcode: Opcode, operand: Operand) -> Option<Self> {
        if validate_ops_for_opcode(opcode, &operand) {
            Some(Self::new_unchecked(opcode, operand))
        } else {
            None
        }
    }

    pub const fn new_unchecked(opcode: Opcode, operand: Operand) -> Self {
        Self { opcode, operand }
    }

    #[inline]
    pub fn opcode(&self) -> Opcode {
        self.opcode
    }

    #[inline]
    pub fn operand(&self) -> Operand {
        self.operand
    }
}
