import SpriteKit
import UIKit

/// Oyunun tamamı. Simülasyon sabit adımlı (1/120 s) — yüksek hızda duvarların
/// içinden geçme (tünelleme) olmaz. Çizim ekranın tazeleme hızında (ProMotion'da 120 Hz).
final class GameScene: SKScene {

    // MARK: - Denge değerleri (hepsi ekran boyutuna oranlı, her cihazda aynı hissi verir)

    private enum Tuning {
        static let difficultyRamp: CGFloat = 95      // ekran boyu katı — zorluğun tavana çıkışı
        static let speedMin: CGFloat = 0.55          // ekran boyu / sn
        static let speedMax: CGFloat = 1.42
        static let gapWide: CGFloat = 0.44           // ekran eni katı
        static let gapNarrow: CGFloat = 0.205
        static let spawnFar: CGFloat = 0.60          // ekran boyu katı
        static let spawnNear: CGFloat = 0.345
        static let phaseLength: CGFloat = 20         // ekran boyu katı
        static let multDecayDelay: TimeInterval = 2.6
        static let multDecayRate: Double = 0.55
        static let fixedStep: TimeInterval = 1.0 / 120.0
        static let steerSensitivity: CGFloat = 1.35
        static let collisionForgiveness: CGFloat = 0.86
    }

    // MARK: - Dış bağlantı

    weak var model: GameModel?

    // MARK: - Katmanlar

    private let gridLayer = SKNode()
    private let wallLayer = SKNode()
    private let orbLayer = SKNode()
    private let fxLayer = SKNode()
    private let playerNode = SKShapeNode()
    private let shieldRing = SKShapeNode()

    // MARK: - Durum

    private enum Mode { case idle, countdown, playing, paused, over }
    private var mode: Mode = .idle

    private var lastUpdate: TimeInterval = 0
    private var accumulator: TimeInterval = 0
    private var countdownRemaining: TimeInterval = 0
    private var lastCountdownShown = -1

    private var distance: CGFloat = 0
    private var elapsed: TimeInterval = 0
    private var spawnAccumulator: CGFloat = 0
    private var scoreValue: Double = 0
    private var lastScoreShown = -1
    private var multiplier: Double = 1
    private var lastOrbTime: TimeInterval = 0
    private var wallsPassed = 0
    private var orbsTaken = 0

    private var playerX: CGFloat = 0
    private var targetX: CGFloat = 0
    private var playerRadius: CGFloat = 12
    private var playerY: CGFloat = 0
    private var hasShield = false
    private var invulnerable: TimeInterval = 0

    private var dragStartTouchX: CGFloat?
    private var dragStartPlayerX: CGFloat = 0
    fileprivate var shakeAmount: CGFloat = 0

    // MARK: - Varlıklar

    private struct Wall {
        var node: SKNode
        var y: CGFloat
        /// Ekran koordinatında açık aralıklar (yatay kayma uygulanmadan).
        var holes: [ClosedRange<CGFloat>]
        var driftAmplitude: CGFloat
        var driftSpeed: CGFloat
        var driftPhase: CGFloat
        var offsetX: CGFloat
        var passed: Bool
    }

    private enum OrbKind { case normal, diamond, shield }

    private struct Orb {
        var node: SKNode
        var position: CGPoint
        var radius: CGFloat
        var kind: OrbKind
    }

    private var walls: [Wall] = []
    private var orbs: [Orb] = []

    private var wallHeight: CGFloat = 20
    private var gridSpacing: CGFloat = 100

    private lazy var dotTexture: SKTexture = Self.makeDotTexture()

    // MARK: - Kurulum

    override func didMove(to view: SKView) {
        anchorPoint = .zero
        scaleMode = .resizeFill
        backgroundColor = UIColor(red: 5 / 255, green: 6 / 255, blue: 15 / 255, alpha: 1)

        [gridLayer, wallLayer, orbLayer, fxLayer].forEach { addChild($0) }
        fxLayer.zPosition = 5

        playerNode.zPosition = 10
        playerNode.lineWidth = 2.4
        playerNode.fillColor = UIColor(white: 0.96, alpha: 1)
        addChild(playerNode)

        shieldRing.zPosition = 9
        shieldRing.lineWidth = 2.4
        shieldRing.fillColor = .clear
        shieldRing.strokeColor = UIColor(red: 150 / 255, green: 220 / 255, blue: 255 / 255, alpha: 1)
        shieldRing.glowWidth = 5
        shieldRing.isHidden = true
        addChild(shieldRing)

        layout()
        buildGrid()
        Haptics.prepare()
        Synth.shared.start()
    }

