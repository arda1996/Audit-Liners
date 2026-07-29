# Audit-Liners — İş Planı

Muhasebe + denetim yapabilen, **hem tarayıcıda hem lokalde** çalışan sistem.

## 1. Vizyon ve Hedef Kullanıcı

**Birincil kullanıcı:** SMMM / Mali Müşavir — çok sayıda mükellefi (müşteriyi) tek yerden yöneten,
yevmiye/defter/mizan/beyanname iş akışına ihtiyaç duyan profesyonel.

**Sonraki halkalar:** KOBİ'ler (ön muhasebe), bağımsız denetçiler (denetim modülü).

**Temel ilke:** Çift taraflı (double-entry) muhasebe çekirdeği üzerine kurulu; Türkiye mevzuatına
(Tek Düzen Hesap Planı, VUK) uyumlu; her kayıt denetlenebilir (audit trail).

## 2. Mimari Kararı

**Seçim:** Web backend + masaüstü paketleme. Performans-öncelikli, tek-dilli (Rust) çekirdek;
genişletilebilirlik için **kendi domain framework'ümüzü** kanıtlanmış ince bir taban üstüne kuruyoruz.

| Katman | Karar | Neden |
|--------|-------|-------|
| Masaüstü kabuk | **Tauri (Rust)** | Hafif; backend ile **tek dil** |
| Backend | **Rust + Axum** | En üst performans + bellek güvenliği; büyük-veri denetimi için ideal; kabukla tek dil |
| DB erişimi | **sqlx** | Derleme-zamanı kontrollü SQL; az soyutlama = "az köşeli", karmaşık mali sorgularda tam kontrol |
| Çekirdek DB (OLTP) | **PostgreSQL** | ACID zorunlu; yevmiye satırları **dönem-bazlı range partitioning** ile milyarlara ölçeklenir |
| Denetim/analitik (OLAP) | **DuckDB + Polars** | Gömülü, kolonsal; örnekleme/Benford/yevmiye testi milyonlarca satırda hızlı; sunucusuz |
| İleride bulut ölçeği | ClickHouse (opsiyonel) | Çok kiracılı 1TB+ dağıtık analitik gerekirse; şimdi değil |
| Frontend | **React + TypeScript + Vite** | Tek kod hem web hem Tauri; ağır veri-tabloları için olgun ekosistem |
| Domain framework | **Kendi yazımımız** | Ledger motoru, fiş/denge kuralları, kural DSL'i, plugin kayıt sistemi, rapor/denetim test çerçevesi — ürünün asıl değeri |

**İlkeler:**
- Para/tutar **tamsayı kuruş** (integer minor units) — float **yasak** (yuvarlama hatası kabul edilemez).
- Lokalde de PostgreSQL (pakete gömülü/sidecar) → SQLite↔Postgres lehçe riski yok. Analitik DuckDB ile in-process.
- **Veri-güdümlü sektörler** (seed/config), çekirdek defterde güçlü-tip + **JSONB** esnek alanlar, **plugin** mimarisi (e-dönüşüm/rapor/denetim testi), **modüler monolit → mikroservise hazır**.

## 3. Fazlar (Yol Haritası)

### Faz 0 — Muhasebe Çekirdeği & İskelet
**Önce muhasebe domain'i, sonra infra.** Detay: `docs/tasarim/muhasebe-cekirdek-modeli.md`.
- **0.1 Çekirdek model** — Mükellef / Dönem / Hesap / Fiş / YevmiyeSatırı + kurallar. *(yapıldı: tasarım)*
- **0.2 Kayıt kuralları motoru** — fiş tipleri, denge (Σborç=Σalacak), storno, dönem kilidi; saf domain, infra'sız test.
- **0.3 Türev görünümler** — Yevmiye Defteri / Defter-i Kebir / Mizan sorgu tanımları.
- **0.4 İnfra iskeleti** — git init + monorepo (`backend/` Axum, `desktop/` Tauri, `web/` Vite); Postgres + sqlx migration (partitioned `yevmiye_satiri`); dikey dilim "fiş kaydet → mizan" uçtan uca; CI/lint.

### Faz 1 — Muhasebe Bilgi Temeli (öğrenme + veri modeli)
Amaç: kod yazmadan önce muhasebeyi doğru modellemek. Çıktılar `docs/learnings/` altında (optimizer otomatik enjekte eder):
- `muhasebe-temelleri.md` — Muhasebe nedir; türleri (genel/mali, maliyet, yönetim muhasebesi); tek vs çift taraflı kayıt; borç/alacak mantığı; muhasebe denklemi (Varlık = Kaynak).
- `tek-duzen-hesap-plani.md` — TDHP 7 sınıf + nazım hesaplar (referans veri — bkz. ayrı dosya).
- `sektorel-hesaplar.md` — Sektörlere göre yoğun kullanılan hesapların analizi (ticaret, üretim, hizmet, inşaat).
- `sik-kullanilan-kayitlar.md` — En çok kullanılandan en aza muhasebe fişi örnekleri (alış/satış, KDV, tahsilat/ödeme, ücret, amortisman, dönem sonu...).
- **Veri modeli taslağı:** Hesap (chart of accounts), Fiş (voucher), FişSatırı (journal line), Mükellef (tenant), Dönem (period).

