use crate::{
    span::{Pos, Span, Spanned},
    sym::Symbol,
    targ::TargetMachine,
};
use core::iter::Peekable;

static GROUP_PAIRS: [[char; 2]; 4] = [['{', '}'], ['(', ')'], ['[', ']'], ['<', '>']];

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Token {
    LineTerminator,
    Error,
    Group(char, Vec<Spanned<Token>>),
    Identifier(String),
    Sigil(String),
    StringLiteral(String),
    IntegerLiteral(u128),
}

pub struct Lexer<'a, I: Iterator, A: ?Sized>(&'a mut Peekable<I>, &'a A, Option<char>, Pos, Symbol);

impl<'a, I: Iterator<Item = char>, A: ?Sized> Lexer<'a, I, A> {
    pub fn new(mach: &'a A, it: &'a mut Peekable<I>, file: impl Into<Symbol>) -> Self {
        Self(it, mach, None, Pos::new(0, 0), file.into())
    }
}

impl<I: Iterator<Item = char>, A: ?Sized + TargetMachine> Iterator for Lexer<'_, I, A> {
    type Item = Spanned<Token>;
    fn next(&mut self) -> Option<Spanned<Token>> {
        let mut comment = false;
        let file = self.4;
        let pos = self.3;
        let mut end_pos = pos;
        let c = loop {
            let c = self.0.next()?;
            match c {
                '\r' => match self.0.next() {
                    Some('\n') => {
                        self.3 = pos.next_row(1);
                        return Some(Spanned::new(
                            Token::LineTerminator,
                            Span::new_simple(pos, pos.next_col(2), file),
                        ));
                    }
                    _ => {
                        return Some(Spanned::new(
                            Token::Error,
                            Span::new_simple(pos, pos.next_col(1), file),
                        ))
                    }
                },
                '\n' => {
                    self.3 = pos.next_row(1);
                    return Some(Spanned::new(
                        Token::LineTerminator,
                        Span::new_simple(pos, pos.next_col(1), file),
                    ));
                }
                c if self.1.comment_chars().contains(&c) => {
                    end_pos = end_pos.next_col(1);
                    comment = true;
                }
                c if comment || c.is_whitespace() => {
                    end_pos = end_pos.next_col(1);
                }
                '/' => match self.0.peek() {
                    Some('/') => {
                        end_pos = end_pos.next_col(2);
                        comment = true;
                    }
                    _ => break '/',
                },
                c => break c,
            }
        };
        end_pos = end_pos.next_col(1);
        let tok = match c {
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
                            end_pos = end_pos.next_col(1);
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
                let mut lexer = Lexer(&mut self.0, self.1, Some(end), end_pos.next_col(1), file);
                let tokens = lexer.by_ref().collect();
                end_pos = lexer.3;
                Some(Token::Group(x, tokens))
            }
            ':' | ',' | ';' | '#' | '?' => {
                let sigil = String::from(c);
                Some(Token::Sigil(sigil))
            }
            '+' | '-' | '*' | '/' | '!' | '=' | '^' | '~' | '>' => {
                let mut sigil = String::from(c);

                match self.0.peek() {
                    Some('=') => {
                        end_pos = end_pos.next_col(1);
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
                        end_pos = end_pos.next_col(1);
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
                        end_pos = end_pos.next_col(1);
                        self.0.next();
                        sigil.push('=')
                    }
                    Some(x) if x == &c => {
                        end_pos = end_pos.next_col(1);
                        self.0.next();
                        sigil.push(c)
                    }
                    _ => {}
                }
                Some(Token::Sigil(sigil))
            }
            '0' => match self.0.peek() {
                Some('x') => {
                    end_pos = end_pos.next_col(1);
                    self.0.next();
                    let mut val = 0u128;
                    while let Some(c @ ('0'..='9' | 'A'..='F' | 'a'..='f')) = self.0.peek() {
                        end_pos = end_pos.next_col(1);
                        val <<= 4;
                        val |= c.to_digit(16).unwrap() as u128;
                        self.0.next();
                    }
                    Some(Token::IntegerLiteral(val))
                }
                Some('0'..='7') => {
                    end_pos = end_pos.next_col(1);
                    let mut val = 0u128;
                    while let Some(c @ ('0'..='7')) = self.0.peek() {
                        end_pos = end_pos.next_col(1);
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
                    end_pos = end_pos.next_col(1);
                    val *= 10;
                    val += c.to_digit(10).unwrap() as u128;
                    self.0.next();
                }
                Some(Token::IntegerLiteral(val))
            }
            '"' => {
                let mut str = String::from('"');

                loop {
                    match self.0.next() {
                        None => break Some(Token::Error),
                        Some('"') => {
                            end_pos = end_pos.next_col(1);
                            str.push('"');
                            break Some(Token::StringLiteral(str));
                        }
                        Some('\\') => {
                            let escape_pos = end_pos;
                            end_pos = end_pos.next_col(1);
                            str.push('\\');
                            match self.0.next() {
                                None => {
                                    return Some(Spanned::new(
                                        Token::Error,
                                        Span::new_simple(escape_pos, end_pos, file),
                                    ))
                                }
                                Some(c @ ('\\' | '\'' | 'b' | 'r' | 'n' | 'a' | 'e' | '"')) => {
                                    str.push(c)
                                }
                                Some('x') => {
                                    str.push(c);
                                    match self.0.next() {
                                        Some(c @ ('0'..='9' | 'A'..='F' | 'a'..='f')) => {
                                            str.push(c)
                                        }
                                        _ => {
                                            return Some(Spanned::new(
                                                Token::Error,
                                                Span::new_simple(escape_pos, end_pos, file),
                                            ))
                                        }
                                    }
                                    match self.0.next() {
                                        Some(c @ ('0'..='9' | 'A'..='F' | 'a'..='f')) => {
                                            str.push(c)
                                        }
                                        _ => {
                                            return Some(Spanned::new(
                                                Token::Error,
                                                Span::new_simple(escape_pos, end_pos, file),
                                            ))
                                        }
                                    }
                                }
                                Some('u') => match self.0.peek() {
                                    Some('{') => {
                                        end_pos = end_pos.next_col(2);
                                        self.0.next();
                                        str.push('{');
                                        loop {
                                            end_pos = end_pos.next_col(1);
                                            match self.0.next() {
                                                Some(c @ ('0'..='9' | 'A'..='F' | 'a'..='f')) => {
                                                    str.push(c)
                                                }
                                                Some('}') => {
                                                    str.push('}');
                                                    break;
                                                }
                                                _ => {
                                                    return Some(Spanned::new(
                                                        Token::Error,
                                                        Span::new_simple(escape_pos, end_pos, file),
                                                    ))
                                                }
                                            }
                                        }
                                    }
                                    Some('0'..='9' | 'A'..='F' | 'a'..='f') => {
                                        end_pos = end_pos.next_col(1);
                                        for i in 0..4 {
                                            end_pos = end_pos.next_col(1);
                                            match self.0.next() {
                                                Some(c @ ('0'..='9' | 'A'..='F' | 'a'..='f')) => {
                                                    str.push(c)
                                                }
                                                _ => {
                                                    return Some(Spanned::new(
                                                        Token::Error,
                                                        Span::new_simple(escape_pos, end_pos, file),
                                                    ))
                                                }
                                            }
                                        }
                                    }
                                    _ => {
                                        return Some(Spanned::new(
                                            Token::Error,
                                            Span::new_simple(escape_pos, end_pos, file),
                                        ))
                                    }
                                },
                                Some('U') => {
                                    for i in 0..8 {
                                        match self.0.next() {
                                            Some(c @ ('0'..='9' | 'A'..='F' | 'a'..='f')) => {
                                                str.push(c)
                                            }
                                            _ => {
                                                return Some(Spanned::new(
                                                    Token::Error,
                                                    Span::new_simple(escape_pos, end_pos, file),
                                                ))
                                            }
                                        }
                                    }
                                    end_pos = end_pos.next_col(9);
                                }
                                _ => {
                                    return Some(Spanned::new(
                                        Token::Error,
                                        Span::new_simple(escape_pos, end_pos, file),
                                    ))
                                }
                            }
                        }
                        Some(c) => {
                            end_pos = end_pos.next_col(1);
                            str.push(c)
                        }
                    }
                }
            }
            _ => Some(Token::Error),
        };
        self.3 = end_pos;
        let span = Span::new_simple(pos, end_pos, file);
        tok.map(|tok| Spanned::new(tok, span))
    }
}
