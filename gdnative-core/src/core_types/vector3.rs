use crate::Vector3;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Axis(pub i64);

impl Axis {
    pub const X: Axis = Axis(sys::godot_vector3_axis_GODOT_VECTOR3_AXIS_X as i64);
    pub const Y: Axis = Axis(sys::godot_vector3_axis_GODOT_VECTOR3_AXIS_Y as i64);
    pub const Z: Axis = Axis(sys::godot_vector3_axis_GODOT_VECTOR3_AXIS_Z as i64);
}

impl std::ops::Deref for Axis {
    type Target = i64;
    #[inline]
    fn deref(&self) -> &i64 {
        &self.0
    }
}

impl From<i64> for Axis {
    #[inline]
    fn from(v: i64) -> Self {
        Self(v)
    }
}

impl From<Axis> for i64 {
    #[inline]
    fn from(v: Axis) -> Self {
        v.0
    }
}

/// Helper methods for `Vector3`.
///
/// Trait used to provide additional methods that are equivalent to Godot's methods.
/// See the official [`Godot documentation`](https://docs.godotengine.org/en/3.1/classes/class_vector3.html).
pub trait Vector3Godot {
    /// Internal API for converting to `sys` representation. Makes it possible to remove
    /// `transmute`s elsewhere.
    #[doc(hidden)]
    fn to_sys(self) -> sys::godot_vector3;
    /// Internal API for converting to `sys` representation. Makes it possible to remove
    /// `transmute`s elsewhere.
    #[doc(hidden)]
    fn sys(&self) -> *const sys::godot_vector3;
    /// Internal API for converting from `sys` representation. Makes it possible to remove
    /// `transmute`s elsewhere.
    #[doc(hidden)]
    fn from_sys(v: sys::godot_vector3) -> Self;
}

impl Vector3Godot for Vector3 {
    #[inline]
    fn to_sys(self) -> sys::godot_vector3 {
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    fn sys(&self) -> *const sys::godot_vector3 {
        self as *const _ as *const _
    }

    #[inline]
    fn from_sys(v: sys::godot_vector3) -> Self {
        unsafe { std::mem::transmute(v) }
    }
}

godot_test!(
    test_vector3_variants {
        use crate::{FromVariant, ToVariant, Vector3};

        fn test(vector: Vector3, set_to: Vector3) {
            let api = crate::private::get_api();

            let copied = vector;
            unsafe {
                assert_relative_eq!(vector.x, (api.godot_vector3_get_axis)(
                    &copied as *const _ as *const sys::godot_vector3,
                    i64::from(Axis::X) as u32
                ));
                assert_relative_eq!(vector.y, (api.godot_vector3_get_axis)(
                    &copied as *const _ as *const sys::godot_vector3,
                    i64::from(Axis::Y) as u32
                ));
                assert_relative_eq!(vector.z, (api.godot_vector3_get_axis)(
                    &copied as *const _ as *const sys::godot_vector3,
                    i64::from(Axis::Z) as u32
                ));
            }
            assert_eq!(vector, copied);

            let mut copied = vector;
            unsafe {
                (api.godot_vector3_set_axis)(
                    &mut copied as *mut _ as *mut sys::godot_vector3,
                    i64::from(Axis::X) as u32,
                    set_to.x
                );
                (api.godot_vector3_set_axis)(
                    &mut copied as *mut _ as *mut sys::godot_vector3,
                    i64::from(Axis::Y) as u32,
                    set_to.y
                );
                (api.godot_vector3_set_axis)(
                    &mut copied as *mut _ as *mut sys::godot_vector3,
                    i64::from(Axis::Z) as u32,
                    set_to.z
                );
            }
            assert_eq!(set_to, copied);

            let variant = vector.to_variant();
            let vector_from_variant = Vector3::from_variant(&variant).unwrap();
            assert_eq!(vector, vector_from_variant);
        }

        test(Vector3::new(1.0, 2.0, 3.0), Vector3::new(4.0, 5.0, 6.0));
        test(Vector3::new(4.0, 5.0, 6.0), Vector3::new(7.0, 8.0, 9.0));
    }
);

#[cfg(test)]
mod tests {
    use crate::Vector3;

    #[test]
    fn it_is_copy() {
        fn copy<T: Copy>() {}
        copy::<Vector3>();
    }

    #[test]
    fn it_has_the_same_size() {
        use std::mem::size_of;
        assert_eq!(size_of::<sys::godot_vector3>(), size_of::<Vector3>());
    }

    #[test]
    fn it_supports_equality() {
        assert_eq!(Vector3::new(1.0, 2.0, 3.0), Vector3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn it_supports_inequality() {
        assert_ne!(Vector3::new(1.0, 10.0, 100.0), Vector3::new(1.0, 2.0, 3.0));
    }
}
