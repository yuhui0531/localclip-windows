import Foundation
import LocalClipCore

@MainActor
func exportPure(_ writer: Writer) throws {
    let strings = ["", "HELLO world", "需求", "ＡＢＣ１２３", "e\u{301}", "ÉÉé", "Straße", "İΣσ", "👨‍👩‍👧‍👦👍🏽", "A\0B", "\r\n", "%_\"*", "ÅA\u{30a}", "中文ABC", "路径/Name.TXT"]
    try writer.write("normalize", strings.map { ["input": $0, "normalized": SearchNormalizer.normalize($0)] }, sources: ["SearchNormalizer.normalize"])
    let contexts: [(String, String)] = strings.map { ($0, $0) } + [
        (String(repeating: "前", count: 50) + "NEEDLE" + String(repeating: "后", count: 100), "needle"),
        (String(repeating: "👨‍👩‍👧‍👦", count: 40) + "e\u{301}" + String(repeating: "👍🏽", count: 90), "é"),
        (String(repeating: "a", count: 200), "missing"), (String(repeating: "b", count: 200), ""),
        ("abc\0def", "\0d"), ("one\r\ntwo", "two"), ("ＡBC", "a")]
    try writer.write("context", contexts.map { ["original": $0.0, "literal": $0.1, "context": SearchNormalizer.context($0.0, literal: $0.1)] }, sources: ["SearchNormalizer.context"])
    let html = ["", "<b>Hello</b><br>World", "<p>一</p><div>二</div>", "<script>secret</script>safe", "<STYLE>x</STYLE><head>y</head>z",
                "<!-- comment -->text", "&lt;&gt;&quot;&apos;&nbsp;&amp;", "&#65;&#x1F600;&#0;", "&#x110000;&#99999999;", "&amp;lt;", "<a href='https://example.com'>link</a>", "<img src='https://example.com/test.png'>alt", "<p>one\r\ntwo</p>", "<broken"]
    try writer.write("html", html.map { ["html": $0, "text": HTMLText.extract($0)] }, sources: ["HTMLText.extract"])
    let links = ["", " ", "https://example.com", "http://example.com/a?b=1#c", "HTTPS://EXAMPLE.COM", "example.com", "www.example.org/a", "example.net:8080/path",
                 "localhost", "localhost:3000", "http://localhost:3000", "127.0.0.1", "http://127.0.0.1:8000", "https://[::1]/", "192.168.1.1:8080/a",
                 "README.md", "readme.txt", "v0.3.22", "1.2.3", "see example.com", "example.com other.org", "example.com\nnext", "example.com\tpath",
                 " https://example.com\r\n", "ftp://example.com", "file:///tmp/test", "mailto:a@example.com", "a@example.com", "https://", "http:///path",
                 "example.invalid", "example.co.uk", "sub.example.dev/path", "example.io", "example.ai", "example.app", "example.xyz", "例子.中国", "https://例子.中国/a",
                 "https://example.com/a%20b", "https://example.com/a b", "https://example.com:443/a", "https://user:pass@example.com", "https://example.com?x=1&y=2",
                 "example.com.", "-bad.com", "https://example.com/😀", "https://example.com/" + String(repeating: "a", count: 8192),
                 "https://example.com/" + String(repeating: "b", count: 8172), "example.com\0x"]
    try writer.write("weblink", links.map { text in
        var row: [String: Any] = ["text": text]
        if let url = ClipboardPayload.webLink(text) { row["url"] = url.absoluteString }
        return row
    }, sources: ["ClipboardPayload.webLink"])

    let representations: [[Representation]] = [[], [.init(SupportedType.string, Data("text".utf8))], [.init(SupportedType.url, Data("https://example.com".utf8))],
        [.init(SupportedType.html, Data("<b>x</b>".utf8))], [.init(SupportedType.rtf, Data("{\\rtf1 x}".utf8))],
        [.init(SupportedType.png, Data([0, 1, 255]))], [.init(SupportedType.jpeg, Data([255, 216, 255]))],
        [.init(SupportedType.png, Data([1])), .init(SupportedType.jpeg, Data([2]))],
        [.init(SupportedType.string, Data("x".utf8)), .init(SupportedType.png, Data([3]))],
        [.init(SupportedType.fileURL, Data("file:///C:/fixtures/a.png".utf8))]]
    var payloads = representations.map { ClipboardPayload([PasteboardItemValue($0)]) }
    payloads += [textPayload(["example.com"]), textPayload(["README.md"]), textPayload(["a", "b"]),
                 ClipboardPayload([PasteboardItemValue(representations[9]), PasteboardItemValue(representations[9])]),
                 ClipboardPayload([PasteboardItemValue(representations[9]), PasteboardItemValue(representations[1])])]
    try writer.write("kind", try payloads.map { payload in ["types": try mappedItems(payload).map { $0.map { $0["type"]! } }, "items": try mappedItems(payload), "kind": payload.kind.rawValue] }, sources: ["ClipboardPayload.kind"], notes: ["TIFF has no Windows mapping; DIBV5 is covered by Windows-specific tests, never mislabeled TIFF."])
    var fingerprintPayloads = payloads + [textPayload(["A\0B", "e\u{301}"]), textPayload(["b", "a"]), textPayload([""]), ClipboardPayload([])]
    fingerprintPayloads += [ClipboardPayload([PasteboardItemValue(representations[8].reversed())])]
    var fingerprints: [[String: Any]] = try fingerprintPayloads.map { original in
        let items = try mappedItems(original)
        let neutral = ClipboardPayload(items.map { PasteboardItemValue($0.map { Representation($0["type"]!, Data(base64Encoded: $0["dataBase64"]!)!) }) })
        return ["items": items, "fingerprint": neutral.fingerprint]
    }
    for types in [["image/x-win-dibv5"], ["image/png", "image/jpeg", "image/x-win-dibv5"]] {
        let items = types.enumerated().map { ["type": $0.element, "dataBase64": Data([UInt8($0.offset), 0, 255]).base64EncodedString()] }
        let neutral = ClipboardPayload([PasteboardItemValue(items.map { Representation($0["type"]!, Data(base64Encoded: $0["dataBase64"]!)!) })])
        fingerprints.append(["items": [items], "fingerprint": neutral.fingerprint])
    }
    try writer.write("fingerprint", fingerprints, sources: ["ClipboardPayload.canonicalData", "ClipboardPayload.fingerprint"], notes: ["Public canonicalData/fingerprint APIs receive neutral IDs directly; kind is evaluated on original UTIs.", "Image bytes are synthetic identity inputs, not decoder fixtures; DIBV5 is never produced by converting TIFF."])
    let extensions = ["png", "jpg", "jpeg", "gif", "bmp", "tif", "tiff", "webp", "ico", "svg", "heic", "pdf", "txt", "", "PNG", "JpEg"]
    let paths = extensions.map { "C:/fixtures/中文 image." + $0 } + ["C:/fixtures/no-extension", "C:/fixtures/a.png.txt", "C:/fixtures/.png", "C:\\fixtures\\a.png"]
    let allowed: Set<String> = ["png", "jpg", "jpeg", "gif", "bmp", "tif", "tiff", "webp", "ico"]
    try writer.write("imagefile", paths.map { path in
        let mac = ClipboardPayload.isImageFile(path)
        let expected = allowed.contains((path as NSString).pathExtension.lowercased())
        return ["path": path, "isImage": expected, "macosIsImage": mac, "macosDiffers": mac != expected]
    }, sources: ["ClipboardPayload.isImageFile", "Windows PRD §4.3 extension whitelist"])
    try exportTransforms(writer)
    try exportInspection(writer)
    let versions = [("v0.3.29-51", "0.3.28"), ("v0.3.29-52", "0.3.29-51"), ("0.3.28", "0.3.29"), ("1.0", "1.0.0"), ("1.10", "1.9"),
                    ("V2.0+build", "1.99"), ("vv1.2", "1.1"), ("", "0"), ("1.foo.1", "1.0.0"), ("1..2", "1.1"), ("01.02", "1.2"), ("2-beta", "1.9")]
    try writer.write("version", versions.map { ["candidate": $0.0, "current": $0.1, "newer": AppVersion.isNewer($0.0, than: $0.1)] }, sources: ["AppVersion.isNewer"])
    try exportPolicy(writer)
}

