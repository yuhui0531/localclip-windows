// swift-tools-version: 6.0
import PackageDescription
import Foundation

let repository = URL(fileURLWithPath: #filePath).deletingLastPathComponent().deletingLastPathComponent()
let reference = ProcessInfo.processInfo.environment["LOCALCLIP_MAC_PATH"]
    ?? repository.deletingLastPathComponent().appendingPathComponent("rea/macos-app").path
let package = Package(
    name: "VectorExport",
    platforms: [.macOS(.v15)],
    dependencies: [.package(name: "LocalClip", path: reference)],
    targets: [.executableTarget(name: "VectorExport", dependencies: [.product(name: "LocalClipCore", package: "LocalClip")])]
)
