import Foundation
import SwiftUI

/// Sahne ile SwiftUI arayüzü arasındaki tek köprü.
/// Sahne her karede değil, yalnızca görünen bir değer değiştiğinde buraya yazar —
/// aksi halde SwiftUI saniyede 120 kez yeniden çizilirdi.
final class GameModel: ObservableObject {

    enum Mode: Equatable {
        case menu
        case countdown(Int)
        case playing
        case paused
        case over
    }

    @Published var mode: Mode = .menu
    @Published var score: Int = 0
    @Published var multiplier: Double = 1
    @Published var wallsPassed: Int = 0
    @Published var orbsTaken: Int = 0
    @Published var elapsed: TimeInterval = 0
    @Published var isNewBest: Bool = false
    @Published var primaryColor: Color = Palette.all[0].primary.color
    @Published var secondaryColor: Color = Palette.all[0].secondary.color

    @Published private(set) var best: Int
    @Published private(set) var recentRuns: [Int]
    @Published private(set) var totalRuns: Int

    @Published var soundOn: Bool {
        didSet { Store.soundOn = soundOn }
    }
    @Published var hapticsOn: Bool {
        didSet { Store.hapticsOn = hapticsOn }
    }

    init() {
        best = Store.best
        recentRuns = Store.recentRuns
        totalRuns = Store.totalRuns
        soundOn = Store.soundOn
        hapticsOn = Store.hapticsOn
    }

    var isPlaying: Bool {
        if case .playing = mode { return true }
        return false
    }

    var isCountingDown: Bool {
        if case .countdown = mode { return true }
        return false
    }

    /// Koşu bittiğinde çağrılır; rekoru ve son koşuları kalıcılaştırır.
    func finishRun() {
        let final = score
        isNewBest = final > best
        if isNewBest { best = final; Store.best = final }
        recentRuns = ([final] + recentRuns).prefix(5).map { $0 }
        totalRuns += 1
        Store.recentRuns = recentRuns
        Store.totalRuns = totalRuns
        mode = .over
    }

    func resetForNewRun() {
        score = 0
        multiplier = 1
        wallsPassed = 0
        orbsTaken = 0
        elapsed = 0
        isNewBest = false
    }
}

/// UserDefaults sarmalayıcı — oyun tamamen çevrimdışı, veri telefondan çıkmaz.
enum Store {
    private static let d = UserDefaults.standard

    static var best: Int {
        get { d.integer(forKey: "nk.best") }
        set { d.set(newValue, forKey: "nk.best") }
    }
    static var recentRuns: [Int] {
        get { d.array(forKey: "nk.runs") as? [Int] ?? [] }
        set { d.set(newValue, forKey: "nk.runs") }
    }
    static var totalRuns: Int {
        get { d.integer(forKey: "nk.totalRuns") }
        set { d.set(newValue, forKey: "nk.totalRuns") }
    }
    static var soundOn: Bool {
        get { d.object(forKey: "nk.sound") as? Bool ?? true }
        set { d.set(newValue, forKey: "nk.sound") }
    }
    static var hapticsOn: Bool {
        get { d.object(forKey: "nk.haptics") as? Bool ?? true }
        set { d.set(newValue, forKey: "nk.haptics") }
    }
}
