# Frontend Dönüşüm Planı — Figma Katalog Dili ("Atelyé")

**Tarih:** 2026-07-18 · **Statü:** plan (kod değişikliği YOK)
**Kaynak katalog:** `scratchpad/figma-katalog/src/App.tsx` (1608 satır, 13 bileşen kategorisi) + `src/index.css` (78 satır token+keyframe)
**Hedef:** `web/src/styles.css` (230 satır) + `web/src/App.tsx` (2609 satır, 544 inline stil) + `HesapSecici/AciklamaSecici/CommandPalette/Login/Sidebar/Rehber`
**Bağlam:** routing göçü Adım 1-2 tamam (`frontend-routing-plani.md`); görsel format protokolü `gorsel-format-kutuphanesi.md` — bu doküman o protokole yeni kayıt olarak düşülmeli.

---

## 0. İki dilin özeti ve ana karar

| | Bizim mevcut dil | Katalog dili ("Atelyé") |
|---|---|---|
| Zemin | Soğuk gri-mavi (`#EEF1F5`), beyaz kart | Krem (`#F5F0E8`), bej kart (`#EDE7D9`) |
| Vurgu | Kırmızı `#E23A32` + mavi `#2563EB` | Bordo `#6B1E2E` + taba `#A67C52` + altın `#C9A84C` |
| Köşe | 8–16px yumuşak | **2px keskin** (tek `--radius`) |
| Font | IBM Plex Sans + IBM Plex Mono | **Playfair Display (başlık, italik) + Jost (gövde) + DM Mono (etiket/kod)** |
| Buton dili | Normal case, dolgun radius | **UPPERCASE + letter-spacing 0.06–0.12em**, keskin |
| Etiket dili | 12px sans pill (999px radius) | **10px DM Mono uppercase, 2px radius, kare** |
| Gölge | Yumuşak katmanlı (`--sh-*`) | Daha sert, `rgba(28,20,16,…)` sıcak ton |
| Dark mode | VAR (`prefers-color-scheme` + `data-theme`) | **YOK** |

**Ana karar (kullanıcı beyanı: "beyaz tema güven veriyor"):** Katalogdan **yapısal dili** alıyoruz (keskin 2px köşe, tipografi üçlüsü, uppercase buton/etiket ritmi, buton varyant sistemi, modal/drawer/toast desenleri, badge sistemi) ama **zemini krem değil beyaz tutuyoruz**. Krem `#F5F0E8` zemini birebir kopyalamak "beyaz tema" talebiyle çelişir.

**Karar noktası (kullanıcıya sorulacak, F1 öncesi):** marka kırmızısı `#E23A32` → bordo `#6B1E2E` swap'ı yapılacak mı?
- **Seçenek A (önerilen):** Bordo primary olur — katalog dilinin kimliği bordodur; kırmızı yalnız `--neg` (hata) olarak kalır. Denetim/güven algısına bordo daha uygun.
- **Seçenek B:** Kırmızı kalır, yalnız yapı (radius/font/uppercase) alınır. Daha az risk, ama katalog "hissi" yarım kalır.
Aşağıdaki eşleme tablosu Seçenek A'ya göre yazıldı.

---

## 1. Katman stratejisi — token eşleme tablosu

### 1a. Renk eşlemesi (beyaz-tema uyarlaması)

