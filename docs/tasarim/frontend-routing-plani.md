Araştırma tamamlandı. Kod tabanını (web/src/App.tsx, 2483 satır) inceledim ve kütüphaneleri güncel verilerle karşılaştırdım. Aşağıda tam rapor.

---

# Audit-Liners: URL Routing Mimarisi Araştırması

## 0. Mevcut durum tespiti (kod tabanından)

Öneriyi somutlaştırmak için önce gerçek yapıyı çıkardım:

- **Stack:** React 18.3 + Vite 5.4 + TS 5.5, **Tauri masaüstü** (`src-tauri/` var) → **SSR/SEO gereksiz, saf client-side SPA**. Bu, kütüphane seçimini basitleştiriyor.
- **Ana navigasyon:** `web/src/App.tsx:83` → `const [gorunum, setGorunum] = useState("dashboard")`. Ekranlar `{gorunum === "muhasebe" && (...)}` bloklarıyla koşullu render ediliyor (10 ana ekran: dashboard, muhasebe, hesaplar, vergi, banka, belgeler, analiz, denetim, ufrs, yonetim).
- **Alt navigasyon state'leri (URL'e taşınacak asıl zorluk bunlar):**
  - `muhAlt` (App.tsx:84): yeni / fisler / yevmiye / kebir / muavin / mizan
  - `vergiAlt` (App.tsx:214): kdv / gecici / takvim / param
  - `analizAlt` (App.tsx:323): oranlar / mali
  - `ufrsSekme` (App.tsx:366) + `ufrsWs` (App.tsx:369, seçili çalışma id'si, ör. `tms16-...`) + `ufrsDetay`
  - `kagit` (App.tsx:343): seçili denetim çalışma kağıdı
- **Kesişen bağlam (kritik):** `aktifMuk` (App.tsx:186, aktif mükellef) ve dönem. Mükellef değişince **tüm** veri yeniden çekiliyor (App.tsx:199 yorumu: "çalışma seti swap oldu"). Bu, "şu çalışmayı meslektaşıma göndereyim" akışının merkezinde.
- **Veri çekme deseni:** `useEffect(() => { if (gorunum === "ufrs") yenileUfrs(); }, [gorunum])` gibi ~20 effect, `gorunum`/alt-state'e bağlı fetch. (App.tsx:392, 308, 325, 621-627 vb.)
- **Zaten ayrık bileşenler:** Sidebar, CommandPalette, Login, Rehber, HesapSecici, AciklamaPanel/Secici. **Sidebar `aktif` + `sec(id)` callback alıyor** (Sidebar.tsx) → Link'e çevirmesi çok kolay. CommandPalette `git={setGorunum}`, Rehber `git` alıyor.

Bu yapı **kademeli göçe çok uygun**: navigasyon zaten tek bir `gorunum` string'i etrafında toplanmış; onu URL ile senkronlamak ilk adımı tek başına değerli kılıyor.

---

## 1. Kütüphane seçimi — ÖNERİ: **React Router v7 (declarative / SPA modu)**

### Karşılaştırma tablosu

| Kriter | React Router v7 | TanStack Router | Wouter |
|---|---|---|---|
| Bundle (min+gz) | ~18–20 KB (yalın `react-router` SPA modu; framework modu değil) | ~40 KB | **~2.1 KB** |
| Lisans | MIT | MIT | ISC (MIT-uyumlu, permisif) |
| Vite uyumu | Tam (plugin gerektirmez, `createBrowserRouter` yeterli) | Tam (route-tree **codegen** build adımı ister) | Tam (sıfır kurulum) |
| Nested route (UFRS içi sekme) | `<Outlet/>` ile birinci sınıf | Birinci sınıf, tipli | Var ama derin yapıda daha zayıf ergonomi |
| Tip güvenliği (URL param/search) | SPA modunda manuel cast | **%100 tipli** (öne çıkan özellik) | Tipsiz |
| Öğrenme eğrisi | Düşük (ekip zaten tanıyor, en büyük ekosistem) | Orta-yüksek (route tree + codegen zihinsel modeli) | En düşük |
| Kademeli göçe uygunluk | **En yüksek** (dev switch'i korurken adım adım) | Orta (route tree'yi baştan tanımlamayı teşvik eder) | Yüksek (hook ile `gorunum` yerine geçer) |

Kaynak: bundle/lisans/tip verileri [PkgPulse — TanStack Router vs React Router v7 (2026)](https://www.pkgpulse.com/blog/tanstack-router-vs-react-router-v7-2026), [TanStack Router Docs — Comparison](https://tanstack.com/router/latest/docs/comparison), [Better Stack — TanStack vs React Router](https://betterstack.com/community/comparisons/tanstack-router-vs-react-router/), [wouter — GitHub (~2.2KB, nested routing)](https://github.com/molefrog/wouter).

### Neden React Router v7?

1. **Kademeli göç önceliği (kullanıcının asıl talebi: "her noktayı birlikte", kırılmadan).** RR v7 declarative modda dev `gorunum` switch'ini **hiç bozmadan** yanına eklenebilir: önce sadece `<BrowserRouter>` sar, sonra URL↔`gorunum` köprüsü kur. TanStack Router seni baştan bir route ağacı + codegen adımı tanımlamaya iter — 2483 satırlık tek dosyayı "bir ekranı bir güne taşı" akışına daha az uygun.
2. **Nested route → `<Outlet/>`** deseni, senin iki katmanlı yapına (sidebar layout + UFRS worksheet sekmeleri) doğrudan oturuyor.
3. **Tauri (SSR yok):** RR v7'nin ağır tarafı (framework/Remix modu, Vite plugin, SSR, loader server fonksiyonları) **gerekmiyor**. Sadece `react-router-dom`'un `createBrowserRouter` API'sini kullanacaksın — bu tam da en hafif, plugin'siz yol.
4. **Ekosistem + tanıdıklık:** en düşük risk, en çok örnek/StackOverflow.

### Ne zaman diğerleri?

- **TanStack Router**, eğer "meslektaşıma link göndereyim" akışını **tipli search param** (mükellef+dönem URL'de, derleyici garantili) olarak birinci sınıf istiyorsan ciddi bir alternatif. Bozuk link/geçersiz param sınıfı hataları derleme zamanında yakalanır. Maliyeti: codegen build adımı + daha dik öğrenme eğrisi + tek-dosyadan göçün daha zahmetli olması. **Önerim:** şimdilik RR v7 ile başla; ileride tip güvenliği kritikleşirse TanStack'e geçiş yolu açık kalır.
- **Wouter (2.1 KB, ISC):** Tauri'de bundle boyutu kritikse ve en ucuz göçü istiyorsan cazip — `useLocation`/`useRoute` hook'ları `gorunum` state'inin yerine neredeyse birebir geçer. Ama tipli route, data loader ve derin nested yapı ergonomisi zayıf. Denetim+UFRS+Vergi modülleri büyüdükçe RR'nin sağladığı hareket alanını vermez. **Sadece "minimal bağımlılık" felsefen ağır basarsa.**

> **Tauri notu:** `BrowserRouter` (history API) Tauri webview'de dev + prod'da genelde sorunsuz çalışır. Prod build'de custom protokol nedeniyle derin link/asset yolu kırılırsa **`HashRouter`'a geç** (`/#/ufrs/...`) — tek satır değişiklik, güvenli geri dönüş.

---

## 2. URL şeması (tam liste)

**Tasarım kararı — mükellef ve dönem `query` param olarak, path temiz semantik.**

Gerekçe: `aktifMuk` + dönem **her ekrandaki fetch'i etkileyen kesişen bağlam**. Path'e gömülürse (`/m/:muk/...`) her route bunu tekrarlar ve navigasyonda taşımak zahmetli olur. **Query param** ise tüm route'larda korunur ve "linki meslektaşıma gönder" akışı için idealdir: link tek başına mükellef+dönem+ekranı taşır. (Bu ayrıca TanStack Router'ın tipli search param'ının parladığı yer — ileride geçiş yaparsan doğal uyum.)

```
/                              → /panel'e yönlendir
/panel                          Genel Bakış (dashboard)

/muhasebe                       → /muhasebe/yeni
/muhasebe/yeni                  Yeni kayıt
/muhasebe/fisler                Fişler listesi
/muhasebe/fisler/:fisId         Fiş detay (acFis)
/muhasebe/yevmiye               Yevmiye defteri
/muhasebe/kebir                 Defter-i kebir
/muhasebe/kebir/:hesapKod       Kebir — seçili hesap
/muhasebe/muavin                Muavin
/muhasebe/mizan                 Mizan

/banka                          Banka eşleştirme
/belgeler                       e-Fatura/e-Arşiv

/vergi                          → /vergi/kdv
/vergi/kdv                      KDV beyanname taslağı
/vergi/gecici                   Geçici vergi
/vergi/takvim                   Vergi takvimi
/vergi/parametreler             Vergi parametreleri

/analiz                         → /analiz/oranlar
/analiz/oranlar                 Oranlar + banka görünümü
/analiz/mali                    Mali tablolar

/raporlar/bilanco               Bilanço
/raporlar/gelir-tablosu         Gelir tablosu

/denetim                        Sektörel programlar
/denetim/:kagitId               Çalışma kağıdı (BDS 230)

/ufrs                           → /ufrs/wtb
/ufrs/wtb                       Working Trial Balance
/ufrs/kayitlar                  Kayıt defteri (AJE/RJE)
/ufrs/c/:calismaId              Tek çalışma  (ör. /ufrs/c/tms16-amortisman)

/hesaplar                       Hesap planı
/hesaplar/:kod                  Seçili hesap

/yonetim                        Kullanıcı/yetki (yalnız admin)
/firma                          Firma/Mükellef (F1)
/parametre                      Parametreler (F1)

── TÜM route'larda korunan query ──
?muk=<mukellefId>&donem=<YYYY-MM>
   ör: /ufrs/c/tms16-amortisman?muk=m1&donem=2026-12
```

**"Çalışmayı meslektaşıma gönder" akışı:** UFRS çalışma sayfasında bir "Bağlantıyı kopyala" butonu → `window.location.href` (mükellef+dönem query dahil) panoya. Alıcı açtığında `RootLayout` query'den mükellef+dönemi okur, doğru çalışma seti yüklenir, `/ufrs/c/:calismaId` doğru çalışmayı açar. Tam bağlam tek linkte.

> Not: `ufrsWs` id'leri (`ufrsKatalog.calismalar`) zaten slug benzeri (`tms16-...`) → doğrudan `:calismaId` olarak kullanılabilir, ekstra eşleme gerekmez.

İleride mükellef **sert bağlam sınırı** olarak istenirse path'e terfi ettirilebilir (`/m/:vkn/ufrs/...`); ama kademeli göç için query ile başlamak doğru.

---

## 3. Tek dosyadan bölme planı (route başına dosya)

Hedef dizin yapısı:

```
web/src/
  main.tsx                 → <RouterProvider router={router}/>
  router.tsx               → createBrowserRouter([...])  (route tanımı tek yerde)
  App.tsx                  → SADECE RootLayout'a küçülür (giriş/oturum sarmalayıcı)

  routes/
    RootLayout.tsx         Sidebar + header + <Outlet/>  + Mükellef/Dönem provider
    Dashboard.tsx
    muhasebe/
      MuhasebeLayout.tsx   (pill-tab'lar + <Outlet/>)
      Yeni.tsx  Fisler.tsx  Yevmiye.tsx  Kebir.tsx  Muavin.tsx  Mizan.tsx
    Banka.tsx
    Belgeler.tsx
    vergi/
      VergiLayout.tsx      Kdv.tsx  Gecici.tsx  Takvim.tsx  Parametreler.tsx
    analiz/  AnalizLayout.tsx  Oranlar.tsx  Mali.tsx
    raporlar/ Bilanco.tsx  GelirTablosu.tsx
    denetim/  Denetim.tsx  Kagit.tsx
    ufrs/     UfrsLayout.tsx  Wtb.tsx  Kayitlar.tsx  Calisma.tsx
    Hesaplar.tsx
    Yonetim.tsx

  context/
    MukellefContext.tsx    aktifMuk + donem + değiştirme + "swap" mantığı
  lib/
    api.ts                 API base, authFetch, ortak fetch yardımcıları
    types.ts               Hesap, Satir, MizanSatir, FisDetay, UfrsKayitT, WtbT, KagitT ...

  components/               (mevcut: Sidebar, CommandPalette, HesapSecici, Rehber, ...)
```

**Ortak layout korunması:** `RootLayout.tsx` sidebar + header + `<Outlet/>` render eder; içinde `<MukellefProvider>` sarmalayıcı olur. Her route bileşeni mükellef/dönemi `useMukellef()` hook'undan, fetch yardımcılarını `lib/api.ts`'ten okur — böylece App.tsx'teki mevcut fetch mantığı **kopyalanmaz, taşınır**.

**Kırılmayı önleyen anahtar:** State'i route bileşenlerine dağıtmadan önce **paylaşılan bağlamı** (MukellefContext + api.ts + types.ts) çıkar. Böylece her ekran aynı mükellef/dönem/fetch kaynağını okur; bir ekranı taşırken diğerleri etkilenmez.

---

## 4. Kademeli geçiş adımları (her biri tek başına çalışır, ayrı ayrı ship edilir)

> İlke: hiçbir adım diğerine bağımlı değil; her adımdan sonra uygulama çalışır, `gorunum` switch'i son adıma kadar yaşamaya devam eder.

**Adım 1 — Router'ı sar (görünür değişiklik yok).**
`npm i react-router-dom@7`. `main.tsx`'te `<App/>`'i `<BrowserRouter>` ile sar. Başka hiçbir şey değişmez. Ship. *(Risk: sıfır.)*

**Adım 2 — URL↔`gorunum` köprüsü (EN YÜKSEK DEĞER, en küçük adım).**
- Sidebar'ın `sec(id)`'i `navigate("/" + (id==="dashboard"?"panel":id))` çağırsın.
- App'te `const loc = useLocation()` + bir `useEffect` ile `pathname`'in ilk segmentini `setGorunum`'a eşle.
- CommandPalette `git` ve Rehber `git` de `navigate` kullansın.
→ Artık **URL değişiyor, geri/ileri çalışıyor, yenilemede ekran korunuyor, link paylaşılabiliyor.** Hâlâ tek dosya, hiçbir render mantığı taşınmadı. Ship.

**Adım 3 — Alt sekmeleri URL'e taşı.**
`muhAlt → /muhasebe/:alt`, `vergiAlt → /vergi/:alt`, `analizAlt → /analiz/:alt`, `ufrsWs → /ufrs/c/:calismaId`, `kagit → /denetim/:kagitId`, `fis → /muhasebe/fisler/:fisId`. Aynı köprü deseni: `useParams` oku → ilgili `setXxx`. Ship.

**Adım 4 — Mükellef + dönemi search param yap.**
`?muk=&donem=` oku/yaz (`useSearchParams`). Fetch'lerin bağımlılık dizisine bunları ekle. → **Linkler artık tam bağlam taşıyor** ("meslektaşıma gönder" akışı çalışır). Ship.

**Adım 5 — `createBrowserRouter` + gerçek `RootLayout` + `<Outlet/>`.**
`router.tsx` oluştur; `RootLayout` sidebar+header+`<Outlet/>` render etsin. Başlangıçta her route hâlâ dev switch'ini render eden bir `<LegacyScreen/>` catch-all'a düşsün. → Layout ayrıştı, ekranlar henüz taşınmadı. Ship.

**Adım 6 — Paylaşılan bağlamı çıkar (saf taşıma, davranış değişmez).**
`lib/api.ts`, `lib/types.ts`, `context/MukellefContext.tsx` oluştur; ilgili fonksiyon/tip/state'leri App'ten buraya taşı. Derleme yeşil kalmalı. Ship.

**Adım 7 — Ekranları teker teker soy (en izole olandan başla).**
Her adımda bir `gorunum === "X"` bloğunu `routes/X.tsx`'e taşı, bağlamdan besle, eski bloğu sil. **Sıra:** önce yaprak raporlar (bilanco, gelir-tablosu, hesaplar, banka, belgeler — en az kesişen state) → sonra muhasebe → vergi → analiz → denetim → **en son UFRS** (en karmaşık, nested). Her ekran ayrı ship.

**Adım 8 — `gorunum` state'ini ve köprüyü sil.**
Son ekran taşınınca köprü effect'i, `gorunum`/`setGorunum` ve `LegacyScreen` kaldırılır. Göç tamam.

**Opsiyonel (sonra) — Data loader'lar.** `useEffect(if gorunum===...) fetch` desenini RR v7 `loader` fonksiyonlarıyla değiştir: veri route girişinde yüklenir, `useLoaderData` ile okunur. Göç için gerekli değil, ama effect gürültüsünü ciddi azaltır. TanStack'e geçersen bunun tipli karşılığı var.

---

## Özet

- **Kütüphane:** React Router v7 (declarative/SPA modu, plugin'siz `createBrowserRouter`). Gerekçe: kademeli göçe en uygun, nested `<Outlet/>` senin sidebar+UFRS-sekme yapına oturuyor, Tauri'de SSR gereksiz, ekosistem/tanıdıklık en yüksek. Alternatifler: tipli paylaşılabilir URL kritikse **TanStack Router**; minimal bundle felsefesi ağır basarsa **Wouter (ISC, 2.1KB)**.
- **URL:** path semantik (`/ufrs/c/:calismaId`), mükellef+dönem **query param** (`?muk=&donem=`) — tüm route'larda korunur, "linki gönder" akışının çekirdeği.
- **Bölme:** `router.tsx` + `RootLayout` (`<Outlet/>`) + `routes/` (ekran başına dosya) + `context/MukellefContext` + `lib/api|types`. App.tsx yalnız layout'a küçülür.
- **Göç:** 8 küçük, bağımsız, tek tek ship edilebilir adım; dev `gorunum` switch'i son adıma kadar bozulmadan yaşar. En değerli tek adım: **Adım 2 (URL↔gorunum köprüsü)** — bir oturumda back/forward + shareable URL + yenilemede kalıcılık kazandırır.

**İlgili dosyalar (mutlak yol):**
- `/Users/arda/Desktop/Audit-Liners/web/src/App.tsx` (2483 satır — bölünecek ana dosya; navigasyon: satır 83, 772-817; UFRS: 366-474; alt-state'ler: 84/214/323/343)
- `/Users/arda/Desktop/Audit-Liners/web/src/Sidebar.tsx` (`sec(id)` → `navigate`'e çevrilecek)
- `/Users/arda/Desktop/Audit-Liners/web/src/CommandPalette.tsx`, `Rehber.tsx` (`git` callback'leri)
- `/Users/arda/Desktop/Audit-Liners/web/src/main.tsx` (Router sarmalama noktası)
- `/Users/arda/Desktop/Audit-Liners/web/vite.config.ts` (`/api` proxy — routing'den etkilenmez)

**Kaynaklar:**
- [PkgPulse — TanStack Router vs React Router v7 (2026)](https://www.pkgpulse.com/blog/tanstack-router-vs-react-router-v7-2026)
- [TanStack Router Docs — Comparison](https://tanstack.com/router/latest/docs/comparison)
- [Better Stack — TanStack Router vs React Router](https://betterstack.com/community/comparisons/tanstack-router-vs-react-router/)
- [wouter — GitHub (~2.2KB, nested routing, ISC)](https://github.com/molefrog/wouter)
- [Medium/ekino — TanStack Router vs React Router v7](https://medium.com/ekino-france/tanstack-router-vs-react-router-v7-32dddc4fcd58)