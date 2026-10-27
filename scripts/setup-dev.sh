#!/bin/sh
# Once per clone: use the repo's git hooks (the leak check before every push, F004/S01), and
# check that what the check needs is there.
set -e
cd "$(git rev-parse --show-toplevel)"
git config core.hooksPath .githooks
echo "core.hooksPath = .githooks"
command -v gitleaks >/dev/null || echo "missing: gitleaks (brew install gitleaks)"
[ -f "${LEAK_TERMS:-$HOME/.config/family-lightbox/leak-terms.txt}" ] \
  || echo "missing: the private terms list (~/.config/family-lightbox/leak-terms.txt, kept outside the repo)"
