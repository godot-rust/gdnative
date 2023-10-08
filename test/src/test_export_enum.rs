use gdnative::prelude::*;

#[derive(Debug, PartialEq, Clone, Copy, Export, ToVariant, FromVariant)]
#[variant(enum = "repr")]
#[repr(i32)]
enum Dir {
    Up = 1,
    Down = -1,
}

pub(crate) fn run_tests() -> bool {
    let mut ok = true;

    ok &= test_from_variant();
    ok &= test_to_variant();

    ok
}

crate::godot_itest!(test_from_variant {
    assert_eq!(Dir::from_variant(&1_i32.to_variant()), Ok(Dir::Up));
    assert_eq!(Dir::from_variant(&(-1_i32).to_variant()), Ok(Dir::Down));
    // 42 isn't mapped to any variant of `Dir`
    assert!(Dir::from_variant(&42_i32.to_variant()).is_err());
});

crate::godot_itest!(test_to_variant {
    assert_eq!(Dir::Up.to_variant(), 1_i32.to_variant());
    assert_eq!(Dir::Down.to_variant(), (-1_i32).to_variant());
});
