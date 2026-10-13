#!/usr/bin/env python3
"""Validate fixture contracts independently, without importing macOS implementation code."""
import base64
from collections import Counter
import hashlib
import json
from pathlib import Path
import struct
import sys

FILES = {
    "normalize", "rank", "context", "kind", "fingerprint", "html", "policy",
    "store-scenarios", "weblink", "imagefile", "query", "transform", "join", "inspect", "version",
}
TYPES = {"text/plain", "text/uri", "text/html", "text/rtf", "image/png", "image/jpeg", "image/x-win-dibv5", "text/uri-list"}


def validate(root):
    assert {p.stem for p in root.glob("*.json")} == FILES, "Expected exactly 15 fixture files"
    documents = {name: json.loads((root / f"{name}.json").read_text()) for name in sorted(FILES)}
    for name, document in documents.items():
        assert next(iter(document)) == "metadata", "Provenance must be the file header"
        metadata = document["metadata"]
        assert metadata["macosCommit"] == "c097591431c00982ad34186068064b40965c95fd"
        assert metadata["macosVersion"] == "0.3.29"
        assert metadata["timezone"] == "Asia/Shanghai"
        assert metadata["sourceFunctions"]
        assert document["cases"], name
    assert len(documents["weblink"]["cases"]) >= 40
    image_cases = {c["path"]: c for c in documents["imagefile"]["cases"]}
    assert image_cases["C:/fixtures/中文 image.png"]["macosIsImage"] is True
    assert image_cases["C:/fixtures/中文 image.jpg"]["macosIsImage"] is True
    transformations = Counter(c["transform"] for c in documents["transform"]["cases"])
    assert len(transformations) == 25 and min(transformations.values()) >= 3
    assert any("\r\n" in c["input"] for c in documents["transform"]["cases"])
    assert any(c.get("error") for c in documents["transform"]["cases"])
    assert len({c["style"] for c in documents["join"]["cases"]}) == 7
    rules = {r for c in documents["store-scenarios"]["cases"] for r in c["rules"]}
    assert {"AC03", "AC12", "AC13", "AC17", "AC19", "AC20", "AC26", "AC27", "5.6-batch-touch"} <= rules
    for case in documents["store-scenarios"]["cases"]:
        assert case["steps"] and all("expected" in step for step in case["steps"])
    scenarios = {case["name"]: case["steps"] for case in documents["store-scenarios"]["cases"]}
    dedup = scenarios["dedup-protection"]
    assert any(s["expected"].get("result") == "deduplicated" for s in dedup)
    touch = next(s for s in dedup if s["operation"]["kind"] == "touch")
    assert touch["expected"]["categoryOrder"]["work"] == ["b", "a", "c"]
    assert dedup[-1]["expected"]["protectedIDs"] == ["a", "b", "c"]
    assert dedup[-1]["expected"]["removed"] == 1
    retention = scenarios["retention-boundaries"]
    assert any(s["operation"].get("id") == "refresh" and s["expected"].get("result") == "deduplicated" for s in retention)
    unprotect = [s for s in retention if s["operation"]["kind"] == "favorite" and s["operation"]["value"] is False]
    assert all(s["expected"]["preview"] == {"affectedCount": 1, "needsConfirmation": True} for s in unprotect)
    assert "favorite" in unprotect[0]["expected"]["protectedIDs"]
    assert "favorite" not in unprotect[1]["expected"]["protectedIDs"]
    rejected = scenarios["quota-reject-preserves-old"][-1]["expected"]
    assert rejected["error"] == "quotaExceeded" and rejected["orderGroups"] == [["ordinary"], ["protected"]]
    rollback = scenarios["failed-write-rolls-back-eviction"]
    assert rollback[2]["expected"]["error"] == "ioFailure" and rollback[2]["expected"]["orderGroups"] == [["old"]]
    assert rollback[-1]["expected"]["orderGroups"] == [["new"]]
    lower = scenarios["lower-quota-preview-confirm"]
    assert lower[-2]["expected"]["quotaBytes"] == 500 * 1048576
    assert lower[-2]["expected"]["preview"]["affectedCount"] == 1
    assert lower[-1]["expected"]["removed"] == 1 and lower[-1]["expected"]["quotaBytes"] == 100 * 1048576
    protected = scenarios["protected-over-quota"][-1]["expected"]
    assert protected["canCapture"] is False and len(protected["protectedIDs"]) == 3
    undo = scenarios["undo-boundary"]
    assert undo[3]["expected"]["orderGroups"] == [["a"]]
    assert undo[-1]["expected"]["error"] == "notFound"
    all_types = set()
    for case in documents["fingerprint"]["cases"]:
        integer = lambda value: struct.pack(">Q", value)
        canonical = integer(1) + integer(len(case["items"]))
        for item in case["items"]:
            canonical += integer(len(item))
            for representation in sorted(item, key=lambda r: r["type"].encode()):
                assert representation["type"] in TYPES
                all_types.add(representation["type"])
                kind = representation["type"].encode()
                data = base64.b64decode(representation["dataBase64"], validate=True)
                canonical += integer(len(kind)) + kind + integer(len(data)) + data
        assert hashlib.sha256(canonical).hexdigest() == case["fingerprint"]
    assert all_types == TYPES, "Every neutral format must participate in fingerprints"
    for name, document in documents.items():
        print(f"{name}.json: {len(document['cases'])} cases")
    print("PASS: 15 files; provenance, minimum coverage, neutral fingerprints, required storage invariants")


if __name__ == "__main__":
    validate(Path(sys.argv[1] if len(sys.argv) > 1 else "test-vectors"))
