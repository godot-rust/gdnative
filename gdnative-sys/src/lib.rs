#![allow(
    non_upper_case_globals,
    non_camel_case_types,
    improper_ctypes,
    clippy::style
)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
include!(concat!(env!("OUT_DIR"), "/api_wrapper.rs"));

unsafe fn find_api_ptr(
    core_api: *const godot_gdnative_core_api_struct,
    api_type: GDNATIVE_API_TYPES,
    version_major: u32,
    version_minor: u32,
) -> *const godot_gdnative_api_struct {
    let mut api = core_api as *const godot_gdnative_api_struct;
    if (*api).type_ == api_type {
        while !api.is_null() {
            if (*api).version.minor == version_minor
                && (*api).version.major == version_major
                // The boolean expression below SHOULD always be true;
                // we will double check to be safe.
                && (*api).type_ == api_type
            {
                return api;
            }
            api = (*api).next;
        }
    }
    for i in 0..(*core_api).num_extensions {
        let mut extension =
            *(*core_api).extensions.offset(i as _) as *const godot_gdnative_api_struct;
        if (*extension).type_ == api_type {
            while !extension.is_null() {
                if (*extension).version.minor == version_minor
                    && (*extension).version.major == version_major
                    // The boolean expression below SHOULD always be true;
                    // we will double check to be safe.
                    && (*extension).type_ == api_type
                {
                    return extension;
                }
                extension = (*extension).next;
            }
        }
    }
    panic!(
        "Couldn't find API: {:?}",
        (api_type, version_major, version_minor)
    );
}
