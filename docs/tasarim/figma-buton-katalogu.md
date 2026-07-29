# Figma Buton Kataloğu — Tasarım Sistemi Damıtımı

**Tarih:** 2026-07-18 · **Kaynak:** Figma Make çıktısı ("Atelyé UI Arşivi", React 19 + Vite + Tailwind v4 — ama stiller %100 inline/CSS değişkeni, Tailwind sınıfı KULLANILMAMIŞ → saf CSS'e taşınabilir)
· **Kaynak dosyalar:** `src/App.tsx` (1609 satır, 13 bileşen kategorisi), `src/index.css` (79 satır: token + keyframe)

> Tüm değerler dosyadan birebir alınmıştır; uydurma yoktur. Kataloğun kendi teması "bej/bordo atölye"
> temasıdır — beyaz temaya uyarlama önerisi §6'dadır.

---

## 1. Renk Paleti + Tokenlar

### 1.1 Renk tokenları (`index.css` `:root`)

| Token | Değer | Rol |
|---|---|---|
| `--background` | `#F5F0E8` | Sayfa zemini (krem/bej) |
| `--foreground` | `#1C1410` | Ana metin (sıcak siyah-kahve) |
| `--card` | `#EDE7D9` | Yüzey/kart zemini (zeminden 1 ton koyu) |
| `--card-foreground` | `#1C1410` | Kart metni |
| `--primary` | `#6B1E2E` | Birincil aksiyon (bordo) |
| `--primary-foreground` | `#F5F0E8` | Birincil üstü metin |
| `--secondary` | `#A67C52` | İkincil aksiyon (deri kahve) — focus border olarak da kullanılıyor |
| `--secondary-foreground` | `#F5F0E8` | |
| `--accent` | `#C9A84C` | Vurgu (hardal altın) |
| `--accent-foreground` | `#1C1410` | |
| `--muted` | `#D6CDBF` | Pasif zemin (pill-tab rayı, toggle off) |
| `--muted-foreground` | `#6B5E4E` | İkincil metin/etiket |
| `--border` | `#C4B89A` | Kenarlık |
| `--ring` | `#A67C52` | Focus halkası |
| `--radius` | `2px` | Global köşe — **neredeyse keskin köşe, sistemin imzası** |

### 1.2 Durum (semantik) renkleri — badge/toast/tablo durumlarında sabit kodlu

| Durum | bg | fg | border |
|---|---|---|---|
| success | `#e8f5e0` | `#3a6b20` | `#c8e4b0` |
| warning | `#fef3cd` | `#7a5a10` | `#e8d890` |
| error | `#fde8ea` | `#b03040` | `#f0b8be` |
| info | `#e0edf8` | `#1e5a8a` | `#b0cfe8` |

Ek sabitler: destructive buton `#b03040`; press-butonu 3D gölge rengi `#7a5a38`; Ink buton zemini `#1C1410`; split-buton koyu segmenti `#5a1826`; hata border `#b03040`.

### 1.3 Tipografi — üç fontlu sistem (sistemin en güçlü fikri)

| Font | Rol | Kullanım |
|---|---|---|
| **Playfair Display** (serif, çoğu italic) | Display/başlık | h1 48/700/italic · h2 36/600 · h3 28/400/italic · modal başlık 19/600 · kart başlık 14-15/600 · stat değeri 30/700 |
| **Jost** (geometrik sans) | Gövde + buton | body 14/400 lh 1.6 · gövde 16-14-12 (ağırlık 400/400/300) · buton 11-13-15/500, `letterSpacing: 0.09em`, `textTransform: uppercase` |
| **DM Mono** | Etiket, kod, veri, para | form label 10/`0.12em`/uppercase · tablo başlığı 9/`0.14em`/uppercase · badge 10/`0.1em` · fiyat/tutar 12-13/500 |

Kural: **para/kod/tarih daima DM Mono; başlık daima serif; etkileşim (buton/sekme) daima Jost uppercase + geniş letter-spacing.** Muhasebe uygulaması için mono-rakam kuralı birebir değerli.

### 1.4 Spacing / radius / gölge skalaları (Tokens bölümünden)

- **Spacing:** `4, 8, 12, 16, 24, 32, 48, 64`
- **Radius:** `0, 2, 4, 8, 12, 24, full` — pratikte **2px her yerde**; istisna: pill-tab rayı 4px, toggle 12px, daire (avatar/step/bildirim) full
- **Gölgeler:**
  - xs `0 1px 3px rgba(28,20,16,0.08)`
  - sm `0 2px 8px rgba(28,20,16,0.10)`
  - md `0 8px 20px rgba(28,20,16,0.14)`
  - lg `0 20px 50px rgba(28,20,16,0.18)`
  - modal `0 32px 80px rgba(28,20,16,0.28)` · drawer `±20px 0 50px rgba(28,20,16,0.16)` · kart hover `0 10px 28px rgba(28,20,16,0.12)`
  - **Gölge rengi nötr siyah değil, foreground'un kendisi** (`rgba(28,20,16,…)`) → sıcak, mürekkep hissi.

**Beyaz tema uyumu:** Paletin yapısı (bg → card 1 ton koyu → muted → border, hepsi aynı sıcak eksende) beyaz temaya birebir taşınabilir; sadece eksen bej yerine gri-beyaz olur (bkz. §6).

---

## 2. Buton Anatomisi

**Ortak anatomi (tüm varyantlar):**
- Boyut skalası sm/md/lg → padding-x `14/22/32px`, padding-y `7/11/15px` (outline'lı varyantlarda py `6/10/14px` — 2px border telafisi, hizalar eşit kalır), font-size `11/13/15px`
- `fontFamily: Jost, fontWeight 500, letterSpacing 0.09em, textTransform: uppercase, borderRadius: var(--radius)` (=2px)
- **Disabled: `opacity: 0.42` + `cursor: not-allowed`** (tüm varyantlarda aynı)
- Loading: `opacity 0.8` + 12px spinner (`border: 2px solid rgba(245,240,232,0.3); borderTopColor: #F5F0E8; animation: spin 700ms linear infinite`) + `cursor: wait`

**8 etkileşim tipi + 3 kompozit = 11 desen:**

| # | Varyant | Zemin/kenar | Etkileşim | Süre |
|---|---|---|---|---|
| 1 | **Ripple** (primary/secondary/accent) | dolu `var(--primary/secondary/accent)`, border yok | tıklamada 36px daire `rgba(255,255,255,0.35)` → `scale(4.5)` sönümlenir (`ripple-out 550ms ease-out`); hover `filter: brightness(1.1)` (0.2s) | 550ms |
| 2 | **Press** (mekanik) | `var(--secondary)` + `boxShadow: 0 5px 0 #7a5a38` | mousedown: `translateY(4px)` + gölge `0 1px 0` | 70ms transform+shadow |
| 3 | **Stamp** (damga) | transparent + `2px solid var(--primary)`, metin primary | hover: dolu primary'ye döner; tıklamada `stamp-press 400ms` (scale 1→0.87→1.05→1) | 400ms |
| 4 | **Shiver** | dolu `var(--accent)` | tıklamada `shiver 450ms` (skewX ±2.5° salınım) | 450ms |
| 5 | **Ink** (mürekkep) | `#1C1410` zemin, `#F5F0E8` metin, **DM Mono** 400/`0.12em` | tıklamada primary katman `clip-path: circle(0%→150%)` (`ink-expand 640ms ease-out`) | 640ms |
| 6 | **Ghost** | transparent + `1px solid var(--border)`, metin muted-fg, ağırlık 400/`0.06em`, uppercase DEĞİL | hover: bg `rgba(107,30,46,0.07)` + metin primary + `underline` (`textUnderlineOffset: 3`), `transition: all 0.22s` | 220ms |
| 7 | **Embossed** | `linear-gradient(145deg,#EDE7D9,#D0C8B8)` + border; **Playfair 600 italic** | press: `inset 2px 2px 4px rgba(0,0,0,0.16)` + `scale(0.98)` (90ms); normal: `3px 3px 5px rgba(0,0,0,0.13), -1px -1px 3px rgba(255,255,255,0.65)` | 90ms |
| 8 | **Loading** | primary dolu | tıkla → spinner + "Yükleniyor…" + disabled | spin 700ms |
| 9 | **Icon group** (segment) | `var(--card)` + border, `marginLeft:-1` bitişik; aktif segment primary dolu, `zIndex:1` | hover: bg `var(--muted)` | 0.15s |
| 10 | **Split button** | Ripple + koyu ok segmenti `#5a1826`, `borderLeft: 1px solid rgba(255,255,255,0.15)` | dropdown: `position:absolute top:100% right:0 marginTop:4`, card zemin + border, `boxShadow 0 8px 24px rgba(28,20,16,0.14)`, `fade-in 150ms`; öğe hover bg muted | 150ms |
| 11 | **Destructive** (ConfirmModal içinde) | `#b03040` dolu, beyaz metin, `10px 20px`, `0.08em` uppercase | — | — |

**Audit-Liners için damıtım:** üretimde 4 çekirdek yeter — **Primary (Ripple), Outline (Stamp'ın statik hali), Ghost, Destructive** + loading/disabled durumları. Press/Shiver/Ink/Embossed gösteri parçası; ciddi denetim arayüzünde kullanma (Stamp'ın "mühür" animasyonu belki onay/kilitleme aksiyonunda tematik olarak tutulabilir).

---

## 3. Açılır Sayfa Sistemleri

### 3.1 Modal (merkez diyalog)
- **Backdrop:** `position:fixed inset:0 z-index:1000; background: rgba(28,20,16,0.55); backdrop-filter: blur(4px)`; `fade-in 180ms ease-out`; backdrop'a tıkla → kapanır (`e.target === e.currentTarget` kontrolü)
- **Panel:** `background: var(--background); border: 1px solid var(--border)`; `maxWidth: sm 380 / md 520 / lg 760 / full calc(100vw-48px)`; `maxHeight: 90vh; overflowY: auto`; gölge `0 32px 80px rgba(28,20,16,0.28)`; giriş `slide-up 260ms cubic-bezier(0.22,1,0.36,1)` (translateY(24px)+scale(0.97) → 0/1)
- **Başlık şeridi:** `padding: 18px 24px; borderBottom: 1px solid var(--border); background: var(--card)` — başlık Playfair 19/600, sağda × butonu (20px, muted-fg → hover foreground, 0.15s)
- **Gövde:** `padding: 24`; aksiyonlar gövde sonunda `display:flex gap:10` — birincil solda, ghost "Vazgeç" sağında
- **Kapatma:** Escape tuşu (window keydown listener) + backdrop tıklama + × butonu — üçü de var
- Türler: bilgi (sm), detay (md), karşılaştırma grid'i (lg), destructive onay (sm + kırmızı buton), form modalı (md, dikey `gap:16` form + aksiyon satırı)

### 3.2 Drawer (kenar paneli) — sağ / sol / alt
- **Backdrop:** modaldan hafif: `rgba(28,20,16,0.4)` + `blur(2px)`, `fade-in 180ms`
- **Panel:** `width: 380` (sağ/sol) veya `height: 380` (alt); kenara `1px solid var(--border)` + yönlü gölge `∓20px 0 50px rgba(28,20,16,0.16)`; `display:flex column` → başlık şeridi (modalla aynı: 18px 24px, card zemin, Playfair 17/600) + gövde `flex:1 padding:24 overflowY:auto`
- Kapatma: Escape + backdrop + ×. (Not: panelin kendi slide-in animasyonu tanımlı değil — sadece backdrop fade; uygularken `slide-in-right` eklemek iyileştirme olur.)

### 3.3 Dropdown/popover (split button menüsü)
- Tetikleyene `position:relative`; menü `absolute top:100%` + `marginTop:4`, `minWidth:160 z-index:50`, card zemin + border + `0 8px 24px` gölge, `fade-in 150ms`; öğeler `10px 14px` Jost 12, hover bg muted (0.15s)

### 3.4 Toast (bildirim)
- Yığın: `position:fixed bottom:24 right:24 z-index:2000; column gap:10`
- Kutu: `padding:12px 16px; minWidth:280 maxWidth:380`; durum bg + `1px solid ${fg}22` + durum fg metin; gölge `0 8px 24px rgba(28,20,16,0.16)`; giriş `slide-up 250ms cubic-bezier(0.22,1,0.36,1)`; **3500ms sonra otomatik kaybolur** + × ile manuel; ikon DM Mono (✓ ✗ ! i)

### 3.5 Accordion
- Öğe: `border: 1px solid var(--border); background: var(--card)`, aralarında `gap:2`; başlık `padding:14px 18px` Jost 13/500; sağda `+` işareti açıkken `rotate(45deg)` (0.2s); içerik `padding: 0 18px 14px` + `fade-in 180ms`; tek-açık ve çok-açık modları

---

## 4. Diğer Bileşenler

- **Input/Textarea/Select:** label DM Mono 10/`0.12em`/uppercase/muted-fg üstte (gap 5); alan `padding:10px 12px` Jost 13, card zemin, `1px solid var(--border)`; **focus: border `var(--secondary)` + halo `0 0 0 3px rgba(166,124,82,0.15)`** (0.2s); hata: border+mesaj `#b03040`, mesaj DM Mono 10. Select: `appearance:none` + inline SVG ok (`right 12px center`). Search: sol `⌕` ikonu (`padding-left:36`), doluyken sağda × temizleme.
- **Checkbox/Radio:** 18×18, `2px solid` border → seçili primary dolu (checkbox radius 2 + beyaz ✓; radio daire + 8px iç nokta), 0.15s.
- **Toggle:** 44×24 ray radius 12, off `var(--muted)` / on primary (0.25s); 18px beyaz top `left 3↔22`, `transition: left 0.22s cubic-bezier(0.22,1,0.36,1)`.
- **Range:** native, `accentColor: var(--primary)`, altında DM Mono min/max, sağ üstte canlı değer (DM Mono, primary).
- **Badge (8 varyant):** `padding:3px 9px; borderRadius:2`; DM Mono 10/`0.1em`; default (primary dolu), success/warning/error/info (§1.2 üçlüsü), muted, outline, dot (6px accent nokta); removable ×. Sayı badge'i: 18px daire, primary, DM Mono 9, `top:-6 right:-8`.
- **Kartlar:** Swatch card — border'lı, hover `translateY(-3px)` + gölge `0 10px 28px` (0.25s), hover'da "Detay →" `opacity 0→1`; Stat card — `padding:18px 20px`, DM Mono 9 etiket / Playfair 30/700 primary değer / yeşil delta DM Mono 10 (**Audit-Liners panel KPI'ları için birebir**); Horizontal card (72px renk şeridi + içerik + sm butonlar); Feature card.
- **Tablo:** `borderCollapse:collapse`, Jost 13; thead card zemin + `2px solid var(--border)` alt çizgi, başlıklar DM Mono 9/`0.14em`/uppercase **tıklanır-sıralanır** (aktif kolon primary + ↑↓); hücre `padding:13px 14px`; satır ayracı `1px solid var(--muted)`; hover bg card; seçili satır `rgba(107,30,46,0.05)`; ID/miktar/tutar kolonları DM Mono (tutar primary/500); durum hücresi mini-badge; seçim → altta toplu-işlem çubuğu ("N seçili" + temizle). **Muhasebe fişi/mizan listesi için hazır reçete.**
- **Sekmeler (3 tip):** Line (alt `2px` ray, aktif primary alt çizgi + 600, uppercase Jost 13, `marginBottom:-2`); Pill (muted ray `padding:5 radius:4`, aktif bg background + `0 1px 4px` gölge + primary — görünüm değiştirici); Vertical (sağ kenar ray, aktif card zemin + sağda 2px primary çizgi). İçerik değişimi hep `fade-in 180ms`.
- **Navigasyon:** Breadcrumb (linkler secondary altı çizili → hover primary; ayraç `›` border renginde; son öğe muted-fg); Pagination (36px min kare butonlar, DM Mono, aktif primary dolu, uçlarda ←/→ disabled `opacity 0.4`); Steps (28px daire — bitti: primary+✓, aktif: secondary, bekleyen: muted; arada 60px×1px bağlantı çizgisi, biten kısmı primary — **denetim akışı adımlayıcısı için uygun**).
- **Sidebar (App kabuğu):** 200px, card zemin + sağ border, sticky; öğe `9px 20px` = DM Mono numara + Jost 12 etiket; aktif: `borderLeft: 2px solid var(--primary)` + bg `rgba(107,30,46,0.07)` + primary 600; hover bg `rgba(28,20,16,0.04)`; scroll-spy (IntersectionObserver `-30% 0px -60%`).
- **Scrollbar:** 5px, thumb `var(--border)` → hover muted-fg.

---

## 5. Etkileşim Dili (özet sözlük)

| Bağlam | Süre / easing |
|---|---|
| Mikro hover (renk, bg, border) | `0.15s`–`0.2s` (default ease) |
| Ghost hover / kart hover | `0.22s` / `0.25s` |
| Giriş animasyonları (fade-in) | `150–180ms ease-out` |
| Panel girişi (slide-up: modal/toast) | `250–260ms cubic-bezier(0.22,1,0.36,1)` ← **sistemin imza easing'i (ease-out-quint benzeri)** |
| Fiziksel tepkiler (press/emboss) | `70–90ms` (anlık his) |
| Tek seferlik keyframe'ler | ripple 550 · stamp 400 · shiver 450 · ink 640 · spin 700 linear |
| Toggle topu | `0.22s cubic-bezier(0.22,1,0.36,1)` |

Kural: **durum değişimi hızlı (≤200ms), sahneye giriş yumuşak yaylı (~250ms özel bezier), fiziksel bası anlık (<100ms).**

---

## 6. Önerilen Nihai Palet — Beyaz Tema (Audit-Liners uyarlaması)

Kullanıcı isteği: **beyaz tema = güven + temizlik.** Katalogdaki yapı korunur (bg→card→muted→border tek sıcak eksen; tek güçlü birincil; mono veri dili), yalnızca eksen bej'den beyaz-nötre kaydırılır.

### 6.1 Marka kırmızısı kararı: `#E23A32` yerine **`#6B1E2E` bordoya geçiş ÖNERİLİR** (veya ara ton)
- Mevcut `--brand #E23A32` parlak bir alarm kırmızısı; muhasebe/denetim ekranında **hata/borç/eksi ile karışır** (durum error `#b03040` ile neredeyse aynı aile). Birincil aksiyon = kırmızımsı olacaksa hata dilinden ayrışmalı.
- Katalogdaki `#6B1E2E` bordo: koyu, kurumsal, "mühür/mürekkep" hissi — güven mesajıyla ve beyaz zeminle kontrastı (≈9.4:1, AAA) mükemmel. Error `#b03040` ondan net ayrışır (parlaklık farkı büyük).
- **Öneri A (tercih):** `--primary: #6B1E2E`; `#E23A32` tamamen emekli edilir ya da yalnız hata vurgusu ailesine devredilir.
- **Öneri B (marka sürekliliği istenirse):** `--primary: #8C2430` gibi ara koyu kırmızı — E23A32'nin ailesinde ama ciddi; error yine `#b03040`.

### 6.2 Beyaz tema token seti (öneri)

```css
:root {
  --background: #FFFFFF;        /* saf beyaz sayfa */
  --foreground: #1C1917;        /* sıcak siyah (katalog #1C1410'un nötrlenmişi) */
  --card: #FAF9F7;              /* yüzey: beyazdan 1 ton sıcak-gri (katalog deseni: card ≠ bg) */
  --card-foreground: #1C1917;
  --primary: #6B1E2E;           /* bordo — §6.1 */
  --primary-foreground: #FFFFFF;
  --secondary: #8A7357;         /* sıcak kahve-gri (A67C52'nin beyaza uyarlanmışı, focus rengi) */
  --secondary-foreground: #FFFFFF;
  --accent: #A8861F;            /* altın — C9A84C beyaz üstünde soluk kalır, koyulaştırıldı */
  --accent-foreground: #FFFFFF;
  --muted: #F0EEE9;             /* pasif zemin */
  --muted-foreground: #6E6558;  /* ikincil metin (beyazda 4.5:1 üstü) */
  --border: #E2DED6;            /* katalog border'ı beyaz temada çok koyu; yumuşatıldı */
  --border-strong: #C9C2B4;     /* tablo başlık çizgisi / vurgulu kenar */
  --ring: rgba(138,115,87,0.35);
  --radius: 2px;                /* keskin köşe imzası korunur */
  /* durum renkleri — katalogdan aynen (beyaz temada da doğru çalışıyorlar) */
  --ok-bg:#e8f5e0; --ok-fg:#3a6b20; --warn-bg:#fef3cd; --warn-fg:#7a5a10;
  --err-bg:#fde8ea; --err-fg:#b03040; --info-bg:#e0edf8; --info-fg:#1e5a8a;
  --shadow-ink: 28,25,23;       /* gölgeler rgba(var(--shadow-ink),…) — sıcak gölge ilkesi korunur */
}
```

Notlar:
- Katalogda focus halkası `rgba(166,124,82,0.15)` — beyaz temada `rgba(138,115,87,0.18)` önerilir (aynı 0 0 0 3px deseni).
- Katalog border `#C4B89A` beyaz zeminde fazla sarı/koyu durur → iki kademeli border (normal + strong) daha temiz.
- Tipografi üçlüsü (Playfair/Jost/DM Mono) beyaz temada aynen çalışır; TR muhasebe bağlamında Playfair italic başlıklar "rapor/antet" havası verir — kurumsal PDF rapor kimliğiyle de uyumlu. Alternatif isterse gövdede Jost yerine mevcut sistem fontu kalabilir, ama **DM Mono'yu tutar/kod/tarih için almak net kazanım.**

---

## 7. En Değerli Bulgular (uygulama sırası önerisiyle)

1. **Üç-font rol ayrımı + DM Mono veri dili** — tutar, fiş no, tarih, hesap kodu için mono + letter-spacing: mizan/tablo okunurluğunu anında yükseltir.
2. **2px radius + card-zemin başlık şeridi + sıcak gölge** üçlüsü sistemin görsel imzası; az maliyetle "ciddi arşiv/defter" hissi.
3. **Tablo reçetesi** (DM Mono sıralanabilir başlık, 13px hücre ritmi, seçim + toplu işlem çubuğu, durum mini-badge) fiş listesi/mizan için hazır.
4. **Modal/Drawer/Toast tek animasyon dili** (fade 180ms + slide-up 250ms `cubic-bezier(0.22,1,0.36,1)`) — tüm açılır yüzeylerde tek easing kullan.
5. Buton gösterisinden üretime **4 varyant** damıt: Primary(ripple), Outline(stamp), Ghost, Destructive + ortak disabled `opacity 0.42` kuralı.
