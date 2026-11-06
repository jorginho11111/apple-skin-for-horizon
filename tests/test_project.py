import json, tomllib, unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

class ProjectTests(unittest.TestCase):
    def test_manifest_is_android_hmod(self):
        m = tomllib.loads((ROOT / "horizon-mod.toml").read_text())
        self.assertEqual(m["id"], "appleskin")
        self.assertEqual(m["api"], "0.1")
        self.assertEqual(m["side"], "client")
        self.assertIn("input", m["capabilities"])
        self.assertIn("render", m["capabilities"])
        self.assertEqual(set(m["experimental"]), {"input", "render", "ui"})
        self.assertFalse((ROOT / "horizon-native.json").exists())

    def test_versions_match(self):
        m = tomllib.loads((ROOT / "horizon-mod.toml").read_text())
        c = tomllib.loads((ROOT / "Cargo.toml").read_text())
        compat = json.loads((ROOT / "compatibility.json").read_text())
        self.assertEqual(m["version"], c["package"]["version"])
        self.assertEqual(m["version"], compat["mod_version"])
        self.assertEqual(compat["game"], "26.3")
        self.assertEqual(compat["package_format"], "hmod")
        self.assertEqual(compat["horizon_android_release"], "v0.7.7")

    def test_food_table_sorted_and_edible(self):
        data = json.loads((ROOT / "food-data/foods-26.3.json").read_text())
        items = data["items"]
        self.assertEqual(list(items), sorted(items))
        self.assertGreaterEqual(len(items), 40)
        self.assertEqual(items["minecraft:bread"]["nutrition"], 5)
        self.assertAlmostEqual(items["minecraft:bread"]["saturation"], 6.0)
        self.assertNotIn("minecraft:cod_bucket", items)
        self.assertNotIn("minecraft:cake", items)
        self.assertTrue(items["minecraft:golden_apple"]["can_always_eat"])

    def test_generated_foods_matches_data(self):
        import subprocess, sys
        before = (ROOT / "src/foods.rs").read_text()
        subprocess.run([sys.executable, str(ROOT / "tools/generate_foods.py")], check=True, capture_output=True)
        self.assertEqual(before, (ROOT / "src/foods.rs").read_text())

if __name__ == "__main__":
    unittest.main()
