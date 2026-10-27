#!/usr/bin/env python3
"""Sets the app's version in its three places, refreshes Cargo.lock, and makes the release commit
and tag (F004/S03). Doesn't push: see docs/releasing.md.

  scripts/bump-version.py 0.2.0
"""

import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CONF = ROOT / "src-tauri" / "tauri.conf.json"
PKG = ROOT / "package.json"
CARGO = ROOT / "src-tauri" / "Cargo.toml"
SEMVER = re.compile(r"^(\d+)\.(\d+)\.(\d+)$")


def git(*args: str) -> str:
    return subprocess.run(["git", *args], cwd=ROOT, check=True, capture_output=True, text=True).stdout


def set_json_version(path: Path, version: str) -> None:
    # Replace only the version line, so the file keeps its own formatting.
    text = path.read_text()
    new, n = re.subn(r'^(\s*"version":\s*")[^"]*(")', rf"\g<1>{version}\g<2>", text, count=1, flags=re.M)
    if n != 1:
        sys.exit(f"no version in {path}")
    path.write_text(new)


def main(argv: list[str]) -> int:
    if len(argv) != 1 or not SEMVER.match(argv[0]):
        print(__doc__, file=sys.stderr)
        return 2
    version = argv[0]
    current = json.loads(CONF.read_text())["version"]
    as_tuple = lambda v: tuple(int(x) for x in SEMVER.match(v).groups())
    if as_tuple(version) <= as_tuple(current):
        sys.exit(f"{version} is not higher than the current {current}")
    if git("status", "--porcelain").strip():
        sys.exit("the working tree isn't clean: commit or stash first")
    if git("tag", "--list", f"v{version}").strip():
        sys.exit(f"tag v{version} exists already")

    set_json_version(CONF, version)
    set_json_version(PKG, version)
    cargo = CARGO.read_text()
    new, n = re.subn(r'^version = "[^"]*"', f'version = "{version}"', cargo, count=1, flags=re.M)
    if n != 1:
        sys.exit(f"no version in {CARGO}")
    CARGO.write_text(new)
    # Cargo.lock carries the package's own version too
    subprocess.run(["cargo", "update", "--offline", "--workspace"], cwd=CARGO.parent, check=True)

    git("add", str(CONF), str(PKG), str(CARGO), str(CARGO.parent / "Cargo.lock"))
    git("commit", "-q", "-m", f"Release v{version}")
    git("tag", "-a", f"v{version}", "-m", f"FamilyLightbox v{version}")
    print(f"v{version}: committed and tagged. Push with: git push origin main v{version}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
