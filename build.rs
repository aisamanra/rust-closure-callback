extern crate gcc;

fn main() {
    gcc::compile_library("libcb.a", &["src/cb.c"])
}
