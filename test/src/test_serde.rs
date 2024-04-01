use gdnative::core_types::variant::VariantDispatch;
use gdnative::prelude::*;
use serde::{Deserialize, Serialize};

pub(crate) fn run_tests() -> bool {
    println!(" -- serde tests:");
    let mut status = true;

    // All tests depend on these invariants
    status &= test_variant_eq();
    status &= test_dispatch_eq();
    if !status {
        godot_error!("   !!!! Can't run serde tests, Foo::[to/from]_variant is broken!");
        return false;
    }

    status &= test_ron();
    status &= test_json();
    status &= test_msgpack();
    status &= test_bincode();

    status
}

#[derive(Debug, PartialEq, Serialize, Deserialize, ToVariant, FromVariant)]
struct Foo {
    some: Option<bool>,
    none: Option<bool>,
    b: bool,
    int: i64,
    float: f64,
    str: GodotString,
    vec2: Vector2,
    rect2: Rect2,
    vec3: Vector3,
    xform_2d: Transform2D,
    plane: Plane,
    quat: Quat,
    aabb: Aabb,
    basis: Basis,
    xform: Transform,
    color: Color,
    path: NodePath,
    // dict: Dictionary, //TODO(#990): PartialEq
    // v_arr: VariantArray, //TODO(#990): PartialEq
    byte_arr: PoolArray<u8>,
    int_arr: PoolArray<i32>,
    float_arr: PoolArray<f32>,
    str_arr: PoolArray<GodotString>,
    vec2_arr: PoolArray<Vector2>,
    vec3_arr: PoolArray<Vector3>,
    color_arr: PoolArray<Color>,
}

impl Foo {
    fn new() -> Self {
        Self {
            some: Some(true),
            none: None,
            b: false,
            int: 1,
            float: 2.0,
            str: "this is a GodotString".into(),
            vec2: Vector2::RIGHT,
            rect2: Rect2 {
                position: Vector2 { x: 46.47, y: -2.0 },
                size: Vector2 { x: 3.0, y: 4.8 },
            },
            vec3: Vector3::BACK,
            xform_2d: Transform2D {
                a: Vector2::RIGHT,
                b: Vector2::DOWN,
                origin: Vector2::ZERO,
            },
            plane: Plane {
                normal: Vector3::ONE.normalized(),
                d: 3.0,
            },
            quat: Quat::new(4.1, 5.2, 6.3, 7.5),
            aabb: Aabb {
                position: Vector3::new(8.2, 9.8, 10.11),
                size: Vector3::new(12.13, 14.15, 16.17),
            },
            basis: Basis::IDENTITY.rotated(Vector3::UP, std::f32::consts::TAU / 3.0),
            xform: Transform {
                basis: Basis::from_euler(Vector3::new(18.19, -20.21, 22.23)),
                origin: Vector3::new(24.25, 26.27, 28.29),
            },
            color: Color::from_rgb(0.549, 0.0, 1.0),
            path: "/root/Node".into(),
            byte_arr: PoolArray::from_slice(&[30u8, 31u8, 32u8]),
            int_arr: PoolArray::from_slice(&[33i32, 34i32, 35i32, 36i32]),
            float_arr: PoolArray::from_slice(&[37.38, 39.40]),
            str_arr: PoolArray::from_vec(vec!["hello".into(), "world".into()]),
            vec2_arr: PoolArray::from_slice(&[
                Vector2::UP,
                Vector2::UP,
                Vector2::DOWN,
                Vector2::DOWN,
                Vector2::LEFT,
                Vector2::RIGHT,
                Vector2::LEFT,
                Vector2::RIGHT,
            ]),
            vec3_arr: PoolArray::from_slice(&[
                Vector3::ONE * 41.0,
                Vector3::BACK * 42.43,
                Vector3::FORWARD * 44.45,
            ]),
            color_arr: PoolArray::from_slice(&[Color::from_rgba(0.0, 1.0, 0.627, 0.8)]),
        }
    }
}

