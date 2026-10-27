#!/usr/bin/env python3
"""Writes the updater's latest.json for a release (F004/S04), from the .app.tar.gz and its .sig in
a folder. The app reads it at
https://github.com/<repo>/releases/latest/download/latest.json.

  scripts/build-latest-json.py 0.2.0 artefacts/ atomicbits/family-lightbox
"""

import json
import re
import sys
from datetime import datetime, timezone
from pathlib import Path
from urllib.parse import quote

SEMVER = re.compile(r"^\d+\.\d+\.\d+$")
PLATFORM = "darwin-aarch64"  # Apple Silicon only (F004, decision 2)


def build(version: str, folder: Path, repo: str, now: datetime) -> dict:
    if not SEMVER.match(version):
        raise ValueError(f"not a version: {version}")
    payloads = sorted(folder.glob("*.app.tar.gz"))
    if len(payloads) != 1:
        raise ValueError(f"expected exactly one .app.tar.gz in {folder}, found {len(payloads)}")
    payload = payloads[0]
    sig = payload.with_name(payload.name + ".sig")
    if not sig.is_file():
        raise ValueError(f"no signature next to {payload.name}")
    signature = sig.read_text().strip()
    if not signature:
        raise ValueError(f"{sig.name} is empty")
    url = f"https://github.com/{repo}/releases/download/v{version}/{quote(payload.name)}"
    return {
        "version": version,
        "notes": f"FamilyLightbox {version}",
        "pub_date": now.astimezone(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "platforms": {PLATFORM: {"signature": signature, "url": url}},
    }


def main(argv: list[str]) -> int:
    if len(argv) != 3:
        print(__doc__, file=sys.stderr)
        return 2
    version, folder, repo = argv[0], Path(argv[1]), argv[2]
    try:
        manifest = build(version, folder, repo, datetime.now(timezone.utc))
    except ValueError as e:
        print(f"build-latest-json: {e}", file=sys.stderr)
        return 1
    (folder / "latest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    print(f"latest.json: {version} for {PLATFORM}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
