use gdnative::api::Resource;
use gdnative::prelude::*;

#[derive(NativeClass)]
#[inherit(Resource)]
struct GreetingResource {
    #[property]
    name: String,
}

#[methods]
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
    // It's possible to export any type that implements `Export`, `ToVariant` and `FromVariant` using `#[property]`
    // All these traits are implemented for `Instance<T, Shared>` where the base class of `T` is reference-counted.
    // `Resource` inherits from `Reference`, so all native scripts extending `Resource` have reference-counted base classes.
    #[property]
    greeting_resource: Option<Instance<GreetingResource>>,
}

#[methods]
impl Greeter {
    fn new(_owner: &Node) -> Self {
        Greeter {
            greeting_resource: None,
        }
    }

    #[method]
    fn _ready(&self) {
        if let Some(greeting_resource) = self.greeting_resource.as_ref() {
            let greeting_resource = unsafe { greeting_resource.assume_safe() };
            greeting_resource.map(|s, o| s.say_hello(&o)).unwrap();
        }
    }
}

struct ResourceLibrary;

#[gdnative::init::callbacks]
impl GDNativeCallbacks for ResourceLibrary {
    fn nativescript_init(handle: InitHandle) {
        handle.add_class::<Greeter>();
        handle.add_class::<GreetingResource>();
    }
}
