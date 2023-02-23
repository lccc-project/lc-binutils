use std::{
    io::{Error as IOError, ErrorKind},
    path::{Path, PathBuf},
};

use target_tuples::Target;

use crate::{output::OutputType, targ::TargetInfo};

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]

pub enum LibraryType {
    Static,
    Dynamic,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]

pub struct InputStatus {
    prefer_mode: LibraryType,
    allow_static: bool,
    allow_dynamic: bool,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum InputSet {
    Single(PathBuf),
    Group(Vec<InputSet>),
    Library(String),
    LinkStatic,
    LinkDynamic,
    AsNeeded,
    NoAsNeeded,
}

fn find_library<P: AsRef<Path>>(
    lib: &str,
    search_dirs: impl IntoIterator<Item = P>,
    info: &TargetInfo,
    status: &InputStatus,
) -> std::io::Result<PathBuf> {
    let mut dynlib_paths = Vec::new();
    let mut staticlib_paths = Vec::new();
    for dir in search_dirs {
        if status.allow_dynamic {
            for suffix in info.dynamicsuffixes {
                let file = info.libprefix.to_string() + lib + *suffix;
                let mut path = dir.as_ref().to_path_buf();
                path.push(file);
                match std::fs::metadata(&path) {
                    Ok(_) => dynlib_paths.push(path),
                    Err(e) if e.kind() == ErrorKind::NotFound => {}
                    Err(e) => Err(e)?,
                }
            }
        }
        if status.allow_static {
            for suffix in info.staticsuffixes {
                let file = info.libprefix.to_string() + lib + *suffix;
                let mut path = dir.as_ref().to_path_buf();
                path.push(file);
                match std::fs::metadata(&path) {
                    Ok(_) => staticlib_paths.push(path),
                    Err(e) if e.kind() == ErrorKind::NotFound => {}
                    Err(e) => Err(e)?,
                }
            }
        }
    }
    match (
        status.prefer_mode,
        staticlib_paths.is_empty(),
        dynlib_paths.is_empty(),
    ) {
        (LibraryType::Dynamic, _, false) | (_, true, false) => {
            Ok(dynlib_paths.into_iter().next().unwrap())
        }
        (LibraryType::Static, false, _) | (_, false, true) => {
            Ok(staticlib_paths.into_iter().next().unwrap())
        }
        (_, true, true) => Err(std::io::Error::new(
            ErrorKind::NotFound,
            format!("Library {} not found", lib),
        )),
    }
}

fn print_input_and_ident<'a, R, P: AsRef<Path>>(
    p: &InputSet,
    search_dirs: &'a R,
    info: &TargetInfo,
    status: &mut InputStatus,
) -> std::io::Result<()>
where
    &'a R: IntoIterator<Item = P>,
{
    match p {
        InputSet::Single(path) => {
            let ty = crate::input::ident_input(path)?;
            eprint!("{} {{{}}}", path.display(), ty);
        }
        InputSet::Group(inputs) => {
            let mut sep = "(";
            for input in inputs {
                eprint!("{}", sep);
                sep = " ";
                print_input_and_ident(input, search_dirs, info, status)?;
            }
            eprint!(")");
        }
        InputSet::Library(lib) => {
            let file = find_library(lib, search_dirs, info, status)?;
            let ty = crate::input::ident_input(&file)?;
            eprint!("-l{}: {} {{{}}}", lib, file.display(), ty);
        }
        InputSet::LinkStatic => status.prefer_mode = LibraryType::Static,
        InputSet::LinkDynamic => status.prefer_mode = LibraryType::Dynamic,
        _ => {}
    }
    Ok(())
}

