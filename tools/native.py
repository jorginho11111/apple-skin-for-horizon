#!/usr/bin/env python3
"""Package this native mod and compile it into a pinned Horizon checkout.

Only GitHub HTTPS endpoints are used for source downloads. No Minecraft assets,
account credentials, or prebuilt game binaries are downloaded by this script.
"""
from __future__ import annotations

import argparse
import hashlib
import io
import json
import os
import re
import shutil
import subprocess
import sys
import tarfile
import tomllib
import urllib.request
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
TARGET = ROOT / "target"
BUNDLE = ROOT / "mixins" / "appleskin"
SHA = re.compile(r"[0-9a-f]{40}\Z")


def log(message: str) -> None:
    print(message, file=sys.stderr, flush=True)


def request(url: str) -> urllib.request.Request:
    headers = {"User-Agent": "apple-skin-for-horizon-ci"}
    token = os.environ.get("GITHUB_TOKEN") or os.environ.get("GH_TOKEN")
    if token and url.startswith("https://api.github.com/"):
        headers["Authorization"] = f"Bearer {token}"
    return urllib.request.Request(url, headers=headers)


def get(url: str) -> bytes:
    with urllib.request.urlopen(request(url), timeout=300) as response:
        return response.read()


def metadata() -> tuple[dict, dict]:
    compat = json.loads((ROOT / "compatibility.json").read_text())
    manifest = tomllib.loads((BUNDLE / "horizon-mixin.toml").read_text())
    if manifest["mod"]["id"] != "appleskin":
        raise ValueError("unexpected mod id")
    if manifest["compatibility"]["horizon_revision"] != compat["horizon_revision"]:
        raise ValueError("Horizon pins differ")
    for key in ("horizon_revision", "horizon_mixins_revision"):
        if not SHA.fullmatch(compat[key]):
            raise ValueError(f"{key} must be an exact lowercase commit SHA")
    return compat, manifest


