#!/usr/bin/bash

# Small utility to detect todo comments
# Used by godot-rust developers

REGEX='(?<!clippy::)(TODO|FIXME)(?!!\(\)|[a-zA-Z]|\((#|(\w+/)+)\d+\))'

# Self test

TEST_OUTPUT=$(rg -i --pcre2 "$REGEX" <<EOF
# Should hit - Naive

todo
TODO
fixme
FiXmE
reallytodo
trulyfixme

# Should hit - Malformed

todo123
fixme123
TODO(123)
TODO(/////)
TODO(T123)

# Should not hit - With issue numbers

todo(#123)
todo(foo/bar/123)
fixme(#123)
fixme(foo/bar/123)

# Should not hit - Other uses of the words

clippy::todo
todo!()

# Should not hit - Words merely containing the keywords

mastodon
ictodosaur
EOF
)

diff -u --color <(printf '%s\n' "$TEST_OUTPUT") <(cat <<EOF
todo
TODO
fixme
FiXmE
reallytodo
trulyfixme
todo123
fixme123
TODO(123)
TODO(/////)
TODO(T123)
EOF
)

if [[ $? -ne 0 ]]
then
    echo 'Test run of detection regex failed.'
    exit 1
fi

# Actual run

rg -iTh -Tsh --pcre2 "$REGEX" "$(pwd)"
if [[ $? -eq 0 ]]
then
    echo 'Found TODO comments without issue numbers.'
    exit 1
fi
