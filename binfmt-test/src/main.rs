use std::{fs::File, io::Read};

use binfmt::elf::ParsedHeader;

fn main() {
    let mut args = std::env::args();
    let prg_name = args.next().unwrap();

    if let Some(fname) = args.next() {
        match File::open(&fname) {
            Ok(mut file) => {
                let mut bytes = Vec::new();
                if let Err(e) = file.read_to_end(&mut bytes) {
                    eprintln!("Failed to read from {}: {}", &fname, e);
                    std::process::exit(1)
                }
                if let Ok(ehdr) = binfmt::elf::parse_header(&*bytes) {
                    println!("{:#?}", ehdr);
                    match ehdr {
                        ParsedHeader::Elf32(ehdr32) => {
                            println!("{:#?}", ehdr32.get_program_headers(&*bytes))
                        }
                        ParsedHeader::Elf64(ehdr64) => {
                            println!("{:#?}", ehdr64.get_program_headers(&*bytes))
                        }
                    }
                } else {
                    eprintln!("Failed to parse object file, unknown format")
                }
            }
            Err(e) => {
                eprintln!("Failed to open {}: {}", &fname, e);
                std::process::exit(1)
            }
        }
    } else {
        eprintln!("Usage: {} <file>", prg_name);
        std::process::exit(1)
    }
}
