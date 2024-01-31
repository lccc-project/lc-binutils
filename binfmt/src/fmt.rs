use std::{
    any::Any,
    collections::{hash_map::Values, HashMap},
    io::{self, Read, Write},
    ops::BitOr,
    slice::{Iter, IterMut},
};

use crate::{
    howto::{HowTo, Reloc, RelocCode},
    sym::{Symbol, SymbolKind, SymbolType},
    traits::ReadSeek,
};

use arch_ops::{
    disasm::OpcodePrinter,
    traits::{Address, InsnWrite},
};

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum CallbackError {
    InvalidType,
    NotAccepted,
}

pub trait Binfmt {
    fn relnum_to_howto(&self, relnum: u32) -> Option<&dyn HowTo>;
    fn code_to_howto(&self, code: RelocCode) -> Option<&dyn HowTo>;

    fn name(&self) -> &'static str;
    fn create_file(&self, ty: FileType) -> BinaryFile;
    fn ident_file(&self, file: &mut (dyn Read + '_)) -> io::Result<bool>;
    fn file_priority(&self) -> i32 {
        0
    }
    fn read_file(&self, file: &mut (dyn ReadSeek + '_)) -> io::Result<Option<BinaryFile>>;
    fn write_file(&self, file: &mut (dyn Write + '_), bfile: &BinaryFile) -> io::Result<()>;

    fn has_sections(&self) -> bool;

    fn disassembler(&self) -> Option<&dyn OpcodePrinter> {
        None
    }

    fn create_section(&self, _section: &mut Section) -> Result<(), CallbackError> {
        Ok(())
    }
    fn create_symbol(&self, _sym: &mut Symbol) -> Result<(), CallbackError> {
        Ok(())
    }
    fn create_reloc(&self, _reloc: &mut Reloc) -> Result<(), CallbackError> {
        Ok(())
    }
    fn before_relocate(&self, _reloc: &mut Reloc, _symbol: &Symbol) {}

    fn create_group(&self, _group: &mut SectionGroup) -> Result<(), CallbackError> {
        Ok(())
    }

    fn has_groups(&self) -> bool {
        false
    }
}

impl core::fmt::Debug for dyn Binfmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

impl core::cmp::PartialEq for dyn Binfmt {
    fn eq(&self, rhs: &Self) -> bool {
        core::ptr::eq(self as *const _ as *const u8, rhs as *const _ as *const u8)
        // Binary Formats are unique and singleton
    }
}

impl core::cmp::Eq for dyn Binfmt {}

impl core::hash::Hash for dyn Binfmt {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::ptr::hash(self as *const _ as *const u8, state)
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum FileType {
    Exec,
    Relocatable,
    SharedObject,
    FormatSpecific(u32),
}

pub struct BinaryFile<'a> {
    sections: Option<Vec<Section>>,
    symbols: Option<HashMap<String, Symbol>>,
    relocs: Option<Vec<Reloc>>,
    groups: Option<Vec<SectionGroup>>,
    fmt: &'a dyn Binfmt,
    data: Box<dyn Any>,
    ty: FileType,
}

impl<'a> BinaryFile<'a> {
    pub fn create(fmt: &'a dyn Binfmt, data: Box<dyn Any>, ty: FileType) -> Self {
        Self {
            sections: None,
            symbols: None,
            relocs: None,
            groups: None,
            fmt,
            data,
            ty,
        }
    }

    pub fn file_type(&self) -> &FileType {
        &self.ty
    }

    pub fn data(&self) -> &dyn Any {
        &*self.data
    }

    pub fn data_mut(&mut self) -> &mut dyn Any {
        &mut *self.data
    }

    pub fn add_section(&mut self, mut sect: Section) -> Result<u32, Section> {
        if self.sections.is_none() {
            if !self.fmt.has_sections() {
                return Err(sect);
            }
            self.sections = Some(Vec::new());
        }
        let sections = self.sections.as_mut().unwrap();
        let num = sections.len();
        if num >= (u32::max_value() as usize) {
            panic!("Too many sections created in a binary file");
        }
        if self.fmt.create_section(&mut sect).is_err() {
            return Err(sect);
        }
        sections.push(sect);
        Ok(num as u32)
    }

    pub fn add_section_group(&mut self, mut group: SectionGroup) -> Result<u32, SectionGroup> {
        if self.groups.is_none() {
            if !self.fmt.has_groups() {
                return Err(group);
            }
            self.groups = Some(Vec::new());
        }
        let groups = self.groups.as_mut().unwrap();
        let num = groups.len();
        if num >= (u32::max_value() as usize) {
            panic!("Too many sections created in a binary file");
        }
        if self.fmt.create_group(&mut group).is_err() {
            return Err(group);
        }
        groups.push(group);
        Ok(num as u32)
    }

    pub fn sections(&self) -> Sections<'_> {
        Sections(self.sections.as_ref().map(|x| x.iter()))
    }

    pub fn sections_mut(&mut self) -> SectionsMut<'_> {
        SectionsMut(self.sections.as_mut().map(|x| x.iter_mut()))
    }

    pub fn section_groups(&self) -> SectionGroups<'_> {
        SectionGroups(self.groups.as_ref().map(|x| x.iter()))
    }

    pub fn get_section(&self, secno: u32) -> Option<&Section> {
        self.sections
            .as_ref()
            .and_then(|sect| sect.get(secno as usize))
    }

    pub fn remove_section(&mut self, x: u32) -> Option<Section> {
        self.sections
            .as_mut()
            .filter(|v| (x as usize) < v.len())
            .map(|v| v.remove(x as usize))
    }

    pub fn add_symbols<I: IntoIterator<Item = Symbol>>(
        &mut self,
        syms: I,
    ) -> Result<(), CallbackError> {
        if self.symbols.is_none() {
            self.symbols = Some(HashMap::new());
        }

        let symtab = self.symbols.as_mut().unwrap();

        for mut sym in syms {
            self.fmt.create_symbol(&mut sym)?;
            symtab.insert(sym.name().to_string(), sym);
        }

        Ok(())
    }

    pub fn get_or_create_symbol(&mut self, name: &str) -> Result<&mut Symbol, CallbackError> {
        if self.symbols.is_none() {
            self.symbols = Some(HashMap::new());
        }

        let symtab = self.symbols.as_mut().unwrap();
        // SAFETY: Hecking NLL not being powerful enough
        if let Some(x) = unsafe { &mut *(symtab as *mut HashMap<String, Symbol>) }.get_mut(name) {
            return Ok(x);
        }
        {
            let mut sym = Symbol::new_undef(name.to_string(), SymbolType::Null, SymbolKind::Local);
            self.fmt.create_symbol(&mut sym)?;
            symtab.insert(name.to_string(), sym);
            Ok(symtab.get_mut(name).unwrap())
        }
    }

    pub fn insert_symbol(&mut self, mut sym: Symbol) -> Result<(), Symbol> {
        if self.symbols.is_none() {
            self.symbols = Some(HashMap::new());
        }

        let symbols = self.symbols.as_mut().unwrap();
        if self.fmt.create_symbol(&mut sym).is_err() {
            Err(sym)
        } else {
            let name = sym.name().to_string();
            symbols.insert(name, sym);
            Ok(())
        }
    }

    pub fn symbols(&self) -> Symbols {
        Symbols(self.symbols.as_ref().map(|x| x.values()))
    }

    pub fn remove_symbol(&mut self, name: &str) -> Option<Symbol> {
        self.symbols.as_mut().and_then(|x| x.remove(name))
    }

    pub fn create_reloc(&mut self, mut reloc: Reloc) -> Result<(), Reloc> {
        if self.relocs.is_none() {
            self.relocs = Some(Vec::new());
        }

        let relocs = self.relocs.as_mut().unwrap();
        if self.fmt.create_reloc(&mut reloc).is_err() {
            Err(reloc)
        } else {
            relocs.push(reloc);
            Ok(())
        }
    }

    pub fn relocs(&self) -> Relocs {
        Relocs(self.relocs.as_ref().map(|x| x.iter()))
    }

    pub fn remove_reloc(&mut self, x: usize) -> Option<Reloc> {
        self.relocs
            .as_mut()
            .filter(|v| x < v.len())
            .map(|v| v.remove(x))
    }

    pub fn fmt(&self) -> &'a (dyn Binfmt + 'a) {
        self.fmt
    }
}

pub struct SectionGroups<'a>(Option<Iter<'a, SectionGroup>>);

impl<'a> Iterator for SectionGroups<'a> {
    type Item = &'a SectionGroup;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.as_mut().and_then(|x| x.next())
    }
}

