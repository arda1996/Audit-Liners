---
name: belge-oku
description: Muhasebe belgelerinden (fatura, banka dekontu, ekstre, makbuz, gider pusulası, tapu/sözleşme) veri çıkarır — PDF, PNG, JPEG ve EL YAZISI dahil. Türkçe ve İngilizce tam kapsam; Almanca, Fransızca, Rusça ikinci halka. Okunanı POST /api/belge/coz ile belgenin kendi aritmetiğine doğrulatır (rakam↔yazı, kalem↔toplam), sonra projenin fiş/tahsilat/fatura modeline eşler. Kullanıcı bir belge dosyası verdiğinde, "şu faturayı oku", "dekonttan kayıt çıkar", "ekstreyi işle", "bu fişi kaydet" dediğinde kullan.
---

# Belge Okuma — Türk muhasebe belgelerinden veri çıkarma

## 0. ÖNCE DOĞRU YOLU SEÇ (en sık yapılan hata burada)

Belgenin **metin katmanı var mı** sorusu her şeyi belirler:

| Belge | Doğru yol | Neden |
|---|---|---|
| e-Fatura / e-Arşiv **XML** (UBL-TR) | Doğrudan XML ayrıştır | **Birebir doğru.** OCR'a hiç gerek yok — sayı tahmini olmaz |
| Metin katmanlı PDF (e-fatura çıktısı, ekstre) | `pdftotext -layout dosya.pdf -` | Hızlı ve **birebir**; render+görüntü gereksiz |
| Taranmış PDF (metin katmanı yok) | `tools/belge-render.py` → PNG → **Read** | Görüntüyü Claude'un görüşüyle oku |
| PNG / JPEG (dekont fotoğrafı, fiş) | Doğrudan **Read** | Claude görüntüyü zaten okur |
| **EL YAZISI** (gider pusulası, tutanak, not) | Doğrudan **Read**, gerekirse 300 dpi | Claude'un görüşü el yazısında OCR'dan **iyidir** |

**Tesseract kurmaya çalışma.** Bu projede kurulu değil ve gerekmiyor: Claude'un görme
yeteneği hem matbu hem el yazısı için yeterli, Türkçe karakterlerde de daha isabetli.

Metin katmanı var mı diye bak:
```bash
pdftotext -layout belge.pdf - | head -30    # boş çıkarsa taranmış demektir
```

Taranmışsa görüntüye çevir (araç, metin katmanı varsa seni uyarır):
```bash
tools/belge-venv/bin/python tools/belge-render.py belge.pdf --dpi 200
# el yazısı / küçük punto / soluk baskı ise:  --dpi 300
```

---

## 1. ALTIN KURAL: sayı uydurma

Bu bir **muhasebe** sistemi. Yanlış okunan bir rakam yanlış beyana, yanlış beyan cezaya gider.

- Okuyamadığın alanı **boş bırak ve söyle** — "tutar okunamadı" demek, yanlış tutar yazmaktan iyidir.
- **Aritmetiği doğrula:** `matrah + KDV = toplam` tutmuyorsa okuma hatalıdır, düzeltmeden ilerleme.
- **KDV oranını doğrula:** `KDV / matrah` beklenen orana (%1/%10/%20) düşmüyorsa uyar.
- Belgede **ÖTV** varsa: `KDV matrahı = bedel + ÖTV` (ÖTVK 11 · KDVK 24/b) — ÖTV'yi matraha ekle.
- Türk sayı biçimi: **nokta binlik, virgül ondalık** → `1.234,56` = 1234.56. Ters çevirme.
- Belge **kuruş** olarak modele girer: `1.234,56 TL` → `123456`.

Şüpheye düştüğünde kullanıcıya belgenin o bölgesini göster ve sor.

---

## 1B. OKUDUĞUNU MOTORA DOĞRULAT — `POST /api/belge/coz`

**Kendi okumana güvenme; belgeye doğrulat.** Görme yeteneği yanlış okuduğunda bunu
*kendinden emin* biçimde yapar — "0/O", "1/7", "3/8" karışır ve çıktı sağlam görünür.
Bir okumayı kanıtlayan tek şey **belgenin kendi aritmetiğidir**.

Alanları okuduktan sonra HAM haliyle (belgede yazdığı gibi, metin olarak) motora ver:

```bash
curl -s -X POST http://127.0.0.1:8787/api/belge/coz -H 'content-type: application/json' -d '{
  "metin": "<belgenin tüm okunan metni — dil ve tür sezimi için>",
  "dil": "tr",
  "tarih": "12.03.2026", "karsi_taraf": "ACME A.Ş.",
  "matrah": "1.000,00", "vergi_tutari": "200,00", "vergi_orani": "%20",
  "toplam": "1.200,00", "tutar_yaziyla": "binikiyüz",
  "kalemler": ["600,00", "400,00"]
}'
```

