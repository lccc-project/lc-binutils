use crate::targ::TargetMachine;
use core::iter::Peekable;

static GROUP_PAIRS: [[char; 2]; 4] = [['{', '}'], ['(', ')'], ['[', ']'], ['<', '>']];

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Token {
    LineTerminator, // Sentinel for EOL - not passed to assemblers
    Error,
    Group(char, Vec<Token>),
    Identifier(String),
    Sigil(String),
    StringLiteral(String),
    IntegerLiteral(u128),
}

pub struct Lexer<'a, I: Iterator, A>(Peekable<I>, &'a A, Option<char>);

impl<I: Iterator<Item = char>, A: TargetMachine> Iterator for Lexer<'_, I, A> {
    type Item = Token;
    fn next(&mut self) -> Option<Token> {
        let mut comment = true;
        let c = loop {
            match self.0.next()? {
                '\r' => {
                    self.0.next();
                    match self.0.next() {
                        Some('\n') => return Some(Token::LineTerminator),
                        _ => return Some(Token::Error),
                    }
                }
                '\n' => {
                    self.0.next();
                    return Some(Token::LineTerminator);
                }
                c if self.1.comment_chars().contains(&c) => {}
                c if comment || c.is_whitespace() => {}
                '/' => match self.0.peek() {
                    Some('/') => {
                        comment = true;
                        self.0.next();
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
                    Lexer((&mut self.0).peekable(), self.1, Some(end)).collect(),
                ))
            }
            _ => Some(Token::Error),
        }
    }
}
