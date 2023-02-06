use gdnative::api::MeshInstance;
use gdnative::prelude::*;

use gdnative::export::hint::{EnumHint, IntHint, StringHint};

#[derive(gdnative::derive::NativeClass)]
#[inherit(MeshInstance)]
#[register_with(register_members)]
struct RustTest {
    start: Vector3,
    time: f32,
    #[property(path = "base/rotate_speed")]
    rotate_speed: f64,
}

fn register_members(builder: &ClassBuilder<RustTest>) {
    builder
        .property::<String>("test/test_enum")
        .with_hint(StringHint::Enum(EnumHint::new(vec![
            "Hello".into(),
            "World".into(),
            "Testing".into(),
        ])))
        .with_getter(|_: &RustTest, _| "Hello".to_string())
        .done();

    builder
        .property("test/test_flags")
        .with_hint(IntHint::Flags(EnumHint::new(vec![
            "A".into(),
            "B".into(),
            "C".into(),
            "D".into(),
        ])))
        .with_getter(|_: &RustTest, _| 0)
        .done();
}

#[methods]
impl RustTest {
    fn new(_owner: &MeshInstance) -> Self {
        RustTest {
            start: Vector3::new(0.0, 0.0, 0.0),
            time: 0.0,
            rotate_speed: 0.05,
        }
    }

    #[method]
    fn _ready(&mut self, #[base] owner: &MeshInstance) {
        owner.set_physics_process(true);
    }

    #[method]
    fn _physics_process(&mut self, #[base] owner: &MeshInstance, delta: f64) {
        use gdnative::api::SpatialMaterial;

        self.time += delta as f32;
        owner.rotate_y(self.rotate_speed * delta);

        let offset = Vector3::new(0.0, 1.0, 0.0) * self.time.cos() * 0.5;
        owner.set_translation(self.start + offset);

        if let Some(mat) = owner.get_surface_material(0) {
            let mat = unsafe { mat.assume_safe() };
            let mat = mat.cast::<SpatialMaterial>().expect("Incorrect material");
            mat.set_albedo(Color::from_rgba(self.time.cos().abs(), 0.0, 0.0, 1.0));
        }
    }
}

struct CubeLibrary;

#[gdnative::init::callbacks]
impl GDNativeCallbacks for CubeLibrary {
    fn nativescript_init(handle: InitHandle) {
        handle.add_class::<RustTest>();
    }
}
