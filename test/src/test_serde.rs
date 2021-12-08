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
    status &= test_yaml();
    status &= test_cbor();
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
    // dict: Dictionary, //TODO: PartialEq
    // v_arr: VariantArray, //TODO: PartialEq
    byte_arr: ByteArray,
    int_arr: Int32Array,
    float_arr: Float32Array,
    str_arr: StringArray,
    vec2_arr: Vector2Array,
    vec3_arr: Vector3Array,
    color_arr: ColorArray,
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
                x: Vector2::RIGHT,
                y: Vector2::DOWN,
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
            byte_arr: ByteArray::from_slice(&[30u8, 31u8, 32u8]),
            int_arr: Int32Array::from_slice(&[33i32, 34i32, 35i32, 36i32]),
            float_arr: Float32Array::from_slice(&[37.38, 39.40]),
            str_arr: StringArray::from_vec(vec!["hello".into(), "world".into()]),
            vec2_arr: Vector2Array::from_slice(&[
                Vector2::UP,
                Vector2::UP,
                Vector2::DOWN,
                Vector2::DOWN,
                Vector2::LEFT,
                Vector2::RIGHT,
                Vector2::LEFT,
                Vector2::RIGHT,
            ]),
            vec3_arr: Vector3Array::from_slice(&[
                Vector3::ONE * 41.0,
                Vector3::BACK * 42.43,
                Vector3::FORWARD * 44.45,
            ]),
            color_arr: ColorArray::from_slice(&[Color::from_rgba(0.0, 1.0, 0.627, 0.8)]),
        }
    }
}

/// Sanity check that a round trip through Variant preserves equality for Foo.
fn test_variant_eq() -> bool {
    println!("   -- test_variant_eq");

    let ok = std::panic::catch_unwind(|| {
        let foo = Foo::new();
        let variant = foo.to_variant();
        let result = Foo::from_variant(&variant).expect("Foo::from_variant");
        assert_eq!(foo, result);
    })
    .is_ok();

    if !ok {
        godot_error!("     !! Test test_variant_eq failed");
    }

    ok
}

/// Sanity check that a round trip through VariantDispatch preserves equality for Foo.
fn test_dispatch_eq() -> bool {
    println!("   -- test_dispatch_eq");

    let ok = std::panic::catch_unwind(|| {
        let foo = Foo::new();
        let dispatch = foo.to_variant().dispatch();
        let result = Foo::from_variant(&Variant::from(&dispatch)).expect("Foo from Dispatch");
        assert_eq!(foo, result);
    })
    .is_ok();

    if !ok {
        godot_error!("     !! Test test_dispatch_eq failed");
    }

    ok
}

fn test_ron() -> bool {
    println!("   -- test_ron");

    let ok = std::panic::catch_unwind(|| {
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
    })
    .is_ok();

    if !ok {
        godot_error!("     !! Test test_ron failed");
    }

    ok
}

fn test_json() -> bool {
    println!("   -- test_json");

    let ok = std::panic::catch_unwind(|| {
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
    })
    .is_ok();

    if !ok {
        godot_error!("     !! Test test_json failed");
    }

    ok
}

fn test_yaml() -> bool {
    println!("   -- test_yaml");

    let ok = std::panic::catch_unwind(|| {
        let foo = Foo::new();

        let yaml_str = serde_yaml::to_string(&foo).expect("Foo to YAML");
        let result = serde_yaml::from_str::<Foo>(&yaml_str).expect("Foo from YAML");
        assert_eq!(foo, result);

        let yaml_str =
            serde_yaml::to_string(&foo.to_variant().dispatch()).expect("Dispatch to YAML");
        let disp = serde_yaml::from_str::<VariantDispatch>(&yaml_str).expect("Dispatch from YAML");
        let result = Foo::from_variant(&Variant::from(&disp)).expect("Foo from Dispatch from YAML");
        assert_eq!(foo, result);
    })
    .is_ok();

    if !ok {
        godot_error!("     !! Test test_yaml failed");
    }

    ok
}

fn test_cbor() -> bool {
    println!("   -- test_cbor");

    let ok = std::panic::catch_unwind(|| {
        let foo = Foo::new();

        let cbor_bytes = serde_cbor::to_vec(&foo).expect("Foo to CBOR");
        let result = serde_cbor::from_slice::<Foo>(&cbor_bytes).expect("Foo from CBOR");
        assert_eq!(foo, result);

        let cbor_bytes =
            serde_cbor::to_vec(&foo.to_variant().dispatch()).expect("Dispatch to CBOR");
        let disp =
            serde_cbor::from_slice::<VariantDispatch>(&cbor_bytes).expect("Dispatch from CBOR");
        let result = Foo::from_variant(&Variant::from(&disp)).expect("Foo from Dispatch from CBOR");
        assert_eq!(foo, result);
    })
    .is_ok();

    if !ok {
        godot_error!("     !! Test test_cbor failed");
    }

    ok
}

fn test_msgpack() -> bool {
    println!("   -- test_msgpack");

    let ok = std::panic::catch_unwind(|| {
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
    })
    .is_ok();

    if !ok {
        godot_error!("     !! Test test_msgpack failed");
    }

    ok
}

fn test_bincode() -> bool {
    println!("   -- test_bincode");

    let ok = std::panic::catch_unwind(|| {
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
    })
    .is_ok();

    if !ok {
        godot_error!("     !! Test test_bincode failed");
    }

    ok
}
