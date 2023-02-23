#![deny(warnings)]

use std::io::{Error as IOError, ErrorKind};

pub mod driver;
pub mod input;
pub mod link;
pub mod lto;
pub mod output;
pub mod script;
pub mod targ;

pub enum Mode {
    Unix,
    Darwin,
    Windows,
    Wasm,
}

#[allow(clippy::single_match)] // May get more arms added in the future
fn main() {
    let mut args = std::env::args();

    let mut exe_name = args.next().unwrap();

    if exe_name.ends_with(".exe") {
        exe_name.truncate(exe_name.len() - 4);
    }

    let mut driver = match &*exe_name {
        x if x.ends_with("ld") || x.ends_with("ld.lc") => Some(Mode::Unix),
        x if x.ends_with("ld64") || x.ends_with("ld64.lc") => Some(Mode::Darwin),
        x if x.ends_with("link") => Some(Mode::Windows),
        _ => None,
    };

    match args.next().as_deref() {
        Some(x @ ("-flavor" | "-flavour" | "--flavor" | "--flavour" | "/flavour")) => {
            driver = match args.next().as_deref().unwrap_or_else(|| {
                eprintln!("{}: Expected an argument for {}", exe_name, x);
                std::process::exit(1)
            }) {
                "ld" | "ld.lc" | "gnu" | "unix" => Some(Mode::Unix),
                "ld64" | "ld64.lc" | "darwin" => Some(Mode::Darwin),
                "link" | "lc-link" | "windows" => Some(Mode::Windows),
                "wasm" | "wasm-ld" | "wasm-ld.lc" => Some(Mode::Wasm),
                x => {
                    eprintln!("{}: Unknown flavor {}", exe_name, x);
                    std::process::exit(1)
                }
            }
        }
        _ => {}
    }

    match driver{
        Some(Mode::Unix) => {
            driver::ld::main()
        },
        Some(Mode::Darwin) => {
            driver::ld64::main()
        },
        Some(Mode::Windows) => {
            driver::link::main()
        },
        Some(Mode::Wasm) => {
            driver::wasm::main()
        }
        None => {
            Err(IOError::new(ErrorKind::Other,"Generic driver for lcld. Run ld.lc (unix), ld64.lc (MacOS), or lc-link (Windows) for an actual driver"))
        }
    }.unwrap_or_else(|e| {
        println!("{}: {}",exe_name, e);
        std::process::exit(1);
    })
}
