use gdnative::api::Resource;
use gdnative::prelude::*;

#[derive(NativeClass)]
#[inherit(Resource)]
struct GreetingResource {
    #[property]
    name: String,
}

#[gdnative::methods]
impl GreetingResource {
    fn new(_owner: &Resource) -> Self {
        Self { name: "".into() }
    }

    fn say_hello(&self, _owner: &Reference) {
        godot_print!("Hello {}!", self.name);
    }
}

#[derive(NativeClass)]
#[inherit(Node)]
struct Greeter {
    #[property]
    greeting_resource: Option<Instance<GreetingResource, Shared>>,
}

#[gdnative::methods]
impl Greeter {
    fn new(_owner: &Node) -> Self {
        Greeter {
            greeting_resource: None,
        }
    }

    #[export]
    fn _ready(&self, _owner: &Node) {
        if let Some(greeting_resource) = self.greeting_resource.as_ref() {
            let greeting_resource = unsafe { greeting_resource.assume_safe() };
            greeting_resource.map(|s, o| s.say_hello(&*o)).unwrap();
        }
    }
}

fn init(handle: InitHandle) {
    handle.add_class::<Greeter>();
    handle.add_class::<GreetingResource>();
}

godot_init!(init);
