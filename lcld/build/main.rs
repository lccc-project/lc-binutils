use std::{
    io::{ErrorKind, Read, Write},
    path::PathBuf,
};

mod config;

fn main() -> Result<(), std::io::Error> {
    println!("cargo:rerun-if-changed={}", "config.toml");

    let mut file = std::fs::File::open("config.toml")?;

    let mut str = String::new();
    file.read_to_string(&mut str)?;

    let cfg = toml::from_str::<config::Config>(&str)
        .map_err(|e| std::io::Error::new(ErrorKind::InvalidData, e))?;

    let host = cfg
        .target
        .host_target
        .map(|t| t.0)
        .unwrap_or_else(|| target_tuples::Target::parse(&std::env::var("TARGET").unwrap()));

    let target = cfg
        .target
        .default_target
        .map(|t| t.0)
        .unwrap_or_else(|| host.clone());

    println!("cargo:rustc-env=HOST={}", host);
    println!("cargo:rustc-env=default_target={}", target);

    let mut targ_output = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    targ_output.push("targ-generated.rs");

    let mut file = std::fs::File::create(&targ_output)?;

    writeln!(
        file,
        r#"

    pub fn target_config(targ: &Target) -> TargetConfig<'static>{{
        target_tuples::match_targets!{{
            match (targ){{
    "#
    )?;

    for (id, cfg) in cfg.target.targets {
        writeln!(file, "{} => construct_cfg!({:?}),", id, cfg)?;
    }
    writeln!(file, "* => construct_cfg!({:?})", cfg.target.default_cfg)?;
    writeln!(file, "}}}}}}")?;

    println!(
        "cargo:rustc-env=config_targ_generated={}",
        targ_output.display()
    );

    Ok(())
}
