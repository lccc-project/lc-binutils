use crate::x86::features::X86Feature;

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub struct X86MachineFromStrError;

impl core::fmt::Display for X86MachineFromStrError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.write_str("Unknown x86 feature")
    }
}

impl std::error::Error for X86MachineFromStrError {}

macro_rules! define_x86_cpus{
    {
        $(($enum:ident, $cpu_type:literal $(| $($extra_names:literal)|*)?, [$($feature:ident),* $(,)?])),* $(,)?
    } => {
        #[derive(Copy,Clone,Hash,PartialEq,Eq)]
        #[non_exhaustive]
        #[repr(i32)]
        pub enum X86Machine{
            $($enum ,)*
        }

        impl X86Machine{
            pub fn cpu_name(&self) -> &'static str{
                match self{
                    $(X86Machine:: $enum => $cpu_type,)*
                }
            }

            pub fn cpu_features(&self) -> &'static [X86Feature]{
                match self{
                    $(Self::$enum => &[$(X86Feature:: $feature),*],)*
                }
            }
        }

        impl core::fmt::Display for X86Machine{
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result{
                match self{
                    $(Self:: $enum => f.write_str($cpu_type),)*
                }
            }
        }

        impl core::str::FromStr for X86Machine{
            type Err = X86MachineFromStrError;

            fn from_str(x: &str) -> Result<Self,Self::Err>{
                match x{
                    $(
                    #[allow(unreachable_patterns)] $cpu_type => Ok(Self:: $enum),
                    $($(#[allow(unreachable_patterns)] $extra_names => Ok(Self:: $enum),)*)?
                    )*
                    _ => Err(X86MachineFromStrError)
                }
            }
        }
    }
}

define_x86_cpus! {
    (I386, "i386", [X87]),
    (I486, "i486", [X87]),
    (I586, "i587", [X87]),
    (Lakemont,"lakemont",[X87]),
    (PentiumMmx, "pentium-mmx", [X87,Mmx]),
    (PentiumPro, "pentiumpro" | "i686", [X87]),
    (Pentium2, "pentium2", [X87, Mmx, Fxsr]),
    (Pentium3, "pentium3", [X87, Mmx, Fxsr, Sse]),
    (Pentium3m, "pentium3m", [X87, Mmx, Fxsr, Sse]),
    (PentiumM, "pentium-m", [X87, Mmx, Fxsr, Sse, Sse2]),
    (Pentium4, "pentium4" | "pentium4m", [X87, Mmx, Fxsr, Sse,Sse2]),
    (Prescott, "prescott", [X87, Mmx, Fxsr, Sse, Sse2, Sse3]),
    (X86_64, "x86-64", [X87, Mmx, Fxsr, Sse, Sse2, Cx8]),
    (Nocona, "nocona", [X87, Mmx, Fxsr, Sse, Sse2, Cx8, Sse3]),
    (Core2, "core2", [X87, Mmx, Fxsr, Sse, Sse2, Sse3, Ssse3, Cx8, Cx16]),
    (Nehalem, "nehalem", [X87, Mmx, Fxsr, Sse, Sse2, Sse3, Sse4, Sse4_1, Sse4_2, Popcnt, Cx8, Cx16, Sahf]),
    (Westmere, "westmere", [X87, Mmx, Fxsr, Sse, Sse2, Sse3, Ssse3, Sse4, Sse4_1, Sse4_2, Popcnt, Cx16, Pclmul]),
    (SandyBridge, "sandybridge", [X87, Mmx, Fxsr, Sse, Sse2, Sse3, Ssse3, Sse4, Sse4_1, Sse4_2, Popcnt, Cx8, Cx16,Sahf, Avx, XSave, Pclmul]),
    (IvyBridge, "ivybridge", [X87, Mmx, Fxsr, Sse, Sse2, Sse3, Ssse3, Sse4, Sse4_1, Sse4_2, Popcnt, Cx8, Cx16,Sahf, Avx, XSave, Pclmul, FsGsBase, Rdrand, F16c]),
    (Haswell, "haswell", [X87, Mmx, Fxsr, Sse, Sse2, Sse3, Ssse3, Sse4, Sse4_1, Sse4_2, Popcnt, Cx8, Cx16,Sahf, Avx, XSave, Pclmul, FsGsBase,
         Rdrand, F16c, Avx2, Bmi, Bmi2, Lzcnt, Fma, MovBe, Hle]),
    (Broadwell, "broadwell",[X87, Mmx, Fxsr, Sse, Sse2, Sse3, Ssse3, Sse4, Sse4_1, Sse4_2, Popcnt, Cx8, Cx16,Sahf, Avx, XSave, Pclmul, FsGsBase,
         Rdrand, F16c, Avx2, Bmi, Bmi2, Lzcnt, Fma, MovBe, Hle, Rdseed, Adcx, PrefecthW]),
    (Skylake, "skylake",[X87, Mmx, Fxsr, Sse, Sse2, Sse3, Ssse3, Sse4, Sse4_1, Sse4_2, Popcnt, Cx8, Cx16,Sahf, Avx, XSave, Pclmul, FsGsBase,
         Rdrand, F16c, Avx2, Bmi, Bmi2, Lzcnt, Fma, MovBe, Hle, Rdseed, Adcx, PrefecthW, Aes, ClFlushOpt, XSaveC, XSaveS, Sgx]),
    (Bonnell, "bonnell", [X87, Mmx, Sse, Sse2, Sse3, Ssse3,MovBe]),
    (Silvermont, "silvermont", [X87, Mmx, Sse, Sse2, Sse3, Ssse3, Sse4, Sse4_1, Sse4_2, Popcnt, Cx8, Cx16, Sahf, Fxsr, Pclmul, PrefecthW, Rdrand]),
    (Goldmont, "goldmont", [X87, Mmx, Sse, Sse2, Sse3, Ssse3, Sse4, Sse4_1, Sse4_2, Popcnt, Cx8, Cx16, Sahf, Fxsr, Pclmul, PrefecthW, Rdrand, Aes, Sha, Rdseed,
        XSave, XSaveC, XSaveS, XSaveOpt, ClFlushOpt, FsGsBase]),

}

pub mod timings;
