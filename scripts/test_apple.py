#!/usr/bin/env python3
"""Tests for scripts/apple.py (F004/S06), without Apple: a fake `xcrun` and `spctl`.

  python3 scripts/test_apple.py
"""

import importlib.util
import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

spec = importlib.util.spec_from_file_location("apple", Path(__file__).with_name("apple.py"))
apple = importlib.util.module_from_spec(spec)
spec.loader.exec_module(apple)

os.environ.update({"APPLE_ID": "id", "APPLE_PASSWORD": "pw", "APPLE_TEAM_ID": "team"})
failures = 0


def check(name: str, ok: bool, info: object = "") -> None:
    global failures
    print(("  ✓ " if ok else "  ✗ ") + name + ("" if ok else f"  {info}"))
    failures += 0 if ok else 1


def done(out: object = "", code: int = 0, err: str = "") -> subprocess.CompletedProcess:
    return subprocess.CompletedProcess([], code, json.dumps(out) if isinstance(out, dict) else out, err)


class Fake:
    """Answers per verb from a list, one answer per call; records the calls."""

    def __init__(self, answers: dict[str, list]):
        self.answers = answers
        self.calls: list[list[str]] = []

    def __call__(self, cmd: list[str]) -> subprocess.CompletedProcess:
        self.calls.append(cmd)
        verb = cmd[2] if cmd[0] == "xcrun" else cmd[0]
        return self.answers[verb].pop(0)

    def verbs(self) -> list[str]:
        return [c[2] if c[0] == "xcrun" else c[0] for c in self.calls]


no_sleep = lambda s: None
d = Path(tempfile.mkdtemp())
app = d / "FamilyLightbox.app.zip"
app.write_bytes(b"app")
name = apple.upload_name(app)

print("— names and history")
check("upload name carries the hash", name.startswith("FamilyLightbox.app-") and name.endswith(".zip") and len(name) == len("FamilyLightbox.app-") + 12 + len(".zip"), name)
dmg = d / "FamilyLightbox_0.4.4_aarch64.dmg"
dmg.write_bytes(b"dmg")
check("the version stays whole", apple.upload_name(dmg).startswith("FamilyLightbox_0.4.4_aarch64-") and apple.upload_name(dmg).endswith(".dmg"), apple.upload_name(dmg))
hist = {"history": [
    {"name": name, "status": "Invalid", "id": "bad", "createdDate": "2026-10-30T22:00:00Z"},
    {"name": name, "status": "Accepted", "id": "old", "createdDate": "2026-10-30T21:00:00Z"},
    {"name": name, "status": "In Progress", "id": "new", "createdDate": "2026-10-30T23:00:00Z"},
    {"name": "other.zip", "status": "Accepted", "id": "x", "createdDate": "2026-10-31T00:00:00Z"},
]}
check("the newest accepted or running submission of this file", apple.earlier(hist, name)["id"] == "new")
check("an invalid one isn't reused", apple.earlier({"history": hist["history"][:1]}, name) is None)

print("— notarize")
f = Fake({"history": [done({"history": []})], "submit": [done(code=1, err="network lost"), done({"id": "s1"})], "wait": [done({"status": "Accepted"})]})
check("submits (retrying the network) and waits", apple.notarize(app, f, no_sleep) == "Accepted" and f.verbs() == ["history", "submit", "submit", "wait"], f.verbs())
check("the upload has the hashed name", any(c[2] == "submit" and c[3].endswith(name) for c in f.calls))
f = Fake({"history": [done({"history": [{"name": name, "status": "Accepted", "id": "s9", "createdDate": "x"}]})], "wait": [done({"status": "Accepted"})]})
apple.notarize(app, f, no_sleep)
check("a re-run reuses the earlier submission", f.verbs() == ["history", "wait"] and f.calls[1][3] == "s9", f.calls)
f = Fake({"history": [done({"history": []})], "submit": [done({"id": "s2"})], "wait": [done({"status": "Invalid"}, code=1)], "log": [done("the log")]})
try:
    apple.notarize(app, f, no_sleep)
    check("an Invalid notarization fails at once", False)
except apple.Refused:
    check("an Invalid notarization fails at once, with the log", f.verbs() == ["history", "submit", "wait", "log"], f.verbs())
f = Fake({"history": [done({"history": []})], "submit": [done({"id": "s3"})], "wait": [done(code=1, err="timeout"), done({"status": "Accepted"})]})
check("a wait that loses the network is tried again", apple.notarize(app, f, no_sleep) == "Accepted")
summary = d / "summary.md"
f = Fake({"history": [done({"history": []})], "submit": [done({"id": "s4"})], "wait": [done({"status": "Accepted"})]})
apple.notarize(app, f, no_sleep, summary=summary)
check("the id goes to the job summary", "`s4`" in summary.read_text())

print("— staple and assess")
f = Fake({"staple": [done(code=65, err="CloudKit"), done("ok")], "validate": [done("The validate action worked!")]})
apple.staple(app, f, no_sleep)
check("a failed staple is tried again, then validated", f.verbs() == ["staple", "staple", "validate"], f.verbs())
f = Fake({"staple": [done("ok")] * 5, "validate": [done(code=1, err="no ticket")] * 5})
try:
    apple.staple(app, f, no_sleep)
    check("gives up after five attempts", False)
except RuntimeError as e:
    check("gives up after five attempts", "5 attempts" in str(e) and f.verbs().count("staple") == 5, e)
f = Fake({"validate": [done(code=1, err="lost"), done("ok")]})
apple.staple(app, f, no_sleep, verbs=("validate",))
check("validate alone staples nothing", f.verbs() == ["validate", "validate"], f.verbs())
f = Fake({"spctl": [done(err="rejected"), done(err="x: accepted\nsource=Notarized Developer ID")]})
apple.assess(app, "open", f, no_sleep)
check("spctl is tried again, with the context for a disk image", len(f.calls) == 2 and "context:primary-signature" in f.calls[0], f.calls)
pauses = []
apple.retry("x", lambda: (_ for _ in ()).throw(RuntimeError("n")) if len(pauses) < 2 else "ok", sleep=pauses.append)
check("the pause grows", pauses == [20, 40], pauses)

print(f"\n{'all passed' if failures == 0 else f'{failures} failed'}")
sys.exit(1 if failures else 0)
