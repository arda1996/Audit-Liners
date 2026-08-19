# Neon Kaçış — native iOS uygulaması

Swift + SpriteKit ile yazılmış, **gerçek native** iOS oyunu. Tarayıcı yok, WebView yok.
Ana ekranda kendi ikonu olan, tamamen çevrimdışı çalışan bir uygulama.

---

## Neden bu kurulum? (Mac olmadan native geliştirme)

Bu depo üzerinde çalışılan ortam **Linux**. iOS uygulaması Linux'ta derlenemez, çünkü:

- **UIKit / SwiftUI / SpriteKit Linux'ta yok** — kapalı kaynak, yalnızca Apple platformlarında.
  Swift dili Linux'ta çalışır, ama bu çerçeveler çalışmaz.
- **iOS SDK yalnızca Xcode ile dağıtılır**, Xcode da yalnızca macOS'ta çalışır.
- **iOS Simulator macOS'a bağlıdır** — Linux'ta çalıştırmanın yolu yoktur.

Topluluğun bu duvarı aşmak için yazdığı araç [**xtool**](https://github.com/xtool-org/xtool)
(MIT, Swift Forums'da tanıtıldı) Linux'ta iOS uygulaması derleyebiliyor — ama
[kurulum belgesine göre](https://deepwiki.com/xtool-org/xtool/3.2-linux-installation)
*"xtool requires an official Xcode.xip to build the cross-compilation SDK"*: Apple hesabıyla
~10 GB'lık Xcode arşivini indirip iOS SDK'sını çıkarman gerekiyor. Kendi Linux/Windows
makinende bu yol açık; bu CI ortamında disk ve lisans nedeniyle uygun değil.

Seçilen yol daha basit ve tamamen resmî araçlarla: **derlemeyi GitHub Actions'ın
macOS runner'ı yapıyor.** Bu, Xcode kurulu gerçek bir Mac. Depo public olduğu için
standart runner'lar ücretsiz.

```
Kod (burada, Linux)  →  git push  →  macOS runner + xcodebuild  →  imzasız .ipa
                                                                        ↓
                                          telefonda kendi Apple ID'nle imzala ve kur
```

---

## Neden `.swiftpm` (App Playground) biçimi?

Tek kaynak, üç araç. Aynı klasör:

| Araç | Nasıl açılır | Notu |
|---|---|---|
| **Swift Playgrounds** (iPad/Mac) | `.swiftpm`'i aç, Çalıştır'a bas | Cihazda anında derler ve çalıştırır |
| **Xcode** (Mac) | `.swiftpm`'i aç | App Store'a çıkarken kullanılacak yol |
| **xcodebuild** (CI / Mac) | `xcodebuild -project NeonKacis.swiftpm` | Bu deponun iş akışı |

Ayrı bir `.xcodeproj` tutup senkron kalmaya çalışmaya gerek kalmıyor.

---

## Uygulamayı telefonuna kurmak

### 1. `.ipa`'yı indir

[Actions sekmesinde](https://github.com/arda1996/Audit-Liners/actions/workflows/ios-build.yml)
son başarılı çalıştırmayı aç → sayfanın altındaki **`NeonKacis-imzasiz-ipa`** ekini indir.
Telefondan da indirilebilir (GitHub'a giriş yapmış olman gerekir).

Dosya **imzasızdır** — imzalama telefonda, senin Apple ID'nle yapılır. Kimlik bilgilerin
hiçbir zaman bu depoya veya CI'ya girmez.

### 2. Telefonda imzala ve kur

Bir yandan yükleyici (sideloader) gerekiyor. Yaygın seçenekler:

| Araç | Bilgisayar gerekir mi | Not |
|---|---|---|
| **SideStore** | İlk kurulumda bir kez (eşleştirme dosyası); sonrası telefondan | En yaygın; WireGuard ile telefondan yenileme |
| **AltStore** | AltServer için bir bilgisayar | SideStore'un kaynağı |
| **Xcode** | Mac gerekir | Elinde Mac varsa en temiz yol |

Kurduktan sonra `.ipa`'yı yükleyiciye ver, **kendi Apple ID'nle** imzalat.

### 3. Süre sınırı

| Hesap | Uygulama ne kadar çalışır | Ücret |
|---|---|---|
| **Ücretsiz Apple ID** | 7 gün, sonra telefondan yenilersin | Ücretsiz |
| **Apple Developer Program** | 1 yıl | Yıllık $99 |

7 günlük yenileme telefondan, WiFi üzerinden yapılır — bilgisayar gerekmez.
Uygulamayı sildirmez, sadece imzayı tazeler; skorların durur.

> Bu yol Apple'ın kendi izin verdiği "kendi cihazına kendi uygulamanı kurma" yoludur.
> Jailbreak gerekmez.

### Elinde iPad varsa — çok daha kısa yol

`ios/NeonKacis.swiftpm` klasörünü iPad'e taşı (iCloud Drive / Dosyalar yeter),
**Swift Playgrounds**'ta aç, **Çalıştır**'a bas. Derleme cihazda yapılır: CI yok,
`.ipa` yok, yükleyici yok, 7 gün yok. Kodu orada da düzenleyebilirsin.

Apple'ın Swift Playgrounds'u [iPad ve Mac uygulamasıdır](https://support.apple.com/guide/playgrounds-ipad/welcome/ipados) —
iPhone *için* uygulama yapar ama kendisi iPhone'da çalışmaz.

---

## Native'in web sürümüne göre somut kazancı

| | Web (PWA) | Bu sürüm |
|---|---|---|
| Kare hızı | 60 Hz (tarayıcı sınırı) | **120 Hz** ProMotion |
| Titreşim | ❌ iOS Safari `navigator.vibrate` desteklemez | ✅ **Taptic Engine** — küre, elmas, kalkan, çarpışma ayrı ayrı |
| Çizim | Canvas 2D | **Metal** hızlandırmalı SpriteKit |
| Parçacıklar | JS döngüsü | `SKEmitterNode` — GPU'da |
| Ana ekran | "Ana Ekrana Ekle" kısayolu | Gerçek uygulama |
| App Store'a çıkış | ❌ | ✅ (aynı proje Xcode'da açılır) |

---

## Dosyalar

| Dosya | Ne yapar |
|---|---|
| `NeonKacisApp.swift` | Uygulama girişi |
| `GameView.swift` | SwiftUI katmanı: HUD, düğmeler, sahne barındırma |
| `Overlays.swift` | Menü, duraklatma ve oyun sonu ekranları |
| `GameScene.swift` | Oyunun tamamı: simülasyon, çarpışma, üretim, çizim |
| `Palette.swift` | Faz renkleri ve harmanlama |
| `GameModel.swift` | Sahne ↔ arayüz köprüsü + `UserDefaults` kalıcılığı |
| `Haptics.swift` | Taptic Engine geri bildirimi |
| `Synth.swift` | Ses dosyasız efekt sentezi (PCM tamponları) |

---

## Teknik notlar

- **Simülasyon sabit 1/120 s adımlı.** Hız arttıkça duvarların içinden geçme
  (tünelleme) olmaz. Çizim ekranın tazeleme hızında, simülasyondan bağımsız.
- **Çarpışma ile görüntü ayrışamaz:** duvarın dolu parçaları hem çizimde hem
  çarpışmada tek fonksiyondan (`segments(from:)`) türetilir.
- Duvar kenar parçaları ekran dışına taşırılır; duvar yatay kayarken yanlardan
  boşluk açılmaz.
- Oyuncu yarıçapı çarpışmada `×0.86` alınır — görselden biraz bağışlayıcı,
  "haksız ölüm" hissini azaltır.
- SwiftUI arayüzü saniyede 120 kez değil, yalnızca **görünen bir değer değiştiğinde**
  yeniden çizilir (skor tamsayısı değişmedikçe modele yazılmaz).
- Ses motoru açılamazsa oyun **sessiz çalışmaya devam eder** — ses hiçbir zaman
  oynanışı engellemez.
- Uygulama arka plana geçince koşu otomatik duraklar, dönünce 3-2-1 geri sayımla sürer.

## Denge ayarları

Hepsi `GameScene.swift` içindeki `Tuning` bloğunda, tek yerde:

| Ayar | Anlamı | Şu an |
|---|---|---|
| `difficultyRamp` | Zorluğun tavana çıkma mesafesi | ekran boyu × 95 (~90 sn) |
| `speedMin` / `speedMax` | Hız aralığı (ekran boyu / sn) | 0.55 → 1.42 |
| `gapWide` / `gapNarrow` | Boşluk genişliği (ekran eni katı) | 0.44 → 0.205 |
| `spawnFar` / `spawnNear` | Duvarlar arası mesafe | 0.60 → 0.345 |
| `phaseLength` | Renk fazı uzunluğu | ekran boyu × 20 |
| `multDecayDelay` | Çarpanın düşmeye başlama gecikmesi | 2,6 sn |
| `collisionForgiveness` | Çarpışma bağışlayıcılığı | 0.86 |
