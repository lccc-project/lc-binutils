use std::borrow::Cow;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::Display;
use std::io::{ErrorKind, SeekFrom};
use std::marker::PhantomData;
use std::mem::size_of;

use arch_ops::disasm::OpcodePrinter;
use bytemuck::{Pod, Zeroable};

use crate::debug::PrintHex;
use crate::fmt::{BinaryFile, Binfmt, FileType, Section, SectionType};
use crate::howto::HowTo;
use crate::sym::{SymbolKind, SymbolType};
use crate::traits::private::Sealed;
use crate::traits::{Numeric, ReadSeek};

pub type ElfByte<E> = <E as ElfClass>::Byte;
pub type ElfHalf<E> = <E as ElfClass>::Half;
pub type ElfWord<E> = <E as ElfClass>::Word;
pub type ElfSword<E> = <E as ElfClass>::Sword;
pub type ElfXword<E> = <E as ElfClass>::Xword;
pub type ElfSxword<E> = <E as ElfClass>::Sxword;
pub type ElfAddr<E> = <E as ElfClass>::Addr;
pub type ElfOffset<E> = <E as ElfClass>::Offset;
pub type ElfSection<E> = <E as ElfClass>::Section;
pub type ElfVersym<E> = <E as ElfClass>::Versym;
pub type Symbol<E> = <E as ElfClass>::Symbol;
pub type ElfSize<E> = <E as ElfClass>::Size;

pub trait ElfSymbol: Sealed {
    type Class: ElfClass;
    fn name_idx(&self) -> ElfWord<Self::Class>;
    fn value(&self) -> ElfAddr<Self::Class>;
    fn size(&self) -> ElfSize<Self::Class>;
    fn info(&self) -> ElfByte<Self::Class>;
    fn other(&self) -> ElfByte<Self::Class>;
    fn section(&self) -> ElfSection<Self::Class>;
}

pub trait ElfRelocation: Sealed {
    type Class: ElfClass;
    fn at_offset(&self) -> ElfAddr<Self::Class>;
    fn rel_type(&self) -> ElfSize<Self::Class>;
    fn symbol(&self) -> ElfSize<Self::Class>;
    fn addend(&self) -> ElfOffset<Self::Class> {
        Numeric::zero()
    }
}

pub trait ElfProgramHeader: Sealed {
    type Class: ElfClass;
    fn pt_type(&self) -> consts::ProgramType;
    fn offset(&self) -> ElfOffset<Self::Class>;
    fn vaddr(&self) -> ElfAddr<Self::Class>;
    fn paddr(&self) -> ElfAddr<Self::Class>;
    fn memsize(&self) -> ElfSize<Self::Class>;
    fn filesize(&self) -> ElfSize<Self::Class>;
    fn align(&self) -> ElfSize<Self::Class>;
    fn flags(&self) -> ElfWord<Self::Class>;
}

pub trait ElfClass: Sealed + Sized + Copy + core::fmt::Debug + 'static {
    type Byte: Numeric;
    const EI_CLASS: consts::EiClass;
    type Half: Numeric;
    type Word: Numeric;
    type Sword: Numeric;
    type Xword: Numeric;
    type Sxword: Numeric;
    type Addr: Numeric;
    type Offset: Numeric;
    type Section: Numeric;
    type Versym: Numeric;
    type Size: Numeric;
    type Symbol: ElfSymbol<Class = Self> + Pod;
    type Rel: ElfRelocation<Class = Self> + Pod;
    type Rela: ElfRelocation<Class = Self> + Pod;
    type ProgramHeader: ElfProgramHeader<Class = Self> + Pod;

    fn new_sym(
        st_name: Self::Word,
        st_value: Self::Addr,
        st_size: Self::Size,
        st_info: u8,
        st_other: u8,
        st_shndx: Self::Half,
    ) -> Self::Symbol;

    fn mk_rinfo(symno: usize, relcode: usize) -> Self::Size;
}

#[derive(Copy, Clone, Debug)]
pub enum Elf64 {}

#[derive(Copy, Clone, Debug)]
pub enum Elf32 {}

#[repr(C)]
#[derive(Copy, Clone, Debug, Zeroable, Pod)]
pub struct Elf32Sym {
    st_name: ElfWord<Elf32>,
    st_value: ElfAddr<Elf32>,
    st_size: ElfSize<Elf32>,
    st_info: ElfByte<Elf32>,
    st_other: ElfByte<Elf32>,
    st_shndx: ElfSection<Elf32>,
}

impl Sealed for Elf32Sym {}
impl ElfSymbol for Elf32Sym {
    type Class = Elf32;

    fn name_idx(&self) -> u32 {
        self.st_name
    }

    fn value(&self) -> <Self::Class as ElfClass>::Addr {
        self.st_value
    }

    fn size(&self) -> ElfSize<Self::Class> {
        self.st_size
    }

    fn info(&self) -> u8 {
        self.st_info
    }

    fn other(&self) -> u8 {
        self.st_other
    }

    fn section(&self) -> u16 {
        self.st_shndx
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Zeroable, Pod)]
pub struct Elf64Sym {
    st_name: ElfWord<Elf64>,
    st_info: ElfByte<Elf64>,
    st_other: ElfByte<Elf64>,
    st_shndx: ElfSection<Elf64>,
    st_value: ElfAddr<Elf64>,
    st_size: ElfSize<Elf64>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ElfRel<Class: ElfClass> {
    r_offset: ElfAddr<Class>,
    r_info: ElfSize<Class>,
}

unsafe impl<Class: ElfClass> Zeroable for ElfRel<Class> {}
unsafe impl<Class: ElfClass> Pod for ElfRel<Class> {}

mod private {
    use super::*;
    pub trait ElfRelocationExtractHelpers: ElfClass {
        fn symbol(info: ElfSize<Self>) -> ElfSize<Self>;
        fn rel_type(info: ElfSize<Self>) -> ElfSize<Self>;
    }
}

use private::*;

use self::consts::ElfIdent;

impl<Class: ElfClass + ElfRelocationExtractHelpers> Sealed for ElfRel<Class> {}

impl<Class: ElfClass + ElfRelocationExtractHelpers> ElfRelocation for ElfRel<Class> {
    type Class = Class;

    fn at_offset(&self) -> <Self::Class as ElfClass>::Addr {
        self.r_offset
    }

    fn rel_type(&self) -> <Self::Class as ElfClass>::Size {
        Class::symbol(self.r_info)
    }

