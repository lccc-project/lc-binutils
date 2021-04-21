use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Debug, Zeroable, Pod)]
pub struct CoffHeader {
    mach: u16,
    nsect: u16,
    timestamp: u32,
    symtab_off: u32,
    n_symtab: u32,
    opthead_size: u16,
    characteristics: u16,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Zeroable, Pod)]
pub struct CoffSectionHeader {
    name: [u8; 8],
    vsize: u32,
    vaddr: u32,
    fsize: u32,
    foff: u32,
    relocoff: u32,
    lnoff: u32,
    nreloc: u16,
    nlines: u16,
    characteristics: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Zeroable, Pod)]
pub struct CoffOptionalHeader {
    magic: u16,
    link_maj: u8,
    lin_min: u8,
    code_size: u32,
    init_size: u32,
    uninit_size: u32,
    entry_addr: u32,
    code_base: u32,
}
