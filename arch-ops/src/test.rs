use std::io::Write;

use crate::traits::{Address, InsnWrite, Reloc};

pub struct TestWriter {
    pub inner: Vec<u8>,
}

impl Write for TestWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

impl InsnWrite for TestWriter {
    fn write_addr(&mut self, size: usize, addr: Address, rel: bool) -> std::io::Result<()> {
        match (addr, rel) {
            (Address::Disp(disp), true) => self.write_all(&disp.to_le_bytes()[..(size / 8)]),
            (Address::Abs(addr), false) => self.write_all(&addr.to_le_bytes()[..(size / 8)]),
            (_, _) => panic!(),
        }
    }

    fn offset(&self) -> usize {
        self.inner.len()
    }

    fn write_reloc(&mut self, _: Reloc) -> std::io::Result<()> {
        panic!("Cannot write Reloc to TestWriter")
    }
}

#[test]
fn test_sanity_writer_write() {
    let mut w = TestWriter { inner: Vec::new() };
    w.write_all(&[0u8]).unwrap();
    assert_eq!(&*w.inner, &[0u8]);
}

#[test]
fn test_sanity_writer_write_many() {
    let mut w = TestWriter { inner: Vec::new() };
    w.write_all(&[0u8, 1u8, 2u8, 3u8]).unwrap();
    assert_eq!(&*w.inner, &[0u8, 1u8, 2u8, 3u8]);
}

#[test]
fn test_sanity_writer_write_addr() {
    let mut w = TestWriter { inner: Vec::new() };
    w.write_addr(32, Address::Disp(1), true).unwrap();
    assert_eq!(&*w.inner, &[1u8, 0u8, 0u8, 0u8]);
}

#[test]
fn test_sanity_writer_write_addr_m1() {
    let mut w = TestWriter { inner: Vec::new() };
    w.write_addr(32, Address::Disp(-1), true).unwrap();
    assert_eq!(&*w.inner, &[0xffu8, 0xffu8, 0xffu8, 0xffu8]);
}

#[test]
fn test_sanity_writer_write_addr_abs() {
    let mut w = TestWriter { inner: Vec::new() };
    w.write_addr(32, Address::Abs(32767), false).unwrap();
    assert_eq!(&*w.inner, &[0xffu8, 0x7fu8, 0u8, 0u8]);
}
