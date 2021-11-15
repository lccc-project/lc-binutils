#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
#[non_exhaustive]
pub enum X86RegisterClass {
    Byte,
    ByteRex,
    Word,
    Double,
    Quad,
    Mmx,
    Xmm,
    Ymm,
    Zmm,
    Tmm,
    Sreg,
    Cr,
    Dr,
    Tr,
    St,
    AvxMask,
}

impl X86RegisterClass {
    pub fn size(&self, mode: X86Mode) -> usize {
        match self {
            Self::Byte | Self::ByteRex => 1,
            Self::Word | Self::Sreg => 2,
            Self::Double | Self::Tr | Self::Dr => 4,
            Self::Quad | Self::Mmx => 8,
            Self::Xmm => 16,
            Self::Ymm => 32,
            Self::Zmm => 64,
            Self::Tmm => 1024,
            Self::Cr if mode == X86Mode::Long => 8,
            Self::Cr => 4,
            Self::St => 10,
            Self::AvxMask => 8,
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
#[non_exhaustive]
pub enum X86Register {
    // r8
    Al,
    Cl,
    Dl,
    Bl,

    // r8 high (no REX prefix)
    Ah,
    Ch,
    Dh,
    Bh,
    // r8 high (any REX prefix)
    Spl,
    Bpl,
    Sil,
    Dil,

    // r8 (REX.B/REX.R)
    R8b,
    R9b,
    R10b,
    R11b,
    R12b,
    R13b,
    R14b,
    R15b,

    // r16
    Ax,
    Cx,
    Dx,
    Bx,
    Sp,
    Bp,
    Si,
    Di,

    // r16 high (REX.B/REX.R)
    R8w,
    R9w,
    R10w,
    R11w,
    R12w,
    R13w,
    R14w,
    R15w,

    // r32
    Eax,
    Ecx,
    Edx,
    Ebx,
    Esp,
    Ebp,
    Esi,
    Edi,

    // r32 high (REX.B/REX.R)
    R8d,
    R9d,
    R10d,
    R11d,
    R12d,
    R13d,
    R14d,
    R15d,

    // r64
    Rax,
    Rcx,
    Rdx,
    Rbx,
    Rsp,
    Rbp,
    Rsi,
    Rdi,

    // r64 high (REX.B/REX.R)
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,

    Mmx(u8),

    Xmm(u8),
    Ymm(u8),
    Zmm(u8),
    Tmm(u8),

    Cr(u8),
    Dr(u8),
    Tr(u8),

    Fp(u8),

    K(u8),

    Es,
    Cs,
    Ss,
    Ds,
    Fs,
    Gs,
    UndefSeg,
}

use std::{collections::HashMap, fmt::Display};

macro_rules! define_x86_registers{
    {
        $(regs [$($regs:expr),*]: $class:ident ;)*
    } => {
        lazy_static::lazy_static!{
            static ref X86REGISTERS: HashMap<X86RegisterClass,&'static [X86Register]> = {
                let mut map = HashMap::<X86RegisterClass,&'static [X86Register]>::new();
                $({
                    map.insert(X86RegisterClass:: $class, &[$($regs),*]);
                })*

                map
            };
        }
    }
}

use X86Register::*;

use self::insn::X86Mode;

define_x86_registers! {
    regs [Al, Cl, Dl, Bl, Ah, Ch, Dh, Bh]: Byte;
    regs [Al, Cl, Dl, Bl, Spl, Bpl, Sil, Dil, R8b, R9b, R10b, R11b, R12b, R13b, R14b, R15b]: ByteRex;
    regs [Ax, Cx, Dx, Bx, Sp, Bp, Si, Di, R8w, R9w, R10w, R11w, R12w, R13w, R14w, R15w]: Word;
    regs [Eax, Ecx, Edx, Ebx, Esp, Ebp, Esi, Edi, R8d, R9d, R10d, R11d, R12d, R13d, R14d, R15d]: Double;
    regs [Rax, Rcx, Rdx, Rbx, Rsp, Rbp, Rsi, Rdi, R8, R9, R10, R11, R12, R13, R14, R15]: Quad;
    regs [Mmx(0), Mmx(1), Mmx(2), Mmx(3), Mmx(4), Mmx(5), Mmx(6), Mmx(7), Mmx(0), Mmx(1), Mmx(2), Mmx(3), Mmx(4), Mmx(5), Mmx(6), Mmx(7)]: Mmx;
    regs [Xmm(0), Xmm(1), Xmm(2), Xmm(3), Xmm(4), Xmm(5), Xmm(6), Xmm(7), Xmm(8), Xmm(9), Xmm(10), Xmm(11), Xmm(12), Xmm(13), Xmm(14), Xmm(15),
        Xmm(16), Xmm(17), Xmm(18), Xmm(19), Xmm(20), Xmm(21), Xmm(22), Xmm(23), Xmm(24), Xmm(25), Xmm(26), Xmm(27), Xmm(28), Xmm(29), Xmm(30), Xmm(31)]: Xmm;
    regs [Ymm(0), Ymm(1), Ymm(2), Ymm(3), Ymm(4), Ymm(5), Ymm(6), Ymm(7), Ymm(8), Ymm(9), Ymm(10), Ymm(11), Ymm(12), Ymm(13), Ymm(14), Ymm(15),
        Ymm(16), Ymm(17), Ymm(18), Ymm(19), Ymm(20), Ymm(21), Ymm(22), Ymm(23), Ymm(24), Ymm(25), Ymm(26), Ymm(27), Ymm(28), Ymm(29), Ymm(30), Ymm(31)]: Ymm;
    regs [Zmm(0), Zmm(1), Zmm(2), Zmm(3), Zmm(4), Zmm(5), Zmm(6), Zmm(7), Zmm(8), Zmm(9), Zmm(10), Zmm(11), Zmm(12), Zmm(13), Zmm(14), Zmm(15),
        Zmm(16), Zmm(17), Zmm(18), Zmm(19), Zmm(20), Zmm(21), Zmm(22), Zmm(23), Zmm(24), Zmm(25), Zmm(26), Zmm(27), Zmm(28), Zmm(29), Zmm(30), Zmm(31)]: Zmm;
    regs [Tmm(0), Tmm(1), Tmm(2), Tmm(3), Tmm(4), Tmm(5), Tmm(6), Tmm(7)]: Tmm;
    regs [Es, Cs, Ss, Ds, Fs, Gs, UndefSeg, UndefSeg, Es, Cs, Ss, Ds, Fs, Gs, UndefSeg, UndefSeg]: Sreg;
    regs [Cr(0), Cr(1), Cr(2), Cr(3), Cr(4), Cr(5), Cr(6), Cr(7), Cr(8), Cr(9), Cr(10), Cr(11), Cr(12), Cr(13), Cr(14), Cr(15)]: Cr;
    regs [Dr(0), Dr(1), Dr(2), Dr(3), Dr(4), Dr(5), Dr(6), Dr(7), Dr(8), Dr(9), Dr(10), Dr(11), Dr(12), Dr(13), Dr(14), Dr(15)]: Dr;
    regs [Tr(0), Tr(1), Tr(2), Tr(3), Tr(4), Tr(5), Tr(6), Tr(7)]: Tr;
    regs [Fp(0), Fp(1), Fp(2), Fp(3), Fp(4), Fp(5), Fp(6), Fp(7),Fp(0), Fp(1), Fp(2), Fp(3), Fp(4), Fp(5), Fp(6), Fp(7)]: St;
    regs [K(0), K(1), K(2), K(3), K(4), K(5), K(6), K(7)]: AvxMask;
}

impl X86Register {
    pub fn from_class(rclass: X86RegisterClass, rnum: u8) -> Option<X86Register> {
        X86REGISTERS[&rclass].get(rnum as usize).copied()
    }

    pub fn regnum(self) -> u8 {
        match self {
            Al => 0,
            Cl => 1,
            Dl => 2,
            Bl => 3,
            Ah => 4,
            Ch => 5,
            Dh => 6,
            Bh => 7,
            Spl => 4,
            Bpl => 5,
            Sil => 6,
            Dil => 7,
            R8b => 8,
            R9b => 9,
            R10b => 10,
            R11b => 11,
            R12b => 12,
            R13b => 13,
            R14b => 14,
            R15b => 15,
            Ax => 0,
            Cx => 1,
            Dx => 2,
            Bx => 3,
            Sp => 4,
            Bp => 5,
            Si => 6,
            Di => 7,
            R8w => 8,
            R9w => 9,
            R10w => 10,
            R11w => 11,
            R12w => 12,
            R13w => 13,
            R14w => 14,
            R15w => 15,
            Eax => 0,
            Ecx => 1,
            Edx => 2,
            Ebx => 3,
            Esp => 4,
            Ebp => 5,
            Esi => 6,
            Edi => 7,
            R8d => 8,
            R9d => 9,
            R10d => 10,
            R11d => 11,
            R12d => 12,
            R13d => 13,
            R14d => 14,
            R15d => 15,
            Rax => 0,
            Rcx => 1,
            Rdx => 2,
            Rbx => 3,
            Rsp => 4,
            Rbp => 5,
            Rsi => 6,
            Rdi => 7,
            R8 => 8,
            R9 => 9,
            R10 => 10,
            R11 => 11,
            R12 => 12,
            R13 => 13,
            R14 => 14,
            R15 => 15,
            Mmx(m) => m,
            Xmm(m) => m,
            Ymm(m) => m,
            Zmm(m) => m,
            Tmm(m) => m,
            Cr(m) => m,
            Dr(m) => m,
            Tr(m) => m,
            Fp(m) => m,
            K(m) => m,
            Es => 0,
            Cs => 1,
            Ss => 2,
            Ds => 3,
            Fs => 4,
            Gs => 5,
            UndefSeg => 255,
        }
    }

    pub fn class(&self) -> X86RegisterClass {
        match self {
            Al => X86RegisterClass::Byte,
            Cl => X86RegisterClass::Byte,
            Dl => X86RegisterClass::Byte,
            Bl => X86RegisterClass::Byte,
            Ah => X86RegisterClass::Byte,
            Ch => X86RegisterClass::Byte,
            Dh => X86RegisterClass::Byte,
            Bh => X86RegisterClass::Byte,
            Spl => X86RegisterClass::ByteRex,
            Bpl => X86RegisterClass::ByteRex,
            Sil => X86RegisterClass::ByteRex,
            Dil => X86RegisterClass::ByteRex,
            R8b => X86RegisterClass::ByteRex,
            R9b => X86RegisterClass::ByteRex,
            R10b => X86RegisterClass::ByteRex,
            R11b => X86RegisterClass::ByteRex,
            R12b => X86RegisterClass::ByteRex,
            R13b => X86RegisterClass::ByteRex,
            R14b => X86RegisterClass::ByteRex,
            R15b => X86RegisterClass::ByteRex,
            Ax => X86RegisterClass::Word,
            Cx => X86RegisterClass::Word,
            Dx => X86RegisterClass::Word,
            Bx => X86RegisterClass::Word,
            Sp => X86RegisterClass::Word,
            Bp => X86RegisterClass::Word,
            Si => X86RegisterClass::Word,
            Di => X86RegisterClass::Word,
            R8w => X86RegisterClass::Word,
            R9w => X86RegisterClass::Word,
            R10w => X86RegisterClass::Word,
            R11w => X86RegisterClass::Word,
            R12w => X86RegisterClass::Word,
            R13w => X86RegisterClass::Word,
            R14w => X86RegisterClass::Word,
            R15w => X86RegisterClass::Word,
            Eax => X86RegisterClass::Double,
            Ecx => X86RegisterClass::Double,
            Edx => X86RegisterClass::Double,
            Ebx => X86RegisterClass::Double,
            Esp => X86RegisterClass::Double,
            Ebp => X86RegisterClass::Double,
            Esi => X86RegisterClass::Double,
            Edi => X86RegisterClass::Double,
            R8d => X86RegisterClass::Double,
            R9d => X86RegisterClass::Double,
            R10d => X86RegisterClass::Double,
            R11d => X86RegisterClass::Double,
            R12d => X86RegisterClass::Double,
            R13d => X86RegisterClass::Double,
            R14d => X86RegisterClass::Double,
            R15d => X86RegisterClass::Double,
            Rax => X86RegisterClass::Quad,
            Rcx => X86RegisterClass::Quad,
            Rdx => X86RegisterClass::Quad,
            Rbx => X86RegisterClass::Quad,
            Rsp => X86RegisterClass::Quad,
            Rbp => X86RegisterClass::Quad,
            Rsi => X86RegisterClass::Quad,
            Rdi => X86RegisterClass::Quad,
            R8 => X86RegisterClass::Quad,
            R9 => X86RegisterClass::Quad,
            R10 => X86RegisterClass::Quad,
            R11 => X86RegisterClass::Quad,
            R12 => X86RegisterClass::Quad,
            R13 => X86RegisterClass::Quad,
            R14 => X86RegisterClass::Quad,
            R15 => X86RegisterClass::Quad,
            Mmx(_) => X86RegisterClass::Mmx,
            Xmm(_) => X86RegisterClass::Xmm,
            Ymm(_) => X86RegisterClass::Ymm,
            Zmm(_) => X86RegisterClass::Zmm,
            Tmm(_) => X86RegisterClass::Tmm,
            Cr(_) => X86RegisterClass::Cr,
            Dr(_) => X86RegisterClass::Dr,
            Tr(_) => X86RegisterClass::Tr,
            Fp(_) => X86RegisterClass::St,
            K(_) => X86RegisterClass::AvxMask,
            Es => X86RegisterClass::Sreg,
            Cs => X86RegisterClass::Sreg,
            Ss => X86RegisterClass::Sreg,
            Ds => X86RegisterClass::Sreg,
            Fs => X86RegisterClass::Sreg,
            Gs => X86RegisterClass::Sreg,
            UndefSeg => X86RegisterClass::Sreg,
        }
    }
}

impl Display for X86Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Al => f.write_str("al"),
            Cl => f.write_str("cl"),
            Dl => f.write_str("dl"),
            Bl => f.write_str("bl"),
            Ah => f.write_str("ah"),
            Ch => f.write_str("ch"),
            Dh => f.write_str("dh"),
            Bh => f.write_str("bh"),
            Spl => f.write_str("spl"),
            Bpl => f.write_str("bpl"),
            Sil => f.write_str("sil"),
            Dil => f.write_str("dil"),
            R8b => f.write_str("r8b"),
            R9b => f.write_str("r9b"),
            R10b => f.write_str("r10b"),
            R11b => f.write_str("r11b"),
            R12b => f.write_str("r12b"),
            R13b => f.write_str("r13b"),
            R14b => f.write_str("r14b"),
            R15b => f.write_str("r15b"),
            Ax => f.write_str("ax"),
            Cx => f.write_str("cx"),
            Dx => f.write_str("dx"),
            Bx => f.write_str("bx"),
            Sp => f.write_str("sp"),
            Bp => f.write_str("bp"),
            Si => f.write_str("si"),
            Di => f.write_str("di"),
            R8w => f.write_str("r8w"),
            R9w => f.write_str("r9w"),
            R10w => f.write_str("r10w"),
            R11w => f.write_str("r11w"),
            R12w => f.write_str("r12w"),
            R13w => f.write_str("r13w"),
            R14w => f.write_str("r14w"),
            R15w => f.write_str("r15w"),
            Eax => f.write_str("eax"),
            Ecx => f.write_str("ecx"),
            Edx => f.write_str("edx"),
            Ebx => f.write_str("ebx"),
            Esp => f.write_str("esp"),
            Ebp => f.write_str("ebp"),
            Esi => f.write_str("esi"),
            Edi => f.write_str("edi"),
            R8d => f.write_str("r8d"),
            R9d => f.write_str("r9d"),
            R10d => f.write_str("r10d"),
            R11d => f.write_str("r11d"),
            R12d => f.write_str("r12d"),
            R13d => f.write_str("r13d"),
            R14d => f.write_str("r14d"),
            R15d => f.write_str("r15d"),
            Rax => f.write_str("rax"),
            Rcx => f.write_str("rcx"),
            Rdx => f.write_str("rdx"),
            Rbx => f.write_str("rbx"),
            Rsp => f.write_str("rsp"),
            Rbp => f.write_str("rbp"),
            Rsi => f.write_str("rsi"),
            Rdi => f.write_str("rdi"),
            R8 => f.write_str("r8"),
            R9 => f.write_str("r9"),
            R10 => f.write_str("r10"),
            R11 => f.write_str("r11"),
            R12 => f.write_str("r12"),
            R13 => f.write_str("r13"),
            R14 => f.write_str("r14"),
            R15 => f.write_str("r15"),
            Mmx(n) => f.write_fmt(format_args!("mm{}", n)),
            Xmm(n) => f.write_fmt(format_args!("xmm{}", n)),
            Ymm(n) => f.write_fmt(format_args!("ymm{}", n)),
            Zmm(n) => f.write_fmt(format_args!("zmm{}", n)),
            Tmm(n) => f.write_fmt(format_args!("tmm{}", n)),
            Cr(n) => f.write_fmt(format_args!("cr{}", n)),
            Dr(n) => f.write_fmt(format_args!("dr{}", n)),
            Tr(n) => f.write_fmt(format_args!("tr{}", n)),
            Fp(n) => f.write_fmt(format_args!("st{}", n)),
            K(n) => f.write_fmt(format_args!("k{}", n)),
            Es => f.write_str("es"),
            Cs => f.write_str("cs"),
            Ss => f.write_str("ss"),
            Ds => f.write_str("ds"),
            Fs => f.write_str("fs"),
            Gs => f.write_str("gs"),
            UndefSeg => f.write_str("undef"),
        }
    }
}

pub mod cpu;
pub mod features;

pub mod insn;
