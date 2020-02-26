#![allow(
    non_upper_case_globals,
    non_camel_case_types,
    improper_ctypes,
    clippy::style
)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
include!(concat!(env!("OUT_DIR"), "/api_wrapper.rs"));

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

unsafe fn find_api_ptr(
    core_api: *const godot_gdnative_core_api_struct,
    api_type: GDNATIVE_API_TYPES,
    version_major: u32,
    version_minor: u32,
) -> Result<*const godot_gdnative_api_struct, InitError> {
    let mut got = None;
    let mut api = core_api as *const godot_gdnative_api_struct;
    if (*api).type_ as u32 == api_type as u32 {
        while !api.is_null() {
            // The boolean expression below SHOULD always be true;
            // we will double check to be safe.
            if (*api).type_ as u32 == api_type as u32 {
                let (major, minor) = ((*api).version.major, (*api).version.minor);
                if major == version_major && minor == version_minor {
                    return Ok(api);
                } else {
                    got = Some(godot_gdnative_api_version { major, minor });
                }
            }
            api = (*api).next;
        }
    }
    for i in 0..(*core_api).num_extensions {
        let mut extension =
            *(*core_api).extensions.offset(i as _) as *const godot_gdnative_api_struct;
        if (*extension).type_ as u32 == api_type as u32 {
            while !extension.is_null() {
                // The boolean expression below SHOULD always be true;
                // we will double check to be safe.
                if (*extension).type_ as u32 == api_type as u32 {
                    let (major, minor) = ((*extension).version.major, (*extension).version.minor);
                    if major == version_major && minor == version_minor {
                        return Ok(extension);
                    } else {
                        got = Some(godot_gdnative_api_version { major, minor });
                    }
                }
                extension = (*extension).next;
            }
        }
    }
    match got {
        Some(got) => Err(InitError::VersionMismatch {
            want: godot_gdnative_api_version {
                major: version_major,
                minor: version_minor,
            },
            got,
            api_type,
        }),
        None => Err(InitError::Generic {
            message: format!("Couldn't find API struct with type {}", api_type),
        }),
    }
}