// Sanity check that a round trip through Variant preserves equality for Foo.
crate::godot_itest! { test_variant_eq {
    let foo = Foo::new();
    let variant = foo.to_variant();
    let result = Foo::from_variant(&variant).expect("Foo::from_variant");
    assert_eq!(foo, result);
}}

// Sanity check that a round trip through VariantDispatch preserves equality for Foo.
crate::godot_itest! { test_dispatch_eq {
    let foo = Foo::new();
    let dispatch = foo.to_variant().dispatch();
    let result = Foo::from_variant(&Variant::from(&dispatch)).expect("Foo from Dispatch");
    assert_eq!(foo, result);
}}

crate::godot_itest! { test_ron {
    let foo = Foo::new();

    let ron_str = ron::to_string(&foo).expect("Foo to RON str");
    let mut de = ron::Deserializer::from_str(ron_str.as_ref());
    let result = Foo::deserialize(de.as_mut().expect("deserialize Foo from RON")).unwrap();
    assert_eq!(foo, result);

    let ron_disp_str = ron::to_string(&foo.to_variant().dispatch()).expect("Dispatch to RON");
    let mut de = ron::Deserializer::from_str(ron_disp_str.as_ref());
    let de = de
        .as_mut()
        .expect("disp_round_trip ron::Deserializer::from_str");
    let disp = VariantDispatch::deserialize(de).expect("Dispatch from RON");
    let result = Foo::from_variant(&Variant::from(&disp)).expect("Foo from Dispatch from RON");
    assert_eq!(foo, result);
}}

crate::godot_itest! { test_json {
    let foo = Foo::new();

    let json_str = serde_json::to_string(&foo).expect("Foo to JSON");
    let result = serde_json::from_str::<Foo>(json_str.as_ref()).expect("Foo from JSON");
    assert_eq!(foo, result);

    let foo = Foo::new();
    let json_disp_str =
        serde_json::to_string(&foo.to_variant().dispatch()).expect("Foo Dispatch to JSON");
    let disp = serde_json::from_str::<VariantDispatch>(json_disp_str.as_ref())
        .expect("Dispatch from JSON");

    let result = Foo::from_variant(&Variant::from(&disp)).expect("Foo from Dispatch from JSON");
    assert_eq!(foo, result);
}}

crate::godot_itest! { test_msgpack {
    let foo = Foo::new();

    let msgpack_bytes = rmp_serde::to_vec_named(&foo).expect("Foo to MessagePack");
    let result =
        rmp_serde::from_read_ref::<_, Foo>(&msgpack_bytes).expect("Foo from MessagePack");
    assert_eq!(foo, result);

    let msgpack_disp_bytes =
        rmp_serde::to_vec_named(&foo.to_variant().dispatch()).expect("Dispatch to MessagePack");
    let disp = rmp_serde::from_read_ref::<_, VariantDispatch>(&msgpack_disp_bytes)
        .expect("Dispatch from MessagePack");
    let result =
        Foo::from_variant(&Variant::from(&disp)).expect("Foo from Dispatch from MessagePack");
    assert_eq!(foo, result);
}}

crate::godot_itest! { test_bincode {
    let foo = Foo::new();

    let bincode_bytes = bincode::serialize(&foo).expect("Foo to bincode");
    let result = bincode::deserialize::<Foo>(bincode_bytes.as_ref()).expect("Foo from bincode");
    assert_eq!(foo, result);

    let bincode_bytes =
        bincode::serialize(&foo.to_variant().dispatch()).expect("Dispatch to bincode");
    let disp = bincode::deserialize::<VariantDispatch>(bincode_bytes.as_ref())
        .expect("Dispatch from bincode");
    let result =
        Foo::from_variant(&Variant::from(&disp)).expect("Foo from Dispatch from bincode");
    assert_eq!(foo, result);
}}