| Bizim token | Mevcut değer | Katalog karşılığı | Yeni değer (beyaz uyarlama) | Not |
|---|---|---|---|---|
| `--bg` | `#EEF1F5` | `--background #F5F0E8` | `#FAF9F7` | Kremin beyaza çekilmiş hali — sıcaklık kalır, beyaz kalır |
| `--bg2` | `#F6F8FA` | (yok) | `#F5F3EF` | Zemin-2, tablo hover/degrade |
| `--card` | `#fff` | `--card #EDE7D9` | `#FFFFFF` | **Kart beyaz kalır** (katalogdaki bej alınmaz) |
| `--border` / `--border2` / `--border3` | gri tonları | `--border #C4B89A` | `#E5E1D8` / `#EEEBE4` / `#D8D3C6` | Sıcak-gri kenar ailesi; katalog kenarının açılmışı |
| `--ink` | `#16191D` | `--foreground #1C1410` | `#1C1410` | Birebir — sıcak siyah |
| `--brand` | `#E23A32` | `--primary #6B1E2E` | `#6B1E2E` | **Seçenek A.** `--brand-dark: #521622`, `--brand-bg: #F4E9EB` |
| `--acc` | `#2563EB` | `--secondary #A67C52` | `#A67C52` | Mavi vurgu → taba; `--acc-bg: #F3EDE6` |
| (yok) | — | `--accent #C9A84C` | `--gold: #C9A84C` | YENİ token — vurgu ikonları, dot badge |
| `--nav` / `--lbl` / `--mut` / `--mut2` | soğuk griler | `--muted-foreground #6B5E4E` | `#5C534A` / `#6B5E4E` / `#7A6F62` / `#948A7C` | Sıcak nötr aile |
| `--pos/--ok` (+bg) | `#12885E` | success `#3a6b20 / #e8f5e0` | `#3a6b20` / `#EDF5E4` | Katalog başarı yeşili |
| `--neg` (+bg) | `#E5484D` | error `#b03040 / #fde8ea` | `#B03040` / `#FDE8EA` | Katalog hata kırmızısı (marka kırmızısının yerini alır) |
| `--warn` (+bg) | `#C8760A` | warning `#7a5a10 / #fef3cd` | `#7A5A10` / `#FEF3CD` | |
| (yok) | — | info `#1e5a8a / #e0edf8` | `--info: #1E5A8A` / `--info-bg: #E0EDF8` | YENİ — bilgi rozeti/toast |
| `--sh-sm/--sh/--sh-md/--sh-lg` | `rgba(16,24,40,…)` soğuk | xs/sm/md/lg `rgba(28,20,16,…)` | Katalog dört kademesi birebir | Gölge rengi sıcaklaşır |

### 1b. Radius / spacing / tipografi eşlemesi

| Boyut | Bizim (dağınık, literal) | Katalog | Yeni token |
|---|---|---|---|
| Buton/input radius | 8–10px literal | 2px | `--r: 2px` (tek token; styles.css'teki TÜM literal radius'lar buna bağlanır) |
| Kart radius | 12–18px literal | 0–2px | `--r-card: 2px` (ayrı token: geri adım gerekirse tek satır) |
| Tam yuvarlak (avatar, dot) | 50% / 999px | 50% | değişmez |
| Spacing | 4/8/12/16/24 fiili | 4/8/12/16/24/32/48/64 | aynı ölçek, token'a almak F1'de şart değil |
| Başlık fontu | IBM Plex Sans 700 | **Playfair Display** (600–700, çoğu italik) | `--font-display` — h1, `.card-title`, modal başlık, KPI başlıkları |
| Gövde | IBM Plex Sans | **Jost** (400/500) | `--font-body` — body'e |
| Mono/etiket | IBM Plex Mono | **DM Mono** (uppercase + tracking) | `--font-mono` — `.mono .num .kpi-val`, th, badge |

**Font notu (Tauri):** katalog Google Fonts CDN import'u kullanıyor; masaüstü (offline) için üç font **self-host** edilmeli (`web/public/fonts/` + `@font-face`). CDN import kabul edilmez.

**Dark mode notu:** katalogda dark yok. F1'de dark bloğu **silinmez**; sıcak-nötr karşılıklarla kabaca güncellenir (`--bg #17130F`, `--card #211B15`, bordo aynı kalır) ama dark cilası bilinçli olarak F5 sonrasına ertelenir. Beyaz tema esas.

---

## 2. Bileşen eşleme tablosu

