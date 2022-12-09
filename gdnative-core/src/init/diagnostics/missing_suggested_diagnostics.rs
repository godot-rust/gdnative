use gdnative_impl_proc_macros::cfg_ex;

/// Checks if all suggested diagnostics have been ran depending on the current platform, at
/// the point of invocation. This is automatically ran as part of the init macro, and do not
/// usually need to be manually invoked.
///
/// Returns `true` in a release build, or if no such diagnostics were found.
#[inline]
pub fn missing_suggested_diagnostics() -> bool {
    check_missing_suggested_diagnostics()
}

#[cfg(not(debug_assertions))]
fn check_missing_suggested_diagnostics() -> bool {
    true
}

#[cfg(debug_assertions)]
fn check_missing_suggested_diagnostics() -> bool {
    check_missing_suggested_diagnostics_inventory_unavailable()
}

#[cfg_ex(all(feature = "inventory", not(gdnative::inventory_platform_available)))]
fn check_missing_suggested_diagnostics_inventory_unavailable() -> bool {
    if !super::missing_manual_registration::CHECKED.load(std::sync::atomic::Ordering::Acquire) {
        godot_warn!(concat!(
            "gdnative-core: `gdnative` was compiled with the `inventory` feature, but the current platform ",
            "does not support automatic registration. As such, only manually registered types will be available.\n",
            "Call `gdnative::init::diagnostics::missing_manual_registration()` at the end your init callback to "
            "suppress this message."
        ));

        false
    } else {
        true
    }
}

#[cfg_ex(not(all(feature = "inventory", not(gdnative::inventory_platform_available))))]
fn check_missing_suggested_diagnostics_inventory_unavailable() -> bool {
    true
}
