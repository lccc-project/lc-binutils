use std::{
    fs::File,
    io::{Seek, SeekFrom},
};

fn main() -> std::io::Result<()> {
    let mut args = std::env::args();
    args.next().unwrap();
    let fname = args.next().unwrap();

    let mut file = File::open(&fname)?;

    let binfile = binfmt::open_file(file)?;

    println!("File information: {}", fname);
    println!("Format: {}", binfile.fmt().name());
    println!("File Type: {:?}", binfile.file_type());
    print!("Sections: [");
    let mut sep = "";
    for s in binfile.sections() {
        print!("{}{}", sep, s.name);
        sep = ", ";
    }
    println!("]");
    print!("Symbols: [");
    let mut sep = "";
    for s in binfile.symbols() {
        print!("{}{}", sep, s.name());
        sep = ", ";
    }
    println!("]");

    Ok(())
}
