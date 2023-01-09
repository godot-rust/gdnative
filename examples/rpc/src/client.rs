use gdnative::api::NetworkedMultiplayerENet;
use gdnative::prelude::*;

const ADDRESS: &str = "127.0.0.1";
const PORT: i64 = 9876;
const IN_BANDWIDTH: i64 = 1000;
const OUT_BANDWIDTH: i64 = 1000;
const CLIENT_PORT: i64 = 9877;

#[derive(NativeClass)]
#[inherit(Node)]
pub struct ServerPuppet;

#[methods]
impl ServerPuppet {
    fn new(_owner: &Node) -> Self {
        Self
    }

    #[method]
    fn _ready(&mut self, #[base] owner: TRef<Node>) {
        let peer = NetworkedMultiplayerENet::new();
        peer.create_client(
            GodotString::from(ADDRESS),
            PORT,
            IN_BANDWIDTH,
            OUT_BANDWIDTH,
            CLIENT_PORT,
        )
        .unwrap();

        let tree = owner.get_tree().expect("could not retreive SceneTree");
        let tree = unsafe { tree.assume_safe() };

        tree.set_network_peer(peer);

        tree.connect(
            "connected_to_server",
            owner,
            "on_connected_to_server",
            VariantArray::new_shared(),
            0,
        )
        .unwrap();
    }

    #[method]
    fn on_connected_to_server(&mut self, #[base] owner: TRef<Node>) {
        owner.rpc("greet_server", &[Variant::new("hello")]);
        owner.rset("foo", 42);
    }

    #[method(rpc = "puppet")]
    fn return_greeting(&mut self, msg: GodotString) {
        godot_print!("Server says: {}", msg);
    }
}
