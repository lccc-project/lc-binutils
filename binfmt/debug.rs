use std::fmt::LowerHex;



#[repr(transparent)]
#[derive(Copy,Clone)]
pub struct PrintHex<T>(pub T);

impl<T: LowerHex> ::core::fmt::Debug for PrintHex<T>{
    fn fmt(&self,f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result{
        f.write_fmt(format_args!("{:#x}",self.0))
    }
}


