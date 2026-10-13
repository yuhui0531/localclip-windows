import Foundation
import LocalClipCore

@MainActor
func exportQueries(_ writer: Writer, root: URL) async throws {
    let store = try HistoryStore(location: root.appendingPathComponent("queries"), clock: FixtureClock())
    let work = try await store.createCategory(name: "工作", colorHex: "34C759")
    let personal = try await store.createCategory(name: "Personal", colorHex: "123456")
    let categories = [work, personal]
    let aliases = [work.id: "category-work", personal.id: "category-personal"]
    let sources = [SourceAttribution(bundleID: "fixture.notepad", name: "Notepad"),
                   SourceAttribution(bundleID: "fixture.code", name: "Code"),
                   SourceAttribution(bundleID: "fixture.code-insiders", name: "Code Insiders"), .unknown]
    let inputs = ["", "需求", "type:image", "app:notepad 需求", "in:工作 after:7d", "\"type:url\"", "app:不存在",
                  "type:text", "type:link", "type:rich", "type:richtext", "type:img", "type:file", "type:mixed",
                  "type:文本", "type:链接", "type:富文本", "type:图片", "type:文件", "type:混合",
                  "is:fav", "is:favorite", "is:favorites", "is:收藏", "is:bad", "TYPE:IMAGE X", "type:", "type::url", "prefix:type:url",
                  "app:Code", "app:co", "app:fixture.notepad", "app:未知", "in:personal", "in:missing", "in:工作 in:Personal",
                  "after:2026-10-14", "before:2026-10-14", "after:2024-02-29", "after:2023-02-29", "after:0d before:7d",
                  "after:9999d", "after:10000d", "before:-1d", "after:bad", " a  type:url  b ", "a\ttype:url\tb", "\"app:notepad\" literal",
                  "type:text type:image", "app:co app:notepad", "in:工作 in:工作", "https://example.com % _ * \"", "a\napp:notepad\nb"]
    let rows: [[String: Any]] = inputs.map { text in
        let parsed = SearchQueryParser.parse(text, sources: sources, categories: categories, now: fixedNow)
        var expected: [String: Any] = ["literal": parsed.literal, "favorites": parsed.favorites,
                                     "categoryIDs": parsed.categoryIDs.map { aliases[$0]! }.sorted(), "labels": parsed.labels, "problems": parsed.problems]
        expected["kind"] = parsed.kind?.rawValue; expected["sourceAppID"] = parsed.sourceBundleID
        expected["from"] = parsed.from; expected["until"] = parsed.until
        return ["text": text, "sources": sources.map { ["sourceAppID": $0.bundleID as Any? ?? NSNull(), "name": $0.name] },
                "categories": categories.map { ["id": aliases[$0.id]!, "name": $0.name] },
                "nowMs": fixedNow, "localOffsetSeconds": 28800, "expected": expected]
    }
    try writer.write("query", rows, sources: ["SearchQueryParser.parse", "HistoryStore.createCategory"],
                     notes: ["Random category IDs are replaced consistently by fixture aliases; parser receives actual public-store categories."])
}

@MainActor
func exportRanks(_ writer: Writer, root: URL) async throws {
    let corpora: [([(texts: [String], source: String, title: String?)], [String])] = [
        ([( ["needle"], "Exact", nil), (["needle prefix"], "Prefix", nil), (["a needle b"], "Contains", nil),
          (["other"], "NEEDLE source", nil), (["no match"], "Other", nil), (["title body"], "Other", "My NEEDLE title")], ["needle", "", "no-match"]),
        ([( ["e\u{301}"], "Source", nil), (["é suffix"], "Source", nil), (["other"], "É", nil)], ["é", "É", "e\u{301}"]),
        ([( ["first", "second"], "Source", nil), (["first second"], "Source", nil), (["second"], "Source", nil)], ["first\nsecond", "second", "\n"]),
        ([( ["a\0b"], "Source", nil), (["%_\"*"], "Source", nil), (["需求ABC"], "Source", nil), (["ＡBC"], "Source", nil)], ["\0", "%", "_", "\"", "*", "需求", "abc", "Ａ"])
    ]
    var rows: [[String: Any]] = []
    for (corpusIndex, fixture) in corpora.enumerated() {
        let clock = FixtureClock()
        let store = try HistoryStore(location: root.appendingPathComponent("rank-\(corpusIndex)"), clock: clock)
        var indices: [ClipID: Int] = [:]
        for (index, entry) in fixture.0.enumerated() {
            let result = try await store.ingest(ClipboardSnapshot(payload: textPayload(entry.texts), at: clock.now(), source: .init(bundleID: nil, name: entry.source)), at: clock.now())
            guard case .inserted(let id) = result else { throw ExportError.unexpectedResult }
            indices[id] = index
            if let title = entry.title { try await store.renameClip(id: id, title: title) }
            clock.advance(1000)
        }
        for needle in fixture.1 {
            let page = try await store.search(SearchRequest(needle))
            rows.append(["corpus": fixture.0.enumerated().map { index, entry in
                var row: [String: Any] = ["text": entry.texts.joined(separator: "\n"), "items": entry.texts, "source": entry.source, "copiedAt": fixedNow + Int64(index) * 1000]
                row["title"] = entry.title
                return row
            }, "needle": needle, "expectedOrder": try page.items.map { summary -> Int in
                guard let index = indices[summary.id] else { throw ExportError.unexpectedResult }; return index
            }])
        }
    }
    try writer.write("rank", rows, sources: ["HistoryStore.ingest", "HistoryStore.renameClip", "HistoryStore.search"],
                     notes: ["Distinct timestamps avoid random-ID tie ordering; expectedOrder indexes corpus. Empty-query browse semantics come from the public search API."])
}
