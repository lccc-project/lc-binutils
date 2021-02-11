pub fn main() {
    let targets = ["wc65c816"];
    println!("cargo:rerun-if-changed=generator/generic.td");
    for i in &targets {
        println!("cargo:rerun-if-changed=generator/{}.td", i);
    }
}
