import SwiftUI

// MARK: - Menü

struct MenuOverlay: View {
    @ObservedObject var model: GameModel
    let onStart: () -> Void

    var body: some View {
        ZStack {
            GlassBackdrop()
            VStack(spacing: 14) {
                Text("NEON KAÇIŞ")
                    .font(.system(size: 36, weight: .heavy, design: .rounded))
                    .tracking(5)
                    .foregroundStyle(model.primaryColor)
                    .shadow(color: model.primaryColor.opacity(0.7), radius: 24)

                Text("Duvarlardaki boşluklardan süz. Ne kadar uzak gidersen o kadar hızlanır.")
                    .font(.system(size: 14, weight: .medium, design: .rounded))
                    .foregroundStyle(.secondary)
                    .multilineTextAlignment(.center)
                    .frame(maxWidth: 300)

                VStack(spacing: 7) {
                    rule("hand.point.up.left.fill",
                         "Ekranın **herhangi bir yerinden** parmağını sağa-sola sürükle — gemi seni takip eder.")
                    rule("circle.fill",
                         "Küreleri topla: **çarpan** artar, skor katlanır. Toplamayı bırakırsan çarpan düşer.")
                    rule("diamond.fill",
                         "**Elmaslar** kenarlarda durur — oraya gitmek riskli ama üç kat değerli.")
                    rule("shield.lefthalf.filled",
                         "**Kalkan** bir çarpışmayı yutar.")
                }

                NeonButton(title: "BAŞLA",
                           primary: model.primaryColor,
                           secondary: model.secondaryColor,
                           action: onStart)

                Text(model.best > 0
                     ? "En iyi skorun \(model.best) · \(model.totalRuns) koşu"
                     : "İlk koşun. Bol şans.")
                    .font(.system(size: 11, weight: .medium, design: .rounded))
                    .foregroundStyle(.tertiary)
            }
            .padding(24)
            .frame(maxWidth: 380)
        }
    }

    private func rule(_ icon: String, _ text: LocalizedStringKey) -> some View {
        HStack(alignment: .top, spacing: 11) {
            Image(systemName: icon)
                .font(.system(size: 15))
                .frame(width: 22)
                .foregroundStyle(model.primaryColor.opacity(0.9))
            Text(text)
                .font(.system(size: 12, weight: .medium, design: .rounded))
                .foregroundStyle(.secondary)
                .fixedSize(horizontal: false, vertical: true)
            Spacer(minLength: 0)
        }
        .padding(.horizontal, 13)
        .padding(.vertical, 10)
        .background(RoundedRectangle(cornerRadius: 11).fill(.white.opacity(0.04)))
        .overlay(RoundedRectangle(cornerRadius: 11).stroke(.white.opacity(0.10)))
    }
}

// MARK: - Duraklatma

struct PauseOverlay: View {
    @ObservedObject var model: GameModel
    let onResume: () -> Void
    let onQuit: () -> Void

    var body: some View {
        ZStack {
            GlassBackdrop()
            VStack(spacing: 14) {
                Text("DURAKLATILDI")
                    .font(.system(size: 26, weight: .heavy, design: .rounded))
                    .tracking(4)
                    .foregroundStyle(model.primaryColor)

                HStack(spacing: 0) {
                    StatTile(value: "\(model.score)", label: "SKOR")
                    Divider().frame(height: 40)
                    StatTile(value: String(format: "×%.1f", model.multiplier), label: "ÇARPAN")
                    Divider().frame(height: 40)
                    StatTile(value: timeText(model.elapsed), label: "SÜRE")
                }
                .background(RoundedRectangle(cornerRadius: 14).fill(.white.opacity(0.04)))
                .overlay(RoundedRectangle(cornerRadius: 14).stroke(.white.opacity(0.10)))

                NeonButton(title: "DEVAM ET",
                           primary: model.primaryColor,
                           secondary: model.secondaryColor,
                           action: onResume)
                GhostButton(title: "KOŞUYU BİTİR", action: onQuit)
            }
            .padding(24)
            .frame(maxWidth: 380)
        }
    }
}

// MARK: - Oyun sonu

struct GameOverOverlay: View {
    @ObservedObject var model: GameModel
    let onAgain: () -> Void
    let onMenu: () -> Void

    var body: some View {
        ZStack {
            GlassBackdrop()
            VStack(spacing: 14) {
                if model.isNewBest {
                    Text("YENİ REKOR")
                        .font(.system(size: 11, weight: .heavy, design: .rounded))
                        .tracking(3)
                        .foregroundStyle(Color(red: 4 / 255, green: 6 / 255, blue: 14 / 255))
                        .padding(.horizontal, 14)
                        .padding(.vertical, 8)
                        .background(Capsule().fill(model.primaryColor))
                } else {
                    Text("SKOR")
                        .font(.system(size: 11, weight: .semibold, design: .rounded))
                        .tracking(3)
                        .foregroundStyle(.secondary)
                }

                Text("\(model.score)")
                    .font(.system(size: 68, weight: .bold, design: .monospaced))
                    .monospacedDigit()
                    .foregroundStyle(model.primaryColor)
                    .shadow(color: model.primaryColor.opacity(0.7), radius: 28)

                HStack(spacing: 0) {
                    StatTile(value: "\(model.best)", label: "EN İYİ")
                    Divider().frame(height: 40)
                    StatTile(value: "\(model.wallsPassed)", label: "DUVAR")
                    Divider().frame(height: 40)
                    StatTile(value: "\(model.orbsTaken)", label: "KÜRE")
                    Divider().frame(height: 40)
                    StatTile(value: timeText(model.elapsed), label: "SÜRE")
                }
                .background(RoundedRectangle(cornerRadius: 14).fill(.white.opacity(0.04)))
                .overlay(RoundedRectangle(cornerRadius: 14).stroke(.white.opacity(0.10)))

                NeonButton(title: "TEKRAR OYNA",
                           primary: model.primaryColor,
                           secondary: model.secondaryColor,
                           action: onAgain)

                if !model.recentRuns.isEmpty {
                    VStack(spacing: 6) {
                        ForEach(Array(model.recentRuns.enumerated()), id: \.offset) { index, run in
                            HStack {
                                Text(index == 0 ? "BU KOŞU" : "\(index + 1).")
                                Spacer()
                                Text("\(run)")
                            }
                            .font(.system(size: 12, weight: .semibold, design: .monospaced))
                            .monospacedDigit()
                            .foregroundStyle(run == model.best ? Color.primary : Color.secondary)
                            .padding(.horizontal, 14)
                            .padding(.vertical, 9)
                            .background(RoundedRectangle(cornerRadius: 10)
                                .fill(.white.opacity(run == model.best ? 0.07 : 0.03)))
                        }
                    }
                }

                GhostButton(title: "MENÜ", action: onMenu)
            }
            .padding(24)
            .frame(maxWidth: 380)
        }
    }
}

func timeText(_ seconds: TimeInterval) -> String {
    let total = Int(seconds)
    return String(format: "%d:%02d", total / 60, total % 60)
}
