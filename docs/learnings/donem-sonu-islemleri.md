# Dönem Sonu İşlemleri — Tam Harita (öğrenme)

Dökümantasyondan derlendi (muhasebetr.com, muhasebenet.net, muhasebedunyasi.com).
"A-Z" kapsamı için: kapanış sadece son adım; öncesinde **envanter + değerleme düzeltmeleri** var.
Bunlar ileride domain'e **dönem sonu kuralları** olarak eklenecek (şimdilik bilgi/harita).

## 1. Envanter
- **Muhasebe içi envanter:** kayıtlı (defter) durum.
- **Muhasebe dışı envanter:** fiili sayım/tespit. İkisi karşılaştırılır → düzeltme kayıtları.

## 2. Değerleme & düzeltme kayıtları (VUK) — dönem sonu fişleri
| Konu | Tipik kayıt |
|------|-------------|
| Kasa sayım farkı | Noksan 197→risk; fazla 397; sonra 689/671 ile sonuçlandırma |
| Yabancı para / kur değerleme | Kâr `646`, zarar `656` (kasa/banka/alacak/borç) |
| Alacak/borç **reeskont** | `122/322` → gelir `642` / gider `657` |
| **Şüpheli alacak** karşılığı | `128 Şüpheli Alacak` / `129 Karşılık` · gider `654` |
| **Stok** değerleme / değer düşüklüğü | `158` / gider `654`; envanter farkı |
| **Amortisman** | gider `770/730/...` / birikmiş `257/268` |
| Menkul kıymet değer düşüklüğü | `119` / `654` |
| Dönemsellik: peşin gider/gelir | `180/280`, `380/480` ayrımı |
| Gider/gelir **tahakkuku** | `181/381` |
| **KDV tahakkuku** | `391` / `191` → fark `360` (ödenecek) veya `190` (devreden) |

## 3. Sonuç hesaplarının kapatılması (kapanış) — doğrulanmış sıra
0. **Maliyet (7) → gelir tablosu (6):** yansıtma (711–781) ile devir; 7 ve yansıtma kapanır.
1. **Gelir/gider (6) → 690:** gelir hesapları BORÇ, gider hesapları ALACAK → `690 Dönem Kârı/Zararı`.
2. **Vergi karşılığı:** `691 BORÇ / 370 ALACAK` (kurumlar/gelir vergisi karşılığı).
3. **690 → 692:** `690 BORÇ · 691 ALACAK · 692 ALACAK` (net kâr 692'de).
4. **692 → öz kaynak:** kâr `692 BORÇ / 590 ALACAK`; zarar `591 BORÇ / 692 ALACAK`.
5. **Bilanço hesapları (1–5) → kapanış fişi:** aktifler ALACAK, pasifler BORÇ ile sıfırlanır.

## 4. Açılış (yeni dönem)
Kapanışın tersi: bilanço bakiyeleri (1–5) BORÇ/ALACAK ile yeniden açılır. Sonuç hesapları (6,7) sıfırdan.

## Bizim sıramıza etkisi
- Çekirdek kayıt + defter + muavin + cari (A-C) **var/yapılıyor**.
- **D (kapanış/açılış virman):** yukarıdaki adım 0–5 + 4 → `kapanis()`/`acilis()` otomatik üretimi.
- **Değerleme düzeltmeleri (madde 2):** "tam yapı"nın ileri fazı — her biri ayrı dönem sonu kuralı/sihirbazı.
> Detaylı tasarım: `docs/tasarim/donem-kapanis-acilis.md`.
