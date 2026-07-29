# Muhasebe Çekirdek Modeli (Faz 0.1)

Sistemin kalbi. Infra (Rust/Axum/Postgres) bunu *uygular*; önce model doğru oturmalı.
Türkiye uygulamasına dayanır: VUK'a göre zorunlu defterler **Yevmiye Defteri**, **Defter-i Kebir**,
**Envanter Defteri**; **Mizan** dönemsel kontrol aracı. Kayıt çift taraflıdır (Σborç = Σalacak).

## Temel kavram: Kayıt nasıl akar?

```
 ┌─ MUHASEBE (KAYIT) MODÜLÜ — temel, tek doğruluk kaynağı ─────────────┐
 │  FİŞ (muhasebe fişi)   → bir olayın belgesi + DAYANAK (fatura/makbuz)│
 │   └─ YEVMİYE SATIRLARI → her satır: bir hesap + borç VEYA alacak     │
 └─────────────────────────────────┬───────────────────────────────────┘
                                    │  (salt-okunur çıktı)
 ┌─ RAPORLAMA MODÜLÜ — AYRI, kayıttan beslenir ───────▼──────────────────┐
 │  YEVMİYE DEFTERİ = satırlar TARİH sırasıyla                            │
 │  DEFTER-İ KEBİR  = satırlar HESAP bazında                             │
 │  MİZAN           = hesap başına borç/alacak/bakiye                     │
 │       └─► BİLANÇO + GELİR TABLOSU + analizler                          │
 └───────────────────────────────────────────────────────────────────────┘
```

**Modül sınırı (kritik karar):**
- **Muhasebe modülü = yazma tarafı.** Sorumluluğu: temiz, dengeli, değişmez, **dayanaklı**, denetlenebilir
  kayıt. Tek sakladığı şey **Fiş + YevmiyeSatırı** (tek doğruluk kaynağı). Önceliğimiz budur.
- **Raporlama modülü = okuma tarafı, AYRI.** Yevmiye defteri / defter-i kebir / mizan / mali tablolar burada
  **türetilir** (saklanmaz, sorguyla — DuckDB ile hızlı). Muhasebenin çıktısından beslenir, kayıt üretmez.
- Bu ayrım (komut↔sorgu) tutarsızlığı imkânsız kılar ve iki modülü bağımsız geliştirilebilir yapar.

## Varlıklar (Entities)

### Mükellef (tenant)
`id · unvan · vkn_tckn · tip(şahıs/şirket) · varsayılan_kdv · oluşturma`
Her mükellefin kendi hesapları, dönemleri, fişleri. İzolasyon: tüm tablolarda `mukellef_id`.

### Hesap Dönemi (fiscal period)
`id · mukellef_id · ad("2026") · baslangic · bitis · durum(AÇIK/KİLİTLİ/KAPANMIŞ)`
Fişler bir döneme bağlı. KİLİTLİ/KAPANMIŞ döneme kayıt girilemez.

### Hesap (account)
`id · mukellef_id · kod("100","100.01") · ad · sinif(1-9) · tip(AKTİF/PASİF/GELİR/GİDER/MALİYET/NAZIM)`
`· dogasi(BORÇ/ALACAK) · ust_hesap_id · seviye(1=sınıf,2=grup,3=ana,4+=alt) · aktif`
TDHP şablonundan tohumlanır (bkz. [[tek-duzen-hesap-plani]]). Hiyerarşi: 1 → 10 → 100 → 100.01.
Bakiyeler alt hesaptan üst hesaba toplanır. **Kayıt yalnızca en alt (muavin) hesaba yapılır.**

### Fiş (voucher / muhasebe fişi)
`id · mukellef_id · donem_id · fis_tipi(AÇILIŞ/TAHSİL/TEDİYE/MAHSUP/KAPANIŞ)`
`· fis_seri · fis_sira · tarih · aciklama · durum(TASLAK/KESİN) · storno_mu · storno_kaynak_id · olusturan · olusturma`

**Numaralama — AYRI seri:** her fiş tipi kendi sırasını tutar → `TAH-1, TAH-2…`, `TED-1…`, `MAH-1…`, `AÇ-1`, `KAP-1`.
`(donem_id, fis_seri, fis_sira)` benzersiz; sıra **boşluksuz** artar (denetim için).

**Dayanak (belge):** Her fiş gerçek bir olaya dayanmalı.
`dayanak: belge_tipi(FATURA/E-FATURA/MAKBUZ/DEKONT/POS_FİŞİ/SÖZLEŞME/BORDRO/BEYANNAME/DİĞER) · belge_no · belge_tarihi · ek_dosya · kaynak(MANUEL/OTOMATİK)`
Dayanağı olmayan fiş **`dayanaksiz=true`** ile işaretlenir — engellenmez ama denetimde **uyarı/raporlanır**.

**Dayanak zorunluluğu — fiş tipine göre ESNEK** (veri-güdümlü config, kod değil):

| Fiş tipi | Dayanak | Not |
|----------|---------|-----|
| TAHSİL | **Zorunlu** | makbuz/dekont |
| TEDİYE | **Zorunlu** | makbuz/dekont/fatura |
| MAHSUP | **Esnek** | önerilir; yoksa `dayanaksiz=true` işaretle, denetim raporlar |
| AÇILIŞ | Gerekmez | sistemsel; kaynağı önceki dönem kapanışı |
| KAPANIŞ | Gerekmez | sistemsel; dönem sonu |

> Otomatik fiş (Belge İşleme modülü) `dayanak.kaynak=OTOMATİK` + ekli belge ile gelir → her zaman dayanaklıdır.

