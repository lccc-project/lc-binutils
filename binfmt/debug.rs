use core::fmt::LowerHex;

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct PrintHex<T>(pub T);

impl<T> From<T> for PrintHex<T> {
    fn from(x: T) -> Self {
        Self(x)
    }
}

impl<T> ::core::ops::Deref for PrintHex<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> ::core::ops::DerefMut for PrintHex<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: LowerHex> ::core::fmt::Debug for PrintHex<T> {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.write_fmt(format_args!("{:#x}", self.0))
    }
}
