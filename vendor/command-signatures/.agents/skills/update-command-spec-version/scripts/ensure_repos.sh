#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/common.sh"

# command-signatures is the repo these scripts live in, so it always exists.
# Only refresh its remotes; never mutate the working tree or checked-out branch
# (the skill reads origin/main directly via get_latest_hash.sh).
#
# Fetch the *exact* remote ref each later script reads (origin/main here,
# origin/master in brasa) with an explicit refspec. A bare `git fetch origin`
# follows the checkout's configured refspec, so a branch-limited or shallow
# clone could otherwise leave origin/main / origin/master missing or stale.
echo "Fetching command-signatures (${CMD_SIGS_DIR})..." >&2
git -C "${CMD_SIGS_DIR}" fetch origin main:refs/remotes/origin/main

if [ -d "${BRASA_DIR}" ]; then
    echo "Fetching brasa (${BRASA_DIR})..." >&2
    git -C "${BRASA_DIR}" fetch origin master:refs/remotes/origin/master
else
    echo "Cloning brasa into ${BRASA_DIR}..." >&2
    git clone ssh://git@github.com/SifG22/brasa.git "${BRASA_DIR}"
fi

echo "Both repositories are ready." >&2
