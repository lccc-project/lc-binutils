use crate::targ::TargetMachine;
use core::iter::Peekable;

static GROUP_PAIRS: [[char; 2]; 4] = [['{', '}'], ['(', ')'], ['[', ']'], ['<', '>']];

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Token {
    LineTerminator,
    Error,
    Group(char, Vec<Token>),
    Identifier(String),
    Sigil(String),
    StringLiteral(String),
    IntegerLiteral(u128),
}

pub struct Lexer<'a, I: Iterator, A: ?Sized>(&'a mut Peekable<I>, &'a A, Option<char>);

impl<'a, I: Iterator<Item = char>, A: ?Sized> Lexer<'a, I, A> {
    pub fn new(mach: &'a A, it: &'a mut Peekable<I>) -> Self {
        Self(it, mach, None)
    }
}

impl<I: Iterator<Item = char>, A: ?Sized + TargetMachine> Iterator for Lexer<'_, I, A> {
    type Item = Token;
    fn next(&mut self) -> Option<Token> {
        let mut comment = false;
        let c = loop {
            let c = self.0.next()?;
            match c {
                '\r' => match self.0.next() {
                    Some('\n') if self.1.newline_sensitive() => return Some(Token::LineTerminator),
                    Some('\n') => {
                        comment = false;
                        continue;
                    }
                    _ => return Some(Token::Error),
                },
                '\n' if self.1.newline_sensitive() => {
                    return Some(Token::LineTerminator);
                }
                '\n' => {
                    comment = false;
                    continue;
                }
                c if self.1.comment_chars().contains(&c) => {
                    comment = true;
                }
                c if comment || c.is_whitespace() => {}
                '/' => match self.0.peek() {
                    Some('/') => {
                        comment = true;
                    }
                    _ => break '/',
                },
                c => break c,
            }
        };
        match c {
            x if Some(x) == self.2 => None,
            x if self.1.extra_sym_chars().contains(&x)
                || x.is_alphabetic()
                || x == '_'
                || x == '.' =>
            {
                let mut id = String::new();
                id.push(x);
                loop {
                    match self.0.peek() {
                        Some(x)
                            if self.1.extra_sym_part_chars().contains(x)
                                || self.1.extra_sym_chars().contains(&x)
                                || x.is_alphanumeric()
                                || *x == '_'
                                || *x == '.' =>
                        {
                            id.push(self.0.next().unwrap());
                        }
                        _ => break,
                    }
                }
                Some(Token::Identifier(id))
            }
            x if self.1.group_chars().contains(&x) => {
                let mut end = None;

                for [a, b] in GROUP_PAIRS {
                    if a == x {
                        end = Some(b);
                    }
                }

                let end = end.expect("Internal Error: Unexpected group char");

                Some(Token::Group(
                    x,
                    Lexer(&mut self.0, self.1, Some(end)).collect(),
                ))
            }
            ':' | ',' | ';' | '#' | '?' => {
                let sigil = String::from(c);
                Some(Token::Sigil(sigil))
            }
            '+' | '-' | '*' | '/' | '!' | '=' | '^' | '~' | '>' => {
                let mut sigil = String::from(c);

                match self.0.peek() {
                    Some('=') => {
                        self.0.next();
                        sigil.push('=')
                    }
                    _ => {}
                }

                Some(Token::Sigil(sigil))
            }
            '<' => {
                let mut sigil = String::from(c);
                match self.0.peek() {
                    Some(c @ ('=' | '>')) => {
                        let c = *c;
                        self.0.next();
                        sigil.push(c);
                    }
                    _ => {}
                }

                Some(Token::Sigil(sigil))
            }
            '&' | '|' => {
                let mut sigil = String::from(c);

                match self.0.peek() {
                    Some('=') => {
                        self.0.next();
                        sigil.push('=')
                    }
                    Some(x) if x == &c => {
                        self.0.next();
                        sigil.push(c)
                    }
                    _ => {}
                }
                Some(Token::Sigil(sigil))
            }
            '0' => match self.0.peek() {
                Some('x') => {
                    self.0.next();
                    let mut val = 0u128;
                    while let Some(c @ ('0'..='9' | 'A'..='F' | 'a'..='f')) = self.0.peek() {
                        val <<= 4;
                        val |= c.to_digit(16).unwrap() as u128;
                        self.0.next();
                    }
                    Some(Token::IntegerLiteral(val))
                }
                Some('0'..='7') => {
                    let mut val = 0u128;
                    while let Some(c @ ('0'..='7')) = self.0.peek() {
                        val <<= 3;
                        val |= c.to_digit(8).unwrap() as u128;
                        self.0.next();
                    }
                    Some(Token::IntegerLiteral(val))
                }
                _ => Some(Token::IntegerLiteral(0)),
            },
            '1'..='9' => {
                let mut val = c.to_digit(10).unwrap() as u128;
                while let Some(c @ ('0'..='9')) = self.0.peek() {
                    val *= 10;
                    val += c.to_digit(10).unwrap() as u128;
                    self.0.next();
                }
                Some(Token::IntegerLiteral(val))
            }
            _ => Some(Token::Error),
        }
    }
}
