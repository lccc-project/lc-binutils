#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
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
}

#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
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

    Es,
    Cs,
    Ss,
    Ds,
    Fs,
    Gs,
    UndefSeg,

    #[doc(hidden)]
    __Nonexhaustive,
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
}

impl X86Register {
    pub fn from_class(rclass: X86RegisterClass, rnum: usize) -> Option<X86Register> {
        X86REGISTERS[&rclass].get(rnum).copied()
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
            Es => f.write_str("es"),
            Cs => f.write_str("cs"),
            Ss => f.write_str("ss"),
            Ds => f.write_str("ds"),
            Fs => f.write_str("fs"),
            Gs => f.write_str("gs"),
            UndefSeg => f.write_str("undef"),

            __Nonexhaustive => unreachable!(),
        }
    }
}
