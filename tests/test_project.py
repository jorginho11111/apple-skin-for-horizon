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
        self.assertNotIn("registries", m["capabilities"])
        self.assertFalse((ROOT / "assets").exists())
        self.assertFalse((ROOT / "data").exists())

    def test_build_workflow_targets_the_active_branch_and_main(self):
        import re
        workflow = (ROOT / ".github/workflows/build.yml").read_text()
        branches = re.search(r"branches: \[(.*?)\]", workflow).group(1)
        self.assertEqual(
            {b.strip().strip('"') for b in branches.split(",")},
            {"arena/eb13e5b2-apple-skin-for-horizon", "main"},
        )

    def test_hud_uses_gain_only_hunger_and_original_yellow_runs(self):
        lib = (ROOT / "src/lib.rs").read_text()
        overlay = (ROOT / "src/overlay.rs").read_text()
        self.assertIn("mod saturation_icons;", lib)
        self.assertIn("hud/food_empty", lib)
        self.assertIn("p.hunger_before, p.hunger_after, x_right, y", lib)
        self.assertIn("icon.kind.runs()", lib)
        self.assertIn("| run.rgb", lib)
        self.assertIn("rem_euclid(PULSE_PERIOD)", lib)
        for source in (lib, overlay):
            self.assertNotIn("projected_icons", source)
            self.assertNotIn("saturation_rects", source)
            self.assertNotIn("SAT_OVERLAY_COLOR", source)

    def test_versions_match(self):
        m = tomllib.loads((ROOT / "horizon-mod.toml").read_text())
        c = tomllib.loads((ROOT / "Cargo.toml").read_text())
        compat = json.loads((ROOT / "compatibility.json").read_text())
        self.assertEqual(m["version"], c["package"]["version"])
        self.assertEqual(m["version"], compat["mod_version"])
        lock = tomllib.loads((ROOT / "Cargo.lock").read_text())
        root_package = next(p for p in lock["package"] if p["name"] == "appleskin-hmod")
        self.assertEqual(m["version"], root_package["version"])
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
