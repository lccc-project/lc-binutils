use std::{
    any::Any,
    iter::Peekable,
    ops::{Deref, DerefMut},
};

use arch_ops::traits::InsnWrite;

use crate::{expr::Expression, lex::Token, span::Spanned, targ::TargetMachine};

pub trait PeekToken: Iterator {
    fn peek(&mut self) -> Option<&Self::Item>;

    fn next_ignore_newline(&mut self) -> Option<Spanned<Token>>
    where
        Self: Iterator<Item = Spanned<Token>>,
    {
        loop {
            match self.next()? {
                tok if matches!(tok.body(), Token::LineTerminator) => {
                    self.next();
                    continue;
                }
                tok => break Some(tok),
            }
        }
    }

    fn peek_ignore_newline(&mut self) -> Option<&Spanned<Token>>
    where
        Self: Iterator<Item = Spanned<Token>>,
    {
        loop {
            match self.peek()? {
                tok if matches!(tok.body(), Token::LineTerminator) => {
                    self.next();
                    continue;
                }
                tok => break Some(unsafe { &*(tok as *const _) }), // Hecking NLL
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
        tokens: &'a mut (dyn Iterator<Item = Spanned<Token>> + 'a),
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
            match maybe_mnemonic.into_inner() {
                Token::Identifier(id) => match self.state.iter.peek()?.body() {
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

        if mnemonic.starts_with('.') {
            if self.mach.directive_names().contains(&(&*mnemonic)) {
                Some(self.mach.handle_directive(&mnemonic, self))
            } else {
                match &*mnemonic {
                    ".asciz" => {
                        let mut buf = match self.state.iter.next_ignore_newline()?.body() {
                            Token::StringLiteral(x) => x.bytes().collect::<Vec<_>>(),
                            tok => panic!("Unexpected token {:?}. Expected a string literal", tok),
                        };
                        buf.push(0);
                        Some(self.state.output.write_all(&buf))
                    }
                    ".ascii" => {
                        let buf = match self.state.iter.next_ignore_newline()?.body() {
                            Token::StringLiteral(x) => x.bytes().collect::<Vec<_>>(),
                            tok => panic!("Unexpected token {:?}. Expected a string literal", tok),
                        };
                        Some(self.state.output.write_all(&buf))
                    }
                    ".long" => {
                        loop {
                            let expr = crate::expr::parse_expression(self.iter());
                            let expr = self.eval_expr(expr);

                            let len = self.state.mach.long_width();

                            match expr {
                                Expression::Symbol(sym) => {
                                    let output = self.output();

                                    match output.write_addr(
                                        len * 8,
                                        arch_ops::traits::Address::Symbol { name: sym, disp: 0 },
                                        false,
                                    ) {
                                        Ok(_) => {}
                                        Err(e) => return Some(Err(e)),
                                    }
                                }
                                Expression::Integer(val) => {
                                    let mut bytes = [0u8; 16];
                                    self.machine().int_to_bytes(val, &mut bytes[..len]);
                                    let output = self.output();
                                    match output.write_all(&bytes[..len]) {
                                        Ok(_) => {}
                                        Err(e) => return Some(Err(e)),
                                    }
                                }
                                expr => todo!("{:?}", expr),
                            }

                            match self.iter().peek().map(Spanned::body) {
                                Some(Token::Sigil(s)) if s == "," => {
                                    self.iter().next();
                                }
                                _ => break,
                            }
                        }
                        Some(Ok(()))
                    }
                    ".quad" => {
                        loop {
                            let expr = crate::expr::parse_expression(self.iter());
                            let expr = self.eval_expr(expr);

                            match expr {
                                Expression::Symbol(sym) => {
                                    let output = self.output();

                                    match output.write_addr(
                                        64,
                                        arch_ops::traits::Address::Symbol { name: sym, disp: 0 },
                                        false,
                                    ) {
                                        Ok(_) => {}
                                        Err(e) => return Some(Err(e)),
                                    }
                                }
                                Expression::Integer(val) => {
                                    let mut bytes = [0u8; 8];
                                    self.machine().int_to_bytes(val, &mut bytes);
                                    let output = self.output();
                                    match output.write_all(&bytes) {
                                        Ok(_) => {}
                                        Err(e) => return Some(Err(e)),
                                    }
                                }
                                expr => todo!("{:?}", expr),
                            }

                            match self.iter().peek().map(Spanned::body) {
                                Some(Token::Sigil(s)) if s == "," => {
                                    self.iter().next();
                                }
                                _ => break,
                            }
                        }
                        Some(Ok(()))
                    }
                    ".space" => {
                        let expr = crate::expr::parse_expression(self.iter());
                        let expr = self.eval_expr(expr);

                        match expr {
                            Expression::Integer(mut i) => {
                                let output = self.output();

                                Some(output.write_zeroes(i.try_into().unwrap()))
                            }
                            expr => panic!("Invalid expression for .space: {:?}", expr),
                        }
                    }
                    _ => Some(self.as_callbacks.handle_directive(self, &mnemonic)),
                }
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
    iter: Peekable<&'a mut (dyn Iterator<Item = Spanned<Token>> + 'a)>,
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

    pub fn iter(&mut self) -> &mut Peekable<&'a mut (dyn Iterator<Item = Spanned<Token>> + 'a)> {
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

pub fn float_to_bytes_le(val: f64, bytes: &mut [u8]) -> &mut [u8] {
    bytes.copy_from_slice(&val.to_le_bytes()[..bytes.len()]);
    bytes
}
