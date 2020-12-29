# RPC Demo

A very simple example to show peer to peer communication with Godot's high level multiplayer API calls.


## Setup

1. Build the library with `cargo build`.

2. Import the Godot project in the Godot editor.

3. Open the `Server.tscn` scene and start the current scene with `F6`.

4. Start a second instance of the Godot editor.

5. Set a separate "Remote Port" for the second editor (e.g. 6012). The two editor instances need to have different debug ports so that they can connect to their own running scenes. Needed to show the godot print messages.

    `Editor > Editor Settings > Network > Debug`

5. Open the `Client.tscn` scene and start the current scene with `F6`


## References

* [Godot hih-level multiplayer API](https://docs.godotengine.org/en/stable/tutorials/networking/high_level_multiplayer.html)
