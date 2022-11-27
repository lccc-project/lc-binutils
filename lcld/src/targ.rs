
use target_tuples::Target;

use crate::output::OutputType;

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct TargetConfig<'a>{
    pub search_paths: &'a [&'a str],
    pub libdirs: &'a [&'a str],
    pub use_target: bool,
    pub sysroot: &'a str,
    pub find_msvc: bool,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct TargetInfo<'a>{
    pub objsuffix: &'a str,
    pub libprefix: &'a str,
    pub staticsuffixes: &'a [&'a str],
    pub dynamicsuffixes: &'a [&'a str],
    pub output_dynsuffix: &'a str,
    pub default_output: OutputType,
    pub need_dylib_link: bool,

}

macro_rules! construct_cfg{
    (TargetConfig { paths: PathConfig { search_paths: [$($search_paths:expr),* $(,)?], libdirs: [$($libdirs:expr),* $(,)?], use_target: $usetarget:expr, find_msvc: $find_msvc:expr $(, $_field:ident : $_init:expr)* $(, ..)? }, sysroot: $sysroot:expr $(, $_ofield:ident : $_oinit:expr)* $(, ..)? }) => {
        TargetConfig{
            search_paths: &[$($search_paths),*],
            libdirs: &[$($libdirs),*],
            use_target: $usetarget,
            sysroot: $sysroot,
            find_msvc: $find_msvc,
        }
    }
}

include!(env!("config_targ_generated"));


pub static ELF_TARG: TargetInfo = TargetInfo{
    objsuffix: ".o",
    libprefix: "lib",
    staticsuffixes: &[".a"],
    dynamicsuffixes: &[".so"],
    output_dynsuffix: ".so",
    default_output: OutputType::PieExecutable,
    need_dylib_link: false
};