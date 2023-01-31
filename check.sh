#!/bin/bash

# Small utility to run tests locally
# Similar to minimal-ci

# No args specified: do everything
if [ "$#" -eq 0 ]; then
    args=("fmt" "clippy" "test" "itest")
else
    args=("$@")
fi

# --help menu
for arg in "${args[@]}"; do
    if [ "$arg" == "--help" ]; then
        echo "Usage: check.sh [<commands>]"
        echo ""
        echo "Each specified command will be run (until one fails)."
        echo "If no commands are specified, all checks are run (no doc; may take several minutes)."
        echo ""
        echo "Commands:"
        echo "    fmt           format code, fail if bad"
        echo "    clippy        validate clippy lints"
        echo "    test          run unit tests (no Godot)"
        echo "    itest         run integration tests (Godot)"
        echo "    doc           generate docs for 'gdnative' crate"
        echo "    dok           generate docs and open in browser"
        echo ""
        echo "Examples:"
        echo "    check.sh fmt clippy"
        echo "    check.sh"
        exit 0
    fi
done

# For integration tests
function findGodot() {
    # User-defined GODOT_BIN
    if [ -n "$GODOT_BIN" ]; then
        echo "Found GODOT_BIN env var ($GODOT_BIN)"
        godotBin="$GODOT_BIN"

    #  Executable in path
    elif command -v godot &>/dev/null; then
        echo "Found 'godot' executable"
        godotBin="godot"

    # Special case for Windows when there is a .bat file
    # Also consider that 'cmd /c' would need 'cmd //c' (https://stackoverflow.com/q/21357813)
    elif
        # Godot returns 255 for older versions, but 0 for newer ones
        godot.bat --version
        [[ $? -eq 255 || $? -eq 0 ]]
    then
        echo "Found 'godot.bat' script"
        godotBin="godot.bat"

    # Error case
    else
        echo "Godot executable not found"
        exit 2
    fi
}

features="gdnative/async,gdnative/serde"
itest_toggled_features="gdnative/inventory,no-manual-register"
cmds=()

for arg in "${args[@]}"; do
    case "$arg" in
    fmt)
        cmds+=("cargo fmt --all -- --check")
        ;;
    clippy)
        cmds+=("cargo clippy --workspace --features $features -- -D clippy::style -D clippy::complexity -D clippy::perf -D clippy::dbg_macro -D clippy::todo -D clippy::unimplemented -D warnings")
        ;;
    test)
        cmds+=("cargo test --features $features")
        ;;
    itest)
        findGodot
        cmds+=("cargo build --manifest-path test/Cargo.toml --features $features")
        cmds+=("cp target/debug/*gdnative_test* test/project/lib/")
        cmds+=("$godotBin --path test/project")
        cmds+=("cargo build --manifest-path test/Cargo.toml --features $features,$itest_toggled_features")
        cmds+=("cp target/debug/*gdnative_test* test/project/lib/")
        cmds+=("$godotBin --path test/project")
        ;;
    doc)
        cmds+=("cargo doc --lib -p gdnative --no-deps --features $features")
        ;;
    dok)
        cmds+=("cargo doc --lib -p gdnative --no-deps --features $features --open")
        ;;
    *)
        echo "Unrecognized command '$arg'"
        exit 2
        ;;
    esac
done

RED='\033[1;31m'
GREEN='\033[1;36m'
END='\033[0m'
for cmd in "${cmds[@]}"; do
    echo "> $cmd"
    $cmd || {
        printf "$RED\n=========================="
        printf "\ngodot-rust checker FAILED."
        printf "\n==========================\n$END"
        exit 1
    }
done

printf "$GREEN\n=============================="
printf "\ngodot-rust checker SUCCESSFUL."
printf "\n==============================\n$END"
