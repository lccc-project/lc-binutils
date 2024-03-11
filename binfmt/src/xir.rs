#![allow(clippy::explicit_auto_deref)] // I want to be explicit because I have walked into it not actually derefing before

use core::{
    char::CharTryFromError,
    convert::TryFrom,
    ops::{Deref, DerefMut, Index, IndexMut},
    slice::SliceIndex,
};

extern crate alloc;
use alloc::string::{FromUtf16Error, FromUtf8Error};

pub mod raw {
    use core::num::NonZeroU32;
    macro_rules! fake_enum{
        {#[repr($t:ty)] $vis:vis enum $name:ident {
            $($item:ident = $expr:literal),*$(,)?
        }} => {
            #[derive(Copy,Clone,Eq,PartialEq)]
            #[repr(transparent)]
            $vis struct $name($t);
            impl $name{

                $(#[allow(non_upper_case_globals)] $vis const $item: $name = $name($expr);)*
            }
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
        #[repr(u32)] pub enum XLangArch{
            X86_64 = 0,
            IX86 = 1,
            W65816 = 2,
            ARM = 3,
            AARCH64 = 4,
            SUPERH = 5,
            MIPS = 6,
            POWERPC = 7,
            POWERPC64 = 8,
            AVR = 9,
            M68000 = 10,

            SPARC = 14,
            RISCV32 = 15,
            RISCV64 = 16,
            WASM32 = 17,
            WASM64 = 18,

            UNKNOWN = 0xFFFFFFFF
        }
    }

    fake_enum! {
        #[repr(u32)] pub enum XLangVendor{
            PC = 0,
            APPLE = 1,
            SNES = 2,

            UNKNOWN = 0xFFFFFFFF
        }
    }

    fake_enum! {
        #[repr(u32)] pub enum XLangOperatingSystem{
            LINUX =0,
            WINDOWS = 2,
            MINGW32 = 3,
            MACOS = 4,
            IOS = 5,
            PHANTOM = 6,

            UNKNOWN = 0xFFFFFFFF
        }
    }

    fake_enum! {
        #[repr(u32)] pub enum XLangEnvironment{
            GNU = 0,
            EABI = 1,
            NONE = 2,
            MSVC = 3,
            MUSL = 4,
            LC = 5,
            PHANTOM_STD = 6,
            PHANTOM_KERNEL = 7,

            UNKNOWN = 0xFFFFFFFF
        }
    }

    #[repr(C)]
    pub struct XLangTarget {
        pub arch: XLangArch,
        pub vendor: XLangVendor,
        pub os: XLangOperatingSystem,
        pub env: XLangEnvironment,
    }

    pub const XIR_MAGIC: [u8; 4] = [0xC6, b'X', b'I', b'R'];

    #[repr(C)]
    pub struct XiRHeader {
        pub magic: [u8; 4],
        pub version: [u8; 2],
        pub target_components: XLangTarget,
        pub source_file_name: Option<NonZeroU32>,
        pub st_tbl_offset: u32,
        pub st_tbl_count: u32,
    }

    fake_enum! {
        #[repr(u16)]
        pub enum StChar{
            Bytes = 0,
            Utf8 = 2,
            Utf16 = 3,
            Utf32 = 4
        }
    }

    #[repr(C)]
    pub struct StEntry {
        pub char: StChar,
        pub char_width: u16,
        pub sz: u32,
    }

    fake_enum! {
        #[repr(u16)]
        pub enum IdComponentType{
            None = 0,
            Root = 1,
            Normal = 2,
            Special = 3,
            Param = 4,
            GenericArg = 5
        }
    }

    #[repr(C)]
    pub struct IdComponent {
        pub id_type: IdComponentType,
        pub position: u16,
        pub value: u32,
    }
}

pub enum StringEntry {
    Bytes(Vec<u8>),
    Utf8(String),
    Utf16(Vec<u16>),
    Utf32(Vec<u32>),
}

#[derive(Debug)]
pub enum FromStringEntryError {
    Utf8(FromUtf8Error),
    Utf16(FromUtf16Error),
    Utf32(CharTryFromError),
}

impl From<FromUtf8Error> for FromStringEntryError {
    fn from(v: FromUtf8Error) -> Self {
        Self::Utf8(v)
    }
}

impl From<FromUtf16Error> for FromStringEntryError {
    fn from(v: FromUtf16Error) -> Self {
        Self::Utf16(v)
    }
}

impl From<CharTryFromError> for FromStringEntryError {
    fn from(v: CharTryFromError) -> Self {
        Self::Utf32(v)
    }
}

impl TryFrom<StringEntry> for String {
    type Error = FromStringEntryError;

    fn try_from(value: StringEntry) -> Result<Self, Self::Error> {
        match value {
            StringEntry::Bytes(b) => String::from_utf8(b).map_err(Into::into),
            StringEntry::Utf8(s) => Ok(s),
            StringEntry::Utf16(v) => String::from_utf16(&v).map_err(Into::into),
            StringEntry::Utf32(v) => v
                .into_iter()
                .map(char::try_from)
                .collect::<Result<_, _>>()
                .map_err(Into::into),
        }
    }
}

pub struct StringTable {
    items: Vec<StringEntry>,
}

impl<I: SliceIndex<[StringEntry]>> Index<I> for StringTable {
    type Output = <I as SliceIndex<[StringEntry]>>::Output;

    fn index(&self, si: I) -> &Self::Output {
        &self.items[si]
    }
}

impl<I: SliceIndex<[StringEntry]>> IndexMut<I> for StringTable {
    fn index_mut(&mut self, si: I) -> &mut Self::Output {
        &mut self.items[si]
    }
}

impl Deref for StringTable {
    type Target = [StringEntry];
    fn deref(&self) -> &Self::Target {
        &*self.items
    }
}

impl DerefMut for StringTable {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.items
    }
}

pub enum Type {
    Scalar(ScalarType),
}

pub struct ScalarType {}

pub enum Value {
    Int(Type, u64),
    ExtendedInt(Type, Vec<u8>),
    Float(Type, f64),
}
