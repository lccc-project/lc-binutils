use serde::{Deserialize, Deserializer};
use std::io::Read;

use target_tuples::Target;

fn default_target() -> Target {
    Target::parse(&std::env::var("TARGET").unwrap())
}

fn deserialize_target<'de, De: Deserializer<'de>>(de: De) -> Result<Target, De::Error> {
    let str = <&str>::deserialize(de)?;

    Ok(Target::parse(str))
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Targets {
    #[serde(default = "default_target")]
    #[serde(deserialize_with = "deserialize_target")]
    default_target: Target,
}

fn main() {
    let base_dir = std::env!("CARGO_MANIFEST_DIR");

    let mut file = std::path::Path::new(base_dir).to_path_buf();

    file.push("config.toml");

    println!("cargo:rerun-if-changed={}", file.display());

    let mut file = std::fs::File::open(file).unwrap();

    let mut buf = String::with_capacity(file.metadata().map(|m| m.len() as usize).unwrap_or(1024));

    file.read_to_string(&mut buf).unwrap();

    let targets: Targets = toml::from_str(&buf).unwrap();

    println!("cargo:rustc-env=default_target={}", targets.default_target)
}
