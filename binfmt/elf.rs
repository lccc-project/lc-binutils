
use crate::traits::Numeric;
use crate::traits::private::Sealed;

pub type ElfByte<E: ElfClass> = <E as ElfClass>::Byte;
pub type ElfHalf<E: ElfClass> = <E as ElfClass>::Half;
pub type ElfWord<E: ElfClass> = <E as ElfClass>::Word;
pub type ElfSword<E: ElfClass> = <E as ElfClass>::Sword;
pub type ElfXword<E: ElfClass> = <E as ElfClass>::Xword;
pub type ElfSxword<E: ElfClass> = <E as ElfClass>::Sxword;
pub type ElfAddr<E: ElfClass> = <E as ElfClass>::Addr;
pub type ElfOffset<E: ElfClass> = <E as ElfClass>::Offset;
pub type ElfSection<E: ElfClass> = <E as ElfClass>::Section;
pub type ElfVersym<E: ElfClass> = <E as ElfClass>::Versym;
pub type Symbol<E: ElfClass> = <E as ElfClass>::Symbol;
pub type ElfSize<E: ElfClass> = <E as ElfClass>::Size;

pub trait ElfSymbol: Sealed{
    type Class: ElfClass;
    fn name_idx(&self)->ElfWord<Self::Class>;
    fn value(&self)->ElfAddr<Self::Class>;
    fn size(&self)->ElfSize<Self::Class>;
    fn info(&self)->ElfByte<Self::Class>;
    fn other(&self)->ElfByte<Self::Class>;
    fn section(&self)->ElfSection<Self::Class>;
}

pub trait ElfRelocation: Sealed{
    type Class: ElfClass;
    fn at_offset(&self)->ElfAddr<Self::Class>;
    fn rel_type(&self)->ElfSize<Self::Class>;
    fn symbol(&self)->ElfSize<Self::Class>;
    fn addend(&self)->ElfOffset<Self::Class>{
        Numeric::zero()
    }
}



pub trait ElfClass : Sealed{
    type Byte = u8;
    const EI_CLASS: Self::ElfByte;
    type Half: Numeric = u16;
    type Word: Numeric = u32;
    type Sword: Numeric = i32;
    type Xword: Numeric = u64;
    type Sxword: Numeric = i64;
    type Addr: Numeric;
    type Offset: Numeric;
    type Section: Numeric = u16;
    type Versym: Numeric = u16;
    type Size: Numeric;
    type Symbol: ElfSymbol<Class=Self>;
    type Rel: ElfRelocation<Class=Self>;
    type Rela: ElfRelocation<Class=Self>;
}

pub enum Elf64{}
pub enum Elf32{}

#[repr(C,packed)]
pub struct Elf32Sym{
    st_name: ElfWord<Elf32>,
    st_value: ElfAddr<Elf32>,
    st_size: ElfSize<Elf32>,
    st_info: ElfByte<Elf32>,
    st_other: ElfByte<Elf32>,
    st_shnidx: ElfSection<Elf32>
}

impl Sealed for Elf32Sym{}
impl ElfSymbol for Elf32Sym{
    type Class = Elf32;

    fn name_idx(&self) -> u32 {
        self.st_name
    }

    fn value(&self) -> <Self::Class as ElfClass>::Addr {
        self.st_value
    }

    fn size(&self) -> ElfSize<Self::Class> {
        self.st_size
    }

    fn info(&self) -> u8 {
        self.st_info
    }

    fn other(&self) -> u8 {
        self.st_other
    }

    fn section(&self) -> u16 {
        self.st_shnidx
    }
}

#[repr(C,packed)]
pub struct Elf64Sym{
    st_name: ElfWord<Elf64>,
    st_info: ElfByte<Elf64>,
    st_other: ElfByte<Elf64>,
    st_shnidx: ElfSection<Elf64>,
    st_value: ElfAddr<Elf64>,
    st_size: ElfSize<Elf64>
}

#[repr(C)]
pub struct ElfRel<Class: ElfClass>{
    r_offset: ElfAddr<Class>,
    r_info: ElfSize<Class>
}

trait ElfRelocationExtractHelpers: ElfClass{
    fn symbol(info: ElfSize<Self>) -> ElfSize<Self>;
    fn rel_type(info: ElfSize<Self>) -> ElfSize<Self>;
}

impl<Class: ElfClass + ElfRelocationExtractHelpers> ElfRelocation for ElfRel<Class>{
    type Class = Class;

    fn at_offset(&self) -> <Self::Class as ElfClass>::Addr {
        self.r_offset
    }

    fn rel_type(&self) -> <Self::Class as ElfClass>::Size {
        Class::symbol(self.r_info)
    }

    fn symbol(&self) -> <Self::Class as ElfClass>::Size {
        Class::rel_type(self.r_info)
    }
}

#[repr(C)]
pub struct ElfRela<Class: ElfClass>{
    r_offset: ElfAddr<Class>,
    r_info: ElfSize<Class>,
    r_addend: ElfOffset<Class>
}

impl<Class: ElfClass + ElfRelocationExtractHelpers> ElfRelocation for ElfRela<Class>{
    type Class = Class;

    fn at_offset(&self) -> <Self::Class as ElfClass>::Addr {
        self.r_offset
    }

    fn rel_type(&self) -> <Self::Class as ElfClass>::Size {
        Class::symbol(self.r_info)
    }

    fn symbol(&self) -> <Self::Class as ElfClass>::Size {
        Class::rel_type(self.r_info)
    }
    fn addend(&self) -> <Self::Class as ElfClass>::Offset{
        self.r_addend
    }
}


impl Sealed for Elf64Sym{}
impl ElfSymbol for Elf64Sym{
    type Class = Elf64;

    fn name_idx(&self) -> u32 {
        self.st_name
    }

    fn value(&self) -> <Self::Class as ElfClass>::Addr {
        self.st_value
    }

    fn size(&self) -> ElfSize<Self::Class> {
        self.st_size
    }

    fn info(&self) -> u8 {
        self.st_info
    }

    fn other(&self) -> u8 {
        self.st_other
    }

    fn section(&self) -> u16 {
        self.st_shnidx
    }
}


impl Sealed for Elf64{}
impl ElfClass for Elf64{
    const EI_CLASS: u8 = 2;
    type Addr = u64;
    type Offset = i64;
    type Size = u64;
    type Symbol = Elf64Sym;
    type Rel = ElfRel<Self>;
    type Rela = ElfRela<Self>;
}

impl ElfRelocationExtractHelpers for Elf64{
    fn symbol(info: Self::Size) -> Self::Size {
        info >> 32
    }

    fn rel_type(info: Self::Size) -> Self::Size {
        info & 0xffffffff
    }
}

impl Sealed for Elf32{}
impl ElfClass for Elf32{
    const EI_CLASS: u8 = 1;
    type Addr = u32;
    type Offset = i32;
    type Size = u32;
    type Symbol = Elf32Sym;
    type Rel = ElfRel<Self>;
    type Rela = ElfRela<Self>;
}

impl ElfRelocationExtractHelpers for Elf32{
    fn symbol(info: Self::Size) -> Self::Size {
        info >> 8
    }

    fn rel_type(info: Self::Size) -> Self::Size {
        info & 0xff
    }
}