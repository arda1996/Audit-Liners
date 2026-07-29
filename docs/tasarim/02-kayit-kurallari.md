# 0.2 — Kayıt Kuralları Motoru (saf domain)

Infra'sız, test edilebilir çekirdek mantık. Sonra Rust `ledger` crate'inde birebir uygulanır;
buradaki test senaryoları doğrudan birim testleri olur. Amaç: **temiz, dengeli, değişmez, denetlenebilir kayıt.**

## Fiş durum makinesi

```
   [oluştur]            [kesinleştir]              [storno]
 ─────────────►  TASLAK ───────────►  KESİN ───────────────►  (KESİN kalır, iptal işaretli)
                   │  ▲ düzenlenebilir   │  değişmez            + yeni STORNO fişi (ters kayıt)
                   └──┘                  └── silinemez/düzeltilemez
```

- **TASLAK:** serbest düzenlenir, numara YOK, denge şart değil. Silinebilir.
- **KESİN:** numara atanır (ayrı seri, boşluksuz), **değişmez**. Tüm doğrulama kuralları geçmiş olmalı.
- **İPTAL:** KESİN fiş silinmez; **storno** ile ters kayıt üretilir, `storno_kaynak_id` bağlanır, kaynak `iptal=true`.

## Kesinleştirme doğrulama kuralları (hepsi geçmeli)

| # | Kural | Hata |
|---|-------|------|
| V1 | **Denge:** Σborç = Σalacak (kuruş, tam eşit) | `DengeBozuk` |
| V2 | En az 2 satır | `YetersizSatir` |
| V3 | Her satır: `borc>0` **XOR** `alacak>0`; negatif yok | `GecersizTutar` |
| V4 | Hesap: mükellefe ait, **aktif**, **yaprak (muavin)** | `GecersizHesap` |
| V5 | Dönem **AÇIK** (kilitli/kapanmış → red) | `DonemKapali` |
| V6 | Fiş tarihi dönem aralığında | `TarihDonemDisi` |
| V7 | Dayanak: tip zorunluysa dolu; değilse `dayanaksiz=true` işaretle (bilgi, engel değil) | `DayanakEksik` |
| V8 | Gider/maliyet (6,7) satırında `masraf_yeri` config'e göre uyarı (zorunlu değil) | *(uyarı)* |
| V9 | Tüm tutarlar tamsayı kuruş | `ParaTipi` |

> V7 tip eşlemesi: TAHSİL/TEDİYE → zorunlu, MAHSUP → esnek, AÇILIŞ/KAPANIŞ → muaf (sistemsel).

## Numaralandırma (ayrı seri, boşluksuz)
- `(donem_id, fis_seri)` başına monotonik artan `fis_sira`; seri = fiş tipi (TAH/TED/MAH/AÇ/KAP).
- **Kesinleştirme anında** atanır (taslakta numara tüketilmez → boşluk olmaz).
- Eşzamanlılık: seri başına kilit/sequence; **boşluk denetim ihlalidir.**

## Storno (düzeltme)
- Kaynak fişin satırları **borç↔alacak ters çevrilerek** yeni MAHSUP/STORNO fişine yazılır → net etki sıfır.
- Kaynak ve storno karşılıklı bağlanır; ikisi de kayıtta kalır (iz korunur).

## Dönem döngüsü
- **KAPANIŞ:** sonuç hesapları (6,7) kapatılır → dönem net kâr/zarar (590/690) belirlenir; dönem `KAPANMIŞ`.
- **AÇILIŞ:** yeni dönem, önceki kapanışın **bilanço bakiyelerinden** otomatik AÇ fişiyle başlar (devir).

## Test senaryoları (→ birim testleri)
- **T1** Dengeli fiş kesinleşir, numara `TAH-1` atanır. ✅
- **T2** Σborç≠Σalacak → `DengeBozuk`, kesinleşmez. ❌
- **T3** Tek satırlı fiş → `YetersizSatir`. ❌
- **T4** Bir satırda hem borç hem alacak > 0 → `GecersizTutar`. ❌
- **T5** Ana/üst hesaba (yaprak değil) kayıt → `GecersizHesap`. ❌
- **T6** Kapalı döneme fiş → `DonemKapali`. ❌
- **T7** TEDİYE dayanaksız → `DayanakEksik`. MAHSUP dayanaksız → kesinleşir, `dayanaksiz=true`. ✅
- **T8** İki fiş ardışık kesinleşir → `TAH-1`, `TAH-2` (boşluksuz). ✅
- **T9** KESİN fiş düzenleme/silme denemesi → reddedilir. ❌
- **T10** Storno → ters kayıt üretir, net etki 0, her iki fiş kayıtta. ✅
- **T11** Açılış, önceki kapanış bakiyeleriyle birebir eşleşir. ✅

## Uygulama notu (0.3'te)
Bu kurallar Rust'ta **bağımlılıksız saf fonksiyonlar** olarak yazılır (DB yok) → çok hızlı birim test.
Tutar tipi: `Kurus(i64)` newtype; float'a dönüşüm yok. Hatalar `enum KayitHatasi`.
