#!/usr/bin/env python3
"""Leak check for this public repo (F004/S01): refuses a push that adds a secret or a private term.

Secrets are found by gitleaks. Private terms come from a list outside the repo, because the list
is exactly what may not leak (default ~/.config/family-lightbox/leak-terms.txt). Only what is
being added counts: the added lines of the diffs, file names, commit messages and authors. A
push that removes a leak goes through.

  leak-check.py <remote> <url>   as git's pre-push hook (the refs arrive on stdin)
  leak-check.py --push           every local commit that is on no remote branch
  leak-check.py --tree           every file in HEAD
  leak-check.py --history        every commit
  leak-check.py --agent-hook     Claude Code PreToolUse hook: checks before a `git push`

Fails closed: without gitleaks or without the list, nothing passes.
"""

from __future__ import annotations

import json
import os
import re
import shlex
import shutil
import subprocess
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path

TERMS = Path(os.environ.get("LEAK_TERMS", Path.home() / ".config/family-lightbox/leak-terms.txt"))
GITLEAKS = os.environ.get("LEAK_GITLEAKS", "gitleaks")
ZERO = "0" * 40
MARK = "\x1e@@commit@@"


class Refused(Exception):
    pass


@dataclass
class Terms:
    deny: list[tuple[int, re.Pattern]]
    allow: list[re.Pattern]

    def findings(self, text: str) -> list[int]:
        """The line numbers (in the terms file) of the private patterns that match outside an allowed span."""
        spans = [m.span() for a in self.allow for m in a.finditer(text)]
        out = []
        for n, p in self.deny:
            for m in p.finditer(text):
                if not any(s <= m.start() and m.end() <= e for s, e in spans):
                    out.append(n)
                    break
        return out


def load_terms(path: Path = TERMS) -> Terms:
    if not path.is_file():
        raise Refused(f"de lijst met privétermen ontbreekt: {path}\n(zie LibraryLocal docs/features/F010-privacy-app-repo)")
    deny, allow = [], []
    for n, raw in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        try:
            if line.startswith("allow:"):
                allow.append(re.compile(line[6:].strip(), re.I))
            else:
                deny.append((n, re.compile(line, re.I)))
        except re.error as e:
            raise Refused(f"{path}:{n}: ongeldig patroon ({e})")
    if not deny:
        raise Refused(f"{path} bevat geen enkel patroon")
    return Terms(deny, allow)


def git(*args: str, cwd: Path | None = None) -> str:
    r = subprocess.run(["git", *args], cwd=cwd, capture_output=True, text=True, encoding="utf-8", errors="replace")
    if r.returncode != 0:
        raise Refused(f"git {' '.join(args)}: {r.stderr.strip()}")
    return r.stdout


def scan_log(terms: Terms, revs: list[str], cwd: Path | None = None) -> list[str]:
    """Private terms in the authors, messages, file names and added lines of the given commits."""
    if not revs:
        return []
    fmt = f"{MARK}%H%x00%an <%ae>%x00%cn <%ce>%x00%B%x00"
    log = git("log", "-p", "--no-color", "--no-ext-diff", "-M", f"--format={fmt}", *revs, cwd=cwd)
    out: list[str] = []
    for chunk in log.split(MARK)[1:]:
        sha, author, committer, message, diff = (chunk.split("\x00", 4) + [""] * 5)[:5]
        short = sha[:9]
        for what, text in (("auteur", author), ("committer", committer), ("bericht", message)):
            for n in terms.findings(text):
                out.append(f"{short} {what}: privéterm (regel {n} van de lijst)")
        path, line = "?", 0
        for d in diff.splitlines():
            if d.startswith("+++ "):
                path = d[6:] if d.startswith("+++ b/") else d[4:]
                if path != "/dev/null":
                    for n in terms.findings(path):
                        out.append(f"{short} {path}: privéterm in de bestandsnaam (regel {n} van de lijst)")
            elif d.startswith("rename to "):
                for n in terms.findings(d[10:]):
                    out.append(f"{short} {d[10:]}: privéterm in de bestandsnaam (regel {n} van de lijst)")
            elif d.startswith("@@"):
                m = re.search(r"\+(\d+)", d)
                line = int(m.group(1)) if m else 0
            elif d.startswith("+"):
                for n in terms.findings(d[1:]):
                    out.append(f"{short} {path}:{line}: privéterm (regel {n} van de lijst)")
                line += 1
            elif d.startswith(" "):
                line += 1
    return out


