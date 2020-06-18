
use std::io::{Error as IOError, ErrorKind};

pub fn main() -> Result<(),IOError>{

    Err(IOError::new(ErrorKind::Other,"Incomplete Driver, ld support is Work-In-Progress"))
}