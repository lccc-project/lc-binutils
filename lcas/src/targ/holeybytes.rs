use {
    super::TargetMachine,
    crate::expr::{parse_simple_expr, BinaryOp},
    crate::{
        as_state::{float_to_bytes_le, int_to_bytes_le, AsState},
        expr::Expression,
        lex::Token,
    },
    arch_ops::holeybytes::{
        self, Address, Instruction, Opcode, Operands, OpsType, Register, Relative16, Relative32,
    },
    std::{fmt::Display, iter::Peekable, str::FromStr},
};

#[derive(Default, Clone, Hash, PartialEq, Eq)]
struct Data {}

pub struct HbTargetMachine;
impl TargetMachine for HbTargetMachine {
    #[inline]
    fn group_chars(&self) -> &[char] {
        &['(', '[']
    }

    #[inline]
    fn comment_chars(&self) -> &[char] {
        &[';']
    }

    #[inline]
    fn extra_sym_chars(&self) -> &[char] {
        &['_', '$', '.']
    }

    #[inline]
    fn extra_sym_part_chars(&self) -> &[char] {
        &['_', '$', '.']
    }

    #[inline]
    fn extra_sigil_chars(&self) -> &[char] {
        &[]
    }

    #[inline]
    fn create_data(&self) -> Box<dyn std::any::Any> {
        Box::<Data>::default()
    }

    #[inline]
    fn int_to_bytes<'a>(&self, val: u128, buf: &'a mut [u8]) -> &'a mut [u8] {
        int_to_bytes_le(val, buf)
    }

    #[inline]
    fn float_to_bytes<'a>(&self, val: f64, buf: &'a mut [u8]) -> &'a mut [u8] {
        float_to_bytes_le(val, buf)
    }

    #[inline]
    fn long_width(&self) -> usize {
        core::mem::size_of::<u64>()
    }

    #[inline]
    fn assemble_insn(&self, opc: &str, state: &mut AsState) -> std::io::Result<()> {
        let opcode = Opcode::from_str(opc)
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid opcode"))?;

        let ops = extract_ops(opcode.ops_type(), state.iter())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;

        holeybytes::codec::HbEncoder::new(state.output())
            .write_instruction(Instruction::new_unchecked(opcode, ops))
    }

    #[inline]
    fn directive_names(&self) -> &[&str] {
        &[]
    }

    #[inline]
    fn handle_directive(&self, _dir: &str, _state: &mut AsState) -> std::io::Result<()> {
        unreachable!("There ain't no directives yet.")
    }

    #[inline]
    fn def_section_alignment(&self) -> u64 {
        8 // ?
    }

    #[inline]
    fn newline_sensitive(&self) -> bool {
        false
    }
}

#[inline]
pub fn get_target_def() -> &'static HbTargetMachine {
    &HbTargetMachine
}

