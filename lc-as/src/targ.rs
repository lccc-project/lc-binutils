use std::any::Any;

use arch_ops::traits::InsnWrite;

use crate::as_state::AsState;

pub trait TargetMachine {
    fn group_chars(&self) -> &[char];
    fn comment_chars(&self) -> &[char];
    fn extra_sym_chars(&self) -> &[char];
    fn extra_sym_part_chars(&self) -> &[char];
    fn extra_sigil_chars(&self) -> &[char];

    fn create_data(&self) -> Box<dyn Any>;

    fn int_to_bytes<'a>(&self, val: u128, buf: &'a mut [u8]) -> &'a mut [u8];
    fn float_to_bytes<'a>(&self, val: f64, buf: &'a mut [u8]) -> &'a mut [u8];

    fn assemble_insn(&self, state: &mut AsState) -> std::io::Result<()>;
    fn directive_names(&self) -> &[&str];
    fn handle_directive(&self, dir: &str, state: &mut AsState) -> std::io::Result<()>;
}
