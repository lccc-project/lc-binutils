#[derive(Debug)]
pub struct X86FeatureFromStrError;

impl core::fmt::Display for X86FeatureFromStrError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.write_str("Unknown x86 feature")
    }
}

impl std::error::Error for X86FeatureFromStrError {}

macro_rules! define_x86_features{
    {
        $(($enum:ident, $feature:literal)),* $(,)?
    } => {
        #[derive(Copy,Clone,Debug,Hash,PartialEq,Eq)]
        #[non_exhaustive]
        #[repr(i32)]
        pub enum X86Feature{
            $($enum,)*
        }

        impl X86Feature{
            pub fn feature_name(&self) -> &'static str{
                match self{
                    $(#[allow(unreachable_patterns)] Self::$enum => $feature,)*
                }
            }
        }

        impl core::str::FromStr for X86Feature{
            type Err = X86FeatureFromStrError;
            fn from_str(x: &str) -> Result<Self,Self::Err>{
                match x{

                    $(#[allow(unreachable_patterns)] $feature => Ok(X86Feature::$enum),)*
                    _ => Err(X86FeatureFromStrError)
                }
            }
        }

        impl core::fmt::Display for X86Feature{
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result{
                match self{
                    $(Self::$enum => f.write_str($feature),)*
                }
            }
        }
    }
}

define_x86_features! {
    (Sce, "sce"),
    (Mmx, "mmx"),
    (Sse, "sse"),
    (Sse2, "sse2"),
    (Sse3, "sse3"),
    (Ssse3, "ssse3"),
    (Sse4, "sse4"),
    (Sse4a, "sse4a"),
    (Sse4_1,"sse4.1"),
    (Sse4_2, "sse4.2"),
    (Avx, "avx"),
    (Avx2,"avx2"),
    (Avx512f,"avx512f"),
    (Avx512pf,"avx512pf"),
    (Avx512er, "avx512er"),
    (Avx512cd, "avx512cd"),
    (Avx512vl, "avx512vl"),
    (Avx512bw, "avx512bw"),
    (Avx512dq,"avx512dq"),
    (Avx512ifma, "avx512ifma"),
    (Avx512vbmi, "avx512vbmi"),
    (Sha, "sha"),
    (Aes, "aes"),
    (Pclmul, "pclmul"),
    (ClFlushOpt, "clflushopt"),
    (Clwb, "clwb"),
    (FsGsBase, "fsgsbase"),
    (Ptwrite, "ptwrite"),
    (Rdrand, "rdrand"),
    (F16c, "f16c"),
    (Fma, "fma"),
    (Pconfig,"pconfig"),
    (Wbnoinvd, "wbnoinvd"),
    (Fma4, "fma4"),
    (Prfchw,"prfchw"),
    (Rdpid, "rdpid"),
    (PrefetchWt11,"prefetchwt11"),
    (Rdseed, "rdseed"),
    (Sgx, "sgx"),
    (Xop, "xop"),
    (Lwp, "lwp"),
    (M3dNow, "3dnow"),
    (M3dNowA, "3dnowa"),
    (Popcnt, "popcnt"),
    (Abm, "abm"),
    (Adx, "adx"),
    (Bmi, "bmi"),
    (Bmi2, "bmi2"),
    (Lzcnt, "lzcnt"),
    (Fxsr, "fxsr"),
    (XSave, "xsave"),
    (XSaveOpt, "xsaveopt"),
    (XSaveC, "xsavec"),
    (XSaveS,"xsaves"),
    (Rtm, "rtm"),
    (Hle, "hle"),
    (Tbm, "tbm"),
    (MWaitX, "mwaitx"),
    (ClZero, "clzero"),
    (Pku, "pku"),
    (Avx512vbmi2, "avx512vbmi2"),
    (Avx512bf16, "avx512bf16"),
    (Avx512fp16, "avx512fp16"),
    (Gfni, "gfni"),
    (Vaes, "vaes"),
    (WaitPkg, "waitpkg"),
    (VpclMulQdq, "vpclmulqdq"),
    (Avx512BitAlg, "avx512bitalg"),
    (MovDirI,"movdiri"),
    (MovDir64b, "movdir64b"),
    (Enqcmd, "enqcmd"),
    (Uintr, "uintr"),
    (Tsxldtrk, "tsxldtrk"),
    (Avx512VPopcntDq, "avx512vpopcntdq"),
    (Avx512Vp2Intersect, "avx512vp2intersect"),
    (Avx5124Fmaps, "avx5124fmaps"),
    (Avx512Vnni, "avx512vnni"),
    (AvxVnni, "avxvnni"),
    (Avx5124VnniW, "avx5124vnniw"),
    (ClDemote, "cldemote"),
    (Serialize, "serialize"),
    (AmxTile, "amx-tile"),
    (AmxInt8, "amx-int8"),
    (AmxBf16, "amx-bf16"),
    (HReset, "hreset"),
    (Kl, "kl"),
    (WideKl, "widekl"),
    (AvxIfma, "avximfa"),
    (AvxVnniInt8,"avxvnniint8"),
    (AvxNeConvert, "avxneconvert"),
    (CmpCxAdd, "cmpcxadd"),
    (AmxFp16, "amx-fp16"),
    (PrefectI, "prefetchi"),
    (RaoInt, "raoint"),
    (AmxComplex, "amx-complex"),
    (AvxVnniInt16, "avxvnniint16"),
    (Sm3, "sm3"),
    (Sha512, "sha512"),
    (Sm4, "sm4"),
    (UserMsr, "usermsr"),
    (X87, "x87"),
    (Cx8, "cx8"),
    (Cx16, "cx16"),
    (Sahf, "sahf"),
    (MovBe, "movbe"),
    (ShStk, "shstk"),
    (Crc32, "crc32"),
    (Mwait, "mwait"),
    (Adcx, "adcx"),
    (PrefecthW, "prefetchw"),
    (ApxF, "apxf"),
    (Avx10, "avx10"),
    (Avx10_128, "avx10-128"),
    (Avx10_256, "avx10-256"),
    (Avx10_512, "avx10-512"),
}
