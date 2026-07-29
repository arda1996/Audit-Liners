# Hesap İşleyiş Kuralları — karşı bacaklar & karşılıklı kapatma

Her TDHP hesabının kendi **işleyiş kuralı** vardır: ne zaman borçlanır/alacaklanır, hangi hesaplarla
(**karşı bacak**) etkileşir, ve hangi hesapla **karşılıklı kapanır**. Veri: `data/hesap-kurallari.json`
(kod, doğa, hizmet, borç/alacak koşulu, `karsi[]`, `kapatma`). Program bunu **karşı hesap önerisi +
doğrulama + otomatik fiş** için kullanır. Çekirdek 83 hesap işlendi; kalan ~180 (nadir) kademeli.

## İşleyişin üç boyutu
1. **Doğa & yön:** Aktif/Gider artış=Borç; Pasif/Gelir artış=Alacak. Düzenleyici (-) hesaplar ters.
2. **Karşı bacak (`karsi`):** Bir hesap tek başına hareket etmez; tipik karşı hesapları bellidir.
   Ör. 120 girince genelde 600 + 391 (satış) ya da 100/102 (tahsilat).
3. **Karşılıklı kapatma (`kapatma`):** Bazı hesaplar **yalnız belirli bir hesapla** karşılıklı kapanır.

## Karşılıklı kapatma haritası (kritik — "şu hesapla kapanır")
| Hesap | Kapanır | Ne zaman / nasıl |
|-------|---------|------------------|
| **191 İndirilecek KDV** ↔ **391 Hesaplanan KDV** | ay sonu | 391 borç / 191 alacak; fark **360** (ödenecek) veya **190** (devreden) |
| **190 Devreden KDV** → 191 | sonraki ay | indirilemeyen KDV devreder |
| **Gelir/gider (6, 69x hariç)** → **690** | dönem sonu | gelir borç / gider alacak → 690 |
| **690** → **692** | dönem sonu | (+691 vergi karşılığı) net sonuç 692'de |
| **692** → **590** (kâr) / **591** (zarar) | dönem sonu | öz kaynağa taşınır |
| **590** → **570**, **591** → **580** | sonraki yıl | geçmiş yıllara devir |
| **7xx gider** ↔ **7x1 yansıtma** | dönem sonu | 711→151, 721→151, 731→151, 741→622, 761→631, 771→632, 781→660 |
| **129 Şüpheli alacak karşılığı** ↔ **128** | tahsil/tasfiye | (ayrılırken 654 borç / 129 alacak) |
| **257 Birikmiş amortisman** ↔ **25x** | varlık çıkışı | (ayrılırken 770/730 borç / 257 alacak) |
| **501 Ödenmemiş sermaye** ↔ **500** | sermaye ödenince | |
| **193 Peşin ödenen vergiler** ↔ **371** | yıllık vergide | mahsup |
| **340 Alınan avans** ↔ **600** | fatura kesilince | |
| **196 Personel avansı** ↔ **335** | maaştan mahsup | |

## Worked örnek — KDV ay sonu (191↔391)
İndirilecek 2.000, Hesaplanan 3.000 → 1.000 ödenecek:
`391 BORÇ 3.000 · 191 ALACAK 2.000 · 360 ALACAK 1.000`. (İndirilecek fazla olsaydı fark **190 BORÇ**.)

## Programa nasıl hizmet eder (sonraki adımlar)
- **Karşı hesap önerisi:** Fiş satırında 320 seçilince → öner: 153, 191 (borç bacakları).
- **Doğrulama/uyarı:** 600 ile 153 doğrudan eşleşmesi olağandışı → uyarı (karsi listesinde yok).
- **Otomatik fiş şablonları:** "Peşin mal alışı" → 153+191 / 100; "KDV tahakkuku" → 391/191/360.
- **Kapanış motoru** (var): kapatma haritasını zaten uyguluyor (`donem.rs`).

## Veri derinliği (2 katman)
Her hesap için iki düzey:
1. **`hizmet/borc/alacak/karsi/kapatma`** — program mantığı (öneri/doğrulama/otomatik fiş). 83 hesapta var.
2. **`nitelik`** (resmi MSUGT tanımı) + ayrıntılı borç/alacak — **gerçek kaynak araştırmasıyla** doldurulur.
   Kaynak: muhasebedersleri.com per-hesap "Niteliği/İşleyişi", muhasebetr.com grup yazıları, MSUGT tebliğ.

**Araştırma durumu:** `nitelik` eklenen hesaplar (kaynaklı): **100, 102, 120, 153, 191, 320, 770** (7/262).
Bu, hedef derinliği gösteriyor — örnek (191):
> *"Mal/hizmet alımında satıcılara ödenen, hesaplanan KDV'den indirilinceye kadar bekletilen KDV hesabı."*

## ⭐ Resmi kaynak işlendi — `data/hesap-aciklamalari.json`
Kullanıcının verdiği **MSUGT Tekdüzen Hesap Planı Açıklamaları PDF**'i ayrıştırıldı:
- **205 hesap**, **158'inde resmi "İşleyişi" metni**; TDHP 3-haneli hesapların **%75'i (196/261)** kapsanıyor.
- Her hesap: `kod, ad, tanim (resmi tanım), isleyis (ne zaman borç/alacak)`.
- Örnek (320): *tanım* "faaliyet konusu mal/hizmet alımından kaynaklanan senetsiz borçlar"; *işleyiş*
  "Senetsiz borcun doğması ile alacak, ödenmesi halinde borç."
- Not: PDF OCR'ında ufak kusurlar (ör. "yapı1an"); kritik hesaplar doğrulandı.

İki veri birlikte tam resmi verir:
- `hesap-aciklamalari.json` → **resmi tanım + işleyiş** (ne/nasıl) — 196 hesap.
- `hesap-kurallari.json` → **karşı bacak + kapatma** (program mantığı) — 83 hesap (yansıtma/sonuç dahil).

**Eksik ~65** (yansıtma 7x1, sonuç 690/691/692, nadir 127/170/350...): MSUGT'de grup düzeyinde anlatılı;
karsi/kapatma `hesap-kurallari.json`'da mevcut. Kalan grup açıklamaları + kullanıcının verdiği kaynak ile tamamlanacak.

## Sonraki: programa bağla (görev #17)
UI'da hesaba tıklayınca **işleyiş paneli** (resmi tanım + işleyiş + karşı bacaklar + kapatma);
fişte **karşı hesap önerisi** + olağandışı eşleşme uyarısı. API: `GET /api/hesap/:kod/aciklama`.
