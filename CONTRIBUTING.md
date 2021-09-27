# Contributing to godot's rust bindings.

The godot-rust bindings developers welcome contribution from everyone. Here are the guidelines if you are thinking of helping us:

## Contributions

Contributions should be made in the form of GitHub pull requests (PRs). Each pull request will be reviewed by a core contributor (someone with permission to land patches) and either landed in the main tree or given feedback for changes that would be required.

When opening a PR, our continuous integration (CI) pipeline will automatically perform a few basic checks (formatting, lints, unit tests). Please make sure those checks pass, adjusting your code if necessary.

Should you wish to work on an issue, please claim it first by commenting on the GitHub issue that you want to work on it. This is to prevent duplicated efforts from contributors on the same issue.

## Getting started

Have a look at the [issues](https://github.com/godot-rust/godot-rust/issues) to find good tasks to start with.

## Pull Request Checklist

- Branch from the master branch and, if needed, rebase to the current master branch before submitting your pull request. If it doesn't merge cleanly with master you may be asked to rebase your changes.

- Use one commit per logical change. Often, PRs only consist of a single commit. If you change unrelated parts of the code, split it into separate commits.

- If your patch is not getting reviewed or you need a specific person to review it, you can @-reply a reviewer asking for a review in the pull request or a comment.

- Whenever applicable, add tests relevant to the fixed bug or new feature.

- Use `cargo fmt` to format your code.

## Testing

Everywhere: run `cargo test --workspace` from the root of the repository.

On GNU/Linux operating systems:

```
cd test
cargo build
cp ../target/debug/libgdnative_test.so ./project/lib/
godot --path ./project
```

The `godot` command in the above snippet is your local installation of godot and may vary depending on how it was installed.

### Automating tests

If you are on a bash-compatible system, you can use the `pre-commit.sh` to automatically run your tests every time you try to commit code.  You can install it with:

```sh
$ ln -s ../../hooks/pre-commit.sh .git/hooks/pre-commit
$ chmod +x .git/hooks/pre-commit
```

If you don't need to run tests on your commit, you can simply run `git commit --no-verify` to skip the pre-commit script.  The pre-commit script handles a few edge cases as well, by stashing all of your changes before running tests, just in case your unstashed changes mask errors in the bare commit.  This is especially useful if you've stopped working on something to make a quick patch.

## Writing tests

Some types can only be used if the engine is running, in order to test them, use the `godot_test!` macro (see examples in [variant.rs](gdnative/src/variant.rs)), and explicitly invoke the test functions in [test/src/lib.rs](test/src/lib.rs).

## Communication

Primary communication between developers of the project happens on the `godot-rust` Discord server. [Invite link here](https://discord.gg/FNudpBD).

For questions about using `godot-rust` the `#gdnative` channel on the [Godot Engine community Discord server](https://godotengine.org/community) can be used.
On IRC the `#godotengine-gdnative` channel on freenode can be used for questions as well.

If you are only interested in occasional announcements and showcases, follow [@GodotRust on Twitter](https://twitter.com/GodotRust). For questions and design discussions, please use Discord.

## License

Any contribution submitted for inclusion in the work by you shall be licensed under the [MIT license](LICENSE.md), without any additional terms or conditions.
