# Audit-Liners

**Türk muhasebe ve bağımsız denetim motoru.** Belgeyi okur, muhasebe kaydını üretir,
kaydı mevzuata göre denetler.

[![Lisans: MIT](https://img.shields.io/badge/lisans-MIT-blue.svg)](LICENSE)
![Rust](https://img.shields.io/badge/Rust-2021-orange)
![TypeScript](https://img.shields.io/badge/TypeScript-React-3178c6)

> **Durum: geliştirme aşamasında.** Üretimde kullanılmaya hazır değildir.
> Ne çalışıyor, ne çalışmıyor — [docs/analiz/10-acik-isler.md](docs/analiz/10-acik-isler.md)
> dosyasında **açıkça** yazılıdır. Eksikler gizlenmez.

---

## Neden başka bir muhasebe yazılımı değil

Piyasadaki belge okuma sistemlerinin çoğu **tanımaya** dayanır: "Matrah" yazan etiketi bul,
yanındaki sayıyı al. Bu yaklaşım her fatura farklı düzende olduğu için kırılgandır ve
—daha kötüsü— **yanlış okuduğunda bunu bilmez.**

Bu proje tersini yapar: **belgenin kendi aritmetiği hangi adayın doğru olduğuna karar verir.**

```
matrah + KDV = toplam
miktar × birim fiyat = tutar
Σ kalem = matrah
rakamla yazılan = yazıyla yazılan
```

Motor tüm adayları üretir, sonra denklemi sınar. Denklem tutuyorsa okuma **kanıtlanmıştır**;
tutmuyorsa hiçbir alan otomatik doldurulmaz ve kullanıcıya sorulur.

**Güven mekanizması tanıma değil, doğrulamadır.** Bu sayede "hata oranı sıfır" bir istatistik
iddiası değil, yapısal bir özelliktir: kanıtlanmamış hiçbir şey deftere girmez.

> Bu yaklaşımın literatürdeki adı *neurosymbolic constraint validation*
> ([arXiv 2512.09666](https://www.arxiv.org/pdf/2512.09666)).

### Üç alım kipi

| Kip | Ne zaman | Davranış |
|---|---|---|
| **OTOMATİK** | Denklem kanıtlandı | Alanlar doldurulur, kullanıcı yalnız onaylar |
| **SEÇİMLİ** | Belge okundu ama kanıt yok | Belge görüntülenir, kullanıcı alanları **sırayla** seçer, her değeri onaylar |
| **MANUEL** | Belge okunamadı | Kullanıcı elle girer; girdiği değer bizim taramamıza aykırıysa **uyarı** verilir |

---

## Mimari

```
crates/domain/     Saf muhasebe çekirdeği — kuruş aritmetiği, fiş, hesap planı, TDHP
crates/api/        axum HTTP sunucusu (117 uç) + belge okuma motoru + denetim
web/               React + TypeScript arayüz, Playwright uçtan uca testler
src-tauri/         Masaüstü kabuğu (Tauri)
data/              Mevzuat ve sözlük parametreleri — 25 JSON/CSV, KOD DEĞİL VERİ
docs/              Tasarım kararları, analizler, mevzuat notları
```

### Belge okuma boru hattı

```
PDF (metin katmanlı) ──► pdftotext -bbox-layout ──┐
                                                   ├─► Kelime{metin, x0,y0,x1,y1}
Görüntü / taranmış PDF ──► RapidOCR ──────────────┘            │
                                                                ▼
                                    satırlaştırma → sütun koridoru tespiti → hücre ızgarası
                                                                │
                                        ┌───────────────────────┼───────────────────────┐
                                        ▼                       ▼                       ▼
                                   aday üretimi          kalem çıkarımı          şablon belleği
                                  (etiket→değer)        (a × b = c arayışı)      (öğrenilen kural)
                                        └───────────────────────┼───────────────────────┘
                                                                ▼
                                                     ⚖️  DENKLEM HAKEMİ
                                                                │
                                                    kanıtlandı ──┴── kanıtlanmadı
                                                         │              │
                                                    otomatik doldur   kullanıcıya sor
```

Boru hattının geri kalanı belgenin **taranmış mı dijital mi** olduğunu bilmez ve bilmesi
gerekmez — iki kaynak da aynı `Kelime` biçimini üretir.

### Sütun tespiti şablonsuzdur

Sütun sınırı bulmak için şablon gerekmez: sayfadaki kelimelerin x aralıkları izdüşürülür,
hiç dolmayan dikey şeritler bulunur. Geometrik bir olgudur, her belgede vardır.

Bir sayfa tek tip ızgara değildir — başlık bloğu 2 sütun, kalem tablosu 6 sütun olabilir.
Koridor **o satırda gerçekten boşsa** böler, kelimenin üstünden geçiyorsa bölmez.

Şablon, sütunun *ne olduğunu* saklar (miktar mı, tutar mı) — o da öğrenme katmanındadır.

---

## Muhasebe doğruluğu

Kod, kural katmanlarıyla korunur:

| Kod | Katman | Amaç |
|---|---|---|
| **K** | Ön kontrol | Kullanıcı hatasını **işlem öncesi** engeller |
| **D** | Değişmezler | Sistemin kendi hatasını yakalar (defter dengede mi, yön tutarlı mı) |
| **A** | Alım | Belge alım kipi geçişleri |
| **B** | Belge okuma | Okuma bulguları ve güven düzeyi |

Mevzuat dayanağı koda gömülmez, `data/` altındaki parametre dosyalarında durur:
oranlar, hadler, hesap eşleşmeleri, gider kataloğu, sektör tanımları.

Bazı örnek kurallar:

- **İPTAL ≠ İADE** — iptal storno kaydıdır; iade `610 Satıştan İadeler`e yazılır,
  `600` asla ters çevrilmez (MSUGT).
- **Tevkifat (KDVK 9)** — alıcı, tevkif edilen KDV'yi satıcıya ödemez; `320` hesabına
  `toplam − tevkifat` gider.
- **KKEG** — beyannamede matraha geri eklenir, **yevmiyede ters kayıt yapılmaz.**
  Defter ticari kârı gösterir, beyanname mali kârı.
- **FIFO tahsis (TBK 100)** — para faturaya değil cariye aittir; dondurulmuş ve manuel
  kararlar FIFO'nun önündedir.

---

## Kurulum

Gereksinimler: Rust (2021), Node.js 18+, `poppler-utils` (`pdftotext`, `pdftoppm`).

```bash
git clone https://github.com/arda1996/Audit-Liners.git
cd Audit-Liners
cargo test --workspace          # muhasebe çekirdeği + okuma motoru testleri
cargo run -p audit-api          # API → http://localhost:8787
```

Arayüz:

```bash
cd web && npm install && npm run dev    # → http://localhost:5174
```

Taranmış belge okuma (isteğe bağlı, ~200 MB model indirir):

```bash
bash tools/belge-venv-kur.sh
```

Model kurulu değilse görüntü okuma **sessizce yanlış okumaz** — "okunamadı, elle gir" der.

---

## Katkı

Katkılar açıktır. Üç kural:

1. **Gerçek mükellef verisi asla depoya girmez.** Ad, VKN/TCKN, adres, gerçek belge
   görüntüsü — hiçbiri. Test için anonim örnek üretin (KVKK, VUK 227).
2. **Muhasebe değişikliği dayanak ister.** Bir hesabı, oranı veya kuralı değiştiriyorsanız
   mevzuat maddesini yazın (VUK/GVK/KVK/KDVK/TMS/BDS). "Böyle daha mantıklı" yeterli değil.
3. **Bilinmiyorsa uydurulmaz.** Bir had veya oran doğrulanamıyorsa motor tahmin yürütmez,
   uyarı verir. Bu bilinçli bir tasarım kararıdır.

---

## Lisans

[MIT](LICENSE). Bu yazılım muhasebe işini **destekler**, mali müşavirin yerine geçmez.

---

<details>
<summary><b>English summary</b></summary>

**Audit-Liners** is an open-source Turkish accounting and independent-audit engine: it reads
source documents, produces the journal entries, and audits those entries against Turkish
tax and accounting law (VUK, GVK, KVK, KDVK, TMS/TFRS, BDS).

Its distinguishing idea is that **document reading is verified, not recognized.** Instead of
trusting a label match ("the number next to the word *Total*"), the engine generates every
candidate reading and lets the document's own arithmetic decide which one is correct
(`net + VAT = gross`, `qty × price = amount`, `Σ line items = net`). If the equation cannot
be proven, nothing is auto-filled and the user is asked. Trust comes from verification, so
"zero error" is a structural property rather than a statistical claim.

Column detection is template-free and geometric (vertical whitespace corridors); only the
*meaning* of a column is learned per template. Scanned images and digital PDFs converge on
the same coordinate-bearing word representation, so the rest of the pipeline is
source-agnostic.

Rust workspace (`crates/domain` pure accounting core, `crates/api` axum server) plus a
React/TypeScript front end. Primary languages Turkish and English, with German, French and
Russian in the second ring.

**Status: under development, not production-ready.** Known gaps are documented openly in
[docs/analiz/10-acik-isler.md](docs/analiz/10-acik-isler.md).

</details>
