# Entropik Belgelerden Düzen Çıkarma — Tasarım

> Üç paralel topluluk araştırmasının sentezi. Her iddia kaynağa bağlıdır; doğrulanmamış
> olanlar **DOĞRULANMADI** diye işaretlidir.

---

## 0. Önce kötü haber: A1 bir doğruluk sorunu değil

RapidOCR'ın varsayılan tanıma sözlüğü `ppocr_keys_v1.txt`, **6.623 satır** ve Türkçe'ye
özgü 12 karakterden yalnızca **2'sini** içeriyor (ü, Ü).

Yani model `ŞELALE`'yi *yanlış okumuyor*. `ş` çıktı alfabesinde **yok**; üretmesi fiziksel
olarak imkânsız. Kayıp **%100 ve deterministik** — modelin kalitesiyle ilgisi yok.

| Sözlük | TR karakter kapsaması |
|---|---|
| `ppocr_keys_v1.txt` ← **şu an kullandığımız** | **2/12** |
| `latin_dict.txt` (v3/v4) | 7/12 (ğ Ğ ş Ş İ yok) |
| **`ppocrv5_latin_dict.txt`** | **12/12 ✅** |
| EasyOCR `tr_char.txt` | 12/12 ✅ |
| ocrs (Rust) | 0/12 |

**Çözüm:** `PaddlePaddle/latin_PP-OCRv5_mobile_rec` — Apache-2.0, mobil boyut, CPU'da
çalışır, 32 Latin dili. RapidOCR 3.x `rec_keys_path` ile sözlük yolu alıyor.

Bu, listedeki **en yüksek getirili tek madde**: yarım günlük iş, %100 deterministik bir
kaybı kapatıyor.

### Ama tek başına yetmez

Türkçe OCR'ın **ölçülmüş** tek karşılaştırması: **OCRTurk** (ODTÜ + Roketsan,
EACL 2026 SIGTURK, 180 Türkçe belge). `TCS` metriği tam bizim sorunumuzu ölçüyor —
Türkçe'ye özgü karakterlerde başarım:

| Model | TCS ↑ | Lisans |
|---|---|---|
| HunyuanOCR | **0,88** | Tencent Community — **ticari kısıtlı** |
| PaddleOCR-VL | 0,82 | Apache-2.0 |
| DeepSeek-OCR | 0,81 | MIT |
| Docling | 0,71 | MIT |
| Nemotron v1.1 | 0,47 | — |

**En iyi model bile TR karakterlerin %12'sini kaybediyor.** Hiçbir motor bunu tek başına
çözmüyor. Düzeltme katmanı yapısal olarak şart.

### Ucuz ek kazanç: ayrışmış diakritik onarımı

OCRTurk yazarları değerlendirme öncesi `˘g` → `ğ`, `¸s` → `ş` dönüşümü yapmak zorunda
kalmış. Motorlar karakteri **birleşik değil parçalı** üretiyor. Unicode NFC normalizasyonu
+ birleşen-işaret onarımı birkaç satır, kazancı yüksek.

Ayrıca kod tarafında: `"İSTANBUL".to_lowercase()` tr-TR yerel ayarı olmadan yanlış sonuç
verir. Bu OCR'dan bağımsız, **bizim kendi** bozulmamız.

### Deasciifier'a dikkat — muhasebede tehlikeli

`Kadikoy` → `Kadıköy` düzeltmesi cazip. Ama deasciifier korpus desenleriyle çalışır ve
**özel isimler tam da kör noktasıdır**. Denetim bağlamında uydurulmuş makul bir ünvan,
bariz bozuk bir ünvandan **daha tehlikelidir**: `SELALE` gözle yakalanır, "Şelale Tekstil"
Ba/Bs mutabakatına sessizce girer.

**Kural:** unvan alanlarında otomatik uygulanmaz; öneri üretilir, kullanıcı onaylar, karar
izlenir. Sayısal alanlardaki denklem korumasının metin karşılığı bu — ama denklem değil,
olasılık.

---

## 1. İyi haber: doktrinimizin literatürde adı var

