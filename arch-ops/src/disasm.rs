use crate::traits::InsnRead;


pub trait OpcodePrinter{
    fn print_opcode(&self, f: &mut core::fmt::Formatter, read: &mut dyn InsnRead) -> std::io::Result<()>;

    fn handle_option(&mut self, _key: &str, _value: &str) -> bool{
        false
    }
}