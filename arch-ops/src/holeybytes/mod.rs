with_builtin_macros::with_builtin! {
    let $spec = include_from_root!("src/holeybytes/instructions.in") in {
        macro_rules! invoke_with_def {
            ($macro:path) => {
                $macro! { $spec }
            };
        }
    }
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

        pub enum Operands {
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
