#!/usr/bin/env python3
"""Tests for leak-check.py (F004/S01), on throwaway git repos in a temporary folder.

Run: python3 scripts/test_leak_check.py

The private terms here are made up, and the fake secrets are put together at run time, so this
file itself never trips the check.
"""

import json
import os
import random
import string
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).resolve().parent / "leak-check.py"
TERMS = "# test list\nzebrafish\nquokka\nallow:quokka-street\n"


def fake_token() -> str:
    r = random.Random(42)
    return "ghp" + "_" + "".join(r.choice(string.ascii_letters + string.digits) for _ in range(36))


def run(*args, cwd, env=None, stdin=""):
    return subprocess.run([sys.executable, str(SCRIPT), *args], cwd=cwd, input=stdin, text=True,
                          capture_output=True, env=env)


class Repo:
    def __init__(self, base: Path):
        self.remote = base / "remote.git"
        self.path = base / "work"
        self.terms = base / "terms.txt"
        self.terms.write_text(TERMS)
        self.env = {**os.environ, "LEAK_TERMS": str(self.terms)}
        subprocess.run(["git", "init", "-q", "--bare", "-b", "main", str(self.remote)], check=True)
        subprocess.run(["git", "clone", "-q", str(self.remote), str(self.path)], check=True, capture_output=True)
        self.git("config", "user.name", "Test")
        self.git("config", "user.email", "test@example.org")
        self.commit("README.md", "hello\n", "first")
        self.git("push", "-q", "origin", "HEAD:main")

    def git(self, *args):
        return subprocess.run(["git", *args], cwd=self.path, check=True, capture_output=True, text=True).stdout

    def commit(self, name, text, message):
        p = self.path / name
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(text)
        self.git("add", "-A")
        self.git("commit", "-q", "-m", message)

    def check(self, *args, env=None):
        return run(*(args or ("--push",)), cwd=self.path, env=env or self.env)

    def pre_push(self, ref="main"):
        """What git passes the hook on stdin for pushing `ref` to main."""
        local = self.git("rev-parse", ref).strip()
        remote = self.git("rev-parse", "origin/main").strip()
        return run("origin", str(self.remote), cwd=self.path, env=self.env,
                   stdin=f"refs/heads/{ref} {local} refs/heads/main {remote}\n")