    override func didChangeSize(_ oldSize: CGSize) {
        super.didChangeSize(oldSize)
        guard size.width > 0, size.height > 0 else { return }
        layout()
        buildGrid()
    }

    private func layout() {
        playerRadius = size.width * 0.031
        playerY = size.height * 0.20
        wallHeight = max(14, size.height * 0.032)
        gridSpacing = size.height * 0.13
        if playerX == 0 { playerX = size.width / 2; targetX = playerX }

        let r = playerRadius
        let path = CGMutablePath()
        path.move(to: CGPoint(x: 0, y: r * 1.5))
        path.addLine(to: CGPoint(x: r * 1.05, y: -r * 1.15))
        path.addLine(to: CGPoint(x: 0, y: -r * 0.5))
        path.addLine(to: CGPoint(x: -r * 1.05, y: -r * 1.15))
        path.closeSubpath()
        playerNode.path = path
        playerNode.glowWidth = r * 0.5

        shieldRing.path = CGPath(ellipseIn: CGRect(x: -r * 2.05, y: -r * 2.05,
                                                   width: r * 4.1, height: r * 4.1),
                                 transform: nil)
    }

    private func buildGrid() {
        gridLayer.removeAllChildren()
        let count = Int(size.height / max(gridSpacing, 1)) + 3
        for i in 0..<count {
            let line = SKShapeNode(rectOf: CGSize(width: size.width, height: 1))
            line.strokeColor = .clear
            line.fillColor = .white
            line.name = "h\(i)"
            gridLayer.addChild(line)
        }
        for i in 1..<8 {
            let line = SKShapeNode(rectOf: CGSize(width: 1, height: size.height))
            line.strokeColor = .clear
            line.fillColor = .white
            line.alpha = 0.05
            line.position = CGPoint(x: size.width * CGFloat(i) / 8, y: size.height / 2)
            line.name = "v"
            gridLayer.addChild(line)
        }
    }

    // MARK: - Dışarıdan kontrol

    func startNewRun() {
        walls.forEach { $0.node.removeFromParent() }
        orbs.forEach { $0.node.removeFromParent() }
        walls.removeAll()
        orbs.removeAll()
        fxLayer.removeAllChildren()

        distance = 0
        elapsed = 0
        scoreValue = 0
        lastScoreShown = -1
        multiplier = 1
        lastOrbTime = 0
        wallsPassed = 0
        orbsTaken = 0
        hasShield = false
        invulnerable = 0
        shieldRing.isHidden = true
        playerX = size.width / 2
        targetX = playerX
        dragStartTouchX = nil
        // İlk duvar oyuncu yerleşene kadar gelmesin.
        spawnAccumulator = -size.height * 0.55

        model?.resetForNewRun()
        beginCountdown()
        Synth.shared.play(.start)
    }

    func pauseGame() {
        guard mode == .playing || mode == .countdown else { return }
        mode = .paused
        model?.mode = .paused
    }

    func resumeGame() {
        guard mode == .paused else { return }
        beginCountdown()
    }

    func abandonRun() {
        guard mode == .paused else { return }
        finish()
    }

    func returnToMenu() {
        mode = .idle
        model?.mode = .menu
    }

    private func beginCountdown() {
        mode = .countdown
        countdownRemaining = 3
        lastCountdownShown = -1
        lastUpdate = 0
        accumulator = 0
        model?.mode = .countdown(3)
    }

    // MARK: - Ana döngü