### Faz 2 — Muhasebe (Kayıt) Modülü — TEMEL
Yalnızca yazma tarafı; temiz kayıt sistemi. (Mizan/kebir/yevmiye defteri = ayrı Raporlama modülü.)
- Hesap planı yönetimi (TDHP'yi tohum verisi olarak yükle, alt hesap açma).
- Muhasebe fişi girişi (tahsil/tediye/mahsup) — ayrı seri numaralama, **dayanak** (tipe göre zorunlu/esnek).
- **Denge zorunluluğu** (Σborç=Σalacak), kesinleştirme, **storno**, dönem kilidi.
- Dönem açma/kapama; açılış = önceki kapanıştan **otomatik devir**.
- **Doğrulama:** her fiş dengeli, değişmez, dayanak durumu işaretli (otomatik test).

### Faz 2B — Belge İşleme / Otomatik Fiş (ayrı modül, çekirdeği BESLER)
Otomasyon **TASLAK** fiş üretir (dayanak ekli) → muhasebeci onaylar/kesinleştirir. Çekirdeğe asla doğrudan kesin kayıt yazmaz.
**Strateji: yapısal kaynak öncelikli, OCR son çare.**
- **Gelen e-Fatura/e-Arşiv:** birincil = **entegratör API'sinden yapısal veri** (UBL-TR ayrıştırma + format değişimi onların işi); çevrimdışı yedek = ince `quick-xml` alan-okuyucu.
- **Fiş/QR + banka ekstresi:** QR kod (deterministik) + POS tahsilatı banka ekstresiyle mutabakat — OCR'sız.
- **OCR yalnız son çare:** kağıt kaldıysa bulut OCR API (kendi sunucumuzda kütüphane değil), sonraki faz.
- **Hesap eşleme motoru** (asıl değer): VKN→cari hesap, ürün/KDV→hesap; kural tabanlı + öğrenen.

### Faz 3 — Raporlama Modülü (ayrı) + Çok Müşterili Yönetim (SMMM)
Muhasebenin çıktısından beslenir; salt-okunur, kayıt üretmez.
- **Yevmiye Defteri / Defter-i Kebir / Mizan** (geçici/kesin) — DuckDB ile.
- Bilanço ve Gelir Tablosu (TDHP eşlemesiyle), cari ekstre, KDV özeti; PDF/Excel çıktı.
- Mükellef yönetimi (çok şirket, izolasyon), kullanıcı/rol.

### Faz 4 — İç Kontrol & Denetim İzi
- Değişmez audit log (kim/ne/ne zaman; kayıt sonrası değiştirilemez fiş — storno ile düzeltme).
- Yetki/rol matrisi, dönem kilitleme.
- Anomali/uyarı kontrolleri (dengesiz fiş, olağandışı tutar, eksik belge).

### Faz 5 — E-Dönüşüm (Türkiye)
- e-Fatura / e-Arşiv / e-İrsaliye (entegratör veya GİB test ortamı).
- e-Defter (berat/XBRL-GL), beyanname hazırlık (KDV, Muhtasar, Geçici).
- *Regüle ve ağır; çekirdek oturduktan sonra.*

### Faz 6 — Bağımsız Denetim Modülü
- Örnekleme (sampling), önemlilik (materiality) hesabı.
- Yevmiye testi (journal entry testing), oran analizi, çalışma kâğıtları.
- BDS/ISA uyumlu denetim dosyası.

### Faz 7 — Paketleme & Dağıtım
- Tauri imzalı masaüstü kurulumları (Win/macOS/Linux).
- Bulut dağıtımı (Docker), yedekleme/geri yükleme, güncelleme akışı.

## 4. Referans Projeler (dökümantasyon/öğrenme amaçlı)

| Proje | Ne için bakacağız | Lisans (dikkat) |
|-------|-------------------|-----------------|
| [ERPNext](https://github.com/frappe/erpnext) | Veri modeli, GL Entry yapısı, çok şirket | GPLv3 |
| [Akaunting](https://github.com/akaunting/akaunting) | KOBİ UX, fatura/gider akışı | Karma (çekirdek açık) |
| [Firefly III](https://github.com/firefly-iii/firefly-iii) | Temiz çift taraflı kayıt, REST API | AGPLv3 |
| [IndeksTicari](https://github.com/fatihgokce/IndeksTicari) | Türkçe ön muhasebe örneği | repo'da kontrol |
| [Stok-Muhasebe](https://github.com/Brktrlw/Stok-Muhasebe-Yazilimi) | Türkçe stok+muhasebe akışı | repo'da kontrol |

**Lisans uyarısı:** GPL/AGPL **kodunu** doğrudan kopyalamak kendi projemizi de aynı lisansa zorlar.
Bizim yaklaşımımız: **fikir, veri modeli, ekran akışı ve hesap yapısını** referans almak (bu serbest);
kod kopyalamamak. Maintainer'lara katkı/iş birliği için ulaşmak da bir seçenek.

## 5. Önümüzdeki Adım
Onay verirsen Faz 1'i başlatıyoruz: `docs/learnings/muhasebe-temelleri.md` ve diğer öğrenme
dosyalarını yazıp, çekirdek **veri modelini** taslaklaştırırız (Hesap / Fiş / FişSatırı / Mükellef / Dönem).
Kod yazmadan önce muhasebe modelini doğru oturtmak, sonradan en çok maliyeti önler.
