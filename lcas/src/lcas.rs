use arch_ops::traits::InsnWrite;
use binfmt::{
    fmt::{BinaryFile, FileType, Section, SectionType},
    sym::{Symbol, SymbolKind},
};
use lcas_core::{
    as_state::{Assembler, AssemblerCallbacks},
    expr::Expression,
    lex::Token,
};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    fs::File,
    io::{Read, Write},
    ptr::NonNull,
    rc::Rc,
};
use target_tuples::Target;

pub struct Data {
    binfile: BinaryFile<'static>,
    sections: HashMap<String, Rc<RefCell<Section>>>,
    curr_section: String,
    syms: HashMap<String, (String, usize)>,
    global_syms: HashSet<String>,
    weak_syms: HashSet<String>,
}

pub struct SharedSection(Rc<RefCell<Section>>);

impl Write for SharedSection {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.borrow_mut().write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0.borrow_mut().flush()
    }
}

impl InsnWrite for SharedSection {
    fn offset(&self) -> usize {
        self.0.borrow().offset()
    }

    fn write_addr(
        &mut self,
        size: usize,
        addr: arch_ops::traits::Address,
        rel: bool,
    ) -> std::io::Result<()> {
        self.0.borrow_mut().write_addr(size, addr, rel)
    }

    fn write_reloc(&mut self, reloc: arch_ops::traits::Reloc) -> std::io::Result<()> {
        self.0.borrow_mut().write_reloc(reloc)
    }
}

pub struct Callbacks;

impl AssemblerCallbacks for Callbacks {
    fn handle_directive(&self, asm: &mut Assembler, dir: &str) -> std::io::Result<()> {
        match dir {
            ".text" | ".rodata" | ".data" | ".bss" => {
                let data = asm.as_data_mut().downcast_mut::<Data>().unwrap();
                let sect = if let Some(sect) = data.sections.get(dir) {
                    sect.clone()
                } else {
                    let ty = if dir == ".bss" {
                        SectionType::NoBits
                    } else {
                        SectionType::ProgBits
                    };
                    let sect = Section {
                        name: dir.to_string(),
                        align: asm.machine().def_section_alignment() as usize,
                        ty,
                        ..Default::default()
                    };
                    let data = asm.as_data_mut().downcast_mut::<Data>().unwrap();

                    let sect = Rc::new(RefCell::new(sect));

                    data.sections.insert(dir.to_string(), sect.clone());

                    sect
                };

                let data = asm.as_data_mut().downcast_mut::<Data>().unwrap();

                data.curr_section = dir.to_string();

                asm.set_output(Box::new(SharedSection(sect)));

                Ok(())
            }
            ".section" => match asm.iter().next().unwrap() {
                Token::Identifier(tok) => {
                    let data = asm.as_data_mut().downcast_mut::<Data>().unwrap();
                    let sect = if let Some(sect) = data.sections.get(&tok) {
                        sect.clone()
                    } else {
                        let sect = Section {
                            name: tok.clone(),
                            align: asm.machine().def_section_alignment() as usize,
                            ty: SectionType::ProgBits,
                            ..Default::default()
                        };
                        let data = asm.as_data_mut().downcast_mut::<Data>().unwrap();

                        let sect = Rc::new(RefCell::new(sect));

                        data.sections.insert(tok.clone(), sect.clone());

                        sect
                    };

                    let data = asm.as_data_mut().downcast_mut::<Data>().unwrap();

                    data.curr_section = tok;

                    asm.set_output(Box::new(SharedSection(sect)));

                    Ok(())
                }
                tok => panic!(
                    "Invalid token after .section: Exception an identifier {:?}",
                    tok
                ),
            },
            ".global" | ".globl" => {
                loop {
                    match asm.iter().next().unwrap() {
                        Token::Identifier(id) => {
                            let data = asm.as_data_mut().downcast_mut::<Data>().unwrap();
                            data.global_syms.insert(id);
                        }
                        Token::LineTerminator => break,
                        tok => panic!(
                            "Unexpected token for .global directive: {:?}, expected an identifier",
                            tok
                        ),
                    }
                }

                Ok(())
            }
            ".weak" => {
                loop {
                    match asm.iter().next().unwrap() {
                        Token::Identifier(id) => {
                            let data = asm.as_data_mut().downcast_mut::<Data>().unwrap();
                            data.weak_syms.insert(id);
                        }
                        Token::LineTerminator => break,
                        tok => panic!(
                            "Unexpected token for .weak directive: {:?}, expected an identifier",
                            tok
                        ),
                    }
                }

                Ok(())
            }
            ".align" => {
                let expr = lcas_core::expr::parse_expression(asm.iter());
                let expr = asm.eval_expr(expr);

                match expr {
                    Expression::Integer(mut i) => {
                        let align = i as usize;

                        let data = asm.as_data_mut().downcast_mut::<Data>().unwrap();

                        let mut sec = data.sections[&data.curr_section].borrow_mut();

                        if sec.align < align {
                            sec.align = align;
                        }

                        let off = sec.offset();

                        let nlen = (off + (align - 1)) & !(align - 1);

                        sec.content.resize(nlen, 0);
                        Ok(())
                    }
                    expr => panic!("Invalid expression for .space: {:?}", expr),
                }
            }
            x => todo!("Unrecognized directive {}", dir),
        }
    }

