use crate::core_types::PoolArray;
use crate::core_types::Vector2;

/// A reference-counted vector of `Vector2` that uses Godot's pool allocator.
///
/// See [`PoolVector2Array`](https://docs.godotengine.org/en/stable/classes/class_poolvector2array.html) in Godot.
#[deprecated = "Specialized pool array aliases will be removed in a future godot-rust version. Use PoolArray<T> instead."]
pub type Vector2Array = PoolArray<Vector2>;

godot_test!(
    test_vector2_array_access {
        use crate::object::NewRef as _;

        let arr = PoolArray::from_vec(vec![
            Vector2::new(1.0, 2.0),
            Vector2::new(3.0, 4.0),
            Vector2::new(5.0, 6.0),
        ]);

        let original_read = {
            let read = arr.read();
            assert_eq!(&[
                Vector2::new(1.0, 2.0),
                Vector2::new(3.0, 4.0),
                Vector2::new(5.0, 6.0),
            ], read.as_slice());
            read
        };

        let mut cow_arr = arr.new_ref();

        {
            let mut write = cow_arr.write();
            assert_eq!(3, write.len());
            for s in write.as_mut_slice() {
                s.x += 1.0;
            }
        }

        assert_eq!(Vector2::new(2.0, 2.0), cow_arr.get(0));
        assert_eq!(Vector2::new(4.0, 4.0), cow_arr.get(1));
        assert_eq!(Vector2::new(6.0, 6.0), cow_arr.get(2));

        // the write shouldn't have affected the original array
        assert_eq!(&[
            Vector2::new(1.0, 2.0),
            Vector2::new(3.0, 4.0),
            Vector2::new(5.0, 6.0),
        ], original_read.as_slice());
    }
);

godot_test!(
    test_vector2_array_debug {
        let arr = PoolArray::from_vec(vec![
            Vector2::new(1.0, 2.0),
            Vector2::new(3.0, 4.0),
            Vector2::new(5.0, 6.0),
        ]);

        assert_eq!(format!("{arr:?}"), format!("{:?}", &[Vector2::new(1.0, 2.0), Vector2::new(3.0, 4.0), Vector2::new(5.0, 6.0)]));
    }
);
