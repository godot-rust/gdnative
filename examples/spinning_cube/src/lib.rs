#[macro_use]
extern crate gdnative as godot;

#[derive(godot::NativeClass)]
#[inherit(godot::MeshInstance)]
#[user_data(godot::user_data::MutexData<RustTest>)]
#[register_with(my_register_function)]
struct RustTest {
    start: godot::Vector3,
    time: f32,
    #[property(default = 0.05)]
    rotate_speed: f64,
}

fn my_register_function(builder: &godot::init::ClassBuilder<RustTest>) {
    builder.add_property(godot::init::Property {
        name: "test/test_enum",
        default: godot::GodotString::from_str("Hello"),
        hint: godot::init::PropertyHint::Enum {
            values: &["Hello", "World", "Testing"],
        },
        getter: |_: &RustTest| godot::GodotString::from_str("Hello"),
        setter: (),
        usage: godot::init::PropertyUsage::DEFAULT,
    });
    builder.add_property(godot::init::Property {
        name: "test/test_flags",
        default: 0,
        hint: godot::init::PropertyHint::Flags {
            values: &["A", "B", "C", "D"],
        },
        getter: |_: &RustTest| 0,
        setter: (),
        usage: godot::init::PropertyUsage::DEFAULT,
    });
}

#[godot::methods]
impl RustTest {
    fn _init(_owner: godot::MeshInstance) -> Self {
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
        use godot::{Color, SpatialMaterial, Vector3};

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
