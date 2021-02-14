#[doc(hidden)]
const fn __from_bit(t: &str) -> u8 {
    if t.len() != 1 {
        return 255;
    } else {
        match t.as_bytes()[0] {
            b'0' => 0,
            b'1' => 1,
            _ => 255,
        }
    }
}

#[doc(hidden)]
pub const fn __from_bits_arrays<const N: usize>(v: [[u8; 8]; N]) -> [u8; N] {
    let mut out = [0u8; N];

    let mut i = 0;
    while i < N {
        out[i] = match v[i] {
            [_, _, _, _, _, _, _, 0] => 0,
            [_, _, _, _, _, _, _, 1] => 1,
            [_, _, _, _, _, _, _, _] => 255,
        };

        out[i] |= match v[i] {
            [_, _, _, _, _, _, 0, _] => 0,
            [_, _, _, _, _, _, 1, _] => 2,
            [_, _, _, _, _, _, _, _] => 255,
        };
        out[i] |= match v[i] {
            [_, _, _, _, _, 0, _, _] => 0,
            [_, _, _, _, _, 1, _, _] => 4,
            [_, _, _, _, _, _, _, _] => 255,
        };
        out[i] |= match v[i] {
            [_, _, _, _, 0, _, _, _] => 0,
            [_, _, _, _, 1, _, _, _] => 8,
            [_, _, _, _, _, _, _, _] => 255,
        };
        out[i] |= match v[i] {
            [_, _, _, 0, _, _, _, _] => 0,
            [_, _, _, 1, _, _, _, _] => 16,
            [_, _, _, _, _, _, _, _] => 255,
        };
        out[i] |= match v[i] {
            [_, _, 0, _, _, _, _, _] => 0,
            [_, _, 1, _, _, _, _, _] => 32,
            [_, _, _, _, _, _, _, _] => 255,
        };
        out[i] |= match v[i] {
            [_, 0, _, _, _, _, _, _] => 0,
            [_, 1, _, _, _, _, _, _] => 64,
            [_, _, _, _, _, _, _, _] => 255,
        };
        out[i] |= match v[i] {
            [0, _, _, _, _, _, _, _] => 0,
            [1, _, _, _, _, _, _, _] => 128,
            [_, _, _, _, _, _, _, _] => 255,
        };
        i += 1;
    }

    out
}

#[macro_export]
macro_rules! parse_instruction{
    ($arch:ident, $([$decl:ident, $name:literal, $opcode:tt]),*) => {
        #[derive(Debug,PartialEq,Eq)]
        pub enum Instructions{
            $($decl),*
        }
        impl ::std::fmt::Display for Instructions{
            fn fmt(&self,f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result{
                match self{
                    $(Self:: $decl => f.write_str($name)),*
                }
            }
        }
        impl Instructions{
            pub fn write_opcode<W: ::std::io::Write>(&self,w: &mut W) -> ::std::io::Result<()>{
                match self{
                    $(Self:: $decl => w.write_all(&$crate::generators::instructions::__from_bits_arrays($opcode))?),*
                }
                ::std::result::Result::Ok(())
            }
        }
    }
}
