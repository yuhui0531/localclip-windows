import Foundation
import LocalClipCore
import Synchronization

let baseline = "c097591431c00982ad34186068064b40965c95fd"
let fixedNow: Instant = 1_760_486_400_000
let neutralTypes = [SupportedType.string: "text/plain", SupportedType.url: "text/uri",
                    SupportedType.html: "text/html", SupportedType.rtf: "text/rtf",
                    SupportedType.png: "image/png", SupportedType.jpeg: "image/jpeg",
                    SupportedType.fileURL: "text/uri-list"]

final class FixtureClock: ClipClock, Sendable {
    let value = Mutex<Instant>(fixedNow)
    func now() -> Instant { value.withLock { $0 } }
    func advance(_ milliseconds: Instant) { value.withLock { $0 += milliseconds } }
}

final class FixtureFault: StorageFaultInjector, Sendable {
    let armed = Mutex(false)
    func check(_ point: FaultPoint) throws {
        if point == .candidateWritten, armed.withLock({ $0 }) { throw StoreError.ioFailure }
    }
}

enum ExportError: Error { case invalidEnvironment, unsafeOutput, unexpectedResult, referenceChanged, imageTypeServicesUnavailable }

func git(_ path: String, _ arguments: [String]) throws -> String {
    let process = Process(), pipe = Pipe()
    process.executableURL = URL(fileURLWithPath: "/usr/bin/git")
    process.arguments = ["-C", path] + arguments
    process.environment = ProcessInfo.processInfo.environment.merging(["GIT_OPTIONAL_LOCKS": "0"]) { _, new in new }
    process.standardOutput = pipe
    try process.run()
    let data = pipe.fileHandleForReading.readDataToEndOfFile()
    process.waitUntilExit()
    guard process.terminationStatus == 0 else { throw ExportError.invalidEnvironment }
    return String(decoding: data, as: UTF8.self).trimmingCharacters(in: .whitespacesAndNewlines)
}

func textPayload(_ values: [String]) -> ClipboardPayload {
    ClipboardPayload(values.map { PasteboardItemValue([Representation(SupportedType.string, Data($0.utf8))], plainText: $0) })
}

func mappedItems(_ payload: ClipboardPayload) throws -> [[[String: String]]] {
    try payload.items.map { item in
        try item.representations.map { representation in
            guard let type = neutralTypes[representation.type] else { throw ExportError.unexpectedResult }
            return ["type": type, "dataBase64": representation.data.base64EncodedString()]
        }
    }
}

@MainActor
struct Writer {
    let output: URL
    let metadata: [String: Any]
    func write(_ name: String, _ cases: [[String: Any]], sources: [String], notes: [String] = []) throws {
        var header = metadata
        header["sourceFunctions"] = sources
        header["notes"] = notes
        let options: JSONSerialization.WritingOptions = [.prettyPrinted, .sortedKeys, .withoutEscapingSlashes]
        let headerData = try JSONSerialization.data(withJSONObject: header, options: options)
        let casesData = try JSONSerialization.data(withJSONObject: cases, options: options)
        let data = Data("{\n  \"metadata\": ".utf8) + headerData + Data(",\n  \"cases\": ".utf8) + casesData + Data("\n}\n".utf8)
        try data.write(to: output.appendingPathComponent(name + ".json"), options: .atomic)
        print("\(name).json: \(cases.count) cases")
    }
}

@main
struct VectorExport {
    @MainActor static func main() async throws {
        let repository = URL(fileURLWithPath: #filePath).deletingLastPathComponent().deletingLastPathComponent()
            .deletingLastPathComponent().deletingLastPathComponent().resolvingSymlinksInPath()
        let reference = URL(fileURLWithPath: ProcessInfo.processInfo.environment["LOCALCLIP_MAC_PATH"]
                            ?? repository.deletingLastPathComponent().appendingPathComponent("rea/macos-app").path).resolvingSymlinksInPath()
        guard CommandLine.arguments.count == 2, ProcessInfo.processInfo.environment["TZ"] == "Asia/Shanghai",
              try git(reference.path, ["rev-parse", "HEAD"]) == baseline,
              try git(reference.path, ["status", "--porcelain"]).isEmpty else { throw ExportError.invalidEnvironment }
        // A denied LaunchServices lookup returns false even for PNG; never export that as a behavioral difference.
        guard ClipboardPayload.isImageFile("fixture.png"), ClipboardPayload.isImageFile("fixture.jpg") else {
            throw ExportError.imageTypeServicesUnavailable
        }
        let output = URL(fileURLWithPath: CommandLine.arguments[1], relativeTo: URL(fileURLWithPath: FileManager.default.currentDirectoryPath)).standardizedFileURL.resolvingSymlinksInPath()
        guard output.path.hasPrefix(repository.path + "/"), output != repository,
              !output.path.hasPrefix(reference.deletingLastPathComponent().path + "/") else { throw ExportError.unsafeOutput }
        let versionData = try Data(contentsOf: reference.appendingPathComponent("Resources/Info.plist"))
        let plist = try PropertyListSerialization.propertyList(from: versionData, format: nil) as? [String: Any]
        guard let version = plist?["CFBundleShortVersionString"] as? String, version == "0.3.29" else { throw ExportError.invalidEnvironment }
        try FileManager.default.createDirectory(at: output, withIntermediateDirectories: true)
        let temporary = repository.appendingPathComponent("vector-export/.build/fixtures-" + UUID().uuidString)
        try FileManager.default.createDirectory(at: temporary, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: temporary) }
        let writer = Writer(output: output, metadata: ["macosCommit": baseline, "macosVersion": version,
                           "timezone": "Asia/Shanghai", "localOffsetSeconds": 28800, "nowMs": fixedNow, "schemaVersion": 1])
        try exportPure(writer)
        try await exportQueries(writer, root: temporary)
        try await exportRanks(writer, root: temporary)
        try await exportStore(writer, root: temporary)
        guard try git(reference.path, ["status", "--porcelain"]).isEmpty,
              try git(reference.path, ["rev-parse", "HEAD"]) == baseline else { throw ExportError.referenceChanged }
        print("PASS: reference clean before and after; 15 files exported")
    }
}
