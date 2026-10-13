import Foundation
import LocalClipCore

@MainActor
final class Scenario {
    let name: String
    let rules: [String]
    let initialSettings: SettingsSnapshot
    let clock = FixtureClock()
    let fault = FixtureFault()
    let store: HistoryStore
    var ids: [String: ClipID] = [:]
    var categories: [String: String] = [:]
    var steps: [[String: Any]] = []

    init(_ name: String, rules: [String], root: URL, quota: Int64 = 500 * MiB, retention: Int = 30) throws {
        self.name = name; self.rules = rules
        var settings = SettingsSnapshot(); settings.quotaBytes = quota; settings.retentionDays = retention
        initialSettings = settings
        store = try HistoryStore(location: root.appendingPathComponent(name), clock: clock, faults: fault, initialSettings: settings)
    }

    func state() async throws -> [String: Any] {
        let page = try await store.search(SearchRequest(pageSize: 50))
        let alias = Dictionary(uniqueKeysWithValues: ids.map { ($0.value, $0.key) })
        // Equal timestamps sort by generated UUID; represent the tie as a set and retain strict category ordering separately.
        var groups: [[String]] = [], last: Instant?
        for item in page.items {
            guard let key = alias[item.id] else { throw ExportError.unexpectedResult }
            if last == item.lastCopiedAt { groups[groups.count - 1].append(key) } else { groups.append([key]); last = item.lastCopiedAt }
        }
        groups = groups.map { $0.sorted() }
        var categoryOrder: [String: [String]] = [:]
        for (key, id) in categories {
            categoryOrder[key] = try await store.search(SearchRequest(categoryIDs: [id])).items.map { alias[$0.id]! }
        }
        let settings = try await store.settings()
        return ["orderGroups": groups, "tieBreak": "actual-id-ascending", "items": page.items.map { item in
            ["id": alias[item.id]!, "copyCount": item.copyCount, "firstCopiedAt": item.firstCopiedAt,
             "lastCopiedAt": item.lastCopiedAt, "favorite": item.isFavorite, "protected": item.isProtected] as [String: Any]
        }.sorted { ($0["id"] as! String) < ($1["id"] as! String) },
                "protectedIDs": page.items.filter(\.isProtected).map { alias[$0.id]! }.sorted(), "categoryOrder": categoryOrder,
                "retentionDays": settings.retentionDays, "quotaBytes": settings.quotaBytes, "settingsRevision": settings.revision,
                "undoAvailable": await store.undoAvailable(), "canCapture": try await store.canCapture()]
    }

    func record(_ operation: [String: Any], outcome: [String: Any] = [:]) async throws {
        var expected = try await state()
        for (key, value) in outcome { expected[key] = value }
        steps.append(["operation": operation, "nowMs": clock.now(), "expected": expected])
    }

    func ingest(_ key: String, text: String? = nil, bytes: Int? = nil, byte: UInt8 = 65, expectError: StoreError? = nil) async throws {
        let payload: ClipboardPayload
        var operation: [String: Any] = ["kind": "ingest", "id": key]
        if let bytes {
            payload = ClipboardPayload([PasteboardItemValue([Representation(SupportedType.png, Data(repeating: byte, count: bytes))])])
            operation["generatedPayload"] = ["format": "image/png", "byte": Int(byte), "byteCount": bytes]
        } else { payload = textPayload([text ?? key]); operation["text"] = text ?? key }
        if let expectError { operation["expectError"] = expectError.rawValue }
        var outcome: [String: Any] = [:]
        do {
            let settings = try await store.settings()
            let result = try await store.ingest(ClipboardSnapshot(payload: payload, at: clock.now(), revision: settings.revision), at: clock.now())
            guard expectError == nil else { throw ExportError.unexpectedResult }
            switch result {
            case .inserted(let id): ids[key] = id; outcome["result"] = "inserted"
            case .deduplicated(let id): guard ids[key] == id else { throw ExportError.unexpectedResult }; outcome["result"] = "deduplicated"
            case .skipped(let reason): outcome["skipReason"] = reason.rawValue
            }
        } catch let error as StoreError {
            guard error == expectError else { throw error }
            outcome["error"] = error.rawValue
        }
        try await record(operation, outcome: outcome)
    }

