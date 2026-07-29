# Maliyet Muhasebesi (7) + Vergiyi Doğuran Olay

Çekirdeğin atlanmaması gereken iki ayağı: **7'li maliyet akışı** (sektörel) ve **vergiyi doğuran olay**
motoru. İkisi de `domain`'de modellenir (veri-güdümlü), raporlama değil **kayıt** tarafıdır.

## A. 7/A maliyet akışı
Gider doğrudan 6'ya yazılmaz: **gider çeşidi → gider yeri (masraf_yeri) → 7/A fonksiyonel → yansıtma → hedef.**

| 7/A | Yansıtma | Hedef |
|-----|----------|-------|
| 71 Direkt İlk Madde · 72 Direkt İşçilik · 73 GÜG | 711/721/731 | 151 Yarı Mamul → 152 Mamul |
| 74 Hizmet Üretim Maliyeti | 741 | 622 |
| 75 Ar-Ge | 751 | 630 |
| 76 Pazarlama | 761 | 631 |
| 77 Genel Yönetim | 771 | 632 |
| 78 Finansman | 781 | 660 |

**7/B (küçük işletme):** gider çeşidi (790–797) → 798 yansıtma → maliyet/6. Sektör şablonu 7/A↔7/B seçer.
> Not: yansıtma hesapları (711…798, 701) **alacak** çalışır — `hesap-plani-format.md` istisnası.

## B. Sektörel akış (sektör şablonunun sebebi)
| Sektör | Akış | Satış maliyeti |
|--------|------|----------------|
| Ticaret | 153 alış → — | 621 |
| Üretim | 150→710→151→152 (+720,+730) | 620 |
| Hizmet | 740 | 622 |
| İnşaat (yıllara yaygın) | 170 maliyet / 350 hakediş | özel dönemsellik |

`masraf_yeri` = muhasebedeki **gider yeri**. **Sektör şablonu** (config): 7/A↔7/B, akış tipi,
varsayılan gider yerleri, varsayılan hesap eşlemeleri. Reel finansal tablolardan kurgulanır.

## C. Vergiyi doğuran olay motoru
**Tanım:** vergi borcunun hukuken doğduğu an (VUK md.19, KDVK md.10).

| Vergi | Doğuran olay | Sonuç |
|-------|--------------|-------|
| KDV | **Teslim / hizmet ifası** (ödeme değil) | Veresiyede bile anında 191/391 |
| Gelir/Kurumlar V. | **Tahakkuk** (hak ediş) | Nakitten bağımsız |
| Stopaj (GV) | Ücret/kira tahakkuku | 360 ödenecek |
| Damga V. | Kâğıdın düzenlenmesi | 360 ödenecek |

**Model:** her **işlem tipi** → bir **vergi olayına** bağlı → vergi satırları (KDV/stopaj/damga)
**oran tablosundan otomatik türetilir.** Bu "vergiyi doğuran unsurlar motoru" = veri-güdümlü kural katmanı.
- `VergiOlayi { tip, matrah, oran, hesap }` — orana göre satır üretir.
- KDV oranları (%1/%10/%20), tevkifat (kısmi KDV), istisna durumları config'te.

## Tasarım etkisi (domain)
1. **Maliyet akışı**: fiş üretimi 7/A↔7/B ve yansıtma kurallarını bilmeli (sektör şablonundan).
2. **Vergi kuralları motoru**: işlem → vergi olayı → vergi satırı (oran/istisna/tevkifat tablosu).
3. **Sektör şablonu** boyutu somutlaşır: maliyet şeması + gider yerleri + hesap eşlemeleri.
4. Bunlar dikey dilimde **fiş üretimini** zenginleştirir; çekirdek kuralları (denge/değişmezlik) bozmaz —
   üstüne **kural katmanı** olarak eklenir (genişletilebilirlik).
