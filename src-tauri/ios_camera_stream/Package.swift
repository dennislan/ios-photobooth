// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "ios_camera_stream",
    products: [
        .executable(name: "ios_camera_stream", targets: ["ios_camera_stream"])
    ],
    dependencies: [],
    targets: [
        .executableTarget(
            name: "ios_camera_stream",
            path: "Sources"
        )
    ]
)
