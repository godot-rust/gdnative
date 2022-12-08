//#![allow(unused_variables, dead_code)]

use regex::Regex;
use std::error::Error;

pub struct GodotVersion {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,         //< 0 if none
    pub stability: String, // stable|beta|dev
}

pub fn parse_godot_version(version_str: &str) -> Result<GodotVersion, Box<dyn Error>> {
    let regex = Regex::new("(\\d+)\\.(\\d+)(?:\\.(\\d+))?\\.(stable|beta|dev)")?;

    let caps = regex.captures(version_str).ok_or("Regex capture failed")?;

    let fail = || format!("Version substring could not be matched in '{version_str}'");

    Ok(GodotVersion {
        major: caps.get(1).ok_or_else(fail)?.as_str().parse::<u8>()?,
        minor: caps.get(2).ok_or_else(fail)?.as_str().parse::<u8>()?,
        patch: caps
            .get(3)
            .map(|m| m.as_str().parse::<u8>())
            .transpose()?
            .unwrap_or(0),
        stability: caps.get(4).ok_or_else(fail)?.as_str().to_string(),
    })
}

#[test]
fn test_godot_versions() {
    let good_versions = [
        ("3.0.stable.official", 3, 0, 0, "stable"),
        ("3.0.1.stable.official", 3, 0, 1, "stable"),
        ("3.2.stable.official", 3, 2, 0, "stable"),
        ("3.37.stable.official", 3, 37, 0, "stable"),
        ("3.4.stable.official.206ba70f4", 3, 4, 0, "stable"),
        ("3.4.1.stable.official.aa1b95889", 3, 4, 1, "stable"),
        ("3.5.beta.custom_build.837f2c5f8", 3, 5, 0, "beta"),
        ("4.0.dev.custom_build.e7e9e663b", 4, 0, 0, "dev"),
    ];

    let bad_versions = [
        "4.0.unstable.custom_build.e7e9e663b", // "unstable"
        "4.0.3.custom_build.e7e9e663b",        // no stability
        "3.stable.official.206ba70f4",         // no minor
    ];

    // From Rust 1.56: 'for (...) in good_versions'
    for (full, major, minor, patch, stability) in good_versions {
        let parsed: GodotVersion = parse_godot_version(full).unwrap();
        assert_eq!(parsed.major, major);
        assert_eq!(parsed.minor, minor);
        assert_eq!(parsed.patch, patch);
        assert_eq!(parsed.stability, stability);
    }

    for full in bad_versions {
        let parsed = parse_godot_version(full);
        assert!(parsed.is_err());
    }
}
