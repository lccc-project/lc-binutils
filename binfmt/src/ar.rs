use std::{
    convert::TryFrom,
    error::Error,
    ffi::{CString, OsStr, OsString},
    fmt::Display,
    io::{Cursor, ErrorKind, Read, Write},
    mem::size_of,
    slice,
    time::SystemTime,
};

pub const ARMAG: [u8; 8] = *b"!<arch>\n";
pub const FMAG: [u8; 2] = [0x60, 0x0A];
#[repr(C, align(1))]
#[derive(Copy, Clone, Debug)]
pub struct ArchiveHeader {
    pub ar_name: [u8; 16],
    pub ar_date: [u8; 12],
    pub ar_uid: [u8; 6],
    pub ar_gid: [u8; 6],
    pub ar_mode: [u8; 8],
    pub ar_size: [u8; 10],
    pub ar_fmag: [u8; 2],
}

#[derive(Debug)]
pub struct Archive {
    mag: [u8; 8],
    symtab: Option<ArchiveMember>,
    esymtab: Option<ArchiveMember>,
    strtab: Option<ArchiveMember>,
    members: Vec<ArchiveMember>,
}

#[derive(Debug)]
pub struct ArchiveMember {
    header: ArchiveHeader,
    long_name: Option<OsString>,
    bytes: Vec<u8>,
}

#[derive(Copy, Clone, Debug)]
pub struct ArchiveMetaOutOfRange<T>(T);

impl<T> ArchiveMetaOutOfRange<T> {
    pub fn into_inner(self) -> T {
        self.0
    }

    pub fn as_inner(&self) -> &T {
        &self.0
    }

    pub fn as_inner_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T: Display> Display for ArchiveMetaOutOfRange<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("value for field is out of range {}", &self.0))
    }
}

impl<T> Error for ArchiveMetaOutOfRange<T> where Self: std::fmt::Debug + Display {}

impl ArchiveMember {
    pub fn new() -> ArchiveMember {
        Self {
            header: ArchiveHeader {
                ar_name: [b' '; 16],
                ar_date: [b' '; 12],
                ar_uid: [b' '; 6],
                ar_gid: [b' '; 6],
                ar_mode: [b' '; 8],
                ar_size: [b' '; 10],
                ar_fmag: FMAG,
            },
            long_name: None,
            bytes: Vec::new(),
        }
    }

    pub fn read<R: Read>(mut r: R) -> std::io::Result<Self> {
        let mut bytes = [0u8; size_of::<ArchiveHeader>()];
        r.read_exact(&mut bytes)?;
        // SAFETY:
        // bytes is in lifetime
        // size is guaranteed above, and structure is read
        // ArchiveHeader is statically guaranteed to have alignment of at most 1
        let header: ArchiveHeader = unsafe { core::mem::transmute(bytes) };

        if header.ar_fmag != FMAG {
            return Err(std::io::Error::new(
                ErrorKind::InvalidData,
                "Invalid Archive Header",
            ));
        }

        let size = std::str::from_utf8(&header.ar_size)
            .map_err(|v| std::io::Error::new(ErrorKind::InvalidData, v))
            .and_then(|s| {
                s.trim()
                    .parse::<u64>()
                    .map_err(|e| std::io::Error::new(ErrorKind::InvalidData, e))
            })?;
        #[cfg(target_pointer_width = "32")]
        {
            if size > (usize::MAX as u64) {
                return Err(std::io::Error::new(
                    ErrorKind::InvalidData,
                    ArchiveMetaOutOfRange(size),
                ));
            }
        }

        let mut bytes = vec![0u8; size as usize];
        r.read_exact(&mut bytes)?;
        if size % 2 != 0 {
            r.read_exact(slice::from_mut(&mut 0u8))?;
        }
        Ok(Self {
            header,
            long_name: None,
            bytes,
        })
    }

    pub fn write<W: Write>(&self, mut w: W) -> std::io::Result<()> {
        let bytes: [u8; size_of::<ArchiveHeader>()] = unsafe { core::mem::transmute(self.header) };
        w.write_all(&bytes)?;
        w.write_all(&self.bytes)?;
        if self.bytes.len() % 2 != 0 {
            w.write_all(slice::from_ref(&10u8))?;
        }
        Ok(())
    }

    pub fn set_date(&mut self, date: SystemTime) -> Result<(), ArchiveMetaOutOfRange<SystemTime>> {
        let dur = date
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if dur > 999999999999 {
            Err(ArchiveMetaOutOfRange(date))
        } else {
            write!((&mut self.header.ar_date) as &mut [_], "{:<12}", dur).unwrap();
            Ok(())
        }
    }

