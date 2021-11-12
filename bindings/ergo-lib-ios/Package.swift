// swift-tools-version:5.2

import PackageDescription

let package = Package(
    name: "ErgoLib",
    products: [
        .library(
            name: "ErgoLib",
            targets: ["ErgoLib"]
        )
    ],
    dependencies:  [
        .package(url: "https://github.com/SwiftyJSON/SwiftyJSON.git", from: "5.0.0"),
    ],
    targets: [
        .systemLibrary(name: "ErgoLibC"),
        .target(
            name: "ErgoLib",
            dependencies: ["ErgoLibC"]
        ),
        .testTarget(name: "ErgoLibTests", dependencies: ["ErgoLib"])
    ]
)
