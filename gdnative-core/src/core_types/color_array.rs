use crate::core_types::Color;
use crate::core_types::PoolArray;

/// A reference-counted vector of `Color` that uses Godot's pool allocator.
///
/// See [`PoolColorArray`](https://docs.godotengine.org/en/stable/classes/class_poolcolorarray.html) in Godot.
#[deprecated = "Specialized pool array aliases will be removed in a future godot-rust version. Use PoolArray<T> instead."]
pub type ColorArray = PoolArray<Color>;

godot_test!(
    test_color_array_access {
        use crate::object::NewRef as _;

        let arr = PoolArray::from_vec(vec![
            Color::from_rgb(1.0, 0.0, 0.0),
            Color::from_rgb(0.0, 1.0, 0.0),
            Color::from_rgb(0.0, 0.0, 1.0),
        ]);

        let original_read = {
            let read = arr.read();
            assert_eq!(&[
                Color::from_rgb(1.0, 0.0, 0.0),
                Color::from_rgb(0.0, 1.0, 0.0),
                Color::from_rgb(0.0, 0.0, 1.0),
            ], read.as_slice());
            read
        };

        let mut cow_arr = arr.new_ref();

        {
            let mut write = cow_arr.write();
            assert_eq!(3, write.len());
            for i in write.as_mut_slice() {
                i.b = 1.0;
            }
        }

        assert_eq!(Color::from_rgb(1.0, 0.0, 1.0), cow_arr.get(0));
        assert_eq!(Color::from_rgb(0.0, 1.0, 1.0), cow_arr.get(1));
        assert_eq!(Color::from_rgb(0.0, 0.0, 1.0), cow_arr.get(2));

        // the write shouldn't have affected the original array
        assert_eq!(&[
            Color::from_rgb(1.0, 0.0, 0.0),
            Color::from_rgb(0.0, 1.0, 0.0),
            Color::from_rgb(0.0, 0.0, 1.0),
        ], original_read.as_slice());
    }
);

godot_test!(
    test_color_array_debug {
        let arr = PoolArray::from_vec(vec![
            Color::from_rgb(1.0, 0.0, 0.0),
            Color::from_rgb(0.0, 1.0, 0.0),
            Color::from_rgb(0.0, 0.0, 1.0),
        ]);

        assert_eq!(format!("{arr:?}"), "[Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 }, Color { r: 0.0, g: 1.0, b: 0.0, a: 1.0 }, Color { r: 0.0, g: 0.0, b: 1.0, a: 1.0 }]");
    }
);
