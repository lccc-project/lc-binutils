use std::{
    io::{Read, Write},
    num::NonZeroU32,
};

pub trait Symbol {
    fn address(&self) -> Option<u64>;
    fn name(&self) -> &str;
    fn section(&self) -> Option<NonZeroU32>;
    fn flags(&self) -> u64;
    fn size(&self) -> u64;
    fn alignment(&self) -> u64;
    fn set_name(&mut self, st: String);
    fn set_address(&mut self, addr: u64);
    fn set_size(&mut self, size: u64);
    fn set_alignment(&mut self, align: u64);
}

pub trait Section {
    fn relocation_address(&self) -> Option<u64>;
    fn load_address(&self) -> Option<u64>;
    fn name(&self) -> &str;
    fn size(&self) -> u64;
    fn allocated_size(&self) -> u64;
    fn read_section(&self) -> Box<dyn Read + '_>;
    fn write_section(&mut self) -> Box<dyn Write + '_>;
    fn set_name(&mut self, st: String);
    fn set_alignment(&mut self, align: u64);
    fn set_relocation_address(&self, addr: u64);
    fn set_load_address(&self, addr: u64);
}

pub trait Segment {
    fn load_address(&self) -> u64;
    fn set_load_address(&mut self, addr: u64);
    fn segment_type(&self) -> u64;
    fn set_segment_type(&mut self, addr: u64);
    fn flags(&self) -> u64;
    fn set_flags(&mut self, flags: u64);
    fn name(&self) -> &str;
    fn set_name(&mut self, name: String);
    fn read_segment(&self) -> Box<dyn Read + '_>;
    fn write_segment(&self) -> Box<dyn Write + '_>;
}

pub trait Relocation {
    fn symbol(&self) -> u32;
    fn relocation_type(&self) -> u32;
    fn addend(&self) -> Option<u32>;
    fn set_symbol(&mut self, symbol: u32);
    fn set_addend(&mut self, addend: u32);
}

pub trait Sections {
    fn iter(&self) -> Box<dyn Iterator<Item = &'_ dyn Section> + '_>;
    fn iter_mut(&mut self) -> Box<dyn Iterator<Item = &'_ mut dyn Section> + '_>;
    fn insert_section(&mut self) -> &mut dyn Section;
}

pub trait BinaryFormat {
    fn read<R: Read>(r: R) -> std::io::Result<Self>
    where
        Self: Sized;
    fn write(&self, w: &mut dyn Write) -> std::io::Result<()>;
    fn sections(&self) -> &dyn Sections;
}
