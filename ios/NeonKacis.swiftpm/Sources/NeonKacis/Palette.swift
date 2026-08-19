import UIKit
import SwiftUI

/// Oyunun renk dünyası. Faz ilerledikçe paletler arasında yumuşak geçiş yapılır,
/// böylece oyuncu ne kadar ilerlediğini renkten de okur.
struct Palette {
    let primary: SIMD3<Double>     // ana neon (gemi, kenarlar, skor)
    let secondary: SIMD3<Double>   // gövde dolgusu, ızgara
    let name: String

    static let all: [Palette] = [
        Palette(primary: [0, 229, 255],   secondary: [0, 90, 255],   name: "MAVİ"),
        Palette(primary: [255, 45, 149],  secondary: [122, 0, 255],  name: "MOR"),
        Palette(primary: [57, 255, 20],   secondary: [0, 168, 160],  name: "YEŞİL"),
        Palette(primary: [255, 212, 0],   secondary: [255, 96, 0],   name: "ALTIN"),
        Palette(primary: [255, 70, 60],   secondary: [255, 0, 140],  name: "KIRMIZI")
    ]

    /// Her `phaseLength` mesafede bir sonraki palete, son %14'lük dilimde harmanlanarak geçer.
    static func at(distance: CGFloat, phaseLength: CGFloat) -> Palette {
        guard phaseLength > 0 else { return all[0] }
        let f = Double(distance / phaseLength)
        let i = Int(f.rounded(.down)) % all.count
        let j = (i + 1) % all.count
        let frac = f - f.rounded(.down)
        let t = min(max((frac - 0.86) / 0.14, 0), 1)
        return Palette(
            primary: mix(all[i].primary, all[j].primary, t),
            secondary: mix(all[i].secondary, all[j].secondary, t),
            name: all[i].name
        )
    }

    private static func mix(_ a: SIMD3<Double>, _ b: SIMD3<Double>, _ t: Double) -> SIMD3<Double> {
        a + (b - a) * t
    }
}

extension SIMD3 where Scalar == Double {
    var uiColor: UIColor {
        UIColor(red: x / 255, green: y / 255, blue: z / 255, alpha: 1)
    }
    var color: Color {
        Color(red: x / 255, green: y / 255, blue: z / 255)
    }
    func uiColor(alpha: CGFloat) -> UIColor {
        UIColor(red: x / 255, green: y / 255, blue: z / 255, alpha: alpha)
    }
}
