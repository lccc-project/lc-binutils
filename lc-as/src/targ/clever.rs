use std::{convert, str::FromStr};

use crate::{
    as_state::int_to_bytes_le,
    expr::{BinaryOp, Expression},
    lex::Token,
};

use arch_ops::clever::{
    CleverEncoder, CleverInstruction, CleverOpcode, CleverOperand, CleverOperandKind,
    CleverRegister,
};

use super::TargetMachine;

#[derive(Default, Clone, Hash, PartialEq, Eq)]
struct CleverData {}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub enum CleverExpr {
    Register(CleverRegister),
    Immediate(u128),
    RelImm(i64),
    Symbol(String, i64),
    RelSym(String, i64),
    Scaled(Box<CleverExpr>, u8),
    Indexed(CleverRegister, Box<CleverExpr>),
}

pub fn convert_expr(ex: Expression, is_addr: bool) -> CleverExpr {
    match ex {
        Expression::Symbol(s) => {
            if !is_addr {
                if let Ok(reg) = CleverRegister::from_str(&s) {
                    CleverExpr::Register(reg)
                } else {
                    CleverExpr::Symbol(s, 0)
                }
            } else {
                CleverExpr::Symbol(s, 0)
            }
        }
        Expression::Integer(val) => CleverExpr::Immediate(val),
        Expression::Binary(BinaryOp::Add, left, right) => {
            let left = convert_expr(*left, is_addr);
            let right = convert_expr(*right, is_addr);

            match (left, right) {
                (CleverExpr::Immediate(a), CleverExpr::Immediate(b)) => {
                    CleverExpr::Immediate(a.wrapping_add(b))
                }
                (CleverExpr::Symbol(a, disp), CleverExpr::Immediate(b)) => {
                    CleverExpr::Symbol(a, disp.wrapping_add(b as i64))
                }
                (CleverExpr::Symbol(a, disp), CleverExpr::Register(CleverRegister::ip)) => {
                    CleverExpr::RelSym(a, disp)
                }
                (CleverExpr::Immediate(a), CleverExpr::Register(CleverRegister::ip)) => {
                    CleverExpr::RelImm(a as i64)
                }
                (CleverExpr::Register(CleverRegister::ip), CleverExpr::Symbol(a, disp)) => {
                    CleverExpr::RelSym(a, disp)
                }
                (CleverExpr::Register(CleverRegister::ip), CleverExpr::Immediate(a)) => {
                    CleverExpr::RelImm(a as i64)
                }
                (
                    CleverExpr::Register(reg @ CleverRegister(0..=15)),
                    idx @ (CleverExpr::Scaled(_, _)
                    | CleverExpr::Immediate(_)
                    | CleverExpr::Register(CleverRegister(0..=15))),
                ) => CleverExpr::Indexed(reg, Box::new(idx)),
                (a, b) => todo!("Unsupported operand pair for addition {:?} and {:?}", a, b),
            }
        }
        Expression::Binary(BinaryOp::Mul, a, b) => {
            let left = convert_expr(*a, is_addr);
            let right = convert_expr(*b, is_addr);

            match (left, right) {
                (CleverExpr::Immediate(a), CleverExpr::Immediate(b)) => {
                    CleverExpr::Immediate(a.wrapping_mul(b))
                }
                (
                    CleverExpr::Immediate(a @ 0..=128),
                    reg @ CleverExpr::Register(CleverRegister(0..=15)),
                ) => CleverExpr::Scaled(Box::new(reg), a as u8),
                (
                    reg @ CleverExpr::Register(CleverRegister(0..=15)),
                    CleverExpr::Immediate(a @ 0..=128),
                ) => CleverExpr::Scaled(Box::new(reg), a as u8),
                (a, b) => todo!(
                    "Unsupported operand pair for multiplication {:?} and {:?}",
                    a,
                    b
                ),
            }
        }
        Expression::Binary(op, _, _) => todo!("binary op {:?}", op),
        Expression::Unary(_, _) => todo!("unary expr"),
        Expression::Group(_, _) => todo!("group"),
    }
}

