use std::collections::HashMap;

use unix_path::PathBuf;

use serde::Deserialize;
use target_tuples::Target;

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub struct WrappedTarget(pub Target);

impl<'de> Deserialize<'de> for WrappedTarget {
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let string = <&str as Deserialize>::deserialize(de)?;

        let targ = string
            .parse()
            .map_err(<D::Error as serde::de::Error>::custom)?;

        Ok(WrappedTarget(targ))
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub target: Targets,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Targets {
    pub default_target: Option<WrappedTarget>,
    pub host_target: Option<WrappedTarget>,
    #[serde(rename = "default")]
    pub default_cfg: TargetConfig,
    #[serde(flatten)]
    pub targets: HashMap<String, TargetConfig>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct TargetConfig {
    pub paths: PathConfig,
    #[serde(default)]
    pub sysroot: PathBuf,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct PathConfig {
    pub search_paths: Vec<PathBuf>,
    pub libdirs: Vec<PathBuf>,
    #[serde(default)]
    pub use_target: bool,
    #[serde(default)]
    pub find_msvc: bool,
}
