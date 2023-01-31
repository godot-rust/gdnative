use approx::relative_eq;

use crate::core_types::{Color, GodotString, Vector2, Vector3};
use crate::object::NewRef as _;

use super::{PoolArray, PoolElement};

fn test_array_access<T: PoolElement + Clone>(
    elements: impl IntoIterator<Item = T>,
    mutate: impl Fn(&mut T),
    eq: impl Fn(&T, &T) -> bool,
) {
    let src = elements.into_iter().collect::<Vec<_>>();
    let arr = src.iter().cloned().collect::<PoolArray<_>>();

    let original_read = {
        let read = arr.read();
        assert_eq!(src.len(), read.len());
        assert!(read.iter().zip(&src).all(|(a, b)| eq(a, b)));
        read
    };

    let mut cow_arr = arr.new_ref();

    {
        let mut write = cow_arr.write();
        assert_eq!(src.len(), write.len());
        for i in write.as_mut_slice() {
            mutate(i);
        }
    }

    for (n, mut i) in src.iter().cloned().enumerate() {
        mutate(&mut i);
        assert!(eq(&i, &cow_arr.get(n as i32)));
    }

    // the write shouldn't have affected the original array
    assert_eq!(src.len(), original_read.len());
    assert!(original_read.iter().zip(&src).all(|(a, b)| eq(a, b)));
}

godot_test!(
    test_byte_array_access {
        test_array_access(0u8..8, |i| *i *= 2, |a, b| a == b);
    }

    test_color_array_access {
        test_array_access(
            [
                Color::from_rgb(1.0, 0.0, 0.0),
                Color::from_rgb(0.0, 1.0, 0.0),
                Color::from_rgb(0.0, 0.0, 1.0),
            ],
            |i| i.b = 1.0,
            |a, b| a == b,
        );
    }

    test_float32_array_access {
        test_array_access((0..8).map(|i| i as f32), |i| *i *= 2.0, |a, b| relative_eq!(a, b));
    }

    test_int32_array_access {
        test_array_access(0i32..8, |i| *i *= 2, |a, b| a == b);
    }

    test_string_array_access {
        test_array_access(
            [
                GodotString::from("foo"),
                GodotString::from("bar"),
                GodotString::from("baz"),
            ],
            |i| *i = i.to_uppercase(),
            |a, b| a == b,
        );
    }

    test_vector2_array_access {
        test_array_access(
            [
                Vector2::new(1.0, 2.0),
                Vector2::new(3.0, 4.0),
                Vector2::new(5.0, 6.0),
            ],
            |i| i.x += 1.0,
            |a, b| a == b,
        );
    }

    test_vector3_array_access {
        test_array_access(
            [
                Vector3::new(1.0, 2.0, 3.0),
                Vector3::new(3.0, 4.0, 5.0),
                Vector3::new(5.0, 6.0, 7.0),
            ],
            |i| {
                i.x += 2.0;
                i.y += 1.0;
            },
            |a, b| a == b,
        );
    }
);
