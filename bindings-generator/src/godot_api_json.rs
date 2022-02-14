use crate::godot_version;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub fn generate_json_if_needed() -> bool {
    let godot_bin: PathBuf = if let Ok(string) = std::env::var("GODOT_BIN") {
        println!("Found GODOT_BIN with path to executable: '{}'", string);
        PathBuf::from(string)
    } else if let Ok(path) = which::which("godot") {
        println!("Found 'godot' executable in PATH: {}", path.display());
        path
    } else {
        panic!(
            "Feature 'custom-godot' requires an accessible 'godot' executable or \
             a GODOT_BIN environment variable (with the path to the executable)."
        );
    };

    let version = exec(1, Command::new(&godot_bin).arg("--version"));

    let has_generate_bug = match godot_version::parse_godot_version(&version) {
        Ok(parsed) => {
            assert!(
                parsed.major == 3 && parsed.minor >= 2,
                "Only Godot versions >= 3.2 and < 4.0 are supported; found version {}.",
                version.trim()
            );

            // bug for versions < 3.3.1
            parsed.major == 2 || parsed.major == 3 && parsed.minor == 0
        }
        Err(e) => {
            // Don't treat this as fatal error
            eprintln!("Warning, failed to parse version: {}", e);
            true // version not known, conservatively assume bug
        }
    };

    // Workaround for Godot bug, where the generate command crashes the engine.
    // Try 10 times (should be reasonably high confidence that at least 1 run succeeds).
    println!("Found Godot version < 3.3.1 with potential generate bug; trying multiple times...");

    exec(
        if has_generate_bug { 10 } else { 1 },
        Command::new(&godot_bin)
            .arg("--no-window")
            .arg("--gdnative-generate-json-api")
            .arg("api.json"),
    );

    // Best effort for generating the XML documentation
    // Note: this documentation is not the same as shipped by official Godot releases, but it's the best we have.
    exec_maybe(
        1,
        Command::new(&godot_bin)
            .arg("--no-window")
            .arg("--doctool")
            .arg("."),
    );

    // Godot output structure           required structure
    //
    // +-- api.json
    // +-- modules                      +-- api.json
    // +-- doc                          +-- docs [*]
    //     +-- classes [*]
    //
    let _ = fs::remove_dir_all("docs");
    let _ = fs::rename("doc/classes", "docs");
    let _ = fs::remove_dir("doc");
    let _ = fs::remove_dir_all("modules");

    true
}

/// Executes a command and returns stdout. Panics on failure.
fn exec(attempts: i32, command: &mut Command) -> String {
    exec_maybe(attempts, command)
        .unwrap_or_else(|| panic!("Could not execute Godot command (see above)."))
}

fn exec_maybe(attempts: i32, command: &mut Command) -> Option<String> {
    let command_line = format!("{:?}", command);

    for _attempt in 0..attempts {
        match command.output() {
            Ok(output) => {
                return Some(String::from_utf8(output.stdout).expect("parse UTF8 string"))
            }
            Err(err) => {
                eprintln!(
                    "Godot command failed:\n  command: {}\n  error: {}",
                    command_line, err
                )
            }
        }
    }

    None
}
