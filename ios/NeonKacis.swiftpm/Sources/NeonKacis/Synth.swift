import AVFoundation

/// Ses dosyası taşımamak için tüm efektler açılışta PCM tamponuna sentezlenir.
/// Motor açılamazsa oyun sessiz çalışır — ses hiçbir zaman oynanışı engellemez.
final class Synth {

    enum Effect: String, CaseIterable {
        case orb, diamond, pass, shieldUp, shieldBreak, crash, start, tick
    }

    static let shared = Synth()

    var enabled = true

    private let engine = AVAudioEngine()
    private var players: [AVAudioPlayerNode] = []
    private var buffers: [Effect: AVAudioPCMBuffer] = [:]
    private var nextPlayer = 0
    private var ready = false

    private let sampleRate: Double = 44_100
    private lazy var format = AVAudioFormat(standardFormatWithSampleRate: sampleRate, channels: 1)

    private init() {}

    func start() {
        guard !ready, let format else { return }
        do {
            try AVAudioSession.sharedInstance().setCategory(.ambient, options: [.mixWithOthers])
            try AVAudioSession.sharedInstance().setActive(true)
        } catch {
            // Ses oturumu açılmazsa sessiz devam et.
        }

        for _ in 0..<6 {
            let p = AVAudioPlayerNode()
            engine.attach(p)
            engine.connect(p, to: engine.mainMixerNode, format: format)
            players.append(p)
        }

        for effect in Effect.allCases {
            buffers[effect] = render(effect, format: format)
        }

        do {
            try engine.start()
            players.forEach { $0.play() }
            ready = true
        } catch {
            ready = false
        }
    }

    func play(_ effect: Effect) {
        guard enabled, ready, let buffer = buffers[effect], !players.isEmpty else { return }
        let player = players[nextPlayer]
        nextPlayer = (nextPlayer + 1) % players.count
        player.scheduleBuffer(buffer, at: nil, options: .interrupts, completionHandler: nil)
    }

    // MARK: - Sentez

    /// Basit bir zarf + dalga biçimi. Her efekt tek seferde üretilip saklanır.
    private func render(_ effect: Effect, format: AVAudioFormat) -> AVAudioPCMBuffer? {
        let spec = Self.spec(for: effect)
        let frames = AVAudioFrameCount(spec.duration * sampleRate)
        guard frames > 0,
              let buffer = AVAudioPCMBuffer(pcmFormat: format, frameCapacity: frames),
              let channel = buffer.floatChannelData?[0] else { return nil }
        buffer.frameLength = frames

        var phase: Double = 0
        let n = Double(frames)

        for i in 0..<Int(frames) {
            let t = Double(i) / n                               // 0 → 1
            let freq = spec.from + (spec.to - spec.from) * t
            phase += 2 * Double.pi * freq / sampleRate
            if phase > 2 * Double.pi { phase -= 2 * Double.pi }

            let wave: Double
            switch spec.shape {
            case .sine:     wave = sin(phase)
            case .square:   wave = sin(phase) >= 0 ? 1 : -1
            case .triangle: wave = 2 * abs(2 * (phase / (2 * Double.pi)) - 1) - 1
            case .saw:      wave = 2 * (phase / (2 * Double.pi)) - 1
            }

            // Hızlı atak, üstel sönüm — kısa ve tok bir arcade tınısı.
            let attack = min(1, t / 0.02)
            let decay = exp(-t * spec.decay)
            channel[i] = Float(wave * attack * decay * spec.gain)
        }
        return buffer
    }

    private enum Shape { case sine, square, triangle, saw }

    private struct Spec {
        var from: Double
        var to: Double
        var duration: Double
        var shape: Shape
        var gain: Double
        var decay: Double
    }

    private static func spec(for effect: Effect) -> Spec {
        switch effect {
        case .orb:
            return Spec(from: 620, to: 880, duration: 0.10, shape: .triangle, gain: 0.22, decay: 6)
        case .diamond:
            return Spec(from: 720, to: 1320, duration: 0.20, shape: .triangle, gain: 0.28, decay: 4)
        case .pass:
            return Spec(from: 220, to: 180, duration: 0.07, shape: .sine, gain: 0.12, decay: 9)
        case .shieldUp:
            return Spec(from: 300, to: 900, duration: 0.26, shape: .saw, gain: 0.22, decay: 4)
        case .shieldBreak:
            return Spec(from: 260, to: 70, duration: 0.28, shape: .saw, gain: 0.30, decay: 5)
        case .crash:
            return Spec(from: 340, to: 45, duration: 0.60, shape: .saw, gain: 0.34, decay: 3)
        case .start:
            return Spec(from: 420, to: 700, duration: 0.16, shape: .square, gain: 0.20, decay: 6)
        case .tick:
            return Spec(from: 880, to: 880, duration: 0.07, shape: .square, gain: 0.18, decay: 10)
        }
    }
}
