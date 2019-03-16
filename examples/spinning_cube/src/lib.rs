#[macro_use]
extern crate gdnative as godot;

use godot::init::{Property, PropertyHint, PropertyUsage};
use godot::GodotString;

struct RustTest {
    start: godot::Vector3,
    time: f32,
    rotate_speed: f64,
}

impl godot::NativeClass for RustTest {
    type Base = godot::MeshInstance;

    fn class_name() -> &'static str {
        "RustTest"
    }

    fn init(_owner: Self::Base) -> Self {
        Self::_init()
    }

    fn register_properties(builder: &godot::init::ClassBuilder<Self>) {
        builder.add_property(
            Property {
                name: "base/rotate_speed",
                default: 0.05,
                hint: PropertyHint::Range {
                    range: 0.05..1.0,
                    step: 0.01,
                    slider: true
                },
                getter: |this: &mut RustTest| this.rotate_speed,
                setter: |this: &mut RustTest, v| this.rotate_speed = v,
                usage: PropertyUsage::DEFAULT,
            }
        );

        builder.add_property(
            Property {
                name: "test/test_enum",
                default: GodotString::from_str("Hello"),
                hint: PropertyHint::Enum {
                    values: &[
                        "Hello",
                        "World",
                        "Testing",
                    ]
                },
                getter: |_: &mut RustTest| { GodotString::from_str("Hello") },
                setter: (),
                usage: PropertyUsage::DEFAULT,
            }
        );

        builder.add_property(
            Property {
                name: "test/test_flags",
                default: 0,
                hint: PropertyHint::Flags {
                    values: &["A", "B", "C", "D" ],
                },
                getter: |_: &mut RustTest| 0,
                setter: (),
                usage: PropertyUsage::DEFAULT,
            }
        );
    }
}

#[godot::methods]
impl RustTest {

    fn _init() -> Self {
        RustTest {
            start: godot::Vector3::new(0.0, 0.0, 0.0),
            time: 0.0,
            rotate_speed: 0.05,
        }
    }

    #[export]
    unsafe fn _ready(&mut self, mut owner: godot::MeshInstance) {
        owner.set_physics_process(true);
        self.start = owner.get_translation();
        godot_warn!("Start: {:?}", self.start);
        godot_warn!(
            "Parent name: {:?}",
            owner.get_parent().expect("Missing parent").get_name()
        );
    }

    #[export]
    unsafe fn _physics_process(&mut self, mut owner: godot::MeshInstance, delta: f64) {
        use godot::{Color, Vector3, SpatialMaterial};

        self.time += delta as f32;
        owner.rotate_y(self.rotate_speed * delta);

        let offset = Vector3::new(0.0, 1.0, 0.0) * self.time.cos() * 0.5;
        owner.set_translation(self.start + offset);

        if let Some(mat) = owner.get_surface_material(0) {
            let mut mat = mat.cast::<SpatialMaterial>().expect("Incorrect material");
            mat.set_albedo(Color::rgba(self.time.cos().abs(), 0.0, 0.0, 1.0));
        }
    }
}

fn init(handle: godot::init::InitHandle) {
    handle.add_class::<RustTest>();
}

godot_gdnative_init!();
godot_nativescript_init!(init);
godot_gdnative_terminate!();
