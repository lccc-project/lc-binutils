use std::{convert::TryFrom, str::FromStr};

use crate::{
    as_state::{int_to_bytes_le, PeekToken},
    expr::{BinaryOp, Expression},
    lex::Token,
};

use arch_ops::{
    clever::{
        CleverEncoder, CleverImmediate, CleverIndex, CleverInstruction, CleverOpcode,
        CleverOperand, CleverOperandKind, CleverRegister,
    },
    traits::Address,
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
        &[';']
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

    fn long_width(&self) -> usize{
        8
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
        match dir {
            _ => unreachable!(),
        }
    }

    fn newline_sensitive(&self) -> bool {
        false
    }

    fn def_section_alignment(&self) -> u64 {
        8
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

                let mut iter = group.into_iter().peekable();

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

                let expr = crate::expr::parse_expression(&mut iter);

                let expr = convert_expr(expr, isrel.is_some());

                match expr {
                    CleverExpr::Immediate(imm) if isrel == Some(true) => {
                        Some(CleverOperand::Immediate(CleverImmediate::LongMemRel(
                            inner_size.unwrap_or(64),
                            Address::Disp(imm as i64),
                            size.unwrap_or(64),
                        )))
                    }
                    CleverExpr::Immediate(imm) => {
                        Some(CleverOperand::Immediate(CleverImmediate::LongMem(
                            inner_size.unwrap_or(64),
                            Address::Abs(imm),
                            size.unwrap_or(64),
                        )))
                    }
                    CleverExpr::RelImm(disp) => {
                        Some(CleverOperand::Immediate(CleverImmediate::LongMemRel(
                            inner_size.unwrap_or(64),
                            Address::Disp(disp),
                            size.unwrap_or(64),
                        )))
                    }
                    CleverExpr::Symbol(sym, disp) if isrel == Some(true) => {
                        Some(CleverOperand::Immediate(CleverImmediate::LongMemRel(
                            inner_size.unwrap_or(64),
                            Address::Symbol { name: sym, disp },
                            size.unwrap_or(64),
                        )))
                    }
                    CleverExpr::Symbol(sym, disp) => {
                        Some(CleverOperand::Immediate(CleverImmediate::LongMem(
                            inner_size.unwrap_or(64),
                            Address::Symbol { name: sym, disp },
                            size.unwrap_or(64),
                        )))
                    }
                    CleverExpr::RelSym(sym, disp) => {
                        Some(CleverOperand::Immediate(CleverImmediate::LongMemRel(
                            inner_size.unwrap_or(64),
                            Address::Symbol { name: sym, disp },
                            size.unwrap_or(64),
                        )))
                    }
                    CleverExpr::Register(r) => Some(CleverOperand::Indirect {
                        size: size.unwrap_or(64),
                        base: r,
                        scale: 1,
                        index: arch_ops::clever::CleverIndex::Abs(0),
                    }),
                    CleverExpr::Indexed(reg, idx) => match *idx {
                        CleverExpr::Register(idx) => Some(CleverOperand::Indirect {
                            size: size.unwrap_or(64),
                            base: reg,
                            scale: 1,
                            index: CleverIndex::Register(idx),
                        }),
                        CleverExpr::Immediate(imm) => {
                            let scale = imm.trailing_zeros().min(7);
                            let imm = imm >> scale;

                            Some(CleverOperand::Indirect {
                                size: size.unwrap_or(64),
                                base: reg,
                                scale: (1<<scale) as u8,
                                index: CleverIndex::Abs(imm as i16),
                            })
                        }
                        CleverExpr::Scaled(idx, scale) => match *idx {
                            CleverExpr::Register(idx) => Some(CleverOperand::Indirect {
                                size: size.unwrap_or(64),
                                base: reg,
                                scale,
                                index: CleverIndex::Register(idx),
                            }),
                            CleverExpr::Immediate(imm) => Some(CleverOperand::Indirect {
                                size: size.unwrap_or(64),
                                base: reg,
                                scale,
                                index: CleverIndex::Abs(imm as i16),
                            }),
                            _ => None,
                        },
                        _ => None,
                    },
                    _ => None,
                }
            }
            _ => unreachable!(),
        },
        _ => {
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

            match expr {
                CleverExpr::Register(reg) => Some(CleverOperand::Register {
                    size: size.unwrap_or(64),
                    reg,
                }),
                CleverExpr::Immediate(val) => {
                    let bitsize = 128 - val.leading_zeros();

                    let size = size.unwrap_or_else(|| match bitsize {
                        0..=12 => 12,
                        13..=16 => 16,
                        17..=32 => 32,
                        _ => 64,
                    });

                    let imm_val = match (size, isrel) {
                        (12, Some(true)) => CleverImmediate::ShortRel(val as i16),
                        (12, _) => CleverImmediate::Short(val as u16),
                        (16 | 32 | 64, Some(true)) => CleverImmediate::LongRel(size, val as i64),
                        (16 | 32 | 64, _) => CleverImmediate::Long(size, val as u64),
                        (128, Some(true)) => None?,
                        (128, _) => CleverImmediate::Vec(val),
                        (val, _) => panic!("Impossible immediate size {}", val),
                    };

                    Some(CleverOperand::Immediate(imm_val))
                }
                CleverExpr::RelImm(val) => {
                    let bitsize = (64 - val.leading_zeros()) + (64 - val.leading_ones());

                    let size = size.unwrap_or_else(|| match bitsize {
                        0..=12 => 12,
                        13..=16 => 16,
                        17..=32 => 32,
                        _ => 64,
                    });

                    let imm_val = match size {
                        12 => CleverImmediate::ShortRel(val as i16),
                        16 | 32 | 64 => CleverImmediate::LongRel(size, val as i64),
                        128 => None?,
                        val => panic!("Impossible immediate size {}", val),
                    };

                    Some(CleverOperand::Immediate(imm_val))
                }
                CleverExpr::Symbol(sym, disp) => {
                    let addr = Address::Symbol { name: sym, disp };
                    let size = size.unwrap_or(64);

                    let imm_val = match (size, isrel) {
                        (12, Some(true)) => CleverImmediate::ShortAddrRel(addr),
                        (12, _) => CleverImmediate::ShortAddr(addr),
                        (16 | 32 | 64, Some(true)) => CleverImmediate::LongAddrRel(size, addr),
                        (16 | 32 | 64, _) => CleverImmediate::LongAddr(size, addr),
                        (128, _) => None?,
                        (val, _) => panic!("Impossible immediate size {}", val),
                    };
                    Some(CleverOperand::Immediate(imm_val))
                }
                CleverExpr::RelSym(sym, disp) => {
                    let addr = Address::Symbol { name: sym, disp };
                    let size = size.unwrap_or(64);

                    let imm_val = match size {
                        12 => CleverImmediate::ShortAddrRel(addr),
                        16 | 32 | 64 => CleverImmediate::LongAddrRel(size, addr),
                        128 => None?,
                        val => panic!("Impossible immediate size {}", val),
                    };

                    Some(CleverOperand::Immediate(imm_val))
                }
                expr => None?,
            }
        }
    }
}

