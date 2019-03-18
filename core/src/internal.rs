use libc;
use crate::sys::*;

macro_rules! def_api {
    (
struct GodotApi {
    $(
        $sub:ident($ty_key:ident, $st:ident) {
            $(
                pub $name:ident : ::std::option::Option<$ty:ty>,
            )*
        }
    )*
}
    ) => (
#[doc(hidden)]
#[derive(Clone, Copy)]
pub struct GodotApi {
    $(
    $(
        pub $name: $ty,
    )*
    )*
}

impl GodotApi {
    pub unsafe fn from_raw(api_raw: *const godot_gdnative_core_api_struct) -> GodotApi {
        $(
            let mut $sub: Option<&$st> = None;
        )*
        let api = &*api_raw;
        for i in 0 .. api.num_extensions {
            let ext = api.extensions.offset(i as _);
            $(
                if (**ext).type_ == $ty_key as u32 {
                    $sub = Some(&*((*ext) as *const $st));
                }
            )*
        }

        $(
            if GDNATIVE_API_TYPES_GDNATIVE_CORE == $ty_key {
                $sub = Some(&*(api_raw as *const $st));
            }
        )*

        $(
            let $sub: &$st = $sub.expect(concat!("Missing extension: ", stringify!($sub)));
        )*
        GodotApi {
            $(
                $(
                    $name: $sub.$name.expect(concat!("Missing function: ", stringify!($name))),
                )*
            )*
        }
    }
}
    )
}

