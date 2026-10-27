#!/usr/bin/env python3
"""Tests for build-latest-json.py, in throwaway folders."""

import importlib.util
import json
import tempfile
import unittest
from datetime import datetime, timezone
from pathlib import Path

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("build_latest_json", HERE / "build-latest-json.py")
blj = importlib.util.module_from_spec(spec)
spec.loader.exec_module(blj)

NOW = datetime(2026, 10, 27, 20, 0, 0, tzinfo=timezone.utc)
REPO = "atomicbits/family-lightbox"


class BuildLatestJson(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.dir = Path(self.tmp.name)

    def tearDown(self):
        self.tmp.cleanup()

    def payload(self, name="FamilyLightbox.app.tar.gz", sig="dW50cnVzdGVkIGNvbW1lbnQ=\n"):
        (self.dir / name).write_bytes(b"tar")
        if sig is not None:
            (self.dir / (name + ".sig")).write_text(sig)

    def test_manifest_for_apple_silicon(self):
        self.payload()
        (self.dir / "FamilyLightbox_0.2.0_aarch64.dmg").write_bytes(b"dmg")
        m = blj.build("0.2.0", self.dir, REPO, NOW)
        self.assertEqual(m["version"], "0.2.0")
        self.assertEqual(m["pub_date"], "2026-10-27T20:00:00Z")
        self.assertEqual(list(m["platforms"]), ["darwin-aarch64"])
        p = m["platforms"]["darwin-aarch64"]
        self.assertEqual(p["signature"], "dW50cnVzdGVkIGNvbW1lbnQ=")
        self.assertEqual(
            p["url"], "https://github.com/atomicbits/family-lightbox/releases/download/v0.2.0/FamilyLightbox.app.tar.gz"
        )

    def test_url_is_quoted(self):
        self.payload(name="Family Lightbox.app.tar.gz")
        url = blj.build("0.2.0", self.dir, REPO, NOW)["platforms"]["darwin-aarch64"]["url"]
        self.assertTrue(url.endswith("/Family%20Lightbox.app.tar.gz"))

    def test_refuses_what_the_updater_would_choke_on(self):
        with self.assertRaisesRegex(ValueError, "found 0"):
            blj.build("0.2.0", self.dir, REPO, NOW)
        self.payload(sig=None)
        with self.assertRaisesRegex(ValueError, "no signature"):
            blj.build("0.2.0", self.dir, REPO, NOW)
        self.payload(sig="  \n")
        with self.assertRaisesRegex(ValueError, "empty"):
            blj.build("0.2.0", self.dir, REPO, NOW)
        self.payload(name="Other.app.tar.gz")
        with self.assertRaisesRegex(ValueError, "found 2"):
            blj.build("0.2.0", self.dir, REPO, NOW)
        with self.assertRaisesRegex(ValueError, "not a version"):
            blj.build("v0.2.0", self.dir, REPO, NOW)

    def test_main_writes_the_file(self):
        self.payload()
        self.assertEqual(blj.main(["0.2.1", str(self.dir), REPO]), 0)
        m = json.loads((self.dir / "latest.json").read_text())
        self.assertEqual(m["version"], "0.2.1")
        self.assertIn("/v0.2.1/", m["platforms"]["darwin-aarch64"]["url"])


if __name__ == "__main__":
    unittest.main()