    override func update(_ currentTime: TimeInterval) {
        if lastUpdate == 0 { lastUpdate = currentTime }
        var delta = currentTime - lastUpdate
        lastUpdate = currentTime
        if delta > 0.25 { delta = 0.25 }

        switch mode {
        case .countdown:
            countdownRemaining -= delta
            let shown = max(0, Int(countdownRemaining.rounded(.up)))
            if shown != lastCountdownShown {
                lastCountdownShown = shown
                if shown > 0 {
                    model?.mode = .countdown(shown)
                    Synth.shared.play(.tick)
                    Haptics.tick()
                }
            }
            if countdownRemaining <= 0 {
                mode = .playing
                model?.mode = .playing
                accumulator = 0
            }

        case .playing:
            accumulator += delta
            var guardCount = 0
            while accumulator >= Tuning.fixedStep && guardCount < 40 {
                accumulator -= Tuning.fixedStep
                guardCount += 1
                step(Tuning.fixedStep)
                if mode != .playing { accumulator = 0; break }
            }
            publish()

        case .idle, .paused, .over:
            break
        }

        paint(delta)
    }

    // MARK: - Simülasyon

    private var difficulty: CGFloat {
        min(1, distance / (size.height * Tuning.difficultyRamp))
    }

    private func smoothstep(_ t: CGFloat) -> CGFloat { t * t * (3 - 2 * t) }

    private func step(_ dt: TimeInterval) {
        elapsed += dt
        let d = difficulty
        let e = smoothstep(d)

        let speed = size.height * (Tuning.speedMin + (Tuning.speedMax - Tuning.speedMin) * e)
        let dy = speed * CGFloat(dt)
        distance += dy

        // Gemi hedefe süzülür.
        playerX += (targetX - playerX) * min(1, CGFloat(dt) * 16)
        playerX = min(max(playerX, playerRadius), size.width - playerRadius)
        if invulnerable > 0 { invulnerable -= dt }

        // Duvarlar
        var index = walls.count - 1
        while index >= 0 {
            walls[index].y -= dy

            if walls[index].driftAmplitude > 0 {
                let raw = sin(CGFloat(elapsed) * walls[index].driftSpeed + walls[index].driftPhase)
                    * walls[index].driftAmplitude
                let low = walls[index].holes.map(\.lowerBound).min() ?? 0
                let high = walls[index].holes.map(\.upperBound).max() ?? size.width
                let minOffset = -low + size.width * 0.015
                let maxOffset = size.width - high - size.width * 0.015
                walls[index].offsetX = min(max(raw, minOffset), max(minOffset, maxOffset))
            }

            if !walls[index].passed && walls[index].y + wallHeight < playerY - playerRadius {
                walls[index].passed = true
                wallsPassed += 1
                Synth.shared.play(.pass)
            }

            if invulnerable <= 0 && collides(with: walls[index]) {
                takeHit()
                if mode != .playing { return }
            }

            if walls[index].y + wallHeight < -wallHeight {
                walls[index].node.removeFromParent()
                walls.remove(at: index)
            }
            index -= 1
        }

        // Küreler
        var orbIndex = orbs.count - 1
        while orbIndex >= 0 {
            orbs[orbIndex].position.y -= dy
            let orb = orbs[orbIndex]
            let dx = orb.position.x - playerX
            let dyy = orb.position.y - playerY
            let reach = orb.radius + playerRadius
            if dx * dx + dyy * dyy < reach * reach {
                collect(orb)
                orb.node.removeFromParent()
                orbs.remove(at: orbIndex)
            } else if orb.position.y < -orb.radius * 3 {
                orb.node.removeFromParent()
                orbs.remove(at: orbIndex)
            }
            orbIndex -= 1
        }

        // Üretim
        spawnAccumulator += dy
        let spawnGap = size.height * (Tuning.spawnFar + (Tuning.spawnNear - Tuning.spawnFar) * e)
        if spawnAccumulator >= spawnGap {
            spawnAccumulator -= spawnGap
            spawnWall(difficulty: d)
            spawnOrbs(above: size.height + wallHeight + spawnGap * 0.5)
        }

        // Skor ve çarpan
        scoreValue += Double(dy) * 0.05 * multiplier
        if elapsed - lastOrbTime > Tuning.multDecayDelay {
            multiplier = max(1, multiplier - Tuning.multDecayRate * dt)
        }
    }

    // MARK: - Çarpışma

    /// Duvarın dolu parçaları — çizimde kullanılan `segments(from:)` ile aynı kaynak,
    /// yalnızca yatay kayma uygulanmış hâli. Çarpışma ile görüntü ayrışamaz.
    private func solidSegments(of wall: Wall) -> [ClosedRange<CGFloat>] {
        segments(from: wall.holes.map {
            ($0.lowerBound + wall.offsetX)...($0.upperBound + wall.offsetX)
        })
    }