pub fn extract_ops(
    opsty: OpsType,
    iter: &mut Peekable<impl Iterator<Item = Token>>,
) -> Result<Operands> {
    mod addressing {
        use super::*;
        type RegAddr = (Register, Address);

        macro_rules! generate {
            { $($a_name:ident, $b_name:ident => $a_mapper:expr),* $(,)? } => {
                $(
                    #[allow(non_snake_case)]
                    #[inline]
                    pub fn $a_name(p0: Register, r1: RegAddr, r2: u16) -> holeybytes::$a_name {
                        holeybytes::$a_name(p0, r1.0, $a_mapper(r1.1), r2)
                    }

                    #[allow(non_snake_case)]
                    #[inline]
                    pub fn $b_name(p0: Register, r1: RegAddr) -> holeybytes::$b_name {
                        holeybytes::$b_name(p0, r1.0, $a_mapper(r1.1))
                    }
                )*
            };
        }

        generate! {
            OpsRRAH, OpsRRA => std::convert::identity,
            OpsRROH, OpsRRO => holeybytes::Relative32,
            OpsRRPH, OpsRRP => holeybytes::Relative16,
        }
    }

    macro_rules! ignore_const_one {
        ($_:tt) => {
            1
        };
    }

    macro_rules! instruction {
        (
            $module:ident,
            $iter:expr,
            $name:ident
            ($($subst:pat),* $(,)?)
            $(,)?
        ) => {{
                #[allow(unused)] let iter = $iter;
                #[allow(unused)] const OPSN: u8 = 0 $(+ ignore_const_one!($subst))*;
                #[allow(unused)] let mut counter = 0;

                Operands::$name(
                    $module::$name(
                        $({
                            #[allow(clippy::let_unit_value)]
                            let $subst = ();
                            let item = FromToken::from_token(iter)?;

                            counter += 1;
                            if counter < OPSN
                                && !matches!(iter.next().ok_or(Error::NotEnoughTokens)?, Token::Sigil(s) if s == ",")
                                { return Err(Error::TooManyOps); }

                            item
                        }),*
                    )
                )
        }};
    }

    macro_rules! generate {
        (
            $opsty:expr, $iter:expr,
            simple {
                $($s_name:ident (
                    $($s_subst:pat),* $(,)?
                )),* $(,)?
            },
            addressing {
                $($a_name:ident (
                    $($a_subst:pat),* $(,)?
                )),* $(,)?
            }
            $(,)?
        ) => {{
            let opsty = $opsty;
            let iter  = $iter;

            Ok(match opsty {
                $(OpsType::$s_name =>
                    instruction!(holeybytes, iter, $s_name ($($s_subst),*))
                ),*,
                $(OpsType::$a_name =>
                    instruction!(addressing, iter, $a_name ($($a_subst),*))
                ),*
            })
        }};
    }

    generate!(opsty, iter,
        simple {
            OpsRR   (_, _),
            OpsRRR  (_, _, _),
            OpsRRRR (_, _, _, _),
            OpsRRB  (_, _, _),
            OpsRRH  (_, _, _),
            OpsRRW  (_, _, _),
            OpsRB   (_, _),
            OpsRH   (_, _),
            OpsRW   (_, _),
            OpsRD   (_, _),
            OpsRRD  (_, _, _),
            OpsO    (_),
            OpsP    (_),
            OpsN    ( ),
        },

        addressing {
            OpsRRAH (_, _, _),
            OpsRROH (_, _, _),
            OpsRRPH (_, _, _),
            OpsRRO  (_, _),
            OpsRRA  (_, _),
            OpsRRP  (_, _),
        },
    )
}

fn address(iter: &mut Peekable<impl Iterator<Item = Token>>) -> Result<(Register, Address)> {
    match parse_simple_expr(iter) {
        Expression::Symbol(name) => Ok((Register(0), Address::Symbol { name, disp: 0 })),
        Expression::Integer(abs) => Ok((Register(0), Address::Abs(abs))),
        Expression::Group('[', grp) => {
            let res = gsqb_address(*grp)?;
            let disp = |n: u128| (n as i128).try_into().map_err(|_| Error::IntTooBig);

            // > I hate else-if with passion.
            // — Erin
            if let Some(name) = res.sym {
                if res.pcrel {
                    return Err(Error::AddressItemTwiceSet);
                }

                Ok((
                    res.register,
                    Address::Symbol {
                        name,
                        disp: disp(res.imm)?,
                    },
                ))
            } else if res.pcrel {
                Ok((res.register, Address::Disp(disp(res.imm)?)))
            } else {
                Ok((res.register, Address::Abs(res.imm)))
            }
        }
        _ => Err(Error::UnexpectedToken),
    }
}

struct GsQbResultOk {
    register: Register,
    sym: Option<String>,
    pcrel: bool,
    imm: u128,
}

/// Group square bracked address
fn gsqb_address(expr: Expression) -> Result<GsQbResultOk> {
    #[derive(Debug, Default)]
    struct ReductionData {
        register: Option<Register>,
        sym: Option<String>,
        pcrel: bool,
    }

    fn reduce(expr: Expression, data: &mut ReductionData) -> Result<u128> {
        match expr {
            Expression::Symbol(sym) if sym == "pc" && !data.pcrel => data.pcrel = true,
            Expression::Symbol(sym) if sym == "pc" => return Err(Error::AddressItemTwiceSet),
            Expression::Symbol(sym) => {
                if let Some(n) = sym.strip_prefix('r') {
                    if let Ok(n) = n.parse() {
                        if data.register.is_none() {
                            data.register = Some(Register(n))
                        } else {
                            return Err(Error::AddressItemTwiceSet);
                        }
                    }
                } else if data.sym.is_none() {
                    data.sym = Some(sym)
                } else {
                    return Err(Error::AddressItemTwiceSet);
                }
            }
            Expression::Integer(int) => return Ok(int),
            Expression::Binary(op, lhs, rhs) => {
                use std::ops;

                #[allow(clippy::nonminimal_bool)]
                if false
                    || (![BinaryOp::Add, BinaryOp::Sub].contains(&op)
                        && matches!(&*lhs, Expression::Symbol(_)))
                    || (op != BinaryOp::Add && matches!(&*rhs, Expression::Symbol(_)))
                {
                    return Err(Error::InvalidOps);
                }

                return Ok(match op {
                    BinaryOp::Add => u128::wrapping_add,
                    BinaryOp::Sub => u128::wrapping_sub,
                    BinaryOp::Mul => u128::wrapping_mul,
                    BinaryOp::Div => u128::wrapping_div,
                    BinaryOp::Mod => ops::Rem::rem,
                    BinaryOp::Lsh => ops::Shl::shl,
                    BinaryOp::Rsh => ops::Shr::shr,
                    BinaryOp::And => ops::BitAnd::bitand,
                    BinaryOp::Or => ops::BitOr::bitor,
                    BinaryOp::Xor => ops::BitXor::bitxor,
                    _ => return Err(Error::InvalidOps),
                }(reduce(*lhs, data)?, reduce(*rhs, data)?));
            }
            Expression::Unary(_, _) => todo!(),
            Expression::Group(_, _) => todo!(),
        }

        Ok(0)
    }

    let mut data = ReductionData::default();
    let imm = reduce(expr, &mut data)?;
    Ok(GsQbResultOk {
        register: data.register.unwrap_or(Register(0)),
        sym: data.sym,
        pcrel: data.pcrel,
        imm,
    })
}

