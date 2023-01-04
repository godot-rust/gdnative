use std::sync::atomic::{AtomicBool, Ordering};

use gdnative_impl_proc_macros::cfg_ex;

pub static CHECKED: AtomicBool = AtomicBool::new(false);

/// Checks for any `NativeClass` types that are registered automatically, but not manually.
/// Returns `true` if the test isn't applicable, or if no such types were found.
///
/// Some platforms may not have support for automatic registration. On such platforms, only
/// manually registered classes are visible at run-time.
///
/// Please refer to [the `rust-ctor` README][ctor-repo] for an up-to-date listing of platforms
/// that *do* support automatic registration.
///
/// [ctor-repo]: https://github.com/mmastrac/rust-ctor
#[inline]
pub fn missing_manual_registration() -> bool {
    CHECKED.store(true, Ordering::Release);
    check_missing_manual_registration()
}

#[cfg_ex(not(all(feature = "inventory", gdnative::inventory_platform_available)))]
fn check_missing_manual_registration() -> bool {
    true
}

#[cfg_ex(all(feature = "inventory", gdnative::inventory_platform_available))]
fn check_missing_manual_registration() -> bool {
    use crate::init::InitLevel;

    let types =
        crate::export::class_registry::types_with_init_level(InitLevel::AUTO, InitLevel::USER);

    if types.is_empty() {
        return true;
    }

    let mut message = format!(
        "gdnative-core: {} NativeScript(s) are not manually registered: ",
        types.len()
    );

    let mut first = true;
    for name in types {
        if first {
            first = false;
        } else {
            message.push_str(", ");
        }
        message.push_str(&name);
    }

    godot_warn!("{message}");
    godot_warn!(concat!(
        "gdnative-core: Types that are not manually registered will not be available on platforms ",
        "where automatic registration is unavailable.",
    ));

    false
}