Motor şunları yapar ve **B-kodlu bulgu** döner:

| Kod | Ne yakalar |
|---|---|
| **B00** | BİLGİ — bir çapraz doğrulama TUTTU (güveni bu yükseltir) |
| **B01** | Dil sezilemedi → **ENGEL**. Dil bilinmeden sayı okunamaz |
| **B02** | Belge türü anlaşılamadı |
| **B03** | **Rakam ≠ yazıyla tutar → ENGEL.** Ya okuma hatalı ya belge tahrif edilmiş |
| **B04** | Kalem toplamı ≠ belge tutarı (farkı da söyler) |
| **B05** | matrah + vergi ≠ toplam |
| **B06** | Beyan edilen oranla hesaplanan vergi tutmuyor |
| **B07** | Zorunlu alan okunamadı → **ENGEL** (VUK 227) |
| **B08** | Bazı kalemler sayıya çevrilemedi |
| **B09** | İkinci halka dilde belge — kapsam sınırlı |

`deftere_hazir: false` ise **fiş üretme.** `guven` skoru yalnız DOĞRULANMIŞ alanlardan
yükselir; okunmuş ama doğrulanmamış alan güven vermez.

### Neden bu kadar önemli — gerçek sınama
İki gerçek belgeyle sınandı (`crates/api/src/belge_oku/testler.rs` içinde sabit veri):
- **1889 el yazısı fatura:** 14 kalem okundu, alt toplamlar ve devir tutarı (2101.07)
  kuruşuna oturdu. Tek hane yanlış okunduğunda (41.50 → 47.50) motor **B04** ile yakalıyor.
