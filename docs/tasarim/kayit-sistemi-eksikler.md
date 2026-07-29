# Muhasebe Kayıt Sistemi — Derin Eksik Analizi

Kaynak: kendi kodumuz + MSUGT + denetim raporu + SMMM günlük pratiği (Luca/Zirve akışları).

## KATMAN 1 — Kayıt yaşam döngüsü (EN KRİTİK, uygulanıyor)
| # | Eksik | Neden kritik |
|---|-------|--------------|
| 1 | **İptal (storno) UI/API'de YOK** | Domain'de `iptal_fisi` var ama uç/buton yok → kesin fiş düzeltilemiyor. Kayıt sistemi iptalsiz olmaz. |
| 2 | **Yevmiye madde numarası YOK** | VUK: yevmiye maddeleri MÜTESELSİL numaralanır. Bizde seri-sıra (TAH-1) var, global yevmiye no yok → yasal defter üretilemez. |
| 3 | **Taslak kaydetme YOK** | UI tek buton "Kesinleştir" — incele/onayla akışı yok. Gerçek akış: kaydet(taslak)→kontrol→kesinleştir. |
| 4 | **Ters bakiye uyarısı YOK** | MSUGT 100: "Kasa hiçbir şekilde alacak bakiyesi VERMEZ." Kayıt kasayı eksiye düşürüyorsa sistem uyarmalı (genel kural: doğasına ters bakiye). |
| 5 | **Fiş şablonları YOK** | En sık 10 kayıt (ornek-kayitlar.md) elle giriliyor. Şablon = hız + hata azaltma. |
| 6 | **Denge farkı otomatik tamamlama YOK** | Muhasebeci kalan farkı son satıra elle hesaplıyor — tek tuş olmalı. |

## KATMAN 2 — Doğruluk korumaları (sıradaki)
7. **Karşı bacak uyarısı** — `karsi[]` verisi hazır, motoru yok (600↔153 olağandışı → uyar).
8. **Geriye dönük tarih uyarısı** — son kesin fişten eski tarihe kayıt → kronoloji bozulur (VUK 10 gün kuralı).
9. **Aynı hesap hem borç hem alacak** aynı fişte → uyarı (virman istisna).
10. **Taslak düzenle/sil** (3'ün devamı: tam CRUD).
11. **Kullanıcı + zaman damgası** — `olusturan/olusturma` tasarımda vardı, koda girmedi (denetim izi).

## KATMAN 3 — Kayıt hızı / operasyon (sonra)
12. Klavye akışı (Enter ile alan-satır geçişi, kısayollar).
13. KDV yardımcısı: matrah satırı girilince %X KDV satırı önerisi.
14. Fiş arama/filtre (tarih/hesap/tutar/tip); fiş kopyala.
15. Dönem yönetimi UI (aç/kilitle; şu an tek hardcoded dönem).
16. Yevmiye defteri + defter-i kebir ekranları (motor hazır).

## Uygulama sırası
Bu tur: 1, 2, 4, 5, 6 (+ FisOzet'e durum). Sonraki: 3+10 (taslak CRUD), 7, 8, 16.

## GÜNCEL DURUM (2026-07-03 — yeniden tespit)
TAMAM: 1 (iptal, VUK217), 2 (yevmiye no), 4 (ters bakiye uyarısı), 5 (şablonlar), 6 (± fark),
16 (yevmiye/kebir/muavin+TXT ekranları), kronoloji koruması (API), mizan kebir filtresi + kebire iniş,
banka ekstresi alanı + eşleştirme, gelen belge kutusu (e-Fatura/e-Arşiv) + kayda bağlama, lite/sayfalı büyük veri.

KALAN (öncelik sırasıyla):
1. **Taslak CRUD** (3+10) — kaydet/düzenle/sil; şu an tek yol doğrudan kesinleştirme.
2. **Dönem yönetimi UI** — kapanış/açılış ekranı (motor hazır: kapanis/acilis/bilanco_kapanis); dönem kilidi görünür değil.
3. **Karşı bacak uyarısı** (7) — karsi[] verisi hazır, kesinleştirmede kontrol yok.
4. **Fiş arama/filtre** (14) — 19k fişte tarih/hesap/tutar/no araması yok.
5. **Kullanıcı + zaman damgası** (11) — denetim izi alanları.
6. **Dönem sonu değerleme** (görev #4) — amortisman/reeskont/şüpheli alacak/kur sihirbazları.
7. **Çoklu dönem/mükellef** (görev #5) — tek hardcoded dönem.
8. **Kalıcılık** (görev #12) — bellekte; muhasebe A-Z bitince gömülü Postgres.
9. Banka eşleştirmeden **otomatik fiş üretimi** (eşleşmeyen satırdan taslak kayıt önerisi).
