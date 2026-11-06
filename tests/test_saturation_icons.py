"""Verify the vendored AppleSkin texture and its dependency-free extraction."""
import hashlib
import importlib.util
import json
import tempfile
import unittest
from pathlib import Path
from unittest import mock

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location(
    "saturation_icons_generator", ROOT / "tools/generate_saturation_icons.py"
)
icons = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(icons)

# Independently transcribed from the original first four 9x9 texture cells.
# A/B/C/D are the four original gold colors; dots must remain transparent.
EXPECTED = [
    [
        ".........",
        ".........",
        ".........",
        ".........",
        "......C..",
        "......C..",
        ".....D.CC",
        "......D.C",
        "......DD.",
    ],
    [
        "...A.....",
        "....B....",
        ".....C...",
        "......C..",
        "......C..",
        "......C..",
        "....DD.CC",
        "......D.C",
        "......DD.",
    ],
    [
        "..AA.....",
        "....B....",
        ".....C...",
        "......C..",
        "......C..",
        "..D...C..",
        "...DDD.CC",
        "......D.C",
        "......DD.",
    ],
    [
        "..AA.....",
        ".B..B....",
        "C....C...",
        "C.....C..",
        ".D....C..",
        "..D...C..",
        "...DDD.CC",
        "......D.C",
        "......DD.",
    ],
]
COLORS = {0xE9D262: "A", 0xDDC241: "B", 0xC2A100: "C", 0x9F8609: "D"}


class SaturationIconTests(unittest.TestCase):
    def test_vendored_texture_and_license_match_the_pinned_source(self):
        data = icons.SOURCE.read_bytes()
        self.assertEqual(hashlib.sha256(data).hexdigest(), icons.SOURCE_SHA256)
        compat = json.loads((ROOT / "compatibility.json").read_text())
        source = compat["appleskin_overlay"]
        self.assertEqual(source["revision"], icons.REVISION)
        self.assertEqual(source["sha256"], icons.SOURCE_SHA256)
        self.assertEqual(ROOT / source["local"], icons.SOURCE)
        self.assertEqual(ROOT / source["generator"], ROOT / "tools/generate_saturation_icons.py")
        self.assertEqual(ROOT / source["generated"], icons.OUTPUT)
        self.assertEqual(
            icons.SOURCE.with_name("LICENSE").read_bytes(),
            (ROOT / "LICENSE-APPLESKIN").read_bytes(),
        )

    def test_indexed_png_decoder_preserves_every_original_pixel_and_color(self):
        width, height, rows = icons.rgba_rows(icons.SOURCE.read_bytes())
        self.assertEqual((width, height, len(rows)), (256, 256, 256))
        for variant, expected in enumerate(EXPECTED):
            actual = []
            for y, expected_row in enumerate(expected):
                row = ""
                for x, expected_pixel in enumerate(expected_row):
                    offset = (variant * 9 + x) * 4
                    r, g, b, alpha = rows[y][offset : offset + 4]
                    if expected_pixel == ".":
                        self.assertEqual(alpha, 0)
                        row += "."
                    else:
                        self.assertEqual(alpha, 255)
                        row += COLORS[(r << 16) | (g << 8) | b]
                actual.append(row)
            self.assertEqual(actual, expected)

    def test_pixel_runs_reconstruct_the_masks_without_overlap(self):
        counts = []
        for runs, expected in zip(icons.icon_runs(), EXPECTED, strict=True):
            pixels = set()
            for y, x, width, rgb in runs:
                for dx in range(width):
                    self.assertLess(x + dx, 9)
                    self.assertLess(y, 9)
                    self.assertNotIn((x + dx, y), pixels)
                    pixels.add((x + dx, y))
                    self.assertEqual(expected[y][x + dx], COLORS[rgb])
            self.assertEqual(len(pixels), sum(row.count(c) for row in expected for c in "ABCD"))
            counts.append(len(pixels))
        self.assertEqual(counts, [9, 14, 17, 21])
        run_counts = [len(runs) for runs in icons.icon_runs()]
        self.assertEqual(run_counts, [7, 11, 12, 16])
        # Worst case: 10 empty + 10 restored sprites + 10 full yellow outlines.
        self.assertLessEqual(20 + 10 * run_counts[-1], 180)

    def test_generated_rust_is_current_and_deterministic(self):
        before = icons.OUTPUT.read_text(encoding="utf-8")
        rendered = icons.render(icons.icon_runs())
        self.assertEqual(before, rendered)
        self.assertEqual(rendered, icons.render(icons.icon_runs()))
        self.assertIn(icons.REVISION, rendered)
        self.assertIn(icons.SOURCE_SHA256, rendered)
        self.assertIn("pub const ICONS: [&[PixelRun]; 4]", rendered)

    def test_changed_texture_is_rejected_before_generation(self):
        with tempfile.TemporaryDirectory() as tmp:
            changed = Path(tmp) / "icons.png"
            changed.write_bytes(icons.SOURCE.read_bytes() + b"changed")
            with mock.patch.object(icons, "SOURCE", changed):
                with self.assertRaisesRegex(ValueError, "pinned AppleSkin icons.png changed"):
                    icons.icon_runs()

    def test_decoder_rejects_non_png_input(self):
        with self.assertRaisesRegex(ValueError, "not a PNG"):
            icons.rgba_rows(b"not a texture")


if __name__ == "__main__":
    unittest.main()