@MainActor
func exportTransforms(_ writer: Writer) throws {
    let jwt = "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJmaXh0dXJlIn0.signature"
    let samples: [TextTransform: [String]] = [
        .markdownCodeBlock: ["let x = 1\r\n", "```code```", "<html>Hi</html>"], .dedent: ["  a\r\n  b\r\n", "\ta\n\tb", "a\nb"],
        .trimWhitespace: [" \r\n a  \r\n b\t\r\n", "  a  ", "\n\n"], .stripPrompt: ["$ echo a\r\n% pwd\r\n", "  $ hi\nplain", "plain"],
        .stripLineNumbers: ["1 a\r\n2 b\r\n", "10 | a\n11 | b", "2 a\n1 b"], .formatJSON: ["{\r\n\"z\":1.00e+2,\"a\":[]\r\n}", "[true,null,\"a\\nb\"]", "{bad}"],
        .minifyJSON: ["{\r\n \"b\": 2, \"a\": 1\r\n}", " \"a b\" ", "[1,]"], .quoteString: ["a\r\nb", "\"\\\t\0", "你好😀"],
        .unquoteString: ["\"a\\r\\nb\"", "\"\\u4f60\\u597d\"", "'bad'"], .base64Encode: ["a\r\nb", "你好😀", "\0"],
        .base64Decode: ["SGVsbG8=", "5L2g5aW9", "AA==", "%%%%", "SGVs\r\nbG8="], .urlEncode: ["a\r\nb", "你好 &?=/+", "-._~!"],
        .urlDecode: ["a%0D%0Ab", "a+b%20c", "%GG", "%E4%BD%A0"], .camelCase: ["HTTPServer_name\r\nnext-ID", "  hello-world", "中文_名字"],
        .snakeCase: ["HTTPServer\r\nmyURL", "some-ID2Name", "  hello world"], .kebabCase: ["helloWorld\r\nHTTPServer", "  one_two", "中文名字"],
        .uppercase: ["a\r\nb", "Straße", "你好"], .lowercase: ["A\r\nB", "İΣ", "你好"],
        .sortLines: ["item10\r\nitem2\r\nitem1\r\n", "b\na\n", "one"], .uniqueLines: ["a\r\nb\r\na\r\n", "é\ne\u{301}\n", "one"],
        .jwtPayload: [jwt, "eyJhbGciOiJub25lIn0.eyJhIjoxfQ.", "bad"], .timestampToISO: ["1700000000", "1700000000123", "123"],
        .isoToTimestamp: ["2023-11-14T22:13:20Z", "2023-11-14T22:13:20.123Z", "not-date"],
        .colorToRGB: ["#abc", "#12345680", "rgb(999,0,0)"], .colorToHex: ["rgb(1, 2, 3)", "hsla(120, 100%, 50%, .5)", "not-color"]]
    var rows: [[String: Any]] = []
    for transform in TextTransform.allCases {
        for input in samples[transform, default: []] + [""] {
            var row: [String: Any] = ["transform": transform.rawValue, "input": input, "applicable": transform.isApplicable(input)]
            do { row["output"] = try transform.apply(input) }
            catch let error as TransformError { row["error"] = error.message }
            rows.append(row)
        }
    }
    try writer.write("transform", rows, sources: ["TextTransform.isApplicable", "TextTransform.apply"], notes: ["apply is called even when applicable=false to preserve both public API behaviors; menu consumers must honor applicability."])
    let values = [[" one ", "two\r\nnext", "O'Brien"], [], [""], [" a\r\n", " b\n"], ["你好", "\"\\\t"]]
    try writer.write("join", JoinStyle.allCases.flatMap { style in values.map { ["style": style.rawValue, "values": $0, "output": style.join($0)] } }, sources: ["JoinStyle.join"], notes: ["These are pure-text joins; original-format newline composition requires separate rich-text integration tests."])
}