*Neurosymbolic Information Extraction from Transactional Documents*
([arXiv 2512.09666](https://www.arxiv.org/pdf/2512.09666)) tam bizim yaptığımızı yapıyor:
nöral çıkarımı muhasebe kısıtlarıyla doğruluyor ve adayları mantıksal tutarlılığa göre
puanlıyor. Adı **neurosymbolic constraint validation**.

Yani yaklaşımımız özgün değil ama **doğrulanmış** — bu daha iyi bir haber.

### Ve bir hatamızı gösteriyor

Aynı literatür uyarıyor: **sert eşitlik kullanma.** Tedarikçiler satır bazında yuvarlar,
KDV gruplu satırlarda hesaplanır, iskonto eşitsiz dağıtılır.

Biz `matrah + kdv == toplam` sert eşitliğini kullanıyoruz. **Bu muhtemelen bugün sessizce
doğru okumaları eliyor** ve kullanıcıyı gereksiz yere seçimli kipe düşürüyor. Tolerans
bandı gerekiyor — kalem sayısıyla orantılı, çünkü yuvarlama hatası kalem başına birikir.

---

## 2. Şablon ailesi keşfi — ölçülmüş bulgu

**Kritik sonuç:** baskı-tarama bozulması altında **metin sinyali hayatta kalır, görsel
sinyal çöker** ([arXiv 2506.12116](https://arxiv.org/abs/2506.12116), IJDAR):

| Kodlayıcı | Temiz (ARI) | Bozulmuş (ARI) |
|---|---|---|
| DiT (görsel) | 0,99 | **0,30** |
| Donut (görsel) | 0,98 | **0,16** |
| SBERT (metin) | 0,77 | **0,74** |

Yani A9 için doğru öznitelik sırası: **normalize metin > kelime konumları > sütun yapısı >
görsel iskelet.** pHash türevleri bizim durumumuzda yanlış yatırım olurdu.

### Endüstri deseni (Intellix, ICDAR 2013)

Şablon eşleme yalnızca **değişmez konumdaki sabit kelimeler** üzerinden yapılıyor:
etiketler, satıcı unvanı. **Değişken içerik — tutar, tarih, seri no — parmak izinden
atılıyor.**

Bizde şu an ham metne bakılıyor ve `parmak_izi()` rakamları siliyor ama bu yetmiyor.

### Türkçe'ye özgü kaldıraç: VKN kontrol hanesi

VKN 10 hanelidir ve **son hane ilk 9'dan türeyen bir kontrol hanesidir.** Yani OCR'ın
okuduğu VKN'yi **doğrulayabiliriz** — hatta tek haneli hatayı onarabiliriz.

```
tmp    = (A[i] + (9-i)) mod 10
res    = (tmp · 2^(9-i)) mod 9        // tmp≠0 ve res=0 ise res=9
kontrol = (10 - (Σres mod 10)) mod 10
```

Bu, satıcı kimliğini bulanık bir problemden **neredeyse kesin bir anahtara** çeviriyor.
Şu an kullanmıyoruz — muhtemelen en büyük tek kaçırılmış fırsat.

> **DOĞRULANMADI:** algoritma tek bir topluluk kaynağından alındı; GİB dokümanıyla
> karşılaştırılmalı ve bilinen VKN'lerle sınanmalı.

### Önerilen parmak izi — üç katman

```
L0  ÇAPA      VKN (checksum doğrulamalı) — varsa sert bölme: farklı VKN = farklı aile
L1  SÖZCÜK    normalize token kümesi → MinHash(64)
              normalize: TR-upper → diakritik katla → OCR karışımı katla (0OQD→O, 1IL→I,
              5S→S, 8B→B) → rakam ve noktalamayı TAMAMEN at → len<4 ise at
L2  YAPI      metin bbox'ına göre normalize 8×16 ızgara bitmap, ±1 hücre yayma
              (kağıt kenarına göre DEĞİL — tarama kırpması yüzünden)

skor = 0.65·jaccard(L1) + 0.35·(1 − hamming(L2))     ← metin ağırlıklı, literatür böyle diyor
```

### En önemli tek fikir: aile = üyelerin çoğunluk oyu

Tek belgenin parmak izini aile parmak izi sanmak hatadır. OCR gürültüsü **belgeye özgü ve
rastgeledir**; gerçek etiket kelimeleri **her üyede tekrar eder**.

```
aile.token_sayaci[h] += 1
aile.kanonik = { h : sayac[h] >= 0.5 × uye_sayisi }    // çoğunluk oyu
```

3-4 üyeden sonra aile parmak izi **hiçbir tek belgeninkinden daha temiz** olur.
A9'un büyük kısmı bu tek fikirle kapanıyor.

### Atama: parmak izi aday getirir, denklem karar verir

```
skor ≥ 0.72          → aileye ata
0.55 ≤ skor < 0.72   → ilk 3 adayın kurallarını UYGULA;
                       belgenin kendi aritmetiği tutan adayı seç
skor < 0.55          → yeni aile
```

Doktrinimizle tam uyumlu: tanıma aday üretir, **hakem yine denklemdir.**

---

## 3. Kural itibarı (A2) — teşhis düzeltildi

Yaşadığımız olay **bir istatistik problemi değil, köken (provenance) problemi.** Kod
sürümü değiştiğinde o sürümde toplanan kanıt geçersizleşir. Hiçbir bandit algoritması
bunu tek başına çözmez — **epoch damgası** gerekiyor.

```rust
Kural { alpha: f32, beta: f32, last_seen: u64, epoch: u32 }   // baslangic 1.0, 1.0

const LAMBDA: f32 = 0.97;   // gozlem basi iskonto, yari-omur ~23
const KAPPA:  f32 = 0.20;   // epoch degisiminde saklanan kanit orani
const Z:      f32 = 1.645;  // tek yonlu %95

observe(kural, dogru_mu, kaynak, simdi):
    d = LAMBDA^(simdi - last_seen)
    alpha = 1 + d·(alpha − 1);  beta = 1 + d·(beta − 1)      // prior'a sonumle, sifira degil
    w = match (kaynak, dogru_mu) {
        (Insan, false) => 3.0,      // kullanici duzeltti: sert ceza
        (Insan, true ) => 1.0,
        (Aritmetik, _) => 0.3,
        (Zayif,     _) => 0.1,
    }
    if dogru_mu { alpha += w } else { beta += w }
    if son_3_gozlem_yanlis { beta += 5.0 }                    // nobetci

motor_surumu_degisti(kural):                                  // ← ASIL COZUM
    alpha = 1 + KAPPA·(alpha − 1);  beta = 1 + KAPPA·(beta − 1)
    epoch += 1

guven(kural) = Wilson_alt_sinir(alpha, beta, Z)               // kapali form, ~5 satir

// POLITIKA
guven ≥ 0.70          → otomatik uygula
0.40 ≤ guven < 0.70   → uygula + "gozden gecir" bayragi
guven < 0.40          → sadece aday, insana sor
n ≥ 5 && guven < 0.20 → EMEKLI ET
```

Davranış: 3 isabet/0 hata → 0,44 (otomatik **değil** — doğru). 5/0 → 0,57. 10/0 → 0,72.
1 isabet 1 hata → 0,09.

**Beş gözlem asla 1,0 vermez.** A2'nin çekirdek nedeni tam olarak buydu: `isabet/deneme`
nokta tahmini küçük n'de anlamsızdır, posterior anlamlıdır.

---

## 4. Ölçüm paneli (A10)

| Metrik | Ne söyler |
|---|---|
| **N'inci belge eğrisi** | Aile içi sıraya göre alan doğruluğu. Yükselmiyorsa öğrenme birikmiyor — A9'un tek doğrudan testi |
| **Aile şişme oranı** = aile / farklı VKN | İdeal 1-3. 10+ ise parmak izi bozuk. Tek sayıyla teşhis |
| **Şablon isabet oranı** | Mevcut aileye atanan belge yüzdesi |
| **STP (dokunulmamış işlem)** | Endüstride en iyi AP ekiplerinde gerçek değer **%25-35** |
| **Alan vs belge doğruluğu — AYRI** | 15 alanda %97 alan doğruluğu → belgelerin **%36'sında** en az bir hata |
| **Görülmüş / görülmemiş düzen ayrı F1** | CloudScan deseni — genelleme mi ezber mi ayırt eder |
| **Kural devir hızı** | 100 belgede emekli olan kural + kazananın medyan yaşı |

---

## 5. Ölçüm için gerçek veri seti bulundu (A11 / C1)

**FATURA** — [Zenodo 8261508](https://zenodo.org/records/8261508), **CC BY 4.0**,
10.000 fatura görüntüsü / **50 bilinen şablon**, 24 alan sınıfı, COCO + HF anotasyon, 363 MB.

Bu tam olarak eksik olan şey: **yer gerçeği bilinen** çok şablonlu set. Kümeleme 50 şablonu
ayırt edebiliyor mu — ölçülebilir hale geliyor.

Uyarı: FATURA **sentetik ve İngilizce.** Türkçe taranmış faturayı temsil etmez. Şablon
keşfi ölçümü için geçerli, Türkçe okuma doğruluğu için değil.

---

## 6. ⛔ LİSANS TUZAĞI — kalıcı kural

**LayoutLMv3 lisansı CC BY-NC-SA 4.0 → TİCARİ KULLANIM YASAK.**

Belge yapay zekâsında en çok atıf alan model budur ve neredeyse her öğretici onu önerir.
Bu ürüne **giremez**.

| Model | Lisans | Durum |
|---|---|---|
| LayoutLMv3 | CC BY-NC-SA 4.0 | ⛔ **YASAK** |
| **LiLT** | **MIT** | ✅ Dil-bağımsız tasarım; layout kodlayıcısı **BERTurk** ile birleştirilebilir |
| Donut | MIT | ✅ ama Türkçe için sıfırdan eğitim + halüsinasyon riski |
| MinerU | Apache-2.0 **+ ek şart** | ⚠️ **Arayüzde atıf zorunlu** |
| Surya ağırlıkları | modified AI Pubs Open RAIL-M | ⚠️ $5M gelir altı serbest — SMMM ürünü için hukuk incelemesi |
| HunyuanOCR | Tencent Community | ⚠️ Ticari kısıtlı |
| `latin_PP-OCRv5_mobile_rec` | **Apache-2.0** | ✅ Temiz |
| `ort`, `pdfium-render` (Rust) | MIT/Apache-2.0 | ✅ Temiz |

**HuggingFace gerçeği:** üretime uygun Türkçe OCR fine-tune'u **yok**. Var olan tek fiş
modeli (`pix2struct-turkish-receipts`) **CC-BY-NC-4.0** — o da yasak.

---

## 7. Önerilen sıra

| # | İş | Maliyet | Kazanç |
|---|---|---|---|
| 1 | **OCR sözlüğünü değiştir** (`ppocrv5_latin_dict`) | ½ gün | %100 deterministik kaybı kapatır |
| 2 | **Sert eşitliği tolerans bandına çevir** | 2 saat | Bugün sessizce elenen doğru okumalar kurtulur |
| 3 | **VKN checksum** — doğrula + tek hane onar | ½ gün | Satıcı kimliği bulanıktan kesine döner |
| 4 | **NFC + tr-TR casing onarımı** | 2 saat | Ayrışmış diakritikler; kendi bozulmamız |
| 5 | **Parmak izi 3 katman + aile çoğunluk oyu** | 2 gün | A9'un çekirdeği |
| 6 | **Kural itibarı: iskontolu Beta + epoch** | 1 gün | A2 — elle silme biter |
| 7 | **Ölçüm paneli** | 1 gün | A10 — tez kanıta döner |
| 8 | FATURA setiyle kümeleme ölçümü | 1 gün | A11/C1 |

1-4 arası **bir günden az toplam iş** ve dördü de bugün sessizce zarar veriyor.

---

## Kaynaklar

[arXiv 2512.09666 (neurosymbolic)](https://www.arxiv.org/pdf/2512.09666) ·
[arXiv 2506.12116 (şablon kümeleme)](https://arxiv.org/abs/2506.12116) ·
[OCRTurk, EACL 2026 SIGTURK](https://aclanthology.org/2026.sigturk-1.16/) ·
[Intellix, ICDAR 2013](https://www.weidlings.de/christoph/papers/Intellix2013.pdf) ·
[CloudScan, arXiv 1708.07403](https://arxiv.org/abs/1708.07403) ·
[RETSim, arXiv 2311.17264](https://arxiv.org/abs/2311.17264) ·
[Thompson Sampling Tutorial (Stanford)](https://web.stanford.edu/~bvr/pubs/TS_Tutorial.pdf) ·
[Weighted Majority (Littlestone & Warmuth)](https://mwarmuth.bitbucket.io/pubs/C14.pdf) ·
[Wilson sıralama (Evan Miller)](https://www.evanmiller.org/how-not-to-sort-by-average-rating.html) ·
[latin_PP-OCRv5_mobile_rec](https://huggingface.co/PaddlePaddle/latin_PP-OCRv5_mobile_rec) ·
[FATURA veri seti](https://zenodo.org/records/8261508) ·
[LiLT](https://github.com/jpWang/LiLT) ·
[gaoya (Rust MinHash, MIT)](https://lib.rs/crates/gaoya)
