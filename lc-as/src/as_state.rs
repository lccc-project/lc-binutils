use std::{
    any::Any,
    iter::Peekable,
    ops::{Deref, DerefMut},
};

use arch_ops::traits::InsnWrite;

use crate::{expr::Expression, lex::Token, targ::TargetMachine};

pub trait PeekToken: Iterator {
    fn peek(&mut self) -> Option<&Self::Item>;

    fn next_ignore_newline(&mut self) -> Option<Token>
    where
        Self: Iterator<Item = Token>,
    {
        loop {
            match self.next()? {
                Token::LineTerminator => continue,
                tok => break Some(tok),
            }
        }
    }

    fn peek_ignore_newline(&mut self) -> Option<&Token>
    where
        Self: Iterator<Item = Token>,
    {
        loop {
            match self.peek()? {
                Token::LineTerminator => {
                    self.next();
                    continue;
                }
                tok => break Some(unsafe { &*(tok as *const Token) }), // Hecking NLL
            }
        }
    }
}

impl<I: Iterator> PeekToken for Peekable<I> {
    fn peek(&mut self) -> Option<&Self::Item> {
        self.peek()
    }
}

pub trait AssemblerCallbacks {
    fn handle_directive(&self, asm: &mut Assembler, dir: &str) -> std::io::Result<()>;
    fn create_symbol_now(&self, asm: &mut Assembler, sym: &str);
}

pub struct Assembler<'a> {
    state: AsState<'a>,
    as_data: Box<dyn Any>,
    as_callbacks: &'a dyn AssemblerCallbacks,
}

impl<'a> Assembler<'a> {
    pub fn new(
        mach: &'a dyn TargetMachine,
        output: Box<dyn InsnWrite + 'a>,
        as_data: Box<dyn Any>,
        as_callbacks: &'a dyn AssemblerCallbacks,
        tokens: &'a mut (dyn Iterator<Item = Token> + 'a),
    ) -> Assembler<'a> {
        Assembler {
            state: AsState {
                mach,
                output,
                mach_data: mach.create_data(),
                iter: tokens.peekable(),
            },
            as_data,
            as_callbacks,
        }
    }

    pub fn as_data(&self) -> &dyn Any {
        &*self.as_data
    }

    pub fn as_data_mut(&mut self) -> &mut dyn Any {
        &mut *self.as_data
    }

    pub fn set_output(&mut self, output: Box<dyn InsnWrite + 'a>) {
        self.state.output = output;
    }

    pub fn assemble_instr(&mut self) -> Option<std::io::Result<()>> {
        let mnemonic;

        loop {
            let maybe_mnemonic = self.state.iter.next_ignore_newline()?;
            match maybe_mnemonic {
                Token::Identifier(id) => match self.state.iter.peek()? {
                    Token::Sigil(x) if x == ":" => {
                        self.state.iter.next();
                        self.as_callbacks.create_symbol_now(self, &id)
                    }
                    _ => {
                        mnemonic = id;
                        break;
                    }
                },
                tok => panic!("Unexpected token {:?}. Expected a label or a mnemonic", tok),
            }
        }

        eprintln!("DEBUG: Assembling instruction: {}", mnemonic);

        if mnemonic.starts_with('.') {
            if self.mach.directive_names().contains(&(&*mnemonic)) {
                Some(self.mach.handle_directive(&mnemonic, self))
            } else {
                Some(self.as_callbacks.handle_directive(self, &mnemonic))
            }
        } else {
            Some(self.mach.assemble_insn(&mnemonic, self))
        }
    }
}

impl<'a> Deref for Assembler<'a> {
    type Target = AsState<'a>;
    fn deref(&self) -> &AsState<'a> {
        &self.state
    }
}

impl<'a> DerefMut for Assembler<'a> {
    fn deref_mut(&mut self) -> &mut AsState<'a> {
        &mut self.state
    }
}

pub struct AsState<'a> {
    mach: &'a dyn TargetMachine,
    output: Box<dyn InsnWrite + 'a>,
    mach_data: Box<dyn Any>,
    iter: Peekable<&'a mut (dyn Iterator<Item = Token> + 'a)>,
}

impl<'a> AsState<'a> {
    pub fn machine(&self) -> &dyn TargetMachine {
        self.mach
    }

    pub fn output(&mut self) -> &mut (dyn InsnWrite + 'a) {
        &mut *self.output
    }

    pub fn mach_data_mut(&mut self) -> &mut dyn Any {
        &mut *self.mach_data
    }

    pub fn mach_data(&self) -> &dyn Any {
        &*self.mach_data
    }

    pub fn iter(&mut self) -> &mut Peekable<&'a mut (dyn Iterator<Item = Token> + 'a)> {
        &mut self.iter
    }

    pub fn eval_expr(&mut self, expr: Expression) -> Expression {
        expr
    }
}

pub fn int_to_bytes_le(val: u128, bytes: &mut [u8]) -> &mut [u8] {
    let val = val.to_le_bytes();
    bytes.copy_from_slice(&val[..bytes.len()]);
    bytes
}

pub fn int_to_bytes_be(val: u128, bytes: &mut [u8]) -> &mut [u8] {
    let val = val.to_be_bytes();
    bytes.copy_from_slice(&val[..bytes.len()]);
    bytes
}
