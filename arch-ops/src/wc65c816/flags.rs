macro_rules! template{
    ($arch:ident, $([$def:ident, $name:literal]),*) => {
        #[derive(Debug,Copy,Clone,PartialEq,Eq)]
        pub enum Wc65c816Flag{
            $($def),*
        }

        impl ::std::fmt::Display for Wc65c816Flag{
            fn fmt(&self,f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result{
                match self{
                    $(Self::$def => f.write_str($name)),*
                }
            }
        }
    }
}

#[tablegen(arch = wc65c816)]
template!(Wc65c816Flag[name]);
