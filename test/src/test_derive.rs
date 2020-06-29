// use gdnative::*;
use gdnative::prelude::*;

pub(crate) fn run_tests() -> bool {
    let mut status = true;

    status &= test_derive_to_variant();
    status &= test_derive_owned_to_variant();

    status
}

pub(crate) fn register(_handle: InitHandle) {}

fn test_derive_to_variant() -> bool {
    println!(" -- test_derive_to_variant");

    #[derive(Clone, Eq, PartialEq, Debug, ToVariant, FromVariant)]
    struct ToVar<T, R>
    where
        T: Associated,
        R: Default,
    {
        foo: T::A,
        bar: T,
        baz: ToVarEnum<T::B>,
        #[variant(with = "variant_with")]
        ptr: *mut (),
        #[variant(skip)]
        skipped: R,
    }

    #[derive(Clone, Eq, PartialEq, Debug, ToVariant, FromVariant)]
    enum ToVarEnum<T> {
        Foo(T),
        Bar,
        Baz { baz: u8 },
    }

    #[derive(Clone, Eq, PartialEq, Debug, ToVariant, FromVariant)]
    struct ToVarTuple<T, R>(T::A, #[variant(skip)] R, T::B)
    where
        T: Associated,
        R: Default;

    trait Associated {
        type A;
        type B;
    }

    impl Associated for f64 {
        type A = i64;
        type B = bool;
    }

    mod variant_with {
        use gdnative::core_types::{FromVariantError, GodotString, ToVariant, Variant};

        #[allow(clippy::trivially_copy_pass_by_ref)]
        pub fn to_variant(_ptr: &*mut ()) -> Variant {
            GodotString::from("*mut ()").to_variant()
        }

        pub fn from_variant(_variant: &Variant) -> Result<*mut (), FromVariantError> {
            Ok(std::ptr::null_mut())
        }
    }

    let ok = std::panic::catch_unwind(|| {
        let data = ToVar::<f64, i128> {
            foo: 42,
            bar: 54.0,
            baz: ToVarEnum::Foo(true),
            ptr: std::ptr::null_mut(),
            skipped: 42,
        };

        let variant = data.to_variant();
        let dictionary = variant.try_to_dictionary().expect("should be dictionary");
        assert_eq!(Some(42), dictionary.get("foo").try_to_i64());
        assert_eq!(Some(54.0), dictionary.get("bar").try_to_f64());
        assert_eq!(
            Some("*mut ()".into()),
            dictionary.get("ptr").try_to_string()
        );
        assert!(!dictionary.contains("skipped"));

        let enum_dict = dictionary
            .get("baz")
            .try_to_dictionary()
            .expect("should be dictionary");
        assert_eq!(Some(true), enum_dict.get("Foo").try_to_bool());

        assert_eq!(
            Ok(ToVar::<f64, i128> {
                foo: 42,
                bar: 54.0,
                baz: ToVarEnum::Foo(true),
                ptr: std::ptr::null_mut(),
                skipped: 0,
            }),
            ToVar::from_variant(&variant)
        );

        let data = ToVarTuple::<f64, i128>(1, 2, false);
        let variant = data.to_variant();
        let tuple_array = variant.try_to_array().expect("should be array");

        assert_eq!(2, tuple_array.len());
        assert_eq!(Some(1), tuple_array.get(0).try_to_i64());
        assert_eq!(Some(false), tuple_array.get(1).try_to_bool());
        assert_eq!(
            Ok(ToVarTuple::<f64, i128>(1, 0, false)),
            ToVarTuple::from_variant(&variant)
        );
    })
    .is_ok();

    if !ok {
        gdnative::godot_error!("   !! Test test_derive_to_variant failed");
    }

    ok
}

fn test_derive_owned_to_variant() -> bool {
    println!(" -- test_derive_owned_to_variant");

    #[derive(OwnedToVariant)]
    struct ToVar {
        arr: VariantArray<Unique>,
    }

    let ok = std::panic::catch_unwind(|| {
        let data = ToVar {
            arr: [1, 2, 3].iter().collect(),
        };

        let variant = data.owned_to_variant();
        let dictionary = variant.try_to_dictionary().expect("should be dictionary");
        let array = dictionary
            .get("arr")
            .try_to_array()
            .expect("should be array");
        assert_eq!(3, array.len());
        assert_eq!(
            &[1, 2, 3],
            array
                .iter()
                .map(|v| v.try_to_i64().unwrap())
                .collect::<Vec<_>>()
                .as_slice()
        );
    })
    .is_ok();

    if !ok {
        gdnative::godot_error!("   !! Test test_derive_owned_to_variant failed");
    }

    ok
}