pub struct Sections<'a>(Option<Iter<'a, Section>>);

impl<'a> Iterator for Sections<'a> {
    type Item = &'a Section;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.as_mut().and_then(|x| x.next())
    }
}

pub struct SectionsMut<'a>(Option<IterMut<'a, Section>>);

impl<'a> Iterator for SectionsMut<'a> {
    type Item = &'a mut Section;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.as_mut().and_then(|x| x.next())
    }
}

pub struct Symbols<'a>(Option<Values<'a, String, Symbol>>);

impl<'a> Iterator for Symbols<'a> {
    type Item = &'a Symbol;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.as_mut().and_then(|x| x.next())
    }
}

pub struct Relocs<'a>(Option<Iter<'a, Reloc>>);

impl<'a> Iterator for Relocs<'a> {
    type Item = &'a Reloc;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.as_mut().and_then(|x| x.next())
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
#[non_exhaustive]
pub enum HashTableType {
    Elf,
    Gnu,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
#[non_exhaustive]
pub enum SectionType {
    NoBits,
    ProgBits,
    SymbolTable,
    SymbolHashTable(HashTableType),
    StringTable,
    Dynamic,
    ProcedureLinkageTable,
    GlobalOffsetTable,
    RelocationTable,
    RelocationAddendTable,
    Note,
    FormatSpecific(u32),
}

impl Default for SectionType {
    fn default() -> SectionType {
        SectionType::NoBits
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
#[non_exhaustive]
pub enum SectionFlag {
    Writable,
    Alloc,
    Executable,
    FormatSpecific(u32),
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Default)]
#[non_exhaustive]
pub struct SectionFlags(u64);

impl From<SectionFlag> for SectionFlags {
    #[inline]
    fn from(value: SectionFlag) -> Self {
        match value {
            SectionFlag::Writable => Self(1),
            SectionFlag::Alloc => Self(2),
            SectionFlag::Executable => Self(4),
            SectionFlag::FormatSpecific(val) => {
                assert!(val.is_power_of_two());
                Self((val as u64) << 32)
            }
        }
    }
}

impl BitOr for SectionFlags {
    type Output = Self;
    #[inline]
    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl BitOr<SectionFlag> for SectionFlags {
    type Output = Self;
    #[inline]
    fn bitor(self, rhs: SectionFlag) -> Self::Output {
        self | Self::from(rhs)
    }
}

impl BitOr<SectionFlags> for SectionFlag {
    type Output = SectionFlags;
    #[inline]
    fn bitor(self, rhs: SectionFlags) -> Self::Output {
        SectionFlags::from(self) | rhs
    }
}

impl BitOr for SectionFlag {
    type Output = SectionFlags;
    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        SectionFlags::from(self) | SectionFlags::from(rhs)
    }
}

pub struct SectionFlagsIter(u64);

impl Iterator for SectionFlagsIter {
    type Item = SectionFlag;

