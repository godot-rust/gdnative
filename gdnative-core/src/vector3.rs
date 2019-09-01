use crate::ToVariant;
use crate::FromVariant;
use crate::Variant;
use crate::Vector3;

impl ToVariant for Vector3 {
    fn to_variant(&self) -> Variant {
        Variant::from_vector3(self)
    }
}

impl FromVariant for Vector3 {
    fn from_variant(variant: &Variant) -> Option<Self> {
        variant.try_to_vector3()
    }
}

godot_test!(
    test_vector3_variants {
        fn test(vector: Vector3, set_to: Vector3) {
            let api = crate::get_api();

            let copied = vector;
            unsafe {
                assert_eq!(vector.x, (api.godot_vector3_get_axis)(&copied as *const _ as *const sys::godot_vector3, crate::Vector3Axis::X as u32));
                assert_eq!(vector.y, (api.godot_vector3_get_axis)(&copied as *const _ as *const sys::godot_vector3, crate::Vector3Axis::Y as u32));
                assert_eq!(vector.z, (api.godot_vector3_get_axis)(&copied as *const _ as *const sys::godot_vector3, crate::Vector3Axis::Z as u32));
            }
            assert_eq!(vector, copied);

            let mut copied = vector;
            unsafe {
                (api.godot_vector3_set_axis)(&mut copied as *mut _ as *mut sys::godot_vector3, crate::Vector3Axis::X as u32, set_to.x);
                (api.godot_vector3_set_axis)(&mut copied as *mut _ as *mut sys::godot_vector3, crate::Vector3Axis::Y as u32, set_to.y);
                (api.godot_vector3_set_axis)(&mut copied as *mut _ as *mut sys::godot_vector3, crate::Vector3Axis::Z as u32, set_to.z);
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
    use super::Vector3;

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
