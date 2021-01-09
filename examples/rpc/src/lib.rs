use gdnative::prelude::*;

mod client;
mod server;

fn init(handle: InitHandle) {
    handle.add_class::<client::ServerPuppet>();
    handle.add_class::<server::Server>();
}

godot_init!(init);
