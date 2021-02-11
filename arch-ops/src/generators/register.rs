#[macro_export]
macro_rules! registers{
    ($arch:ident[$($def:ident, $name:literal, $size:literal),*]) => {
        pub enum Registers{
            $($def),*
        }
        impl ::std::fmt::Display for Registers{
            pub fn fmt(&self,f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result{
                match self{
                    ($def => f.write_str($name)),*
                }
            }
        }
    }
}