    func advance(_ milliseconds: Instant) async throws {
        clock.advance(milliseconds)
        try await record(["kind": "advanceClock", "milliseconds": milliseconds])
    }

    func change(_ operation: [String: Any], _ change: HistoryChange, confirm: Bool = true) async throws {
        let preview = try await store.preview(change)
        var op = operation; op["confirm"] = confirm
        var outcome: [String: Any] = ["preview": ["affectedCount": preview.affectedCount, "needsConfirmation": preview.requiresConfirmation]]
        if confirm { outcome["removed"] = try await store.apply(preview.token).removed }
        try await record(op, outcome: outcome)
    }

    func favorite(_ key: String, _ value: Bool, confirm: Bool = true) async throws {
        try await change(["kind": "favorite", "id": key, "value": value], .favorite(ids[key]!, value), confirm: confirm)
    }

    func category(_ key: String) async throws {
        categories[key] = try await store.createCategory(name: key, colorHex: "34C759").id
        try await record(["kind": "createCategory", "id": key, "name": key, "colorHex": "34C759"])
    }

    func assign(_ keys: [String], to category: String) async throws {
        try await change(["kind": "assign", "ids": keys, "categoryID": category, "included": true], .category(keys.map { ids[$0]! }, categories[category]!, true))
    }

    func touch(_ keys: [String]) async throws {
        try await store.touch(ids: keys.map { ids[$0]! }, at: clock.now())
        try await record(["kind": "touch", "ids": keys])
    }

    func expire() async throws {
        let result = try await store.expire(at: clock.now())
        try await record(["kind": "expire"], outcome: ["removed": result.removed])
    }

    func delete(_ keys: [String]) async throws {
        try await change(["kind": "delete", "ids": keys], .delete(keys.map { ids[$0]! }))
    }

    func undo(expectError: StoreError? = nil) async throws {
        var outcome: [String: Any] = [:]
        do {
            try await store.undoDelete()
            guard expectError == nil else { throw ExportError.unexpectedResult }
        } catch let error as StoreError {
            guard error == expectError else { throw error }; outcome["error"] = error.rawValue
        }
        try await record(["kind": "undo"], outcome: outcome)
    }

    var document: [String: Any] {
        ["name": name, "rules": rules, "initialSettings": ["quotaBytes": initialSettings.quotaBytes, "retentionDays": initialSettings.retentionDays],
         "initialNowMs": fixedNow, "steps": steps]
    }
}

