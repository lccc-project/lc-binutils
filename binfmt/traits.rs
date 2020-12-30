use crate::traits::private::Sealed;
use std::{fmt::{Debug, LowerHex}, ops::*};

#[doc(hide)]
pub(crate) mod private {
    pub trait Sealed {}
}

pub trait Numeric:
    Add + Mul + Sub + Div + BitAnd + BitOr + BitXor + Not + Copy + Shl + Shr + Sized + Debug + LowerHex + Sealed
{
    fn zero() -> Self;
    fn one() -> Self;
    fn min() -> Self;
    fn max() -> Self;
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
            }
            )*
        };
    }

impl_numeric!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, isize);
