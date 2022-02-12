# Contributing to the godot-rust library

The godot-rust bindings developers welcome contribution from everyone. Here are the guidelines if you are thinking of helping us:


## Submitting a PR

Contributions should be made in the form of GitHub pull requests (PRs). Each pull request will be reviewed by the godot-rust team or other contributors.


### Picking a task

Have a look at the [issue tracker] to find good tasks to start with. Should you wish to work on an issue, please claim it first by commenting in it. This is to prevent duplicated efforts from contributors on the same issue.

We especially appreciate input on the issues labeled [`help wanted`]. The label [`good first issue`] encompasses issues which do not require a deep prior knowledge of the godot-rust intricacies and could be picked up by a newcomer.

You can also open a pull request directly without an associated issue. Note however that discussing the issue with other developers beforehand increases the chances of a PR being accepted, and may save time on both your and the reviewer's end. Such discussions can also happen in our Discord server.


### Git workflow

* Branch from the `master` branch and, if needed, rebase to the current `master` branch before submitting your pull request. If it doesn't merge cleanly, you may be asked to rebase your changes.

* Use one commit per logical change. Often, PRs only consist of a single commit. If you change unrelated parts of the code, split it into separate commits.

* If your patch is not getting reviewed or you need a specific person to review it, you can @-reply a reviewer asking for a review in the pull request or a comment.


### Writing tests

Whenever applicable, add tests relevant to the fixed bug or new feature.

Some types and functions can only be used if the engine is running. In order to test them, use the `godot_test!` macro, and explicitly invoke the test functions in [test/src/lib.rs](test/src/lib.rs). Don't hesitate to see how existing code does this.

  
### Local and CI checks

The repository root contains a script `check.sh`. If you invoke it without arguments, it will run a list of basic checks locally. This tool also allows you to quickly build RustDoc documentation of changed APIs. Please run `./check.sh --help` for more information about its usage.

If you use Windows, you might consider using a Unix shell interpreter (e.g. one shipped with Git-for-Windows, or WSL2, or similar).

Additionally, when opening a PR, our continuous integration (CI) pipeline will automatically perform a few checks (formatting, lints, unit tests). Please make sure all of them pass, adjusting your code where necessary.


## Communication

Primary communication between developers of the project happens on the `godot-rust` Discord server. [Invite link here][godot-rust-discord].

For questions about Godot, check out the options on the [Godot community page][godot-community]. For example, their Discord server hosts a `#gdnative-dev` channel for questions regarding GDNative interface. The contributor chat is helpful in case of discussions about Godot's own design and implementation.

If you are only interested in occasional announcements and showcases, follow [@GodotRust on Twitter][godot-rust-twitter]. For questions and design discussions, please use Discord.

## License

Any contribution submitted for inclusion in the work by you shall be licensed under the [MIT license](LICENSE.md), without any additional terms or conditions.

[issue tracker]: https://github.com/godot-rust/godot-rust/issues
[`help wanted`]: https://github.com/godot-rust/godot-rust/issues?q=is%3Aissue+is%3Aopen+label%3A%22help+wanted%22
[`good first issue`]: https://github.com/godot-rust/godot-rust/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22
[godot-community]: https://godotengine.org/community
[godot-rust-discord]: https://discord.gg/FNudpBD
[godot-rust-twitter]: https://twitter.com/GodotRust
