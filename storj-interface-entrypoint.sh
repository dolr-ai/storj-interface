#!/bin/bash

echo "=== Loading secrets and setting as environment variables ==="
if [ -n "$CREDENTIALS_DIRECTORY" ] && [ -d "$CREDENTIALS_DIRECTORY" ]; then
    echo "found secrets:"
    for cred in "$CREDENTIALS_DIRECTORY"/*; do
        if [ -f "$cred" ]; then
            secret_name=$(basename "$cred")
            declare -x $secret_name=$(cat "$cred")
            echo "  $secret_name"
        fi
    done
else
    echo "No credentials available"
fi

exec /usr/local/bin/storj-interface
