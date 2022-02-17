# Small utility to run update crate versions
# Used by godot-rust developers

# No args specified: do everything
if [ "$#" -eq 0 ]; then
    echo "Usage: update-version.sh <newVersion>"
    exit 1
fi

# --help menu
args=("$@")
for arg in "${args[@]}"; do
    if [ "$arg" == "--help" ]; then
        echo "Usage: update-version.sh <newVersion>"
        echo ""
        echo "Replaces currently published version with <newVersion>".
        echo "Does not git commit."
        exit 0
    fi
done

# Uncommitted changes, see https://stackoverflow.com/a/3879077
#if git diff --quiet --exit-code; then
git diff-index --quiet HEAD -- || {
    echo "Repo contains uncommitted changes; make sure working tree is clean."
    exit 1
}

# https://stackoverflow.com/a/11114547
scriptFile=$(realpath "$0")
scriptPath=$(dirname "$scriptFile")

newVersion="${args[0]}"
oldVersion=$(grep -Po '^version = "\K[^"]*' "$scriptPath/../gdnative/Cargo.toml")

publishedCrates=(
    "impl/proc-macros"
    "gdnative-sys"
    "gdnative-derive"
    "gdnative-core"
    "bindings-generator"
    "gdnative-bindings"
    "gdnative-async"
    "gdnative"
)

for crate in "${publishedCrates[@]}"; do
    # Don't just replace version string itself -- the following only replaces the crate's own version
    # (with 'version = "1.2.3"') and dependencies with "=1.2.3", which makes false positives unlikely
    sed -i "s!version = \"${oldVersion}\"!version = \"${newVersion}\"!g" "$scriptPath/../$crate/Cargo.toml" || exit 2
    sed -i "s!\"=${oldVersion}\"!\"=${newVersion}\"!g" "$scriptPath/../$crate/Cargo.toml" || exit 2
done

git commit -am "Update godot-rust version: $oldVersion -> $newVersion" || exit 2
git tag "$newVersion" || exit 2

echo "Updated version $oldVersion -> $newVersion"
