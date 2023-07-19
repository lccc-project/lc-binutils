use std::ffi::OsStr;

use super::Archive;

#[test]
#[ignore] // broken - fix later
pub fn archive() {
    let bytes: &[u8] = include_bytes!("test_archive1.a");
    let archive = Archive::read(bytes).unwrap();
    let members = archive.members();
    assert_eq!(members.len(), 1);
    let m1 = &members[0];
    assert_eq!(m1.get_name(), OsStr::new("empty_rel.o"));
}