- **1999 Rusça tahsilat makbuzu:** rakam (102.177,36) ile yazı ("сто две тысячи сто
  семьдесят семь") birbirini doğruladı. Bu, muhasebecinin elle yaptığı kontrolün ta kendisi.

---

## 1C. DİL — okumadan ÖNCE bilinmeli

`GET /api/belge/diller` kapsamı verir.

| | Diller | Kapsam |
|---|---|---|
| **Çekirdek** | **Türkçe · İngilizce** | Alan etiketleri, sayı biçimi, yazıyla tutar, vergi alanları |
| İkinci halka | Almanca · Fransızca · Rusça | Alan etiketleri, sayı biçimi, yazıyla tutar. **Ülkeye özgü vergi alanları yok** |

**Dil bilinmeden sayı okunamaz** — en pahalı hata burada:

| Metin | Türkçe | İngilizce |
|---|---:|---:|
| `1.234` | **1.234,00** | **1,23** |

Bin kat fark. Bu yüzden dil sezilemezse motor **B01 ENGEL** verir ve durur; sen de dili
kullanıcıya sor, varsayma. Sözlük `data/belge-sozlugu.json`'da — yeni etiket eklemek kod
değişikliği gerektirmez.

Dile özgü tuzaklar (hepsi testli):
- **Almanca** sayıyı tek sözcük yazar ve birler önce gelir: `siebenundsiebzig` = 7+70 = **77**.
- **Fransızca**'da 70/80/90 yoktur: `soixante-dix` = 60+10, `quatre-vingts` = **4×20** (çarpma).
- **Rusça** sayı sözcükleri çekimlidir: `тысяча / тысячи / тысяч` hepsi 1000.

---

## 2. BELGE TİPİNE GÖRE ÇIKARILACAK ALANLAR

### Fatura (e-Fatura / e-Arşiv / kağıt)
Zorunlu: `fatura_no · tarih · satıcı ünvan+VKN · alıcı ünvan+VKN · matrah · KDV oranı · KDV · toplam`
İsteğe bağlı ama önemli: `ÖTV · tevkifat oranı ve tutarı · istisna kodu (KDV=0 ise ZORUNLU) · vade · kalemler`

Doğrulama:
- VKN 10 hane, TCKN 11 hane. Hangisi olduğunu haneden anla.
- KDV=0 ise **istisna sebep kodu olmalı** — yoksa belge eksiktir, uyar.
- Tevkifatlı faturada **ödenecek tutar = toplam − tevkifat**; cariye bu yazılır.

### Banka dekontu (fotoğraf/PDF)
`banka · hesap/IBAN · tarih · tutar · yön (giriş/çıkış) · karşı taraf ünvanı · açıklama · referans no`

**Karşı taraf ünvanı yoksa bunu açıkça söyle** — projede o hareket otomatik eşleşemez,
maker kuyruğuna (havada bakiye) düşer. Uydurma.

### Banka ekstresi (çok satırlı PDF)
Her satır: `tarih · açıklama · borç/alacak · tutar · bakiye`
- **Yürüyen bakiye zincirini kontrol et:** `önceki bakiye ± tutar = yeni bakiye`. Kopuksa
  satır atlanmış demektir — kaç satır okuduğunu ve toplamı bildir.
- Açılış/kapanış bakiyesi ile satır toplamı tutmalı.

### Makbuz / gider pusulası / kasa fişi (çoğu EL YAZISI)
`tarih · tutar · kime/kimden · açıklama · stopaj (varsa)`
- Gider pusulasında **stopaj** olur (GVK 94) — brüt/net ayrımını sor, uydurma.
- El yazısında rakam belirsizse (1↔7, 3↔8, 0↔6) **sor**, tahmin etme.

### Sözleşme / tapu
`taraflar · tutar · tarih · süre` → damga vergisi matrahı olur (`/api/hesapla/damga`).

---

## 3. ÇIKARDIĞIN VERİYİ PROJEYE BAĞLA

Belgeyi okumak yarım iş; asıl değer **doğru muhasebe kaydına** dönüşmesidir.

| Belge | Projedeki karşılığı |
|---|---|
| Satış faturası | tahakkuk fişi `120 B / 600 + 391.xx A` |
| Alış faturası | `153 + 191.xx B / 320 A` |
| Banka dekontu | tahsilat olayı → tahsis motoru (FIFO/manuel) |
| Ekstre | çok satırlı tahsilat; havada kalanlar maker kuyruğuna |
| Gider pusulası | `770 B / 100 A + 360 A (stopaj)` |

**KDV muavini ARDIŞIKTIR** — `391.20` diye yazma. Oranın kodunu şuradan çöz:
```bash
curl -s 'localhost:8787/api/hesapla/oranlar?tarih=<belge tarihi>'
```
Tarihi de gönder: **%8 ve %18 10.07.2023'ten sonra geçersizdir**; eski tarihli belgede geçerlidir.

Hesaplamayı elle yapma, motora yaptır:
```bash
curl -s localhost:8787/api/hesapla/kdv  -d '{"matrah":..., "oran":..., "tarih":"gg.aa.yyyy"}' -H 'content-type: application/json'
curl -s localhost:8787/api/hesapla/otv  -d '{"bedel":..., "otv_oran":..., "kdv_oran":...}'   -H 'content-type: application/json'
```

---

## 4. ÇIKTI BİÇİMİ

Her belge için şunu üret — **güven bilgisiyle birlikte**:

```json
{
  "belge_tipi": "SATIS_FATURA",
  "kaynak": "dosya.pdf s.1",
  "okuma_yolu": "metin-katmani | goruntu | el-yazisi",
  "alanlar": { "fatura_no": "...", "tarih": "gg.aa.yyyy", "matrah": 123456, "kdv_oran": 20, "kdv": 24691, "toplam": 148147 },
  "dogrulama": { "aritmetik_tutuyor": true, "kdv_orani_tutuyor": true },
  "okunamayan": ["alıcı VKN — mühür üzerine gelmiş"],
  "uyarilar": ["KDV %0 ama istisna kodu yok"],
  "onerilen_fis": [{ "hesap": "120", "borc": 148147 }, { "hesap": "600", "alacak": 123456 }, { "hesap": "391.06", "alacak": 24691 }]
}
```

`okunamayan` **boş değilse fiş önerme** — eksik veriyle kayıt atılmaz (VUK 227: kayıt belgeye dayanır).

---

## 5. TOPLU İŞLEME

Çok belge varsa: önce **hepsini oku**, sonra **tek bir özet tablo** ver (belge · tip · tutar · durum),
sonra kullanıcı onaylayınca kayda dönüştür. Belge belge onay sorma — muhasebeci yığınla çalışır.

Okunamayan belgeleri ayrı listede topla; sessizce atlama.

---

## 6. YAPMA

- ❌ Tutarı tahmin etme, "muhtemelen" deme.
- ❌ VKN/TCKN uydurma — eksikse eksik yaz.
- ❌ Belgeyi okumadan fiş önerme.
- ❌ Muavin kodunu orandan türetme (`391.20` yanlış) — katalogdan çöz.
- ❌ Metin katmanlı PDF'i görüntüye çevirip okumaya çalışma — gereksiz ve daha hatalı.
- ❌ Belgede olmayan bir alanı "standart böyledir" diye doldurma.
