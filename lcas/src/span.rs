use std::ops::{Deref, DerefMut};

use crate::sym::Symbol;

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Pos(u32);

impl Pos {
    pub const fn new(row: u32, col: u32) -> Self {
        [()][(row > 0xFFFFF) as usize];
        [()][(col > 0xFFF) as usize];
        Self(row | (col << 20))
    }

    pub const fn synthetic() -> Self {
        Self(!0)
    }

    pub const fn is_synthetic(&self) -> bool {
        self.0 == !0
    }

    #[allow(dead_code)]
    pub const fn row(self) -> u32 {
        self.0 & 0xFFFFF
    }

    #[allow(dead_code)]
    pub const fn col(self) -> u32 {
        self.0 >> 20
    }

    pub const fn next_col(self, n: u32) -> Self {
        Self::new(self.row(), self.col() + n)
    }

    pub const fn next_row(self, n: u32) -> Self {
        Self::new(self.row() + n, 0)
    }
}

impl core::fmt::Debug for Pos {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}:{}", self.row(), self.col())
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Span {
    begin: Pos,
    end: Pos,
    file: Symbol,
}

impl Span {
    pub fn new_simple<S: Into<Symbol>>(begin: Pos, end: Pos, file: S) -> Self {
        Self {
            begin,
            end,
            file: file.into(),
        }
    }

    pub fn synthetic() -> Self {
        Self::new_simple(Pos::synthetic(), Pos::synthetic(), "<synthetic>")
    }

    pub fn file(&self) -> Symbol {
        self.file
    }

    pub fn begin(&self) -> Pos {
        self.begin
    }

    pub fn end(&self) -> Pos {
        self.end
    }

    pub fn is_empty(&self) -> bool {
        self.begin == self.end
    }

    pub fn is_synthetic(&self) -> bool {
        self.is_empty() && self.begin.is_synthetic()
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Spanned<T> {
    val: T,
    span: Span,
}

impl<T> Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.val
    }
}

impl<T> DerefMut for Spanned<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.val
    }
}

impl<T> Spanned<T> {
    pub const fn new(val: T, span: Span) -> Self {
        Self { val, span }
    }

    pub fn into_inner(self) -> T {
        self.val
    }

    pub const fn span(&self) -> &Span {
        &self.span
    }

    pub const fn body(&self) -> &T {
        &self.val
    }

    pub fn body_mut(&mut self) -> &mut T {
        &mut self.val
    }

    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Spanned<U> {
        let Self { val, span } = self;

        Spanned::new(f(val), span)
    }

    pub fn try_map<U, E, F: FnOnce(T) -> Result<U, E>>(self, f: F) -> Result<Spanned<U>, E> {
        let Self { val, span } = self;

        f(val).map(|val| Spanned::new(val, span))
    }

    pub fn copy_span<U, F: FnOnce(&T) -> U>(&self, f: F) -> Spanned<U> {
        let Self { val, span } = self;

        Spanned::new(f(val), *span)
    }

    pub fn try_copy_span<U, E, F: FnOnce(&T) -> Result<U, E>>(
        &self,
        f: F,
    ) -> Result<Spanned<U>, E> {
        let Self { val, span } = self;

        f(val).map(|val| Spanned::new(val, *span))
    }
}

impl<A, B> Spanned<(A, B)> {
    pub fn unzip(self) -> (Spanned<A>, Spanned<B>) {
        let Self { val, span } = self;

        (Spanned::new(val.0, span), Spanned::new(val.1, span))
    }
}