    pub fn set_uid(&mut self, id: u32) -> Result<(), ArchiveMetaOutOfRange<u32>> {
        if id > 999999 {
            Err(ArchiveMetaOutOfRange(id))
        } else {
            write!((&mut self.header.ar_uid) as &mut [_], "{:<6}", id).unwrap();
            Ok(())
        }
    }

    pub fn set_gid(&mut self, id: u32) -> Result<(), ArchiveMetaOutOfRange<u32>> {
        if id > 999999 {
            Err(ArchiveMetaOutOfRange(id))
        } else {
            write!((&mut self.header.ar_gid) as &mut [_], "{:<6}", id).unwrap();
            Ok(())
        }
    }

    pub fn set_name(&mut self, st: &str) {
        if st.len() > 15 {
            self.long_name = Some(OsString::from(st));
            write!((&mut self.header.ar_name) as &mut [_], "/{:<15}", st.len()).unwrap();
        } else {
            self.long_name = None;
            let mut name = format!("{}/", st);
            name += &" ".repeat(16 - name.len());
            write!((&mut self.header.ar_name) as &mut [_], "{}", name).unwrap();
        }
    }

    pub fn get_name(&self) -> &OsStr {
        if let Some(o) = &self.long_name {
            o
        } else {
            let mut bytes: &[u8] = &self.header.ar_name;
            for i in 0..bytes.len() {
                if bytes[i] == b'/' {
                    bytes = &bytes[..i];
                    break;
                }
            }
            {
                #[cfg(unix)]
                {
                    use std::os::unix::ffi::OsStrExt as _;
                    OsStr::from_bytes(bytes)
                }
                #[cfg(not(unix))]
                {
                    OsStr::new(std::str::from_utf8(bytes).unwrap())
                }
            }
        }
    }

    pub fn get_header(&self) -> &ArchiveHeader {
        &self.header
    }

    pub fn content(&self) -> &[u8] {
        &self.bytes
    }

    pub fn truncate(&mut self) {
        self.bytes.clear();
        write!((&mut self.header.ar_size) as &mut [u8], "{:<10}", 0).unwrap();
    }
}

impl Default for ArchiveMember {
    fn default() -> Self {
        Self::new()
    }
}

impl Write for ArchiveMember {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let prev_size = self.bytes.len();
        let size = self.bytes.write(buf)?;
        let len = self.bytes.len() as u64;
        if len > 9999999999 {
            self.bytes.resize_with(9999999999, || unreachable!());
            write!((&mut self.header.ar_size) as &mut [u8], "9999999999").unwrap();
            Ok(9999999999usize.saturating_sub(prev_size))
        } else {
            write!((&mut self.header.ar_size) as &mut [u8], "{:<10}", len).unwrap();
            Ok(size)
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.bytes.flush()
    }
}

impl Archive {
    pub fn new() -> Self {
        Self {
            mag: ARMAG,
            members: Vec::new(),
            strtab: None,
            symtab: None,
            esymtab: None,
        }
    }

    pub fn get_extsymtable(&mut self) -> &mut ArchiveMember {
        if self.symtab.is_none() {
            panic!("Cannot Create extended symbol table without creating symbol table")
        } else {
            self.esymtab.get_or_insert_with(|| {
                let mut esymtab = ArchiveMember::new();
                esymtab.set_name("");
                esymtab
            })
        }
    }

    #[allow(clippy::unused_io_amount)] // FIXME: Ray wrote this code. Is this correct?
    pub fn ranlib(&mut self) -> &mut ArchiveMember {
        if self.symtab.is_none() {
            let mut symtab = ArchiveMember::new();
            symtab.set_name("");
            self.symtab = Some(symtab);
        }
        let mut sym_entries = Vec::new();

        for (i, m) in self.members.iter().enumerate() {
            let read = Cursor::new(&m.bytes);
            let file = crate::open_file(read).unwrap();
            for sym in file.symbols() {
                sym_entries.push((u32::try_from(i).unwrap(), sym.name().to_owned()));
            }
        }

        let mem = self.symtab.as_mut().unwrap();
        mem.truncate();

        let size = u32::try_from(sym_entries.len()).unwrap();
        let bytes = &mut mem.bytes;

        bytes.write_all(&size.to_be_bytes()).unwrap();

        for (file, sym) in sym_entries {
            let sym = CString::new(sym).unwrap();
            bytes.write_all(&file.to_be_bytes()).unwrap();
            bytes.write_all(sym.as_bytes()).unwrap();
        }

        mem.write(&[]).unwrap(); // Because we appended bytes in a bit of a hacky way, use this to trigger correct size setting

        mem
    }

