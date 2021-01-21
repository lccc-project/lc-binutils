use std::{
    convert::TryInto,
    fmt::{Debug, Display, LowerHex},
    ops::{Add, BitAnd, BitOr, BitXor, Not, RangeBounds, Sub},
    sync::{Arc, Mutex},
};

pub trait Scalar:
    Sized
    + Copy
    + Clone
    + Display
    + LowerHex
    + Add
    + Sub
    + BitAnd
    + BitOr
    + BitXor
    + Not
    + TryInto<usize>
    + Send
    + Sync
{
}

impl Scalar for u8 {}
impl Scalar for u16 {}
impl Scalar for u32 {}
impl Scalar for u64 {}
impl Scalar for u128 {}
impl Scalar for usize {}
impl Scalar for i8 {}
impl Scalar for i16 {}
impl Scalar for i32 {}
impl Scalar for i64 {}
impl Scalar for i128 {}
impl Scalar for isize {}

pub trait Address: Sized + Display + 'static {
    type Value: Scalar;

    fn is_symbol(&self) -> bool;

    fn to_value(&self) -> Option<Self::Value>;

    fn symbol_name(&self) -> Option<&str>;

    fn is_absolute(&self) -> bool;

    fn to_absolute(&self, base: Self::Value) -> Option<Self::Value>;
}

pub trait Operand: Sized + Display + 'static {
    type Arch: Architecture;

    fn as_address(&self) -> Option<&<Self::Arch as Architecture>::Address>;
    fn as_indirect_address(&self) -> Option<&<Self::Arch as Architecture>::Address>;
    fn as_immediate(&self) -> Option<&<Self::Arch as Architecture>::Immediate>;
    fn as_address_fragment(&self) -> Option<&<Self::Arch as Architecture>::AddressPart>;
    fn as_register(&self) -> Option<&<Self::Arch as Architecture>::Register>;
    fn is_implied(&self) -> bool;
}

pub trait Register: Sized + Display + 'static {
    type Value: Scalar;

    fn known_size(&self) -> Option<u32>;
    fn size_range(&self) -> (u32, u32);
}

pub trait AddressPart: Sized + Display + 'static {
    type Address: Address;
    type PositionRange: RangeBounds<u32>;
    fn get_bits(&self) -> Self::PositionRange;
    fn mask(&self) -> <Self::Address as Address>::Value;
    fn get_address(&self) -> &Self::Address;
}

pub trait InstructionLifetime<'a>: 'a {
    type Arch: Architecture;
    type Operands: IntoIterator<Item = &'a <Self::Arch as Architecture>::Operand>;
}

pub trait Instruction: Sized + Display + for<'a> InstructionLifetime<'a> {
    fn name(&self) -> &str;
    fn operands(&self) -> <Self as InstructionLifetime<'_>>::Operands;
}

pub trait Relocation: Sized + 'static {
    type Address: Address;
    type AddressPart: AddressPart<Address = Self::Address>;
    type RelocationType: Debug + Eq + Copy;

    fn get_type(&self) -> Self::RelocationType;

    fn get_address(&self) -> Option<&Self::Address>;
    fn get_part(&self) -> Option<&Self::AddressPart>;
}

pub trait RelocationWriter: 'static {
    type Relocation: Relocation;

    fn write_relocation(&mut self, reloc: Self::Relocation);
}

pub trait Architecture: Sized + 'static {
    type Operand: Operand<Arch = Self> + 'static;
    type Address: Address;
    type Immediate: Scalar;
    type AddressPart: AddressPart<Address = Self::Address>;
    type Register: Register;
    type Instruction: Instruction<Arch = Self>;
    type Relocation: Relocation<Address = Self::Address>;
    type InstructionWriter: InstructionWriter<Arch = Self, Instruction = Self::Instruction>;
    type InstructionReader: InstructionReader<Arch = Self, Instruction = Self::Instruction>;

    fn registers(&self) -> &[Self::Register];
    fn new_writer(
        &self,
        relocs: Arc<Mutex<dyn RelocationWriter<Relocation = Self::Relocation>>>,
    ) -> Self::InstructionWriter;
    fn new_reader(&self) -> Self::InstructionReader;
}

pub trait InstructionWriter {
    type Arch: Architecture;
    type Instruction: Instruction;
    type Relocation: Relocation<
        Address = <Self::Arch as Architecture>::Address,
        AddressPart = <Self::Arch as Architecture>::AddressPart,
    >;
    type Error: std::error::Error;

    fn get_architecture(&self) -> &Self::Arch;

    fn write_instruction(&mut self, ins: Self::Instruction) -> Result<(), Self::Error>;

    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), Self::Error>;
}

pub trait InstructionReader {
    type Arch: Architecture;
    type Instruction: Instruction;
    type Error: std::error::Error;

    fn get_architecture(&self) -> &Self::Arch;

    fn read_instruction(&mut self) -> Result<Self::Instruction, Self::Error>;

    fn read_bytes(&mut self, bytes: &mut [u8]) -> Result<(), Self::Error>;
}
