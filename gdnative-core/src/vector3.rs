use crate::ToVariant;
use crate::Variant;
use crate::Vector3;

impl ToVariant for Vector3 {
    fn to_variant(&self) -> Variant {
        Variant::from_vector3(self)
    }

    fn from_variant(variant: &Variant) -> Option<Self> {
        variant.try_to_vector3()
    }
}

godot_test!(
    test_vector3_variants {
        let vector = Vector3::new(1.0, 2.0, 3.0);
        let variant = vector.to_variant();
        let vector_from_variant = Vector3::from_variant(&variant).unwrap();

        assert_eq!(vector, vector_from_variant);
    }
    );

#[cfg(test)]
mod tests {
    use super::Vector3;

    #[test]
    fn it_supports_equality() {
        assert_eq!(Vector3::new(1.0, 2.0, 3.0), Vector3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn it_supports_inequality() {
        assert_ne!(Vector3::new(1.0, 10.0, 100.0), Vector3::new(1.0, 2.0, 3.0));
    }
}
