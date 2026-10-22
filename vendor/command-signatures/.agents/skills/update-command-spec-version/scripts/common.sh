#!/bin/bash
# Shared repo-path resolution for the update-command-spec-version scripts.
#
#   CMD_SIGS_DIR - the command-signatures repo, inferred from this script's own
#                  location (these scripts live inside that repo), so it always
#                  points at the checkout the skill ships from.
#   BRASA_DIR     - the brasa repo, taken from the $BRASA_DIR environment variable
#                  when set; otherwise a sibling of the command-signatures repo
#                  (same parent directory, named "brasa").

# Directory this file lives in (resolved through symlinks).
_COMMON_SH_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# The command-signatures repo root that contains these scripts.
CMD_SIGS_DIR="$(git -C "${_COMMON_SH_DIR}" rev-parse --show-toplevel)"

# The brasa repo: prefer $BRASA_DIR, else a sibling of command-signatures.
if [ -n "${BRASA_DIR:-}" ]; then
    BRASA_DIR="${BRASA_DIR}"
else
    BRASA_DIR="$(dirname "${CMD_SIGS_DIR}")/brasa"
fi

export CMD_SIGS_DIR BRASA_DIR