    fn symbol(&self) -> <Self::Class as ElfClass>::Size {
        Class::rel_type(self.r_info)
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ElfRela<Class: ElfClass> {
    r_offset: ElfAddr<Class>,
    r_info: ElfSize<Class>,
    r_addend: ElfOffset<Class>,
}

unsafe impl<Class: ElfClass> Zeroable for ElfRela<Class> {}
unsafe impl<Class: ElfClass> Pod for ElfRela<Class> {}

impl<Class: ElfClass + ElfRelocationExtractHelpers> Sealed for ElfRela<Class> {}

impl<Class: ElfClass + ElfRelocationExtractHelpers> ElfRelocation for ElfRela<Class> {
    type Class = Class;

    fn at_offset(&self) -> <Self::Class as ElfClass>::Addr {
        self.r_offset
    }

    fn rel_type(&self) -> <Self::Class as ElfClass>::Size {
        Class::symbol(self.r_info)
    }

    fn symbol(&self) -> <Self::Class as ElfClass>::Size {
        Class::rel_type(self.r_info)
    }
    fn addend(&self) -> <Self::Class as ElfClass>::Offset {
        self.r_addend
    }
}

impl Sealed for Elf64Sym {}
impl ElfSymbol for Elf64Sym {
    type Class = Elf64;

    fn name_idx(&self) -> u32 {
        self.st_name
    }

    fn value(&self) -> <Self::Class as ElfClass>::Addr {
        self.st_value
    }

    fn size(&self) -> ElfSize<Self::Class> {
        self.st_size
    }

    fn info(&self) -> u8 {
        self.st_info
    }

    fn other(&self) -> u8 {
        self.st_other
    }

    fn section(&self) -> u16 {
        self.st_shndx
    }
}

impl Sealed for Elf64 {}
impl ElfClass for Elf64 {
    const EI_CLASS: consts::EiClass = consts::ELFCLASS64;
    type Addr = u64;
    type Offset = i64;
    type Size = u64;
    type Symbol = Elf64Sym;
    type Rel = ElfRel<Self>;
    type Rela = ElfRela<Self>;

    type Byte = u8;

    type Half = u16;

    type Word = u32;

    type Sword = i32;

    type Xword = u64;

    type Sxword = u64;

    type Section = u16;

    type Versym = u16;

    type ProgramHeader = Elf64Phdr;

    fn new_sym(
        st_name: Self::Word,
        st_value: Self::Addr,
        st_size: Self::Size,
        st_info: u8,
        st_other: u8,
        st_shndx: Self::Half,
    ) -> Self::Symbol {
        Elf64Sym {
            st_name,
            st_info,
            st_other,
            st_shndx,
            st_value,
            st_size,
        }
    }

    fn mk_rinfo(symno: usize, relcode: usize) -> Self::Size {
        ((symno as u64) << 32) + (relcode as u64)
    }
}

impl ElfRelocationExtractHelpers for Elf64 {
    fn symbol(info: Self::Size) -> Self::Size {
        info >> 32
    }

    fn rel_type(info: Self::Size) -> Self::Size {
        info & 0xffffffff
    }
}

impl Sealed for Elf32 {}
impl ElfClass for Elf32 {
    const EI_CLASS: consts::EiClass = consts::ELFCLASS32;
    type Addr = u32;
    type Offset = i32;
    type Size = u32;
    type Symbol = Elf32Sym;
    type Rel = ElfRel<Self>;
    type Rela = ElfRela<Self>;
    type Byte = u8;

    type Half = u16;

    type Word = u32;

    type Sword = i32;

    type Xword = u64;

    type Sxword = u64;

    type Section = u16;

    type Versym = u16;
    type ProgramHeader = Elf32Phdr;

    fn new_sym(
        st_name: Self::Word,
        st_value: Self::Addr,
        st_size: Self::Size,
        st_info: u8,
        st_other: u8,
        st_shndx: Self::Half,
    ) -> Self::Symbol {
        Elf32Sym {
            st_name,
            st_value,
            st_size,
            st_info,
            st_other,
            st_shndx,
        }
    }

    fn mk_rinfo(symno: usize, relcode: usize) -> Self::Size {
        eprintln!("symno");
        ((symno as u32) << 8) + (relcode as u32)
    }
}

impl ElfRelocationExtractHelpers for Elf32 {
    fn symbol(info: Self::Size) -> Self::Size {
        info >> 8
    }

    fn rel_type(info: Self::Size) -> Self::Size {
        info & 0xff
    }
}

pub mod consts {
    use bytemuck::{Pod, Zeroable};

    pub const ELFMAG: [u8; 4] = *b"\x7fELF";

    fake_enum::fake_enum! {
        #[repr(pub u16)]
        #[derive(Zeroable,Pod)]
        pub enum ElfType{
            ET_NONE = 0,
            ET_REL = 1,
            ET_EXEC = 2,
            ET_DYN = 3,
            ET_CORE = 4
        }
    }

    fake_enum::fake_enum! {
        #[repr(u16)]
        #[derive(Zeroable,Pod)]
        pub enum ElfMachine{
            EM_NONE = 0,           // No machine
            EM_M32 = 1,            // AT&T WE 32100
            EM_SPARC = 2,          // SPARC
            EM_386 = 3,            // Intel 386
            EM_68K = 4,            // Motorola 68000
            EM_88K = 5,            // Motorola 88000
            EM_IAMCU = 6,          // Intel MCU
            EM_860 = 7,            // Intel 80860
            EM_MIPS = 8,           // MIPS R3000
            EM_S370 = 9,           // IBM System/370
            EM_MIPS_RS3_LE = 10,   // MIPS RS3000 Little-endian
            EM_PARISC = 15,        // Hewlett-Packard PA-RISC
            EM_VPP500 = 17,        // Fujitsu VPP500
            EM_SPARC32PLUS = 18,   // Enhanced instruction set SPARC
            EM_960 = 19,           // Intel 80960
            EM_PPC = 20,           // PowerPC
            EM_PPC64 = 21,         // PowerPC64
            EM_S390 = 22,          // IBM System/390
            EM_SPU = 23,           // IBM SPU/SPC
            EM_V800 = 36,          // NEC V800
            EM_FR20 = 37,          // Fujitsu FR20
            EM_RH32 = 38,          // TRW RH-32
            EM_RCE = 39,           // Motorola RCE
            EM_ARM = 40,           // ARM
            EM_ALPHA = 41,         // DEC Alpha
            EM_SH = 42,            // Hitachi SH
            EM_SPARCV9 = 43,       // SPARC V9
            EM_TRICORE = 44,       // Siemens TriCore
            EM_ARC = 45,           // Argonaut RISC Core
            EM_H8_300 = 46,        // Hitachi H8/300
            EM_H8_300H = 47,       // Hitachi H8/300H
            EM_H8S = 48,           // Hitachi H8S
            EM_H8_500 = 49,        // Hitachi H8/500
            EM_IA_64 = 50,         // Intel IA-64 processor architecture
            EM_MIPS_X = 51,        // Stanford MIPS-X
            EM_COLDFIRE = 52,      // Motorola ColdFire
            EM_68HC12 = 53,        // Motorola M68HC12
            EM_MMA = 54,           // Fujitsu MMA Multimedia Accelerator
            EM_PCP = 55,           // Siemens PCP
            EM_NCPU = 56,          // Sony nCPU embedded RISC processor
            EM_NDR1 = 57,          // Denso NDR1 microprocessor
            EM_STARCORE = 58,      // Motorola Star*Core processor
            EM_ME16 = 59,          // Toyota ME16 processor
            EM_ST100 = 60,         // STMicroelectronics ST100 processor
            EM_TINYJ = 61,         // Advanced Logic Corp. TinyJ embedded processor family
            EM_X86_64 = 62,        // AMD x86-64 architecture
            EM_PDSP = 63,          // Sony DSP Processor
            EM_PDP10 = 64,         // Digital Equipment Corp. PDP-10
            EM_PDP11 = 65,         // Digital Equipment Corp. PDP-11
            EM_FX66 = 66,          // Siemens FX66 microcontroller
            EM_ST9PLUS = 67,       // STMicroelectronics ST9+ 8/16 bit microcontroller
            EM_ST7 = 68,           // STMicroelectronics ST7 8-bit microcontroller
            EM_68HC16 = 69,        // Motorola MC68HC16 Microcontroller
            EM_68HC11 = 70,        // Motorola MC68HC11 Microcontroller
            EM_68HC08 = 71,        // Motorola MC68HC08 Microcontroller
            EM_68HC05 = 72,        // Motorola MC68HC05 Microcontroller
            EM_SVX = 73,           // Silicon Graphics SVx
            EM_ST19 = 74,          // STMicroelectronics ST19 8-bit microcontroller
            EM_VAX = 75,           // Digital VAX
            EM_CRIS = 76,          // Axis Communications 32-bit embedded processor
            EM_JAVELIN = 77,       // Infineon Technologies 32-bit embedded processor
            EM_FIREPATH = 78,      // Element 14 64-bit DSP Processor
            EM_ZSP = 79,           // LSI Logic 16-bit DSP Processor
            EM_MMIX = 80,          // Donald Knuth's educational 64-bit processor
            EM_HUANY = 81,         // Harvard University machine-independent object files
            EM_PRISM = 82,         // SiTera Prism
            EM_AVR = 83,           // Atmel AVR 8-bit microcontroller
            EM_FR30 = 84,          // Fujitsu FR30
            EM_D10V = 85,          // Mitsubishi D10V
            EM_D30V = 86,          // Mitsubishi D30V
            EM_V850 = 87,          // NEC v850
            EM_M32R = 88,          // Mitsubishi M32R
            EM_MN10300 = 89,       // Matsushita MN10300
            EM_MN10200 = 90,       // Matsushita MN10200
            EM_PJ = 91,            // picoJava
            EM_OPENRISC = 92,      // OpenRISC 32-bit embedded processor
            EM_ARC_COMPACT = 93,   // ARC International ARCompact processor (old
                                    // spelling/synonym: EM_ARC_A5)
            EM_XTENSA = 94,        // Tensilica Xtensa Architecture
            EM_VIDEOCORE = 95,     // Alphamosaic VideoCore processor
            EM_TMM_GPP = 96,       // Thompson Multimedia General Purpose Processor
            EM_NS32K = 97,         // National Semiconductor 32000 series
            EM_TPC = 98,           // Tenor Network TPC processor
            EM_SNP1K = 99,         // Trebia SNP 1000 processor
            EM_ST200 = 100,        // STMicroelectronics (www.st.com) ST200
            EM_IP2K = 101,         // Ubicom IP2xxx microcontroller family
            EM_MAX = 102,          // MAX Processor
            EM_CR = 103,           // National Semiconductor CompactRISC microprocessor
            EM_F2MC16 = 104,       // Fujitsu F2MC16
            EM_MSP430 = 105,       // Texas Instruments embedded microcontroller msp430
            EM_BLACKFIN = 106,     // Analog Devices Blackfin (DSP) processor
            EM_SE_C33 = 107,       // S1C33 Family of Seiko Epson processors
            EM_SEP = 108,          // Sharp embedded microprocessor
            EM_ARCA = 109,         // Arca RISC Microprocessor
            EM_UNICORE = 110,      // Microprocessor series from PKU-Unity Ltd. and MPRC
                                    // of Peking University
            EM_EXCESS = 111,       // eXcess: 16/32/64-bit configurable embedded CPU
            EM_DXP = 112,          // Icera Semiconductor Inc. Deep Execution Processor
            EM_ALTERA_NIOS2 = 113, // Altera Nios II soft-core processor
            EM_CRX = 114,          // National Semiconductor CompactRISC CRX
            EM_XGATE = 115,        // Motorola XGATE embedded processor
            EM_C166 = 116,         // Infineon C16x/XC16x processor
            EM_M16C = 117,         // Renesas M16C series microprocessors
            EM_DSPIC30F = 118,     // Microchip Technology dsPIC30F Digital Signal
                                    // Controller
            EM_CE = 119,           // Freescale Communication Engine RISC core
            EM_M32C = 120,         // Renesas M32C series microprocessors
            EM_TSK3000 = 131,      // Altium TSK3000 core
            EM_RS08 = 132,         // Freescale RS08 embedded processor
            EM_SHARC = 133,        // Analog Devices SHARC family of 32-bit DSP
                                    // processors
            EM_ECOG2 = 134,        // Cyan Technology eCOG2 microprocessor
            EM_SCORE7 = 135,       // Sunplus S+core7 RISC processor
            EM_DSP24 = 136,        // New Japan Radio (NJR) 24-bit DSP Processor
            EM_VIDEOCORE3 = 137,   // Broadcom VideoCore III processor
            EM_LATTICEMICO32 = 138, // RISC processor for Lattice FPGA architecture
            EM_SE_C17 = 139,        // Seiko Epson C17 family
            EM_TI_C6000 = 140,      // The Texas Instruments TMS320C6000 DSP family
            EM_TI_C2000 = 141,      // The Texas Instruments TMS320C2000 DSP family
            EM_TI_C5500 = 142,      // The Texas Instruments TMS320C55x DSP family
            EM_MMDSP_PLUS = 160,    // STMicroelectronics 64bit VLIW Data Signal Processor
            EM_CYPRESS_M8C = 161,   // Cypress M8C microprocessor
            EM_R32C = 162,          // Renesas R32C series microprocessors
            EM_TRIMEDIA = 163,      // NXP Semiconductors TriMedia architecture family
            EM_HEXAGON = 164,       // Qualcomm Hexagon processor
            EM_8051 = 165,          // Intel 8051 and variants
            EM_STXP7X = 166,        // STMicroelectronics STxP7x family of configurable
                                    // and extensible RISC processors
            EM_NDS32 = 167,         // Andes Technology compact code size embedded RISC
                                    // processor family
            EM_ECOG1 = 168,         // Cyan Technology eCOG1X family
            EM_ECOG1X = 168,        // Cyan Technology eCOG1X family
            EM_MAXQ30 = 169,        // Dallas Semiconductor MAXQ30 Core Micro-controllers
            EM_XIMO16 = 170,        // New Japan Radio (NJR) 16-bit DSP Processor
            EM_MANIK = 171,         // M2000 Reconfigurable RISC Microprocessor
            EM_CRAYNV2 = 172,       // Cray Inc. NV2 vector architecture
            EM_RX = 173,            // Renesas RX family
            EM_METAG = 174,         // Imagination Technologies META processor
                                    // architecture
            EM_MCST_ELBRUS = 175,   // MCST Elbrus general purpose hardware architecture
            EM_ECOG16 = 176,        // Cyan Technology eCOG16 family
            EM_CR16 = 177,          // National Semiconductor CompactRISC CR16 16-bit
                                    // microprocessor
            EM_ETPU = 178,          // Freescale Extended Time Processing Unit
            EM_SLE9X = 179,         // Infineon Technologies SLE9X core
            EM_L10M = 180,          // Intel L10M
            EM_K10M = 181,          // Intel K10M
            EM_AARCH64 = 183,       // ARM AArch64
            EM_AVR32 = 185,         // Atmel Corporation 32-bit microprocessor family
            EM_STM8 = 186,          // STMicroeletronics STM8 8-bit microcontroller
            EM_TILE64 = 187,        // Tilera TILE64 multicore architecture family
            EM_TILEPRO = 188,       // Tilera TILEPro multicore architecture family
            EM_CUDA = 190,          // NVIDIA CUDA architecture
            EM_TILEGX = 191,        // Tilera TILE-Gx multicore architecture family
            EM_CLOUDSHIELD = 192,   // CloudShield architecture family
            EM_COREA_1ST = 193,     // KIPO-KAIST Core-A 1st generation processor family
            EM_COREA_2ND = 194,     // KIPO-KAIST Core-A 2nd generation processor family
            EM_ARC_COMPACT2 = 195,  // Synopsys ARCompact V2
            EM_OPEN8 = 196,         // Open8 8-bit RISC soft processor core
            EM_RL78 = 197,          // Renesas RL78 family
            EM_VIDEOCORE5 = 198,    // Broadcom VideoCore V processor
            EM_78KOR = 199,         // Renesas 78KOR family
            EM_56800EX = 200,       // Freescale 56800EX Digital Signal Controller (DSC)
            EM_BA1 = 201,           // Beyond BA1 CPU architecture
            EM_BA2 = 202,           // Beyond BA2 CPU architecture
            EM_XCORE = 203,         // XMOS xCORE processor family
            EM_MCHP_PIC = 204,      // Microchip 8-bit PIC(r) family
            EM_INTEL205 = 205,      // Reserved by Intel
            EM_INTEL206 = 206,      // Reserved by Intel
            EM_INTEL207 = 207,      // Reserved by Intel
            EM_INTEL208 = 208,      // Reserved by Intel
            EM_INTEL209 = 209,      // Reserved by Intel
            EM_KM32 = 210,          // KM211 KM32 32-bit processor
            EM_KMX32 = 211,         // KM211 KMX32 32-bit processor
            EM_KMX16 = 212,         // KM211 KMX16 16-bit processor
            EM_KMX8 = 213,          // KM211 KMX8 8-bit processor
            EM_KVARC = 214,         // KM211 KVARC processor
            EM_CDP = 215,           // Paneve CDP architecture family
            EM_COGE = 216,          // Cognitive Smart Memory Processor
            EM_COOL = 217,          // iCelero CoolEngine
            EM_NORC = 218,          // Nanoradio Optimized RISC
            EM_CSR_KALIMBA = 219,   // CSR Kalimba architecture family
            EM_AMDGPU = 224,        // AMD GPU architecture
            EM_RISCV = 243,         // RISC-V
            EM_LANAI = 244,         // Lanai 32-bit processor
            EM_BPF = 247,           // Linux kernel bpf virtual machine
            EM_VE = 251,            // NEC SX-Aurora VE
            EM_CSKY = 252,          // C-SKY 32-bit processor
            EM_WC65C816 = 257,      // 65816/65c816

            EM_CLEVER     = 0x434C, // Clever-ISA
            EM_HOLEYBYTES = 0xAB1E, // Holey Bytes
        }
    }

    fake_enum::fake_enum! {
        #[repr(u8)]
        #[derive(Zeroable,Pod)]
        pub enum EiClass{
            ELFCLASSNONE = 0,
            ELFCLASS32 = 1,
            ELFCLASS64 = 2
        }
    }

    fake_enum::fake_enum! {
        #[repr(u8)]
        #[derive(Zeroable,Pod)]
        pub enum EiData{
            ELFDATANONE = 0,
            ELFDATA2LSB = 1,
            ELFDATA2MSB = 2
        }
    }

    fake_enum::fake_enum! {
        #[repr(u8)]
        #[derive(Zeroable,Pod)]
        pub enum EiVersion{
            EV_NONE = 0,
            EV_CURRENT = 1
        }
    }

    fake_enum::fake_enum! {
        #[repr(u8)]
        #[derive(Zeroable,Pod)]
        pub enum EiOsAbi{
            ELFOSABI_NONE = 0,           // UNIX System V ABI
            ELFOSABI_HPUX = 1,           // HP-UX operating system
            ELFOSABI_NETBSD = 2,         // NetBSD
            ELFOSABI_GNU = 3,            // GNU/Linux
            ELFOSABI_LINUX = 3,          // Historical alias for ELFOSABI_GNU.
            ELFOSABI_HURD = 4,           // GNU/Hurd
            ELFOSABI_SOLARIS = 6,        // Solaris
            ELFOSABI_AIX = 7,            // AIX
            ELFOSABI_IRIX = 8,           // IRIX
            ELFOSABI_FREEBSD = 9,        // FreeBSD
            ELFOSABI_TRU64 = 10,         // TRU64 UNIX
            ELFOSABI_MODESTO = 11,       // Novell Modesto
            ELFOSABI_OPENBSD = 12,       // OpenBSD
            ELFOSABI_OPENVMS = 13,       // OpenVMS
            ELFOSABI_NSK = 14,           // Hewlett-Packard Non-Stop Kernel
            ELFOSABI_AROS = 15,          // AROS
            ELFOSABI_FENIXOS = 16,       // FenixOS
            ELFOSABI_CLOUDABI = 17,      // Nuxi CloudABI
            ELFOSABI_FIRST_ARCH = 64,    // First architecture-specific OS ABI
            ELFOSABI_AMDGPU_HSA = 64,    // AMD HSA runtime
            ELFOSABI_AMDGPU_PAL = 65,    // AMD PAL runtime
            ELFOSABI_AMDGPU_MESA3D = 66, // AMD GCN GPUs (GFX6+) for MESA runtime
            ELFOSABI_ARM = 97,           // ARM
            ELFOSABI_C6000_ELFABI = 64,  // Bare-metal TMS320C6000
            ELFOSABI_C6000_LINUX = 65,   // Linux TMS320C6000
            ELFOSABI_STANDALONE = 255,   // Standalone (embedded) application
        }
    }

    #[repr(C, packed)]
    #[derive(Copy, Clone, Debug, Zeroable, Pod)]
    pub struct ElfIdent {
        pub ei_mag: [u8; 4],
        pub ei_class: EiClass,
        pub ei_data: EiData,
        pub ei_version: EiVersion,
        pub ei_osabi: EiOsAbi,
        pub ei_abiversion: u8,
        pub ei_pad: [u8; 7],
    }

    static_assertions::const_assert_eq!(core::mem::size_of::<ElfIdent>(), 16);
    static_assertions::const_assert_eq!(core::mem::align_of::<ElfIdent>(), 1);

    fake_enum::fake_enum! {
        #[repr(pub u32)]
        #[derive(Zeroable,Pod)]
        pub enum ProgramType{
            PT_NULL = 0,
            PT_LOAD = 1,
            PT_DYNAMIC = 2,
            PT_INTERP = 3,
            PT_NOTE = 4,
            PT_SHLIB = 5,
            PT_PHDR = 6,
        }
    }

    fake_enum::fake_enum! {
        #[repr(pub u32)]
        #[derive(Zeroable,Pod)]
        pub enum SectionType{
            SHT_NULL = 0,
            SHT_PROGBITS = 1,
            SHT_SYMTAB = 2,
            SHT_STRTAB = 3,
            SHT_RELA = 4,
            SHT_HASH = 5,
            SHT_DYNAMIC = 6,
            SHT_NOTE = 7,
            SHT_NOBITS = 8,
            SHT_REL = 9,
            SHT_SHLIB = 10,
            SHT_DYNSYM = 11,
        }
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct ElfHeader<E: ElfClass> {
    pub e_ident: consts::ElfIdent,
    pub e_type: consts::ElfType,
    pub e_machine: consts::ElfMachine,
    pub e_version: ElfWord<E>,
    pub e_entry: ElfAddr<E>,
    pub e_phoff: ElfOffset<E>,
    pub e_shoff: ElfOffset<E>,
    pub e_flags: ElfWord<E>,
    pub e_ehsize: ElfHalf<E>,
    pub e_phentsize: ElfHalf<E>,
    pub e_phnum: ElfHalf<E>,
    pub e_shentsize: ElfHalf<E>,
    pub e_shnum: ElfHalf<E>,
    pub e_shstrndx: ElfHalf<E>,
}

unsafe impl<E: ElfClass> Zeroable for ElfHeader<E> {}
unsafe impl<E: ElfClass + 'static> Pod for ElfHeader<E> {}

pub trait SectionHeader {}

#[derive(Copy, Clone, Debug, Zeroable, Pod)]
#[repr(C)]
pub struct Elf32Phdr {
    pub p_type: consts::ProgramType,
    pub p_offset: ElfOffset<Elf32>,
    pub p_vaddr: ElfAddr<Elf32>,
    pub p_paddr: ElfAddr<Elf32>,
    pub p_filesz: ElfSize<Elf32>,
    pub p_memsz: ElfSize<Elf32>,
    pub p_flags: ElfWord<Elf32>,
    pub p_align: ElfSize<Elf32>,
}

impl Sealed for Elf32Phdr {}

impl ElfProgramHeader for Elf32Phdr {
    type Class = Elf32;

    fn pt_type(&self) -> consts::ProgramType {
        self.p_type
    }

    fn offset(&self) -> ElfOffset<Self::Class> {
        self.p_offset
    }

    fn vaddr(&self) -> ElfAddr<Self::Class> {
        self.p_vaddr
    }

    fn paddr(&self) -> ElfAddr<Self::Class> {
        self.p_paddr
    }

    fn memsize(&self) -> ElfSize<Self::Class> {
        self.p_memsz
    }

    fn filesize(&self) -> ElfSize<Self::Class> {
        self.p_filesz
    }

    fn align(&self) -> ElfSize<Self::Class> {
        self.p_align
    }

    fn flags(&self) -> ElfWord<Self::Class> {
        self.p_flags
    }
}

#[derive(Copy, Clone, Debug, Zeroable, Pod)]
#[repr(C)]
pub struct Elf64Phdr {
    pub p_type: consts::ProgramType,
    pub p_flags: ElfWord<Elf64>,
    pub p_offset: ElfOffset<Elf64>,
    pub p_vaddr: ElfAddr<Elf64>,
    pub p_paddr: ElfAddr<Elf64>,
    pub p_filesz: ElfSize<Elf64>,
    pub p_memsz: ElfSize<Elf64>,
    pub p_align: ElfSize<Elf64>,
}

impl Sealed for Elf64Phdr {}

impl ElfProgramHeader for Elf64Phdr {
    type Class = Elf64;

    fn pt_type(&self) -> consts::ProgramType {
        self.p_type
    }

    fn offset(&self) -> ElfOffset<Self::Class> {
        self.p_offset
    }

    fn vaddr(&self) -> ElfAddr<Self::Class> {
        self.p_vaddr
    }

    fn paddr(&self) -> ElfAddr<Self::Class> {
        self.p_paddr
    }

    fn memsize(&self) -> ElfSize<Self::Class> {
        self.p_memsz
    }

    fn filesize(&self) -> ElfSize<Self::Class> {
        self.p_filesz
    }

    fn align(&self) -> ElfSize<Self::Class> {
        self.p_align
    }

    fn flags(&self) -> ElfWord<Self::Class> {
        self.p_flags
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ElfSectionHeader<Class: ElfClass> {
    pub sh_name: ElfWord<Class>,
    pub sh_type: consts::SectionType,
    pub sh_flags: ElfOffset<Class>,
    pub sh_addr: ElfAddr<Class>,
    pub sh_offset: ElfOffset<Class>,
    pub sh_size: ElfSize<Class>,
    pub sh_link: ElfWord<Class>,
    pub sh_info: ElfWord<Class>,
    pub sh_addralign: ElfAddr<Class>,
    pub sh_entsize: ElfSize<Class>,
}

impl<Class: ElfClass> core::fmt::Debug for ElfSectionHeader<Class> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("ElfSectionHeader")
            .field("sh_name", &self.sh_name)
            .field("sh_type", &self.sh_type)
            .field("sh_flags", &PrintHex(self.sh_flags))
            .field("sh_addr", &PrintHex(self.sh_addr))
            .field("sh_offset", &self.sh_offset)
            .field("sh_size", &self.sh_size)
            .field("sh_link", &PrintHex(self.sh_link))
            .field("sh_info", &PrintHex(self.sh_info))
            .field("sh_addralign", &self.sh_addralign)
            .field("sh_entsize", &self.sh_entsize)
            .finish()
    }
}

unsafe impl<Class: ElfClass> Zeroable for ElfSectionHeader<Class> {}
unsafe impl<Class: ElfClass + 'static> Pod for ElfSectionHeader<Class> {}

#[derive(Copy, Clone, Debug)]
pub struct BadElfHeader;

impl Display for BadElfHeader {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.write_str("Invalid Elf Header")
    }
}

impl std::error::Error for BadElfHeader {}

pub struct ElfFileData<Class: ElfClass> {
    header: ElfHeader<Class>,
    phdrs: Vec<Class::ProgramHeader>,
}

impl<Class: ElfClass> ElfFileData<Class> {
    pub fn header(&self) -> &ElfHeader<Class> {
        &self.header
    }

    pub fn flags_mut(&mut self) -> &mut ElfWord<Class> {
        &mut self.header.e_flags
    }

    pub fn phdrs(&self) -> &[Class::ProgramHeader] {
        &self.phdrs
    }

    pub fn phdrs_mut(&mut self) -> &mut Vec<Class::ProgramHeader> {
        &mut self.phdrs
    }
}

pub struct ElfFormat<Class: ElfClass, Howto> {
    em: consts::ElfMachine,
    data: consts::EiData,
    create_header: Option<fn(&mut ElfHeader<Class>)>,
    name: &'static str,
    _cl: PhantomData<Class>,
    _howto: PhantomData<fn() -> Howto>,
    disassembler: Option<Box<dyn OpcodePrinter + Sync + Send>>,
}

impl<Class: ElfClass, Howto> ElfFormat<Class, Howto> {
    pub fn new(
        em: consts::ElfMachine,
        data: consts::EiData,
        name: &'static str,
        create_header: Option<fn(&mut ElfHeader<Class>)>,
        disassembler: Option<Box<dyn OpcodePrinter + Sync + Send>>,
    ) -> Self {
        Self {
            em,
            data,
            create_header,
            name,
            disassembler,
            _cl: PhantomData,
            _howto: PhantomData,
        }
    }
}

fn file_type_to_elf_type(ty: FileType) -> consts::ElfType {
    match ty {
        FileType::Exec => consts::ET_EXEC,
        FileType::Relocatable => consts::ET_REL,
        FileType::SharedObject => consts::ET_DYN,
        FileType::FormatSpecific(val) => consts::ElfType(val as u16),
    }
}

fn elf_type_to_file_type(ty: consts::ElfType) -> FileType {
    match ty {
        consts::ET_EXEC => FileType::Exec,
        consts::ET_REL => FileType::Relocatable,
        consts::ET_DYN => FileType::SharedObject,
        consts::ElfType(x) => FileType::FormatSpecific(x as u32),
    }
}

fn elf_shtype_to_file_type(ty: consts::SectionType) -> SectionType {
    match ty {
        consts::SHT_PROGBITS => SectionType::ProgBits,
        consts::SHT_SYMTAB => SectionType::SymbolTable,
        consts::SHT_STRTAB => SectionType::StringTable,
        consts::SHT_REL => SectionType::RelocationTable,
        consts::SHT_RELA => SectionType::RelocationAddendTable,
        consts::SHT_DYNAMIC => SectionType::Dynamic,
        consts::SHT_NOBITS => SectionType::NoBits,
        consts::SectionType(ty) => SectionType::FormatSpecific(ty),
    }
}

fn from_null_term_str(bytes: &[u8]) -> std::io::Result<String> {
    let l = bytes.split(|b| *b == 0).next().unwrap();

    core::str::from_utf8(l)
        .map_err(|e| std::io::Error::new(ErrorKind::InvalidData, e))
        .map(str::to_string)
}

impl<Class: ElfClass + 'static, Howto: HowTo + 'static> Binfmt for ElfFormat<Class, Howto> {
    fn relnum_to_howto(&self, relnum: u32) -> Option<&dyn HowTo> {
        Howto::from_relnum(relnum).map(|x| x as &dyn HowTo)
    }

    fn code_to_howto(&self, code: crate::howto::RelocCode) -> Option<&dyn HowTo> {
        Howto::from_reloc_code(code).map(|x| x as &dyn HowTo)
    }

    fn name(&self) -> &'static str {
        self.name
    }

    fn create_file(&self, ty: FileType) -> crate::fmt::BinaryFile {
        let mut header = ElfHeader {
            e_ident: ElfIdent {
                ei_mag: consts::ELFMAG,
                ei_class: Class::EI_CLASS,
                ei_data: self.data,
                ei_version: consts::EV_CURRENT,
                ei_osabi: consts::ELFOSABI_NONE,
                ei_abiversion: 0,
                ..Zeroable::zeroed()
            },
            e_type: file_type_to_elf_type(ty),
            e_machine: self.em,
            e_version: Numeric::from_usize(bytemuck::cast::<_, u8>(consts::EV_CURRENT) as usize),
            e_entry: Numeric::from_usize(0),
            e_phoff: Numeric::from_usize(0),
            e_shoff: Numeric::from_usize(0),
            e_flags: Numeric::from_usize(0),
            e_ehsize: Numeric::from_usize(0),
            e_phentsize: Numeric::from_usize(size_of::<Class::ProgramHeader>()),
            e_phnum: Numeric::from_usize(0),
            e_shentsize: Numeric::from_usize(size_of::<ElfSectionHeader<Class>>()),
            e_shnum: Numeric::from_usize(0),
            e_shstrndx: Numeric::from_usize(0),
        };

        if let Some(f) = self.create_header {
            (f)(&mut header);
        }

        let data = ElfFileData {
            header,
            phdrs: Vec::new(),
        };

        crate::fmt::BinaryFile::create(self, Box::new(data), ty)
    }

    #[allow(clippy::single_match)]
    fn read_file(
        &self,
        file: &mut (dyn ReadSeek + '_),
    ) -> std::io::Result<Option<crate::fmt::BinaryFile>> {
        let mut header = ElfHeader::<Class>::zeroed();
        file.read_exact(bytemuck::bytes_of_mut(&mut header.e_ident))?;

        if header.e_ident.ei_mag != consts::ELFMAG {
            return Ok(None);
        }

        if header.e_ident.ei_class != Class::EI_CLASS {
            return Ok(None);
        }

        if header.e_ident.ei_data != self.data {
            return Ok(None);
        }

        file.read_exact(&mut bytemuck::bytes_of_mut(&mut header)[16..])?;

        if header.e_phentsize != Numeric::from_usize(size_of::<Class::ProgramHeader>())
            && header.e_phnum != Numeric::zero()
        {
            return Err(std::io::Error::new(
                ErrorKind::InvalidData,
                "Invalid Program Header Entry Size",
            ));
        }
        let mut phdrs = vec![Class::ProgramHeader::zeroed(); header.e_phnum.as_usize()];
        if header.e_phnum.as_usize() != 0 {
            file.seek(SeekFrom::Start(header.e_phoff.as_u64()))?;
            file.read_exact(bytemuck::cast_slice_mut(&mut phdrs))?;
        }

        let data = ElfFileData { header, phdrs };
        #[allow(unused_mut)]
        let mut bfile =
            BinaryFile::create(self, Box::new(data), elf_type_to_file_type(header.e_type));

        if header.e_shentsize != Numeric::from_usize(size_of::<ElfSectionHeader<Class>>())
            && header.e_shnum != Numeric::zero()
        {
            return Err(std::io::Error::new(
                ErrorKind::InvalidData,
                "Invalid Section Header Entry Size",
            ));
        }

        file.seek(SeekFrom::Start(Numeric::as_u64(header.e_shoff)))?;

        let mut shdrs = vec![ElfSectionHeader::<Class>::zeroed(); header.e_shnum.as_usize()];
        file.read_exact(bytemuck::cast_slice_mut(&mut shdrs))?;

        let mut strings = Vec::new();

        let shstrndx = Numeric::as_usize(header.e_shstrndx);

        let shstrhdr = *shdrs.get(shstrndx).ok_or_else(|| {
            std::io::Error::new(
                ErrorKind::InvalidData,
                "Invalid or Out of range section string table index",
            )
        })?;

        strings.resize(Numeric::as_usize(shstrhdr.sh_size), 0u8);

        file.seek(SeekFrom::Start(Numeric::as_u64(shstrhdr.sh_offset)))?;

        file.read_exact(&mut strings)?;

        for shdr in &shdrs[1..] {
            let mut sect = Section {
                align: Numeric::as_usize(shdr.sh_addralign),
                ty: elf_shtype_to_file_type(shdr.sh_type),
                ..Section::default()
            };

            let noff = Numeric::as_usize(shdr.sh_name);
            let name = &strings[noff..];

            sect.name = from_null_term_str(name)?;

            sect.content.resize(Numeric::as_usize(shdr.sh_size), 0);

            if sect.ty != SectionType::NoBits {
                file.seek(SeekFrom::Start(Numeric::as_u64(shdr.sh_offset)))
                    .unwrap();
                file.read_exact(&mut sect.content)?;
            }

            sect.info = shdr.sh_info.as_u64();
            sect.link = shdr.sh_link.as_u64();

            match sect.ty {
                SectionType::SymbolTable => {
                    if shdr.sh_entsize.as_usize() != size_of::<Class::Symbol>() {
                        return Err(std::io::Error::new(
                            ErrorKind::InvalidData,
                            "Invalid Symbol Header Entry Size",
                        ));
                    }
                }
                _ => {}
            }

            drop(bfile.add_section(sect))
        }

        let mut syms = Vec::new();
        for sect in bfile.sections() {
            match sect.ty {
                SectionType::SymbolTable => {
                    let strsect = bfile.get_section((sect.link as u32) - 1).ok_or_else(|| {
                        std::io::Error::new(
                            ErrorKind::InvalidData,
                            "Out of range str section referenced from symbol table",
                        )
                    })?;
                    for window in sect
                        .content
                        .chunks_exact(size_of::<Class::Symbol>())
                        .skip(1)
                    {
                        let mut sym = Class::Symbol::zeroed();
                        bytemuck::bytes_of_mut(&mut sym).copy_from_slice(window);
                        let name = sym.name_idx().as_usize();

                        let name = from_null_term_str(&strsect.content[name..])?;

                        let value = sym.value().as_u64() as u128;
                        let section = (sym.section().as_u64() as u32).checked_sub(1);

                        let sym = crate::sym::Symbol::new(
                            name,
                            section,
                            section.map(|_| value),
                            SymbolType::Object,
                            SymbolKind::Local,
                        );

                        syms.push(sym);
                    }
                }
                _ => {}
            }
        }

        bfile.add_symbols(syms).unwrap();

        Ok(Some(bfile))
    }

    fn write_file(
        &self,
        file: &mut (dyn std::io::Write + '_),
        bfile: &crate::fmt::BinaryFile,
    ) -> std::io::Result<()> {
        let mut shstrtab = (Vec::new(), HashMap::new());
        fn add_to_strtab<'a>(
            strtab: &mut (Vec<u8>, HashMap<Cow<'a, str>, usize>),
            string: Cow<'a, str>,
        ) -> usize {
            if let Entry::Vacant(e) = strtab.1.entry(string.clone()) {
                let addr = strtab.0.len();
                strtab.0.extend_from_slice(string.as_bytes());
                strtab.0.push(0);
                e.insert(addr);
                addr
            } else {
                strtab.1[&string]
            }
        }
        let mut shdrs = Vec::new();
        let mut offset = size_of::<ElfHeader<Class>>()
            + size_of::<Class::ProgramHeader>()
                * (bfile.data().downcast_ref::<ElfFileData<Class>>())
                    .unwrap()
                    .phdrs
                    .len();
        shdrs.push(ElfSectionHeader::<Class> {
            sh_name: Class::Word::from_usize(add_to_strtab(&mut shstrtab, "".into())),
            sh_type: consts::SHT_NULL,
            sh_flags: Class::Offset::from_usize(0),
            sh_addr: Class::Addr::from_usize(0),
            sh_offset: Class::Offset::from_usize(0),
            sh_size: Class::Size::from_usize(0),
            sh_link: Class::Word::from_usize(0),
            sh_info: Class::Word::from_usize(0),
            sh_addralign: Class::Addr::from_usize(0),
            sh_entsize: Class::Size::from_usize(0),
        });
        for section in bfile.sections() {
            let is_nobits = section.ty == SectionType::NoBits || section.content.is_empty();
            #[allow(clippy::needless_borrow)]
            shdrs.push(ElfSectionHeader::<Class> {
                sh_name: Class::Word::from_usize(add_to_strtab(
                    &mut shstrtab,
                    (&section.name).into(),
                )),
                sh_type: match section.ty {
                    SectionType::NoBits => consts::SHT_NOBITS,
                    SectionType::ProgBits => {
                        if section.content.is_empty() {
                            consts::SHT_NOBITS
                        } else {
                            consts::SHT_PROGBITS
                        }
                    }
                    SectionType::SymbolTable => consts::SHT_SYMTAB,
                    SectionType::StringTable => consts::SHT_STRTAB,
                    SectionType::Dynamic => consts::SHT_DYNAMIC,
                    SectionType::ProcedureLinkageTable => todo!(),
                    SectionType::GlobalOffsetTable => todo!(),
                    SectionType::FormatSpecific(_) => todo!(),
                    SectionType::SymbolHashTable(_) => todo!(),
                    SectionType::RelocationTable => todo!(),
                    SectionType::RelocationAddendTable => todo!(),
                },
                sh_flags: Class::Offset::from_usize(7),
                sh_addr: Class::Addr::from_usize(0),
                sh_offset: Class::Offset::from_usize(offset),
                sh_size: Class::Size::from_usize(section.content.len() + section.tail_size),
                sh_link: Class::Word::from_usize(0),
                sh_info: Class::Word::from_usize(0),
                sh_addralign: Class::Addr::from_usize(section.align),
                sh_entsize: Class::Size::from_usize(0),
            });
            offset += if !is_nobits {
                section.content.len() + section.tail_size
            } else {
                0
            };
        }
        let mut new_symbol_list: Vec<_> = bfile.symbols().cloned().collect();
        new_symbol_list.sort_by_key(|s1| s1.kind());
        let mut num_reloc_sections = 0;
        for section in bfile.sections() {
            if !section.relocs.is_empty() {
                num_reloc_sections += 1;
                for reloc in &section.relocs {
                    if !new_symbol_list.iter().any(|x| x.name() == reloc.symbol) {
                        new_symbol_list.push(crate::sym::Symbol::new(
                            reloc.symbol.clone(),
                            None,
                            None,
                            SymbolType::Null,
                            SymbolKind::Global,
                        ));
                    }
                }
            }
        }
        let mut symbols: Vec<Class::Symbol> = Vec::new();
        let mut strtab = (Vec::new(), HashMap::new());
        symbols.push(Class::new_sym(
            Class::Word::from_usize(add_to_strtab(&mut strtab, "".into())),
            Class::Addr::from_usize(0),
            Class::Size::from_usize(0),
            0,
            0,
            Class::Half::from_usize(0),
        ));
        let mut local_syms = 1; // Includes null symbol
        for sym in &new_symbol_list {
            symbols.push(Class::new_sym(
                Class::Word::from_usize(add_to_strtab(
                    &mut strtab,
                    String::from(sym.name()).into(),
                )),
                Class::Addr::from_usize(sym.value().map_or(0, |x| x as usize)),
                Class::Size::from_usize(0usize),
                (match sym.kind() {
                    SymbolKind::Local => {
                        local_syms += 1;
                        0
                    }
                    SymbolKind::Global => 1,
                    SymbolKind::Weak => 2,
                    SymbolKind::FormatSpecific(x) => x as u8,
                } << 4)
                    | match sym.symbol_type() {
                        SymbolType::Null => 0,
                        SymbolType::Object => 1,
                        SymbolType::Function => 2,
                        SymbolType::Section => 3,
                        SymbolType::File => 4,
                        SymbolType::Common => 5,
                        SymbolType::Tls => 6,
                        SymbolType::FormatSpecific(x) => x as u8,
                    },
                0,
                Class::Half::from_usize(sym.section().map_or(0, |x| x as usize + 1)),
            ));
        }
        let symbols_sec: Vec<u8> = Vec::from(bytemuck::cast_slice(&symbols));
        let symbols_sec_id = shdrs.len();
        shdrs.push(ElfSectionHeader::<Class> {
            sh_name: Class::Word::from_usize(add_to_strtab(&mut shstrtab, ".symtab".into())),
            sh_type: consts::SHT_SYMTAB,
            sh_flags: Class::Offset::from_usize(0),
            sh_addr: Class::Addr::from_usize(0),
            sh_offset: Class::Offset::from_usize(offset),
            sh_size: Class::Size::from_usize(symbols_sec.len()),
            sh_link: Class::Word::from_usize(shdrs.len() + 1 + num_reloc_sections),
            sh_info: Class::Word::from_usize(local_syms),
            sh_addralign: Class::Addr::from_usize(8),
            sh_entsize: Class::Size::from_usize(size_of::<Class::Symbol>()),
        });
        offset += symbols_sec.len();
        let mut all_relocs: Vec<u8> = Vec::new();
        for (i, section) in bfile.sections().enumerate() {
            if !section.relocs.is_empty() {
                let mut relocs = Vec::new();
                for reloc in &section.relocs {
                    eprintln!("Handling reloc {:?}", reloc);
                    let reloc = ElfRela::<Class> {
                        r_offset: Class::Addr::from_usize(reloc.offset as usize),
                        r_info: Class::mk_rinfo(
                            new_symbol_list
                                .iter()
                                .position(|x| x.name() == reloc.symbol)
                                .unwrap()
                                + 1,
                            Howto::from_reloc_code(reloc.code).unwrap().reloc_num() as usize,
                        ),
                        r_addend: Class::Offset::from_usize(reloc.addend.map_or(0, |x| x as usize)),
                    };
                    eprintln!("\t>Elf Relocation {:?}", reloc);
                    relocs.push(reloc);
                }
                let mut relocs = Vec::from(bytemuck::cast_slice(&relocs));
                let target_section = i + 1;
                shdrs.push(ElfSectionHeader::<Class> {
                    sh_name: Class::Word::from_usize(add_to_strtab(
                        &mut shstrtab,
                        (".rela".to_string() + &section.name).into(),
                    )),
                    sh_type: consts::SHT_RELA,
                    sh_flags: Class::Offset::from_usize(7),
                    sh_addr: Class::Addr::from_usize(0),
                    sh_offset: Class::Offset::from_usize(offset),
                    sh_size: Class::Size::from_usize(relocs.len()),
                    sh_link: Class::Word::from_usize(symbols_sec_id),
                    sh_info: Class::Word::from_usize(target_section),
                    sh_addralign: Class::Addr::from_usize(8),
                    sh_entsize: Class::Size::from_usize(size_of::<Class::Rela>()),
                });
                offset += relocs.len();
                all_relocs.append(&mut relocs);
            }
        }
        shdrs.push(ElfSectionHeader::<Class> {
            sh_name: Class::Word::from_usize(add_to_strtab(&mut shstrtab, ".strtab".into())),
            sh_type: consts::SHT_STRTAB,
            sh_flags: Class::Offset::from_usize(0),
            sh_addr: Class::Addr::from_usize(0),
            sh_offset: Class::Offset::from_usize(offset),
            sh_size: Class::Size::from_usize(strtab.0.len()),
            sh_link: Class::Word::from_usize(0),
            sh_info: Class::Word::from_usize(0),
            sh_addralign: Class::Addr::from_usize(1),
            sh_entsize: Class::Size::from_usize(0),
        });
        offset += strtab.0.len();
        shdrs.push(ElfSectionHeader::<Class> {
            sh_name: Class::Word::from_usize(add_to_strtab(&mut shstrtab, ".shstrtab".into())),
            sh_type: consts::SHT_STRTAB,
            sh_flags: Class::Offset::from_usize(0),
            sh_addr: Class::Addr::from_usize(0),
            sh_offset: Class::Offset::from_usize(offset),
            sh_size: Class::Size::from_usize(shstrtab.0.len()),
            sh_link: Class::Word::from_usize(0),
            sh_info: Class::Word::from_usize(0),
            sh_addralign: Class::Addr::from_usize(1),
            sh_entsize: Class::Size::from_usize(0),
        });
        offset += shstrtab.0.len();
        let mut header = bfile
            .data()
            .downcast_ref::<ElfFileData<Class>>()
            .unwrap()
            .header;
        header.e_shnum = Class::Half::from_usize(shdrs.len());
        header.e_shoff = Class::Offset::from_usize(offset);
        header.e_shstrndx = Class::Half::from_usize(shdrs.len() - 1);
        file.write_all(bytemuck::bytes_of(&header))?;
        for section in bfile.sections() {
            if section.ty == SectionType::NoBits || section.content.is_empty() {
                continue;
            }
            file.write_all(&section.content)?;
            arch_ops::traits::default_write_zeroes(&mut *file, section.tail_size)?;
        }
        file.write_all(&symbols_sec)?;
        file.write_all(&all_relocs)?;
        file.write_all(&strtab.0)?;
        file.write_all(&shstrtab.0)?;
        file.write_all(bytemuck::cast_slice(&shdrs))?;
        Ok(())
    }

    fn has_sections(&self) -> bool {
        true
    }

    fn ident_file(&self, file: &mut (dyn std::io::Read + '_)) -> std::io::Result<bool> {
        let mut header = ElfHeader::<Class>::zeroed();
        file.read_exact(bytemuck::bytes_of_mut(&mut header.e_ident))?;

        if header.e_ident.ei_mag != consts::ELFMAG {
            return Ok(false);
        }

        if header.e_ident.ei_class != Class::EI_CLASS {
            return Ok(false);
        }

        if header.e_ident.ei_data != self.data {
            return Ok(false);
        }

        file.read_exact(&mut bytemuck::bytes_of_mut(&mut header)[16..])?;

        if self.em != header.e_machine && self.em != consts::EM_NONE {
            return Ok(false);
        }

        Ok(true)
    }

    fn disassembler(&self) -> Option<&dyn OpcodePrinter> {
        self.disassembler
            .as_deref()
            .map(|d| d as &dyn OpcodePrinter)
    }
}

pub struct ElfHowToUnknown;

impl HowTo for ElfHowToUnknown {
    fn from_relnum<'a>(_: u32) -> Option<&'a Self>
    where
        Self: Sized + 'a,
    {
        Some(&ElfHowToUnknown)
    }

    fn from_reloc_code<'a>(_: arch_ops::traits::RelocCode) -> Option<&'a Self>
    where
        Self: Sized + 'a,
    {
        None
    }

    fn reloc_num(&self) -> u32 {
        0
    }

    fn name(&self) -> &'static str {
        "**UNKNOWN RELOC**"
    }

    fn reloc_size(&self) -> usize {
        0
    }

    fn pcrel(&self) -> bool {
        false
    }

    fn is_relax(&self) -> bool {
        false
    }

    fn relax_size(&self, _: u128, _: u128) -> Option<usize> {
        None
    }

    fn apply(&self, _: u128, _: u128, _: &mut [u8]) -> Result<bool, crate::howto::HowToError> {
        Ok(false)
    }
}