#[allow(unused_variables, unused_assignments)]
pub fn main() -> Result<(), IOError> {
    let mut default_targ = target_tuples::from_env!("default_target");

    let mut targ = None::<Target>;

    let mut args = std::env::args();

    let prg_name = args.next().unwrap();

    let mut output_file = "a.out".to_string();
    let mut output_type = None::<OutputType>;
    let mut suppout = None::<String>;

    let mut inputs = Vec::new();

    let mut cur_group = None::<Vec<_>>;

    let mut add_search_dirs = Vec::new();

    if let Some((left, _)) = prg_name.rsplit_once('-') {
        if let Ok(targ) = left.parse() {
            default_targ = targ;
        }
    }

    while let Some(arg) = args.next() {
        match &*arg {
            "--version" => {
                eprintln!("lcld (GNU Driver) v{}", env!("CARGO_PKG_VERSION"));
                eprintln!("Copyright (C) 2020-2022 Lightning Creations");
                eprintln!("This program is released under the terms of the BSD 2 Clause + Patent License.");
                eprintln!("A copy of the license is available in the source code or at https://github.com/LightningCreations/lc-binutils");
                eprintln!();
                eprint!("lcld is compiled with support for the following binary formats: ");
                for fmt in binfmt::formats() {
                    eprint!("{} ", fmt.name());
                }
                eprintln!();
                std::process::exit(0)
            }
            "-shared" => {
                output_type = Some(OutputType::Shared);
            }
            "--target" => {
                targ = Some(args.next().and_then(|s| s.parse().ok()).unwrap_or_else(|| {
                    eprintln!("{}: Expected a target after --target", prg_name);
                    std::process::exit(1)
                }));
            }
            "-o" => {
                output_file = args.next().unwrap_or_else(|| {
                    eprintln!("{}: Expected a file name after -o", prg_name);
                    std::process::exit(1)
                });
            }
            x if x.starts_with("-o") => {
                output_file = x[2..].to_string();
            }
            "-L" => {
                add_search_dirs.push(args.next().unwrap_or_else(|| {
                    eprintln!("{}: Expected a file name after -o", prg_name);
                    std::process::exit(1)
                }));
            }
            x if x.starts_with("-L") => {
                add_search_dirs.push(x[2..].to_string());
            }
            "-l" => {
                if let Some(group) = &mut cur_group {
                    group.push(InputSet::Library(args.next().unwrap_or_else(|| {
                        eprintln!("{}: Expected a library name after -l", prg_name);
                        std::process::exit(1)
                    })))
                } else {
                    inputs.push(InputSet::Library(args.next().unwrap_or_else(|| {
                        eprintln!("{}: Expected a library name after -l", prg_name);
                        std::process::exit(1)
                    })))
                }
            }
            x if x.starts_with("-l") => {
                if let Some(group) = &mut cur_group {
                    group.push(InputSet::Library(x[2..].to_string()))
                } else {
                    inputs.push(InputSet::Library(x[2..].to_string()))
                }
            }
            "--import-name" => {
                suppout = Some(args.next().unwrap_or_else(|| {
                    eprintln!("{}: Expected a file name after --import-library", prg_name);
                    std::process::exit(1)
                }))
            }
            "--start-group" | "-(" => {
                if cur_group.is_some() {
                    eprintln!("{}: Cannot nest groups", prg_name);
                    std::process::exit(1);
                }
                cur_group = Some(Vec::new())
            }
            "--end-group" | "-)" => {
                if let Some(group) = cur_group.take() {
                    inputs.push(InputSet::Group(group));
                } else {
                    eprintln!("{}: No group to end with --end-group", prg_name);
                    std::process::exit(1);
                }
            }
            "--flavour" | "-flavour" | "--flavor" | "-flavor" => {
                args.next(); // consume the argument to flavour, but we're committed on the unix driver now
            }
            x if x.starts_with('-') => todo!("opts"),
            _ => {
                if let Some(group) = &mut cur_group {
                    group.push(InputSet::Single(PathBuf::from(arg)))
                } else {
                    inputs.push(InputSet::Single(PathBuf::from(arg)))
                }
            }
        }
    }

    let targ = if let Some(targ) = targ {
        targ
    } else {
        default_targ
    };

    if inputs.is_empty() {
        eprintln!("{}: Expected at least one input file", prg_name);
        std::process::exit(1)
    }

    let cfg = crate::targ::target_config(&targ);

    let info = &crate::targ::ELF_TARG; // todo, determine target based on the Binfmt

    let mut status = InputStatus {
        prefer_mode: LibraryType::Dynamic,
        allow_static: true,
        allow_dynamic: true,
    };
    let mut search_dirs = Vec::new();
    cfg.search_paths
        .iter()
        .copied()
        .map(Path::new)
        .flat_map(|p| core::iter::repeat(p).zip(cfg.libdirs.iter().copied()))
        .for_each(|pair| {
            if cfg.use_target {
                search_dirs.extend(
                    core::iter::once(pair)
                        .map(|(dir, stem)| {
                            let mut file = dir.to_path_buf();
                            file.push(stem);
                            file
                        })
                        .chain(
                            core::iter::repeat(pair)
                                .zip([
                                    targ.to_string(),
                                    targ.arch().to_string() + "-" + targ.sys(),
                                    targ.get_name().to_string(),
                                ])
                                .map(|((dir, stem), targ)| {
                                    let mut file = dir.to_path_buf();
                                    file.push(targ);
                                    file.push(stem);
                                    file
                                }),
                        )
                        .chain(
                            core::iter::repeat(pair)
                                .zip([
                                    targ.to_string(),
                                    targ.arch().to_string() + "-" + targ.sys(),
                                    targ.get_name().to_string(),
                                ])
                                .map(|((dir, stem), targ)| {
                                    let mut file = dir.to_path_buf();
                                    file.push(stem);
                                    file.push(targ);
                                    file
                                }),
                        ),
                );
            } else {
                search_dirs.extend(core::iter::once(pair).map(|(dir, stem)| {
                    let mut file = dir.to_path_buf();
                    file.push(stem);
                    file
                }));
            }
        });
    search_dirs.extend(add_search_dirs.into_iter().map(PathBuf::from));

    eprintln!("{}: Search Paths: {:?}", prg_name, search_dirs);

    eprint!("{}: Input Files: ", prg_name);
    for input in &inputs {
        print_input_and_ident(input, &search_dirs, info, &mut status)?;
        eprint!(" ");
    }

    eprintln!();

    Ok(())
}
