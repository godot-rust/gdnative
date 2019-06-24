use crate::ToVariant;
use crate::Variant;
use crate::Vector2;

impl ToVariant for Vector2 {
    fn to_variant(&self) -> Variant {
        Variant::from_vector2(self)
    }

    fn from_variant(variant: &Variant) -> Option<Self> {
        variant.try_to_vector2()
    }
}

godot_test!(
    test_vector2_variants {
        let vector = Vector2::new(1.0, 2.0);
        let variant = vector.to_variant();
        let vector_from_variant = Vector2::from_variant(&variant).unwrap();

        assert_eq!(vector, vector_from_variant);
    }
    );

#[cfg(test)]
mod tests {
    use super::Vector2;

    #[test]
    fn it_supports_equality() {
        assert_eq!(Vector2::new(1.0, 2.0), Vector2::new(1.0, 2.0));
    }

    #[test]
    fn it_supports_inequality() {
        assert_ne!(Vector2::new(1.0, 10.0), Vector2::new(1.0, 2.0));
    }
}