@MainActor
func exportInspection(_ writer: Writer) throws {
    let texts = ["", "plain", "1700000000", "1700000000123", "123", "550e8400-e29b-41d4-a716-446655440000", "550e8400-e29b-01d4-a716-446655440000",
                 "#abc", "#12345680", "rgb(1, 2, 3)", "hsl(120, 100%, 50%)", "SGVsbG8gd29ybGQh", "0123456789abcdef", "{\"a\":1}", "[1,2]", "{bad}",
                 "https://example.com/a?b=1", "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJmaXh0dXJlIn0.signature", "# Title\n- a\n- b", "line1\r\nline2"]
    var rows: [[String: Any]] = texts.map { text in
        var row: [String: Any] = ["operation": "inspect", "text": text, "truncated": false, "nowMs": fixedNow]
        if let badge = ContentInspector.inspect(text, truncated: false)?.badge { row["badge"] = badge }
        if let full = ContentInspector.inspectFull(text, language: nil, now: Date(timeIntervalSince1970: Double(fixedNow) / 1000)) {
            row["kind"] = full.kind
            row["rows"] = full.rows.map { ["label": $0.label, "value": $0.value, "block": $0.block] as [String: Any] }
            row["actions"] = full.actions.map { ["title": $0.title, "transform": $0.transform.rawValue] }
            if let note = full.note { row["note"] = note }
        }
        return row
    }
    let exports: [(String, String?, [String], String?)] = [
        ("let x = 1", "swift", [], nil), ("# Title\n- a\n- b", nil, [], nil), ("rich", nil, [SupportedType.rtf, SupportedType.html], nil),
        ("html", nil, [SupportedType.html], nil), ("plain", nil, [], nil), ("let x = 1", "swift", [SupportedType.html], "<pre>code</pre>"),
        ("let x = 1", "swift", [SupportedType.rtf], nil), ("<html>Hi</html>", "xml", [], nil)]
    rows += exports.map { text, language, types, markup in
        let format = ContentInspector.exportFormat(plainText: text, language: language, types: types, markup: markup)
        var row: [String: Any] = ["operation": "exportFormat", "text": text, "types": types.map { neutralTypes[$0]! }, "exportFormat": format.fileExtension]
        row["language"] = language; row["markup"] = markup
        switch format { case .plain: row["mode"] = "plain"; case .original(let type, _): row["mode"] = "original"; row["originalType"] = neutralTypes[type]! }
        return row
    }
    for title in ["title", "../a:b\n", "", "...", " 你好 ", String(repeating: "😀", count: 70), "back\\slash*?<>|", "CON"] {
        var row: [String: Any] = ["operation": "fileName", "text": title, "copiedAt": fixedNow, "exportBaseName": ContentInspector.exportBaseName(title: title, copiedAt: fixedNow)]
        row["sanitizedFileName"] = ContentInspector.sanitizedFileName(title)
        rows.append(row)
    }
    rows.append(["operation": "exportBaseName", "text": "", "copiedAt": fixedNow, "exportBaseName": ContentInspector.exportBaseName(title: nil, copiedAt: fixedNow)])
    try writer.write("inspect", rows, sources: ["ContentInspector.inspect", "ContentInspector.inspectFull", "ContentInspector.exportFormat", "ContentInspector.sanitizedFileName", "ContentInspector.exportBaseName"], notes: ["Timestamp badge uses the reference API's current-year behavior; full inspection uses fixed nowMs.", "Filename observations preserve macOS behavior, including Windows-invalid characters and reserved names; Windows policy requires product resolution before save_as implementation.", "Windows .url and DIBV5 export have no equivalent public ContentInspector API; test in Windows-specific cases."])
}

