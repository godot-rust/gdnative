#!/bin/bash

# Small utility to compile and run examples

# No args specified: do everything
if [ "$#" -eq 0 ]; then
    args=("--help")
else
    args=("$@")
fi

# --help menu
for arg in "${args[@]}"; do
    if [ "$arg" == "--help" ]; then
        echo "Usage: example.sh <command> <example-name>"
        echo ""
        echo "Commands:"
        echo "    run           run the specified example"
        echo "    edit          open the specified example in the editor"
        echo ""
        echo "Examples:"
        echo "    example.sh run hello-world"
        exit 0
    fi
done

if [ "$#" -ne 2 ]; then
    echo "Both the command and the name of the example are required."
    exit 1
fi

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

example_path="${BASH_SOURCE%/*}/examples/${args[1]}"

if ! [[ -d "$example_path" ]]; then
    echo "The example ${args[1]} is not found."
    exit 2
fi

findGodot

cmds=()

case "${args[0]}" in
run)
    cmds+=("cargo build --manifest-path $example_path/Cargo.toml")
    cmds+=("$godotBin --path $example_path")
    ;;
edit)
    cmds+=("cargo build --manifest-path $example_path/Cargo.toml")
    cmds+=("$godotBin -e --path $example_path")
    ;;
*)
    echo "Unrecognized command '$arg'"
    exit 2
    ;;
esac

for cmd in "${cmds[@]}"; do
    echo "> $cmd"
    $cmd || {
        exit 1
    }
done
