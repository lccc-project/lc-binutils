#![allow(dead_code)] // fixme later

use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Zeroable, Pod, Clone, Copy)]
pub struct O65FixedHeader {
    id1: [u8; 2],
    magic: [u8; 3],
    ver: u8,
    mode: u16,
}

#[repr(C)]
#[derive(Zeroable, Pod, Clone, Copy)]
pub struct O65Header16 {
    fixed: O65FixedHeader,
    tbase: u16,
    tsize: u16,
    dbase: u16,
    dsize: u16,
    bbase: u16,
    bsize: u16,
    zbase: u16,
    zsize: u16,
    stack: u16,
}

#[repr(C)]
#[derive(Zeroable, Pod, Clone, Copy)]
pub struct O65Header32 {
    fixed: O65FixedHeader,
    tbase: u32,
    tsize: u32,
    dbase: u32,
    dsize: u32,
    bbase: u32,
    bsize: u32,
    zbase: u32,
    zsize: u32,
    stack: u32,
}
