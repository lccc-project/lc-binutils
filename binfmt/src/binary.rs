use crate::fmt::{BinaryFile, Binfmt, CallbackError, FileType, Section, SectionType};

pub struct Binary;

pub fn create_format() -> Binary {
    Binary
}

impl Binfmt for Binary {
    fn relnum_to_howto(&self, _relnum: u32) -> Option<&dyn crate::howto::HowTo> {
        None
    }

    fn code_to_howto(&self, _code: crate::howto::RelocCode) -> Option<&dyn crate::howto::HowTo> {
        None
    }

    fn name(&self) -> &'static str {
        "binary"
    }

    fn create_file(&self, ty: FileType) -> crate::fmt::BinaryFile {
        BinaryFile::create(self, Box::new(()), ty)
    }

    fn read_file(
        &self,
        file: &mut (dyn std::io::Read + '_),
    ) -> std::io::Result<Option<crate::fmt::BinaryFile>> {
        let mut vec = Vec::new();
        file.read_to_end(&mut vec)?;
        let mut file = BinaryFile::create(self, Box::new(()), FileType::Exec);
        let _ = file.add_section(Section {
            align: 1,
            content: vec,
            name: ".data".to_string(),
            ty: SectionType::ProgBits,
            __private: (),
        });

        Ok(Some(file))
    }

    fn write_file(
        &self,
        file: &mut (dyn std::io::Write + '_),
        bfile: &crate::fmt::BinaryFile,
    ) -> std::io::Result<()> {
        for s in bfile.sections() {
            file.write_all(&s.content)?;
        }
        Ok(())
    }

    fn has_sections(&self) -> bool {
        true
    }

    fn create_section(
        &self,
        _section: &mut crate::fmt::Section,
    ) -> Result<(), crate::fmt::CallbackError> {
        Ok(())
    }

    fn create_symbol(
        &self,
        _sym: &mut crate::sym::Symbol,
    ) -> Result<(), crate::fmt::CallbackError> {
        Err(CallbackError::NotAccepted)
    }

    fn create_reloc(
        &self,
        _reloc: &mut crate::howto::Reloc,
    ) -> Result<(), crate::fmt::CallbackError> {
        Err(CallbackError::NotAccepted)
    }

    fn before_relocate(&self, _reloc: &mut crate::howto::Reloc, _symbol: &crate::sym::Symbol) {}
}
