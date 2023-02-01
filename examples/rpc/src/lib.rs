use gdnative::prelude::*;

mod client;
mod server;

struct RpcLibrary;

#[gdnative::init::callbacks]
impl GDNativeCallbacks for RpcLibrary {
    fn nativescript_init(handle: InitHandle) {
        handle.add_class::<client::ServerPuppet>();
        handle.add_class::<server::Server>();
    }
}
