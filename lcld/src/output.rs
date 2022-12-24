#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum OutputType {
    Relocatable,      // perform partial link
    StaticExecutable, // position-dependent executable
    PieExecutable,    // position-independent executable
    Shared,           // Shared object/dll
    SharedAndLink,    // dll+lib
}
