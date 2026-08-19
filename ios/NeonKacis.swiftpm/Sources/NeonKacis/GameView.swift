import SwiftUI
import SpriteKit

struct GameView: View {

    @StateObject private var model = GameModel()
    @Environment(\.scenePhase) private var scenePhase

    @State private var scene: GameScene = {
        let scene = GameScene(size: CGSize(width: 390, height: 844))
        scene.scaleMode = .resizeFill
        return scene
    }()

    var body: some View {
        ZStack {
            Color(red: 5 / 255, green: 6 / 255, blue: 15 / 255)
                .ignoresSafeArea()

            SpriteView(scene: scene,
                       preferredFramesPerSecond: 120,
                       options: [.ignoresSiblingOrder])
                .ignoresSafeArea()

            hud
            overlay
        }
        .statusBarHidden()
        .persistentSystemOverlays(.hidden)
        .preferredColorScheme(.dark)
        .onAppear {
            scene.model = model
            Haptics.enabled = model.hapticsOn
            Synth.shared.enabled = model.soundOn
        }
        .onChange(of: scenePhase, perform: { phase in
            // Uygulama arka plana geçince koşu otomatik duraksın.
            if phase != .active { scene.pauseGame() }
        })
    }

    // MARK: - HUD

    private var hud: some View {
        VStack(spacing: 0) {
            HStack(alignment: .top) {
                Text("EN İYİ \(model.best)")
                    .font(.system(size: 11, weight: .semibold, design: .rounded))
                    .tracking(1.4)
                    .foregroundStyle(.secondary)
                    .padding(.horizontal, 12)
                    .padding(.vertical, 7)
                    .background(Capsule().fill(.white.opacity(0.06)))
                    .overlay(Capsule().stroke(.white.opacity(0.12)))

                Spacer()

                HStack(spacing: 8) {
                    iconButton(model.soundOn ? "speaker.wave.2.fill" : "speaker.slash.fill",
                               dimmed: !model.soundOn) {
                        model.soundOn.toggle()
                        Synth.shared.enabled = model.soundOn
                    }
                    iconButton(model.hapticsOn ? "waveform" : "waveform.slash",
                               dimmed: !model.hapticsOn) {
                        model.hapticsOn.toggle()
                        Haptics.enabled = model.hapticsOn
                        if model.hapticsOn { Haptics.tick() }
                    }
                    iconButton("pause.fill", dimmed: false) {
                        scene.pauseGame()
                    }
                    .disabled(!(model.isPlaying || model.isCountingDown))
                    .opacity(model.isPlaying || model.isCountingDown ? 1 : 0.35)
                }
            }

            VStack(spacing: 6) {
                Text("\(model.score)")
                    .font(.system(size: 46, weight: .bold, design: .monospaced))
                    .monospacedDigit()
                    .foregroundStyle(model.primaryColor)
                    .shadow(color: model.primaryColor.opacity(0.7), radius: 18)

                if model.multiplier > 1.02 {
                    Text("×\(model.multiplier, specifier: "%.1f")")
                        .font(.system(size: 13, weight: .bold, design: .monospaced))
                        .tracking(1.2)
                        .foregroundStyle(model.multiplier > 4 ? Color.yellow : model.primaryColor)
                }
            }
            .padding(.top, 14)
            .animation(.easeOut(duration: 0.15), value: model.multiplier > 1.02)

            Spacer()
        }
        .padding(.horizontal, 16)
        .padding(.top, 12)
        .opacity(model.isPlaying || model.isCountingDown ? 1 : 0)
        .animation(.easeInOut(duration: 0.25), value: model.isPlaying)
    }

    private func iconButton(_ systemName: String, dimmed: Bool, action: @escaping () -> Void) -> some View {
        Button(action: action) {
            Image(systemName: systemName)
                .font(.system(size: 14, weight: .semibold))
                .frame(width: 38, height: 38)
                .background(RoundedRectangle(cornerRadius: 12).fill(.white.opacity(0.06)))
                .overlay(RoundedRectangle(cornerRadius: 12).stroke(.white.opacity(0.12)))
        }
        .buttonStyle(.plain)
        .foregroundStyle(dimmed ? Color.secondary : Color.primary)
    }

    // MARK: - Katmanlar

    @ViewBuilder
    private var overlay: some View {
        switch model.mode {
        case .menu:
            MenuOverlay(model: model) { scene.startNewRun() }
        case .countdown(let value):
            Text("\(value)")
                .font(.system(size: 96, weight: .bold, design: .monospaced))
                .foregroundStyle(model.primaryColor)
                .shadow(color: model.primaryColor.opacity(0.8), radius: 30)
                .transition(.scale.combined(with: .opacity))
                .id(value)
        case .paused:
            PauseOverlay(model: model,
                         onResume: { scene.resumeGame() },
                         onQuit: { scene.abandonRun() })
        case .over:
            GameOverOverlay(model: model,
                            onAgain: { scene.startNewRun() },
                            onMenu: { scene.returnToMenu() })
        case .playing:
            EmptyView()
        }
    }
}

// MARK: - Ortak parçalar

struct GlassBackdrop: View {
    var body: some View {
        Color(red: 5 / 255, green: 6 / 255, blue: 15 / 255)
            .opacity(0.72)
            .background(.ultraThinMaterial)
            .ignoresSafeArea()
    }
}

struct NeonButton: View {
    let title: String
    let primary: Color
    let secondary: Color
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            Text(title)
                .font(.system(size: 15, weight: .bold, design: .rounded))
                .tracking(1.6)
                .foregroundStyle(Color(red: 4 / 255, green: 6 / 255, blue: 14 / 255))
                .frame(maxWidth: .infinity)
                .padding(.vertical, 16)
                .background(
                    RoundedRectangle(cornerRadius: 16)
                        .fill(LinearGradient(colors: [primary, secondary],
                                             startPoint: .topLeading,
                                             endPoint: .bottomTrailing))
                )
        }
        .buttonStyle(.plain)
    }
}

struct GhostButton: View {
    let title: String
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            Text(title)
                .font(.system(size: 13, weight: .bold, design: .rounded))
                .tracking(1.4)
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity)
                .padding(.vertical, 13)
                .background(RoundedRectangle(cornerRadius: 16).stroke(.white.opacity(0.12)))
        }
        .buttonStyle(.plain)
    }
}

struct StatTile: View {
    let value: String
    let label: String

    var body: some View {
        VStack(spacing: 6) {
            Text(value)
                .font(.system(size: 18, weight: .bold, design: .monospaced))
                .monospacedDigit()
            Text(label)
                .font(.system(size: 9, weight: .semibold, design: .rounded))
                .tracking(1.4)
                .foregroundStyle(.secondary)
        }
        .frame(maxWidth: .infinity)
        .padding(.vertical, 12)
    }
}
