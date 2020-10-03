use gdnative::prelude::*;

mod path {
    use super::*;

    pub struct To {}
    impl To {
        pub fn to_variant(_value: &String) -> Variant {
            unimplemented!()
        }

        pub fn from_variant(_value: &Variant) -> Result<String, FromVariantError> {
            unimplemented!()
        }
    }
}

#[derive(ToVariant)]
pub struct Foo {
    #[variant(with = "path::To")]
    bar: String,

    #[variant(
        to_variant_with = "path::To::to_variant",
        from_variant_with = "path::To::from_variant"
    )]
    baz: String,
}

fn main() {}
