# Görsel Format Kütüphanesi — Kendini Güncelleyen Kayıt

> **PROTOKOL (kalıcı kural):** Bu dosya, projede kullanılan HER görsel/tasarım formatının, bileşeninin
> ve erişim yönteminin canlı kaydıdır. **Yeni bir format, kaynak veya erişim yöntemi kullanıldığında
> BU DOSYA GÜNCELLENİR** — hangi format, nereden alındı, nasıl erişildi, hangi çalışmada kullanıldı.
> Amaç: bir dahaki sefere sıfırdan aramak yerine, öğrenilmiş yöntemi tekrar kullanmak.
>
> Claude için talimat: UFRS worksheet / dashboard / herhangi bir ekran tasarlarken, önce bu dosyaya
> bak (uygun bir desen var mı?). Yeni bir kaynak/desen kullanırsan, işin sonunda buraya ekle.

**Tarih:** 2026-07-15 · **Statü:** araştırma tamamlandı, kaynak envanteri dolduruldu

**Projenin stack kısıtı (kritik filtre):** `web/` = React 18 + Vite + TypeScript, **Tailwind YOK, saf CSS
değişkenleri** (`--mut`, `--border2`, `--bg2`, kırmızı `#E23A32`). Kaynakları iki gruba ayırıyoruz:
**(A) doğrudan kullanılabilir** (headless / stilsiz / kendi CSS'i) ve **(B) sadece görsel referans**
(Tailwind gerektirir → deseni kopyala, kodu değil).

---

## 1. Erişebildiğimiz format kaynakları (araç envanteri)

| Kaynak | Erişim yöntemi | Ne için | Durum |
|---|---|---|---|
| **Figma MCP** (bu oturuma BAĞLI ✅) | ToolSearch `select:mcp__855e7670…__<tool>`: whoami, get_design_context, get_screenshot, get_metadata, get_variable_defs, search_design_system, use_figma | Figma dosyasından tasarım bağlamı / ekran görüntüsü / renk-spacing token'ı çekmek | ✅ bağlı — hesap `ardakarabulut96`, **starter / View koltuğu**. View koltuğu OKUMAYA yeter; Dev Mode kod çıkarımı + rate limit sınırlı (bkz. §5). Şu an bağlı dosya YOK — kullanıcı frame URL'i verince çekilir |
| **Figma Community** | community.figma.com — ücretsiz hesapla "Duplicate" | Finans/dashboard/tablo/form UI kit'leri, design system | ✅ erişilebilir |
| **GitHub — açık kaynak repolar** | WebSearch/WebFetch + dokümantasyon | TanStack Table, Recharts, AG Grid, Tremor, shadcn/ui | ✅ erişilebilir |
| **dataviz skill** | `Skill(dataviz)` | Grafik/tablo/stat-tile renk sistemi, erişilebilir palet (light+dark) | ✅ mevcut |
| **artifact-design skill** | `Skill(artifact-design)` | Tek sayfalık görsel artefakt tasarımı | ✅ mevcut |
| **theme-factory skill** | `Skill(anthropic-skills:theme-factory)` | Hazır 10 tema (renk/font) | ✅ mevcut |
| **web-artifacts-builder** | `Skill(anthropic-skills:web-artifacts-builder)` | React + Tailwind + shadcn/ui karmaşık artefakt (prototip) | ✅ mevcut |

---

## 2. Projede benimsenen tasarım dili (mevcut — değişecek)

**Şu an:** tek sayfa, state-based navigasyon (`gorunum` state), URL routing YOK.
Bileşenler: `card`, `pill (ok/warn)`, `pill-tab`, `primary`, `.num/.mono`, `HesapSecici (combo)`, modal.
CSS değişkenleri: `--mut`, `--mut2`, `--border2`, `--bg2`. Kırmızı vurgu `#E23A32`.

**Kullanıcı geri bildirimi (2026-07-15):** "tasarımlar çok yetersiz ve tek düze, kullanıcılar için
eziyet." → Kökten yenileme kararı: URL routing + her ekrana ayrı sayfa + zengin görselleştirme.

---

## 3. FIGMA kaynakları

### 3a. Figma Community UI kit'leri (görsel referans + duplicate)
Ücretsiz hesapla açılıp "Duplicate" ile kendi taslaklarına kopyalanır. **Lisans dosya başına değişir** —
her dosyanın açıklamasındaki "Free for personal and commercial use" ibaresini teyit et.

| Kit | Link | Ne işe yarar | Lisans notu |
|---|---|---|---|
| **Untitled UI — FREE v2.0** | [community](https://www.figma.com/community/file/1020079203222518115/untitled-ui-free-figma-ui-kit-and-design-system-v2-0) | En bilinen ücretsiz design system; 100+ bileşen, Auto Layout + Variants, tipografi/renk token'ları. Tasarım dilimizi (kart/tablo/form) oturtmak için **referans sistem** | Ücretsiz sürüm ticari kullanıma açık; PRO ücretli |
| **Free Finance UI Kit** | [community](https://www.figma.com/community/file/1259559940863317584/free-finance-ui-kit) | Banka/finans ekranları, KPI kartları, işlem listeleri | Freebie |
| **Fintech UI Kit (Free)** | [community](https://www.figma.com/community/file/1212747172059114028/fintech-ui-kit-free-version) | Finans/yatırım/bankacılık; design system + variants | Freebie |
| **Finance Dashboard UI Kit (Paperpillar)** | [community](https://www.figma.com/community/file/1401087188287685262/finance-dashboard-ui-kit-by-paperpillar) | Bütçe/gelir-gider dashboard, pie/bar grafik + aktivite listesi | "personal or commercial" |
| **Fintech Dashboard UI Kit** | [community](https://www.figma.com/community/file/1370009358305246750/fintech-dashboard-ui-kit-community) | Yoğun finans dashboard düzenleri | Community |
| **Figma UI kit – Finance Dashboard** | [community](https://www.figma.com/community/file/1152498424680432193/figma-ui-kit-finance-dashboard) | Dashboard grid + kart düzenleri | Community |
| **Tüm UI kit'ler (arama)** | [community/ui-kits](https://www.figma.com/community/ui-kits) | 4.700+ ücretsiz kit | — |

> **Denetim/muhasebe uyarısı:** Bu kitlerin çoğu tüketici-fintech (bütçe app) estetiğinde — renkli, boşluklu.
> Bizim ihtiyacımız **veri-yoğun tablo** (mizan, çalışma kağıdı). Figma kitlerini KART/FORM/KPI şeridi için
> referans al; büyük tablolar için §4 kod kütüphaneleri (CaseWare deseni) daha isabetli.

### 3b. Figma → koda çevirme yolları
1. **Dev Mode MCP (bu oturumdaki `figma` sunucusu):** Masaüstü uygulamasında dosya/seçim açıkken
   `get_screenshot` (görsel), `get_design_context`/`get_metadata` (yapı), `get_variable_defs` (renk/spacing
   token'ı). **Kısıt:** tam design-to-code = **Dev/Full koltuk + ücretli plan** (Professional/Org/Enterprise).
   Bizimki **View / starter** → düşük rate limit, kod çıkarımı sınırlı ([rate limits](https://developers.figma.com/docs/figma-mcp-server/rate-limits-access/)).
2. **Figma Dev Mode (uygulama içi, MCP'siz):** Öğeye tıkla → sağ panelde CSS/spacing oku → elle saf CSS'e
   taşı. Koltuk kısıtından bağımsız; **bizim için en pratik yöntem.**
3. **Plugin'ler:** "Figma to Code (HTML/Tailwind)", "Anima" — Tailwind/HTML üretir; saf CSS'imize birebir
   uymaz, sadece ölçü/renk çıkarımı için.

---

## 4. GITHUB açık kaynak kütüphaneler (React + Vite)

### GRUP A — Doğrudan kullanılabilir (Tailwind gerektirmez, saf CSS'imizle uyumlu) ✅

| Kütüphane | Lisans | Ne için | Uyum |
|---|---|---|---|
| **TanStack Table** ([tanstack.com/table](https://tanstack.com/table/latest) · [column-pinning-sticky örneği](https://tanstack.com/table/v8/docs/framework/react/examples/column-pinning-sticky)) | **MIT** | **Headless** tablo motoru: sıralama, filtre, gruplama, agregasyon, satır genişletme (drill-down), kolon sabitleme/pinning, sanallaştırma. Stil YOK — matematiği/state'i verir, HTML+CSS bize kalır | ★★★ **En iyi eşleşme.** Mizan/çalışma kağıdı için ideal; `.card`/`.num`/sticky CSS'imizle giydiririz |
| **Recharts** ([recharts.org](https://recharts.org)) | **MIT** | SVG grafikler: bar/line/area/waterfall/donut. Tailwind gerektirmez, bağımsız React bileşenleri | ★★★ VUK→TFRS köprü grafiği, KPI trendleri, düzeltme dağılımı |
| **AG Grid Community** ([github](https://github.com/ag-grid/ag-grid) · [community-vs-enterprise](https://www.ag-grid.com/react-data-grid/community-vs-enterprise/)) | **MIT** (community, üretim serbest) | Excel benzeri grid: hücre düzenleme, kolon virtualization, tema CSS'i dahil, klavye nav., ARIA | ★★ Çok satırlı mizan için güçlü; kendi temasını getirir (görsel dilimizle çakışabilir). Row grouping/pivot = Enterprise (ücretli) |
| **Glide Data Grid** ([github](https://github.com/glideapps/glide-data-grid)) | **MIT** | Canvas tabanlı, on binlerce satırı akıcı çizen spreadsheet grid | ★★ Devasa yevmiye/kebir listeleri için performans opsiyonu |

### GRUP B — Sadece görsel referans (Tailwind + Radix tabanlı; deseni kopyala, kodu değil) ⚠️

| Kütüphane | Lisans | Neden sadece referans | Not |
|---|---|---|---|
| **shadcn/ui** ([ui.shadcn.com](https://ui.shadcn.com)) | **MIT** | Kopyala-yapıştır ama **Tailwind + Radix** varsayar | Bileşen anatomisi (data-table, dialog, tabs) için **altın referans**; shadcn'in TanStack Table data-table reçetesini saf CSS'e uyarlayabiliriz |
| **Tremor** ([tremor.so](https://www.tremor.so/) · [npm](https://www.npmjs.com/package/@tremor/react)) | **MIT** (30+ açık bileşen) | Recharts + Radix üzerine ama **Tailwind zorunlu** | KPI kart + dashboard estetiği çok iyi. Tailwind'e geçersek A grubuna taşınır; şimdilik **düzen referansı** |

> **Karar (mevcut haliyle):** Tailwind eklemeden ilerliyoruz → **TanStack Table + Recharts** çekirdek ikili.
> shadcn/ui ve Tremor'u **görsel/anatomi referansı** olarak kullan, sınıfları saf CSS'imize çevir.

---

## 5. MUHASEBE / DENETİM YAZILIMI UI DESENLERİ (ürün referansları)

**CaseWare Working Papers** ([WTB/Report worksheet dok.](https://documentation.caseware.com/2022/WorkingPapers/en/Content/Engagements/Trial-Balance/Accounts-Balances/Report-Worksheet.htm) · [ürün](https://www.caseware.com/products/working-papers)) — UFRS worksheet'imizin birebir muadili:
- **Working Trial Balance (WTB) düzeni:** satır = hesap; kolonlar = *Açılış · VUK Bakiye · Düzeltme (AJE) ·
  Yeniden Sınıflama (RJE) · Nihai (TFRS) Bakiye · Referans*. **Tam olarak bizim dönüşüm tablomuz.**
- **Kolon yönetimi:** kullanılmayan kolonu gizle, kolonları yeniden sırala (sağ tık → Reorder), mantıksal sırala.
- **Lead schedule ↔ WTB ↔ finansal tablo bağlantısı:** düzeltme bir yerde girilince zincirde otomatik akar
  (bizim AJE/RJE → mizan → rapor akışının aynısı). Manuel mutabakatı azaltır.
- **Drill-down:** bakiyeye tıkla → o hesabın hareketleri / bağlı düzeltme fişi.

**Genel muhasebe SaaS (Xero / QuickBooks / ERP) desenleri** (bkz. `frontend-yeniden-tasarim.md`):
- **Sol navigasyon + içerik + üst global bar** üçlüsü; mükellef = birincil bağlam (workspace switcher, sol üst).
- **Sticky başlık** (`position:sticky`), satır 40px (yoğun)/48px (rahat), **metin sola / sayı sağa / durum
  rozeti ortala**, negatif tek kural (renk + işaret).
- **Progressive disclosure:** üstte 4-6 KPI şeridi → tıkla → detay tablo.

---

## 6. ERİŞİM REÇETESİ (nasıl ulaşırım — tekrar için)

**Figma Community kiti almak:**
1. §3a linkini aç (ücretsiz hesap `ardakarabulut96@gmail.com`).
2. Sağ üst "Duplicate" → kendi taslaklarına kopyalanır; dosya açıklamasındaki lisansı teyit et.
3. Öğeye tıkla → Inspect: renk/spacing/tipografi ölç → saf CSS'imize taşı.

**Figma MCP kullanmak (bu oturum):**
1. `ToolSearch("select:mcp__855e7670-3f1d-42c7-88e8-8c941c02a8a2__get_screenshot,…__get_design_context,…__get_metadata,…__get_variable_defs")`.
2. Kullanıcıdan Figma frame/dosya **URL'i** al (veya masaüstü uygulamasında seçim yap).
3. `get_screenshot` → görsel; `get_variable_defs` → token; `get_design_context` → yapı.
4. **Kısıt (2026-07-15 teyitli):** `whoami` = View/starter. Kod çıkarımı + rate limit sınırlı; ağır iş için
   Dev/Full koltuk gerekir. Pratik yol: **Inspect ile elle CSS çıkarımı** (§3b-2).

**GitHub kütüphanesi eklemek (TanStack/Recharts):**
```bash
cd web && npm i @tanstack/react-table recharts   # ikisi de MIT, Tailwind gerektirmez
```
Sonra `App.tsx` içindeki `.card`/`.num`/`.mono`/sticky CSS ile giydir. **Tailwind ekleme.**

**shadcn/Tremor'dan desen almak (kod değil, anatomi):** [ui.shadcn.com](https://ui.shadcn.com) data-table
sayfasını aç → kolon tanımı/sıralama/pinning mantığını oku → TanStack Table + saf CSS'e uyarla.

---

## 7. Audit-Liners için önerilen İLK 3 DESEN (çalışma kağıdı sayfasına)

Hedef ekran: **UFRS WorkSheet** (VUK mizanı → TFRS dönüşümü; her TMS/TFRS için AJE/RJE).

**1) Working Trial Balance grid (CaseWare deseni · TanStack Table + saf CSS)** — ★ birincil.
Satır = hesap (sınıfa göre gruplu, katlanabilir); kolonlar: `Hesap Kodu/Adı · VUK Bakiye · AJE(+/−) ·
RJE(+/−) · TFRS Bakiye · Ref (ULID)`. **Sticky başlık + sticky ilk kolon**, sayılar sağa (`.num/.mono`),
negatif tek kural. Satırı genişlet → o hesabın AJE/RJE fiş satırları (drill-down). Denge kontrol satırı altta
sabit. → *Kaynak: TanStack Table (MIT) + CaseWare WTB düzeni.*

**2) VUK→TFRS köprü (waterfall) grafiği (Recharts)** — ★ görselleştirme.
Tablonun üstünde `VUK Bakiye → +AJE → +RJE → TFRS Bakiye` şelale/köprü grafiği. Standart bazında hangi
düzeltmenin etkisi ne kadar — tek bakışta okunur. → *Kaynak: Recharts (MIT). Renkler: dataviz skill.*

**3) KPI özet şeridi + standart sekmeleri (progressive disclosure)** — ★ navigasyon.
Sayfa başında 4-6 stat-tile: `Toplam Düzeltme · AJE adedi · RJE adedi · Net TFRS Etkisi · Denge (✓/✗)`.
Altında her TMS/TFRS için `pill-tab` (mevcut bileşen) → seçilen standardın worksheet'i. URL routing ile her
standart ayrı adres (kullanıcı: "sidebardan tıklanınca yeni URL açılmalı"). → *Kaynak: dataviz skill
(stat-tile paleti) + mevcut `pill-tab`/`card` + frontend-yeniden-tasarim.md progressive disclosure.*

**Neden bu üçü:** üçü de Tailwind eklemeden saf-CSS stack'e oturur (TanStack+Recharts MIT, stilsiz);
CaseWare'in kanıtlanmış çalışma kağıdı düzenini birebir karşılar; "URL + zengin görselleştirme" talebini çözer.

---

## 8. Kullanılan desenler (her kullanımda buraya eklenir)

*(Henüz uygulanmadı — ilk gerçek uygulamada doldur. Şablon:)*

<!--
### [Desen adı] — [tarih]
- **Kaynak:** (Figma dosyası / GitHub repo / skill / kendi tasarımım)
- **Erişim:** (nasıl ulaştım — tekrar için)
- **Ne için kullanıldı:** (hangi ekran/çalışma)
- **Öğrenilen:** (bu desenden çıkardığım kural)
-->

---

## 9. Lisans hızlı-bakış (üretimde güvenli mi?)

| Kaynak | Lisans | Ticari/üretim | Not |
|---|---|---|---|
| TanStack Table, Recharts, Glide Data Grid, shadcn/ui, Tremor(açık bileşenler) | **MIT** | ✅ serbest | Atıf gerekmez (MIT metnini koru) |
| AG Grid **Community** | **MIT** | ✅ serbest | Row grouping/pivot/master-detail = **Enterprise (ücretli)** |
| Figma Community kitleri | dosya-başına | ⚠️ teyit et | Çoğu "personal & commercial" ama her dosyayı ayrı doğrula |
| Figma Dev Mode MCP | araç | ⚠️ koltuk | Tam özellik = Dev/Full + ücretli plan; bizde View/starter |

---

## 5. Kullanılan kararlar (canlı — her uygulamada güncellenir)

### URL Routing — React Router v7 (HashRouter) — 2026-07-15
- **Kaynak:** frontend-routing-plani.md araştırması (ajan). React Router v7 seçildi (kademeli göçe en uygun).
- **Erişim:** `npm i react-router-dom@7`; main.tsx `<HashRouter>`; App.tsx `useNavigate`+`useLocation` köprüsü.
- **Neden HashRouter:** Tauri + tarayıcı önizlemede tutarlı (asset yolu kırılmaz). URL: `/#/panel`, `/#/ufrs`.
- **Öğrenilen:** `git(id)` helper URL yazar, `gorunum` state URL'den TÜRETİLİR (tek kaynak = URL). Sidebar/
  komut/rehber → git; 9 iç setGorunum → git. Kademeli göç Adım 1+2 tamam; alt sekmeler (Adım 3) sonra.
- **Kanıt:** URL değişiyor (#/muhasebe→#/vergi), geri butonu senkron, yenilemede ekran korunuyor.

### Tasarım sistemi — derinlik + marka + durum renkleri katmanı — 2026-07-15
- **Sorun tespiti:** `.card` gölgesizdi (düz/cansız), `.pill.ok` CSS'te TANIMSIZDI (14 yerde "pill ok"
  kullanılıyor ama yeşil görünmüyordu), aktif durumlar nötr siyahtı (marka #E23A32 az kullanılıyordu).
- **Çözüm (styles.css):** gölge değişken sistemi (--sh-sm/sh/md/lg), marka+accent değişkenleri
  (--brand/acc), dark mode değişkenleri (prefers-color-scheme). `.card` yumuşak gölge + hover;
  iç içe kartlar düz. `.pill.ok/.acc/.brand` eklendi + pill'ler radius 999 (hap) + ince iç kenar.
  Tablo: sticky thead + zebra hover + accent tıklama hover. Aktif sekme/sidebar → marka kırmızısı.
  İçerik alanına hafif dikey degrade (düz gri yerine derinlik).
- **Öğrenilen:** className kullanan tüm ekranlar tek CSS değişikliğiyle iyileşti (77 .card, 14 pill).
  Inline stiller (UFRS worksheet çoğu) hâlâ className'e taşınmayı bekliyor — sonraki adım.
- **Kanıt:** UFRS ekranı — aktif sekme kırmızı, VUK=0/AJE=0 rozetleri yeşil, kartlar gölgeli. Konsol temiz.

---

## Figma Buton Kataloğu (2026-07-18)

- **Ne:** Kullanıcının Figma Make ile ürettiği React bileşen kataloğu ("Atelyé UI Arşivi") derinlemesine incelendi ve tasarım sistemi damıtıldı. 13 kategori: buton (8 etkileşim tipi + grup/split), input, badge, kart, modal, drawer, tab, accordion, toast, tablo, navigasyon, tipografi, token.
- **Erişim:** Kaynak scratchpad'e kopyalanmıştı (`figma-katalog/src/App.tsx` + `index.css`); stiller %100 inline + CSS değişkeni, Tailwind sınıfı yok → saf-CSS stack'imize doğrudan taşınabilir.
- **Damıtılan çekirdek:** 3-font sistemi (Playfair Display başlık / Jost gövde-buton / **DM Mono veri-tutar-kod**), 2px radius imzası, sıcak gölge (`rgba(foreground,…)`), tek easing `cubic-bezier(0.22,1,0.36,1)` (modal/toast/toggle), focus halkası `0 0 0 3px` deseni, durum renk dörtlüsü (success/warn/error/info açık-bg + koyu-fg).
- **Beyaz tema kararı:** katalogdaki bej eksen beyaz-nötr eksene çevrildi; marka `#E23A32` yerine bordo `#6B1E2E` (veya ara `#8C2430`) önerildi — parlak kırmızı, muhasebede hata/borç diliyle çakışıyor.
- **Tam detay (tüm CSS değerleri + önerilen beyaz-tema token seti):** → `docs/tasarim/figma-buton-katalogu.md`
