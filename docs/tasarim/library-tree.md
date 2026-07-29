# Library Tree — Kütüphane Ağacı & Bağımlılık Mimarisi

Amaç: kütüphaneleri **baştan, birbirine ve ileride ekleyeceğimiz modüllere uyumlu** seçmek.
Temel ilke: **domain merkezde, hiçbir şeye bağlı değil**; her şey domain'e bağlanır (hexagonal / ports-adapters).
Yeni modül eklemek = domain'i bozmadan yeni *adapter* eklemek.

## 1. Workspace (Cargo) ağacı

```
audit-liners/                    # cargo workspace
├── crates/
│   ├── domain/    ◄── ÇEKİRDEK — BAĞIMLILIKSIZ (std + sadece thiserror)
│   │                  ledger kuralları, Kurus(i64), Hesap, Fiş, V1–V9, durum makinesi
│   ├── db/        ──► domain   persistence adapter (PostgreSQL)
│   ├── api/       ──► domain   HTTP + auth
│   ├── ingest/    ──► domain   belge işleme (e-Fatura XML, POS OCR) → TASLAK fiş
│   ├── reporting/ ──► domain   mizan/kebir/tablolar (OLAP)
│   ├── audit/     ──► domain   denetim analitiği (örnekleme, Benford)
│   └── app/       ──► hepsi    binary: config + wiring
├── desktop/                     # Tauri 2 — app binary'yi sidecar çalıştırır
└── web/                         # React + TS + Vite (ayrı npm ağacı)
```

**Bağımlılık yönü tek yönlü:** `app → {api, db, ingest, reporting, audit} → domain → ∅`.
domain hiçbir dış crate'e bağlı olmadığı için: test çok hızlı, kurallar her yerde aynı, yeni modül domain'i kırmaz.

## 2. Backend crate'leri → kütüphaneler

### domain (çekirdek — minimum bağımlılık)
| İhtiyaç | Crate | Not |
|---------|-------|-----|
| Hata tipleri | `thiserror` | yalnızca compile-time makro; runtime bağımlılığı yok |
| *(başka hiçbir şey)* | — | para = kendi `Kurus(i64)` newtype'ımız, tarih tipi domain'de sade tutulur |

### Ortak (workspace genel)
| İhtiyaç | Crate |
|---------|-------|
| Async runtime | `tokio` |
| Serileştirme | `serde` + `serde_json` |
| Tarih/saat | `time` (sqlx & serde uyumlu) — alternatif `jiff` |
| Kimlik | `uuid` (v7, zaman-sıralı; çok kiracılı/ölçek dostu) |
| Ondalık (yalnız SINIRDA) | `rust_decimal` — e-Fatura tutarını parse et → **Kurus'a çevir**, çekirdek hep i64 |
| Loglama/izleme | `tracing` + `tracing-subscriber` (denetim izi için kritik) |
| Uygulama hatası | `anyhow` (binary tarafı) |
| Config | `figment` veya `config` (katmanlı: env + dosya) |

### api
| İhtiyaç | Crate |
|---------|-------|
| Web framework | `axum` (tower/hyper üstü — katman/middleware ile genişletilebilir, "az köşeli") |
| Middleware | `tower`, `tower-http` (cors, trace, compression, timeout) |
| JWT | `jsonwebtoken` |
| Şifre hash | `argon2` |
| Girdi doğrulama | `validator` (opsiyonel) |

### db
| İhtiyaç | Crate |
|---------|-------|
| Postgres erişim | `sqlx` (async, **derleme-zamanı SQL kontrolü**, migration) |
| sqlx feature | `postgres, runtime-tokio, tls-rustls, macros, time, uuid, json` |
| Migration (dev) | `sqlx-cli` |

