use target_tuples::Target;

fn main() {
    let mut args = std::env::args();

    let prg_name = args.next().unwrap();

    let mut target = None::<Target>;
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
                    "\t--target <target>: Specify the target to use for the disassembler (default detected)"
                );
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
}
