use crate::traits::{BinaryFile, Segment};

pub struct RawBinaryFile {
    bytes: Vec<u8>,
}

impl Segment for RawBinaryFile {
    fn read(&self) -> Box<dyn std::io::Read + '_> {
        Box::new(self.bytes.as_slice())
    }

    fn write(&mut self) -> Box<dyn std::io::Write + '_> {
        Box::new(&mut self.bytes)
    }

    fn flags(&self) -> u64 {
        0
    }

    fn set_flags(&mut self, _: u64) {}

    fn segment_type(&self) -> u64 {
        0
    }

    fn set_segment_type(&mut self, pt: u64) -> Result<(), u64> {
        Err(pt)
    }

    fn align(&self) -> u64 {
        0
    }

    fn set_alignment(&mut self, _: u64) {}
}

impl BinaryFile for RawBinaryFile {
    fn read(read: &mut (dyn std::io::Read + '_)) -> std::io::Result<Box<Self>>
    where
        Self: Sized,
    {
        let mut bytes = Vec::new();
        read.read_to_end(&mut bytes)?;

        Ok(Box::new(Self { bytes }))
    }

    fn write(&self, write: &mut (dyn std::io::Write + '_)) -> std::io::Result<()> {
        write.write_all(&self.bytes)
    }

    fn is_relocatable(&self) -> bool {
        false
    }

    fn has_symbols(&self) -> bool {
        false
    }

    fn has_sections(&self) -> bool {
        false
    }

    fn section(&self, _: &str) -> Option<&(dyn crate::traits::Section + '_)> {
        None
    }

    fn segments(&self) -> Vec<&(dyn crate::traits::Segment + '_)> {
        vec![self]
    }

    fn section_mut(&mut self, _: &str) -> Option<&(dyn crate::traits::Segment + '_)> {
        None
    }

    fn segments_mut(&mut self) -> Vec<&mut (dyn crate::traits::Segment + '_)> {
        vec![self]
    }

    fn create_segment(&mut self) -> Option<&mut (dyn crate::traits::Segment + '_)> {
        None
    }

    fn insert_segment(&mut self, _idx: u32) -> Option<&mut (dyn crate::traits::Segment + '_)> {
        None
    }
}
