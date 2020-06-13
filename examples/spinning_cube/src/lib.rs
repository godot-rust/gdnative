#[macro_use]
extern crate gdnative;

use gdnative::api::MeshInstance;
use gdnative::init::property::{EnumHint, IntHint, StringHint};

#[derive(gdnative::NativeClass)]
#[inherit(MeshInstance)]
#[register_with(my_register_function)]
struct RustTest {
    start: gdnative::Vector3,
    time: f32,
    #[property(path = "base/rotate_speed")]
    rotate_speed: f64,
}

fn my_register_function(builder: &gdnative::init::ClassBuilder<RustTest>) {
    builder
        .add_property::<String>("test/test_enum")
        .with_hint(StringHint::Enum(EnumHint::new(vec![
            "Hello".into(),
            "World".into(),
            "Testing".into(),
        ])))
        .with_getter(|_: &RustTest, _| "Hello".to_string())
        .done();

    builder
        .add_property("test/test_flags")
        .with_hint(IntHint::Flags(EnumHint::new(vec![
            "A".into(),
            "B".into(),
            "C".into(),
            "D".into(),
        ])))
        .with_getter(|_: &RustTest, _| 0)
        .done();
}

#[gdnative::methods]
impl RustTest {
    fn _init(_owner: MeshInstance) -> Self {
        RustTest {
            start: gdnative::Vector3::new(0.0, 0.0, 0.0),
            time: 0.0,
            rotate_speed: 0.05,
        }
    }

    #[export]
    unsafe fn _ready(&mut self, owner: MeshInstance) {
        owner.set_physics_process(true);
        self.start = owner.translation();
        godot_warn!("Start: {:?}", self.start);
        godot_warn!(
            "Parent name: {:?}",
            owner.get_parent().expect("Missing parent").name()
        );
    }

    #[export]
    unsafe fn _physics_process(&mut self, owner: MeshInstance, delta: f64) {
        use gdnative::{api::SpatialMaterial, Color, Vector3};

        self.time += delta as f32;
        owner.rotate_y(self.rotate_speed * delta);

        let offset = Vector3::new(0.0, 1.0, 0.0) * self.time.cos() * 0.5;
        owner.set_translation(self.start + offset);

        if let Some(mat) = owner.get_surface_material(0) {
            let mat = mat.cast::<SpatialMaterial>().expect("Incorrect material");
            mat.set_albedo(Color::rgba(self.time.cos().abs(), 0.0, 0.0, 1.0));
        }
    }
}

fn init(handle: gdnative::init::InitHandle) {
    handle.add_class::<RustTest>();
}

godot_gdnative_init!();
godot_nativescript_init!(init);
godot_gdnative_terminate!();
