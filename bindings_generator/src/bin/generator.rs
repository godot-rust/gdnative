extern crate gdnative_bindings_generator;

use gdnative_bindings_generator::*;
use std::env;
use std::fs::File;

fn main() {
    let in_path = env::args().nth(1).unwrap();
    let out_path = env::args().nth(2).unwrap();

    let mut output = File::create(&out_path).unwrap();

    let crate_type = if let Some(arg) = env::args().nth(3) {
        Crate::from_str(&arg)
    } else {
        None
    };

    generate_bindings(
        File::open(&in_path).unwrap(),
        &mut output,
        crate_type,
    );
}
