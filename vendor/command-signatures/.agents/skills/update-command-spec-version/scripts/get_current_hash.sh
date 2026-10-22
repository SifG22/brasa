#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/common.sh"

# Read Cargo.toml from origin/master so we don't depend on local checkout state.
HASH=$(git -C "${BRASA_DIR}" show origin/master:Cargo.toml \
    | grep 'brasa-command-signatures.*rev' \
    | grep -o 'rev = "[^"]*"' \
    | grep -o '"[^"]*"' \
    | tr -d '"')

if [ -z "${HASH}" ]; then
    echo "Error: could not find brasa-command-signatures rev in Cargo.toml" >&2
    exit 1
fi

echo "${HASH}"
