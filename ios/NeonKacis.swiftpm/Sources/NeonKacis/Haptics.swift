import UIKit

/// Dokunsal geri bildirim — web sürümünde hiç olmayan, native'in somut kazancı.
/// iOS Safari `navigator.vibrate` desteklemez; burada Taptic Engine doğrudan kullanılır.
enum Haptics {
    private static let light = UIImpactFeedbackGenerator(style: .light)
    private static let medium = UIImpactFeedbackGenerator(style: .medium)
    private static let heavy = UIImpactFeedbackGenerator(style: .heavy)
    private static let notify = UINotificationFeedbackGenerator()

    static var enabled = true

    /// Jeneratörleri önceden hazırlamak ilk darbedeki gecikmeyi kaldırır.
    static func prepare() {
        light.prepare(); medium.prepare(); heavy.prepare(); notify.prepare()
    }

    static func orb() {
        guard enabled else { return }
        light.impactOccurred(intensity: 0.6)
        light.prepare()
    }

    static func diamond() {
        guard enabled else { return }
        medium.impactOccurred(intensity: 0.9)
        medium.prepare()
    }

    static func shieldGained() {
        guard enabled else { return }
        notify.notificationOccurred(.success)
        notify.prepare()
    }

    static func shieldBroken() {
        guard enabled else { return }
        heavy.impactOccurred(intensity: 1.0)
        heavy.prepare()
    }

    static func crash() {
        guard enabled else { return }
        notify.notificationOccurred(.error)
        notify.prepare()
    }

    static func tick() {
        guard enabled else { return }
        light.impactOccurred(intensity: 0.35)
        light.prepare()
    }
}
