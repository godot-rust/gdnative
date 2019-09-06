use crate::FromVariant;
use crate::ToVariant;
use crate::Variant;
use crate::Vector2;

impl ToVariant for Vector2 {
    fn to_variant(&self) -> Variant {
        Variant::from_vector2(self)
    }
}

impl FromVariant for Vector2 {
    fn from_variant(variant: &Variant) -> Option<Self> {
        variant.try_to_vector2()
    }
}

godot_test!(
    test_vector2_variants {
        fn test(vector: Vector2, set_to: Vector2) {
            let api = crate::get_api();

            let copied = vector;
            unsafe {
                assert_eq!(vector.x, (api.godot_vector2_get_x)(&copied as *const _ as *const sys::godot_vector2));
                assert_eq!(vector.y, (api.godot_vector2_get_y)(&copied as *const _ as *const sys::godot_vector2));
            }
            assert_eq!(vector, copied);

            let mut copied = vector;
            unsafe {
                (api.godot_vector2_set_x)(&mut copied as *mut _ as *mut sys::godot_vector2, set_to.x);
                (api.godot_vector2_set_y)(&mut copied as *mut _ as *mut sys::godot_vector2, set_to.y);
            }
            assert_eq!(set_to, copied);

            let variant = vector.to_variant();
            let vector_from_variant = Vector2::from_variant(&variant).unwrap();
            assert_eq!(vector, vector_from_variant);
        }

        test(Vector2::new(1.0, 2.0), Vector2::new(3.0, 4.0));
        test(Vector2::new(3.0, 4.0), Vector2::new(5.0, 6.0));
    }
    );

#[cfg(test)]
mod tests {
    use super::Vector2;

    #[test]
    fn it_is_copy() {
        fn copy<T: Copy>() {}
        copy::<Vector2>();
    }

    #[test]
    fn it_has_the_same_size() {
        use std::mem::size_of;
        assert_eq!(size_of::<sys::godot_vector2>(), size_of::<Vector2>());
    }

    #[test]
    fn it_supports_equality() {
        assert_eq!(Vector2::new(1.0, 2.0), Vector2::new(1.0, 2.0));
    }

    #[test]
    fn it_supports_inequality() {
        assert_ne!(Vector2::new(1.0, 10.0), Vector2::new(1.0, 2.0));
    }
}
