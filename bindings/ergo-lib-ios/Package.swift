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
    targets: [
        .systemLibrary(name: "ErgoLibC"),
        .target(
            name: "ErgoLib",
            dependencies: ["ErgoLibC"]
        ),
        .testTarget(name: "AddressTests", dependencies: ["ErgoLib"])
    ]
)
