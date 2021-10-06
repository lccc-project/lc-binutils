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
        $(($enum:ident, $cpu_type:literal, [$($feature:ident),* $(,)?])),* $(,)?
    } => {
        #[derive(Copy,Clone,Hash,PartialEq,Eq)]
        #[repr(i32)]
        pub enum X86Machine{
            $($enum ,)*

            #[doc(hidden)]
            __Nonexhaustive = -1
        }

        impl X86Machine{
            pub fn cpu_name(&self) -> &'static str{
                match self{
                    $(X86Machine:: $enum => $cpu_type,)*

                    X86Machine::__Nonexhaustive => panic!(),
                }
            }

            pub fn cpu_features(&self) -> &'static [X86Feature]{
                match self{
                    $(Self::$enum => &[$(X86Feature:: $feature),*],)*

                    Self::__Nonexhaustive => panic!(),
                }
            }
        }

        impl core::fmt::Display for X86Machine{
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result{
                match self{
                    $(Self:: $enum => f.write_str($cpu_type),)*

                    Self::__Nonexhaustive => panic!(),
                }
            }
        }

        impl core::str::FromStr for X86Machine{
            type Err = X86MachineFromStrError;

            fn from_str(x: &str) -> Result<Self,Self::Err>{
                match x{
                    $(#[allow(unreachable_patterns)] $cpu_type => Ok(Self:: $enum),)*

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
    (PentiumPro, "pentiumpro", [X87]),
    (Pentium2, "pentium2", [X87, Mmx]),
    (Pentium3, "pentium3", [X87, Mmx, Sse]),
    (Pentium3m, "pentium3m", [X87, Mmx, Sse]),
    (PentiumM, "pentium-m", [X87, Mmx, Sse, Sse2]),
    (Pentium4, "pentium4", [X87, Mmx, Sse,Sse2]),
    (Pentium4m, "penitum4m", [X87, Mmx, Sse, Sse2]),
    (Prescott, "prescott", [X87, Mmx, Sse, Sse2, Sse3]),
    (X86_64, "x86-64", [X87, Mmx, Sse, Sse2, Cx8]),
    (Nocona, "nocona", [X87, Mmx, Sse, Sse2, Sse3]),
    (Core2, "core2", [X87, Mmx, Sse, Sse2, Sse3, Ssse3]),
    (Nehalem, "nehalem", [X87, Mmx, Sse, Sse2, Sse3, Sse4, Sse4_1, Sse4_2, Popcnt]),
    (Westmere, "westmere", [X87, Mmx, Sse2, Sse3, Ssse3, Sse4, Sse4_1, Sse4_2, Popcnt, Aes, Pclmul]),
}