def gitleaks(args: list[str]) -> list[str]:
    """Secrets found by gitleaks, as `<commit> <file>:<line>: <rule>`; the secret itself is never printed."""
    if shutil.which(GITLEAKS) is None:
        raise Refused(f"gitleaks ontbreekt ({GITLEAKS}): brew install gitleaks")
    with tempfile.TemporaryDirectory() as tmp:
        report = Path(tmp) / "report.json"
        r = subprocess.run(
            [GITLEAKS, *args, "--redact", "--no-banner", "--log-level", "error",
             "--report-format", "json", "--report-path", str(report), "--exit-code", "1"],
            capture_output=True, text=True,
        )
        if r.returncode not in (0, 1):
            raise Refused(f"gitleaks faalde: {r.stderr.strip()[:500]}")
        found = json.loads(report.read_text() or "[]") if report.exists() else []
    return [f"{(f.get('Commit') or 'boom')[:9]} {f.get('File')}:{f.get('StartLine')}: geheim ({f.get('RuleID')})" for f in found]


def check_revs(terms: Terms, revs: list[str], cwd: Path) -> list[str]:
    if not revs or not git("rev-list", "--max-count=1", *revs, cwd=cwd).strip():
        return []
    return scan_log(terms, revs, cwd) + gitleaks(["git", str(cwd), "--log-opts", " ".join(revs)])


def check_tree(terms: Terms, cwd: Path) -> list[str]:
    out: list[str] = []
    files = [f for f in git("ls-files", "-z", cwd=cwd).split("\x00") if f]
    with tempfile.TemporaryDirectory() as tmp:
        for f in files:
            for n in terms.findings(f):
                out.append(f"boom {f}: privéterm in de bestandsnaam (regel {n} van de lijst)")
            p = cwd / f
            if not p.is_file():
                continue
            data = p.read_bytes()
            if b"\x00" in data[:8000]:
                continue  # binary
            for i, text in enumerate(data.decode("utf-8", "replace").splitlines(), 1):
                for n in terms.findings(text):
                    out.append(f"boom {f}:{i}: privéterm (regel {n} van de lijst)")
            dest = Path(tmp) / f
            dest.parent.mkdir(parents=True, exist_ok=True)
            dest.write_bytes(data)
        out += gitleaks(["dir", tmp])
    return out


def pre_push_revs(remote: str, stdin: str, cwd: Path) -> list[str]:
    """The commits git is about to push: per ref, those the remote doesn't have yet."""
    revs: list[str] = []
    for line in stdin.splitlines():
        parts = line.split()
        if len(parts) != 4 or parts[1] == ZERO:
            continue  # a delete pushes nothing
        local, remote_sha = parts[1], parts[3]
        known = remote_sha != ZERO and subprocess.run(
            ["git", "cat-file", "-e", f"{remote_sha}^{{commit}}"], cwd=cwd, capture_output=True
        ).returncode == 0
        if known:
            revs += [local, f"^{remote_sha}"]
        else:
            # a new branch, or a remote tip we don't have (a rewritten history): everything the
            # remote's branches as we know them don't already contain
            revs.append(local)
            revs += [f"^{r}" for r in git("for-each-ref", "--format=%(objectname)", f"refs/remotes/{remote}", cwd=cwd).split()]
    return revs


