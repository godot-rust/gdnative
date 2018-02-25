#[macro_use]
extern crate gdnative as godot;


godot_class! {
    class RustTest: godot::MeshInstance {
        fields {
            start: godot::Vector3,
            time: f32,
            rotate_speed: f64,
        }
        setup(builder) {
            builder.add_property(
                godot::Property {
                    name: "base/rotate_speed",
                    default: 0.05,
                    hint: godot::PropertyHint::Range {
                        range: 0.05..1.0,
                        step: 0.01,
                        slider: true
                    },
                    getter: |this: &mut RustTest| this.rotate_speed,
                    setter: |this: &mut RustTest, v| this.rotate_speed = v,
                    usage: godot::PropertyUsage::DEFAULT,
                }
            );

            builder.add_property(
                godot::Property {
                    name: "test/test_enum",
                    default: godot::GodotString::from_str("Hello"),
                    hint: godot::PropertyHint::Enum {
                        values: &[
                            "Hello",
                            "World",
                            "Testing",
                        ]
                    },
                    getter: |_: &mut RustTest| { godot::GodotString::from_str("Hello") },
                    setter: (),
                    usage: godot::PropertyUsage::DEFAULT,
                }
            );

            builder.add_property(
                godot::Property {
                    name: "test/test_flags",
                    default: 0,
                    hint: godot::PropertyHint::Flags {
                        values: &["A", "B", "C", "D" ],
                    },
                    getter: |_: &mut RustTest| 0,
                    setter: (),
                    usage: godot::PropertyUsage::DEFAULT,
                }
            );
        }
        constructor(godot_info) {
            RustTest {
                godot_info: godot_info,
                start: godot::Vector3::new(0.0, 0.0, 0.0),
                time: 0.0,
                rotate_speed: 0.05,
            }
        }

        export fn _ready(&mut self) {
            let p = self.godot_parent();
            p.set_physics_process(true);
            self.start = p.get_translation();
            godot_warn!("Start: {:?}", self.start);
            godot_warn!("Parent name: {:?}", p.get_parent()
                .expect("Missing parent")
                .get_name());
        }

        export fn _physics_process(&mut self, delta: f64) {
            use godot::{Color, SpatialMaterial, Vector3};
            self.time += delta as f32;
            let p = self.godot_parent();
            p.rotate_y(self.rotate_speed);
            let offset = Vector3::new(0.0, 1.0, 0.0) * self.time.cos() * 0.5;
            p.set_translation(self.start + offset);

            if let Some(mat) = p.get_surface_material(0) {
                let mat = mat.cast::<SpatialMaterial>().expect("Incorrect material");
                mat.set_albedo(Color::rgba(self.time.cos().abs(), 0.0, 0.0, 1.0));
            }
        }
    }
}

godot_init! {
    RustTest
}
