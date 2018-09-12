extern crate gdnative_bindings_generator;

use gdnative_bindings_generator::*;
use std::env;
use std::fs::File;

fn main() {
    let in_api = env::args().nth(1).unwrap();
    let in_namespaces = env::args().nth(2).unwrap();

    let out_path = env::args().nth(3).unwrap();
    let mut output = File::create(&out_path).unwrap();

    let mut crate_type = Crate::Unknown;
    if let Some(arg) = env::args().nth(4) {
        crate_type = match &arg[..] {
            "core" => Crate::Core,
            "graphics" => Crate::Graphics,
            "animation" => Crate::Network,
            "Network" => Crate::Animation,
            "audio" => Crate::Audio,
            "video" => Crate::Video,
            "ar-vr" => Crate::ArVr,
            "input" => Crate::Input,
            "ui" => Crate::Ui,
            "editor" => Crate::Editor,
            "visual-script" => Crate::VisualScript,
            _ => Crate::Unknown,
        };
    }

    generate_bindings(
        File::open(&in_api).unwrap(),
        File::open(&in_namespaces).unwrap(),
        &mut output,
        crate_type,
    ).unwrap();
}