trait FromToken: Sized {
    fn from_token(iter: &mut Peekable<impl Iterator<Item = Token>>) -> Result<Self>;
}

impl FromToken for Register {
    fn from_token(iter: &mut Peekable<impl Iterator<Item = Token>>) -> Result<Self> {
        if let Token::Identifier(lit) = iter.next().ok_or(Error::NotEnoughTokens)? {
            Ok(Self(
                lit.strip_prefix('r')
                    .ok_or(Error::ExpectedRegister)?
                    .parse::<u8>()
                    .map_err(|_| Error::ExpectedRegister)?,
            ))
        } else {
            Err(Error::UnexpectedToken)
        }
    }
}

impl FromToken for Address {
    fn from_token(iter: &mut Peekable<impl Iterator<Item = Token>>) -> Result<Self> {
        let (reg, addr) = address(iter)?;
        if reg.0 != 0 {
            return Err(Error::UnexpectedAddressTy);
        }

        Ok(addr)
    }
}

impl FromToken for (Register, Address) {
    #[inline(always)]
    fn from_token(iter: &mut Peekable<impl Iterator<Item = Token>>) -> Result<Self> {
        address(iter)
    }
}

// Can't use generic for 1.56 reasons
macro_rules! from_token_rela {
    ($for:ty, $iter:expr $(,)?) => {
        match $iter.next().ok_or(Error::NotEnoughTokens)? {
            Token::Identifier(name) => Ok(Address::Symbol { name, disp: 0 }),
            Token::IntegerLiteral(disp) => Ok(Address::Disp(
                <$for>::try_from(disp as i128)
                    .map_err(|_| Error::IntTooBig)?
                    .into(),
            )),
            _ => Err(Error::UnexpectedToken),
        }
    };
}

impl FromToken for Relative16 {
    #[inline]
    fn from_token(iter: &mut Peekable<impl Iterator<Item = Token>>) -> Result<Self> {
        from_token_rela!(i16, iter).map(Self)
    }
}

impl FromToken for Relative32 {
    #[inline]
    fn from_token(iter: &mut Peekable<impl Iterator<Item = Token>>) -> Result<Self> {
        from_token_rela!(i32, iter).map(Self)
    }
}

macro_rules! from_token_imms {
    ($($ty:ident),* $(,)?) => {
        $(impl FromToken for $ty {
            fn from_token(iter: &mut Peekable<impl Iterator<Item = Token>>) -> Result<Self> {
                if let Token::IntegerLiteral(lit) = iter.next().ok_or(Error::NotEnoughTokens)?  {
                    Ok($ty::try_from(lit).map_err(|_| Error::IntTooBig)?)
                } else {
                    Err(Error::UnexpectedToken)
                }
            }
        })*
    };
}

from_token_imms!(u8, u16, u32, u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Error {
    IntTooBig,
    UnexpectedToken,
    ExpectedRegister,
    TooManyOps,
    NotEnoughTokens,
    AddressItemTwiceSet,
    InvalidOps,
    UnexpectedAddressTy,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::IntTooBig => "Integer is too big",
            Self::UnexpectedToken => "Unexpected token",
            Self::ExpectedRegister => "Expected register",
            Self::TooManyOps => "Too many operands",
            Self::NotEnoughTokens => "Not enough tokens",
            Self::AddressItemTwiceSet => "Item in address expression set twice",
            Self::InvalidOps => "Attempted to perform invalid operation",
            Self::UnexpectedAddressTy => "Unexpected address type for instruction",
        })
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
