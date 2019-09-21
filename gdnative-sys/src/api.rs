use crate::*;

macro_rules! def_api {
    (
struct GodotApi {
    core {
        $(
            $clabel:ident($cst:ident, $cver_maj:expr, $cver_min: expr) {
                $(
                    pub $core_name:ident : $core_ty:ty,
                )*
            }
        )*
    }
    extensions {
        $(
            $elabel:ident($ety_key:ident, $est:ident, $ever_maj:expr, $ever_min: expr) {
                $(
                    pub $ext_name:ident : $ext_ty:ty,
                )*
            }
        )*
    }
}
    ) => (
#[doc(hidden)]
#[derive(Clone, Copy)]
pub struct GodotApi {
    $(
    $(
        pub $core_name: $core_ty,
    )*
    )*

    $(
    $(
        pub $ext_name: $ext_ty,
    )*
    )*
}

impl GodotApi {
    pub unsafe fn from_raw(api_raw: *const godot_gdnative_core_api_struct) -> GodotApi {
        $(
            let mut $clabel: Option<&$cst> = None;
        )*
        $(
            let mut $elabel: Option<&$est> = None;
        )*

        let api = &*api_raw;
        for i in 0 .. api.num_extensions {
            let ext = api.extensions.offset(i as _);

            let mut ext_api_ptr = *ext as *const godot_gdnative_api_struct;
            while !ext_api_ptr.is_null() {
                $(
                    if (&*ext_api_ptr).type_ == $ety_key as u32 &&
                        ((&*ext_api_ptr).version.major == $ever_maj)
                        && ((&*ext_api_ptr).version.minor == $ever_min) {
                        $elabel = Some(&*(ext_api_ptr as *const $est));
                    }
                )*

                ext_api_ptr = (&*ext_api_ptr).next;
            }
        }

        {
            let mut core_api_ptr = api_raw as *const godot_gdnative_api_struct;
            while !core_api_ptr.is_null() {
                $(
                    if ((&*core_api_ptr).version.major == $cver_maj)
                    && ((&*core_api_ptr).version.minor == $cver_min) {
                        $clabel = Some(&*(core_api_ptr as *const $cst));
                    }
                )*

                core_api_ptr = (&*core_api_ptr).next;
            }
        }

        $(
            let $clabel: &$cst = $clabel.expect(concat!("Missing core API: ", stringify!($clabel)));
        )*
        $(
            let $elabel: &$est = $elabel.expect(concat!("Missing extension: ", stringify!($elabel)));
        )*
        GodotApi {
            $(
                $(
                    $core_name: $clabel.$core_name.expect(concat!("Missing function: ", stringify!($core_name))),
                )*
            )*
            $(
                $(
                    $ext_name: $elabel.$ext_name.expect(concat!("Missing function: ", stringify!($ext_name))),
                )*
            )*
        }
    }
}
    )
}

include!(concat!(env!("OUT_DIR"), "/api_wrapper.rs"));
