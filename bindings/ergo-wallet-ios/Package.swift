// swift-tools-version:5.2

import PackageDescription

let package = Package(
    name: "ErgoWallet",
    products: [
        .library(
            name: "ErgoWallet",
            targets: ["ErgoWallet"]
        )
    ],
    targets: [
        .systemLibrary(name: "ErgoWalletC"),
        .target(
            name: "ErgoWallet",
            dependencies: ["ErgoWalletC"]
        ),
    ]
)
