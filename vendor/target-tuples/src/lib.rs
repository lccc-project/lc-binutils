#![deny(warnings, unsafe_code)]

mod pieces;

pub use pieces::*;

#[cfg(test)]
pub mod test {
    use crate::Architecture;

    #[test]
    pub fn x86_name_i386() {
        let x86 = Architecture::X86;
        let i386: Architecture = "i386".parse().unwrap();
        assert_eq!(x86, i386);
    }

    #[test]
    pub fn x86_name_i486() {
        let x86 = Architecture::X86;
        let i486: Architecture = "i486".parse().unwrap();
        assert_eq!(x86, i486);
    }
    #[test]
    pub fn x86_name_i586() {
        let x86 = Architecture::X86;
        let i586: Architecture = "i586".parse().unwrap();
        assert_eq!(x86, i586);
    }

    #[test]
    pub fn x86_name_i686() {
        let x86 = Architecture::X86;
        let i386: Architecture = "i686".parse().unwrap();
        assert_eq!(x86, i386);
    }

    #[test]
    pub fn x86_name_i786() {
        let x86 = Architecture::X86;
        let i386: Architecture = "i786".parse().unwrap();
        assert_eq!(x86, i386);
    }
    #[test]
    pub fn x86_name_i886() {
        let x86 = Architecture::X86;
        let i386: Architecture = "i886".parse().unwrap();
        assert_eq!(x86, i386);
    }
    #[test]
    pub fn x86_name_i986() {
        let x86 = Architecture::X86;
        let i386: Architecture = "i986".parse().unwrap();
        assert_eq!(x86, i386);
    }

    #[test]
    pub fn x86_name_canonical() {
        assert_eq!(Architecture::X86.canonical_name(), "i386");
    }
}