    fn next(&mut self) -> Option<SectionFlag> {
        let pos = self.0.trailing_zeros();
        if pos == 64 {
            return None;
        } else {
            self.0 &= !(1 << pos);
            Some(match pos {
                0 => SectionFlag::Writable,
                1 => SectionFlag::Alloc,
                2 => SectionFlag::Executable,
                val => SectionFlag::FormatSpecific(1 << (val - 32)),
            })
        }
    }
}

impl IntoIterator for SectionFlags {
    type IntoIter = SectionFlagsIter;
    type Item = SectionFlag;
    fn into_iter(self) -> Self::IntoIter {
        SectionFlagsIter(self.0)
    }
}

#[derive(Clone, Debug, Default)]
pub struct Section {
    pub name: String,
    pub align: usize,
    pub ty: SectionType,
    pub content: Vec<u8>,
    pub tail_size: usize,
    pub relocs: Vec<Reloc>,
    pub info: u64,
    pub link: u64,
    pub flags: Option<SectionFlags>,
    #[doc(hidden)]
    pub __private: (),
}

impl InsnWrite for Section {
    fn write_addr(&mut self, size: usize, addr: Address, rel: bool) -> std::io::Result<()> {
        match (addr, rel) {
            (Address::Abs(_), true) => todo!(),
            (Address::Abs(val), false) => {
                let bytes = val.to_le_bytes();
                self.content.extend_from_slice(&bytes[..(size / 8)]);
                Ok(())
            }
            (Address::Disp(disp), true) => {
                let bytes = disp.to_le_bytes();
                self.content.extend_from_slice(&bytes[..(size / 8)]);
                Ok(())
            }
            (Address::Disp(_), false) => todo!(),
            (Address::Symbol { name, disp }, true) => {
                let bytes = disp.to_le_bytes();
                let code = RelocCode::Rel { addr_width: size };
                let offset = self.content.len() as u64;
                self.content.extend_from_slice(&bytes[..(size / 8)]);
                self.relocs.push(Reloc {
                    code,
                    symbol: name,
                    addend: Some(disp - ((size / 8) as i64)),
                    offset,
                });
                Ok(())
            }
            (Address::Symbol { name, disp }, false) => {
                let bytes = disp.to_le_bytes();
                let code = RelocCode::Abs { addr_width: size };
                let offset = self.content.len() as u64;
                self.content.extend_from_slice(&bytes[..(size / 8)]);
                self.relocs.push(Reloc {
                    code,
                    symbol: name,
                    addend: Some(disp),
                    offset,
                });
                Ok(())
            }
            (Address::PltSym { name }, true) => {
                let bytes = 0u64.to_le_bytes();
                let code = RelocCode::RelPlt { addr_width: size };
                let offset = self.content.len() as u64;
                self.content.extend_from_slice(&bytes[..(size / 8)]);
                self.relocs.push(Reloc {
                    code,
                    symbol: name,
                    addend: Some(-((size / 8) as i64)),
                    offset,
                });
                Ok(())
            }
            (Address::PltSym { name: _ }, false) => todo!(),
        }
    }

