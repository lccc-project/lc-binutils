use std::fs::File;

use target_tuples::Target;

fn main() {
    let mut args = std::env::args();

    let prg_name = args.next().unwrap();

    let mut binfmt_name = None::<String>;

    let mut input_file = None::<String>;

    while let Some(arg) = args.next(){
        match &*arg{
            "--version" => {
                eprintln!("objdump (lc-binutils {})",std::env!("CARGO_PKG_VERSION"));
                eprintln!("Copyright (c) 2022 Lightning Creations");
                eprintln!("Released under the terms of the BSD 2 Clause + Patent License");
                eprintln!();

                eprint!("objdum is compiled with support for the following binfmts: ");

                let mut sep = "";

                for i in binfmt::formats(){
                    eprint!("{}{}",sep,i.name());
                    sep = ", ";
                }

                eprintln!();

                std::process::exit(0);
            }
            "--help" => {
                eprintln!("USAGE: {} [OPTIONS] [--] [input files]..",prg_name);
                eprintln!("Prints ");
                eprintln!("Options:");
                eprintln!(
                    "\t--input-fmt <binfmt>: Specify the input object format (default detected)",
                );

                eprint!("objdump is compiled with support for the following binfmts: ");

                let mut sep = "";

                for i in binfmt::formats(){
                    eprint!("{}{}",sep,i.name());
                    sep = ", ";
                }

                eprintln!();

                std::process::exit(0);
            }
            x => {
                input_file = Some(arg);
                break;
            }
        }
    }

    let input_file = input_file.unwrap_or_else(||{
        eprintln!("USAGE: {} [OPTIONS] [--] [input files]..",prg_name);
        std::process::exit(1);
    });

    let mut file = File::open(&input_file).unwrap_or_else(|e| {
        eprintln!("{}: Failed to open {}: {}",prg_name,input_file,e);
        std::process::exit(1)
    });
    
    let file = if let Some(binfmt) = &binfmt_name{
        let binfmt = binfmt::format_by_name(binfmt).unwrap_or_else(||{
            eprintln!("Invalid binfmt name {}. Run {} --help for list of supported binfmts",binfmt,prg_name);
            std::process::exit(1)
        });

        binfmt.read_file(&mut file).unwrap_or_else(|e| {
            eprintln!("{}: Failed to read {}: {}",prg_name,input_file,e);
            std::process::exit(1)
        }).unwrap_or_else(||{
            eprintln!("{}: Failed to read {}: Invalid format",prg_name,input_file);
            std::process::exit(1)
        })
    }else{
        binfmt::open_file(file).unwrap_or_else(|e| {
            eprintln!("{}: Failed to read {}: {}",prg_name,input_file,e);
            std::process::exit(1)
        })
    };
    println!("Sections");
    println!();
    println!("        Name            Size      Align");
    for sec in file.sections(){
        println!("{:^20} {:^10} {:^8}",sec.name,sec.content.len(),sec.align);
    }
}