@MainActor
func exportStore(_ writer: Writer, root: URL) async throws {
    var cases: [[String: Any]] = []
    let dedup = try Scenario("dedup-protection", rules: ["AC03", "AC12", "AC19", "5.6-batch-touch"], root: root)
    try await dedup.ingest("a"); try await dedup.advance(1000); try await dedup.ingest("b")
    try await dedup.advance(1000); try await dedup.ingest("c")
    try await dedup.category("work"); try await dedup.assign(["a", "b", "c"], to: "work")
    try await dedup.favorite("a", true)
    try await dedup.advance(1000); try await dedup.ingest("a")
    try await dedup.advance(1000); try await dedup.touch(["b", "a", "b"])
    try await dedup.ingest("ordinary"); try await dedup.advance(30 * dayMS); try await dedup.expire()
    cases.append(dedup.document)

    let retention = try Scenario("retention-boundaries", rules: ["AC19", "AC20", "AC27"], root: root)
    try await retention.ingest("refresh"); try await retention.ingest("favorite"); try await retention.favorite("favorite", true)
    try await retention.advance(dayMS); try await retention.ingest("expires")
    try await retention.advance(28 * dayMS); try await retention.ingest("refresh")
    try await retention.advance(2 * dayMS)
    try await retention.favorite("favorite", false, confirm: false)
    try await retention.favorite("favorite", false)
    try await retention.change(["kind": "setRetention", "days": 1], .retention(1), confirm: false)
    try await retention.change(["kind": "setRetention", "days": 1], .retention(1))
    try await retention.change(["kind": "setRetention", "days": 0], .retention(0))
    try await retention.ingest("forever"); try await retention.advance(366 * dayMS); try await retention.expire()
    cases.append(retention.document)

    let undo = try Scenario("undo-boundary", rules: ["5.6-undo"], root: root)
    try await undo.ingest("a"); try await undo.delete(["a"]); try await undo.advance(29_999); try await undo.undo()
    try await undo.delete(["a"]); try await undo.advance(30_000); try await undo.undo(expectError: .notFound)
    cases.append(undo.document)

    let quota = try Scenario("quota-reject-preserves-old", rules: ["AC13", "AC17"], root: root, quota: 512 * 1024)
    try await quota.ingest("protected", bytes: 300 * 1024, byte: 1); try await quota.favorite("protected", true)
    try await quota.advance(1000); try await quota.ingest("ordinary", bytes: 64 * 1024, byte: 2)
    try await quota.ingest("rejected", bytes: 220 * 1024, byte: 3, expectError: .quotaExceeded)
    cases.append(quota.document)

    let rollback = try Scenario("failed-write-rolls-back-eviction", rules: ["AC26"], root: root, quota: 512 * 1024)
    try await rollback.ingest("old", bytes: 260 * 1024, byte: 4)
    rollback.fault.armed.withLock { $0 = true }
    try await rollback.record(["kind": "fault", "point": "candidateWritten", "enabled": true])
    try await rollback.ingest("new", bytes: 260 * 1024, byte: 5, expectError: .ioFailure)
    rollback.fault.armed.withLock { $0 = false }
    try await rollback.record(["kind": "fault", "point": "candidateWritten", "enabled": false])
    try await rollback.ingest("new", bytes: 260 * 1024, byte: 5)
    cases.append(rollback.document)

    let lower = try Scenario("lower-quota-preview-confirm", rules: ["AC12", "AC27"], root: root)
    for index in 0..<3 {
        try await lower.ingest("image\(index)", bytes: 40 * Int(MiB), byte: UInt8(10 + index))
        if index < 2 { try await lower.favorite("image\(index)", true) }
        try await lower.advance(1000)
    }
    try await lower.change(["kind": "setQuota", "bytes": 100 * MiB], .quota(100 * MiB), confirm: false)
    try await lower.change(["kind": "setQuota", "bytes": 100 * MiB], .quota(100 * MiB))
    cases.append(lower.document)

    let protected = try Scenario("protected-over-quota", rules: ["AC13", "AC27"], root: root)
    for index in 0..<3 {
        try await protected.ingest("protected\(index)", bytes: 40 * Int(MiB), byte: UInt8(20 + index))
        try await protected.favorite("protected\(index)", true)
        try await protected.advance(1000)
    }
    try await protected.change(["kind": "setQuota", "bytes": 100 * MiB], .quota(100 * MiB))
    try await protected.ingest("blocked", expectError: .quotaExceeded)
    cases.append(protected.document)
    try writer.write("store-scenarios", cases, sources: ["HistoryStore.ingest", "HistoryStore.search", "HistoryStore.touch", "HistoryStore.preview", "HistoryStore.apply", "HistoryStore.expire", "HistoryStore.undoDelete", "HistoryStore.canCapture", "ClipClock.now", "StorageFaultInjector.check"],
                     notes: ["Only synthetic databases; generatedPayload is repeated-byte test data, not a decoded image.",
                             "AC17 uses a 512 KiB injected quota to test rejection and preservation, not the exact 500 MiB acceptance fixture.",
                             "IDs are fixture aliases; orderGroups preserve timestamp groups with actual UUID ascending as tie-break. Category order is exact.",
                             "macOS quotaExceeded maps to Windows skipped(quotaFull); candidateWritten injection occurs after eviction and before commit.",
                             "Reference c097591 already uses in-place DELETE-journal transactions; Windows §4.5 still requires WAL."])
}