    fn offset(&self) -> usize {
        self.content.len()
    }

    fn write_reloc(&mut self, reloc: Reloc) -> io::Result<()> {
        self.relocs.push(reloc);
        Ok(())
    }
    fn write_zeroes(&mut self, count: usize) -> std::io::Result<()> {
        self.tail_size += count;
        Ok(())
    }
}

impl Write for Section {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.tail_size == 0 {
            arch_ops::traits::default_write_zeroes(&mut self.content, self.tail_size)?;
            self.tail_size = 0;
        }
        self.content.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.content.flush()
    }
}

#[derive(Copy, Clone, Debug, Hash)]
#[non_exhaustive]
pub enum GroupType {
    Normal,
    LinkOnce,
    FormatSpecific(u32),
}

impl Default for GroupType {
    fn default() -> Self {
        GroupType::Normal
    }
}

#[derive(Clone, Debug, Hash, Default)]
pub struct SectionGroup {
    pub name: String,
    pub sections: Vec<u32>,
    pub group_type: GroupType,
    pub id_sym: String,
    #[doc(hidden)]
    pub __private: (),
}

#[cfg(test)]
mod tests {
    use crate::traits::ReadSeek;

    use super::{Binfmt, FileType};

    pub struct TestBinfmt;

    impl super::Binfmt for TestBinfmt {
        fn relnum_to_howto(&self, _: u32) -> Option<&dyn crate::howto::HowTo> {
            None
        }

        fn code_to_howto(&self, _: crate::howto::RelocCode) -> Option<&dyn crate::howto::HowTo> {
            None
        }

        fn name(&self) -> &'static str {
            "test"
        }

        fn create_file(&self, ty: super::FileType) -> super::BinaryFile {
            super::BinaryFile::create(self, Box::new(()), ty)
        }

        fn read_file(
            &self,
            _: &mut (dyn ReadSeek + '_),
        ) -> std::io::Result<Option<super::BinaryFile>> {
            Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "Can't Read/write Test Binfmts",
            ))
        }

        fn write_file(
            &self,
            _: &mut (dyn std::io::Write + '_),
            _: &super::BinaryFile,
        ) -> std::io::Result<()> {
            Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "Can't Read/write Test Binfmts",
            ))
        }

        fn has_sections(&self) -> bool {
            true
        }

        fn ident_file(&self, _: &mut (dyn std::io::Read + '_)) -> std::io::Result<bool> {
            Ok(false)
        }
    }
    #[test]
    pub fn test_data_type() {
        let fmt = TestBinfmt.create_file(FileType::Exec);
        fmt.data().downcast_ref::<()>();
    }
}
