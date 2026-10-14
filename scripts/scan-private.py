"""Scan an explicitly marked synthetic acceptance directory without printing content or paths."""
import argparse
import json
from pathlib import Path


def scan(root, needles_file):
    root, needles_file = root.resolve(), needles_file.resolve()
    if not (root / ".localclip-test-data").is_file() or needles_file.is_relative_to(root):
        raise ValueError("invalid synthetic test directory")
    values = json.loads(needles_file.read_text(encoding="utf-8"))
    if not isinstance(values, list) or not values or any(
        not isinstance(v, str) or not v or len(v) > 4096 for v in values
    ):
        raise ValueError("invalid generated test strings")
    needles = [v.encode(encoding) for v in values for encoding in ("utf-8", "utf-16le")]
    overlap = max(map(len, needles)) - 1
    files = leaks = unreadable = 0
    pending = [root]
    while pending:
        directory = pending.pop()
        try:
            entries = list(directory.iterdir())
        except OSError:
            unreadable += 1
            continue
        for entry in entries:
            if entry.is_symlink():
                unreadable += 1
            elif entry.is_dir():
                pending.append(entry)
            elif entry.is_file():
                files += 1
                try:
                    with entry.open("rb") as stream:
                        tail = b""
                        found = False
                        while chunk := stream.read(1024 * 1024):
                            data = tail + chunk
                            found |= any(needle in data for needle in needles)
                            tail = data[-overlap:] if overlap else b""
                        leaks += int(found)
                except OSError:
                    unreadable += 1
    return {"files": files, "leakingFiles": leaks, "unreadable": unreadable,
            "passed": files > 0 and leaks == 0 and unreadable == 0}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--needles", type=Path, required=True)
    args = parser.parse_args()
    try:
        result = scan(args.root, args.needles)
    except (OSError, ValueError):
        result = {"passed": False, "status": "scanUnavailable"}
    print(json.dumps(result))
    raise SystemExit(0 if result["passed"] else 1)
