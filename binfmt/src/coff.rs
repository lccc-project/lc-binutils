use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Debug, Zeroable, Pod)]
pub struct CoffHeader {
    pub mach: u16,
    pub nsect: u16,
    pub timestamp: u32,
    pub symtab_off: u32,
    pub n_symtab: u32,
    pub opthead_size: u16,
    pub characteristics: u16,
}

bitflags::bitflags! {
    #[derive(Zeroable, Pod, Copy, Clone)]
    #[repr(transparent)]
    pub struct CoffCharacteristics : u16{
        const IMAGE_FILE_RELOCS_STRIPPED = 0x0001;

    }
}

pub mod consts {}

#[repr(C)]
#[derive(Copy, Clone, Debug, Zeroable, Pod)]
pub struct CoffSectionHeader {
    pub name: [u8; 8],
    pub vsize: u32,
    pub vaddr: u32,
    pub fsize: u32,
    pub foff: u32,
    pub relocoff: u32,
    pub lnoff: u32,
    pub nreloc: u16,
    pub nlines: u16,
    pub characteristics: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Zeroable, Pod)]
pub struct CoffOptionalHeader {
    pub magic: u16,
    pub link_maj: u8,
    pub lin_min: u8,
    pub code_size: u32,
    pub init_size: u32,
    pub uninit_size: u32,
    pub entry_addr: u32,
    pub code_base: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Zeroable, Pod)]
pub struct CoffOptionalHeader32 {
    pub base: CoffOptionalHeader,
    pub data_base: u32,
}
