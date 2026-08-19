// swift-tools-version: 5.9

// App Playground biçimi: bu paket hem Swift Playgrounds'ta (iPad) hem Xcode'da
// hem de xcodebuild ile doğrudan açılıp derlenebilir. Tek kaynak, üç araç.

import PackageDescription
import AppleProductTypes

let package = Package(
    name: "NeonKacis",
    platforms: [
        .iOS("16.0")
    ],
    products: [
        .iOSApplication(
            name: "NeonKacis",
            targets: ["NeonKacis"],
            bundleIdentifier: "dev.arda.neonkacis",
            teamIdentifier: "",
            displayVersion: "1.0",
            bundleVersion: "1",
            accentColor: .presetColor(.cyan),
            supportedDeviceFamilies: [.phone],
            supportedInterfaceOrientations: [.portrait],
            appCategory: .games
        )
    ],
    targets: [
        .executableTarget(
            name: "NeonKacis",
            path: "Sources/NeonKacis"
        )
    ]
)
