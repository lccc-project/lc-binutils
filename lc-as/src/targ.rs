use std::any::Any;

pub trait TargetMachine {
    fn group_chars(&self) -> &[char];
    fn comment_chars(&self) -> &[char];
    fn extra_sym_chars(&self) -> &[char];
    fn extra_sym_part_chars(&self) -> &[char];
    fn extra_sigil_chars(&self) -> &[char];

    fn create_data(&self) -> Box<dyn Any>;
}
