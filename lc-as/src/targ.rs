use std::any::Any;

use arch_ops::traits::InsnWrite;

use crate::as_state::AsState;

use target_tuples::Architecture;

pub trait TargetMachine {
    fn group_chars(&self) -> &[char];
    fn comment_chars(&self) -> &[char];
    fn extra_sym_chars(&self) -> &[char];
    fn extra_sym_part_chars(&self) -> &[char];
    fn extra_sigil_chars(&self) -> &[char];

    fn create_data(&self) -> Box<dyn Any>;

    fn int_to_bytes<'a>(&self, val: u128, buf: &'a mut [u8]) -> &'a mut [u8];
    fn float_to_bytes<'a>(&self, val: f64, buf: &'a mut [u8]) -> &'a mut [u8];

    fn assemble_insn(&self, opc: &str, state: &mut AsState) -> std::io::Result<()>;
    fn directive_names(&self) -> &[&str];
    fn handle_directive(&self, dir: &str, state: &mut AsState) -> std::io::Result<()>;

    fn def_section_alignment(&self) -> u64 {
        1024
    }

    /// Whether or not the target assembler cares about newlines in the token stream
    /// If set to false, LineTerminator tokens are stripped from the iterator.
    fn newline_sensitive(&self) -> bool {
        true
    }
}

macro_rules! targ_defs{
    {$(#[cfg($cfg:meta)] arch $arch:ident;)*} => {
        $(#[cfg($cfg)] mod $arch;)*

        pub fn get_target_def(arch: Architecture) -> Option<&'static dyn TargetMachine>{
            match arch.canonical_name(){
                $(stringify!($arch) => Some($arch :: get_target_def()),)*
                _ => None,
            }
        }
    }
}

targ_defs! {
    #[cfg(feature="clever")] arch clever;
}