pub struct CleverTargetMachine;

impl TargetMachine for CleverTargetMachine {
    fn group_chars(&self) -> &[char] {
        &['(', '[']
    }

    fn comment_chars(&self) -> &[char] {
        &[]
    }

    fn extra_sym_chars(&self) -> &[char] {
        &['_', '$', '.']
    }

    fn extra_sym_part_chars(&self) -> &[char] {
        &['_', '$', '.']
    }

    fn extra_sigil_chars(&self) -> &[char] {
        &[]
    }

    fn create_data(&self) -> Box<dyn std::any::Any> {
        Box::new(CleverData::default())
    }

    fn int_to_bytes<'a>(&self, val: u128, buf: &'a mut [u8]) -> &'a mut [u8] {
        int_to_bytes_le(val, buf)
    }

    fn float_to_bytes<'a>(&self, val: f64, buf: &'a mut [u8]) -> &'a mut [u8] {
        todo!("float_to_bytes")
    }

    fn assemble_insn(
        &self,
        opc: &str,
        state: &mut crate::as_state::AsState,
    ) -> std::io::Result<()> {
        let insn = parse_insn(None, opc, state).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Could not parse instruction",
            )
        })?;

        let mut enc = CleverEncoder::new(state.output());

        enc.write_instruction(insn)
    }

    fn directive_names(&self) -> &[&str] {
        &[]
    }

    fn handle_directive(
        &self,
        dir: &str,
        state: &mut crate::as_state::AsState,
    ) -> std::io::Result<()> {
        todo!()
    }
}

fn parse_operand(state: &mut crate::as_state::AsState, isaddr: bool) -> Option<CleverOperand> {
    let data = state
        .mach_data()
        .downcast_ref::<CleverData>()
        .unwrap()
        .clone();

    let mut size = None::<u16>;

    let iter = state.iter();

    match iter.peek()? {
        Token::Identifier(id) => match &**id {
            "byte" => {
                iter.next();
                size = Some(8)
            }
            "short" => {
                iter.next();
                size = Some(12)
            }
            "half" => {
                iter.next();
                size = Some(16)
            }
            "single" => {
                iter.next();
                size = Some(32)
            }
            "double" => {
                iter.next();
                size = Some(64)
            }
            "vector" | "quad" => {
                iter.next();
                size = Some(128)
            }
            _ => {}
        },
        _ => {}
    }

    let mut isrel = None::<bool>;

    match iter.peek()? {
        Token::Group('[', _) => match iter.next().unwrap() {
            Token::Group('[', group) => {
                let mut inner_size = None::<u16>;

                match iter.peek()? {
                    Token::Identifier(id) => match &**id {
                        "byte" => {
                            iter.next();
                            inner_size = Some(8)
                        }
                        "half" => {
                            iter.next();
                            inner_size = Some(16)
                        }
                        "single" => {
                            iter.next();
                            inner_size = Some(32)
                        }
                        "double" => {
                            iter.next();
                            inner_size = Some(64)
                        }
                        _ => {}
                    },
                    _ => {}
                }

                match iter.peek()? {
                    Token::Identifier(id) => match &**id {
                        "abs" => {
                            iter.next();
                            isrel = Some(false);
                        }
                        "rel" => {
                            iter.next();
                            isrel = Some(true);
                        }
                        _ => {}
                    },
                    _ => {}
                }

                let expr = crate::expr::parse_expression(iter);

                let expr = convert_expr(expr, isrel.is_some());

                todo!("memory op")
            }
            _ => unreachable!(),
        },
        _ => todo!("direct op"),
    }
}

