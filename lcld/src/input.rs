use binfmt::fmt::Binfmt;

use std::io::{Read, Seek, SeekFrom};

use std::fs::File;
use std::path::Path;

use crate::lto::LtoProvider;

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum InputFileType {
    Object(&'static dyn Binfmt),
    Archive,
    LtoInput(&'static dyn LtoProvider),
    LinkerScript,
}

impl core::fmt::Display for InputFileType {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::Object(fmt) => f.write_str(fmt.name()),
            Self::Archive => f.write_str("archive"),
            Self::LtoInput(prov) => {
                f.write_str("lto ")?;
                f.write_str(prov.name())
            }
            Self::LinkerScript => f.write_str("script"),
        }
    }
}

#[allow(clippy::unused_io_amount)]
pub fn ident_input(p: &Path) -> std::io::Result<InputFileType> {
    let file = File::open(p)?;

    if let Some(binfmt) = binfmt::identify_file(&file)? {
        Ok(InputFileType::Object(binfmt))
    } else {
        let mut arch_buf = [0u8; 8];
        (&file).seek(SeekFrom::Start(0))?;
        (&file).read(&mut arch_buf)?; // If this doesn't complete, then `arch_buf` will not be filled with the correct value - it is not an error to be given an 6-byte file
        (&file).seek(SeekFrom::Start(0))?;
        if arch_buf == *b"!<arch>\n" {
            Ok(InputFileType::Archive)
        } else {
            // todo: Identify Lto Input objects
            Ok(InputFileType::LinkerScript)
        }
    }
}
