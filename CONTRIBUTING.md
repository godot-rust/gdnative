# Contributing to godot's rust bindings.

The godot rust bindings developers welcome contribution from everyone. Here are the guidelines if you are thinking of helping us:

## Contributions

Contributions to should be made in the form of GitHub pull requests.
Each pull request will be reviewed by a core contributor (someone with permission to land patches) and either landed in the main tree or given feedback for changes that would be required.

Should you wish to work on an issue, please claim it first by commenting on the GitHub issue that you want to work on it. This is to prevent duplicated efforts from contributors on the same issue.

## Getting started

Have a look at the [issues](https://github.com/GodotNativeTools/godot-rust/issues) to find good tasks to start with.

## Pull Request Checklist

- Branch from the master branch and, if needed, rebase to the current master branch before submitting your pull request. If it doesn't merge cleanly with master you may be asked to rebase your changes.

- Commits should be as small as possible, while ensuring that each commit is correct independently (i.e., each commit should compile and pass tests).

- If your patch is not getting reviewed or you need a specific person to review it, you can @-reply a reviewer asking for a review in the pull request or a comment.

- Whenever applicable, add tests relevant to the fixed bug or new feature.

## Testing

- run `cargo test --all` from the root of the repository.
- on GNU/Linux OSes:

```
cd test
cargo build
cp ../target/debug/libgdnative_test.so ./project/lib
godot --path ./project
```

The `godot` command in the above snippet is your local installation of godot and may vary depending on how it was installed.

## Writing tests

Some types can only be used if the engine is running, in order to test them, use the `godot_test!` macro (see examples in [variant.rs](gdnative/src/variant.rs)), and explicitly invoke the test functions in [test/src/lib.rs](test/src/lib.rs).

## Communication

Communcation around the bindings should happen on IRC on one of the following channels:

 - #godotengine-gdnative in freenode
 - #rust-gamedev in irc.mozilla.org.

## License

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you shall be licensed under the [MIT license](LICENSE.md), without any additional terms or conditions.
