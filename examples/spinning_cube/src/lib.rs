#[macro_use]
extern crate gdnative as godot;

use godot::init::{Property, PropertyHint, PropertyUsage};
use godot::GodotString;

godot_class! {
    class RustTest: godot::MeshInstance {
	is_tool: false;
        fields {
            start: godot::Vector3,
            time: f32,
            rotate_speed: f64,
        }
        setup(builder) {
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
        constructor(header) {
            RustTest {
                header,
                start: godot::Vector3::new(0.0, 0.0, 0.0),
                time: 0.0,
                rotate_speed: 0.05,
            }
        }

        export fn _ready(&mut self) {
            unsafe {
                let mut owner = self.get_owner();
                owner.set_physics_process(true);
                self.start = owner.get_translation();
                godot_warn!("Start: {:?}", self.start);
                godot_warn!(
                    "Parent name: {:?}",
                    owner.get_parent().expect("Missing parent").get_name()
                );
            }
        }

        export fn _physics_process(&mut self, delta: f64) {
            use godot::{Color, Vector3, SpatialMaterial};
            unsafe {
                self.time += delta as f32;
                let mut owner = self.get_owner();
                owner.rotate_y(self.rotate_speed);
                let offset = Vector3::new(0.0, 1.0, 0.0) * self.time.cos() * 0.5;
                owner.set_translation(self.start + offset);

                if let Some(mat) = owner.get_surface_material(0) {
                    let mut mat = mat.cast::<SpatialMaterial>().expect("Incorrect material");
                    mat.set_albedo(Color::rgba(self.time.cos().abs(), 0.0, 0.0, 1.0));
                }
            }
        }
    }
}

fn init(handle: godot::init::InitHandle) {
    RustTest::register_class(handle);
}

godot_gdnative_init!();
godot_nativescript_init!(init);
godot_gdnative_terminate!();