def push_revs(cwd: Path) -> list[str]:
    """Every local commit that is on no remote branch: whatever a push could send."""
    branches = git("for-each-ref", "--format=%(objectname)", "refs/heads", cwd=cwd).split()
    remotes = git("for-each-ref", "--format=%(objectname)", "refs/remotes", cwd=cwd).split()
    return branches + [f"^{r}" for r in remotes]


def report(findings: list[str], what: str) -> int:
    if not findings:
        print(f"leak-check: {what}: niets gevonden")
        return 0
    refused = ", push geweigerd" if what == "push" else ""
    print(f"leak-check: {what}: {len(findings)} bevinding(en){refused}:", file=sys.stderr)
    for f in findings:
        print(f"  {f}", file=sys.stderr)
    print("Haal het weg en commit opnieuw (een amend of rebase), of overleg met Peter. Nooit --no-verify.", file=sys.stderr)
    return 1


PUSH = re.compile(r"(^|[;&|(\s])git(\s+-C\s+\S+|\s+-c\s+\S+)*\s+push\b")
HEREDOC = re.compile(r"<<-?\s*(['\"]?)(\w+)\1[^\n]*\n.*?\n\s*\2\b", re.S)
QUOTED = re.compile(r"'[^']*'|\"(?:\\.|[^\"\\])*\"")


def unquoted(command: str) -> str:
    """The command without heredoc bodies and quoted strings, so a commit message that mentions
    `git push` isn't taken for one."""
    return QUOTED.sub("''", HEREDOC.sub("<<''", command))


def push_dir(command: str, cwd: Path) -> Path:
    """Where a `git push` in a shell command runs: after `git -C <dir>`, or a leading `cd <dir>`."""
    m = re.search(r"git\s+-C\s+(\S+)", command)
    if m:
        return (cwd / Path(os.path.expanduser(shlex.split(m.group(1))[0]))).resolve()
    m = re.match(r"\s*cd\s+(\S+)\s*(&&|;)", command)
    if m:
        return (cwd / Path(os.path.expanduser(shlex.split(m.group(1))[0]))).resolve()
    return cwd


def agent_hook(stdin: str) -> int:
    """Claude Code PreToolUse: exit 2 blocks the command and shows stderr to the agent."""
    try:
        event = json.loads(stdin)
    except json.JSONDecodeError:
        return 0
    command = unquoted((event.get("tool_input") or {}).get("command") or "")
    if not PUSH.search(command):
        return 0
    repo = push_dir(command, Path(event.get("cwd") or os.getcwd()))
    try:
        top = Path(git("rev-parse", "--show-toplevel", cwd=repo).strip())
    except Refused:
        return 0
    if not (top / "scripts" / "leak-check.py").is_file():
        return 0  # another repo
    if re.search(r"\s--no-verify\b|\spush\b[^;&|]*\s-n\b", command):
        print("leak-check: `git push --no-verify` is niet toegelaten in deze repo (F004/S01).", file=sys.stderr)
        return 2
    try:
        return 2 if report(check_revs(load_terms(), push_revs(top), top), "push") else 0
    except Refused as e:
        print(f"leak-check: {e}", file=sys.stderr)
        return 2


def main(argv: list[str]) -> int:
    if argv[:1] == ["--agent-hook"]:
        return agent_hook(sys.stdin.read())
    cwd = Path(git("rev-parse", "--show-toplevel").strip())
    try:
        terms = load_terms()
        if argv[:1] == ["--tree"]:
            return report(check_tree(terms, cwd), "boom")
        if argv[:1] == ["--history"]:
            return report(check_revs(terms, ["--all"], cwd), "geschiedenis")
        if argv[:1] == ["--push"]:
            return report(check_revs(terms, push_revs(cwd), cwd), "push")
        if argv and not argv[0].startswith("-"):
            return report(check_revs(terms, pre_push_revs(argv[0], sys.stdin.read(), cwd), cwd), "push")
    except Refused as e:
        print(f"leak-check: {e}", file=sys.stderr)
        return 1
    print(__doc__, file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