def package() -> Path:
    compat, manifest = metadata()
    version = manifest["mod"]["version"]
    native = {
        "schema": 1,
        "id": "appleskin",
        "version": version,
        "game": compat["game"],
        "horizon": compat["horizon_revision"],
        "mixins": compat["horizon_mixins_revision"],
        "hooks": compat["hooks"],
    }
    files = {"horizon-native.json": (json.dumps(native, indent=2, sort_keys=True) + "\n").encode()}
    for path in sorted(BUNDLE.rglob("*")):
        if path.is_symlink():
            raise ValueError(f"symlink in bundle: {path}")
        if path.is_file() and path.suffix in (".rs", ".toml"):
            files[f"mixins/appleskin/{path.relative_to(BUNDLE).as_posix()}"] = path.read_bytes()
    files["mixins/appleskin/LICENSE"] = (ROOT / "LICENSE").read_bytes()
    files["mixins/appleskin/LICENSE-APPLESKIN"] = (ROOT / "LICENSE-APPLESKIN").read_bytes()
    files["mixins/appleskin/NOTICE"] = (ROOT / "NOTICE").read_bytes()
    TARGET.mkdir(exist_ok=True)
    out = TARGET / f"appleskin-{version}.hnmod"
    with zipfile.ZipFile(out, "w", compression=zipfile.ZIP_DEFLATED) as archive:
        for name, content in sorted(files.items()):
            info = zipfile.ZipInfo(name, (2026, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o100644 << 16
            archive.writestr(info, content)
    digest = hashlib.sha256(out.read_bytes()).hexdigest()
    (TARGET / f"{out.name}.sha256").write_text(f"{digest}  {out.name}\n")
    log(f"Packaged {out} ({out.stat().st_size} bytes, sha256 {digest})")
    return out


def extract(data: bytes, destination: Path) -> None:
    destination.parent.mkdir(parents=True, exist_ok=True)
    partial = destination.with_name(destination.name + ".partial")
    shutil.rmtree(partial, ignore_errors=True)
    partial.mkdir(parents=True)
    with tarfile.open(fileobj=io.BytesIO(data), mode="r:gz") as archive:
        # Python 3.11 in the sandbox lacks extractall(filter=...); validate first.
        members = []
        for member in archive.getmembers():
            path = Path(member.name)
            if path.is_absolute() or ".." in path.parts:
                raise ValueError(f"unsafe source archive entry: {member.name}")
            # Documentation links are not needed to compile; never extract links.
            if member.issym() or member.islnk():
                continue
            if not (member.isfile() or member.isdir()):
                raise ValueError(f"special source archive entry: {member.name}")
            members.append(member)
        archive.extractall(partial, members=members)
    top = next(partial.iterdir())
    shutil.rmtree(destination, ignore_errors=True)
    top.rename(destination)
    partial.rmdir()


def source(repo: str, revision: str, destination: Path) -> Path:
    if not SHA.fullmatch(revision):
        raise ValueError("source revisions must be exact commit SHAs")
    if destination.is_dir():
        return destination
    log(f"Downloading {repo}@{revision[:12]}")
    extract(get(f"https://codeload.github.com/{repo}/tar.gz/{revision}"), destination)
    # A codeload archive has no .git. Without this marker the mixin tool walks up
    # to the mod repository's git HEAD and rejects the Horizon pin (E_REVISION).
    (destination / ".horizon-revision").write_text(revision + "\n")
    # Horizon has documentation symlinks; the mixin tool refuses links in a workspace.
    for path in destination.rglob("*"):
        if path.is_symlink():
            path.unlink()
    return destination


def run(args: list[str], cwd: Path | None = None) -> None:
    log("+ " + " ".join(args))
    result = subprocess.run(args, cwd=cwd, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    if result.stdout:
        print(result.stdout, end="", flush=True)
    if result.returncode:
        TARGET.mkdir(exist_ok=True)
        (TARGET / "failed-command.log").write_text(result.stdout or "")
        lines = [line for line in (result.stdout or "").splitlines()
                 if line.lstrip().startswith(("error", "warning:", "Caused by:", "help:", "note:"))
                 or "error[" in line or "error:" in line]
        tail = "\n".join(lines[-80:]) or (result.stdout or "")[-12000:]
        print("::error::" + tail.replace("%", "%25").replace("\r", "%0D").replace("\n", "%0A"), flush=True)
        raise subprocess.CalledProcessError(result.returncode, args)


def resolve(compat: dict, horizon: str | None) -> str:
    if not horizon:
        return compat["horizon_revision"]
    repo = compat["horizon_repository"].removeprefix("https://github.com/")
    if horizon == "latest":
        refs = json.loads(get(f"https://api.github.com/repos/{repo}/git/matching-refs/tags/nightly-source/"))
        tags = [r["ref"].removeprefix("refs/tags/") for r in refs]
        if not tags:
            raise ValueError("Horizon has no nightly-source tags")
        horizon = max(tags, key=lambda tag: tag.removeprefix("nightly-source/")[:10])
    if SHA.fullmatch(horizon):
        return horizon
    commit = json.loads(get(f"https://api.github.com/repos/{repo}/commits/{horizon}"))
    return commit["sha"]


def check(horizon: str | None, build: bool) -> None:
    compat, _ = metadata()
    revision = resolve(compat, horizon)
    game = source("horizon-rust/horizon", revision, TARGET / "horizon" / revision)
    hooks_file = game / "crates/horizon-client/src/game/native_hooks.rs"
    found = re.search(r"^pub\(crate\) const NATIVE_HOOKS_API: u32 = (\d+);$", hooks_file.read_text(), re.M)
    api = int(found.group(1)) if found else 0
    if api < compat["hooks"]:
        raise ValueError(f"Horizon has hooks API {api}; this mod needs {compat['hooks']}")
    mixins_rev = compat["horizon_mixins_revision"]
    tool_root = TARGET / "horizon-mixins" / mixins_rev
    tool = tool_root / "bin" / "horizon-mixins"
    if not tool.is_file():
        run(["cargo", "install", "--locked", "--git", "https://github.com/horizon-rust/horizon-mixins",
             "--rev", mixins_rev, "--root", str(tool_root), "horizon-mixins-cli"])
    bundles = TARGET / "bundles"
    shutil.rmtree(bundles, ignore_errors=True)
    shutil.copytree(BUNDLE, bundles / "appleskin")
    staged_manifest = bundles / "appleskin/horizon-mixin.toml"
    staged_manifest.write_text(re.sub(r'horizon_revision = "[0-9a-f]{40}"',
                                      f'horizon_revision = "{revision}"', staged_manifest.read_text()))
    shutil.copy2(ROOT / "LICENSE", bundles / "appleskin/LICENSE")
    shutil.copy2(ROOT / "NOTICE", bundles / "appleskin/NOTICE")
    stage = TARGET / f"game-{revision}"
    run([str(tool), "apply", "--workspace", str(game), "--mods", str(bundles), "--out", str(stage)])
    command = ["cargo", "build" if build else "check", "--locked", "-p", "horizon-client", "--bin", "horizon-client"]
    if build:
        command += ["--profile", "fast"]
    env = dict(os.environ, CARGO_TARGET_DIR=str(TARGET / "cargo"))
    log("+ " + " ".join(command))
    result = subprocess.run(command, cwd=stage, env=env, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    if result.stdout:
        print(result.stdout, end="", flush=True)
    if result.returncode:
        TARGET.mkdir(exist_ok=True)
        (TARGET / "failed-command.log").write_text(result.stdout or "")
        lines = [line for line in (result.stdout or "").splitlines()
                 if line.lstrip().startswith(("error", "warning:", "Caused by:", "help:", "note:"))
                 or "error[" in line or "error:" in line]
        tail = "\n".join(lines[-80:]) or (result.stdout or "")[-12000:]
        print("::error::" + tail.replace("%", "%25").replace("\r", "%0D").replace("\n", "%0A"), flush=True)
        raise subprocess.CalledProcessError(result.returncode, command)
    report = {"mod": "appleskin", "horizon": revision, "hooks_api": api,
              "mixins": mixins_rev, "result": "build-ok" if build else "check-ok"}
    (TARGET / "compile-report.json").write_text(json.dumps(report, indent=2) + "\n")
    log(json.dumps(report))


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="action", required=True)
    sub.add_parser("package")
    for action in ("check", "build"):
        p = sub.add_parser(action)
        p.add_argument("--horizon", help="exact commit, tag, or latest; default: compatibility.json")
    args = parser.parse_args()
    if args.action == "package":
        package()
    else:
        check(args.horizon, args.action == "build")


if __name__ == "__main__":
    try:
        main()
    except (ValueError, OSError, subprocess.CalledProcessError) as error:
        sys.exit(str(error))