    fn create_symbol_now(&self, asm: &mut Assembler, sym: &str) {
        let data = asm.as_data_mut().downcast_mut::<Data>().unwrap();

        let sec = &data.curr_section;

        let offset = data.sections[sec].borrow().offset();
        if data
            .syms
            .insert(sym.to_string(), (sec.clone(), offset))
            .is_some()
        {
            panic!("Duplicate label {}", sym)
        }
    }
}

fn main() {
    let deftarg = target_tuples::from_env!("default_target");
    let mut targ = None;
    let mut binfmt = None;
    let mut input_files = Vec::new();
    let mut output_name = "a.out".to_string();

    let mut args = std::env::args().map(|s| {
        eprint!("{} ", s);
        s
    });

    let prg_name = args.next().unwrap();

    if let Some(pos) = prg_name.rfind("-") {
        let begin = &prg_name[..pos];

        targ = begin.parse::<Target>().ok();
    }

    while let Some(arg) = args.next() {
        match &*arg {
            "--" => {
                input_files.extend(args);
                break;
            }
            "--target" => {
                targ = Some(Target::parse(&args.next().unwrap()));
            }
            x if x.starts_with("--target=") => {
                let t = &x[9..];
                targ = Some(Target::parse(t));
            }
            "--output-fmt" => {
                binfmt = Some(args.next().unwrap());
            }
            x if x.starts_with("--output-fmt=") => {
                let t = &x[13..];
                binfmt = Some(t.to_string());
            }
            "--version" => {
                eprintln!("lcas v{}", std::env!("CARGO_PKG_VERSION"));
                eprintln!("Copyright (c) 2022 Lightning Creations");
                eprintln!("Released under the terms of the BSD 2 Clause + Patent License");
                eprintln!();

                eprint!("lcas is compiled with support for the following binfmts: ");

                let mut sep = "";

                for i in binfmt::formats() {
                    eprint!("{}{}", sep, i.name());
                    sep = ", ";
                }

                eprintln!();

                std::process::exit(0);
            }
            "--help" => {
                eprintln!("USAGE: {} [OPTIONS] [--] [input files]..", prg_name);
                eprintln!("Assembles give assembly source files into binary files");
                eprintln!("Options:");
                eprintln!(
                    "\t--target <target>: Specify the target to assemble for (default {})",
                    deftarg
                );
                eprintln!(
                    "\t--output-fmt <binfmt>: Specify the output format (default {})",
                    binfmt::def_vec_for(targ.as_ref().unwrap_or(&deftarg)).name()
                );

                eprint!("lcas is compiled with support for the following binfmts: ");

                let mut sep = "";

                for i in binfmt::formats() {
                    eprint!("{}{}", sep, i.name());
                    sep = ", ";
                }

                eprintln!();

                std::process::exit(0);
            }
            "--output-file" | "-o" => {
                output_name = args.next().unwrap();
            }
            x if x.starts_with("--output-file=") => {
                let t = &x[14..];
                output_name = t.to_string();
            }
            x if x.starts_with("-o") => {
                let t = &x[2..];
                output_name = t.to_string();
            }
            x if x.starts_with("-") => {
                eprintln!("Unrecognized option: {}", x);

                std::process::exit(1);
            }

            x => {
                input_files.push(x.to_string());
                input_files.extend(args);
                break;
            }
        }
    }

    eprintln!();

    let targ = match targ {
        Some(targ) => targ,
        None => deftarg,
    };

    eprintln!("Targetting: {}", targ);

    let binfmt = if let Some(fmt) = binfmt {
        binfmt::format_by_name(&fmt).unwrap_or_else(|| {
            eprintln!("Unknown or invalid binfmt name {}", fmt);

            std::process::exit(1)
        })
    } else {
        binfmt::def_vec_for(&targ)
    };

    if input_files.is_empty() {
        eprintln!("At least one input file must be specified");
        std::process::exit(1)
    }

    let targ_def = lcas_core::targ::get_target_def(targ.arch()).unwrap_or_else(|| {
        eprintln!("Unknown target {}", targ);
        std::process::exit(1)
    });

    let mut input = utf::decode_utf8(
        input_files
            .into_iter()
            .map(|s| {
                std::fs::File::open(&s).unwrap_or_else(|e| {
                    eprintln!("Unable to open input file {}: {}", s, e);

                    std::process::exit(1)
                })
            })
            .flat_map(|s| s.bytes())
            .map(|r| {
                r.unwrap_or_else(|e| {
                    eprintln!("Failed to read input file: {}", e);

                    std::process::exit(1)
                })
            }),
    )
    .map(|e| e.unwrap())
    .peekable();

    let mut lex = lcas_core::lex::Lexer::new(targ_def, &mut input);

    let text = Section {
        name: ".text".to_string(),
        align: targ_def.def_section_alignment() as usize,
        ..Default::default()
    };

    let text = Rc::new(RefCell::new(text));

    let mut sections = HashMap::new();
    sections.insert(".text".to_string(), text.clone());

    let binfile = binfmt.create_file(FileType::Relocatable);

    let mut data = Data {
        binfile,
        sections,
        curr_section: ".text".to_string(),
        syms: HashMap::new(),
        global_syms: HashSet::new(),
        weak_syms: HashSet::new(),
    };

    let toks = lex.collect::<Vec<_>>();

    let mut iter = toks.into_iter();

    let mut asm = Assembler::new(
        targ_def,
        Box::new(SharedSection(text)),
        Box::new(data),
        &Callbacks,
        &mut iter,
    );

    while let Some(res) = asm.assemble_instr() {
        if let Err(e) = res {
            eprintln!("Failed to assemble: {}", e);
            std::process::exit(1)
        }
    }

    let data = asm.as_data_mut().downcast_mut::<Data>().unwrap();

    let binfile = &mut data.binfile;

    let mut secnos = HashMap::new();

    for (name, sec) in &data.sections {
        let section = core::mem::take(&mut *sec.borrow_mut());

        let no = binfile.add_section(section).unwrap();

        secnos.insert(name.clone(), no);
    }

    for (name, (sec, offset)) in &data.syms {
        let sec = secnos[sec];
        let sym = Symbol::new(
            name.clone(),
            Some(sec),
            Some(*offset as u128),
            binfmt::sym::SymbolType::Object,
            if data.global_syms.contains(name) {
                SymbolKind::Global
            } else {
                SymbolKind::Local
            },
        );

        *binfile.get_or_create_symbol(name).unwrap() = sym;
    }

    for name in &data.weak_syms {
        *binfile.get_or_create_symbol(name).unwrap().kind_mut() = SymbolKind::Weak;
    }

    let mut output = File::create(output_name).unwrap();

    binfmt.write_file(&mut output, binfile).unwrap();
}
