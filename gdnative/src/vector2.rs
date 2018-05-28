use GodotType;
use Variant;
use Vector2;

impl GodotType for Vector2 {
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

    test_vector2_equality {
        assert_eq!(
            Vector2::new(1.0, 2.0),
            Vector2::new(1.0, 2.0)
            );
    }

    test_vector2_inequality {
        assert_ne!(
            Vector2::new(1.0, 10.0),
            Vector2::new(1.0, 2.0)
            );

    }
    );
