use bytemuck::{Pod, Zeroable};

macro_rules! fake_enum{
    {#[repr($t:ty)] $vis:vis enum $name:ident {
        $($item:ident = $expr:literal),*$(,)?
    }} => {
        #[derive(Copy,Clone,Eq,PartialEq,Zeroable,Pod)]
        #[repr(transparent)]
        $vis struct $name($t);

        $(#[allow(non_upper_case_globals)] $vis const $item: $name = $name($expr);)*

        impl ::core::fmt::Debug for $name{
            #[allow(unreachable_patterns)]
            fn fmt(&self,f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result{
                match self{
                    $(Self($expr) => f.write_str(::core::stringify!($item)),)*
                    e => e.0.fmt(f)
                }
            }
        }
    }
}

fake_enum! {
    #[repr(u32)]
    pub enum CpuType{
        Any = 0xffffffff,
        Vax = 1,
        MC68k = 6,
        X86 = 7,
        X86_64 = 0xff000007,
        MC98k = 10,
        Hppa = 11,
        Arm = 12,
        Arm64 = 0xff00000c,
        MC88k = 13,
        Sparc = 14,
        I860 = 15,
        PowerPC = 16,
        PowerPC64 = 0xff000010
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Zeroable, Pod)]
pub struct MachOHeader32 {
    magic: u32,
    cputype: CpuType,
    cpusubtype: u32,
    filetype: u32,
    ncmds: u32,
    sizecmds: u32,
    flags: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Zeroable, Pod)]
pub struct MachOHeader64 {
    magic: u32,
    cputype: CpuType,
    cpusubtype: u32,
    filetype: u32,
    ncmds: u32,
    sizecmds: u32,
    flags: u32,
    reserved: u32,
}

pub const MH_MAGIC32: u32 = 0xfeedface;
pub const MH_MAGIC64: u32 = 0xfeedfacf;
pub const MH_REVERSED_MAGIC32: u32 = MH_MAGIC32.swap_bytes();
pub const MH_REVERSED_MAGIC64: u32 = MH_MAGIC64.swap_bytes();
