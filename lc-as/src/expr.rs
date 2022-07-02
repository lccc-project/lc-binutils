use std::iter::Peekable;

use crate::lex::Token;

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Lsh,
    Rsh,
    And,
    Or,
    Xor,
    CmpEq,
    CmpNe,
    CmpLt,
    CmpGt,
    CmpLe,
    CmpGe,
    BoolAnd,
    BoolOr,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum UnaryOp {
    Umn,
    Neg,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Expression {
    Symbol(String),
    Integer(u128),
    Binary(BinaryOp, Box<Expression>, Box<Expression>),
    Unary(UnaryOp, Box<Expression>),
    Group(char, Box<Expression>),
}

pub fn parse_expression<I: Iterator<Item = Token>>(it: &mut Peekable<I>) -> Expression {
    parse_binary_expr(it, 0)
}

fn binary_op(st: &str) -> Option<(BinaryOp, u32)> {
    match st {
        "&&" => Some((BinaryOp::BoolAnd, 1)),
        "||" => Some((BinaryOp::BoolOr, 1)),
        "+" => Some((BinaryOp::Add, 3)),
        "-" => Some((BinaryOp::Sub, 3)),
        "==" => Some((BinaryOp::CmpEq, 3)),
        "<>" | "!=" => Some((BinaryOp::CmpNe, 3)),
        "<" => Some((BinaryOp::CmpLt, 3)),
        "<=" => Some((BinaryOp::CmpLe, 3)),
        ">" => Some((BinaryOp::CmpGt, 3)),
        ">=" => Some((BinaryOp::CmpGe, 3)),
        "^" => Some((BinaryOp::Xor, 5)),
        "&" => Some((BinaryOp::And, 5)),
        "|" => Some((BinaryOp::Or, 5)),
        "<<" => Some((BinaryOp::Lsh, 7)),
        ">>" => Some((BinaryOp::Rsh, 7)),
        "%" => Some((BinaryOp::Mod, 7)),
        "/" => Some((BinaryOp::Div, 7)),
        "*" => Some((BinaryOp::Mul, 7)),
        _ => None,
    }
}

pub fn parse_binary_expr<I: Iterator<Item = Token>>(
    it: &mut Peekable<I>,
    precedence: u32,
) -> Expression {
    let mut lhs = parse_unary_expr(it);
    loop {
        let tok = if let Some(Token::Sigil(sig)) = it.peek() {
            sig
        } else {
            break;
        };

        if let Some((op, lbp)) = binary_op(tok) {
            if lbp < precedence {
                break;
            }
            let rhs = parse_binary_expr(it, lbp + 1);
            lhs = Expression::Binary(op, Box::new(lhs), Box::new(rhs));
        } else {
            break;
        }
    }
    lhs
}
pub fn parse_unary_expr<I: Iterator<Item = Token>>(it: &mut Peekable<I>) -> Expression {
    match it.peek() {
        Some(Token::Sigil(x)) if x == "-" => {
            it.next();
            Expression::Unary(UnaryOp::Neg, Box::new(parse_unary_expr(it)))
        }
        Some(Token::Sigil(x)) if x == "!" => {
            it.next();
            Expression::Unary(UnaryOp::Umn, Box::new(parse_unary_expr(it)))
        }
        _ => parse_simple_expr(it),
    }
}

pub fn parse_simple_expr<I: Iterator<Item = Token>>(it: &mut Peekable<I>) -> Expression {
    match it.next().unwrap() {
        Token::Identifier(id) => Expression::Symbol(id),
        Token::IntegerLiteral(n) => Expression::Integer(n),
        Token::Group(c, inner) => {
            let mut it = inner.into_iter().peekable();

            let expr = parse_expression(&mut it);

            if let Some(tok) = it.next() {
                panic!("Unexpected token {:?}", tok)
            }

            Expression::Group(c, Box::new(expr))
        }
        tok => panic!("Unexpected token {:?}", tok),
    }
}