    private func collides(with wall: Wall) -> Bool {
        let r = playerRadius * Tuning.collisionForgiveness
        guard playerY + r > wall.y, playerY - r < wall.y + wallHeight else { return false }
        for segment in solidSegments(of: wall) {
            let nearestX = min(max(playerX, segment.lowerBound), segment.upperBound)
            let nearestY = min(max(playerY, wall.y), wall.y + wallHeight)
            let dx = playerX - nearestX
            let dy = playerY - nearestY
            if dx * dx + dy * dy < r * r { return true }
        }
        return false
    }

    private func takeHit() {
        if hasShield {
            hasShield = false
            shieldRing.isHidden = true
            invulnerable = 0.9
            multiplier = 1
            burst(at: CGPoint(x: playerX, y: playerY),
                  color: UIColor(red: 180 / 255, green: 230 / 255, blue: 255 / 255, alpha: 1),
                  count: 28)
            shake(intensity: size.width * 0.03)
            Synth.shared.play(.shieldBreak)
            Haptics.shieldBroken()
            return
        }
        finish()
    }

    private func finish() {
        mode = .over
        let palette = Palette.at(distance: distance, phaseLength: size.height * Tuning.phaseLength)
        burst(at: CGPoint(x: playerX, y: playerY), color: palette.primary.uiColor, count: 46)
        burst(at: CGPoint(x: playerX, y: playerY), color: .white, count: 18)
        shake(intensity: size.width * 0.045)
        Synth.shared.play(.crash)
        Haptics.crash()
        publish()
        model?.finishRun()
    }

    // MARK: - Dokunma: nispi sürükleme

    /// Parmak nerede olursa olsun, hareketin *farkı* gemiye uygulanır.
    /// Böylece parmak gemiyi kapatmaz — telefonda tek elle oynanabilir.
    override func touchesBegan(_ touches: Set<UITouch>, with event: UIEvent?) {
        guard mode == .playing || mode == .countdown, let touch = touches.first else { return }
        dragStartTouchX = touch.location(in: self).x
        dragStartPlayerX = playerX
    }

    override func touchesMoved(_ touches: Set<UITouch>, with event: UIEvent?) {
        guard mode == .playing || mode == .countdown,
              let touch = touches.first,
              let startX = dragStartTouchX else { return }
        let delta = (touch.location(in: self).x - startX) * Tuning.steerSensitivity
        targetX = min(max(dragStartPlayerX + delta, playerRadius), size.width - playerRadius)
    }

    override func touchesEnded(_ touches: Set<UITouch>, with event: UIEvent?) {
        dragStartTouchX = nil
    }

    override func touchesCancelled(_ touches: Set<UITouch>, with event: UIEvent?) {
        dragStartTouchX = nil
    }

    private func collect(_ orb: Orb) {
        switch orb.kind {
        case .shield:
            hasShield = true
            shieldRing.isHidden = false
            burst(at: orb.position,
                  color: UIColor(red: 160 / 255, green: 220 / 255, blue: 255 / 255, alpha: 1),
                  count: 22)
            Synth.shared.play(.shieldUp)
            Haptics.shieldGained()
        case .diamond:
            orbsTaken += 1
            multiplier = min(9.9, multiplier + 0.6)
            scoreValue += 130 * multiplier
            lastOrbTime = elapsed
            burst(at: orb.position,
                  color: UIColor(red: 1, green: 212 / 255, blue: 0, alpha: 1),
                  count: 20)
            Synth.shared.play(.diamond)
            Haptics.diamond()
        case .normal:
            orbsTaken += 1
            multiplier = min(9.9, multiplier + 0.22)
            scoreValue += 40 * multiplier
            lastOrbTime = elapsed
            let palette = Palette.at(distance: distance, phaseLength: size.height * Tuning.phaseLength)
            burst(at: orb.position, color: palette.primary.uiColor, count: 11)
            Synth.shared.play(.orb)
            Haptics.orb()
        }
    }
}

// MARK: - Üretim

private extension GameScene {

