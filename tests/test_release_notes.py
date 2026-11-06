"""PT-BR release notes must disclose the compatible preview and limitations."""
import json
import subprocess
import sys
import tempfile
import tomllib
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


class ReleaseNotesTests(unittest.TestCase):
    def test_notes_include_download_highlights_and_known_limits(self):
        compat = json.loads((ROOT / "compatibility.json").read_text(encoding="utf-8"))
        version = tomllib.loads((ROOT / "horizon-mod.toml").read_text(encoding="utf-8"))["version"]
        build = {
            "mod_version": version,
            "artifact": f"appleskin-{version}-android.hmod",
            "game": compat["game"],
            "horizon_android_release": compat["horizon_android_release"],
            "horizon_android_revision": compat["horizon_android_revision"],
            "mod_api": compat["mod_api"],
            "rust": compat["rust"],
            "wasm_target": compat["wasm_target"],
            "supported_apk_architectures": compat["supported_apk_architectures"],
            "minimum_android": compat["minimum_android"],
            "sha256": "0" * 64,
            "size_bytes": 1,
            "highlights": compat.get("highlights", []),
            "limitations": compat["limitations"],
            "ci": {"commit": "test", "run_id": "1", "run_url": "https://example.invalid"},
        }
        with tempfile.TemporaryDirectory() as tmp:
            report = Path(tmp) / "build.json"
            notes = Path(tmp) / "RELATORIO.md"
            report.write_text(json.dumps(build), encoding="utf-8")
            subprocess.run(
                [sys.executable, str(ROOT / "tools/release_notes.py"), str(report), str(notes)],
                check=True,
                capture_output=True,
            )
            text = notes.read_text(encoding="utf-8")
        self.assertIn(f"AppleSkin {version}", text)
        self.assertIn("SHA-256", text)
        self.assertIn("Destaques", text)
        self.assertIn("linha fixa logo acima da barra de fome", text)
        self.assertIn("sem movimento", text)
        self.assertIn("ganho potencial de saturação", text)
        self.assertIn("não é a saturação atual do jogador", text)
        self.assertIn("desenhados antes da barra de fome vanilla", text)
        self.assertIn("teste em aparelho Android real ainda é necessário", text)


if __name__ == "__main__":
    unittest.main()
