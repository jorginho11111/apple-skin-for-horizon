import importlib.util
import json
import hashlib
import tempfile
import tomllib
import unittest
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("native_tools", ROOT / "tools" / "native.py")
native = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(native)


class PackageTests(unittest.TestCase):
    def test_pins_and_versions_agree(self):
        compat, manifest = native.metadata()
        native_manifest = json.loads((ROOT / "horizon-native.json").read_text())
        cargo = tomllib.loads((ROOT / "Cargo.toml").read_text())
        self.assertEqual(manifest["mod"]["version"], cargo["package"]["version"])
        self.assertEqual(manifest["mod"]["version"], native_manifest["version"])
        self.assertEqual(compat["horizon_revision"], native_manifest["horizon"])
        self.assertEqual(compat["horizon_mixins_revision"], native_manifest["mixins"])
        self.assertEqual(compat["hooks"], native_manifest["hooks"])
        self.assertEqual(compat["game"], "26.3")

    def test_uses_named_hooks_not_raw_targets(self):
        manifest = tomllib.loads((ROOT / "mixins/appleskin/horizon-mixin.toml").read_text())
        expected = {"hud": "gui_layer_post", "list": "mods", "settings": "settings", "click": "settings_click"}
        for injection in manifest["injection"]:
            self.assertEqual(injection["hook"], expected[injection["id"]])
            self.assertEqual(injection["hook_version"], 1)
            self.assertNotIn("target", injection)
        # The compile job resolves these names against the exact pinned native-hooks.json.

    def test_package_is_deterministic_and_contains_notices(self):
        with tempfile.TemporaryDirectory() as tmp:
            old = native.TARGET
            native.TARGET = Path(tmp)
            try:
                first = native.package()
                before = first.read_bytes()
                second = native.package()
                self.assertEqual(before, second.read_bytes())
                self.assertEqual(hashlib.sha256(before).hexdigest(), hashlib.sha256(second.read_bytes()).hexdigest())
                with zipfile.ZipFile(first) as z:
                    self.assertIsNone(z.testzip())
                    names = z.namelist()
                    self.assertEqual(names, sorted(set(names)))
                    self.assertIn("mixins/appleskin/LICENSE", names)
                    self.assertIn("mixins/appleskin/LICENSE-APPLESKIN", names)
                    self.assertIn("mixins/appleskin/NOTICE", names)
                    self.assertFalse(any(n.endswith((".class", ".jar", ".wasm", ".so", ".dll")) for n in names))
                    self.assertEqual(json.loads(z.read("horizon-native.json"))["schema"], 1)
            finally:
                native.TARGET = old


if __name__ == "__main__":
    unittest.main()
