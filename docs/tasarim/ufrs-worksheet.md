# UFRS WorkSheet — VUK/TDHP → TFRS Dönüşüm Çalışma Sayfası Tasarımı

> Kaynak: UFRS worksheet araştırma kolu (2026-07-12; KGK/TFRS metinleri, Türk denetim pratiği,
> CaseWare/CCH dönüşüm modelleri). Motor temeli: [calisma-kagitlari-hesap-motoru.md](calisma-kagitlari-hesap-motoru.md).
> Doktrin bağı: iki-eksen raporlama (VUK ≠ TFRS+BDS) — bu sayfa TFRS ekseninin raporlama hattıdır.

## 0. Temel teknik: "dışarıdan atılan kayıtlar" (top-side entries)

Türk denetim pratiğinin yerleşik modeli: VUK/TDHP mizanı **hiç dokunulmadan** alınır; TFRS fark
kayıtları **defter dışında** mizanın üzerine eklenir. Deftere hiçbir şey işlenmez — "deftere sızmaz"
doktrinimizin birebir karşılığı. Akış:

```
VUK Mizan → [RJE sınıflama] → Sınıflanmış → [AJE düzeltme] → [TMS 29 endeksleme] → [EV] → TFRS Mizan → Tablolar+Dipnot
```

**Kayıt taksonomisi** (asla birleştirilmez — her kayıt tek olaydan doğar):
- **Açılış kayıtları** (yalnız TFRS 1 geçiş yılı; kümülatif etki 570/580'e)
- **Taşıma kayıtları `CF-`** (önceki dönem AJE'leri "taşı-bırak" ile: içindeki gelir tablosu
  hesapları → 570/580'e çevrilir, kalanı aynen) — Excel'de elle yapılan en hataya açık adım;
  **otomasyonun en yüksek getirili noktası**
- **AJE** (ölçüm/tanıma farkı — kâr etkiler): `AJE-16-01` (standart no gömülü numaralama)
- **RJE** (sınıflama — tutar-nötr, sunum düzeltir): `RJE-01`

Kayıt şeması: `(no, tip, standart, hesap, borç, alacak, açıklama, kaynak_ws+alan, hazırlayan, tarih)`.
Mizan satırı ↔ kayıt ↔ worksheet çift yönlü izlenebilir. Denge kontrolleri her sütun bloğunda ayrı;
kâr köprüsü (VUK ticari kâr → düzeltmeler → TFRS kârı) ayrı özet.

## 1. Sayfa kurgusu: her çalışma AYRI worksheet, ortak zarf

Her WS motor üzerinde tipli bloklardan oluşur; ortak zarf:
```
{ id, standart, dönem, durum(taslak/incelendi/kilitli), hazırlayan/inceleyen+tarih,
  girdi_refs[] (mizan/diğer WS — ID-tabanlı), parametreler{} (sürümlü, onaylı),
  hesap_alanı (korumalı formüller + izli girdi bölgeleri),
  çıktı_kayıtları[] (AJE/RJE), yayın_alanları{} (dipnot + downstream), bağımlılıklar[] }
```
**Üç yapısal ilke:** (1) WS'ler **yayın/abonelik** ile bağlanır — WS-12 ve WS-29 asla elle beslenmez,
diğer WS'lerin yayınladığı `(kalem, TFRS değer, vergi değeri)` üçlülerinden derlenir; (2) her AJE tek
olaydan doğar, `kaynak_ws+alan` referansı taşır; (3) taşıma kayıtları önceki dönem dosyasından
otomatik türetilir.

## 2. Çalışma kataloğu (18 worksheet)

| # | ID | Girdi (TDHP) | Parametreler | Çekirdek hesap | Çıktı | Sıra |
|---|---|---|---|---|---|---|
| 1 | **WS-MAP** | tüm mizan | hesap→TFRS kalemi eşleme | içe aktar, denge, eşle; eşlenmemiş hesap varken "tamam" yok | — | 1 |
| 2 | **WS-RECLASS** | yanlış sınıflı kalemler | vade/nitelik kuralları | kısa/uzun ayrımı, avanslar, YAGM ayrıştırma | RJE'ler (tutar-nötr) | 2 |
| 3 | **WS-16 MDV** | 25x, 257, 730/740/770 + SK defteri | sınıf bazlı ömür, kalıntı, yöntem, kıst(gün) | paralel amortisman VUK/TFRS; açılış/cari ayrımı | `AJE-16: B/A 257↔570/580+730/770` | 3 |
| 4 | **WS-K16 Kiralama** | 770 kira + sözleşme envanteri (defter dışı) | süre+opsiyon, ödeme planı, iskonto, muafiyet | BD→itfa tablosu→KHV amortismanı→gider iptali | açılış `B KHV/A Yük.`; dönem `B Amort+Faiz/A 770` | 3 |
| 5 | **WS-19 Kıdem** | personel master + 372/472 | iskonto, maaş artışı, devir hızı, tavan | kişi bazlı DBO (öngörülen birim kredi); hizmet+faiz+**aktüeryal(OCI)** ayrımı | `B 770+66x+OCI / A 472` | 3 |
| 6 | **WS-9a Reeskont** | 120/121/122, 320/321/322 vade dökümü | etkin faiz | itfa edilmiş maliyet; önceki dönem iptal (CF) | `B 657/A 122`, `B 322/A 647` | 3 |
| 7 | **WS-9b ECL** | 120/128/129 yaşlandırma | kova tanımları, temerrüt oranları, ileriye dönük düzeltme | karşılık matrisi × kova; VUK 129 farkı | `B Karşılık Gid./A 129` | 3 (9a sonrası) |
| 8 | **WS-2 Stok** | 15x, 780, 760 + vade + fiyat listesi | etkin faiz, NRV girdileri | vade farkı ayrıştır; kur/faiz iptal; NRV testi | `B 780/A 153`; `B 654/A 158` | 3 |
| 9 | **WS-15 Hasılat** | 600/601/602, 121, 380/480 + sözleşmeler | finansman bileşeni, edim kriterleri | vade farkı→642; cut-off; değişken bedel | `B 600/A 642+122`; sözleşme yük. | 3 |
| 10 | **WS-21 Kur** | 646/656 + dövizli envanter | kapanış kurları | maliyet içi kur farkı iptali | `B 656/570 / A 15x-25x` | 3 (2/16'yı besler) |
| 11 | **WS-23 Borçlanma** | 25x içi faiz, 780 | özellikli varlık tanımı | özellikli-değil aktifleştirme iptali + amortisman düzeltme | `B 660/570 / A 25x, B 257` | 3 (16'dan önce) |
| 12 | **WS-36 Değer Düş.** | WS-16/K16 çıktısı net değerler | NYB, DCF (projeksiyon+iskonto+uç büyüme) | geri kazanılabilir vs NDV | `B 659 / A 25x` | 4 (varlık WS'lerinden sonra) |
| 13 | **WS-37 Karşılıklar** | 379/479, 654 + dava listesi (hukukçu teyidi) | olasılık eşiği, iskonto | dava bazlı; uzun vadede iskonto | `B 654 / A 479` | 3 |
| 14 | **WS-40 YAGM** | 25x'ten ayrışan GM + ekspertiz | model (GUD/maliyet) | sınıflama + GUD farkı K/Z | RJE + `B YAGM / A 649` | 3 |
| 15 | **WS-29 Enflasyon** | TÜM düzeltilmiş mizan + kalem yaşları + **VUK mük.298 kayıtları** | TÜİK TÜFE serisi, katsayılar, parasal/parasal-olmayan sınıflaması | **VUK düzeltmesini geri çevir** → TÜFE ile endeksle → net parasal pozisyon | `AJE-29` + NPP K/Z kalemi | 5 |
| 16 | **WS-12 Ertelenmiş Vergi (ÇATI)** | tüm WS'lerin yayınları + mali zararlar | vergi oranı (dönemsel), geri kazanılabilirlik | geçici fark tablosu **otomatik derlenir**; K/Z-OCI ayrımı; efektif oran mutabakatı | `B/A EV / A/B 691+OCI` | 6 |
| 17 | **WS-TB TFRS Mizan** | WS-MAP + tüm kayıtlar | FS şablonu (SPK/KAP taksonomi) | sütunlu dönüşüm mizanı; denge + kâr köprüsü | tablolar | 7 |
| 18 | **WS-EQ Özkaynak Mutabakatı** | WS-TB + önceki dönem | geçiş tarihi, TFRS 1 muafiyetleri | VUK↔TFRS özkaynak köprüsü; CF üretimi | `CF-` kayıtları | 7 (+dönem başı) |

**Bağımlılık sırası (motorda zorlanır):** MAP → RECLASS → {21, 23} → {16, K16, 19, 9a→9b, 2, 15, 37, 40}
→ 36 → **29 (önce düzelt, sonra endeksle)** → **12** → TB → EQ. Bir WS'nin çıktısı değişince abone
WS'ler "güncel değil" bayrağı alır (motor K1/K3 sayesinde otomatik).

## 3. Türkiye bağlamı — kritik kararlar

1. **TMS 29 ↔ VUK mük.298 çifte düzeltme:** aynı şirket iki ayrı enflasyon düzeltmesi yürütür —
   VUK (Yİ-ÜFE, ROFM'lu, defterde) ve TMS 29 (TÜFE, tüm tablolar, worksheet'te). WS-29 önce VUK
   düzeltme kayıtlarını **girdi olarak alıp geri çevirmeli**, sonra TÜFE ile yeniden endekslemeli —
   aksi halde çifte düzeltme. Bu fark aynı zamanda WS-12'nin en büyük geçici fark kaynağı
   (TFRS Yorum 7: EV endekslenmiş tutarlar üzerinden). WS-29'un zorunlu iç bloğu:
   **VUK↔TMS29 fark mutabakat tablosu** — kalem bazında `VUK düzeltilmiş değer (Yİ-ÜFE) |
   TMS29 değer (TÜFE) | fark | geçici fark sınıfı` sütunlarıyla; bu tablo hem denetim kanıtı
   hem WS-12'nin doğrudan girdisidir.
2. **Çerçeve-parametrik motor:** TFRS (KAYİK) / **BOBİ FRS** (denetime tabi diğerleri; orta boy'da
   EV ihtiyari, kiralama bilançolaşması farklı) / KÜMİ FRS. WS kataloğu çerçeve profiline göre süzülür.
3. **Denetim eşikleri yıllık parametre tablosu** (koda gömülmez): 2026 için 11066 sayılı CK —
   aktif 500M / satış 1Mr TL (⚠ ikincil kaynaktan; RG metninden teyit edilecek).
4. **Dipnot yayın alanları:** her WS ikinci çıktı kanalı olarak dipnotu besler (MDV hareket tablosu ←
   WS-16; duyarlılık analizi ← WS-19; karşılık matrisi ← WS-9b; efektif oran mutabakatı ← WS-12…).

## 4. Uygulama planı (motor fazlarına bağlı)

1. **F4a:** `data/ufrs-worksheets.json` kataloğu (18 WS'nin zarf+blok şablonları, çerçeve profilleri)
2. **F4b:** WS-MAP + WS-RECLASS + kayıt defteri (AJE/RJE/CF register) — dönüşüm mizanı ucu
3. **F4c:** İlk 3 hesaplamalı WS: **WS-16** (SK defteri girdisiyle), **WS-19** (basit aktüeryal),
   **WS-9a** (reeskont) — üçü de mevcut fixture ile test edilebilir
4. **F4d:** WS-12 çatı (yayın/abonelik derlemesi) + WS-TB + kâr köprüsü
5. **F4e:** WS-29 (TÜFE serisi `data/`e; VUK geri çevirme mantığı) + WS-EQ + CF otomasyonu
6. **F4f:** kalan WS'ler (K16, 9b, 2, 15, 21, 23, 36, 37, 40) + dipnot yayınları

Sıralama gerekçesi: F4c'deki üçlü en yaygın, girdileri defterde mevcut, hesapları kapalı-form;
WS-29 en karmaşık (endeks serisi + geri çevirme) ama WS-12'den önce gelmek zorunda değil —
geçiş döneminde WS-12 endekssiz farklarla da çalışır, WS-29 eklendiğinde otomatik güncellenir.
