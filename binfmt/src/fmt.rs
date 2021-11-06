use std::{
    any::Any,
    collections::{hash_map::Values, HashMap},
    io::{self, Read, Write},
    slice::{Iter, IterMut},
};

use crate::{
    howto::{HowTo, Reloc, RelocCode},
    sym::{Symbol, SymbolKind, SymbolType},
};

pub enum CallbackError {
    InvalidType,
    NotAccepted,
}

pub trait Binfmt {
    fn relnum_to_howto(&self, relnum: u32) -> Option<&dyn HowTo>;
    fn code_to_howto(&self, code: RelocCode) -> Option<&dyn HowTo>;

    fn name(&self) -> &'static str;
    fn create_file(&self, ty: FileType) -> BinaryFile;
    fn read_file(&self, file: &mut (dyn Read + '_)) -> io::Result<Option<BinaryFile>>;
    fn write_file(&self, file: &mut (dyn Write + '_), bfile: &BinaryFile) -> io::Result<()>;

    fn has_sections(&self) -> bool;

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
            fmt,
            data,
            ty,
        }
    }

    pub fn file_type(&self) -> &FileType {
        &self.ty
    }

    pub fn data(&self) -> &dyn Any {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut dyn Any {
        &mut self.data
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

    pub fn sections(&self) -> Sections<'_> {
        Sections(self.sections.as_ref().map(|x| x.iter()))
    }

    pub fn sections_mut(&mut self) -> SectionsMut<'_> {
        SectionsMut(self.sections.as_mut().map(|x| x.iter_mut()))
    }

    pub fn remove_section(&mut self, x: u32) -> Option<Section> {
        self.sections
            .as_mut()
            .filter(|v| (x as usize) < v.len())
            .map(|v| v.remove(x as usize))
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
            let mut sym = Symbol::new(
                name.to_string(),
                None,
                None,
                SymbolType::Null,
                SymbolKind::Local,
            );
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

pub enum SectionType {
    NoBits,
    ProgBits,
    SymbolTable,
    StringTable,
    Dynamic,
    ProcedureLinkageTable,
    GlobalOffsetTable,
    FormatSpecific(u32),
}

pub struct Section {
    pub name: String,
    pub align: usize,
    pub ty: SectionType,
    pub content: Vec<u8>,
}