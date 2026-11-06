#!/usr/bin/env python3
"""Copy the official cargo-horizon-mod output to versioned release files."""
from __future__ import annotations
import hashlib, json, shutil, tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

def main() -> None:
    manifest = tomllib.loads((ROOT / "horizon-mod.toml").read_text())
    compat = json.loads((ROOT / "compatibility.json").read_text())
    version = manifest["version"]
    src = ROOT / "target/horizon-mod/appleskin.hmod"
    if not src.is_file():
        raise SystemExit("missing target/horizon-mod/appleskin.hmod; run cargo horizon-mod build first")
    release = ROOT / "target/release"
    report = ROOT / "target/report"
    # CI caches target/ between runs; never publish a stale versioned artifact.
    if release.exists():
        shutil.rmtree(release)
    release.mkdir(parents=True, exist_ok=True)
    report.mkdir(parents=True, exist_ok=True)
    out = release / f"appleskin-{version}-android.hmod"
    shutil.copyfile(src, out)
    sha = hashlib.sha256(out.read_bytes()).hexdigest()
    (release / f"{out.name}.sha256").write_text(f"{sha}  {out.name}\n")
    build = {
        "name": "AppleSkin",
        "mod_version": version,
        "platform": "android",
        "format": "hmod",
        "game": compat["game"],
        "horizon_android_release": compat["horizon_android_release"],
        "horizon_android_revision": compat["horizon_android_revision"],
        "mod_api": compat["mod_api"],
        "rust": compat["rust"],
        "wasm_target": compat["wasm_target"],
        "supported_apk_architectures": compat["supported_apk_architectures"],
        "artifact": out.name,
        "size_bytes": out.stat().st_size,
        "sha256": sha,
        "limitations": compat["limitations"],
        "ci": {
            "repository": "jorginho11111/apple-skin-for-horizon",
            "workflow": "Build AppleSkin Android hmod",
            "run_id": "${{ github.run_id }}",
        },
    }
    # GitHub Actions expressions are not expanded inside Python; read env if present.
    import os
    build["ci"]["run_id"] = os.environ.get("GITHUB_RUN_ID", "local")
    build["ci"]["run_url"] = os.environ.get("GITHUB_SERVER_URL", "https://github.com") + "/" + os.environ.get("GITHUB_REPOSITORY", "jorginho11111/apple-skin-for-horizon") + "/actions/runs/" + build["ci"]["run_id"]
    build["ci"]["commit"] = os.environ.get("GITHUB_SHA", "local")
    (report / "build.json").write_text(json.dumps(build, indent=2, ensure_ascii=False) + "\n")
    print(f"packaged {out} sha256={sha}")

if __name__ == "__main__":
    main()