    /// Dolu parçaları verilen açık aralıklardan türetir. Kenarlar ekran dışına
    /// taşırılır ki duvar yatay kayarken yanlardan boşluk açılmasın.
    func segments(from holes: [ClosedRange<CGFloat>]) -> [ClosedRange<CGFloat>] {
        let sorted = holes.sorted { $0.lowerBound < $1.lowerBound }
        var result: [ClosedRange<CGFloat>] = []
        var cursor = -size.width
        for hole in sorted {
            if hole.lowerBound > cursor { result.append(cursor...hole.lowerBound) }
            cursor = max(cursor, hole.upperBound)
        }
        if cursor < size.width * 2 { result.append(cursor...(size.width * 2)) }
        return result
    }

    func spawnWall(difficulty d: CGFloat) {
        let e = smoothstep(d)
        let fraction = Tuning.gapWide + (Tuning.gapNarrow - Tuning.gapWide) * e

        var holes: [ClosedRange<CGFloat>]
        let wantsTwo = d > 0.42 && Double.random(in: 0...1) < 0.28

        if walls.isEmpty && wallsPassed == 0 {
            // İlk duvarın boşluğu ortada — açılış adil olsun.
            let width = size.width * fraction
            holes = [((size.width - width) / 2)...((size.width + width) / 2)]
        } else if wantsTwo {
            let width = size.width * fraction * 0.66
            let a = randomX(from: size.width * 0.05, to: size.width * 0.48 - width)
            let b = randomX(from: size.width * 0.52, to: size.width * 0.95 - width)
            holes = [a...(a + width), b...(b + width)]
        } else {
            let width = size.width * fraction
            let a = randomX(from: size.width * 0.035, to: size.width * 0.965 - width)
            holes = [a...(a + width)]
        }

        var amplitude: CGFloat = 0
        var driftSpeed: CGFloat = 0
        var driftPhase: CGFloat = 0
        if d > 0.16 && Double.random(in: 0...1) < 0.22 + Double(d) * 0.42 {
            amplitude = CGFloat.random(in: (size.width * 0.05)...(size.width * (0.055 + 0.13 * d)))
            driftSpeed = CGFloat.random(in: 0.7...(1.5 + d))
            driftPhase = CGFloat.random(in: 0...(.pi * 2))
        }

        let container = SKNode()
        container.zPosition = 3
        for segment in segments(from: holes) {
            let width = segment.upperBound - segment.lowerBound
            guard width > 0.5 else { continue }
            let bar = SKShapeNode(rect: CGRect(x: segment.lowerBound, y: 0,
                                               width: width, height: wallHeight),
                                  cornerRadius: 2)
            bar.lineWidth = 2
            bar.glowWidth = 3
            bar.name = "bar"
            container.addChild(bar)
        }
        wallLayer.addChild(container)

        walls.append(Wall(node: container,
                          y: size.height + wallHeight,
                          holes: holes,
                          driftAmplitude: amplitude,
                          driftSpeed: driftSpeed,
                          driftPhase: driftPhase,
                          offsetX: 0,
                          passed: false))
    }

    /// `Double.random(in:)` alt sınır üstü geçerse çöker; ayar değerleri
    /// değiştiğinde oyunun düşmemesi için aralığı burada güvenceye alıyoruz.
    func randomX(from lower: CGFloat, to upper: CGFloat) -> CGFloat {
        guard upper > lower else { return max(0, min(lower, size.width)) }
        return CGFloat.random(in: lower...upper)
    }

