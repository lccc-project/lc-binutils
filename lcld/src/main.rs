#![deny(warnings)]

use std::io::{Error as IOError, ErrorKind};

pub mod arch;
pub mod driver;

pub enum Mode {
    Unix,
    Darwin,
    Windows,
}

fn main() {
    let exe_name = std::env::current_exe()
        .and_then(|p| {
            p.file_name()
                .ok_or_else(|| IOError::new(ErrorKind::Other, "No executable path was found"))
                .map(|s| String::from(s.to_string_lossy()))
                .map(|s| s.replace(".exe", ""))
        })
        .unwrap_or_else(|e| {
            println!("Error: {}", e);
            std::process::exit(1);
        });

    let driver = match &*exe_name {
        x if x.ends_with("ld") || x.ends_with("ld.lc") => Some(Mode::Unix),
        x if x.ends_with("ld64") || x.ends_with("ld64.lc") => Some(Mode::Darwin),
        x if x.ends_with("link") => Some(Mode::Windows),
        _ => None,
    };

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
        None => {
            Err(IOError::new(ErrorKind::Other,"Generic driver for lcld. Run ld.lc (unix), ld64.lc (MacOS), or lc-link (Windows) for an actual driver"))
        }
    }.unwrap_or_else(|e| {
        println!("Error: {}", e);
        std::process::exit(1);
    })
}
