use super::TargetMachine;
use crate::as_state::int_to_bytes_le;
use arch_ops::w65::W65Mode;

pub struct W65TargetMachine;

impl TargetMachine for W65TargetMachine {
    fn group_chars(&self) -> &[char] {
        &['(', '[']
    }

    fn comment_chars(&self) -> &[char] {
        &[';']
    }

    fn extra_sym_chars(&self) -> &[char] {
        &['.', '$']
    }

    fn extra_sym_part_chars(&self) -> &[char] {
        &['.', '$']
    }

    fn extra_sigil_chars(&self) -> &[char] {
        &['#', '%']
    }

    fn create_data(&self) -> Box<dyn std::any::Any> {
        Box::new(W65Data {
            mode: W65Mode::NONE,
        })
    }

    fn int_to_bytes<'a>(&self, val: u128, buf: &'a mut [u8]) -> &'a mut [u8] {
        int_to_bytes_le(val, buf)
    }

    fn float_to_bytes<'a>(&self, val: f64, buf: &'a mut [u8]) -> &'a mut [u8] {
        todo!()
    }

    fn long_width(&self) -> usize {
        4
    }

    fn assemble_insn(
        &self,
        opc: &str,
        state: &mut crate::as_state::AsState,
    ) -> std::io::Result<()> {
        todo!()
    }

    fn directive_names(&self) -> &[&str] {
        &[
            ".acc8", ".acc16", ".idx8", ".idx16", ".m8", ".m16", ".x8", ".x16", ".mx8", ".mx16",
        ]
    }

    fn handle_directive(
        &self,
        dir: &str,
        state: &mut crate::as_state::AsState,
    ) -> std::io::Result<()> {
        match dir {
            ".acc8" | ".m8" => {
                state
                    .mach_data_mut()
                    .downcast_mut::<W65Data>()
                    .unwrap()
                    .mode |= W65Mode::M;
            }
            ".acc16" | ".m16" => {
                state
                    .mach_data_mut()
                    .downcast_mut::<W65Data>()
                    .unwrap()
                    .mode &= !W65Mode::M;
            }
            ".idx8" | ".x8" => {
                state
                    .mach_data_mut()
                    .downcast_mut::<W65Data>()
                    .unwrap()
                    .mode |= W65Mode::X;
            }
            ".idx16" | ".x16" => {
                state
                    .mach_data_mut()
                    .downcast_mut::<W65Data>()
                    .unwrap()
                    .mode &= !W65Mode::X;
            }
            ".mx8" => {
                state
                    .mach_data_mut()
                    .downcast_mut::<W65Data>()
                    .unwrap()
                    .mode |= W65Mode::M | W65Mode::X;
            }
            ".mx16" => {
                state
                    .mach_data_mut()
                    .downcast_mut::<W65Data>()
                    .unwrap()
                    .mode &= !(W65Mode::M | W65Mode::X);
            }
            _ => unreachable!(),
        }
        Ok(())
    }
}

pub struct W65Data {
    mode: W65Mode,
}

pub fn get_target_def() -> &'static W65TargetMachine {
    &W65TargetMachine
}

pub enum W65Expression {
    Immediate(u16),
}
