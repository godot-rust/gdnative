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
            builder.property("base/rotate_speed", 0.05)
                .hint(godot::PropertyHint::Range {
                    min: 0.05,
                    max: 1.0,
                    step: 0.01,
                    slider: true
                })
                .getter(|s: &mut RustTest| s.rotate_speed)
                .setter(|s: &mut RustTest, v| s.rotate_speed = v)
                .register();
            builder.property("test/test_enum", "Hello".to_owned())
                .hint(godot::PropertyHint::Enum {
                    values: vec![
                        "Hello".to_owned(),
                        "World".to_owned(),
                        "Testing".to_owned()
                    ]
                })
                .getter(|s: &mut RustTest| "Hello".to_owned())
                .register();
            builder.property("test/test_flags", 0)
                .hint(godot::PropertyHint::Flags {
                    values: vec![
                        "A".to_owned(),
                        "B".to_owned(),
                        "C".to_owned(),
                        "D".to_owned()
                    ]
                })
                .getter(|s: &mut RustTest| 0)
                .register();
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
            use godot::{NodePath, Color, Spatial, SpatialMaterial, Vector3};
            self.time += delta as f32;
            let p = self.godot_parent();
            p.rotate_y(self.rotate_speed);
            let offset = Vector3::new(0.0, 1.0, 0.0) * self.time.cos() * 0.5;
            p.set_translation(self.start + offset);

            if let Some(cap) = p.get_node(NodePath::new("./Cap")).and_then(|v| v.cast::<Spatial>()) {
                cap.rotate_x(0.05);
            }

            if let Some(mat) = p.get_surface_material(0) {
                let mat = mat.cast::<SpatialMaterial>().expect("Incorrect material");
                mat.set_albedo(Color::new_rgba(self.time.cos().abs(), 0.0, 0.0, 1.0));
            } else {
                godot_warn!("No material");
            }
        }
    }
}

godot_init! {
    RustTest
}