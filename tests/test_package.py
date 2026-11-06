"""Regression tests for versioned packaging with a restored CI target cache."""
import contextlib
import hashlib
import importlib.util
import io
import json
import os
import tempfile
import unittest
from pathlib import Path
from unittest import mock

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("appleskin_package", ROOT / "tools/package.py")
package = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(package)


class PackageTests(unittest.TestCase):
    def setUp(self):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.root = Path(tmp.name)
        (self.root / "target/horizon-mod").mkdir(parents=True)
        (self.root / "horizon-mod.toml").write_text(
            (ROOT / "horizon-mod.toml").read_text(), encoding="utf-8"
        )
        (self.root / "compatibility.json").write_text(
            (ROOT / "compatibility.json").read_text(), encoding="utf-8"
        )
        self.payload = b"\x00asm-test-payload-for-packaging-only\n"
        (self.root / "target/horizon-mod/appleskin.hmod").write_bytes(self.payload)
        self.compat = json.loads((self.root / "compatibility.json").read_text())
        self.version = self.compat["mod_version"]
        self.release = self.root / "target/release"
        self.report = self.root / "target/report"

    def run_package(self):
        env = {
            "GITHUB_RUN_ID": "123456",
            "GITHUB_SERVER_URL": "https://github.com",
            "GITHUB_REPOSITORY": "jorginho11111/apple-skin-for-horizon",
            "GITHUB_SHA": "a" * 40,
        }
        with mock.patch.object(package, "ROOT", self.root), mock.patch.dict(os.environ, env):
            with contextlib.redirect_stdout(io.StringIO()):
                package.main()

    def test_restored_cache_cannot_leave_stale_release_assets(self):
        self.release.mkdir(parents=True)
        (self.release / "appleskin-0.0.1-android.hmod").write_bytes(b"old package")
        (self.release / "appleskin-0.0.1-android.hmod.sha256").write_text("old checksum")
        (self.release / "stale.txt").write_text("must not be published")
        self.report.mkdir(parents=True)
        (self.report / "clippy.json").write_text("diagnostics must survive packaging")

        self.run_package()

        name = f"appleskin-{self.version}-android.hmod"
        self.assertEqual(
            {p.name for p in self.release.iterdir()}, {name, f"{name}.sha256"}
        )
        artifact = self.release / name
        self.assertEqual(artifact.read_bytes(), self.payload)
        sha = hashlib.sha256(self.payload).hexdigest()
        self.assertEqual(
            (self.release / f"{name}.sha256").read_text(), f"{sha}  {name}\n"
        )
        build = json.loads((self.report / "build.json").read_text())
        self.assertEqual(build["mod_version"], self.version)
        self.assertEqual(build["artifact"], name)
        self.assertEqual(build["size_bytes"], len(self.payload))
        self.assertEqual(build["sha256"], sha)
        self.assertEqual(build["minimum_android"], self.compat["minimum_android"])
        self.assertEqual(build["highlights"], self.compat.get("highlights", []))
        self.assertEqual(build["ci"]["run_id"], "123456")
        self.assertEqual(build["ci"]["commit"], "a" * 40)
        self.assertTrue((self.report / "clippy.json").is_file())

    def test_missing_official_build_fails_without_publishing_old_assets(self):
        (self.root / "target/horizon-mod/appleskin.hmod").unlink()
        self.release.mkdir(parents=True)
        old = self.release / "appleskin-0.0.1-android.hmod"
        old.write_bytes(b"old package")
        with self.assertRaisesRegex(SystemExit, "run cargo horizon-mod build first"):
            self.run_package()
        self.assertTrue(old.is_file())
        self.assertFalse((self.report / "build.json").exists())

    def test_manifest_and_compatibility_versions_must_match(self):
        path = self.root / "compatibility.json"
        self.compat["mod_version"] = "0.0.0"
        path.write_text(json.dumps(self.compat), encoding="utf-8")
        with self.assertRaisesRegex(SystemExit, "disagree on mod version"):
            self.run_package()
        self.assertFalse((self.report / "build.json").exists())


if __name__ == "__main__":
    unittest.main()
