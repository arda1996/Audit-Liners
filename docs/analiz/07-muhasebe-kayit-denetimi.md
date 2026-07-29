# Muhasebe Kayıt Denetimi — İlke & Kanun Kontrolü (2026-07-09)

> Kullanıcı: "muhasebe ilkeleri ve kanunlar çerçevesinde hatalı kayıtlar mevcut." Fixture seed (ornek_veri)
> + şablonlar (sablonlar.json) denetlendi. Şablonlar geçen tur düzeltilmişti; asıl boşluklar FIXTURE'da.

## Bulgular (öncelik sırası)

### F1 — ÖNEMLİ: Kira ödemesinde GVK 94 stopajı yok
Fixture: `770.01 B 4.000.000 · 191.20 B 800.000 / 102.01 A 4.800.000` (kira 40.000 + %20 KDV, peşin).
**Sorun:** İşyeri kirası **gerçek kişiden** ise GVK md.94 gereği **%20 gelir vergisi stopajı** kesilir ve
360'a yazılır. Doğrusu: `770 B 4.000.000 · 191 B 800.000 / 102 A 4.000.000 · 360 A 800.000` (net ödeme
3.200.000 + KDV 800.000 = 4.000.000; stopaj 800.000 → 360). Fixture stopajsız → hem 360 (ödenecek stopaj)
eksik hem muhtasar beyanname beslemesi hatalı. (Kiralayan TÜZEL kişiyse stopaj yok, faturalı KDV'li olur —
o zaman mevcut kayıt doğru. Fixture bunu belirtmiyor; e-ticaret işyeri kirası tipik olarak stopajlıdır.)
→ Fixture'a stopajlı kira; sablonlar.json'a "Kira ödemesi (stopajlı)" şablonu (ornek-kayitlar #17 hazır).

### F2 — ORTA: Maaş kaydı SGK işveren payını içermiyor (tam bordro değil)
Fixture: `770.02 B 18.000.000 / 335 A 12.600.000 · 360 A 3.600.000 · 361 A 1.800.000`.
Brüt 18M = net 12,6M + GV/damga 3,6M + SGK işçi 1,8M. **Eksik:** İşveren SGK payı (~%20,5 ≈ 3,7M) yok —
ne gider (770) ne borç (361) tarafında. Personel maliyeti ve 361 ödenecek SGK **eksik**; dönem kârı fazla,
632 genel yönetim gideri düşük. **Tutarsızlık:** şablon (sablonlar.json "Ücret tahakkuku") işveren payını
İÇERİYOR (2 satır 770), fixture içermiyor → fixture ile şablon çelişiyor.
→ Fixture bordrosunu tam bordroya çevir (işveren payı 770 B + 361 A eklensin) — şablonla hizala.

### F3 — KÜÇÜK/tartışma: Pazaryeri komisyonu 760.02'de
Fixture POS komisyonu 760.02 (Pazarlama Satış Dağıtım). Alternatif: 653 (komisyon giderleri) veya 654.
760 savunulabilir (satışa bağlı dağıtım gideri). Sektör kararı; hata değil, netleştirilmeli.

## Doğru olanlar (teyit)
- SMM: 621.01 B / 153.xx A (aralıksız envanter) ✓
- Ay sonu yansıtma: 760→761→631, 770→771→632 ✓ (vergiye giden yolun 0. adımı)
- KDV tahakkuku: 391/191 → 360/190 ✓ (önce doğrulandı)
- Kargo 760.01 (veresiye 320), reklam 760.03 ✓
- **Ekrandaki bordro şablonu (770/770/335/360/361) DOĞRU** — iki 770 = brüt + işveren payı (tam versiyon)

## Yapılacaklar (kayıt doğruluğu)
1. Fixture: kira stopajı (F1) + tam bordro işveren payı (F2) → hizala.
2. sablonlar.json: "Kira ödemesi (stopajlı)" + eksik senaryolar (tevkifatlı alış, stopajlı serbest meslek).
3. Fixture ↔ şablon tutarlılık ilkesi: fixture kayıtları şablonlarla aynı hesap yapısını kullanmalı.
4. (İleri) Karşı bacak/olağandışı eşleşme motoru (B.8) — bu tip hataları KAYIT ANINDA yakalamalı.