### ingest (belge işleme) — YAPISAL KAYNAK ÖNCELİKLİ (OCR son çare)
**Strateji:** Önce yapısal/elektronik veri; OCR'dan mümkün olduğunca kaçın (ağır + hatalı + bakım).
| İhtiyaç | Yöntem / Crate | Not |
|---------|----------------|-----|
| Gelen e-Fatura/e-Arşiv | **Entegratör API → yapısal veri (JSON)** | Birincil. UBL-TR ayrıştırma + format değişimi entegratörün işi |
| Çevrimdışı yedek | `quick-xml` ile **ince alan-okuyucu** (~15 alan) | Tam UBL kütüphanesi DEĞİL; sadece bilinen alanlar |
| Fiş QR kodu | `rqrr` / `bardecoder` | e-Arşiv fişlerindeki QR → deterministik, OCR'sız |
| Banka/POS ekstresi | `csv` / `calamine` (xlsx) | POS tahsilatları mutabakatı |
| Excel/CSV içe aktarım | `calamine`, `csv` | |
| PDF metin (yapısal) | `pdf-extract` / `lopdf` | yalnız metin-katmanlı PDF |
| OCR (yalnız son çare) | **Bulut OCR API** (Google/Azure) — kendi `leptess`'imiz değil | Gerçekten kağıt kaldıysa; ayrı/sonraki faz |
| Görüntü ön-işleme | `image` | yalnız OCR yoluna girilirse |

### reporting (OLAP)
| İhtiyaç | Crate | Not |
|---------|-------|-----|
| Gömülü OLAP | `duckdb` (`bundled` + `modern-full`) | üretime hazır (v1.5.x) |
| DataFrame | `polars` (streaming) | RAM'den büyük veri |
| Ortak format | `arrow` | **arrow↔arrow2 köprüsü**: DuckDB `arrow`, Polars `arrow2`; Arrow C arayüzüyle kopyasız geçiş |
| Çıktı (Excel/PDF) | `rust_xlsxwriter`, (PDF: `printpdf` veya HTML→PDF) |

### audit (denetim)
| İhtiyaç | Crate |
|---------|-------|
| Analitik | `polars` (yevmiye testi, oran analizi) |
| İstatistik/örnekleme | `statrs`, `rand` (örnekleme), Benford = kendi hesabımız |

### desktop
| İhtiyaç | Crate |
|---------|-------|
| Masaüstü kabuk | `tauri` v2 + `tauri-plugin-*` (app binary'yi sidecar çalıştırır) |

## 3. Frontend (web/) ağacı
```
React 18 + TypeScript + Vite
├── Sunucu durumu  : TanStack Query
├── Tablo (yoğun)  : TanStack Table (headless, genişletilebilir) — güçlü alternatif: AG Grid (lisans dikkat)
├── Form + doğrulama: react-hook-form + zod  (zod ↔ serde şema paralelliği)
├── UI             : Mantine (yoğun veri bileşenleri) — alt. shadcn/ui + Tailwind
├── Routing        : TanStack Router (tip güvenli)
├── i18n (Türkçe)  : i18next
├── Grafik (rapor) : ECharts (büyük veri) / Recharts (basit)
├── Para gösterimi : Intl.NumberFormat('tr-TR') — kuruş(i64) → TL
└── Masaüstü köprü : @tauri-apps/api
```

## 4. Uyumluluğu sağlayan çapraz ilkeler
- **Arrow = ortak analitik dil** → DuckDB ↔ Polars ↔ gelecekte Parquet/ClickHouse sorunsuz.
- **serde her yerde** + FE `zod` → tek şema disiplini, BE↔FE tip tutarlılığı.
- **`Kurus(i64)` tek para tipi**; `rust_decimal` yalnız parse sınırında. Float asla.
- **domain bağımsız** → e-dönüşüm/denetim gibi yeni modüller adapter olarak eklenir, çekirdek değişmez.
- **Versiyon omurgası** (major sabitlenir): `tokio`, `axum`, `sqlx`, `serde`, `tauri`, `polars/duckdb`.

## 5. Açık/dikkat noktaları
- **OCR çekirdekten çıkarıldı:** yapısal kaynak (e-Arşiv XML, QR, banka ekstresi) önceliklidir. OCR yalnız son çare + bulut API + sonraki faz.
- **UBL-TR'yi baştan yazmıyoruz:** birincil = entegratör API'sinden yapısal veri (format değişimi onların sorumluluğu); yalnız çevrimdışı yedek için ince `quick-xml` alan-okuyucu.
- arrow/arrow2 ikiliği → reporting'de tek köprü fonksiyonu ile izole et.
