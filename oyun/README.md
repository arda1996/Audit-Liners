# Neon Kaçış 🕹️

iPhone'da tam ekran, çevrimdışı ve tek parmakla oynanan refleks oyunu.
**Tek dosya, bağımlılık yok, sunucu yok, hesap yok** — skorlar telefonunda kalır.

> Bu klasör Audit-Liners muhasebe/denetim sisteminden tamamen bağımsızdır.
> Rust/Tauri/React tarafındaki hiçbir dosyaya dokunmaz; ortak bir yapı adımı yoktur.

---

## Neden native değil de web?

Gerçek bir `.ipa` üretmek **Xcode** ister, Xcode da yalnızca **macOS**'ta çalışır.
Bu depo üzerinde çalışılan ortam Linux olduğu için native derleme mümkün değil.
Buna karşılık "sadece ben oynayacağım, lokalde çalışsın yeter" senaryosunda
PWA yolu native ile pratikte aynı sonucu verir:

| | Native (Swift) | Bu oyun (PWA) |
|---|---|---|
| Ana ekranda ikon | ✅ | ✅ |
| Tam ekran, tarayıcı çubuğu yok | ✅ | ✅ |
| Çevrimdışı çalışır | ✅ | ✅ (`sw.js`) |
| Mac + Xcode gerekir | zorunlu | gerekmez |
| Apple Developer üyeliği ($99/yıl) | gerekir | gerekmez |
| 7 günde bir yeniden imzalama | gerekir (ücretsiz hesapla) | gerekmez |
| App Store'a çıkma | ✅ | ❌ (bu senaryoda gerekmiyor) |

---

## iPhone'a kurulum

### Yol 1 — Bilgisayardan sunarak (tam PWA, çevrimdışı)

Telefon ve bilgisayar **aynı Wi-Fi ağında** olmalı.

```bash
cd oyun
python3 -m http.server 8099        # ya da:  npx serve -l 8099 .
ipconfig getifaddr en0             # macOS'ta yerel IP;  Linux'ta:  hostname -I
```

Telefonda Safari'de `http://<yerel-ip>:8099/` adresini aç →
**Paylaş ⬆️ → Ana Ekrana Ekle**.

Bir kez açtıktan sonra service worker dosyaları önbelleğe alır; bilgisayar
kapalıyken bile ana ekrandaki ikondan **çevrimdışı** oynanır.

### Yol 2 — Tek dosya olarak

`index.html` tek başına da çalışır (manifest ve service worker sessizce atlanır).
Dosyayı iCloud Drive / Dosyalar'a atıp Safari'de açman yeterli.
Bu yolda çevrimdışı önbellek yoktur; skorlar yine `localStorage`'da tutulur.

---

## Nasıl oynanır

- **Ekranın herhangi bir yerinden** parmağını sağa-sola sürükle. Gemi seni takip eder —
  bu yüzden parmağın gemiyi kapatmaz. (Masaüstünde: ← → tuşları, boşluk, `Esc` duraklat.)
- Duvarlardaki boşluklardan geç. Mesafe arttıkça **hız artar, boşluk daralır**;
  yaklaşık 90 saniyede zorluk tavana ulaşır.
- **● Küre** — çarpanı artırır (`×1` → `×9.9`), skor katlanır.
  2,6 saniye küre toplamazsan çarpan düşmeye başlar.
- **◆ Elmas** — ekranın kenarlarında doğar. Almak için duvar boşluğundan sapman gerekir:
  riskli ama üç kat değerli.
- **◯ Kalkan** — bir çarpışmayı yutar, ardından kısa bir dokunulmazlık verir.
- Her ~20 ekran boyunda **faz** değişir: palet MAVİ → MOR → YEŞİL → ALTIN → KIRMIZI
  sırasıyla harmanlanarak döner.

---

## Dosyalar

| Dosya | Ne işe yarar |
|---|---|
| `index.html` | Oyunun tamamı: stil, oyun döngüsü, çizim, ses. Harici bağımlılık yok. |
| `manifest.webmanifest` | Ana ekran adı/ikonu, `display: standalone`. |
| `sw.js` | Çevrimdışı önbellek (önce ağ, olmazsa önbellek). |
| `icon.svg` | Uygulama ikonu. Ayrıca iOS için `apple-touch-icon` çalışma anında çizilir. |

---

## Teknik notlar

- **Sabit adımlı simülasyon** (1/120 s): yüksek hızda duvarların içinden geçme
  (tünelleme) olmaz. Çizim ekran tazeleme hızında.
- **Çarpışma** daire↔dikdörtgen; oyuncu yarıçapı çarpışmada `×0.86` alınır —
  görselden biraz bağışlayıcıdır, "haksız ölüm" hissini azaltır.
- **Başarım koruması:** kare süresi üst üste iki ölçüm penceresinde yüksek çıkarsa
  parlama (`shadowBlur`) ve parçacık sayısı otomatik kısılır.
- **Ses** WebAudio ile sentezlenir, ses dosyası yoktur. iOS'ta ses ilk dokunuşta açılır.
- **Güvenli alan:** çentik ve ana ekran çubuğu için `env(safe-area-inset-*)` kullanılır.
- Uygulama arka plana alınınca (`visibilitychange`) oyun otomatik duraklar,
  dönünce 3-2-1 geri sayımla devam eder.
- `?debug` parametresiyle açılırsa durum nesnesi `window.__neon` altında görünür
  (yalnızca test amaçlı; normal açılışta tanımlı değildir).

---

## Değiştirmek isteyebileceğin ayarlar

Hepsi `index.html` içinde, `update()` ve `spawnWall()` fonksiyonlarında:

| Ne | Nerede | Şu an |
|---|---|---|
| Zorluğun tavana çıkma süresi | `diff()` | `H * 95` (~90 sn) |
| Hız aralığı | `S.speed` | ekran boyunun `0.55` → `1.42` katı / sn |
| Boşluk genişliği | `frac` | ekran eninin `0.44` → `0.205` katı |
| Duvarlar arası mesafe | `gapDist` | ekran boyunun `0.60` → `0.345` katı |
| Çarpan düşme gecikmesi | `S.lastOrb` karşılaştırması | 2,6 sn |
| Faz uzunluğu (renk değişimi) | `palAt()` | `H * 20` |