    func spawnOrbs(above baseY: CGFloat) {
        guard Double.random(in: 0...1) < 0.74 else { return }

        let wantsShield = !hasShield && distance > size.height * 8 && Double.random(in: 0...1) < 0.09
        let wantsDiamond = !wantsShield && Double.random(in: 0...1) < 0.16

        // Elmaslar kenarlarda doğar: almak için duvar boşluğundan sapmak gerekir.
        let centerX: CGFloat = wantsDiamond
            ? (Bool.random() ? CGFloat.random(in: (size.width * 0.07)...(size.width * 0.20))
                             : CGFloat.random(in: (size.width * 0.80)...(size.width * 0.93)))
            : CGFloat.random(in: (size.width * 0.14)...(size.width * 0.86))

        let count = wantsShield ? 1 : Int.random(in: 1...4)
        let wobble = CGFloat.random(in: (size.width * 0.04)...(size.width * 0.12))
            * (Bool.random() ? 1 : -1)

        for i in 0..<count {
            let kind: OrbKind = wantsShield ? .shield : (wantsDiamond ? .diamond : .normal)
            let x = min(max(centerX + sin(CGFloat(i) * 1.1) * wobble,
                            size.width * 0.06), size.width * 0.94)
            let y = baseY + CGFloat(i) * size.height * 0.052
            let radius = size.width * (kind == .shield ? 0.030 : 0.018)

            let node = SKNode()
            node.zPosition = 4
            let shape = SKShapeNode()
            shape.name = "orb"
            shape.lineWidth = 2
            shape.glowWidth = radius * 0.9

            switch kind {
            case .normal:
                shape.path = CGPath(ellipseIn: CGRect(x: -radius, y: -radius,
                                                      width: radius * 2, height: radius * 2),
                                    transform: nil)
                shape.lineWidth = 0
            case .shield:
                shape.path = CGPath(ellipseIn: CGRect(x: -radius, y: -radius,
                                                      width: radius * 2, height: radius * 2),
                                    transform: nil)
                shape.fillColor = .clear
                shape.lineWidth = 3
            case .diamond:
                // Elmas biçimi: ALTIN fazında bile normal küreden ayrılsın.
                let r = radius * 2
                let path = CGMutablePath()
                path.move(to: CGPoint(x: 0, y: r))
                path.addLine(to: CGPoint(x: r * 0.72, y: 0))
                path.addLine(to: CGPoint(x: 0, y: -r))
                path.addLine(to: CGPoint(x: -r * 0.72, y: 0))
                path.closeSubpath()
                shape.path = path
                shape.lineWidth = 2
                shape.run(.repeatForever(.rotate(byAngle: .pi * 2, duration: 3.6)))
            }

            node.addChild(shape)
            node.position = CGPoint(x: x, y: y)
            orbLayer.addChild(node)

            orbs.append(Orb(node: node, position: node.position, radius: radius, kind: kind))
            if wantsShield { break }
        }
    }

    func burst(at point: CGPoint, color: UIColor, count: Int) {
        let emitter = SKEmitterNode()
        emitter.particleTexture = dotTexture
        emitter.position = point
        emitter.zPosition = 6
        emitter.particleBirthRate = 6000
        emitter.numParticlesToEmit = count
        emitter.particleLifetime = 0.65
        emitter.particleLifetimeRange = 0.35
        emitter.emissionAngleRange = .pi * 2
        emitter.particleSpeed = size.width * 0.75
        emitter.particleSpeedRange = size.width * 0.6
        emitter.yAcceleration = -size.height * 0.55
        emitter.particleAlpha = 0.95
        emitter.particleAlphaSpeed = -1.5
        emitter.particleScale = size.width * 0.00035
        emitter.particleScaleRange = size.width * 0.0003
        emitter.particleScaleSpeed = -size.width * 0.0004
        emitter.particleColor = color
        emitter.particleColorBlendFactor = 1
        emitter.particleBlendMode = .add
        fxLayer.addChild(emitter)
        emitter.run(.sequence([.wait(forDuration: 1.2), .removeFromParent()]))
    }

    func shake(intensity: CGFloat) {
        shakeAmount = max(shakeAmount, intensity)
    }

    static func makeDotTexture() -> SKTexture {
        let side: CGFloat = 32
        let renderer = UIGraphicsImageRenderer(size: CGSize(width: side, height: side))
        let image = renderer.image { context in
            let cg = context.cgContext
            let colors = [UIColor.white.cgColor,
                          UIColor.white.withAlphaComponent(0).cgColor] as CFArray
            if let gradient = CGGradient(colorsSpace: CGColorSpaceCreateDeviceRGB(),
                                         colors: colors, locations: [0, 1]) {
                cg.drawRadialGradient(gradient,
                                      startCenter: CGPoint(x: side / 2, y: side / 2), startRadius: 0,
                                      endCenter: CGPoint(x: side / 2, y: side / 2), endRadius: side / 2,
                                      options: [])
            }
        }
        return SKTexture(image: image)
    }
}

// MARK: - Çizim ve arayüze yayın

private extension GameScene {

