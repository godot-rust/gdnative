use crate::core_types::PoolArray;

/// A reference-counted vector of `u8` that uses Godot's pool allocator.
///
/// See [`PoolByteArray`](https://docs.godotengine.org/en/stable/classes/class_poolbytearray.html) in Godot.
#[deprecated = "Specialized pool array aliases will be removed in a future godot-rust version. Use PoolArray<T> instead."]
pub type ByteArray = PoolArray<u8>;

godot_test!(
    test_byte_array_access {
        use crate::object::NewRef as _;

        let arr = (0..8).collect::<PoolArray<u8>>();

        let original_read = {
            let read = arr.read();
            assert_eq!(&[0, 1, 2, 3, 4, 5, 6, 7], read.as_slice());
            read
        };

        let mut cow_arr = arr.new_ref();

        {
            let mut write = cow_arr.write();
            assert_eq!(8, write.len());
            for i in write.as_mut_slice() {
                *i *= 2;
            }
        }

        cow_arr.append_slice(&[0, 1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(16, cow_arr.len());

        for i in 0..8 {
            assert_eq!(i * 2, cow_arr.get(i as i32));
        }

        for i in 8..16 {
            assert_eq!(i - 8, cow_arr.get(i as i32));
        }

        // the write shouldn't have affected the original array
        assert_eq!(&[0, 1, 2, 3, 4, 5, 6, 7], original_read.as_slice());

        // check to_vec()
        assert_eq!(arr.to_vec(), vec![0, 1, 2, 3, 4, 5, 6, 7]);
    }
);

godot_test!(
    test_byte_array_debug {
        let arr = (0..8).collect::<PoolArray<u8>>();
        assert_eq!(format!("{arr:?}"), "[0, 1, 2, 3, 4, 5, 6, 7]");
    }
);