    pub fn collect_names(&mut self) {
        let names = self
            .members
            .iter()
            .enumerate()
            .filter_map(|(i, f)| f.long_name.clone().map(|x| (i, x)))
            .collect::<Vec<_>>();

        if !names.is_empty() {
            if self.strtab.is_none() {
                let mut strtab = ArchiveMember::new();
                strtab.set_name("/");
                self.strtab = Some(strtab);
            }
            let strtab = self.strtab.as_mut().unwrap();
            strtab.truncate();
            for (d, n) in names.into_iter() {
                let str = n.into_string().unwrap();
                let idx = strtab.bytes.len();
                write!(strtab, "{}\0", str).unwrap();
                let item = &mut self.members[d].header;
                write!((&mut item.ar_name) as &mut [_], "/{:>15}", idx).unwrap();
            }
        }
    }

    pub fn write<W: Write>(&mut self, mut w: W) -> std::io::Result<()> {
        self.collect_names();
        w.write_all(&self.mag)?;
        if let Some(symtab) = &self.symtab {
            symtab.write(&mut w)?;
        }
        if let Some(esymtab) = &self.esymtab {
            esymtab.write(&mut w)?;
        }
        if let Some(strtab) = &self.strtab {
            strtab.write(&mut w)?;
        }

        for member in &self.members {
            member.write(&mut w)?;
        }
        Ok(())
    }

    pub fn read<R: Read>(mut r: R) -> std::io::Result<Self> {
        let mut mag = [0u8; 8];
        r.read_exact(&mut mag)?;
        if mag != ARMAG {
            return Err(std::io::Error::new(
                ErrorKind::InvalidData,
                "Invalid Archive",
            ));
        }
        let mut members = Vec::new();
        let mut symtab = None;
        let mut esymtab = None;
        let mut strtab = None;
        loop {
            match ArchiveMember::read(&mut r) {
                Ok(mut m) => {
                    if m.header.ar_name == "".as_bytes() {
                        if symtab.is_none() {
                            symtab = Some(m);
                        } else if esymtab.is_none() {
                            esymtab = Some(m);
                        } else {
                            return Err(std::io::Error::new(
                                ErrorKind::InvalidData,
                                "Invalid Archive Table (multiple symbol tables present in file)",
                            ));
                        }
                        continue;
                    } else if m.header.ar_name == "".as_bytes() {
                        if strtab.is_none() {
                            strtab = Some(m);
                        } else {
                            return Err(std::io::Error::new(
                                ErrorKind::InvalidData,
                                "Invalid Archive Table (multiple string tables present in file)",
                            ));
                        }
                        continue;
                    } else if m.header.ar_name[0] == b'/' {
                        let name = std::str::from_utf8(&m.header.ar_name)
                            .map_err(|e| std::io::Error::new(ErrorKind::InvalidData, e))?[1usize..]
                            .parse::<usize>()
                            .map_err(|e| std::io::Error::new(ErrorKind::InvalidData, e))?;
                        if let Some(s) = &strtab {
                            let cname = {
                                let mut bytes = &s.bytes[name..];
                                for i in 0..(bytes.len() - 1) {
                                    if &bytes[i..(i + 2)] == b"/\n" {
                                        bytes = &bytes[..i];
                                        break;
                                    }
                                }
                                #[cfg(unix)]
                                {
                                    use std::os::unix::ffi::OsStrExt as _;
                                    OsStr::from_bytes(bytes)
                                }
                                #[cfg(not(unix))]
                                {
                                    OsStr::new(std::str::from_utf8(bytes).unwrap())
                                }
                            };
                            m.long_name = Some(cname.to_os_string());
                        } else {
                            return Err(std::io::Error::new(
                                ErrorKind::InvalidData,
                                "Invalid Archive Table",
                            ));
                        }
                    }
                    members.push(m);
                }
                Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e),
            }
        }

        Ok(Self {
            mag,
            symtab,
            esymtab,
            strtab,
            members,
        })
    }

    pub fn new_member(&mut self) -> &mut ArchiveMember {
        let mut member = ArchiveMember::new();
        member.set_name("");
        self.members.push(member);
        self.members.last_mut().unwrap()
    }

    pub fn members(&self) -> &[ArchiveMember] {
        &self.members
    }

    pub fn members_mut(&mut self) -> &mut [ArchiveMember] {
        &mut self.members
    }
}

impl Default for Archive {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
pub mod tests;
