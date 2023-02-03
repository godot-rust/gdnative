use semver::{BuildMetadata, Prerelease};

use crate::core_types::Dictionary;
use crate::core_types::FromVariant;
use crate::object::ownership::Unique;
use crate::private::{get_api, EngineMethodTable};

/// Checks the version number of the host Godot instance to see if it matches the generated API.
/// Returns `true` if the test isn't applicable, or if no mismatch was found.
#[inline]
pub fn godot_version_mismatch() -> bool {
    let ret = check_godot_version_mismatch();
    if !ret {
        godot_warn!(concat!(
            "gdnative-core: GDNative version mismatches may lead to subtle bugs, undefined behavior or crashes at runtime.\n",
            "Apply the 'custom-godot' feature if you want to use current godot-rust with another Godot engine version.",
        ));
    }

    ret
}

#[cfg(feature = "custom-godot")]
fn check_godot_version_mismatch() -> bool {
    true
}

#[cfg(not(feature = "custom-godot"))]
fn check_godot_version_mismatch() -> bool {
    use semver::VersionReq;

    let version = if let Some(version) = godot_version() {
        version
    } else {
        godot_warn!("gdnative-core: failed to get version info from the engine.");
        return false;
    };

    let version_req = VersionReq::parse("~3.5.1").unwrap();

    if version_req.matches(&version) {
        true
    } else {
        godot_warn!("This godot-rust version is only compatible with Godot `{version_req}`; detected version `{version}`.");
        false
    }
}

fn godot_version() -> Option<semver::Version> {
    let version = unsafe {
        let api = get_api();
        let engine = (api.godot_global_get_singleton)(b"Engine\0".as_ptr() as *mut _);

        let mut dictionary = sys::godot_dictionary::default();

        (api.godot_method_bind_ptrcall)(
            EngineMethodTable::get(api).get_version_info,
            engine,
            [].as_mut_ptr() as *mut _,
            &mut dictionary as *mut _ as *mut _,
        );

        Dictionary::<Unique>::from_sys(dictionary)
    };

    let major = u64::from_variant(&version.get("major")?).ok()?;
    let minor = u64::from_variant(&version.get("minor")?).ok()?;
    let patch = u64::from_variant(&version.get("patch")?).ok()?;

    let pre = version
        .get("status")
        .and_then(|v| {
            let s = String::from_variant(&v).ok()?;
            if s == "stable" {
                return None;
            }

            let s = s.chars().map(sanitize_for_semver).collect::<String>();
            Some(Prerelease::new(&s).expect("string sanitized"))
        })
        .unwrap_or(Prerelease::EMPTY);

    let build = {
        let mut build_metadata = String::new();
        let mut sep = false;

        if let Some(build_name) = version
            .get("build")
            .and_then(|v| String::from_variant(&v).ok())
        {
            build_metadata.extend(build_name.chars().map(sanitize_for_semver));
            sep = true;
        };

        if let Some(hash) = version
            .get("hash")
            .and_then(|v| String::from_variant(&v).ok())
        {
            if sep {
                build_metadata.push('.');
            }
            build_metadata.extend(hash.chars().map(sanitize_for_semver));
        };

        if build_metadata.is_empty() {
            BuildMetadata::EMPTY
        } else {
            BuildMetadata::new(&build_metadata).expect("build metadata sanitized")
        }
    };

    Some(semver::Version {
        major,
        minor,
        patch,
        pre,
        build,
    })
}

fn sanitize_for_semver(s: char) -> char {
    if s.is_ascii_alphanumeric() {
        s
    } else {
        '-'
    }
}