def_api! {
struct GodotApi {
    core(GDNATIVE_API_TYPES_GDNATIVE_CORE, godot_gdnative_core_api_struct) {
        pub godot_color_new_rgba: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                *mut godot_color,
                                                                            p_r:
                                                                                godot_real,
                                                                            p_g:
                                                                                godot_real,
                                                                            p_b:
                                                                                godot_real,
                                                                            p_a:
                                                                                godot_real)>,
        pub godot_color_new_rgb: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                *mut godot_color,
                                                                            p_r:
                                                                                godot_real,
                                                                            p_g:
                                                                                godot_real,
                                                                            p_b:
                                                                                godot_real)>,
        pub godot_color_get_r: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_color)
                                                        -> godot_real>,
        pub godot_color_set_r: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *mut godot_color,
                                                                        r:
                                                                            godot_real)>,
        pub godot_color_get_g: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_color)
                                                        -> godot_real>,
        pub godot_color_set_g: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *mut godot_color,
                                                                        g:
                                                                            godot_real)>,
        pub godot_color_get_b: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_color)
                                                        -> godot_real>,
        pub godot_color_set_b: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *mut godot_color,
                                                                        b:
                                                                            godot_real)>,
        pub godot_color_get_a: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_color)
                                                        -> godot_real>,
        pub godot_color_set_a: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *mut godot_color,
                                                                        a:
                                                                            godot_real)>,
        pub godot_color_get_h: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_color)
                                                        -> godot_real>,
        pub godot_color_get_s: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_color)
                                                        -> godot_real>,
        pub godot_color_get_v: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_color)
                                                        -> godot_real>,
        pub godot_color_as_string: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_color)
                                                            -> godot_string>,
        pub godot_color_to_rgba32: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_color)
                                                            -> godot_int>,
        pub godot_color_to_argb32: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_color)
                                                            -> godot_int>,
        pub godot_color_gray: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_color)
                                                        -> godot_real>,
        pub godot_color_inverted: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_color)
                                                            -> godot_color>,
        pub godot_color_contrasted: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_color)
                                                            -> godot_color>,
        pub godot_color_linear_interpolate: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_color,
                                                                                    p_b:
                                                                                        *const godot_color,
                                                                                    p_t:
                                                                                        godot_real)
                                                                    ->
                                                                        godot_color>,
        pub godot_color_blend: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_color,
                                                                        p_over:
                                                                            *const godot_color)
                                                        -> godot_color>,
        pub godot_color_to_html: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_color,
                                                                            p_with_alpha:
                                                                                godot_bool)
                                                        -> godot_string>,
        pub godot_color_operator_equal: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_color,
                                                                                p_b:
                                                                                    *const godot_color)
                                                                -> godot_bool>,
        pub godot_color_operator_less: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_color,
                                                                                p_b:
                                                                                    *const godot_color)
                                                                -> godot_bool>,
        pub godot_vector2_new: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                            *mut godot_vector2,
                                                                        p_x:
                                                                            godot_real,
                                                                        p_y:
                                                                            godot_real)>,
        pub godot_vector2_as_string: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_vector2)
                                                            -> godot_string>,
        pub godot_vector2_normalized: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_vector2)
                                                                -> godot_vector2>,
        pub godot_vector2_length: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_vector2)
                                                            -> godot_real>,
        pub godot_vector2_angle: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_vector2)
                                                        -> godot_real>,
        pub godot_vector2_length_squared: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_vector2)
                                                                    ->
                                                                        godot_real>,
        pub godot_vector2_is_normalized: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_vector2)
                                                                -> godot_bool>,
        pub godot_vector2_distance_to: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_vector2,
                                                                                p_to:
                                                                                    *const godot_vector2)
                                                                -> godot_real>,
        pub godot_vector2_distance_squared_to: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *const godot_vector2,
                                                                                        p_to:
                                                                                            *const godot_vector2)
                                                                        ->
                                                                            godot_real>,
        pub godot_vector2_angle_to: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_vector2,
                                                                            p_to:
                                                                                *const godot_vector2)
                                                            -> godot_real>,
        pub godot_vector2_angle_to_point: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_vector2,
                                                                                    p_to:
                                                                                        *const godot_vector2)
                                                                    ->
                                                                        godot_real>,
        pub godot_vector2_linear_interpolate: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *const godot_vector2,
                                                                                        p_b:
                                                                                            *const godot_vector2,
                                                                                        p_t:
                                                                                            godot_real)
                                                                        ->
                                                                            godot_vector2>,
        pub godot_vector2_cubic_interpolate: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *const godot_vector2,
                                                                                        p_b:
                                                                                            *const godot_vector2,
                                                                                        p_pre_a:
                                                                                            *const godot_vector2,
                                                                                        p_post_b:
                                                                                            *const godot_vector2,
                                                                                        p_t:
                                                                                            godot_real)
                                                                    ->
                                                                        godot_vector2>,
        pub godot_vector2_rotated: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_vector2,
                                                                            p_phi:
                                                                                godot_real)
                                                            -> godot_vector2>,
        pub godot_vector2_tangent: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_vector2)
                                                            -> godot_vector2>,
        pub godot_vector2_floor: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_vector2)
                                                        -> godot_vector2>,
        pub godot_vector2_snapped: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_vector2,
                                                                            p_by:
                                                                                *const godot_vector2)
                                                            -> godot_vector2>,
        pub godot_vector2_aspect: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_vector2)
                                                            -> godot_real>,
        pub godot_vector2_dot: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_vector2,
                                                                        p_with:
                                                                            *const godot_vector2)
                                                        -> godot_real>,
        pub godot_vector2_slide: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_vector2,
                                                                            p_n:
                                                                                *const godot_vector2)
                                                        -> godot_vector2>,
        pub godot_vector2_bounce: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_vector2,
                                                                            p_n:
                                                                                *const godot_vector2)
                                                            -> godot_vector2>,
        pub godot_vector2_reflect: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_vector2,
                                                                            p_n:
                                                                                *const godot_vector2)
                                                            -> godot_vector2>,
        pub godot_vector2_abs: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_vector2)
                                                        -> godot_vector2>,
        pub godot_vector2_clamped: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_vector2,
                                                                            p_length:
                                                                                godot_real)
                                                            -> godot_vector2>,
        pub godot_vector2_operator_add: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_vector2,
                                                                                p_b:
                                                                                    *const godot_vector2)
                                                                ->
                                                                    godot_vector2>,
        pub godot_vector2_operator_subtract: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *const godot_vector2,
                                                                                        p_b:
                                                                                            *const godot_vector2)
                                                                    ->
                                                                        godot_vector2>,
        pub godot_vector2_operator_multiply_vector: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                                *const godot_vector2,
                                                                                            p_b:
                                                                                                *const godot_vector2)
                                                                            ->
                                                                                godot_vector2>,
        pub godot_vector2_operator_multiply_scalar: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                                *const godot_vector2,
                                                                                            p_b:
                                                                                                godot_real)
                                                                            ->
                                                                                godot_vector2>,
        pub godot_vector2_operator_divide_vector: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                                *const godot_vector2,
                                                                                            p_b:
                                                                                                *const godot_vector2)
                                                                            ->
                                                                                godot_vector2>,
        pub godot_vector2_operator_divide_scalar: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                                *const godot_vector2,
                                                                                            p_b:
                                                                                                godot_real)
                                                                            ->
                                                                                godot_vector2>,
        pub godot_vector2_operator_equal: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_vector2,
                                                                                    p_b:
                                                                                        *const godot_vector2)
                                                                    ->
                                                                        godot_bool>,
        pub godot_vector2_operator_less: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_vector2,
                                                                                    p_b:
                                                                                        *const godot_vector2)
                                                                -> godot_bool>,
        pub godot_vector2_operator_neg: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_vector2)
                                                                ->
                                                                    godot_vector2>,
        pub godot_vector2_set_x: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *mut godot_vector2,
                                                                            p_x:
                                                                                godot_real)>,
        pub godot_vector2_set_y: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *mut godot_vector2,
                                                                            p_y:
                                                                                godot_real)>,
        pub godot_vector2_get_x: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_vector2)
                                                        -> godot_real>,
        pub godot_vector2_get_y: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_vector2)
                                                        -> godot_real>,
        pub godot_quat_new: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                        *mut godot_quat,
                                                                    p_x:
                                                                        godot_real,
                                                                    p_y:
                                                                        godot_real,
                                                                    p_z:
                                                                        godot_real,
                                                                    p_w:
                                                                        godot_real)>,
        pub godot_quat_new_with_axis_angle: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                        *mut godot_quat,
                                                                                    p_axis:
                                                                                        *const godot_vector3,
                                                                                    p_angle:
                                                                                        godot_real)>,
        pub godot_quat_get_x: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_quat)
                                                        -> godot_real>,
        pub godot_quat_set_x: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *mut godot_quat,
                                                                        val:
                                                                            godot_real)>,
        pub godot_quat_get_y: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_quat)
                                                        -> godot_real>,
        pub godot_quat_set_y: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *mut godot_quat,
                                                                        val:
                                                                            godot_real)>,
        pub godot_quat_get_z: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_quat)
                                                        -> godot_real>,
        pub godot_quat_set_z: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *mut godot_quat,
                                                                        val:
                                                                            godot_real)>,
        pub godot_quat_get_w: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_quat)
                                                        -> godot_real>,
        pub godot_quat_set_w: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *mut godot_quat,
                                                                        val:
                                                                            godot_real)>,
        pub godot_quat_as_string: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_quat)
                                                            -> godot_string>,
        pub godot_quat_length: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_quat)
                                                        -> godot_real>,
        pub godot_quat_length_squared: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_quat)
                                                                -> godot_real>,
        pub godot_quat_normalized: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_quat)
                                                            -> godot_quat>,
        pub godot_quat_is_normalized: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_quat)
                                                                -> godot_bool>,
        pub godot_quat_inverse: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_quat)
                                                        -> godot_quat>,
        pub godot_quat_dot: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                        *const godot_quat,
                                                                    p_b:
                                                                        *const godot_quat)
                                                    -> godot_real>,
        pub godot_quat_xform: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_quat,
                                                                        p_v:
                                                                            *const godot_vector3)
                                                        -> godot_vector3>,
        pub godot_quat_slerp: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_quat,
                                                                        p_b:
                                                                            *const godot_quat,
                                                                        p_t:
                                                                            godot_real)
                                                        -> godot_quat>,
        pub godot_quat_slerpni: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_quat,
                                                                        p_b:
                                                                            *const godot_quat,
                                                                        p_t:
                                                                            godot_real)
                                                        -> godot_quat>,
        pub godot_quat_cubic_slerp: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_quat,
                                                                            p_b:
                                                                                *const godot_quat,
                                                                            p_pre_a:
                                                                                *const godot_quat,
                                                                            p_post_b:
                                                                                *const godot_quat,
                                                                            p_t:
                                                                                godot_real)
                                                            -> godot_quat>,
        pub godot_quat_operator_multiply: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_quat,
                                                                                    p_b:
                                                                                        godot_real)
                                                                    ->
                                                                        godot_quat>,
        pub godot_quat_operator_add: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_quat,
                                                                                p_b:
                                                                                    *const godot_quat)
                                                            -> godot_quat>,
        pub godot_quat_operator_subtract: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_quat,
                                                                                    p_b:
                                                                                        *const godot_quat)
                                                                    ->
                                                                        godot_quat>,
        pub godot_quat_operator_divide: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_quat,
                                                                                p_b:
                                                                                    godot_real)
                                                                -> godot_quat>,
        pub godot_quat_operator_equal: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_quat,
                                                                                p_b:
                                                                                    *const godot_quat)
                                                                -> godot_bool>,
        pub godot_quat_operator_neg: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_quat)
                                                            -> godot_quat>,
        pub godot_basis_new_with_rows: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                    *mut godot_basis,
                                                                                p_x_axis:
                                                                                    *const godot_vector3,
                                                                                p_y_axis:
                                                                                    *const godot_vector3,
                                                                                p_z_axis:
                                                                                    *const godot_vector3)>,
        pub godot_basis_new_with_axis_and_angle: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                                *mut godot_basis,
                                                                                            p_axis:
                                                                                                *const godot_vector3,
                                                                                            p_phi:
                                                                                                godot_real)>,
        pub godot_basis_new_with_euler: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                    *mut godot_basis,
                                                                                p_euler:
                                                                                    *const godot_vector3)>,
        pub godot_basis_as_string: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_basis)
                                                            -> godot_string>,
        pub godot_basis_inverse: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_basis)
                                                        -> godot_basis>,
        pub godot_basis_transposed: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_basis)
                                                            -> godot_basis>,
        pub godot_basis_orthonormalized: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_basis)
                                                                ->
                                                                    godot_basis>,
        pub godot_basis_determinant: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_basis)
                                                            -> godot_real>,
        pub godot_basis_rotated: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_basis,
                                                                            p_axis:
                                                                                *const godot_vector3,
                                                                            p_phi:
                                                                                godot_real)
                                                        -> godot_basis>,
        pub godot_basis_scaled: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_basis,
                                                                        p_scale:
                                                                            *const godot_vector3)
                                                        -> godot_basis>,
        pub godot_basis_get_scale: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_basis)
                                                            -> godot_vector3>,
        pub godot_basis_get_euler: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_basis)
                                                            -> godot_vector3>,
        pub godot_basis_tdotx: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_basis,
                                                                        p_with:
                                                                            *const godot_vector3)
                                                        -> godot_real>,
        pub godot_basis_tdoty: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_basis,
                                                                        p_with:
                                                                            *const godot_vector3)
                                                        -> godot_real>,
        pub godot_basis_tdotz: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_basis,
                                                                        p_with:
                                                                            *const godot_vector3)
                                                        -> godot_real>,
        pub godot_basis_xform: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_basis,
                                                                        p_v:
                                                                            *const godot_vector3)
                                                        -> godot_vector3>,
        pub godot_basis_xform_inv: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_basis,
                                                                            p_v:
                                                                                *const godot_vector3)
                                                            -> godot_vector3>,
        pub godot_basis_get_orthogonal_index: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *const godot_basis)
                                                                        ->
                                                                            godot_int>,
        pub godot_basis_new: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                            *mut godot_basis)>,
        pub godot_basis_new_with_euler_quat: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                            *mut godot_basis,
                                                                                        p_euler:
                                                                                            *const godot_quat)>,
        pub godot_basis_get_elements: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_basis,
                                                                                p_elements:
                                                                                    *mut godot_vector3)>,
        pub godot_basis_get_axis: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_basis,
                                                                            p_axis:
                                                                                godot_int)
                                                            -> godot_vector3>,
        pub godot_basis_set_axis: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *mut godot_basis,
                                                                            p_axis:
                                                                                godot_int,
                                                                            p_value:
                                                                                *const godot_vector3)>,
        pub godot_basis_get_row: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_basis,
                                                                            p_row:
                                                                                godot_int)
                                                        -> godot_vector3>,
        pub godot_basis_set_row: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *mut godot_basis,
                                                                            p_row:
                                                                                godot_int,
                                                                            p_value:
                                                                                *const godot_vector3)>,
        pub godot_basis_operator_equal: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_basis,
                                                                                p_b:
                                                                                    *const godot_basis)
                                                                -> godot_bool>,
        pub godot_basis_operator_add: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_basis,
                                                                                p_b:
                                                                                    *const godot_basis)
                                                                -> godot_basis>,
        pub godot_basis_operator_subtract: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_basis,
                                                                                    p_b:
                                                                                        *const godot_basis)
                                                                    ->
                                                                        godot_basis>,
        pub godot_basis_operator_multiply_vector: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                                *const godot_basis,
                                                                                            p_b:
                                                                                                *const godot_basis)
                                                                            ->
                                                                                godot_basis>,
        pub godot_basis_operator_multiply_scalar: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                                *const godot_basis,
                                                                                            p_b:
                                                                                                godot_real)
                                                                            ->
                                                                                godot_basis>,
        pub godot_vector3_new: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                            *mut godot_vector3,
                                                                        p_x:
                                                                            godot_real,
                                                                        p_y:
                                                                            godot_real,
                                                                        p_z:
                                                                            godot_real)>,
        pub godot_vector3_as_string: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_vector3)
                                                            -> godot_string>,
        pub godot_vector3_min_axis: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_vector3)
                                                            -> godot_int>,
        pub godot_vector3_max_axis: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_vector3)
                                                            -> godot_int>,
        pub godot_vector3_length: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_vector3)
                                                            -> godot_real>,
        pub godot_vector3_length_squared: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_vector3)
                                                                    ->
                                                                        godot_real>,
        pub godot_vector3_is_normalized: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_vector3)
                                                                -> godot_bool>,
        pub godot_vector3_normalized: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_vector3)
                                                                -> godot_vector3>,
        pub godot_vector3_inverse: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_vector3)
                                                            -> godot_vector3>,
        pub godot_vector3_snapped: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_vector3,
                                                                            p_by:
                                                                                *const godot_vector3)
                                                            -> godot_vector3>,
        pub godot_vector3_rotated: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_vector3,
                                                                            p_axis:
                                                                                *const godot_vector3,
                                                                            p_phi:
                                                                                godot_real)
                                                            -> godot_vector3>,
        pub godot_vector3_linear_interpolate: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *const godot_vector3,
                                                                                        p_b:
                                                                                            *const godot_vector3,
                                                                                        p_t:
                                                                                            godot_real)
                                                                        ->
                                                                            godot_vector3>,
        pub godot_vector3_cubic_interpolate: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *const godot_vector3,
                                                                                        p_b:
                                                                                            *const godot_vector3,
                                                                                        p_pre_a:
                                                                                            *const godot_vector3,
                                                                                        p_post_b:
                                                                                            *const godot_vector3,
                                                                                        p_t:
                                                                                            godot_real)
                                                                    ->
                                                                        godot_vector3>,
        pub godot_vector3_dot: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_vector3,
                                                                        p_b:
                                                                            *const godot_vector3)
                                                        -> godot_real>,
        pub godot_vector3_cross: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_vector3,
                                                                            p_b:
                                                                                *const godot_vector3)
                                                        -> godot_vector3>,
        pub godot_vector3_outer: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_vector3,
                                                                            p_b:
                                                                                *const godot_vector3)
                                                        -> godot_basis>,
        pub godot_vector3_to_diagonal_matrix: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *const godot_vector3)
                                                                        ->
                                                                            godot_basis>,
        pub godot_vector3_abs: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_vector3)
                                                        -> godot_vector3>,
        pub godot_vector3_floor: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_vector3)
                                                        -> godot_vector3>,
        pub godot_vector3_ceil: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_vector3)
                                                        -> godot_vector3>,
        pub godot_vector3_distance_to: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_vector3,
                                                                                p_b:
                                                                                    *const godot_vector3)
                                                                -> godot_real>,
        pub godot_vector3_distance_squared_to: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *const godot_vector3,
                                                                                        p_b:
                                                                                            *const godot_vector3)
                                                                        ->
                                                                            godot_real>,
        pub godot_vector3_angle_to: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_vector3,
                                                                            p_to:
                                                                                *const godot_vector3)
                                                            -> godot_real>,
        pub godot_vector3_slide: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_vector3,
                                                                            p_n:
                                                                                *const godot_vector3)
                                                        -> godot_vector3>,
        pub godot_vector3_bounce: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_vector3,
                                                                            p_n:
                                                                                *const godot_vector3)
                                                            -> godot_vector3>,
        pub godot_vector3_reflect: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_vector3,
                                                                            p_n:
                                                                                *const godot_vector3)
                                                            -> godot_vector3>,
        pub godot_vector3_operator_add: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_vector3,
                                                                                p_b:
                                                                                    *const godot_vector3)
                                                                ->
                                                                    godot_vector3>,
        pub godot_vector3_operator_subtract: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *const godot_vector3,
                                                                                        p_b:
                                                                                            *const godot_vector3)
                                                                    ->
                                                                        godot_vector3>,
        pub godot_vector3_operator_multiply_vector: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                                *const godot_vector3,
                                                                                            p_b:
                                                                                                *const godot_vector3)
                                                                            ->
                                                                                godot_vector3>,
        pub godot_vector3_operator_multiply_scalar: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                                *const godot_vector3,
                                                                                            p_b:
                                                                                                godot_real)
                                                                            ->
                                                                                godot_vector3>,
        pub godot_vector3_operator_divide_vector: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                                *const godot_vector3,
                                                                                            p_b:
                                                                                                *const godot_vector3)
                                                                            ->
                                                                                godot_vector3>,
        pub godot_vector3_operator_divide_scalar: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                                *const godot_vector3,
                                                                                            p_b:
                                                                                                godot_real)
                                                                            ->
                                                                                godot_vector3>,
        pub godot_vector3_operator_equal: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_vector3,
                                                                                    p_b:
                                                                                        *const godot_vector3)
                                                                    ->
                                                                        godot_bool>,
        pub godot_vector3_operator_less: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_vector3,
                                                                                    p_b:
                                                                                        *const godot_vector3)
                                                                -> godot_bool>,
        pub godot_vector3_operator_neg: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_vector3)
                                                                ->
                                                                    godot_vector3>,
        pub godot_vector3_set_axis: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *mut godot_vector3,
                                                                            p_axis:
                                                                                godot_vector3_axis,
                                                                            p_val:
                                                                                godot_real)>,
        pub godot_vector3_get_axis: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_vector3,
                                                                            p_axis:
                                                                                godot_vector3_axis)
                                                            -> godot_real>,
        pub godot_pool_byte_array_new: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                    *mut godot_pool_byte_array)>,
        pub godot_pool_byte_array_new_copy: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                        *mut godot_pool_byte_array,
                                                                                    p_src:
                                                                                        *const godot_pool_byte_array)>,
        pub godot_pool_byte_array_new_with_array: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                                *mut godot_pool_byte_array,
                                                                                            p_a:
                                                                                                *const godot_array)>,
        pub godot_pool_byte_array_append: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *mut godot_pool_byte_array,
                                                                                    p_data:
                                                                                        u8)>,
        pub godot_pool_byte_array_append_array: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *mut godot_pool_byte_array,
                                                                                        p_array:
                                                                                            *const godot_pool_byte_array)>,
        pub godot_pool_byte_array_insert: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *mut godot_pool_byte_array,
                                                                                    p_idx:
                                                                                        godot_int,
                                                                                    p_data:
                                                                                        u8)
                                                                    ->
                                                                        godot_error>,
        pub godot_pool_byte_array_invert: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *mut godot_pool_byte_array)>,
        pub godot_pool_byte_array_push_back: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *mut godot_pool_byte_array,
                                                                                        p_data:
                                                                                            u8)>,
        pub godot_pool_byte_array_remove: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *mut godot_pool_byte_array,
                                                                                    p_idx:
                                                                                        godot_int)>,
        pub godot_pool_byte_array_resize: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *mut godot_pool_byte_array,
                                                                                    p_size:
                                                                                        godot_int)>,
        pub godot_pool_byte_array_read: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_pool_byte_array)
                                                                ->
                                                                    *mut godot_pool_byte_array_read_access>,
        pub godot_pool_byte_array_write: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *mut godot_pool_byte_array)
                                                                ->
                                                                    *mut godot_pool_byte_array_write_access>,
        pub godot_pool_byte_array_set: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *mut godot_pool_byte_array,
                                                                                p_idx:
                                                                                    godot_int,
                                                                                p_data:
                                                                                    u8)>,
        pub godot_pool_byte_array_get: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_pool_byte_array,
                                                                                p_idx:
                                                                                    godot_int)
                                                                -> u8>,
        pub godot_pool_byte_array_size: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_pool_byte_array)
                                                                -> godot_int>,
        pub godot_pool_byte_array_destroy: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *mut godot_pool_byte_array)>,
        pub godot_pool_int_array_new: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                    *mut godot_pool_int_array)>,
        pub godot_pool_int_array_new_copy: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                        *mut godot_pool_int_array,
                                                                                    p_src:
                                                                                        *const godot_pool_int_array)>,
        pub godot_pool_int_array_new_with_array: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                                *mut godot_pool_int_array,
                                                                                            p_a:
                                                                                                *const godot_array)>,
        pub godot_pool_int_array_append: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *mut godot_pool_int_array,
                                                                                    p_data:
                                                                                        godot_int)>,
        pub godot_pool_int_array_append_array: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *mut godot_pool_int_array,
                                                                                        p_array:
                                                                                            *const godot_pool_int_array)>,
        pub godot_pool_int_array_insert: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *mut godot_pool_int_array,
                                                                                    p_idx:
                                                                                        godot_int,
                                                                                    p_data:
                                                                                        godot_int)
                                                                ->
                                                                    godot_error>,
        pub godot_pool_int_array_invert: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *mut godot_pool_int_array)>,
        pub godot_pool_int_array_push_back: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *mut godot_pool_int_array,
                                                                                    p_data:
                                                                                        godot_int)>,
        pub godot_pool_int_array_remove: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *mut godot_pool_int_array,
                                                                                    p_idx:
                                                                                        godot_int)>,
        pub godot_pool_int_array_resize: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *mut godot_pool_int_array,
                                                                                    p_size:
                                                                                        godot_int)>,
        pub godot_pool_int_array_read: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_pool_int_array)
                                                                ->
                                                                    *mut godot_pool_int_array_read_access>,
        pub godot_pool_int_array_write: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *mut godot_pool_int_array)
                                                                ->
                                                                    *mut godot_pool_int_array_write_access>,
        pub godot_pool_int_array_set: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *mut godot_pool_int_array,
                                                                                p_idx:
                                                                                    godot_int,
                                                                                p_data:
                                                                                    godot_int)>,
        pub godot_pool_int_array_get: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_pool_int_array,
                                                                                p_idx:
                                                                                    godot_int)
                                                                -> godot_int>,
        pub godot_pool_int_array_size: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_pool_int_array)
                                                                -> godot_int>,
        pub godot_pool_int_array_destroy: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *mut godot_pool_int_array)>,
        pub godot_pool_real_array_new: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                    *mut godot_pool_real_array)>,
        pub godot_pool_real_array_new_copy: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                        *mut godot_pool_real_array,
                                                                                    p_src:
                                                                                        *const godot_pool_real_array)>,
        pub godot_pool_real_array_new_with_array: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                                *mut godot_pool_real_array,
                                                                                            p_a:
                                                                                                *const godot_array)>,
        pub godot_pool_real_array_append: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *mut godot_pool_real_array,
                                                                                    p_data:
                                                                                        godot_real)>,
        pub godot_pool_real_array_append_array: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *mut godot_pool_real_array,
                                                                                        p_array:
                                                                                            *const godot_pool_real_array)>,
        pub godot_pool_real_array_insert: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *mut godot_pool_real_array,
                                                                                    p_idx:
                                                                                        godot_int,
                                                                                    p_data:
                                                                                        godot_real)
                                                                    ->
                                                                        godot_error>,
        pub godot_pool_real_array_invert: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *mut godot_pool_real_array)>,
        pub godot_pool_real_array_push_back: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *mut godot_pool_real_array,
                                                                                        p_data:
                                                                                            godot_real)>,
        pub godot_pool_real_array_remove: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *mut godot_pool_real_array,
                                                                                    p_idx:
                                                                                        godot_int)>,
        pub godot_pool_real_array_resize: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *mut godot_pool_real_array,
                                                                                    p_size:
                                                                                        godot_int)>,
        pub godot_pool_real_array_read: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_pool_real_array)
                                                                ->
                                                                    *mut godot_pool_real_array_read_access>,
        pub godot_pool_real_array_write: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *mut godot_pool_real_array)
                                                                ->
                                                                    *mut godot_pool_real_array_write_access>,
        pub godot_pool_real_array_set: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *mut godot_pool_real_array,
                                                                                p_idx:
                                                                                    godot_int,
                                                                                p_data:
                                                                                    godot_real)>,
        pub godot_pool_real_array_get: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_pool_real_array,
                                                                                p_idx:
                                                                                    godot_int)
                                                                -> godot_real>,
        pub godot_pool_real_array_size: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_pool_real_array)
                                                                -> godot_int>,
        pub godot_pool_real_array_destroy: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *mut godot_pool_real_array)>,
        pub godot_pool_string_array_new: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                        *mut godot_pool_string_array)>,
        pub godot_pool_string_array_new_copy: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                            *mut godot_pool_string_array,
                                                                                        p_src:
                                                                                            *const godot_pool_string_array)>,
        pub godot_pool_string_array_new_with_array: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                                *mut godot_pool_string_array,
                                                                                            p_a:
                                                                                                *const godot_array)>,
        pub godot_pool_string_array_append: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *mut godot_pool_string_array,
                                                                                    p_data:
                                                                                        *const godot_string)>,
        pub godot_pool_string_array_append_array: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                                *mut godot_pool_string_array,
                                                                                            p_array:
                                                                                                *const godot_pool_string_array)>,
        pub godot_pool_string_array_insert: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *mut godot_pool_string_array,
                                                                                    p_idx:
                                                                                        godot_int,
                                                                                    p_data:
                                                                                        *const godot_string)
                                                                    ->
                                                                        godot_error>,
        pub godot_pool_string_array_invert: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *mut godot_pool_string_array)>,
        pub godot_pool_string_array_push_back: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *mut godot_pool_string_array,
                                                                                        p_data:
                                                                                            *const godot_string)>,
        pub godot_pool_string_array_remove: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *mut godot_pool_string_array,
                                                                                    p_idx:
                                                                                        godot_int)>,
        pub godot_pool_string_array_resize: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *mut godot_pool_string_array,
                                                                                    p_size:
                                                                                        godot_int)>,
        pub godot_pool_string_array_read: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_pool_string_array)
                                                                    ->
                                                                        *mut godot_pool_string_array_read_access>,
        pub godot_pool_string_array_write: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *mut godot_pool_string_array)
                                                                    ->
                                                                        *mut godot_pool_string_array_write_access>,
        pub godot_pool_string_array_set: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *mut godot_pool_string_array,
                                                                                    p_idx:
                                                                                        godot_int,
                                                                                    p_data:
                                                                                        *const godot_string)>,
        pub godot_pool_string_array_get: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_pool_string_array,
                                                                                    p_idx:
                                                                                        godot_int)
                                                                ->
                                                                    godot_string>,
        pub godot_pool_string_array_size: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_pool_string_array)
                                                                    -> godot_int>,
        pub godot_pool_string_array_destroy: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *mut godot_pool_string_array)>,
        pub godot_pool_vector2_array_new: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                        *mut godot_pool_vector2_array)>,
        pub godot_pool_vector2_array_new_copy: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                            *mut godot_pool_vector2_array,
                                                                                        p_src:
                                                                                            *const godot_pool_vector2_array)>,
        pub godot_pool_vector2_array_new_with_array: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                                    *mut godot_pool_vector2_array,
                                                                                                p_a:
                                                                                                    *const godot_array)>,
        pub godot_pool_vector2_array_append: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *mut godot_pool_vector2_array,
                                                                                        p_data:
                                                                                            *const godot_vector2)>,
        pub godot_pool_vector2_array_append_array: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                                *mut godot_pool_vector2_array,
                                                                                            p_array:
                                                                                                *const godot_pool_vector2_array)>,
        pub godot_pool_vector2_array_insert: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *mut godot_pool_vector2_array,
                                                                                        p_idx:
                                                                                            godot_int,
                                                                                        p_data:
                                                                                            *const godot_vector2)
                                                                    ->
                                                                        godot_error>,
        pub godot_pool_vector2_array_invert: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *mut godot_pool_vector2_array)>,
        pub godot_pool_vector2_array_push_back: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *mut godot_pool_vector2_array,
                                                                                        p_data:
                                                                                            *const godot_vector2)>,
        pub godot_pool_vector2_array_remove: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *mut godot_pool_vector2_array,
                                                                                        p_idx:
                                                                                            godot_int)>,
        pub godot_pool_vector2_array_resize: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *mut godot_pool_vector2_array,
                                                                                        p_size:
                                                                                            godot_int)>,
        pub godot_pool_vector2_array_read: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_pool_vector2_array)
                                                                    ->
                                                                        *mut godot_pool_vector2_array_read_access>,
        pub godot_pool_vector2_array_write: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *mut godot_pool_vector2_array)
                                                                    ->
                                                                        *mut godot_pool_vector2_array_write_access>,
        pub godot_pool_vector2_array_set: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *mut godot_pool_vector2_array,
                                                                                    p_idx:
                                                                                        godot_int,
                                                                                    p_data:
                                                                                        *const godot_vector2)>,
        pub godot_pool_vector2_array_get: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_pool_vector2_array,
                                                                                    p_idx:
                                                                                        godot_int)
                                                                    ->
                                                                        godot_vector2>,
        pub godot_pool_vector2_array_size: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_pool_vector2_array)
                                                                    ->
                                                                        godot_int>,
        pub godot_pool_vector2_array_destroy: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *mut godot_pool_vector2_array)>,
        pub godot_pool_vector3_array_new: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                        *mut godot_pool_vector3_array)>,
        pub godot_pool_vector3_array_new_copy: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                            *mut godot_pool_vector3_array,
                                                                                        p_src:
                                                                                            *const godot_pool_vector3_array)>,
        pub godot_pool_vector3_array_new_with_array: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                                    *mut godot_pool_vector3_array,
                                                                                                p_a:
                                                                                                    *const godot_array)>,
        pub godot_pool_vector3_array_append: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *mut godot_pool_vector3_array,
                                                                                        p_data:
                                                                                            *const godot_vector3)>,
        pub godot_pool_vector3_array_append_array: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                                *mut godot_pool_vector3_array,
                                                                                            p_array:
                                                                                                *const godot_pool_vector3_array)>,
        pub godot_pool_vector3_array_insert: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *mut godot_pool_vector3_array,
                                                                                        p_idx:
                                                                                            godot_int,
                                                                                        p_data:
                                                                                            *const godot_vector3)
                                                                    ->
                                                                        godot_error>,
        pub godot_pool_vector3_array_invert: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *mut godot_pool_vector3_array)>,
        pub godot_pool_vector3_array_push_back: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *mut godot_pool_vector3_array,
                                                                                        p_data:
                                                                                            *const godot_vector3)>,
        pub godot_pool_vector3_array_remove: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *mut godot_pool_vector3_array,
                                                                                        p_idx:
                                                                                            godot_int)>,
        pub godot_pool_vector3_array_resize: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *mut godot_pool_vector3_array,
                                                                                        p_size:
                                                                                            godot_int)>,
        pub godot_pool_vector3_array_read: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_pool_vector3_array)
                                                                    ->
                                                                        *mut godot_pool_vector3_array_read_access>,
        pub godot_pool_vector3_array_write: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *mut godot_pool_vector3_array)
                                                                    ->
                                                                        *mut godot_pool_vector3_array_write_access>,
        pub godot_pool_vector3_array_set: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *mut godot_pool_vector3_array,
                                                                                    p_idx:
                                                                                        godot_int,
                                                                                    p_data:
                                                                                        *const godot_vector3)>,
        pub godot_pool_vector3_array_get: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_pool_vector3_array,
                                                                                    p_idx:
                                                                                        godot_int)
                                                                    ->
                                                                        godot_vector3>,
        pub godot_pool_vector3_array_size: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_pool_vector3_array)
                                                                    ->
                                                                        godot_int>,
        pub godot_pool_vector3_array_destroy: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *mut godot_pool_vector3_array)>,
        pub godot_pool_color_array_new: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                    *mut godot_pool_color_array)>,
        pub godot_pool_color_array_new_copy: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                            *mut godot_pool_color_array,
                                                                                        p_src:
                                                                                            *const godot_pool_color_array)>,
        pub godot_pool_color_array_new_with_array: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                                *mut godot_pool_color_array,
                                                                                            p_a:
                                                                                                *const godot_array)>,
        pub godot_pool_color_array_append: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *mut godot_pool_color_array,
                                                                                    p_data:
                                                                                        *const godot_color)>,
        pub godot_pool_color_array_append_array: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                                *mut godot_pool_color_array,
                                                                                            p_array:
                                                                                                *const godot_pool_color_array)>,
        pub godot_pool_color_array_insert: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *mut godot_pool_color_array,
                                                                                    p_idx:
                                                                                        godot_int,
                                                                                    p_data:
                                                                                        *const godot_color)
                                                                    ->
                                                                        godot_error>,
        pub godot_pool_color_array_invert: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *mut godot_pool_color_array)>,
        pub godot_pool_color_array_push_back: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *mut godot_pool_color_array,
                                                                                        p_data:
                                                                                            *const godot_color)>,
        pub godot_pool_color_array_remove: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *mut godot_pool_color_array,
                                                                                    p_idx:
                                                                                        godot_int)>,
        pub godot_pool_color_array_resize: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *mut godot_pool_color_array,
                                                                                    p_size:
                                                                                        godot_int)>,
        pub godot_pool_color_array_read: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_pool_color_array)
                                                                ->
                                                                    *mut godot_pool_color_array_read_access>,
        pub godot_pool_color_array_write: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *mut godot_pool_color_array)
                                                                    ->
                                                                        *mut godot_pool_color_array_write_access>,
        pub godot_pool_color_array_set: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *mut godot_pool_color_array,
                                                                                p_idx:
                                                                                    godot_int,
                                                                                p_data:
                                                                                    *const godot_color)>,
        pub godot_pool_color_array_get: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_pool_color_array,
                                                                                p_idx:
                                                                                    godot_int)
                                                                -> godot_color>,
        pub godot_pool_color_array_size: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_pool_color_array)
                                                                -> godot_int>,
        pub godot_pool_color_array_destroy: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *mut godot_pool_color_array)>,
        pub godot_pool_byte_array_read_access_copy: ::std::option::Option<unsafe extern "C" fn(p_read:
                                                                                                *const godot_pool_byte_array_read_access)
                                                                            ->
                                                                                *mut godot_pool_byte_array_read_access>,
        pub godot_pool_byte_array_read_access_ptr: ::std::option::Option<unsafe extern "C" fn(p_read:
                                                                                                *const godot_pool_byte_array_read_access)
                                                                            ->
                                                                                *const u8>,
        pub godot_pool_byte_array_read_access_operator_assign: ::std::option::Option<unsafe extern "C" fn(p_read:
                                                                                                            *mut godot_pool_byte_array_read_access,
                                                                                                        p_other:
                                                                                                            *mut godot_pool_byte_array_read_access)>,
        pub godot_pool_byte_array_read_access_destroy: ::std::option::Option<unsafe extern "C" fn(p_read:
                                                                                                    *mut godot_pool_byte_array_read_access)>,
        pub godot_pool_int_array_read_access_copy: ::std::option::Option<unsafe extern "C" fn(p_read:
                                                                                                *const godot_pool_int_array_read_access)
                                                                            ->
                                                                                *mut godot_pool_int_array_read_access>,
        pub godot_pool_int_array_read_access_ptr: ::std::option::Option<unsafe extern "C" fn(p_read:
                                                                                                *const godot_pool_int_array_read_access)
                                                                            ->
                                                                                *const godot_int>,
        pub godot_pool_int_array_read_access_operator_assign: ::std::option::Option<unsafe extern "C" fn(p_read:
                                                                                                            *mut godot_pool_int_array_read_access,
                                                                                                        p_other:
                                                                                                            *mut godot_pool_int_array_read_access)>,
        pub godot_pool_int_array_read_access_destroy: ::std::option::Option<unsafe extern "C" fn(p_read:
                                                                                                    *mut godot_pool_int_array_read_access)>,
        pub godot_pool_real_array_read_access_copy: ::std::option::Option<unsafe extern "C" fn(p_read:
                                                                                                *const godot_pool_real_array_read_access)
                                                                            ->
                                                                                *mut godot_pool_real_array_read_access>,
        pub godot_pool_real_array_read_access_ptr: ::std::option::Option<unsafe extern "C" fn(p_read:
                                                                                                *const godot_pool_real_array_read_access)
                                                                            ->
                                                                                *const godot_real>,
        pub godot_pool_real_array_read_access_operator_assign: ::std::option::Option<unsafe extern "C" fn(p_read:
                                                                                                            *mut godot_pool_real_array_read_access,
                                                                                                        p_other:
                                                                                                            *mut godot_pool_real_array_read_access)>,
        pub godot_pool_real_array_read_access_destroy: ::std::option::Option<unsafe extern "C" fn(p_read:
                                                                                                    *mut godot_pool_real_array_read_access)>,
        pub godot_pool_string_array_read_access_copy: ::std::option::Option<unsafe extern "C" fn(p_read:
                                                                                                    *const godot_pool_string_array_read_access)
                                                                                ->
                                                                                    *mut godot_pool_string_array_read_access>,
        pub godot_pool_string_array_read_access_ptr: ::std::option::Option<unsafe extern "C" fn(p_read:
                                                                                                    *const godot_pool_string_array_read_access)
                                                                            ->
                                                                                *const godot_string>,
        pub godot_pool_string_array_read_access_operator_assign: ::std::option::Option<unsafe extern "C" fn(p_read:
                                                                                                                *mut godot_pool_string_array_read_access,
                                                                                                            p_other:
                                                                                                                *mut godot_pool_string_array_read_access)>,
        pub godot_pool_string_array_read_access_destroy: ::std::option::Option<unsafe extern "C" fn(p_read:
                                                                                                        *mut godot_pool_string_array_read_access)>,
        pub godot_pool_vector2_array_read_access_copy: ::std::option::Option<unsafe extern "C" fn(p_read:
                                                                                                    *const godot_pool_vector2_array_read_access)
                                                                                ->
                                                                                    *mut godot_pool_vector2_array_read_access>,
        pub godot_pool_vector2_array_read_access_ptr: ::std::option::Option<unsafe extern "C" fn(p_read:
                                                                                                    *const godot_pool_vector2_array_read_access)
                                                                                ->
                                                                                    *const godot_vector2>,
        pub godot_pool_vector2_array_read_access_operator_assign: ::std::option::Option<unsafe extern "C" fn(p_read:
                                                                                                                *mut godot_pool_vector2_array_read_access,
                                                                                                            p_other:
                                                                                                                *mut godot_pool_vector2_array_read_access)>,
        pub godot_pool_vector2_array_read_access_destroy: ::std::option::Option<unsafe extern "C" fn(p_read:
                                                                                                        *mut godot_pool_vector2_array_read_access)>,
        pub godot_pool_vector3_array_read_access_copy: ::std::option::Option<unsafe extern "C" fn(p_read:
                                                                                                    *const godot_pool_vector3_array_read_access)
                                                                                ->
                                                                                    *mut godot_pool_vector3_array_read_access>,
        pub godot_pool_vector3_array_read_access_ptr: ::std::option::Option<unsafe extern "C" fn(p_read:
                                                                                                    *const godot_pool_vector3_array_read_access)
                                                                                ->
                                                                                    *const godot_vector3>,
        pub godot_pool_vector3_array_read_access_operator_assign: ::std::option::Option<unsafe extern "C" fn(p_read:
                                                                                                                *mut godot_pool_vector3_array_read_access,
                                                                                                            p_other:
                                                                                                                *mut godot_pool_vector3_array_read_access)>,
        pub godot_pool_vector3_array_read_access_destroy: ::std::option::Option<unsafe extern "C" fn(p_read:
                                                                                                        *mut godot_pool_vector3_array_read_access)>,
        pub godot_pool_color_array_read_access_copy: ::std::option::Option<unsafe extern "C" fn(p_read:
                                                                                                    *const godot_pool_color_array_read_access)
                                                                            ->
                                                                                *mut godot_pool_color_array_read_access>,
        pub godot_pool_color_array_read_access_ptr: ::std::option::Option<unsafe extern "C" fn(p_read:
                                                                                                *const godot_pool_color_array_read_access)
                                                                            ->
                                                                                *const godot_color>,
        pub godot_pool_color_array_read_access_operator_assign: ::std::option::Option<unsafe extern "C" fn(p_read:
                                                                                                            *mut godot_pool_color_array_read_access,
                                                                                                        p_other:
                                                                                                            *mut godot_pool_color_array_read_access)>,
        pub godot_pool_color_array_read_access_destroy: ::std::option::Option<unsafe extern "C" fn(p_read:
                                                                                                    *mut godot_pool_color_array_read_access)>,
        pub godot_pool_byte_array_write_access_copy: ::std::option::Option<unsafe extern "C" fn(p_write:
                                                                                                    *const godot_pool_byte_array_write_access)
                                                                            ->
                                                                                *mut godot_pool_byte_array_write_access>,
        pub godot_pool_byte_array_write_access_ptr: ::std::option::Option<unsafe extern "C" fn(p_write:
                                                                                                *const godot_pool_byte_array_write_access)
                                                                            ->
                                                                                *mut u8>,
        pub godot_pool_byte_array_write_access_operator_assign: ::std::option::Option<unsafe extern "C" fn(p_write:
                                                                                                            *mut godot_pool_byte_array_write_access,
                                                                                                        p_other:
                                                                                                            *mut godot_pool_byte_array_write_access)>,
        pub godot_pool_byte_array_write_access_destroy: ::std::option::Option<unsafe extern "C" fn(p_write:
                                                                                                    *mut godot_pool_byte_array_write_access)>,
        pub godot_pool_int_array_write_access_copy: ::std::option::Option<unsafe extern "C" fn(p_write:
                                                                                                *const godot_pool_int_array_write_access)
                                                                            ->
                                                                                *mut godot_pool_int_array_write_access>,
        pub godot_pool_int_array_write_access_ptr: ::std::option::Option<unsafe extern "C" fn(p_write:
                                                                                                *const godot_pool_int_array_write_access)
                                                                            ->
                                                                                *mut godot_int>,
        pub godot_pool_int_array_write_access_operator_assign: ::std::option::Option<unsafe extern "C" fn(p_write:
                                                                                                            *mut godot_pool_int_array_write_access,
                                                                                                        p_other:
                                                                                                            *mut godot_pool_int_array_write_access)>,
        pub godot_pool_int_array_write_access_destroy: ::std::option::Option<unsafe extern "C" fn(p_write:
                                                                                                    *mut godot_pool_int_array_write_access)>,
        pub godot_pool_real_array_write_access_copy: ::std::option::Option<unsafe extern "C" fn(p_write:
                                                                                                    *const godot_pool_real_array_write_access)
                                                                            ->
                                                                                *mut godot_pool_real_array_write_access>,
        pub godot_pool_real_array_write_access_ptr: ::std::option::Option<unsafe extern "C" fn(p_write:
                                                                                                *const godot_pool_real_array_write_access)
                                                                            ->
                                                                                *mut godot_real>,
        pub godot_pool_real_array_write_access_operator_assign: ::std::option::Option<unsafe extern "C" fn(p_write:
                                                                                                            *mut godot_pool_real_array_write_access,
                                                                                                        p_other:
                                                                                                            *mut godot_pool_real_array_write_access)>,
        pub godot_pool_real_array_write_access_destroy: ::std::option::Option<unsafe extern "C" fn(p_write:
                                                                                                    *mut godot_pool_real_array_write_access)>,
        pub godot_pool_string_array_write_access_copy: ::std::option::Option<unsafe extern "C" fn(p_write:
                                                                                                    *const godot_pool_string_array_write_access)
                                                                                ->
                                                                                    *mut godot_pool_string_array_write_access>,
        pub godot_pool_string_array_write_access_ptr: ::std::option::Option<unsafe extern "C" fn(p_write:
                                                                                                    *const godot_pool_string_array_write_access)
                                                                                ->
                                                                                    *mut godot_string>,
        pub godot_pool_string_array_write_access_operator_assign: ::std::option::Option<unsafe extern "C" fn(p_write:
                                                                                                                *mut godot_pool_string_array_write_access,
                                                                                                            p_other:
                                                                                                                *mut godot_pool_string_array_write_access)>,
        pub godot_pool_string_array_write_access_destroy: ::std::option::Option<unsafe extern "C" fn(p_write:
                                                                                                        *mut godot_pool_string_array_write_access)>,
        pub godot_pool_vector2_array_write_access_copy: ::std::option::Option<unsafe extern "C" fn(p_write:
                                                                                                    *const godot_pool_vector2_array_write_access)
                                                                                ->
                                                                                    *mut godot_pool_vector2_array_write_access>,
        pub godot_pool_vector2_array_write_access_ptr: ::std::option::Option<unsafe extern "C" fn(p_write:
                                                                                                    *const godot_pool_vector2_array_write_access)
                                                                                ->
                                                                                    *mut godot_vector2>,
        pub godot_pool_vector2_array_write_access_operator_assign: ::std::option::Option<unsafe extern "C" fn(p_write:
                                                                                                                *mut godot_pool_vector2_array_write_access,
                                                                                                            p_other:
                                                                                                                *mut godot_pool_vector2_array_write_access)>,
        pub godot_pool_vector2_array_write_access_destroy: ::std::option::Option<unsafe extern "C" fn(p_write:
                                                                                                        *mut godot_pool_vector2_array_write_access)>,
        pub godot_pool_vector3_array_write_access_copy: ::std::option::Option<unsafe extern "C" fn(p_write:
                                                                                                    *const godot_pool_vector3_array_write_access)
                                                                                ->
                                                                                    *mut godot_pool_vector3_array_write_access>,
        pub godot_pool_vector3_array_write_access_ptr: ::std::option::Option<unsafe extern "C" fn(p_write:
                                                                                                    *const godot_pool_vector3_array_write_access)
                                                                                ->
                                                                                    *mut godot_vector3>,
        pub godot_pool_vector3_array_write_access_operator_assign: ::std::option::Option<unsafe extern "C" fn(p_write:
                                                                                                                *mut godot_pool_vector3_array_write_access,
                                                                                                            p_other:
                                                                                                                *mut godot_pool_vector3_array_write_access)>,
        pub godot_pool_vector3_array_write_access_destroy: ::std::option::Option<unsafe extern "C" fn(p_write:
                                                                                                        *mut godot_pool_vector3_array_write_access)>,
        pub godot_pool_color_array_write_access_copy: ::std::option::Option<unsafe extern "C" fn(p_write:
                                                                                                    *const godot_pool_color_array_write_access)
                                                                                ->
                                                                                    *mut godot_pool_color_array_write_access>,
        pub godot_pool_color_array_write_access_ptr: ::std::option::Option<unsafe extern "C" fn(p_write:
                                                                                                    *const godot_pool_color_array_write_access)
                                                                            ->
                                                                                *mut godot_color>,
        pub godot_pool_color_array_write_access_operator_assign: ::std::option::Option<unsafe extern "C" fn(p_write:
                                                                                                                *mut godot_pool_color_array_write_access,
                                                                                                            p_other:
                                                                                                                *mut godot_pool_color_array_write_access)>,
        pub godot_pool_color_array_write_access_destroy: ::std::option::Option<unsafe extern "C" fn(p_write:
                                                                                                        *mut godot_pool_color_array_write_access)>,
        pub godot_array_new: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                            *mut godot_array)>,
        pub godot_array_new_copy: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                *mut godot_array,
                                                                            p_src:
                                                                                *const godot_array)>,
        pub godot_array_new_pool_color_array: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                            *mut godot_array,
                                                                                        p_pca:
                                                                                            *const godot_pool_color_array)>,
        pub godot_array_new_pool_vector3_array: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                            *mut godot_array,
                                                                                        p_pv3a:
                                                                                            *const godot_pool_vector3_array)>,
        pub godot_array_new_pool_vector2_array: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                            *mut godot_array,
                                                                                        p_pv2a:
                                                                                            *const godot_pool_vector2_array)>,
        pub godot_array_new_pool_string_array: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                            *mut godot_array,
                                                                                        p_psa:
                                                                                            *const godot_pool_string_array)>,
        pub godot_array_new_pool_real_array: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                            *mut godot_array,
                                                                                        p_pra:
                                                                                            *const godot_pool_real_array)>,
        pub godot_array_new_pool_int_array: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                        *mut godot_array,
                                                                                    p_pia:
                                                                                        *const godot_pool_int_array)>,
        pub godot_array_new_pool_byte_array: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                            *mut godot_array,
                                                                                        p_pba:
                                                                                            *const godot_pool_byte_array)>,
        pub godot_array_set: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *mut godot_array,
                                                                        p_idx:
                                                                            godot_int,
                                                                        p_value:
                                                                            *const godot_variant)>,
        pub godot_array_get: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_array,
                                                                        p_idx:
                                                                            godot_int)
                                                    -> godot_variant>,
        pub godot_array_operator_index: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *mut godot_array,
                                                                                p_idx:
                                                                                    godot_int)
                                                                ->
                                                                    *mut godot_variant>,
        pub godot_array_operator_index_const: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *const godot_array,
                                                                                        p_idx:
                                                                                            godot_int)
                                                                        ->
                                                                            *const godot_variant>,
        pub godot_array_append: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *mut godot_array,
                                                                        p_value:
                                                                            *const godot_variant)>,
        pub godot_array_clear: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *mut godot_array)>,
        pub godot_array_count: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_array,
                                                                        p_value:
                                                                            *const godot_variant)
                                                        -> godot_int>,
        pub godot_array_empty: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_array)
                                                        -> godot_bool>,
        pub godot_array_erase: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *mut godot_array,
                                                                        p_value:
                                                                            *const godot_variant)>,
        pub godot_array_front: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_array)
                                                        -> godot_variant>,
        pub godot_array_back: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_array)
                                                        -> godot_variant>,
        pub godot_array_find: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_array,
                                                                        p_what:
                                                                            *const godot_variant,
                                                                        p_from:
                                                                            godot_int)
                                                        -> godot_int>,
        pub godot_array_find_last: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_array,
                                                                            p_what:
                                                                                *const godot_variant)
                                                            -> godot_int>,
        pub godot_array_has: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_array,
                                                                        p_value:
                                                                            *const godot_variant)
                                                    -> godot_bool>,
        pub godot_array_hash: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_array)
                                                        -> godot_int>,
        pub godot_array_insert: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *mut godot_array,
                                                                        p_pos:
                                                                            godot_int,
                                                                        p_value:
                                                                            *const godot_variant)>,
        pub godot_array_invert: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *mut godot_array)>,
        pub godot_array_pop_back: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *mut godot_array)
                                                            -> godot_variant>,
        pub godot_array_pop_front: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *mut godot_array)
                                                            -> godot_variant>,
        pub godot_array_push_back: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *mut godot_array,
                                                                            p_value:
                                                                                *const godot_variant)>,
        pub godot_array_push_front: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *mut godot_array,
                                                                            p_value:
                                                                                *const godot_variant)>,
        pub godot_array_remove: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *mut godot_array,
                                                                        p_idx:
                                                                            godot_int)>,
        pub godot_array_resize: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *mut godot_array,
                                                                        p_size:
                                                                            godot_int)>,
        pub godot_array_rfind: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_array,
                                                                        p_what:
                                                                            *const godot_variant,
                                                                        p_from:
                                                                            godot_int)
                                                        -> godot_int>,
        pub godot_array_size: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_array)
                                                        -> godot_int>,
        pub godot_array_sort: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *mut godot_array)>,
        pub godot_array_sort_custom: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *mut godot_array,
                                                                                p_obj:
                                                                                    *mut godot_object,
                                                                                p_func:
                                                                                    *const godot_string)>,
        pub godot_array_bsearch: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *mut godot_array,
                                                                            p_value:
                                                                                *const godot_variant,
                                                                            p_before:
                                                                                godot_bool)
                                                        -> godot_int>,
        pub godot_array_bsearch_custom: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *mut godot_array,
                                                                                p_value:
                                                                                    *const godot_variant,
                                                                                p_obj:
                                                                                    *mut godot_object,
                                                                                p_func:
                                                                                    *const godot_string,
                                                                                p_before:
                                                                                    godot_bool)
                                                                -> godot_int>,
        pub godot_array_destroy: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *mut godot_array)>,
        pub godot_dictionary_new: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                *mut godot_dictionary)>,
        pub godot_dictionary_new_copy: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                    *mut godot_dictionary,
                                                                                p_src:
                                                                                    *const godot_dictionary)>,
        pub godot_dictionary_destroy: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *mut godot_dictionary)>,
        pub godot_dictionary_size: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_dictionary)
                                                            -> godot_int>,
        pub godot_dictionary_empty: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_dictionary)
                                                            -> godot_bool>,
        pub godot_dictionary_clear: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *mut godot_dictionary)>,
        pub godot_dictionary_has: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_dictionary,
                                                                            p_key:
                                                                                *const godot_variant)
                                                            -> godot_bool>,
        pub godot_dictionary_has_all: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_dictionary,
                                                                                p_keys:
                                                                                    *const godot_array)
                                                                -> godot_bool>,
        pub godot_dictionary_erase: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *mut godot_dictionary,
                                                                            p_key:
                                                                                *const godot_variant)>,
        pub godot_dictionary_hash: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_dictionary)
                                                            -> godot_int>,
        pub godot_dictionary_keys: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_dictionary)
                                                            -> godot_array>,
        pub godot_dictionary_values: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_dictionary)
                                                            -> godot_array>,
        pub godot_dictionary_get: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_dictionary,
                                                                            p_key:
                                                                                *const godot_variant)
                                                            -> godot_variant>,
        pub godot_dictionary_set: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *mut godot_dictionary,
                                                                            p_key:
                                                                                *const godot_variant,
                                                                            p_value:
                                                                                *const godot_variant)>,
        pub godot_dictionary_operator_index: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *mut godot_dictionary,
                                                                                        p_key:
                                                                                            *const godot_variant)
                                                                    ->
                                                                        *mut godot_variant>,
        pub godot_dictionary_operator_index_const: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                                *const godot_dictionary,
                                                                                            p_key:
                                                                                                *const godot_variant)
                                                                            ->
                                                                                *const godot_variant>,
        pub godot_dictionary_next: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_dictionary,
                                                                            p_key:
                                                                                *const godot_variant)
                                                            ->
                                                                *mut godot_variant>,
        pub godot_dictionary_operator_equal: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *const godot_dictionary,
                                                                                        p_b:
                                                                                            *const godot_dictionary)
                                                                    ->
                                                                        godot_bool>,
        pub godot_dictionary_to_json: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_dictionary)
                                                                -> godot_string>,
        pub godot_node_path_new: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                *mut godot_node_path,
                                                                            p_from:
                                                                                *const godot_string)>,
        pub godot_node_path_new_copy: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                    *mut godot_node_path,
                                                                                p_src:
                                                                                    *const godot_node_path)>,
        pub godot_node_path_destroy: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *mut godot_node_path)>,
        pub godot_node_path_as_string: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_node_path)
                                                                -> godot_string>,
        pub godot_node_path_is_absolute: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_node_path)
                                                                -> godot_bool>,
        pub godot_node_path_get_name_count: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_node_path)
                                                                    ->
                                                                        godot_int>,
        pub godot_node_path_get_name: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_node_path,
                                                                                p_idx:
                                                                                    godot_int)
                                                                -> godot_string>,
        pub godot_node_path_get_subname_count: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *const godot_node_path)
                                                                        ->
                                                                            godot_int>,
        pub godot_node_path_get_subname: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_node_path,
                                                                                    p_idx:
                                                                                        godot_int)
                                                                ->
                                                                    godot_string>,
        pub godot_node_path_get_concatenated_subnames: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                                    *const godot_node_path)
                                                                                ->
                                                                                    godot_string>,
        pub godot_node_path_is_empty: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_node_path)
                                                                -> godot_bool>,
        pub godot_node_path_operator_equal: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_node_path,
                                                                                    p_b:
                                                                                        *const godot_node_path)
                                                                    ->
                                                                        godot_bool>,
        pub godot_plane_new_with_reals: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                    *mut godot_plane,
                                                                                p_a:
                                                                                    godot_real,
                                                                                p_b:
                                                                                    godot_real,
                                                                                p_c:
                                                                                    godot_real,
                                                                                p_d:
                                                                                    godot_real)>,
        pub godot_plane_new_with_vectors: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                        *mut godot_plane,
                                                                                    p_v1:
                                                                                        *const godot_vector3,
                                                                                    p_v2:
                                                                                        *const godot_vector3,
                                                                                    p_v3:
                                                                                        *const godot_vector3)>,
        pub godot_plane_new_with_normal: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                        *mut godot_plane,
                                                                                    p_normal:
                                                                                        *const godot_vector3,
                                                                                    p_d:
                                                                                        godot_real)>,
        pub godot_plane_as_string: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_plane)
                                                            -> godot_string>,
        pub godot_plane_normalized: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_plane)
                                                            -> godot_plane>,
        pub godot_plane_center: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_plane)
                                                        -> godot_vector3>,
        pub godot_plane_get_any_point: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_plane)
                                                                ->
                                                                    godot_vector3>,
        pub godot_plane_is_point_over: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_plane,
                                                                                p_point:
                                                                                    *const godot_vector3)
                                                                -> godot_bool>,
        pub godot_plane_distance_to: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_plane,
                                                                                p_point:
                                                                                    *const godot_vector3)
                                                            -> godot_real>,
        pub godot_plane_has_point: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_plane,
                                                                            p_point:
                                                                                *const godot_vector3,
                                                                            p_epsilon:
                                                                                godot_real)
                                                            -> godot_bool>,
        pub godot_plane_project: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_plane,
                                                                            p_point:
                                                                                *const godot_vector3)
                                                        -> godot_vector3>,
        pub godot_plane_intersect_3: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_plane,
                                                                                r_dest:
                                                                                    *mut godot_vector3,
                                                                                p_b:
                                                                                    *const godot_plane,
                                                                                p_c:
                                                                                    *const godot_plane)
                                                            -> godot_bool>,
        pub godot_plane_intersects_ray: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_plane,
                                                                                r_dest:
                                                                                    *mut godot_vector3,
                                                                                p_from:
                                                                                    *const godot_vector3,
                                                                                p_dir:
                                                                                    *const godot_vector3)
                                                                -> godot_bool>,
        pub godot_plane_intersects_segment: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_plane,
                                                                                    r_dest:
                                                                                        *mut godot_vector3,
                                                                                    p_begin:
                                                                                        *const godot_vector3,
                                                                                    p_end:
                                                                                        *const godot_vector3)
                                                                    ->
                                                                        godot_bool>,
        pub godot_plane_operator_neg: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_plane)
                                                                -> godot_plane>,
        pub godot_plane_operator_equal: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_plane,
                                                                                p_b:
                                                                                    *const godot_plane)
                                                                -> godot_bool>,
        pub godot_plane_set_normal: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *mut godot_plane,
                                                                            p_normal:
                                                                                *const godot_vector3)>,
        pub godot_plane_get_normal: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_plane)
                                                            -> godot_vector3>,
        pub godot_plane_get_d: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_plane)
                                                        -> godot_real>,
        pub godot_plane_set_d: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *mut godot_plane,
                                                                        p_d:
                                                                            godot_real)>,
        pub godot_rect2_new_with_position_and_size: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                                *mut godot_rect2,
                                                                                            p_pos:
                                                                                                *const godot_vector2,
                                                                                            p_size:
                                                                                                *const godot_vector2)>,
        pub godot_rect2_new: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                            *mut godot_rect2,
                                                                        p_x:
                                                                            godot_real,
                                                                        p_y:
                                                                            godot_real,
                                                                        p_width:
                                                                            godot_real,
                                                                        p_height:
                                                                            godot_real)>,
        pub godot_rect2_as_string: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_rect2)
                                                            -> godot_string>,
        pub godot_rect2_get_area: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_rect2)
                                                            -> godot_real>,
        pub godot_rect2_intersects: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_rect2,
                                                                            p_b:
                                                                                *const godot_rect2)
                                                            -> godot_bool>,
        pub godot_rect2_encloses: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_rect2,
                                                                            p_b:
                                                                                *const godot_rect2)
                                                            -> godot_bool>,
        pub godot_rect2_has_no_area: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_rect2)
                                                            -> godot_bool>,
        pub godot_rect2_clip: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_rect2,
                                                                        p_b:
                                                                            *const godot_rect2)
                                                        -> godot_rect2>,
        pub godot_rect2_merge: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_rect2,
                                                                        p_b:
                                                                            *const godot_rect2)
                                                        -> godot_rect2>,
        pub godot_rect2_has_point: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_rect2,
                                                                            p_point:
                                                                                *const godot_vector2)
                                                            -> godot_bool>,
        pub godot_rect2_grow: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_rect2,
                                                                        p_by:
                                                                            godot_real)
                                                        -> godot_rect2>,
        pub godot_rect2_expand: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_rect2,
                                                                        p_to:
                                                                            *const godot_vector2)
                                                        -> godot_rect2>,
        pub godot_rect2_operator_equal: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_rect2,
                                                                                p_b:
                                                                                    *const godot_rect2)
                                                                -> godot_bool>,
        pub godot_rect2_get_position: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_rect2)
                                                                -> godot_vector2>,
        pub godot_rect2_get_size: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_rect2)
                                                            -> godot_vector2>,
        pub godot_rect2_set_position: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *mut godot_rect2,
                                                                                p_pos:
                                                                                    *const godot_vector2)>,
        pub godot_rect2_set_size: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *mut godot_rect2,
                                                                            p_size:
                                                                                *const godot_vector2)>,
        pub godot_aabb_new: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                        *mut godot_aabb,
                                                                    p_pos:
                                                                        *const godot_vector3,
                                                                    p_size:
                                                                        *const godot_vector3)>,
        pub godot_aabb_get_position: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_aabb)
                                                            -> godot_vector3>,
        pub godot_aabb_set_position: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_aabb,
                                                                                p_v:
                                                                                    *const godot_vector3)>,
        pub godot_aabb_get_size: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_aabb)
                                                        -> godot_vector3>,
        pub godot_aabb_set_size: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_aabb,
                                                                            p_v:
                                                                                *const godot_vector3)>,
        pub godot_aabb_as_string: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_aabb)
                                                            -> godot_string>,
        pub godot_aabb_get_area: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_aabb)
                                                        -> godot_real>,
        pub godot_aabb_has_no_area: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_aabb)
                                                            -> godot_bool>,
        pub godot_aabb_has_no_surface: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_aabb)
                                                                -> godot_bool>,
        pub godot_aabb_intersects: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_aabb,
                                                                            p_with:
                                                                                *const godot_aabb)
                                                            -> godot_bool>,
        pub godot_aabb_encloses: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_aabb,
                                                                            p_with:
                                                                                *const godot_aabb)
                                                        -> godot_bool>,
        pub godot_aabb_merge: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_aabb,
                                                                        p_with:
                                                                            *const godot_aabb)
                                                        -> godot_aabb>,
        pub godot_aabb_intersection: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_aabb,
                                                                                p_with:
                                                                                    *const godot_aabb)
                                                            -> godot_aabb>,
        pub godot_aabb_intersects_plane: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_aabb,
                                                                                    p_plane:
                                                                                        *const godot_plane)
                                                                -> godot_bool>,
        pub godot_aabb_intersects_segment: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_aabb,
                                                                                    p_from:
                                                                                        *const godot_vector3,
                                                                                    p_to:
                                                                                        *const godot_vector3)
                                                                    ->
                                                                        godot_bool>,
        pub godot_aabb_has_point: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_aabb,
                                                                            p_point:
                                                                                *const godot_vector3)
                                                            -> godot_bool>,
        pub godot_aabb_get_support: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_aabb,
                                                                            p_dir:
                                                                                *const godot_vector3)
                                                            -> godot_vector3>,
        pub godot_aabb_get_longest_axis: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_aabb)
                                                                ->
                                                                    godot_vector3>,
        pub godot_aabb_get_longest_axis_index: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *const godot_aabb)
                                                                        ->
                                                                            godot_int>,
        pub godot_aabb_get_longest_axis_size: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *const godot_aabb)
                                                                        ->
                                                                            godot_real>,
        pub godot_aabb_get_shortest_axis: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_aabb)
                                                                    ->
                                                                        godot_vector3>,
        pub godot_aabb_get_shortest_axis_index: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *const godot_aabb)
                                                                        ->
                                                                            godot_int>,
        pub godot_aabb_get_shortest_axis_size: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *const godot_aabb)
                                                                        ->
                                                                            godot_real>,
        pub godot_aabb_expand: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_aabb,
                                                                        p_to_point:
                                                                            *const godot_vector3)
                                                        -> godot_aabb>,
        pub godot_aabb_grow: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_aabb,
                                                                        p_by:
                                                                            godot_real)
                                                    -> godot_aabb>,
        pub godot_aabb_get_endpoint: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_aabb,
                                                                                p_idx:
                                                                                    godot_int)
                                                            -> godot_vector3>,
        pub godot_aabb_operator_equal: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_aabb,
                                                                                p_b:
                                                                                    *const godot_aabb)
                                                                -> godot_bool>,
        pub godot_rid_new: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                        *mut godot_rid)>,
        pub godot_rid_get_id: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_rid)
                                                        -> godot_int>,
        pub godot_rid_new_with_resource: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                        *mut godot_rid,
                                                                                    p_from:
                                                                                        *const godot_object)>,
        pub godot_rid_operator_equal: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_rid,
                                                                                p_b:
                                                                                    *const godot_rid)
                                                                -> godot_bool>,
        pub godot_rid_operator_less: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_rid,
                                                                                p_b:
                                                                                    *const godot_rid)
                                                            -> godot_bool>,
        pub godot_transform_new_with_axis_origin: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                                *mut godot_transform,
                                                                                            p_x_axis:
                                                                                                *const godot_vector3,
                                                                                            p_y_axis:
                                                                                                *const godot_vector3,
                                                                                            p_z_axis:
                                                                                                *const godot_vector3,
                                                                                            p_origin:
                                                                                                *const godot_vector3)>,
        pub godot_transform_new: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                *mut godot_transform,
                                                                            p_basis:
                                                                                *const godot_basis,
                                                                            p_origin:
                                                                                *const godot_vector3)>,
        pub godot_transform_get_basis: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_transform)
                                                                -> godot_basis>,
        pub godot_transform_set_basis: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *mut godot_transform,
                                                                                p_v:
                                                                                    *const godot_basis)>,
        pub godot_transform_get_origin: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_transform)
                                                                ->
                                                                    godot_vector3>,
        pub godot_transform_set_origin: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *mut godot_transform,
                                                                                p_v:
                                                                                    *const godot_vector3)>,
        pub godot_transform_as_string: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_transform)
                                                                -> godot_string>,
        pub godot_transform_inverse: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_transform)
                                                            ->
                                                                godot_transform>,
        pub godot_transform_affine_inverse: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_transform)
                                                                    ->
                                                                        godot_transform>,
        pub godot_transform_orthonormalized: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *const godot_transform)
                                                                    ->
                                                                        godot_transform>,
        pub godot_transform_rotated: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_transform,
                                                                                p_axis:
                                                                                    *const godot_vector3,
                                                                                p_phi:
                                                                                    godot_real)
                                                            ->
                                                                godot_transform>,
        pub godot_transform_scaled: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_transform,
                                                                            p_scale:
                                                                                *const godot_vector3)
                                                            -> godot_transform>,
        pub godot_transform_translated: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_transform,
                                                                                p_ofs:
                                                                                    *const godot_vector3)
                                                                ->
                                                                    godot_transform>,
        pub godot_transform_looking_at: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_transform,
                                                                                p_target:
                                                                                    *const godot_vector3,
                                                                                p_up:
                                                                                    *const godot_vector3)
                                                                ->
                                                                    godot_transform>,
        pub godot_transform_xform_plane: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_transform,
                                                                                    p_v:
                                                                                        *const godot_plane)
                                                                ->
                                                                    godot_plane>,
        pub godot_transform_xform_inv_plane: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *const godot_transform,
                                                                                        p_v:
                                                                                            *const godot_plane)
                                                                    ->
                                                                        godot_plane>,
        pub godot_transform_new_identity: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                        *mut godot_transform)>,
        pub godot_transform_operator_equal: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_transform,
                                                                                    p_b:
                                                                                        *const godot_transform)
                                                                    ->
                                                                        godot_bool>,
        pub godot_transform_operator_multiply: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *const godot_transform,
                                                                                        p_b:
                                                                                            *const godot_transform)
                                                                        ->
                                                                            godot_transform>,
        pub godot_transform_xform_vector3: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_transform,
                                                                                    p_v:
                                                                                        *const godot_vector3)
                                                                    ->
                                                                        godot_vector3>,
        pub godot_transform_xform_inv_vector3: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *const godot_transform,
                                                                                        p_v:
                                                                                            *const godot_vector3)
                                                                        ->
                                                                            godot_vector3>,
        pub godot_transform_xform_aabb: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_transform,
                                                                                p_v:
                                                                                    *const godot_aabb)
                                                                -> godot_aabb>,
        pub godot_transform_xform_inv_aabb: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_transform,
                                                                                    p_v:
                                                                                        *const godot_aabb)
                                                                    ->
                                                                        godot_aabb>,
        pub godot_transform2d_new: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                *mut godot_transform2d,
                                                                            p_rot:
                                                                                godot_real,
                                                                            p_pos:
                                                                                *const godot_vector2)>,
        pub godot_transform2d_new_axis_origin: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                            *mut godot_transform2d,
                                                                                        p_x_axis:
                                                                                            *const godot_vector2,
                                                                                        p_y_axis:
                                                                                            *const godot_vector2,
                                                                                        p_origin:
                                                                                            *const godot_vector2)>,
        pub godot_transform2d_as_string: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_transform2d)
                                                                ->
                                                                    godot_string>,
        pub godot_transform2d_inverse: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_transform2d)
                                                                ->
                                                                    godot_transform2d>,
        pub godot_transform2d_affine_inverse: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *const godot_transform2d)
                                                                        ->
                                                                            godot_transform2d>,
        pub godot_transform2d_get_rotation: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_transform2d)
                                                                    ->
                                                                        godot_real>,
        pub godot_transform2d_get_origin: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_transform2d)
                                                                    ->
                                                                        godot_vector2>,
        pub godot_transform2d_get_scale: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_transform2d)
                                                                ->
                                                                    godot_vector2>,
        pub godot_transform2d_orthonormalized: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *const godot_transform2d)
                                                                        ->
                                                                            godot_transform2d>,
        pub godot_transform2d_rotated: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_transform2d,
                                                                                p_phi:
                                                                                    godot_real)
                                                                ->
                                                                    godot_transform2d>,
        pub godot_transform2d_scaled: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_transform2d,
                                                                                p_scale:
                                                                                    *const godot_vector2)
                                                                ->
                                                                    godot_transform2d>,
        pub godot_transform2d_translated: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_transform2d,
                                                                                    p_offset:
                                                                                        *const godot_vector2)
                                                                    ->
                                                                        godot_transform2d>,
        pub godot_transform2d_xform_vector2: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *const godot_transform2d,
                                                                                        p_v:
                                                                                            *const godot_vector2)
                                                                    ->
                                                                        godot_vector2>,
        pub godot_transform2d_xform_inv_vector2: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                                *const godot_transform2d,
                                                                                            p_v:
                                                                                                *const godot_vector2)
                                                                        ->
                                                                            godot_vector2>,
        pub godot_transform2d_basis_xform_vector2: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                                *const godot_transform2d,
                                                                                            p_v:
                                                                                                *const godot_vector2)
                                                                            ->
                                                                                godot_vector2>,
        pub godot_transform2d_basis_xform_inv_vector2: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                                    *const godot_transform2d,
                                                                                                p_v:
                                                                                                    *const godot_vector2)
                                                                                ->
                                                                                    godot_vector2>,
        pub godot_transform2d_interpolate_with: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *const godot_transform2d,
                                                                                        p_m:
                                                                                            *const godot_transform2d,
                                                                                        p_c:
                                                                                            godot_real)
                                                                        ->
                                                                            godot_transform2d>,
        pub godot_transform2d_operator_equal: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *const godot_transform2d,
                                                                                        p_b:
                                                                                            *const godot_transform2d)
                                                                        ->
                                                                            godot_bool>,
        pub godot_transform2d_operator_multiply: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                                *const godot_transform2d,
                                                                                            p_b:
                                                                                                *const godot_transform2d)
                                                                        ->
                                                                            godot_transform2d>,
        pub godot_transform2d_new_identity: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                        *mut godot_transform2d)>,
        pub godot_transform2d_xform_rect2: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_transform2d,
                                                                                    p_v:
                                                                                        *const godot_rect2)
                                                                    ->
                                                                        godot_rect2>,
        pub godot_transform2d_xform_inv_rect2: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *const godot_transform2d,
                                                                                        p_v:
                                                                                            *const godot_rect2)
                                                                        ->
                                                                            godot_rect2>,
        pub godot_variant_get_type: ::std::option::Option<unsafe extern "C" fn(p_v:
                                                                                *const godot_variant)
                                                            ->
                                                                godot_variant_type>,
        pub godot_variant_new_copy: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                *mut godot_variant,
                                                                            p_src:
                                                                                *const godot_variant)>,
        pub godot_variant_new_nil: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                *mut godot_variant)>,
        pub godot_variant_new_bool: ::std::option::Option<unsafe extern "C" fn(p_v:
                                                                                *mut godot_variant,
                                                                            p_b:
                                                                                godot_bool)>,
        pub godot_variant_new_uint: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                *mut godot_variant,
                                                                            p_i:
                                                                                u64)>,
        pub godot_variant_new_int: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                *mut godot_variant,
                                                                            p_i:
                                                                                i64)>,
        pub godot_variant_new_real: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                *mut godot_variant,
                                                                            p_r:
                                                                                f64)>,
        pub godot_variant_new_string: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                    *mut godot_variant,
                                                                                p_s:
                                                                                    *const godot_string)>,
        pub godot_variant_new_vector2: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                    *mut godot_variant,
                                                                                p_v2:
                                                                                    *const godot_vector2)>,
        pub godot_variant_new_rect2: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                    *mut godot_variant,
                                                                                p_rect2:
                                                                                    *const godot_rect2)>,
        pub godot_variant_new_vector3: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                    *mut godot_variant,
                                                                                p_v3:
                                                                                    *const godot_vector3)>,
        pub godot_variant_new_transform2d: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                        *mut godot_variant,
                                                                                    p_t2d:
                                                                                        *const godot_transform2d)>,
        pub godot_variant_new_plane: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                    *mut godot_variant,
                                                                                p_plane:
                                                                                    *const godot_plane)>,
        pub godot_variant_new_quat: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                *mut godot_variant,
                                                                            p_quat:
                                                                                *const godot_quat)>,
        pub godot_variant_new_aabb: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                *mut godot_variant,
                                                                            p_aabb:
                                                                                *const godot_aabb)>,
        pub godot_variant_new_basis: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                    *mut godot_variant,
                                                                                p_basis:
                                                                                    *const godot_basis)>,
        pub godot_variant_new_transform: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                        *mut godot_variant,
                                                                                    p_trans:
                                                                                        *const godot_transform)>,
        pub godot_variant_new_color: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                    *mut godot_variant,
                                                                                p_color:
                                                                                    *const godot_color)>,
        pub godot_variant_new_node_path: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                        *mut godot_variant,
                                                                                    p_np:
                                                                                        *const godot_node_path)>,
        pub godot_variant_new_rid: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                *mut godot_variant,
                                                                            p_rid:
                                                                                *const godot_rid)>,
        pub godot_variant_new_object: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                    *mut godot_variant,
                                                                                p_obj:
                                                                                    *const godot_object)>,
        pub godot_variant_new_dictionary: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                        *mut godot_variant,
                                                                                    p_dict:
                                                                                        *const godot_dictionary)>,
        pub godot_variant_new_array: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                    *mut godot_variant,
                                                                                p_arr:
                                                                                    *const godot_array)>,
        pub godot_variant_new_pool_byte_array: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                            *mut godot_variant,
                                                                                        p_pba:
                                                                                            *const godot_pool_byte_array)>,
        pub godot_variant_new_pool_int_array: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                            *mut godot_variant,
                                                                                        p_pia:
                                                                                            *const godot_pool_int_array)>,
        pub godot_variant_new_pool_real_array: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                            *mut godot_variant,
                                                                                        p_pra:
                                                                                            *const godot_pool_real_array)>,
        pub godot_variant_new_pool_string_array: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                                *mut godot_variant,
                                                                                            p_psa:
                                                                                                *const godot_pool_string_array)>,
        pub godot_variant_new_pool_vector2_array: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                                *mut godot_variant,
                                                                                            p_pv2a:
                                                                                                *const godot_pool_vector2_array)>,
        pub godot_variant_new_pool_vector3_array: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                                *mut godot_variant,
                                                                                            p_pv3a:
                                                                                                *const godot_pool_vector3_array)>,
        pub godot_variant_new_pool_color_array: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                            *mut godot_variant,
                                                                                        p_pca:
                                                                                            *const godot_pool_color_array)>,
        pub godot_variant_as_bool: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_variant)
                                                            -> godot_bool>,
        pub godot_variant_as_uint: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_variant)
                                                            -> u64>,
        pub godot_variant_as_int: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_variant)
                                                            -> i64>,
        pub godot_variant_as_real: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_variant)
                                                            -> f64>,
        pub godot_variant_as_string: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_variant)
                                                            -> godot_string>,
        pub godot_variant_as_vector2: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_variant)
                                                                -> godot_vector2>,
        pub godot_variant_as_rect2: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_variant)
                                                            -> godot_rect2>,
        pub godot_variant_as_vector3: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_variant)
                                                                -> godot_vector3>,
        pub godot_variant_as_transform2d: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_variant)
                                                                    ->
                                                                        godot_transform2d>,
        pub godot_variant_as_plane: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_variant)
                                                            -> godot_plane>,
        pub godot_variant_as_quat: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_variant)
                                                            -> godot_quat>,
        pub godot_variant_as_aabb: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_variant)
                                                            -> godot_aabb>,
        pub godot_variant_as_basis: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_variant)
                                                            -> godot_basis>,
        pub godot_variant_as_transform: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_variant)
                                                                ->
                                                                    godot_transform>,
        pub godot_variant_as_color: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_variant)
                                                            -> godot_color>,
        pub godot_variant_as_node_path: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_variant)
                                                                ->
                                                                    godot_node_path>,
        pub godot_variant_as_rid: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_variant)
                                                            -> godot_rid>,
        pub godot_variant_as_object: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_variant)
                                                            ->
                                                                *mut godot_object>,
        pub godot_variant_as_dictionary: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_variant)
                                                                ->
                                                                    godot_dictionary>,
        pub godot_variant_as_array: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_variant)
                                                            -> godot_array>,
        pub godot_variant_as_pool_byte_array: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *const godot_variant)
                                                                        ->
                                                                            godot_pool_byte_array>,
        pub godot_variant_as_pool_int_array: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *const godot_variant)
                                                                    ->
                                                                        godot_pool_int_array>,
        pub godot_variant_as_pool_real_array: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *const godot_variant)
                                                                        ->
                                                                            godot_pool_real_array>,
        pub godot_variant_as_pool_string_array: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *const godot_variant)
                                                                        ->
                                                                            godot_pool_string_array>,
        pub godot_variant_as_pool_vector2_array: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                                *const godot_variant)
                                                                        ->
                                                                            godot_pool_vector2_array>,
        pub godot_variant_as_pool_vector3_array: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                                *const godot_variant)
                                                                        ->
                                                                            godot_pool_vector3_array>,
        pub godot_variant_as_pool_color_array: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *const godot_variant)
                                                                        ->
                                                                            godot_pool_color_array>,
        pub godot_variant_call: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *mut godot_variant,
                                                                        p_method:
                                                                            *const godot_string,
                                                                        p_args:
                                                                            *mut *const godot_variant,
                                                                        p_argcount:
                                                                            godot_int,
                                                                        r_error:
                                                                            *mut godot_variant_call_error)
                                                        -> godot_variant>,
        pub godot_variant_has_method: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_variant,
                                                                                p_method:
                                                                                    *const godot_string)
                                                                -> godot_bool>,
        pub godot_variant_operator_equal: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_variant,
                                                                                    p_other:
                                                                                        *const godot_variant)
                                                                    ->
                                                                        godot_bool>,
        pub godot_variant_operator_less: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_variant,
                                                                                    p_other:
                                                                                        *const godot_variant)
                                                                -> godot_bool>,
        pub godot_variant_hash_compare: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_variant,
                                                                                p_other:
                                                                                    *const godot_variant)
                                                                -> godot_bool>,
        pub godot_variant_booleanize: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_variant)
                                                                -> godot_bool>,
        pub godot_variant_destroy: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *mut godot_variant)>,
        pub godot_char_string_length: ::std::option::Option<unsafe extern "C" fn(p_cs:
                                                                                    *const godot_char_string)
                                                                -> godot_int>,
        pub godot_char_string_get_data: ::std::option::Option<unsafe extern "C" fn(p_cs:
                                                                                    *const godot_char_string)
                                                                ->
                                                                    *const libc::c_char>,
        pub godot_char_string_destroy: ::std::option::Option<unsafe extern "C" fn(p_cs:
                                                                                    *mut godot_char_string)>,
        pub godot_string_new: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                            *mut godot_string)>,
        pub godot_string_new_copy: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                *mut godot_string,
                                                                            p_src:
                                                                                *const godot_string)>,
        pub godot_string_new_with_wide_string: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                            *mut godot_string,
                                                                                        p_contents:
                                                                                            *const wchar_t,
                                                                                        p_size:
                                                                                            libc::c_int)>,
        pub godot_string_operator_index: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *mut godot_string,
                                                                                    p_idx:
                                                                                        godot_int)
                                                                ->
                                                                    *mut wchar_t>,
        pub godot_string_operator_index_const: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *const godot_string,
                                                                                        p_idx:
                                                                                            godot_int)
                                                                        ->
                                                                            wchar_t>,
        pub godot_string_wide_str: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_string)
                                                            -> *const wchar_t>,
        pub godot_string_operator_equal: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_string,
                                                                                    p_b:
                                                                                        *const godot_string)
                                                                -> godot_bool>,
        pub godot_string_operator_less: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_string,
                                                                                p_b:
                                                                                    *const godot_string)
                                                                -> godot_bool>,
        pub godot_string_operator_plus: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_string,
                                                                                p_b:
                                                                                    *const godot_string)
                                                                ->
                                                                    godot_string>,
        pub godot_string_length: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_string)
                                                        -> godot_int>,
        pub godot_string_casecmp_to: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_string,
                                                                                p_str:
                                                                                    *const godot_string)
                                                            -> libc::c_schar>,
        pub godot_string_nocasecmp_to: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_string,
                                                                                p_str:
                                                                                    *const godot_string)
                                                                ->
                                                                    libc::c_schar>,
        pub godot_string_naturalnocasecmp_to: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *const godot_string,
                                                                                        p_str:
                                                                                            *const godot_string)
                                                                        ->
                                                                            libc::c_schar>,
        pub godot_string_begins_with: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_string,
                                                                                p_string:
                                                                                    *const godot_string)
                                                                -> godot_bool>,
        pub godot_string_begins_with_char_array: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                                *const godot_string,
                                                                                            p_char_array:
                                                                                                *const libc::c_char)
                                                                        ->
                                                                            godot_bool>,
        pub godot_string_bigrams: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_string)
                                                            -> godot_array>,
        pub godot_string_chr: ::std::option::Option<unsafe extern "C" fn(p_character:
                                                                            wchar_t)
                                                        -> godot_string>,
        pub godot_string_ends_with: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_string,
                                                                            p_string:
                                                                                *const godot_string)
                                                            -> godot_bool>,
        pub godot_string_find: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_string,
                                                                        p_what:
                                                                            godot_string)
                                                        -> godot_int>,
        pub godot_string_find_from: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_string,
                                                                            p_what:
                                                                                godot_string,
                                                                            p_from:
                                                                                godot_int)
                                                            -> godot_int>,
        pub godot_string_findmk: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_string,
                                                                            p_keys:
                                                                                *const godot_array)
                                                        -> godot_int>,
        pub godot_string_findmk_from: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_string,
                                                                                p_keys:
                                                                                    *const godot_array,
                                                                                p_from:
                                                                                    godot_int)
                                                                -> godot_int>,
        pub godot_string_findmk_from_in_place: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *const godot_string,
                                                                                        p_keys:
                                                                                            *const godot_array,
                                                                                        p_from:
                                                                                            godot_int,
                                                                                        r_key:
                                                                                            *mut godot_int)
                                                                        ->
                                                                            godot_int>,
        pub godot_string_findn: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_string,
                                                                        p_what:
                                                                            godot_string)
                                                        -> godot_int>,
        pub godot_string_findn_from: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_string,
                                                                                p_what:
                                                                                    godot_string,
                                                                                p_from:
                                                                                    godot_int)
                                                            -> godot_int>,
        pub godot_string_find_last: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_string,
                                                                            p_what:
                                                                                godot_string)
                                                            -> godot_int>,
        pub godot_string_format: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_string,
                                                                            p_values:
                                                                                *const godot_variant)
                                                        -> godot_string>,
        pub godot_string_format_with_custom_placeholder: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                                        *const godot_string,
                                                                                                    p_values:
                                                                                                        *const godot_variant,
                                                                                                    p_placeholder:
                                                                                                        *const libc::c_char)
                                                                                ->
                                                                                    godot_string>,
        pub godot_string_hex_encode_buffer: ::std::option::Option<unsafe extern "C" fn(p_buffer:
                                                                                        *const u8,
                                                                                    p_len:
                                                                                        godot_int)
                                                                    ->
                                                                        godot_string>,
        pub godot_string_hex_to_int: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_string)
                                                            -> godot_int>,
        pub godot_string_hex_to_int_without_prefix: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                                *const godot_string)
                                                                            ->
                                                                                godot_int>,
        pub godot_string_insert: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_string,
                                                                            p_at_pos:
                                                                                godot_int,
                                                                            p_string:
                                                                                godot_string)
                                                        -> godot_string>,
        pub godot_string_is_numeric: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_string)
                                                            -> godot_bool>,
        pub godot_string_is_subsequence_of: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_string,
                                                                                    p_string:
                                                                                        *const godot_string)
                                                                    ->
                                                                        godot_bool>,
        pub godot_string_is_subsequence_ofi: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *const godot_string,
                                                                                        p_string:
                                                                                            *const godot_string)
                                                                    ->
                                                                        godot_bool>,
        pub godot_string_lpad: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_string,
                                                                        p_min_length:
                                                                            godot_int)
                                                        -> godot_string>,
        pub godot_string_lpad_with_custom_character: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                                    *const godot_string,
                                                                                                p_min_length:
                                                                                                    godot_int,
                                                                                                p_character:
                                                                                                    *const godot_string)
                                                                            ->
                                                                                godot_string>,
        pub godot_string_match: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_string,
                                                                        p_wildcard:
                                                                            *const godot_string)
                                                        -> godot_bool>,
        pub godot_string_matchn: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_string,
                                                                            p_wildcard:
                                                                                *const godot_string)
                                                        -> godot_bool>,
        pub godot_string_md5: ::std::option::Option<unsafe extern "C" fn(p_md5:
                                                                            *const u8)
                                                        -> godot_string>,
        pub godot_string_num: ::std::option::Option<unsafe extern "C" fn(p_num:
                                                                            f64)
                                                        -> godot_string>,
        pub godot_string_num_int64: ::std::option::Option<unsafe extern "C" fn(p_num:
                                                                                i64,
                                                                            p_base:
                                                                                godot_int)
                                                            -> godot_string>,
        pub godot_string_num_int64_capitalized: ::std::option::Option<unsafe extern "C" fn(p_num:
                                                                                            i64,
                                                                                        p_base:
                                                                                            godot_int,
                                                                                        p_capitalize_hex:
                                                                                            godot_bool)
                                                                        ->
                                                                            godot_string>,
        pub godot_string_num_real: ::std::option::Option<unsafe extern "C" fn(p_num:
                                                                                f64)
                                                            -> godot_string>,
        pub godot_string_num_scientific: ::std::option::Option<unsafe extern "C" fn(p_num:
                                                                                        f64)
                                                                ->
                                                                    godot_string>,
        pub godot_string_num_with_decimals: ::std::option::Option<unsafe extern "C" fn(p_num:
                                                                                        f64,
                                                                                    p_decimals:
                                                                                        godot_int)
                                                                    ->
                                                                        godot_string>,
        pub godot_string_pad_decimals: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_string,
                                                                                p_digits:
                                                                                    godot_int)
                                                                -> godot_string>,
        pub godot_string_pad_zeros: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_string,
                                                                            p_digits:
                                                                                godot_int)
                                                            -> godot_string>,
        pub godot_string_replace_first: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_string,
                                                                                p_key:
                                                                                    godot_string,
                                                                                p_with:
                                                                                    godot_string)
                                                                ->
                                                                    godot_string>,
        pub godot_string_replace: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_string,
                                                                            p_key:
                                                                                godot_string,
                                                                            p_with:
                                                                                godot_string)
                                                            -> godot_string>,
        pub godot_string_replacen: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_string,
                                                                            p_key:
                                                                                godot_string,
                                                                            p_with:
                                                                                godot_string)
                                                            -> godot_string>,
        pub godot_string_rfind: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_string,
                                                                        p_what:
                                                                            godot_string)
                                                        -> godot_int>,
        pub godot_string_rfindn: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_string,
                                                                            p_what:
                                                                                godot_string)
                                                        -> godot_int>,
        pub godot_string_rfind_from: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_string,
                                                                                p_what:
                                                                                    godot_string,
                                                                                p_from:
                                                                                    godot_int)
                                                            -> godot_int>,
        pub godot_string_rfindn_from: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_string,
                                                                                p_what:
                                                                                    godot_string,
                                                                                p_from:
                                                                                    godot_int)
                                                                -> godot_int>,
        pub godot_string_rpad: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_string,
                                                                        p_min_length:
                                                                            godot_int)
                                                        -> godot_string>,
        pub godot_string_rpad_with_custom_character: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                                    *const godot_string,
                                                                                                p_min_length:
                                                                                                    godot_int,
                                                                                                p_character:
                                                                                                    *const godot_string)
                                                                            ->
                                                                                godot_string>,
        pub godot_string_similarity: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_string,
                                                                                p_string:
                                                                                    *const godot_string)
                                                            -> godot_real>,
        pub godot_string_sprintf: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_string,
                                                                            p_values:
                                                                                *const godot_array,
                                                                            p_error:
                                                                                *mut godot_bool)
                                                            -> godot_string>,
        pub godot_string_substr: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_string,
                                                                            p_from:
                                                                                godot_int,
                                                                            p_chars:
                                                                                godot_int)
                                                        -> godot_string>,
        pub godot_string_to_double: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_string)
                                                            -> f64>,
        pub godot_string_to_float: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_string)
                                                            -> godot_real>,
        pub godot_string_to_int: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_string)
                                                        -> godot_int>,
        pub godot_string_camelcase_to_underscore: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                                *const godot_string)
                                                                            ->
                                                                                godot_string>,
        pub godot_string_camelcase_to_underscore_lowercased: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                                            *const godot_string)
                                                                                    ->
                                                                                        godot_string>,
        pub godot_string_capitalize: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_string)
                                                            -> godot_string>,
        pub godot_string_char_to_double: ::std::option::Option<unsafe extern "C" fn(p_what:
                                                                                        *const libc::c_char)
                                                                -> f64>,
        pub godot_string_char_to_int: ::std::option::Option<unsafe extern "C" fn(p_what:
                                                                                    *const libc::c_char)
                                                                -> godot_int>,
        pub godot_string_wchar_to_int: ::std::option::Option<unsafe extern "C" fn(p_str:
                                                                                    *const wchar_t)
                                                                -> i64>,
        pub godot_string_char_to_int_with_len: ::std::option::Option<unsafe extern "C" fn(p_what:
                                                                                            *const libc::c_char,
                                                                                        p_len:
                                                                                            godot_int)
                                                                        ->
                                                                            godot_int>,
        pub godot_string_char_to_int64_with_len: ::std::option::Option<unsafe extern "C" fn(p_str:
                                                                                                *const wchar_t,
                                                                                            p_len:
                                                                                                libc::c_int)
                                                                        ->
                                                                            i64>,
        pub godot_string_hex_to_int64: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_string)
                                                                -> i64>,
        pub godot_string_hex_to_int64_with_prefix: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                                *const godot_string)
                                                                            ->
                                                                                i64>,
        pub godot_string_to_int64: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_string)
                                                            -> i64>,
        pub godot_string_unicode_char_to_double: ::std::option::Option<unsafe extern "C" fn(p_str:
                                                                                                *const wchar_t,
                                                                                            r_end:
                                                                                                *mut *const wchar_t)
                                                                        ->
                                                                            f64>,
        pub godot_string_get_slice_count: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_string,
                                                                                    p_splitter:
                                                                                        godot_string)
                                                                    -> godot_int>,
        pub godot_string_get_slice: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_string,
                                                                            p_splitter:
                                                                                godot_string,
                                                                            p_slice:
                                                                                godot_int)
                                                            -> godot_string>,
        pub godot_string_get_slicec: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_string,
                                                                                p_splitter:
                                                                                    wchar_t,
                                                                                p_slice:
                                                                                    godot_int)
                                                            -> godot_string>,
        pub godot_string_split: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_string,
                                                                        p_splitter:
                                                                            *const godot_string)
                                                        -> godot_array>,
        pub godot_string_split_allow_empty: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_string,
                                                                                    p_splitter:
                                                                                        *const godot_string)
                                                                    ->
                                                                        godot_array>,
        pub godot_string_split_floats: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_string,
                                                                                p_splitter:
                                                                                    *const godot_string)
                                                                -> godot_array>,
        pub godot_string_split_floats_allows_empty: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                                *const godot_string,
                                                                                            p_splitter:
                                                                                                *const godot_string)
                                                                            ->
                                                                                godot_array>,
        pub godot_string_split_floats_mk: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_string,
                                                                                    p_splitters:
                                                                                        *const godot_array)
                                                                    ->
                                                                        godot_array>,
        pub godot_string_split_floats_mk_allows_empty: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                                    *const godot_string,
                                                                                                p_splitters:
                                                                                                    *const godot_array)
                                                                                ->
                                                                                    godot_array>,
        pub godot_string_split_ints: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_string,
                                                                                p_splitter:
                                                                                    *const godot_string)
                                                            -> godot_array>,
        pub godot_string_split_ints_allows_empty: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                                *const godot_string,
                                                                                            p_splitter:
                                                                                                *const godot_string)
                                                                            ->
                                                                                godot_array>,
        pub godot_string_split_ints_mk: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_string,
                                                                                p_splitters:
                                                                                    *const godot_array)
                                                                -> godot_array>,
        pub godot_string_split_ints_mk_allows_empty: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                                    *const godot_string,
                                                                                                p_splitters:
                                                                                                    *const godot_array)
                                                                            ->
                                                                                godot_array>,
        pub godot_string_split_spaces: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_string)
                                                                -> godot_array>,
        pub godot_string_char_lowercase: ::std::option::Option<unsafe extern "C" fn(p_char:
                                                                                        wchar_t)
                                                                -> wchar_t>,
        pub godot_string_char_uppercase: ::std::option::Option<unsafe extern "C" fn(p_char:
                                                                                        wchar_t)
                                                                -> wchar_t>,
        pub godot_string_to_lower: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_string)
                                                            -> godot_string>,
        pub godot_string_to_upper: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_string)
                                                            -> godot_string>,
        pub godot_string_get_basename: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_string)
                                                                -> godot_string>,
        pub godot_string_get_extension: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_string)
                                                                ->
                                                                    godot_string>,
        pub godot_string_left: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_string,
                                                                        p_pos:
                                                                            godot_int)
                                                        -> godot_string>,
        pub godot_string_ord_at: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_string,
                                                                            p_idx:
                                                                                godot_int)
                                                        -> wchar_t>,
        pub godot_string_plus_file: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_string,
                                                                            p_file:
                                                                                *const godot_string)
                                                            -> godot_string>,
        pub godot_string_right: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_string,
                                                                        p_pos:
                                                                            godot_int)
                                                        -> godot_string>,
        pub godot_string_strip_edges: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_string,
                                                                                p_left:
                                                                                    godot_bool,
                                                                                p_right:
                                                                                    godot_bool)
                                                                -> godot_string>,
        pub godot_string_strip_escapes: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_string)
                                                                ->
                                                                    godot_string>,
        pub godot_string_erase: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *mut godot_string,
                                                                        p_pos:
                                                                            godot_int,
                                                                        p_chars:
                                                                            godot_int)>,
        pub godot_string_ascii: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_string)
                                                        -> godot_char_string>,
        pub godot_string_ascii_extended: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_string)
                                                                ->
                                                                    godot_char_string>,
        pub godot_string_utf8: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_string)
                                                        -> godot_char_string>,
        pub godot_string_parse_utf8: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *mut godot_string,
                                                                                p_utf8:
                                                                                    *const libc::c_char)
                                                            -> godot_bool>,
        pub godot_string_parse_utf8_with_len: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *mut godot_string,
                                                                                        p_utf8:
                                                                                            *const libc::c_char,
                                                                                        p_len:
                                                                                            godot_int)
                                                                        ->
                                                                            godot_bool>,
        pub godot_string_chars_to_utf8: ::std::option::Option<unsafe extern "C" fn(p_utf8:
                                                                                    *const libc::c_char)
                                                                ->
                                                                    godot_string>,
        pub godot_string_chars_to_utf8_with_len: ::std::option::Option<unsafe extern "C" fn(p_utf8:
                                                                                                *const libc::c_char,
                                                                                            p_len:
                                                                                                godot_int)
                                                                        ->
                                                                            godot_string>,
        pub godot_string_hash: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_string)
                                                        -> u32>,
        pub godot_string_hash64: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_string)
                                                        -> u64>,
        pub godot_string_hash_chars: ::std::option::Option<unsafe extern "C" fn(p_cstr:
                                                                                    *const libc::c_char)
                                                            -> u32>,
        pub godot_string_hash_chars_with_len: ::std::option::Option<unsafe extern "C" fn(p_cstr:
                                                                                            *const libc::c_char,
                                                                                        p_len:
                                                                                            godot_int)
                                                                        -> u32>,
        pub godot_string_hash_utf8_chars: ::std::option::Option<unsafe extern "C" fn(p_str:
                                                                                        *const wchar_t)
                                                                    -> u32>,
        pub godot_string_hash_utf8_chars_with_len: ::std::option::Option<unsafe extern "C" fn(p_str:
                                                                                                *const wchar_t,
                                                                                            p_len:
                                                                                                godot_int)
                                                                            ->
                                                                                u32>,
        pub godot_string_md5_buffer: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_string)
                                                            ->
                                                                godot_pool_byte_array>,
        pub godot_string_md5_text: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_string)
                                                            -> godot_string>,
        pub godot_string_sha256_buffer: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_string)
                                                                ->
                                                                    godot_pool_byte_array>,
        pub godot_string_sha256_text: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_string)
                                                                -> godot_string>,
        pub godot_string_empty: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                            *const godot_string)
                                                        -> godot_bool>,
        pub godot_string_get_base_dir: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_string)
                                                                -> godot_string>,
        pub godot_string_get_file: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_string)
                                                            -> godot_string>,
        pub godot_string_humanize_size: ::std::option::Option<unsafe extern "C" fn(p_size:
                                                                                    usize)
                                                                ->
                                                                    godot_string>,
        pub godot_string_is_abs_path: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_string)
                                                                -> godot_bool>,
        pub godot_string_is_rel_path: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_string)
                                                                -> godot_bool>,
        pub godot_string_is_resource_file: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_string)
                                                                    ->
                                                                        godot_bool>,
        pub godot_string_path_to: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_string,
                                                                            p_path:
                                                                                *const godot_string)
                                                            -> godot_string>,
        pub godot_string_path_to_file: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_string,
                                                                                p_path:
                                                                                    *const godot_string)
                                                                -> godot_string>,
        pub godot_string_simplify_path: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_string)
                                                                ->
                                                                    godot_string>,
        pub godot_string_c_escape: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_string)
                                                            -> godot_string>,
        pub godot_string_c_escape_multiline: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *const godot_string)
                                                                    ->
                                                                        godot_string>,
        pub godot_string_c_unescape: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_string)
                                                            -> godot_string>,
        pub godot_string_http_escape: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_string)
                                                                -> godot_string>,
        pub godot_string_http_unescape: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_string)
                                                                ->
                                                                    godot_string>,
        pub godot_string_json_escape: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_string)
                                                                -> godot_string>,
        pub godot_string_word_wrap: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *const godot_string,
                                                                            p_chars_per_line:
                                                                                godot_int)
                                                            -> godot_string>,
        pub godot_string_xml_escape: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_string)
                                                            -> godot_string>,
        pub godot_string_xml_escape_with_quotes: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                                *const godot_string)
                                                                        ->
                                                                            godot_string>,
        pub godot_string_xml_unescape: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_string)
                                                                -> godot_string>,
        pub godot_string_percent_decode: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_string)
                                                                ->
                                                                    godot_string>,
        pub godot_string_percent_encode: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_string)
                                                                ->
                                                                    godot_string>,
        pub godot_string_is_valid_float: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_string)
                                                                -> godot_bool>,
        pub godot_string_is_valid_hex_number: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *const godot_string,
                                                                                        p_with_prefix:
                                                                                            godot_bool)
                                                                        ->
                                                                            godot_bool>,
        pub godot_string_is_valid_html_color: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *const godot_string)
                                                                        ->
                                                                            godot_bool>,
        pub godot_string_is_valid_identifier: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *const godot_string)
                                                                        ->
                                                                            godot_bool>,
        pub godot_string_is_valid_integer: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                        *const godot_string)
                                                                    ->
                                                                        godot_bool>,
        pub godot_string_is_valid_ip_address: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *const godot_string)
                                                                        ->
                                                                            godot_bool>,
        pub godot_string_destroy: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                *mut godot_string)>,
        pub godot_string_name_new: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                *mut godot_string_name,
                                                                            p_name:
                                                                                *const godot_string)>,
        pub godot_string_name_new_data: ::std::option::Option<unsafe extern "C" fn(r_dest:
                                                                                    *mut godot_string_name,
                                                                                p_name:
                                                                                    *const libc::c_char)>,
        pub godot_string_name_get_name: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_string_name)
                                                                ->
                                                                    godot_string>,
        pub godot_string_name_get_hash: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *const godot_string_name)
                                                                -> u32>,
        pub godot_string_name_get_data_unique_pointer: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                                    *const godot_string_name)
                                                                                ->
                                                                                    *const libc::c_void>,
        pub godot_string_name_operator_equal: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *const godot_string_name,
                                                                                        p_other:
                                                                                            *const godot_string_name)
                                                                        ->
                                                                            godot_bool>,
        pub godot_string_name_operator_less: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                            *const godot_string_name,
                                                                                        p_other:
                                                                                            *const godot_string_name)
                                                                    ->
                                                                        godot_bool>,
        pub godot_string_name_destroy: ::std::option::Option<unsafe extern "C" fn(p_self:
                                                                                    *mut godot_string_name)>,
        pub godot_object_destroy: ::std::option::Option<unsafe extern "C" fn(p_o:
                                                                                *mut godot_object)>,
        pub godot_global_get_singleton: ::std::option::Option<unsafe extern "C" fn(p_name:
                                                                                    *mut libc::c_char)
                                                                ->
                                                                    *mut godot_object>,
        pub godot_method_bind_get_method: ::std::option::Option<unsafe extern "C" fn(p_classname:
                                                                                        *const libc::c_char,
                                                                                    p_methodname:
                                                                                        *const libc::c_char)
                                                                    ->
                                                                        *mut godot_method_bind>,
        pub godot_method_bind_ptrcall: ::std::option::Option<unsafe extern "C" fn(p_method_bind:
                                                                                    *mut godot_method_bind,
                                                                                p_instance:
                                                                                    *mut godot_object,
                                                                                p_args:
                                                                                    *mut *const libc::c_void,
                                                                                p_ret:
                                                                                    *mut libc::c_void)>,
        pub godot_method_bind_call: ::std::option::Option<unsafe extern "C" fn(p_method_bind:
                                                                                *mut godot_method_bind,
                                                                            p_instance:
                                                                                *mut godot_object,
                                                                            p_args:
                                                                                *mut *const godot_variant,
                                                                            p_arg_count:
                                                                                libc::c_int,
                                                                            p_call_error:
                                                                                *mut godot_variant_call_error)
                                                            -> godot_variant>,
        pub godot_get_class_constructor: ::std::option::Option<unsafe extern "C" fn(p_classname:
                                                                                        *const libc::c_char)
                                                                ->
                                                                    godot_class_constructor>,
        pub godot_get_global_constants: ::std::option::Option<unsafe extern "C" fn()
                                                                ->
                                                                    godot_dictionary>,
        pub godot_register_native_call_type: ::std::option::Option<unsafe extern "C" fn(call_type:
                                                                                            *const libc::c_char,
                                                                                        p_callback:
                                                                                            native_call_cb)>,
        pub godot_alloc: ::std::option::Option<unsafe extern "C" fn(p_bytes:
                                                                        libc::c_int)
                                                -> *mut libc::c_void>,
        pub godot_realloc: ::std::option::Option<unsafe extern "C" fn(p_ptr:
                                                                        *mut libc::c_void,
                                                                    p_bytes:
                                                                        libc::c_int)
                                                    -> *mut libc::c_void>,
        pub godot_free: ::std::option::Option<unsafe extern "C" fn(p_ptr:
                                                                    *mut libc::c_void)>,
        pub godot_print_error: ::std::option::Option<unsafe extern "C" fn(p_description:
                                                                            *const libc::c_char,
                                                                        p_function:
                                                                            *const libc::c_char,
                                                                        p_file:
                                                                            *const libc::c_char,
                                                                        p_line:
                                                                            libc::c_int)>,
        pub godot_print_warning: ::std::option::Option<unsafe extern "C" fn(p_description:
                                                                                *const libc::c_char,
                                                                            p_function:
                                                                                *const libc::c_char,
                                                                            p_file:
                                                                                *const libc::c_char,
                                                                            p_line:
                                                                                libc::c_int)>,
        pub godot_print: ::std::option::Option<unsafe extern "C" fn(p_message:
                                                                        *const godot_string)>,
    }
    native_script(GDNATIVE_API_TYPES_GDNATIVE_EXT_NATIVESCRIPT, godot_gdnative_ext_nativescript_api_struct) {
        pub godot_nativescript_register_class: ::std::option::Option<unsafe extern "C" fn(p_gdnative_handle:
                                                                                            *mut libc::c_void,
                                                                                        p_name:
                                                                                            *const libc::c_char,
                                                                                        p_base:
                                                                                            *const libc::c_char,
                                                                                        p_create_func:
                                                                                            godot_instance_create_func,
                                                                                        p_destroy_func:
                                                                                            godot_instance_destroy_func)>,
        pub godot_nativescript_register_tool_class: ::std::option::Option<unsafe extern "C" fn(p_gdnative_handle:
                                                                                                *mut libc::c_void,
                                                                                            p_name:
                                                                                                *const libc::c_char,
                                                                                            p_base:
                                                                                                *const libc::c_char,
                                                                                            p_create_func:
                                                                                                godot_instance_create_func,
                                                                                            p_destroy_func:
                                                                                                godot_instance_destroy_func)>,
        pub godot_nativescript_register_method: ::std::option::Option<unsafe extern "C" fn(p_gdnative_handle:
                                                                                            *mut libc::c_void,
                                                                                        p_name:
                                                                                            *const libc::c_char,
                                                                                        p_function_name:
                                                                                            *const libc::c_char,
                                                                                        p_attr:
                                                                                            godot_method_attributes,
                                                                                        p_method:
                                                                                            godot_instance_method)>,
        pub godot_nativescript_register_property: ::std::option::Option<unsafe extern "C" fn(p_gdnative_handle:
                                                                                                *mut libc::c_void,
                                                                                            p_name:
                                                                                                *const libc::c_char,
                                                                                            p_path:
                                                                                                *const libc::c_char,
                                                                                            p_attr:
                                                                                                *mut godot_property_attributes,
                                                                                            p_set_func:
                                                                                                godot_property_set_func,
                                                                                            p_get_func:
                                                                                                godot_property_get_func)>,
        pub godot_nativescript_register_signal: ::std::option::Option<unsafe extern "C" fn(p_gdnative_handle:
                                                                                            *mut libc::c_void,
                                                                                        p_name:
                                                                                            *const libc::c_char,
                                                                                        p_signal:
                                                                                            *const godot_signal)>,
        pub godot_nativescript_get_userdata: ::std::option::Option<unsafe extern "C" fn(p_instance:
                                                                                            *mut godot_object)
                                                                    ->
                                                                        *mut libc::c_void>,
    }
}}