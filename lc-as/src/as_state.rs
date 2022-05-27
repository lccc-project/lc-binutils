use std::{any::Any, iter::Peekable};

use arch_ops::traits::InsnWrite;

use crate::{lex::Token, targ::TargetMachine};

pub struct AsState<'a> {
    mach: &'a dyn TargetMachine,
    output: &'a mut (dyn InsnWrite + 'a),
    mach_data: &'a mut dyn Any,
    iter: Peekable<&'a mut (dyn Iterator<Item = Token> + 'a)>,
}

impl<'a> AsState<'a> {
    pub fn machine(&self) -> &dyn TargetMachine {
        self.mach
    }

    pub fn output(&mut self) -> &mut (dyn InsnWrite + 'a) {
        self.output
    }

    pub fn mach_data_mut(&mut self) -> &mut dyn Any {
        self.mach_data
    }

    pub fn mach_data(&self) -> &dyn Any {
        self.mach_data
    }

    pub fn iter(&mut self) -> &mut Peekable<&'a mut (dyn Iterator<Item = Token> + 'a)> {
        &mut self.iter
    }
}
