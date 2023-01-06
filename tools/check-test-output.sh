#!/usr/bin/bash

# Look for required error messages and leaked instances in the outputs of the integration test
# Usage: check-test-output.sh stdout.log stderr.log

FAIL=0

while read COUNT && read PATTERN; do
    ACTUAL="$(grep -c "$PATTERN" "$2")"
    if [[ $? -ne 0 ]]; then
        ACTUAL=0
    fi
    if [[ COUNT -ne ACTUAL ]]; then
        FAIL=1
        echo "Found ${ACTUAL} out of ${COUNT} expected instances of message ${PATTERN}"
    fi
done < <(sed -e 's/^#.*//' -e '/^$/d' -e 's/^\([0-9]\+\),/\1\n/' -e 's/\\/\\\\/g' "${BASH_SOURCE%/*}/required-errors.txt")

if grep -q "Leaked instance" "$1"; then
  echo Leaked instances found.
  FAIL=1
fi

exit $FAIL