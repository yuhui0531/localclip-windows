#!/usr/bin/env python3
"""Run two real exports, compare complete bytes, and preserve concise reproducible evidence."""
import hashlib
import os
from pathlib import Path
import shutil
import subprocess
import sys

from validate_vectors import validate

ROOT = Path(__file__).resolve().parent.parent
REFERENCE = Path(os.environ.get("LOCALCLIP_MAC_PATH", ROOT.parent / "rea" / "macos-app")).resolve()
RAW = ROOT / "reports" / "raw"


def status():
    return subprocess.check_output(["git", "--no-optional-locks", "-C", str(REFERENCE), "status", "--porcelain"], text=True)


def main():
    RAW.mkdir(parents=True, exist_ok=True)
    before = status()
    (RAW / "rea-before.txt").write_text(before)
    assert not before, "Reference must be clean before export"
    cache = ROOT / "vector-export" / ".build" / "cache"
    env = dict(os.environ, TZ="Asia/Shanghai", CLANG_MODULE_CACHE_PATH=str(cache), SWIFTPM_MODULECACHE_OVERRIDE=str(cache))
    output_a = ROOT / "test-vectors"
    output_b = ROOT / "vector-export" / ".build" / "repeat-vectors"
    for index, directory in enumerate((output_a, output_b), 1):
        if directory == output_b and directory.exists():
            shutil.rmtree(directory)
        command = ["swift", "run", "--package-path", "vector-export", "--cache-path", str(cache),
                   "--config-path", str(cache / "configuration"), "--security-path", str(cache / "security"),
                   "--manifest-cache", "local", "--disable-sandbox", "VectorExport", str(directory)]
        with (RAW / f"export-{index}.log").open("w") as log:
            result = subprocess.run(command, cwd=ROOT, env=env, stdout=log, stderr=subprocess.STDOUT)
        print(f"Export {index}: exit {result.returncode}", flush=True)
        if result.returncode:
            print((RAW / f"export-{index}.log").read_text())
            raise SystemExit(result.returncode)
        validate(directory)
    names = sorted(p.name for p in output_a.glob("*.json"))
    assert names == sorted(p.name for p in output_b.glob("*.json"))
    hashes = []
    for name in names:
        first, second = (output_a / name).read_bytes(), (output_b / name).read_bytes()
        assert first == second, f"Non-deterministic fixture: {name}"
        hashes.append(f"{hashlib.sha256(first).hexdigest()}  {name}")
    (RAW / "vectors.sha256").write_text("\n".join(hashes) + "\n")
    after = status()
    (RAW / "rea-after.txt").write_text(after)
    assert not after, "Reference must be clean after export"
    assert not list((ROOT / "vector-export" / ".build").glob("fixtures-*")), "Temporary fixture databases leaked"
    print("PASS: 15/15 complete files byte-identical; reference clean before/after; temporary databases removed")


if __name__ == "__main__":
    main()
