pub trait LtoProvider {
    fn name(&self) -> &'static str;
}

impl core::fmt::Debug for dyn LtoProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

impl core::cmp::PartialEq for dyn LtoProvider {
    fn eq(&self, other: &Self) -> bool {
        core::ptr::eq(
            self as *const _ as *const u8,
            other as *const _ as *const u8,
        )
    }
}

impl core::cmp::Eq for dyn LtoProvider {}

impl core::hash::Hash for dyn LtoProvider {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::ptr::hash(self as *const _ as *const u8, state)
    }
}