**Kesinleşince değişmez** (denetim izi). Düzeltme = silme değil, **storno** (ters kayıtla iptal) + yeni fiş.

### Yevmiye Satırı (journal line) — en kritik tablo, en büyük hacim
`id · fis_id · mukellef_id · donem_id · sira · hesap_id · aciklama`
`· borc(BIGINT kuruş) · alacak(BIGINT kuruş)`
`· masraf_yeri_id?(FK boyut) · proje_id?(FK boyut) · ek_alanlar(JSONB: yalnızca serbest ekler — vade vb.)`
Kısıtlar: bir satırda **ya borç ya alacak > 0** (ikisi birden değil, ikisi de sıfır değil).
**Partition:** `donem_id` (ya da tarih) RANGE partition → milyarlarca satıra ölçeklenir.
`masraf_yeri_id` boyutu özellikle **gider/maliyet hesaplarında (6,7)** anlamlı; analiz/raporlama için indekslenir.

## Kurallar (domain framework'ün kalbi)
1. **Denge:** Her fiş için `Σborc = Σalacak` (kuruş, tam eşit). KESİN yapmadan zorunlu.
2. **Para:** Her zaman **tamsayı kuruş** (BIGINT). Float yasak. UI'da /100 gösterilir.
3. **Değişmezlik:** KESİN fiş güncellenmez/silinmez → storno ile iptal.
4. **Dönem kilidi:** Kapalı/kilitli döneme fiş girilemez.
5. **Hesap doğası:** Aktif & Gider/Maliyet → artış BORÇ; Pasif & Gelir → artış ALACAK.
6. **Muavin kuralı:** Kayıt yalnızca yaprak (alt) hesaba; üst hesaplar yalnızca toplam.

## Dönem döngüsü: Açılış ↔ Kapanış
- **Kapanış fişi (KAP):** Dönem sonunda hesaplar kapatılır; sonuç bakiyeleri belirlenir.
- **Açılış fişi (AÇ):** Yeni dönem, **önceki dönemin kapanışından gelen bakiyelerle** otomatik üretilir
  (bilanço hesaplarının devri). Manuel girilmez — devir mekanizmasının çıktısıdır.
- Böylece dönemler kesintisiz zincirlenir; her açılış bir önceki kapanışa **dayanır** (izlenebilirlik).

## Raporlama modülü (AYRI — sonraki faz, burada sadece sınır)
Aşağıdakiler **muhasebe çekirdeğinde DEĞİL**, raporlama modülünde türetilir (saklanmaz):
- **Yevmiye Defteri:** `ORDER BY tarih, fis_seri, fis_sira, sira`.
- **Defter-i Kebir:** `WHERE hesap_id` → hareketler + yürüyen bakiye.
- **Mizan:** `GROUP BY hesap_id → SUM(borc), SUM(alacak), bakiye`. DuckDB ile hızlı.

## Faz 0 alt-adımları (öncelik: TEMİZ KAYIT SİSTEMİ)
- **0.1 Bu model** — varlıklar + kurallar ✓
- **0.2 Kayıt kuralları motoru** — fiş tipleri + ayrı seri numaralama, denge, dayanak/dayanaksız işaretleme,
  storno, dönem kilidi, açılış-kapanış devri. Saf domain, infra'sız test edilebilir. **← sıradaki**
- **0.3 İnfra iskeleti** — Rust/Axum/Postgres bu modeli hayata geçirir; dikey dilim "fiş kaydet (dayanaklı) → kesinleştir → storno".
- *(Raporlama ayrı modül/faz — mizan/kebir/tablolar orada.)*

## Boyutlar & Sektör (gider yeri yapısı)
Gider yeri, şirketin **niteliğine / iş planına / yapısına** bağlıdır ve sektöre göre değişir
(emek, üretim gideri, satış maliyeti... hepsi gider olarak ele alınır). Bu yüzden **birinci sınıf boyut**:

- **MasrafYeri (cost center):** `id · mukellef_id · kod · ad · tip(ÜRETİM/HİZMET/SATIŞ/YÖNETİM/...) · ust_id`
  YevmiyeSatırı'na opsiyonel FK; gider/maliyet hesaplarında (6,7) anlamlı.
- **Sektör tanımlama (config modülü, AYRI):** `Sektör` şablonları → şirkete uygulanınca varsayılan
  masraf yerleri + ek hesap planı kalemleri + gider eşleme kuralları gelir.
  `Mukellef.sektor_id` ile bağlanır. Şablonlar **reel finansal tablolar incelenerek** kurgulanır.
- **Proje (opsiyonel boyut):** proje bazlı gider/gelir takibi için ikinci eksen.

> Bu, projeyi modüllere bölmenin temel sebeplerinden biri: sektör-bağımlı davranış kod değil **veri/şablon**.
> 0.2 çekirdeğinde sadece `masraf_yeri_id`/`proje_id` **FK alanları** var; tam Sektör config modülü sonraki faz.

## Çözülen kararlar
- ✅ Numaralama: fiş tipine göre **ayrı seri**, boşluksuz sıra.
- ✅ Dayanak: tipe göre esnek (TAHSİL/TEDİYE zorunlu, MAHSUP esnek); dayanaksız işaretlenir, denetim raporlar.
- ✅ Açılış fişi: önceki dönem kapanışından otomatik devir.
- ✅ Muhasebe ↔ Raporlama: ayrı modüller; raporlama kayıttan beslenir.
- ✅ Masraf yeri / proje: **birinci sınıf boyut** (FK), JSONB değil. Sektör şablonları ayrı config modülü.
- ✅ Belge İşleme (2B): otomasyon taslak fiş üretir, muhasebeci kesinleştirir.
