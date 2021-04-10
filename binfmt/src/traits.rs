use bytemuck::Pod;

use crate::traits::private::Sealed;
use core::{
    convert::TryInto,
    fmt::{Debug, LowerHex},
    ops::*,
};
use std::io::{Read, Write};

#[doc(hidden)]
pub(crate) mod private {
    pub trait Sealed {}
}

pub trait Numeric:
    Add<Output = Self>
    + Mul<Output = Self>
    + Sub<Output = Self>
    + Div<Output = Self>
    + BitAnd<Output = Self>
    + BitOr<Output = Self>
    + BitXor<Output = Self>
    + Not<Output = Self>
    + Copy
    + Shl<Output = Self>
    + Shr<Output = Self>
    + Sized
    + Debug
    + LowerHex
    + TryInto<usize>
    + Pod
    + Sealed
{
    fn zero() -> Self;
    fn one() -> Self;
    fn min() -> Self;
    fn max() -> Self;
    fn as_usize(self) -> usize;
    fn from_be(self) -> Self;
    fn from_le(self) -> Self;
}

#[doc(hide)]
macro_rules! impl_numeric {
        ($($n:ident),*) => {
            $(
            impl Sealed for $n{}
            impl Numeric for $n{
                fn zero()->Self{
                    0 as $n
                }
                fn one()->Self{
                    1 as $n
                }
                fn min()->Self{
                    $n::MIN
                }
                fn max()->Self{
                    $n::MAX
                }
                fn as_usize(self) -> usize{
                    self as usize
                }

                fn from_be(self) -> Self{
                    $n::from_be(self)
                }

                fn from_le(self) -> Self{
                    $n::from_le(self)
                }
            }
            )*
        };
    }

impl_numeric!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, isize);

pub trait Section {
    fn read(&self) -> Box<dyn Read + '_>;
    fn write(&mut self) -> Box<dyn Write + '_>;
    fn flags(&self) -> u64;
    fn set_flags(&mut self, flg: u64);
}

pub trait Segment {
    fn read(&self) -> Box<dyn Read + '_>;
    fn write(&mut self) -> Box<dyn Write + '_>;
    fn flags(&self) -> u64;
    fn set_flags(&mut self, flg: u64);
    fn segment_type(&self) -> u64;
    fn set_segment_type(&mut self, pt: u64) -> Result<(), u64>;
    fn align(&self) -> u64;
    fn set_alignment(&mut self, align: u64);
}

pub trait BinaryFile {
    fn read(read: &mut (dyn Read + '_)) -> std::io::Result<Box<Self>>
    where
        Self: Sized;
    fn write(&self, write: &mut (dyn Write + '_)) -> std::io::Result<()>;
    fn is_relocatable(&self) -> bool;
    fn has_symbols(&self) -> bool;
    fn has_sections(&self) -> bool;
    fn section(&self, name: &str) -> Option<&(dyn Section + '_)>;
    fn segments(&self) -> Vec<&(dyn Segment + '_)>;
    fn section_mut(&mut self, name: &str) -> Option<&(dyn Segment + '_)>;
    fn segments_mut(&mut self) -> Vec<&mut (dyn Segment + '_)>;
    fn create_segment(&mut self) -> Option<&mut (dyn Segment + '_)>;
    fn insert_segment(&mut self, idx: u32) -> Option<&mut (dyn Segment + '_)>;
}
