extern crate gdnative_bindings_generator;

use gdnative_bindings_generator::*;
use std::env;
use std::fs::File;

fn main() {
    let out_path = env::args().nth(1).unwrap();
    let mut output = File::create(&out_path).unwrap();

    let mut crate_type = Crate::unknown;
    if let Some(arg) = env::args().nth(2) {
        crate_type = match &arg[..] {
            "core" => Crate::core,
            "graphics" => Crate::graphics,
            "animation" => Crate::animation,
            "physics" => Crate::physics,
            "Network" => Crate::animation,
            "audio" => Crate::audio,
            "video" => Crate::video,
            "arvr" => Crate::arvr,
            "input" => Crate::input,
            "ui" => Crate::ui,
            "editor" => Crate::editor,
            "visual_script" => Crate::visual_script,
            _ => Crate::unknown,
        };
    }

    generate_bindings(&mut output, crate_type).unwrap();
}
