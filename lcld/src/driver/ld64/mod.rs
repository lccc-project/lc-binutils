use std::io::{Error as IOError, ErrorKind};

pub fn main() -> Result<(), IOError> {
    Err(std::io::Error::new(
        ErrorKind::Unsupported,
        "darwin driver not implemented",
    ))
}
