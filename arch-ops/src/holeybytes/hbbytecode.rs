//! Vendored and modified `hbbytecode`

pub type OpR = u8;

pub type OpA = u64;
pub type OpO = u32;
pub type OpP = u16;

pub type OpB = u8;
pub type OpH = u16;
pub type OpW = u32;
pub type OpD = u64;

macro_rules! define_items {
    ($($name:ident ($($item:ident),* $(,)?)),* $(,)?) => {
        $(
            #[repr(packed)]
            pub struct $name($(pub $item),*);
        )*
    };
}

define_items! {
    OpsRR   (OpR, OpR          ),
    OpsRRR  (OpR, OpR, OpR     ),
    OpsRRRR (OpR, OpR, OpR, OpR),
    OpsRRB  (OpR, OpR, OpB     ),
    OpsRRH  (OpR, OpR, OpH     ),
    OpsRRW  (OpR, OpR, OpW     ),
    OpsRD   (OpR, OpD          ),
    OpsRRD  (OpR, OpR, OpD     ),
    OpsRRAH (OpR, OpR, OpA, OpH),
    OpsRROH (OpR, OpR, OpO, OpH),
    OpsRRO  (OpR, OpR, OpO     ),
    OpsRRP  (OpR, OpR, OpP     ),
}


::with_builtin_macros::with_builtin! {
    let $spec = include_from_root!("src/holeybytes/instructions.in") in {
        /// Invoke macro with bytecode definition
        ///
        /// # Format
        /// ```text
        /// Opcode, Mnemonic, Type, Docstring;
        /// ```
        ///
        /// # Type
        /// ```text
        /// Types consist of letters meaning a single field
        /// | Type | Size (B) | Meaning                 |
        /// |:-----|:---------|:------------------------|
        /// | N    | 0        | Empty                   |
        /// | R    | 1        | Register                |
        /// | A    | 8        | Absolute address        |
        /// | O    | 4        | Relative address offset |
        /// | P    | 2        | Relative address offset |
        /// | B    | 1        | Immediate               |
        /// | H    | 2        | Immediate               |
        /// | W    | 4        | Immediate               |
        /// | D    | 8        | Immediate               |
        /// ```
        macro_rules! invoke_with_def {
            ($macro:path) => {
                $macro! { $spec }
            };
        }
    }
}

macro_rules! gen_opcodes {
    ($($opcode:expr, $mnemonic:ident, $_ty:ident, $doc:literal;)*) => {
        pub mod opcode {
            $(
                #[doc = $doc]
                pub const $mnemonic: u8 = $opcode;
            )*
        }
    };
}

pub(super) use {invoke_with_def, gen_opcodes};
