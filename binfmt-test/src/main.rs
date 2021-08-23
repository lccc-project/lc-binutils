fn main() {
    for x in binfmt::formats() {
        println!("{}", x.name())
    }
}
