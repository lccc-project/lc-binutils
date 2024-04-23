use bytemuck::Pod;

use crate::traits::private::Sealed;
use core::{
    fmt::{Debug, LowerHex},
    ops::*,
};
use std::io::{Read, Seek};

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
    + TryFrom<usize>
    + Eq
    + Ord
    + Pod
    + Sealed
{
    fn zero() -> Self;
    fn one() -> Self;
    fn min() -> Self;
    fn max() -> Self;
    fn as_usize(self) -> usize;
    fn from_usize(x: usize) -> Self;
    fn as_u64(self) -> u64;
    fn from_u64(x: u64) -> Self;
    #[must_use]
    fn from_be(self) -> Self;
    #[must_use]
    fn from_le(self) -> Self;
}

#[doc(hidden)]
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
                    $n::min_value()
                }
                fn max()->Self{
                    $n::max_value()
                }
                fn as_usize(self) -> usize{
                    self as usize
                }

                fn from_usize(x: usize) -> Self{
                    x as Self
                }
                fn as_u64(self) -> u64{
                    self as u64
                }

                fn from_u64(x: u64) -> Self{
                    x as Self
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

pub trait ReadSeek: Read + Seek {}

impl<T: Read + Seek + ?Sized> ReadSeek for T {}
