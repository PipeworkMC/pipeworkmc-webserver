use std::env;


fn main() {
    println!("cargo::rustc-env=CRATE_ROOT={}", env::current_dir().unwrap().display())
}
