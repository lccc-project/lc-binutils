use bytemuck::Pod;

use crate::traits::private::Sealed;
use core::{
    convert::TryInto,
    fmt::{Debug, LowerHex},
    ops::*,
};

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
