use crate::core_types::PoolArray;

/// A reference-counted vector of `f32` that uses Godot's pool allocator.
pub type Float32Array = PoolArray<f32>;

godot_test!(
    test_float32_array_access {
        use crate::object::NewRef as _;

        let arr = (0..8).map(|i| i as f32).collect::<Float32Array>();

        let original_read = {
            let read = arr.read();
            for (n, i) in read.as_slice().iter().enumerate() {
                assert_relative_eq!(n as f32, i);
            }
            read.clone()
        };

        let mut cow_arr = arr.new_ref();

        {
            let mut write = cow_arr.write();
            assert_eq!(8, write.len());
            for i in write.as_mut_slice() {
                *i *= 2.0;
            }
        }

        for i in 0..8 {
            assert_relative_eq!(i as f32 * 2., cow_arr.get(i as i32));
        }

        // the write shouldn't have affected the original array
        for (n, i) in original_read.as_slice().iter().enumerate() {
            assert_relative_eq!(n as f32, i);
        }
    }
);

godot_test!(
    test_float32_array_debug {
        let arr = (0..8).map(|i| i as f32).collect::<Float32Array>();
        assert_eq!(format!("{:?}", arr), "[0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0]");
    }
);
