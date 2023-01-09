use crate::core_types::GodotString;
use crate::core_types::PoolArray;

/// A reference-counted vector of `GodotString` that uses Godot's pool allocator.
///
/// See [`PoolStringArray`](https://docs.godotengine.org/en/stable/classes/class_poolstringarray.html) in Godot.
#[deprecated = "Specialized pool array aliases will be removed in a future godot-rust version. Use PoolArray<T> instead."]
pub type StringArray = PoolArray<GodotString>;

godot_test!(
    test_string_array_access {
        use crate::object::NewRef as _;

        let arr = PoolArray::from_vec(vec![
            GodotString::from("foo"),
            GodotString::from("bar"),
            GodotString::from("baz"),
        ]);

        let original_read = {
            let read = arr.read();
            assert_eq!(&[
                GodotString::from("foo"),
                GodotString::from("bar"),
                GodotString::from("baz"),
            ], read.as_slice());
            read
        };

        let mut cow_arr = arr.new_ref();

        {
            let mut write = cow_arr.write();
            assert_eq!(3, write.len());
            for s in write.as_mut_slice() {
                *s = s.to_uppercase();
            }
        }

        assert_eq!(GodotString::from("FOO"), cow_arr.get(0));
        assert_eq!(GodotString::from("BAR"), cow_arr.get(1));
        assert_eq!(GodotString::from("BAZ"), cow_arr.get(2));

        // the write shouldn't have affected the original array
        assert_eq!(&[
            GodotString::from("foo"),
            GodotString::from("bar"),
            GodotString::from("baz"),
        ], original_read.as_slice());
    }
);

godot_test!(
    test_string_array_debug {
        let arr = PoolArray::from_vec(vec![
            GodotString::from("foo"),
            GodotString::from("bar"),
            GodotString::from("baz"),
        ]);

        assert_eq!(format!("{arr:?}"), "[\"foo\", \"bar\", \"baz\"]");
    }
);
