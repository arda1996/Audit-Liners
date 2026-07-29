# Vergiye Giden Yol — ana ilke ve iş kronolojisi

**Amaç:** Her kayıt, vergiye giden yolun bir adımıdır. Sistem bu yolu OTOMATİK yürütür:
belge → fiş → defter/mizan → değerleme → kapanış → beyanname. Mizan değişince vergi
pozisyonu canlı güncellenir; kullanıcı vergi hesaplamaz, sistem türetir.

## İş kronolojisi (PDF'ler bu sıraya oturur)
| Aşama | Ne olur | Kaynak PDF'ler |
|---|---|---|
| 1. Belge | Fatura/irsaliye/makbuz/bordro doğar | Ön Büroda Muhasebe |
| 2. Kayıt (fiş) | Çift taraflı yevmiye; KDV satırı otomatik | Genel Muhasebe Kitabı, MSUGT açıklamaları |
| 3. Defter/Mizan | Yevmiye→kebir→mizan türetilir | Genel Muhasebe Kitabı |
| 4. Ay sonu vergi | KDV mahsubu (alt hesaplar), muhtasar/stopaj, bordro tahakkuku | Bordrolama, SGK |
| 5. Dönem sonu değerleme | Amortisman, reeskont, şüpheli alacak, kur, karşılık, stok NGD | TMS 2/16/19/21/36/37, VUK |
| 6. Kapanış | 7→6 yansıtma, 6→690→691/370→692→590/591 | Genel Muhasebe, KV Örnek |
| 7. Beyanname | Ticari kâr→KKEG→istisna→mali kâr→vergi | Gelir V. Rehberi, KV Örnek, Vergi Komitesi |
| 8. Raporlama/denetim | Mali tablolar, dipnot, TMS overlay, etik/denetim | TMS seti, Mesleki Etik, Kayyımlık |

## Alt hesap stratejisi (sonsuz kırılım — çekirdek ilke)
- TDHP **kılavuz**; firmalar sektöre göre sınırsız alt hesap açar (191.10, 191.20, 600.01.TR34…).
- Bizde kimlik = **kod (metin)**, derinlik sınırsız; rollup kod önekiyle (VAR ✓).
- **Kural: motorlar asla tek kodu okumaz, önek altını tarar** (kdv_mahsup düzeltildi ✓;
  kapanış/mizan zaten rollup'lı). Yeni motorlar bu kurala uyar.
- Sektör şablonu = hazır alt hesap seti + oran eşlemesi (191.01→%1, 191.10→%10, 191.20→%20).

## Fatura modülü (vergiye giden yolun kapısı) — plan
Fiş elle de girilir; ama asıl akış **fatura üzerinden**:
- Fatura satırı: mal/hizmet · miktar · birim fiyat · **KDV oranı** · **istisna kodu** (KDVK 11/13/17…).
- İstisna varsa KDV satırı üretilmez; istisna kodu fişte taşınır → KDV beyannamesi istisna satırına gider.
- Fatura → otomatik fiş: 120.xx/320.xx (cari) + 600.xx/153.xx + 391.xx/191.xx (oran alt hesabına).
- Mükerrer belge kontrolü (VAR ✓) fatura no üzerinde de çalışır.

## Kesinleştirme güvenlik katmanları
| Katman | Durum |
|---|---|
| Denge, ≥2 satır, yaprak hesap, dönem açık, tarih aralığı (V1-V6) | ✓ |
| Dayanak tipe göre zorunlu (V7) | ✓ |
| Mükerrer belge tipi+no reddi | ✓ (api) |
| Kesin fiş değişmez → yalnız İptal (ters kayıt) | ✓ |
| Karşı-bacak tutarsızlık uyarısı (hesap-kurallari.json) | plan |
| Kesinleştirme onay adımı (ikinci göz / rol) | plan |
| Fiş hash zinciri (değiştirilemez denetim izi) | plan (denetim modülü) |

## Otomatik vergi türetimi (mizan-canlı)
- KDV pozisyonu = 191/190/391 alt ağaçlarının canlı rollup'ı (VAR ✓ — /api/kdv).
- Geçici vergi matrahı = donem_sonucu() canlı (VAR — beyanname köprüsü plan).
- Kapanışta vergi karşılığı 691/370 (VAR ✓). Mali kâr köprüsü (KKEG/istisna) → vergi motoru (plan).
