#[derive(Clone, Hash, PartialEq)]
pub struct Symbol {
    name: String,
    value: Option<u128>,
    secno: Option<u32>,
    symtype: SymbolType,
    kind: SymbolKind,
}

impl Symbol {
    pub fn new(
        name: String,
        section: Option<u32>,
        value: Option<u128>,
        symtype: SymbolType,
        kind: SymbolKind,
    ) -> Self {
        Self {
            name,
            value,
            secno: section,
            symtype,
            kind,
        }
    }

    pub fn section(&self) -> Option<u32> {
        self.secno
    }

    pub fn section_mut(&mut self) -> &mut Option<u32> {
        &mut self.secno
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn value(&self) -> Option<u128> {
        self.value
    }

    pub fn value_mut(&mut self) -> &mut Option<u128> {
        &mut self.value
    }

    pub fn symbol_type(&self) -> SymbolType {
        self.symtype
    }

    pub fn kind(&self) -> SymbolKind {
        self.kind
    }

    pub fn kind_mut(&mut self) -> &mut SymbolKind {
        &mut self.kind
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum SymbolType {
    Null,
    Function,
    Object,
    File,
    Section,
    Common,
    Tls,
    FormatSpecific(u32),
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum SymbolKind {
    Local,
    Global,
    Weak,
    FormatSpecific(u32),
}
