use gdnative::api::NetworkedMultiplayerENet;
use gdnative::prelude::*;

const PORT: i64 = 9876;
const MAX_CLIENTS: i64 = 1;
const IN_BANDWIDTH: i64 = 1000;
const OUT_BANDWIDTH: i64 = 1000;

#[derive(NativeClass)]
#[inherit(Node)]
pub struct Server {
    #[property(rpc = "master", set = "Self::set_foo")]
    foo: i32,
}

#[methods]
impl Server {
    fn new(_owner: &Node) -> Self {
        Self { foo: 0 }
    }

    #[method]
    fn _ready(&mut self, #[base] owner: &Node) {
        let peer = NetworkedMultiplayerENet::new();
        peer.create_server(PORT, MAX_CLIENTS, IN_BANDWIDTH, OUT_BANDWIDTH)
            .unwrap();

        let tree = owner.get_tree().expect("could not retreive Scene Tree");
        let tree = unsafe { tree.assume_safe() };

        tree.set_network_peer(peer);
    }

    #[method(rpc = "master")]
    fn greet_server(&mut self, #[base] owner: &Node, msg: GodotString) {
        godot_print!("Client says: {}", msg);

        let tree = owner.get_tree().expect("could not retreive Scene Tree");
        let tree = unsafe { tree.assume_safe() };

        owner.rpc_id(
            tree.get_rpc_sender_id(),
            "return_greeting",
            &[Variant::new("hello")],
        );
    }

    #[method]
    fn set_foo(&mut self, #[base] _owner: TRef<Node>, value: i32) {
        godot_print!("Client sets foo to: {}", value);
        self.foo = value;
    }
}
