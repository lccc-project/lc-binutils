use std::convert::TryFrom;

pub mod codec;

with_builtin_macros::with_builtin! {
    let $spec = include_from_root!("src/holeybytes/instructions.in") in {
        /// Invoke with contents of specification
        macro_rules! invoke_with_def {
            ($macro:path) => {
                $macro! { $spec }
            };
        }
    }
}

/// Thingee for counting
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

/// Define newtype for operand type (or something else?)
macro_rules! operands {
    ($($name:ident $inner:tt),* $(,)?) => {
        $(
            #[derive(Clone, Debug, PartialEq, Eq)]
            #[repr(transparent)]
            pub struct $name $inner;
        )*
    };
}

/// Inner definition of operands
macro_rules! define_operands_inner {
    // Raw one, transmutable, memory repr = bytecode repr
    (* $name:ident ($($item:ident),* $(,)?)) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        #[repr(packed)]
        pub struct $name($(pub $item),*);

        impl $name {
            pub fn encode(self) -> [u8; std::mem::size_of::<Self>()] {
                #[allow(unsafe_code)]
                unsafe { std::mem::transmute(self) }
            }
        }
    };

    // Define as new item
    (+ $name:ident ($($item:ident),* $(,)?)) => {
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub struct $name($(pub $item),*);
    };

    // Re-export raw item
    (= $name:ident ($($item:ident),* $(,)?)) => {
        #[doc(inline)]
        pub use raw_ops::$name;
    };
}

use {define_operands_inner, operands};

pub use {crate::traits::Address, raw_ops::Register};
operands!(
    Relative16(pub Address),
    Relative32(pub Address),
);

/// Define operand types
/// - Also defines [`raw_ops`] module containing bytecode representation
///   of operands
/// 
/// # Sigils
/// - `+`: For defining type whose representation for arch-ops and
///   in bytecode is different
/// - `=`: For defining type whose representation is same in arch-ops
///   and bytecode, reexports from [`raw_ops`]
/// - `*`: For internal use only, refer to [`define_operands_inner`]
macro_rules! define_operands {
    ($($sigil:tt $name:ident ($($item:ident),* $(,)?)),* $(,)?) => {
        /// Operands with memory representation same as in the bytecode
        pub mod raw_ops {
            super::operands!(
                Register(pub u8),
                Address(pub u64),
                Relative16(pub u16),
                Relative32(pub u32),
            );

            impl Copy for Register   {}
            impl Copy for Address    {}
            impl Copy for Relative16 {}
            impl Copy for Relative32 {}

            $(super::define_operands_inner!(* $name ($($item),*));)*
        }

        $(define_operands_inner!($sigil $name ($($item),*));)*

        #[derive(Clone, Debug, PartialEq, Eq)]
        pub enum Operands {
            $($name($name)),*
        }
    };
}

define_operands! {
    = OpsRR   (Register  , Register                      ),
    = OpsRRR  (Register  , Register, Register            ),
    = OpsRRRR (Register  , Register, Register  , Register),
    = OpsRRB  (Register  , Register, u8                  ),
    = OpsRRH  (Register  , Register, u16                 ),
    = OpsRRW  (Register  , Register, u32                 ),
    = OpsRD   (Register  , u64                           ),
    = OpsRRD  (Register  , Register, u64                 ),
    + OpsRRA  (Register  , Register, Address             ),
    + OpsRRAH (Register  , Register, Address   , u16     ),
    + OpsRROH (Register  , Register, Relative32, u16     ),
    + OpsRRO  (Register  , Register, Relative32          ),
    + OpsRRP  (Register  , Register, Relative16          ),
    + OpsA    (Address                                   ),
    + OpsO    (Relative32                                ),
    = OpsN    (                                          ),
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

invoke_with_def!(verify_ops);

/// Validate if passed [`Operands`] is correct for passed [`Opcode`]
pub const fn validate_ops_for_opcode(opcode: Opcode, operands: &Operands) -> bool {
    macro_rules! generate {
        ($($_o:expr, $mnemonic:ident, $ty:ident, $_d:literal;)*) => {
            paste::paste! {
                match opcode {
                    $(Opcode::[<$mnemonic:camel>] =>
                        matches!(operands, Operands::[<Ops $ty>] { .. })),*
                }
            }
        };
    }

    invoke_with_def! { generate }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Instruction {
    opcode: Opcode,
    operands: Operands,
}

impl Instruction {
    pub fn new(opcode: Opcode, operands: Operands) -> Option<Self> {
        if validate_ops_for_opcode(opcode, &operands) {
            Some(Self::new_unchecked(opcode, operands))
        } else {
            None
        }
    }

    pub const fn new_unchecked(opcode: Opcode, operands: Operands) -> Self {
        Self { opcode, operands }
    }

    #[inline]
    pub fn opcode(&self) -> Opcode {
        self.opcode
    }

    #[inline]
    pub fn operands(&self) -> &Operands {
        &self.operands
    }

    #[inline]
    pub fn into_pair(self) -> (Opcode, Operands) {
        (self.opcode, self.operands)
    }
}