fn parse_insn(
    prefix: Option<CleverOpcode>,
    opc: &str,
    state: &mut crate::as_state::AsState,
) -> Option<CleverInstruction> {
    if opc=="nop"{
        let mut oprs = Vec::new();
        if let Some(Token::LineTerminator) | None = state.iter().peek(){
        }else{
            for _ in 0..3{
                oprs.push(parse_operand(state, false)?);
                match state.iter().peek(){
                    Some(Token::Sigil(s)) if s=="," => continue,
                    _ => break,
                }
            }
        }
        let opc = 0x010 | (oprs.len() as u16);
        let opc = CleverOpcode::from_opcode(opc).unwrap();

        return Some(CleverInstruction::new(opc, oprs));
    }else if opc=="ifjmp"{
        let opc = 0x7c8f;
        return Some(CleverInstruction::new(CleverOpcode::from_opcode(opc).unwrap(), vec![]));
    }else if opc=="fret"{
        let opc = 0x7c8e;
        return Some(CleverInstruction::new(CleverOpcode::from_opcode(opc).unwrap(), vec![]));
    }
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
        | arch_ops::clever::CleverOperandKind::RelAddr => {
            let op = parse_operand(state, true)?;
            let size = op.size_ss()?;
            let is_rel = match op.immediate_value()? {
                CleverImmediate::Long(_, _) | CleverImmediate::LongAddr(_, _) => false,
                CleverImmediate::LongRel(_, _) | CleverImmediate::LongAddrRel(_, _) => true,
                _ => None?,
            };

            let opc = if opc.is_cbranch() {
                CleverOpcode::cbranch(
                    opc.branch_condition().unwrap(),
                    size,
                    is_rel,
                    opc.branch_weight().unwrap(),
                )
            } else {
                let opc = (opc.opcode() & 0xfcf0) | (size - 1) | (if is_rel { 0x100 } else { 0 });

                eprintln!("Computed branch opcode {:#X}", opc);
                CleverOpcode::from_opcode(opc).unwrap()
            };

            Some(CleverInstruction::new(opc, vec![op]))
        }
        arch_ops::clever::CleverOperandKind::Size => {
            let size = match state.iter().next()? {
                Token::Identifier(id) => match &*id {
                    "byte" => {
                        state.iter().next();
                        0
                    }
                    "half" => {
                        state.iter().next();
                        1
                    }
                    "single" => {
                        state.iter().next();
                        2
                    }
                    "double" => {
                        state.iter().next();
                        3
                    }
                    _ => panic!("Missing size specifier"),
                },
                _ => panic!("Missing size specifier"),
            };

            let opc = (opc.opcode() & 0xfff0) | size;
            eprintln!("Computed sized opcode {:#X}", opc);

            Some(CleverInstruction::new(
                CleverOpcode::from_opcode(opc).unwrap(),
                Vec::new(),
            ))
        }
        arch_ops::clever::CleverOperandKind::Insn => {
            if prefix.is_some(){
                return None
            }
             match state.iter().next()?{
                Token::Identifier(id) => {
                    parse_insn(Some(opc),&id,state)
                }
                tok => panic!("Unexpected token, excepted an instruction, got {:?}",tok)
            }
        },
        CleverOperandKind::HRegister => {
            let reg = match state.iter().next()? {
                Token::Identifier(id) => id
                    .parse::<CleverRegister>()
                    .expect("Expected a register name"),
                tok => panic!("Unexpected Token, expected a register name, got {:?}", tok),
            };

            if reg.0 > 15 {
                panic!("Expected a general purpose register, got {}", reg)
            }
            let opc = (opc.opcode() & 0xfff0) | (reg.0 as u16);

            Some(CleverInstruction::new(
                CleverOpcode::from_opcode(opc).unwrap(),
                Vec::new(),
            ))
        }
        CleverOperandKind::HImmediate => {
            let imm = match state.iter().next()? {
                Token::IntegerLiteral(lit) => lit,
                tok => panic!("Unexpected Token, expected an integer, got {:?}", tok),
            };

            if imm > 15 {
                panic!("Expected a value less than 16, got {}", imm)
            }
            let opc = (opc.opcode() & 0xfff0) | (imm as u16);

            Some(CleverInstruction::new(
                CleverOpcode::from_opcode(opc).unwrap(),
                Vec::new(),
            ))
        }
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
                    if let Some(next) =  x.strip_prefix($mnemonic){
                        let mut opc = ($opcode)<<4;
                        ($parse_h)(&mut opc,next)?;
                        return CleverOpcode::from_opcode(opc);
                    } $( else if let Some(next) =  x.strip_prefix($altopcs) {
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

fn parse_cc(opc: &mut u16, mnemonic: &str) -> Option<()>{
    match mnemonic {
        "p" | "po" => (),
        "c" | "b" => *opc |= 0x1,
        "v" => *opc |= 0x2,
        "z" | "e" | "eq" => *opc |= 0x3,
        "lt" => *opc |= 0x4,
        "le" => *opc |= 0x5,
        "be" => *opc |= 0x6,
        "mi" | "s" => *opc |= 0x7,
        "pl" | "ns" => *opc |= 0x8,
        "a" => *opc |= 0x9,
        "gt" => *opc |= 0xA,
        "ge" => *opc |= 0xB,
        "nz" | "ne" => *opc |= 0xC,
        "nv" => *opc |= 0xD,
        "nc" | "ae" => *opc |= 0xE,
        "np" | "pe" => *opc |= 0xF,
        _ => None?,
    }
    Some(())
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
    } else if mnemonic.is_empty() {
        Some(())
    } else {
        None
    }
}

fn parse_uf(opc: &mut u16, mnemonic: &str) -> Option<()> {
    if mnemonic.starts_with('.') {
        for c in mnemonic.chars().skip(1) {
            match c {
                'u' => *opc |= 0x2,
                'f' => *opc |= 0x1,
                _ => None?,
            }
        }

        Some(())
    } else if mnemonic.is_empty() {
        Some(())
    } else {
        None
    }
}

fn parse_size_suffix(opc: &mut u16, mnemonic: &str) -> Option<()>{
    if mnemonic.starts_with('.') {
        let suffix = &mnemonic[1..];
        match suffix{
            "8" | "byte" => (),
            "16" | "half" => *opc |= 0x01,
            "32" | "single" => *opc |= 0x02,
            "64" | "double" => *opc |= 0x03,
            _ => None?
        }
        Some(())
    }else{
        None
    }
}

fn parse_callsm(opc: &mut u16, mnemonic: &str) -> Option<()>{
    if mnemonic.starts_with('.') {
        let suffix = &mnemonic[1..];
        if suffix=="v"{
            *opc |= 0x01;
        }else{
            None?
        }
        Some(())
    }else if mnemonic.is_empty() {
        Some(())
    }
    else{
        None
    }
}

clever_mnemonics! {
    ["jmp", 0x7c0, parse_none],
    ["j",0x700,parse_jmp],
    ["und" | "und0",0x000,parse_none],
    ["add", 0x001, parse_l00f],
    ["sub", 0x002, parse_l00f],
    ["and", 0x003, parse_l00f],
    ["or" , 0x004, parse_l00f],
    ["xor", 0x005, parse_l00f],
    ["mov", 0x008, parse_none],
    ["lea", 0x009, parse_none],
    ["push", 0x014, parse_none],
    ["pop", 0x015, parse_none],
    ["stogpr",0x018,parse_none],
    ["stoar",0x019,parse_none],
    ["rstogpr",0x01a,parse_none],
    ["rstoar",0x01b,parse_none],
    ["pushgpr",0x01c,parse_none],
    ["pushar",0x01d,parse_none],
    ["popgpr",0x01e,parse_none],
    ["popar",0x01f,parse_none],
    ["movsx",0x020,parse_l00f],
    ["bswap",0x021,parse_l00f],
    ["movif",0x022,parse_uf],
    ["movfi",0x024,parse_uf],
    ["cvtf",0x026,parse_l00f],
    ["repc",0x028,parse_none],
    ["repi",0x029,parse_cc],
    ["bcpy",0x02a,parse_none],
    ["bsto",0x02b,parse_none],
    ["bsca",0x02c,parse_none],
    ["bcmp",0x02d,parse_none],
    ["btst",0x02e,parse_none],
    ["lsh", 0x030, parse_l00f],
    ["rsh",0x031, parse_l00f],
    ["arsh",0x032,parse_l00f],
    ["lshc",0x033,parse_l00f],
    ["rshc",0x034,parse_l00f],
    ["lrot",0x035,parse_l00f],
    ["rrot",0x036,parse_l00f],
    ["bnot",0x046,parse_l00f],
    ["neg",0x047,parse_l00f],

    ["cmovt",0x068,parse_cc],
    ["cmov",0x069,parse_cc],
    ["cmp", 0x06c, parse_none],
    ["test", 0x06d, parse_none],

    ["round",0x100,parse_l00f],
    ["ceil",0x101,parse_l00f],
    ["floor",0x102,parse_l00f],
    ["fabs",0x103,parse_l00f],
    ["fneg",0x104,parse_l00f],
    ["finv",0x105,parse_l00f],
    ["fadd",0x106,parse_l00f],
    ["fsub",0x107,parse_l00f],
    ["fmul",0x108,parse_l00f],
    ["fdiv",0x109,parse_l00f],
    ["frem",0x10a,parse_l00f],
    ["fma",0x10b,parse_l00f],
    ["fcmpz",0x118,parse_none],
    ["fcmp",0x119,parse_none],
    ["exp",0x120,parse_l00f],
    ["ln",0x121,parse_l00f],
    
    ["fraiseexcept",0x130,parse_none],
    ["ftriggerexcept",0x131,parse_none],

    ["xchg",0x200,parse_none],
    ["cmpxchg",0x201,parse_none],
    ["wcmpxchg",0x202,parse_none],
    ["fence",0x203,parse_none],

    ["rpoll",0x230,parse_none],

    ["vec",0x400,parse_size_suffix],
    ["vmov",0x401,parse_none],
    ["vshuffle",0x402,parse_size_suffix],
    ["vextract",0x403,parse_none],
    ["vcmp",0x404,parse_none],
    ["vtest",0x405,parse_none],
    ["vfcmp",0x406,parse_none],

    ["call",0x7c1, parse_none],
    ["fcall",0x7c2,parse_none],
    ["ret",0x7c3, parse_none],
    ["int",0x7c4, parse_none],
    ["ijmp",0x7c8,parse_none],
    ["icall",0x7c9,parse_none],
    ["ifcall",0x7ca,parse_none],
    ["jmpsm",0x7cb,parse_none],
    ["callsm",0x7cc,parse_callsm],
    ["retrsm",0x7cd,parse_none],
    
    ["hlt", 0x801, parse_none],
    ["in", 0x806, parse_none],
    ["out", 0x807, parse_none],
    ["storegf",0x808,parse_none],
    ["rstregf",0x809,parse_none],
    ["vmcreate",0xe00,parse_none],
    ["vmdestroy",0xe01,parse_none],
    
    ["scret",0xfc6,parse_none],
    ["reti",0xfc7,parse_none],
    ["hcall",0xfcb,parse_none],
    ["hret",0xfd6,parse_none],
    ["hresume",0xfd7,parse_none],
}