@MainActor
func exportPolicy(_ writer: Writer) throws {
    var rows: [[String: Any]] = []
    for enabled in [true, false] {
        for basis in ["declared", "foreground", "unknown"] {
            for marker in ["", "org.nspasteboard.ConcealedType", "org.nspasteboard.TransientType"] {
                var settings = SettingsSnapshot(); settings.captureEnabled = enabled; settings.excludedBundleIDs = ["fixture.excluded"]
                let source = SourceAttribution(bundleID: basis == "unknown" ? nil : "fixture.excluded", name: "Fixture", basis: basis)
                let types = [[SupportedType.string], marker.isEmpty ? [SupportedType.png] : [SupportedType.png, marker]]
                var row: [String: Any] = ["settings": ["captureEnabled": enabled, "excludedAppIDs": ["fixture.excluded"]],
                    "source": ["name": "Fixture", "basis": basis == "declared" ? "owner" : basis, "sourceAppID": source.bundleID as Any? ?? NSNull()],
                    "types": types.map { $0.map { neutralTypes[$0] ?? $0 } }]
                row["skipReason"] = settings.rejection(source: source, types: types)?.rawValue
                rows.append(row)
            }
        }
    }
    try writer.write("policy", rows, sources: ["SettingsSnapshot.rejection"], notes: ["declared maps to Windows owner for exclusion semantics. Sensitive marker inputs retain macOS names; Windows four-format filtering must be tested at the Win32 adapter before any body read."])
}
