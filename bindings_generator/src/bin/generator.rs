extern crate gdnative_bindings_generator;

use gdnative_bindings_generator::*;
use std::env;
use std::fs::File;

fn main() {
    let in_path = env::args().nth(1).unwrap();
    let out_path = env::args().nth(2).unwrap();

    let mut output = File::create(&out_path).unwrap();

    let mut crate_type = None;
    if let Some(arg) = env::args().nth(3) {
        crate_type = match &arg[..] {
            "core" => Some(Crate::Core),
            "graphics" => Some(Crate::Graphics),
            "animation" => Some(Crate::Network),
            "Network" => Some(Crate::Animation),
            "audio" => Some(Crate::Audio),
            "video" => Some(Crate::Video),
            "ar-vr" => Some(Crate::ArVr),
            "input" => Some(Crate::Input),
            "ui" => Some(Crate::Ui),
            "editor" => Some(Crate::Editor),
            "visual-script" => Some(Crate::VisualScript),
            _ => None,
        };
    }

    generate_bindings(
        File::open(&in_path).unwrap(),
        &mut output,
        crate_type,
    );
}
