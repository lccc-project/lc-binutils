use std::io::ErrorKind;

pub fn main() -> std::io::Result<()> {
    Err(std::io::Error::new(
        ErrorKind::Unsupported,
        "wasm driver not implemented",
    ))
}
