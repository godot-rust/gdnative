extern crate gdnative_bindings_generator;

use gdnative_bindings_generator::generate_bindings;
use std::path::PathBuf;
use std::env;
use std::fs::File;

fn main() {
    let in_path = env::args().nth(1).unwrap();
    let out_path = env::args().nth(2).unwrap();

    let mut output = File::create(&out_path).unwrap();

    generate_bindings(
        File::open(&in_path).unwrap(),
        &mut output,
    );
}