fn parse_insn(
    prefix: Option<CleverOpcode>,
    opc: &str,
    state: &mut crate::as_state::AsState,
) -> Option<CleverInstruction> {
    let opc = parse_mnemonic(opc)?;

    match opc.operands() {
        arch_ops::clever::CleverOperandKind::Normal(n) => {
            let operands = (0..n)
                .map(|n| {
                    if n != 0 {
                        match state.iter().next()? {
                            Token::Sigil(s) if s == "," => {}
                            _ => None?,
                        }
                    }
                    parse_operand(state, false)
                })
                .collect::<Option<Vec<_>>>()?;

            Some(CleverInstruction::new(opc, operands))
        }
        arch_ops::clever::CleverOperandKind::AbsAddr
        | arch_ops::clever::CleverOperandKind::RelAddr => todo!(),
        arch_ops::clever::CleverOperandKind::Size => todo!(),
        arch_ops::clever::CleverOperandKind::Insn => todo!(),
    }
}

pub fn get_target_def() -> &'static CleverTargetMachine {
    &CleverTargetMachine
}

macro_rules! clever_mnemonics{
    {$([$mnemonic:literal $(| $altopcs:literal)*, $opcode:literal, $parse_h:expr $(,)?]),* $(,)?} => {
        fn parse_mnemonic(x: &str) -> Option<CleverOpcode>{
            $(
                {
                    if x.starts_with($mnemonic){
                        let next = &x[($mnemonic.len())..];
                        let mut opc = ($opcode)<<4;
                        ($parse_h)(&mut opc,next)?;
                        return CleverOpcode::from_opcode(opc);
                    } $( else if x.starts_with($altopcs) {
                        let next = &x[($altopcs.len())..];
                        let mut opc = ($opcode)<<4;
                        ($parse_h)(&mut opc,next)?;
                        return CleverOpcode::from_opcode(opc);
                    }
                    )*
                }
            )*
            None
        }
    }
}

fn parse_jmp(opc: &mut u16, mnemonic: &str) -> Option<()> {
    let pos = mnemonic.find(".");

    let prefix = pos.map(|v| &mnemonic[..v]).unwrap_or(mnemonic);
    let suffix = pos.map(|v| &mnemonic[(v + 1)..]);
    match prefix {
        "p" | "po" => (),
        "c" | "b" => *opc |= 0x10,
        "v" => *opc |= 0x20,
        "z" | "e" | "eq" => *opc |= 0x30,
        "lt" => *opc |= 0x40,
        "le" => *opc |= 0x50,
        "be" => *opc |= 0x60,
        "mi" | "s" => *opc |= 0x70,
        "pl" | "ns" => *opc |= 0x80,
        "a" => *opc |= 0x90,
        "gt" => *opc |= 0xA0,
        "ge" => *opc |= 0xB0,
        "nz" | "ne" => *opc |= 0xC0,
        "nv" => *opc |= 0xD0,
        "nc" | "ae" => *opc |= 0xE0,
        "np" | "pe" => *opc |= 0xF0,
        _ => None?,
    }

    if let Some(suffix) = suffix {
        if suffix == "l" || suffix == "likely" {
            *opc |= 0x7
        } else if suffix == "u" || suffix == "unlikely" {
            *opc |= 0x8
        } else {
            let val = suffix.parse::<i8>().ok()?;
            if !(-8..8).contains(&val) {
                return None;
            } else {
                *opc |= (val & 0xf) as u16;
            }
        }
    }

    Some(())
}

fn parse_none(_: &mut u16, _: &str) -> Option<()> {
    Some(())
}

fn parse_l00f(opc: &mut u16, mnemonic: &str) -> Option<()> {
    if mnemonic.starts_with('.') {
        for c in mnemonic.chars().skip(1) {
            match c {
                'l' => *opc |= 0x8,
                'f' => *opc |= 0x1,
                _ => None?,
            }
        }

        Some(())
    } else {
        None
    }
}

clever_mnemonics! {
    ["j",0x700,parse_jmp],
    ["und" | "und0",0x000,parse_none],
    ["add", 0x001, parse_l00f]
}
