use crate::as_state::int_to_bytes_le;

use super::TargetMachine;

#[derive(Default, Clone, Hash, PartialEq, Eq)]
struct CleverData {}

pub struct CleverTargetMachine;

impl TargetMachine for CleverTargetMachine {
    fn group_chars(&self) -> &[char] {
        &['(', '[']
    }

    fn comment_chars(&self) -> &[char] {
        &[]
    }

    fn extra_sym_chars(&self) -> &[char] {
        &['_', '$', '.']
    }

    fn extra_sym_part_chars(&self) -> &[char] {
        &['_', '$', '.']
    }

    fn extra_sigil_chars(&self) -> &[char] {
        &[]
    }

    fn create_data(&self) -> Box<dyn std::any::Any> {
        Box::new(CleverData::default())
    }

    fn int_to_bytes<'a>(&self, val: u128, buf: &'a mut [u8]) -> &'a mut [u8] {
        int_to_bytes_le(val, buf)
    }

    fn float_to_bytes<'a>(&self, val: f64, buf: &'a mut [u8]) -> &'a mut [u8] {
        todo!("float_to_bytes")
    }

    fn assemble_insn(&self, state: &mut crate::as_state::AsState) -> std::io::Result<()> {
        todo!("assemble_insn")
    }

    fn directive_names(&self) -> &[&str] {
        &[]
    }

    fn handle_directive(
        &self,
        dir: &str,
        state: &mut crate::as_state::AsState,
    ) -> std::io::Result<()> {
        todo!()
    }
}

pub fn get_target_def() -> &'static CleverTargetMachine {
    &CleverTargetMachine
}

macro_rules! clever_mnemonics{
    {$([$mnemonic:literal, $opcode:literal $(, $parse_h:expr)? $(,)?]),* $(,)?} => {

    }
}