class LeakCheck(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.repo = Repo(Path(self.tmp.name))

    def tearDown(self):
        self.tmp.cleanup()

    def test_a_clean_push_passes(self):
        self.repo.commit("src/a.rs", "fn main() {}\n", "add a")
        r = self.repo.pre_push()
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("niets gevonden", r.stdout)

    def test_a_secret_is_refused_without_being_repeated(self):
        token = fake_token()
        self.repo.commit("config.toml", f'token = "{token}"\n', "add config")
        r = self.repo.pre_push()
        self.assertEqual(r.returncode, 1)
        self.assertIn("config.toml:1: geheim (github-pat)", r.stderr)
        self.assertNotIn(token, r.stderr + r.stdout)

    def test_a_private_term_in_a_line_a_file_name_and_a_message_is_refused(self):
        self.repo.commit("docs/a.md", "one\nthe zebrafish lives here\n", "add a")
        self.repo.commit("docs/Quokka.md", "x\n", "add b")
        self.repo.commit("docs/c.md", "x\n", "about the zebrafish")
        r = self.repo.pre_push()
        self.assertEqual(r.returncode, 1)
        self.assertIn("docs/a.md:2: privéterm (regel 2 van de lijst)", r.stderr)
        self.assertIn("docs/Quokka.md: privéterm in de bestandsnaam (regel 3 van de lijst)", r.stderr)
        self.assertIn("bericht: privéterm (regel 2 van de lijst)", r.stderr)
        self.assertNotIn("lives here", r.stderr)

    def test_an_author_with_a_private_term_is_refused(self):
        self.repo.git("config", "user.name", "Pat Zebrafish")
        self.repo.commit("a.txt", "x\n", "add a")
        r = self.repo.check()
        self.assertEqual(r.returncode, 1)
        self.assertIn("auteur: privéterm", r.stderr)

    def test_an_allowed_term_passes_only_inside_its_allow_span(self):
        self.repo.commit("a.md", "we live in Quokka-Street\n", "add a")
        self.assertEqual(self.repo.check().returncode, 0)
        self.repo.commit("b.md", "a quokka and quokka-street\n", "add b")
        r = self.repo.check()
        self.assertEqual(r.returncode, 1)
        self.assertIn("b.md:1:", r.stderr)

    def test_removing_a_line_with_a_term_passes(self):
        self.repo.commit("a.md", "keep\nzebrafish\n", "oops")
        # the leak was pushed earlier (without the check); the clean-up must be pushable
        self.repo.git("push", "-q", "origin", "HEAD:main")
        self.repo.git("fetch", "-q")
        self.repo.commit("a.md", "keep\n", "remove it")
        r = self.repo.pre_push()
        self.assertEqual(r.returncode, 0, r.stderr)

    def test_a_missing_list_or_a_missing_gitleaks_refuses(self):
        self.repo.commit("a.txt", "x\n", "add a")
        env = {**self.repo.env, "LEAK_TERMS": str(Path(self.tmp.name) / "nope.txt")}
        r = self.repo.check(env=env)
        self.assertEqual(r.returncode, 1)
        self.assertIn("lijst met privétermen ontbreekt", r.stderr)
        env = {**self.repo.env, "LEAK_GITLEAKS": "/nonexistent/gitleaks"}
        r = self.repo.check(env=env)
        self.assertEqual(r.returncode, 1)
        self.assertIn("gitleaks ontbreekt", r.stderr)

    def test_a_new_branch_checks_only_what_no_remote_branch_has(self):
        # an old leak already on the remote is not this push's business
        self.repo.commit("old.md", "zebrafish\n", "old")
        self.repo.git("push", "-q", "origin", "HEAD:main")
        self.repo.git("fetch", "-q")
        self.repo.git("checkout", "-q", "-b", "feature")
        self.repo.commit("new.md", "fine\n", "new")
        local = self.repo.git("rev-parse", "feature").strip()
        r = run("origin", str(self.repo.remote), cwd=self.repo.path, env=self.repo.env,
                stdin=f"refs/heads/feature {local} refs/heads/feature {'0' * 40}\n")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.repo.commit("bad.md", "quokka\n", "bad")
        local = self.repo.git("rev-parse", "feature").strip()
        r = run("origin", str(self.repo.remote), cwd=self.repo.path, env=self.repo.env,
                stdin=f"refs/heads/feature {local} refs/heads/feature {'0' * 40}\n")
        self.assertEqual(r.returncode, 1)
        self.assertIn("bad.md:1:", r.stderr)
        self.assertNotIn("old.md", r.stderr)

    def test_tree_and_history(self):
        self.repo.commit("a.md", "zebrafish\n", "add")
        self.repo.commit("a.md", "clean\n", "clean up")
        self.assertEqual(self.repo.check("--tree").returncode, 0)
        r = self.repo.check("--history")
        self.assertEqual(r.returncode, 1)
        self.assertIn("a.md:1:", r.stderr)

    def test_the_agent_hook(self):
        self.repo.commit("a.md", "zebrafish\n", "add")
        hook = lambda cmd, cwd: run("--agent-hook", cwd=self.tmp.name, env=self.repo.env,
                                    stdin=json.dumps({"tool_input": {"command": cmd}, "cwd": str(cwd)}))
        r = hook(f"cd {self.repo.path} && git push", self.tmp.name)
        self.assertEqual(r.returncode, 0, "no leak-check.py in that repo: not ours")
        self.assertEqual(hook("git push --no-verify", self.repo.path).returncode, 0, "not ours either")
        (self.repo.path / "scripts").mkdir()
        (self.repo.path / "scripts" / "leak-check.py").write_text("")
        r = hook(f"cd {self.repo.path} && git push", self.tmp.name)
        self.assertEqual(r.returncode, 2)
        self.assertIn("privéterm", r.stderr)
        self.assertEqual(hook("git -C . push --no-verify", self.repo.path).returncode, 2)
        self.assertEqual(hook("git push -n origin main", self.repo.path).returncode, 2)
        self.assertEqual(hook("git status && ls", self.repo.path).returncode, 0)
        # a commit message or heredoc that mentions a push is not one
        self.assertEqual(hook('git commit -m "never git push --no-verify"', self.repo.path).returncode, 0)
        self.assertEqual(hook("cat > x <<'EOF'\ngit push --no-verify\nEOF\nls", self.repo.path).returncode, 0)
        self.assertEqual(hook('git commit -m "x" && git push --no-verify', self.repo.path).returncode, 2)
        self.repo.commit("a.md", "clean\n", "clean")
        self.repo.git("push", "-q", "origin", "HEAD:main")
        self.repo.git("fetch", "-q")
        self.assertEqual(hook(f"git -C {self.repo.path} push", self.tmp.name).returncode, 0)


if __name__ == "__main__":
    unittest.main(verbosity=2)
