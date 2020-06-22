
#![deny(warnings)]

use std::io::{Error as IOError, ErrorKind};

pub mod arch;
pub mod driver;

pub enum Mode{
    Unix,
    MacOS,
    Windows
}

fn main(){
    let exe_name = std::env::current_exe()
        .and_then(|p|p.file_name()
            .ok_or_else(||IOError::new(ErrorKind::Other,"No executable path was found"))
            .map(|s|String::from(s.to_string_lossy()))
            .map(|s|s.replace(".exe",""))
        ).unwrap_or_else(|e| {
            println!("Error: {}", e);
            std::process::exit(1);
        });

    match if &exe_name == "ld.lc" || &exe_name == "ld" {
        Some(Mode::Unix)
    } else if &exe_name == "ld64.lc" || &exe_name == "ld64" {
        Some(Mode::MacOS)
    } else if &exe_name == "lc-link" || &exe_name == "link" {
        Some(Mode::Windows)
    } else {
        None
    }{
        Some(Mode::Unix) => {
            driver::ld::main()
        },
        Some(Mode::MacOS) => {
            driver::ld64::main()
        },
        Some(Mode::Windows) => {
            driver::link::main()
        },
        None => {
            Err(IOError::new(ErrorKind::Other,"Generic driver for lcld. Run ld.lc (unix), ld64.lc (MacOS), or lc-link (Windows) for an actual driver"))
        }
    }.unwrap_or_else(|e| {
        println!("Error: {}", e);
        std::process::exit(1);
    })
}