### 2a. Butonlar

| Katalog varyantı | Bizim mevcut sınıf | Karar |
|---|---|---|
| **BtnRipple primary** (bordo dolgu, uppercase, tracking) | `.primary` (13 kullanım) | **Birebir yeniden stillenir.** Ripple JS'i alınmaz (F2'de CSS-only; istenirse F5'te 10 satırlık ripple util) |
| **BtnRipple secondary** (taba dolgu) | `.btn-dark` (6 kullanım) kısmen | `.btn-dark` → koyu mürekkep butonu olarak kalır (aşağıda Ink); taba dolgu için **YENİ `.sekonder`** — şimdilik başlıktaki "Yeni Kayıt (N)" adayı |
| **BtnGhost** (kenarlıklı, hover'da bordo + underline) | `.btn` (21) ve `.pill-tab` (pasif hali, 21) | **Birebir yeniden stillenir** — `.btn` ghost diline çekilir |
| **BtnStamp** (outline→hover'da dolgu) | karşılığı yok | **YENİ `.btn-cerceve`** — ikincil onay aksiyonları ("TXT indir", "Devri getir") |
| **BtnInk** (sıcak-siyah, DM Mono) | `.btn-dark` | **Birebir yeniden stillenir** (`.btn-dark` = Ink görünümü; ink-expand animasyonu opsiyonel) |
| **Destructive** (kırmızı dolgu, ConfirmModal'daki) | yok — bugün `pill-tab` + inline `color: var(--neg)` (App.tsx:1931) | **YENİ `.btn-tehlike`** — "Fişi iptal et" gibi kalıcı-etki aksiyonları buna geçer |
| **BtnLoading** (spinner + disabled) | yok | **YENİ `.yukleniyor`** modifier + `spin` keyframe — kaydet/beyanname üret akışları |
| **BtnPress / BtnShiver / BtnEmbossed** | — | **Alınmaz.** Muhasebe aracına oyunbaz animasyon dili uymaz; karar kaydı olarak buraya yazıldı |
| **Button group** (bitişik segment) | yok — `pill-tab` çiftleri (daha eski/daha yeni, App.tsx:1923-24) | **YENİ `.btn-grup`** — sayfalama çiftleri buna geçer |
| **Split button** | yok | Alınmaz (ihtiyaç yok) |
| Size scale sm/md/lg | tek boy | `.kucuk` modifier yeter (tablo içi butonlar); üç boy sistemi gereksiz |

### 2b. Rozet / çip / sekme

| Katalog | Bizim | Karar |
|---|---|---|
| Badge success/warning/error/info/muted (DM Mono 10px uppercase, kare) | `.pill` (+`.ok .warn .neg .acc .brand`, 28 kullanım) | **Birebir yeniden stillenir:** 999px hap → 2px kare, mono uppercase. `info` varyantı eklenir |
| Badge outline / dot | `.rozet` (3) | `.rozet` → outline badge; dot varyantı `--gold` noktalı YENİ `.rozet.canli` |
| Removable tag | yok | Şimdilik alınmaz (filtre çipleri gelince) |
| Sayı badge (bildirim) | `.item .badge` (sidebar) | F5'te katalog konumlandırmasıyla stillenir |
| **TabsPill** (bej konteyner içinde, aktif = beyaz + gölge) | `.pills` + `.pill-tab.akt` (aktif = kırmızı dolgu) | **Desen değişir:** aktif sekme dolgu yerine "beyaz kart + gölge + bordo yazı" olur — `.pills` konteyner arka planı `--bg2`. 21 kullanım tek CSS'ten döner |
| **TabsLine** (alt çizgili) | yok | **YENİ `.sekme-cizgi`** — UFRS ana sekmeleri (`ufrsSekme`) ile Muhasebe alt sekmeleri buna terfi edebilir (F5) |
| TabsVert | yok | Alınmaz (sidebar zaten bu işi görüyor) |

### 2c. Açılır sayfa / katman desenleri

| Katalog | Bizim | Karar |
|---|---|---|
| **Modal** (sm/md/lg, bej başlık şeridi, keskin köşe, `fade-in` + `slide-up`) | `.modal-bg .modal .modal-hd .modal-x .modal-body` (fiş detay) + `CommandPalette` aynı `.modal-bg`'yi kullanıyor | **Birebir yeniden stillenir:** radius 16→2, başlığa `--bg2` şerit + Playfair, `slide-up` animasyonu eklenir. Boyut için `.modal.sm/.lg` modifier |
| **ConfirmModal** (destructive onay) | yok — bugün `confirm()`/doğrudan buton | **YENİ desen:** `.btn-tehlike` + sm modal; "fiş iptal", "dönem kesinleştir" onayları buna geçer (F3'te CSS, akış bağlama F5) |
| **Drawer** (sağ/sol/alt panel) | yok — en yakın: kebir/muavin hesap detayı `card` içinde inline açılıyor | **YENİ `.cekmece`** (sağ, 380–480px) — hedef kullanım: fiş önizleme, kebir hesap detayı, UFRS kayıt detayı. F3'te CSS+bileşen, ekran bağlama F5 |
| **Split/combo dropdown** | `.combo/.combo-btn/.combo-pop/.combo-opt` (HesapSecici, AciklamaSecici, App başlık mükellef menüsü) | **Birebir yeniden stillenir:** radius 12→2, sıcak gölge, `fade-in`. HesapSecici.tsx **dokunulmaz** — tamamı sınıftan döner (içindeki 8 inline stil layout, zararsız) |
| **Toast** (sağ-alt yığın, 4 tip, otomatik kapanma) | `.msg` (3 kullanım, statik satır) | **YENİ `.toast` sistemi** — kayıt başarılı/hata geri bildirimi. CSS F3'te; App'e küçük `useToast` benzeri yığın F5'te. `.msg` sayfa-içi kalıcı uyarı olarak kalır |
| **Accordion** | yok — `AciklamaPanel` benzer iş görüyor | F5'te değerlendirilir, zorunlu değil |
| Command palette | `.palet` (bizde var, katalogda yok) | Bizim desen korunur, sadece token'lardan yeni dili giyer |
| Steps / Breadcrumb / Pagination | yok / yok / `pill-tab` çiftleri | Steps: beyanname sihirbazı gelirse. Breadcrumb: routing Adım 3 sonrası anlamlı. Pagination → `.btn-grup` (2a) |

### 2d. Tablo, form, kart

| Katalog | Bizim | Karar |
|---|---|---|
| Tablo başlığı (DM Mono 9px, tracking 0.14em, uppercase, sıralama oku) | `th` zaten uppercase 11px | Yakın akraba — mono'ya çekilir, tracking artar. Sıralama/checkbox deseni F5+ |
| Satır hover bej / seçili bordo-şeffaf | `tbody tr:hover` var | Token swap'la kendiliğinden gelir |
| Input (kare, taba focus halkası, mono uppercase label) | `input/select/textarea` + `label` | **Birebir yeniden stillenir:** radius 9→2, focus `--ink` halka → taba `rgba(166,124,82,.15)` halka; `label` DM Mono uppercase olur |
| Hata durumu (kırmızı kenar + mono alt yazı) | `.giris-hata` benzer | Genel `.girdi-hata` sınıfına terfi (F5) |
| Toggle / Range / Checkbox-Radio (custom) | native yok denecek kadar az | İhtiyaç düştükçe F5 |
| **StatCard** (mono etiket + Playfair 30px değer) | `.kpi/.kpi-lbl/.kpi-val` (9 kullanım) | Etiket mono-uppercase olur; **değer Playfair'e ÇEVRİLMEZ** — parasal değerler `DM Mono` kalır (hizalı rakam muhasebede vazgeçilmez). Bilinçli sapma, karar kaydı |
| FabricCard hover kalkışı (`translateY(-3px)`) | `.card:hover` gölge var | Tıklanabilir kartlara (`tr.clickable` benzeri) hafif kalkış eklenir |
| Feature card | `.aciklama` | Token'lardan döner |

---

## 3. Kademeli fazlar (her faz tek başına gemiye biner)

### F1 — Token + tipografi swap'ı (görsel tazelenme, sıfır davranış değişikliği)
- **İş:** §1a-1b tablosundaki değerleri `styles.css :root`'a yaz; `--r/--r-card/--gold/--info` token'larını ekle; styles.css içindeki **tüm literal radius/renkleri** token'a bağla; üç fontu self-host edip `@font-face` + body/h/mono ataması; dark bloğuna sıcak-nötr kaba karşılıklar; **App.tsx'teki hardcoded hex temizliği** — inline stillerdeki `#F8F9FB`(5), `#F1F3F5`(4), `#FFF4DC`, `#F0F6FF`, `#FFFDF4`, `#EEF3F8` → `var(--bg2)` vb. (styles.css'te de aynı hex'ler var: 60, 105, 147, 149, 155, 161, 177, 184, 186, 188-190, 211, 218).
- **Dosyalar:** `web/src/styles.css` (esas), `web/index.html` (font preload), `web/public/fonts/` (yeni), `web/src/App.tsx` (yalnız hex→var bul-değiştir, ~25 nokta).
- **Risk:** ORTA. (a) Dark mode kontrastı bozulabilir — kabul edilir, beyaz esas; (b) fontlar ölçü değiştirir (Jost, Plex'ten dar) → satır kaymaları; (c) hex→var değişimi 544 inline stilin yalnız hex'li kısmına dokunur, gerisi `var()` kullandığı için kendiliğinden döner.
- **Doğrulama (tarayıcı listesi):** ① Login → giriş; ② Dashboard KPI'lar + kartlar; ③ Muhasebe/yeni fiş formu + HesapSecici aç; ④ Mizan tablosu (hizalı rakamlar, `.num` sağa dayalı mı); ⑤ Denetim çalışma kağıdı `xgrid` (ov/man/edit hücre renkleri hâlâ ayırt edilir mi); ⑥ UFRS WTB; ⑦ ⌘K palet; ⑧ fiş detay modalı; ⑨ dark toggle'da okunabilirlik (kusursuzluk aranmaz); ⑩ Tauri build'de fontların offline yüklendiği.

### F2 — Buton / çip / rozet sınıfları
- **İş:** §2a-2b kararları: `.primary .btn .btn-dark .pill-tab .pill .rozet .ikon-btn .chip` yeniden stillenir (uppercase + tracking + 2px + katalog etkileşim durumları); YENİ `.btn-cerceve .btn-tehlike .btn-grup .yukleniyor .sekonder` sınıfları eklenir; `pill-tab.akt` segmented-control desenine döner; App.tsx'te yalnız `.btn-tehlike`/`.btn-grup`'a geçmesi gereken ~5 nokta (1923-24, 1931 vb.) className değişir.
- **Dosyalar:** `styles.css`, `App.tsx` (~5 satır className).
- **Risk:** DÜŞÜK-ORTA. Uppercase Türkçe'de uzar ("KESİNLEŞTİR") → dar butonlarda taşma; `text-transform: uppercase` TR locale'de `i→İ` doğru çalışır ama genişlik denetlenmeli. UFRS içi mini `pill-tab`'lar (inline `padding: "1px 8px"` override'lı, App.tsx:2033-2048) yeni stille çakışabilir — bu inline'lar F4'te sınıfa alınana dek gözle kontrol.
- **Doğrulama:** her ekranda buton turu — özellikle Muhasebe sekme şeridi, Denetim kağıt aksiyon şeridi (1841-1847), UFRS ref rozetleri (2033-2048), hover/active/disabled/focus-visible dört durum.

### F3 — Açılır katman sistemi (modal / drawer / combo / toast)
- **İş:** §2c: `.modal*` yeniden stillenir + `sm/lg` modifier + `fade-in/slide-up` keyframe'leri index.css'ten alınır; `.combo-pop .palet` aynı dile çekilir; YENİ `.cekmece` ve `.toast` CSS'i + iki küçük yeniden kullanılabilir bileşen (`Cekmece.tsx`, `Toast.tsx`) yazılır ama **henüz hiçbir ekrana bağlanmaz** (CSS+bileşen gemiye biner, davranış değişmez).
- **Dosyalar:** `styles.css`, yeni `web/src/Cekmece.tsx` + `web/src/Toast.tsx`; `HesapSecici.tsx/AciklamaSecici.tsx/CommandPalette.tsx` dokunulmaz.
- **Risk:** DÜŞÜK. Animasyonlar eklenirken `prefers-reduced-motion` gözetilmeli; `.modal-bg`'yi CommandPalette de kullanıyor (inline `alignItems/paddingTop` override) — palet regresyonu kontrol.
- **Doğrulama:** fiş detay modalı aç/kapa + Esc; HesapSecici klavye gezinme (↑↓/Enter/Esc) bozulmadı mı; ⌘K palet konumu; mükellef menüsü (App.tsx:866, sağa dayalı combo-pop).

### F4 — UFRS ekranı inline stillerinin sınıfa taşınması (pilot ekran)
- **İş:** En yoğun ve en stratejik ekran UFRS (WTB + çalışmalar, App.tsx ~1985-2400). Tekrarlayan inline desenler ortak sınıfa alınır: `style={{paddingLeft:20}}`(24×) + `paddingRight:20`(21×) → `.ic-pad`; `fontSize:11.5, color: var(--mut)`(11×) ve `--mut2`(10×) → `.alt-not` / `.ipucu`; `display:flex, gap:8, alignItems:center`(7×+) → `.satir-flex`; `padding:0, overflow:hidden`(10× kart) → `.card.sifir-pad`; başlık şeridi `padding:"16px 20px", borderBottom`(4×) → `.card-hd`. UFRS içindeki mini pill-tab inline override'ları F2'deki `.kucuk` modifier'a geçer.
- **Dosyalar:** `App.tsx` (UFRS bloğu), `styles.css` (~8 utility sınıf). Routing Adım 7 UFRS'yi en son taşıyacağı için bu temizlik **dosya bölünmesinden önce** yapılmalı — taşınan kod temiz taşınır.
- **Risk:** ORTA. Bul-değiştir sırasında tek tük özel değer (12.5 vs 11.5) yanlış sınıfa yuvarlanabilir; ekran yoğun, satır satır diff okuması şart.
- **Doğrulama:** WTB kolon hizaları, AJE/RJE/CF ref rozetleri, çalışma (tms16 vb.) aç-doldur-kaydet turu, kesinleştirilmiş dönem kilit görünümü.

### F5 — Kalan ekranlar + yeni desenlerin bağlanması
- **İş:** (a) F4 utility'leri Muhasebe/Vergi/Denetim/Analiz/Dashboard'a yayılır (kalan ~300 inline stilin tekrarlı olanları); (b) `.cekmece` fiş önizleme + kebir hesap detayına bağlanır; (c) `.toast` kaydet/hata akışlarına bağlanır; (d) `.btn-tehlike` + sm-modal onayı "fiş iptal"/"kesinleştir"e bağlanır; (e) `.sekme-cizgi` UFRS ana sekmelerine denenir; (f) Login/Sidebar/Rehber cila; (g) dark mode sıcak paletle ciddi biçimde elden geçirilir.
- **Dosyalar:** `App.tsx` (geniş), `Sidebar.tsx`, `Login.tsx`, `Rehber.tsx`, `styles.css`.
- **Risk:** ORTA — geniş yüzey; alt maddeler (a)–(g) ayrı ayrı gemiye binebilir, tek PR yapılmamalı. Routing Adım 3+ ile çakışmaması için koordinasyon: **önce hangi ekran route'a taşınacaksa onun F5 cilası taşınmadan yapılmaz** (çifte iş).
- **Doğrulama:** tam uygulama turu (10 ana ekran) + Tauri build + dark mod kontrol listesi.

---

## 4. Inline stil gerçeği (dürüst değerlendirme)

`App.tsx`'te **544 inline stil** var. Token swap (F1) bunların **`var(--…)` kullananlarını kendiliğinden dönüştürür** (ör. `color: "var(--mut)"` 20×, `var(--mut2)` vb. — çoğunluk böyle). Swap'ın **etkilemediği** ve öncelikle sınıfa taşınması gerekenler:

1. **Hardcoded hex zeminler** (F1'de çözülür): `#F8F9FB`, `#F1F3F5`, `#FFF4DC`, `#F0F6FF`, `#FFFDF4`, `#EEF3F8` — swap sonrası "eski soğuk gri" adacıkları olarak sırıtır. En yüksek öncelik.
2. **Inline `borderRadius` literal'leri** — keskin-köşe diline geçince yumuşak kalan köşeler tutarsızlık yaratır (F2/F4'te ilgili bileşenle birlikte).
3. **Tekrarlayan tipografi/padding desenleri** (F4 listesi) — 155× `.num` zaten sınıf; sorun değil.
4. **Saf layout inline'ları** (`flex/gap/width/flex:1`) — tasarım diline etkisiz, **taşınmaz**; taşımak maliyet üretir, değer üretmez.

---

## 5. Özet

**Eşleme özü:** beyaz zemin korunur, katalogdan bordo primary + taba sekonder + altın vurgu + 2px keskin köşe + Playfair/Jost/DM Mono üçlüsü + uppercase buton/rozet ritmi alınır; `.primary→Ripple-primary`, `.btn→Ghost`, `.btn-dark→Ink`, `.pill→kare mono badge`, `.pill-tab.akt→segmented`, `.modal→katalog modal`, `.combo-pop→keskin dropdown`; YENİ: `.btn-cerceve .btn-tehlike .btn-grup .yukleniyor .cekmece .toast .sekme-cizgi`. Press/Shiver/Embossed/Split alınmaz; KPI değerleri mono kalır (Playfair'e çevrilmez).

**Fazlar:**
- **F1** — token/font swap + hex→var temizliği (styles.css + App hex'leri; davranış sıfır değişir)
- **F2** — buton/çip/rozet sınıfları + yeni buton varyantları (~5 className değişimi)
- **F3** — modal/combo restyle + Cekmece/Toast CSS+bileşen (henüz bağlanmaz)
- **F4** — UFRS pilot: inline desenler → ~8 utility sınıf
- **F5** — kalan ekranlar + yeni desenlerin akışlara bağlanması + dark cila (alt-maddeler ayrı ship)

**En büyük 2 risk:**
1. **Dark mode belirsizliği** — katalogda dark dili hiç yok; bordo/krem ailesinin karanlık karşılığı tasarlanmamış. F1'de kaba karşılık verilip cila F5'e ertelenir; aradaki sürümlerde dark kullanıcıları görsel pürüz görür.
2. **544 inline stil nedeniyle "yarım dönüşüm" görünümü** — F1 sonrası hardcoded hex/radius'lu adacıklar eski dilde kalır; hex temizliği F1'e dahil edilerek ve UFRS pilotu (F4) erken alınarak sınırlandırılır. Ayrıca routing göçüyle (Adım 3+) çakışma: aynı ekrana iki ayrı refactor aynı anda girmemeli.