    /// Görünen değerleri modele yalnızca değiştiklerinde yazar; SwiftUI'yi
    /// saniyede 120 kez yeniden çizmemek için gerekli.
    func publish() {
        guard let model else { return }
        let rounded = Int(scoreValue)
        if rounded != lastScoreShown {
            lastScoreShown = rounded
            model.score = rounded
        }
        if abs(model.multiplier - multiplier) > 0.049 {
            model.multiplier = multiplier
        }
        if model.wallsPassed != wallsPassed { model.wallsPassed = wallsPassed }
        if model.orbsTaken != orbsTaken { model.orbsTaken = orbsTaken }
        if abs(model.elapsed - elapsed) > 0.2 { model.elapsed = elapsed }
    }

    func paint(_ delta: TimeInterval) {
        guard size.width > 0, size.height > 0 else { return }

        let palette = Palette.at(distance: distance, phaseLength: size.height * Tuning.phaseLength)
        let primary = palette.primary
        let secondary = palette.secondary

        if let model {
            let color = primary.color
            if model.primaryColor != color { model.primaryColor = color }
            let second = secondary.color
            if model.secondaryColor != second { model.secondaryColor = second }
        }

        // Kayan ızgara — hız hissini veren tek şey.
        let offset = distance.truncatingRemainder(dividingBy: max(gridSpacing, 1))
        var row = 0
        for child in gridLayer.children {
            guard let line = child as? SKShapeNode else { continue }
            if line.name == "v" {
                line.fillColor = secondary.uiColor(alpha: 0.055)
                continue
            }
            let y = size.height - (CGFloat(row) * gridSpacing - offset)
            line.position = CGPoint(x: size.width / 2, y: y)
            line.fillColor = secondary.uiColor(alpha: 0.05 + 0.10 * (1 - min(max(y / size.height, 0), 1)))
            row += 1
        }

        // Duvarlar
        for wall in walls {
            wall.node.position = CGPoint(x: wall.offsetX, y: wall.y)
            // Ekranın üstünden yeni girenler yumuşakça belirsin.
            let entering = min(max((size.height - wall.y) / (size.height * 0.06), 0), 1)
            wall.node.alpha = entering
            for child in wall.node.children {
                guard let bar = child as? SKShapeNode else { continue }
                bar.fillColor = secondary.uiColor(alpha: 0.30)
                bar.strokeColor = primary.uiColor(alpha: 0.9)
            }
        }

        // Küreler
        for orb in orbs {
            orb.node.position = orb.position
            guard let shape = orb.node.children.first as? SKShapeNode else { continue }
            switch orb.kind {
            case .normal:
                shape.fillColor = primary.uiColor(alpha: 0.95)
                shape.strokeColor = .clear
            case .diamond:
                shape.fillColor = UIColor(red: 1, green: 212 / 255, blue: 0, alpha: 0.9)
                shape.strokeColor = UIColor(white: 1, alpha: 0.85)
            case .shield:
                shape.strokeColor = UIColor(red: 150 / 255, green: 220 / 255, blue: 255 / 255, alpha: 0.95)
                shape.fillColor = .clear
            }
        }

        // Gemi
        let visible = mode == .playing || mode == .countdown || mode == .paused
        let blinking = invulnerable > 0 && Int(invulnerable * 14) % 2 == 0
        playerNode.isHidden = !visible || blinking
        shieldRing.isHidden = !hasShield || !visible
        playerNode.position = CGPoint(x: playerX, y: playerY)
        shieldRing.position = playerNode.position
        playerNode.zRotation = -min(max((targetX - playerX) / (size.width * 0.16), -1), 1) * 0.32
        playerNode.strokeColor = primary.uiColor

        // Sarsıntı — tüm dünya katmanlarına aynı kayma uygulanır.
        if shakeAmount > 0.4 {
            let dx = CGFloat.random(in: -shakeAmount...shakeAmount)
            let dy = CGFloat.random(in: -shakeAmount...shakeAmount)
            let shift = CGPoint(x: dx, y: dy)
            [gridLayer, wallLayer, orbLayer, fxLayer].forEach { $0.position = shift }
            playerNode.position = CGPoint(x: playerX + dx, y: playerY + dy)
            shieldRing.position = playerNode.position
            shakeAmount = max(0, shakeAmount - size.width * 0.16 * CGFloat(delta))
        } else if shakeAmount > 0 {
            shakeAmount = 0
            [gridLayer, wallLayer, orbLayer, fxLayer].forEach { $0.position = .zero }
        }
    }
}
