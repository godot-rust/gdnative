// For silenced lints/warnings, see also gdnative-bindings/src/lib.rs

// Notes:
// * deref_nullptr: since rustc 1.53, bindgen causes UB warnings -- see https://github.com/rust-lang/rust-bindgen/issues/1651
//   remove this once bindgen has fixed the issue (currently at version 1.59.1)
#![allow(
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case,
    improper_ctypes,
    deref_nullptr,
    clippy::style
)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
include!(concat!(env!("OUT_DIR"), "/api_wrapper.rs"));

#[derive(Debug)]
pub enum InitError {
    VersionMismatch {
        api_type: GDNATIVE_API_TYPES,
        want: godot_gdnative_api_version,
        got: godot_gdnative_api_version,
    },
    Generic {
        message: String,
    },
}

fn map_option_to_init_error<T>(t: Option<T>, message: &'static str) -> Result<T, InitError> {
    match t {
        Some(t) => Ok(t),
        None => Err(InitError::Generic {
            message: message.to_string(),
        }),
    }
}

#[allow(clippy::unnecessary_cast)] // False positives: casts necessary for cross-platform
unsafe fn find_version(
    mut api: *const godot_gdnative_api_struct,
    api_type: GDNATIVE_API_TYPES,
    version_major: u32,
    version_minor: u32,
) -> Option<Result<*const godot_gdnative_api_struct, InitError>> {
    let mut got = None;
    if (*api).type_ as u32 == api_type as u32 {
        while !api.is_null() {
            // The boolean expression below SHOULD always be true;
            // we will double check to be safe.
            if (*api).type_ as u32 == api_type as u32 {
                let (major, minor) = ((*api).version.major, (*api).version.minor);
                if major == version_major && minor == version_minor {
                    return Some(Ok(api));
                } else {
                    got = Some(godot_gdnative_api_version { major, minor });
                }
            }
            api = (*api).next;
        }
    }
    got.map(|got| {
        Err(InitError::VersionMismatch {
            want: godot_gdnative_api_version {
                major: version_major,
                minor: version_minor,
            },
            got,
            api_type,
        })
    })
}

unsafe fn find_api_ptr(
    core_api: *const godot_gdnative_core_api_struct,
    api_type: GDNATIVE_API_TYPES,
    version_major: u32,
    version_minor: u32,
) -> Result<*const godot_gdnative_api_struct, InitError> {
    let mut last_error = None;
    match find_version(
        core_api as *const godot_gdnative_api_struct,
        api_type,
        version_major,
        version_minor,
    ) {
        Some(Ok(api)) => {
            return Ok(api);
        }
        Some(Err(error)) => {
            last_error = Some(error);
        }
        None => {}
    }
    for i in 0..(*core_api).num_extensions {
        match find_version(
            *(*core_api).extensions.offset(i as _),
            api_type,
            version_major,
            version_minor,
        ) {
            Some(Ok(api)) => {
                return Ok(api);
            }
            Some(Err(error)) => {
                last_error = Some(error);
            }
            None => {}
        }
    }
    Err(last_error.unwrap_or(InitError::Generic {
        message: format!("Couldn't find API struct with type {}", api_type),
    }))
}
