#!/usr/bin/env python3
"""Apple's side of a release (F004/S06), each step retried with a growing pause, so a release
that stops on Apple's network can be re-run from the job that failed.

  scripts/apple.py notarize <file>          submit, or reuse an earlier submission, and wait
  scripts/apple.py staple <file>            staple the ticket and validate it
  scripts/apple.py validate <file>          only validate the ticket that is there
  scripts/apple.py assess <file> execute|open   Gatekeeper accepts it as notarized

`notarize` names the upload `<name>-<first 12 of its SHA-256>.<ext>`, so a re-run finds the
submission of the same file in `notarytool history` and waits for it instead of submitting again.
It needs APPLE_ID, APPLE_PASSWORD and APPLE_TEAM_ID. A refusal by Apple (Invalid, Rejected)
fails at once, with Apple's log; only what looks like the network is retried.
"""

import hashlib
import json
import os
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path

ATTEMPTS = 5
PAUSE = 20  # seconds, times the attempt: 20, 40, 60, 80


class Refused(Exception):
    """Apple said no: retrying won't help."""


def run(cmd: list[str]) -> subprocess.CompletedProcess:
    return subprocess.run(cmd, capture_output=True, text=True)


def retry(what: str, step, attempts: int = ATTEMPTS, sleep=time.sleep):
    """`step()` until it returns without raising; `Refused` stops at once."""
    for attempt in range(1, attempts + 1):
        try:
            return step()
        except Refused:
            raise
        except Exception as e:  # the network, Apple's service: try again
            if attempt == attempts:
                raise RuntimeError(f"{what}: gave up after {attempts} attempts: {e}") from e
            print(f"::warning::{what}: attempt {attempt} of {attempts} failed: {e}", flush=True)
            sleep(PAUSE * attempt)


def upload_name(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for block in iter(lambda: f.read(4 << 20), b""):
            h.update(block)
    stem, dot, ext = path.name.partition(".")
    return f"{stem}-{h.hexdigest()[:12]}{dot}{ext}"


def earlier(history: dict, name: str) -> dict | None:
    """The newest submission of this upload name that is accepted or still running."""
    found = [h for h in history.get("history", []) if h.get("name") == name and h.get("status") in ("Accepted", "In Progress")]
    found.sort(key=lambda h: h.get("createdDate", ""), reverse=True)
    return found[0] if found else None


def auth() -> list[str]:
    missing = [k for k in ("APPLE_ID", "APPLE_PASSWORD", "APPLE_TEAM_ID") if not os.environ.get(k)]
    if missing:
        raise SystemExit(f"::error::missing {', '.join(missing)}")
    return ["--apple-id", os.environ["APPLE_ID"], "--password", os.environ["APPLE_PASSWORD"], "--team-id", os.environ["APPLE_TEAM_ID"]]


def notary_json(args: list[str], runner=run) -> dict:
    p = runner(["xcrun", "notarytool", *args, *auth(), "--output-format", "json"])
    try:
        out = json.loads(p.stdout or "{}")
    except json.JSONDecodeError:
        out = {}
    if p.returncode != 0 and "status" not in out:
        raise RuntimeError((p.stderr or p.stdout).strip()[-400:] or f"exit {p.returncode}")
    return out


def notarize(path: Path, runner=run, sleep=time.sleep, summary=None) -> str:
    name = upload_name(path)
    found = retry("notarytool history", lambda: earlier(notary_json(["history"], runner), name), sleep=sleep)
    if found:
        sub_id = found["id"]
        print(f"reusing submission {sub_id} of {name} ({found['status']})", flush=True)
    else:
        with tempfile.TemporaryDirectory() as tmp:
            upload = Path(tmp) / name
            shutil.copyfile(path, upload)
            sub_id = retry("notarytool submit", lambda: notary_json(["submit", str(upload)], runner)["id"], sleep=sleep)
        print(f"submitted {name}: {sub_id}", flush=True)
    if summary:
        with open(summary, "a") as f:
            f.write(f"- notarization of `{path.name}`: `{sub_id}`\n")

    def wait() -> str:
        out = notary_json(["wait", sub_id, "--timeout", "30m"], runner)
        status = out.get("status")
        if status == "Accepted":
            return status
        if status in ("Invalid", "Rejected"):
            log = runner(["xcrun", "notarytool", "log", sub_id, *auth()])
            print(log.stdout, flush=True)
            raise Refused(f"Apple: {status} ({sub_id})")
        raise RuntimeError(f"still {status or 'unknown'}")

    return retry("notarytool wait", wait, sleep=sleep)


def staple(path: Path, runner=run, sleep=time.sleep, verbs=("staple", "validate")) -> None:
    def once() -> None:
        for verb in verbs:
            p = runner(["xcrun", "stapler", verb, str(path)])
            if p.returncode != 0:
                raise RuntimeError(f"stapler {verb}: {(p.stdout + p.stderr).strip()[-300:]}")
        print(f"{' and '.join(verbs)}: {path.name}", flush=True)

    retry("stapler", once, sleep=sleep)


def assess(path: Path, kind: str, runner=run, sleep=time.sleep) -> None:
    extra = ["--context", "context:primary-signature"] if kind == "open" else []
    def once() -> None:
        p = runner(["spctl", "--assess", "--type", kind, *extra, "--verbose=4", str(path)])
        text = (p.stdout + p.stderr).strip()
        print(text, flush=True)
        if "source=Notarized Developer ID" not in text:
            raise RuntimeError(f"not accepted as notarized: {text[-200:]}")

    retry("spctl", once, sleep=sleep)


def main(argv: list[str]) -> int:
    if len(argv) < 2 or argv[0] not in ("notarize", "staple", "validate", "assess"):
        print(__doc__)
        return 2
    path = Path(argv[1])
    try:
        if argv[0] == "notarize":
            notarize(path, summary=os.environ.get("GITHUB_STEP_SUMMARY"))
        elif argv[0] == "staple":
            staple(path)
        elif argv[0] == "validate":
            staple(path, verbs=("validate",))
        else:
            assess(path, argv[2] if len(argv) > 2 else "execute")
    except (Refused, RuntimeError) as e:
        print(f"::error::{e}", flush=True)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
