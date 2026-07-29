# Dış Veri Katmanı — "Program Devasa Bir API" Doktrini ve Rota

> Kullanıcı kararı (2026-07-06/08): Yaptığımız iş veriye muhtaç; program veriyi diğer yazılım ve
> donanımlardan TOPLAYABİLİR ve dışarıya SUNABİLİR olmalı — devasa bir API gibi. Bu tur bağlantı
> kurmadı; **veri iskeletini** (data/veri-kaynaklari.json) ve **rotayı** belirledi.

## 1. Mimari ilkeler
1. **Hexagonal ingest:** domain saf kalır. Her dış kaynak = bir adaptör (ileride `crates/ingest`).
   Adaptör sözleşmesi tek: `kaynak → normalize seri (tarih, deger, birim)` + ham yanıt arşivi.
   Banka PDF'i ile TCMB XML'i aynı arayüzden akar (kaynak ha dosya ha web).
2. **Kanıt saklama:** ham yanıt (XML/JSON/PDF) kaynak URL + alınma tarihiyle değişmez arşive yazılır —
   BDS 500 denetim kanıtı + hesap yeniden üretilebilirliği (fixture determinizm ilkesinin dış veri hali).
3. **Üçlü taşıma:** programda hiçbir dış değer çıplak dolaşmaz: `{deger, kaynak, alinma_tarihi}`.
   Çalışma kağıdına/beyanname taslağına kaynak referansı otomatik düşer (beyanname doktrini genelleşir).
4. **Manuel override:** kaynak çöktüğünde/abonelikliyse SMMM değeri elle girer — izli, dayanaklı
   (vergi_duzenlemeler kalıbı). Hiçbir hesap dış kaynağa KİLİTLENMEZ.
5. **Scrape en son çare:** resmi API/XML varsa scrape yok. Scrape gerekiyorsa seçiciler VERİDE tutulur
   (banka-profilleri kalıbı) — site değişince veri güncellenir, kod değişmez.
6. **Dışa sunum (uzak faz):** aynı normalize seriler `/api/veri/:kaynak` uçlarından dışarı da verilir —
   program veri TÜKETİCİSİ olduğu kadar veri SAĞLAYICISI (devasa API).

## 2. İhtiyaç → mevzuat → kaynak eşlemesi (araştırma sonucu)

| İhtiyaç | Dayanak | Birincil kaynak | Erişim | Durum |
|---------|---------|-----------------|--------|-------|
| İşlem günü kuru | VUK 280 / TMS 21 | TCMB günlük bülten | XML, anahtarsız | ✅ canlı doğrulandı (USD 46.6337, bülten 2026/122) |
| Geçmiş tarihli kur | değerleme/denetim tekrar hesabı | TCMB arşiv | XML, anahtarsız | ✅ canlı doğrulandı (02.01.2026 → 42.8810) |
| **Dönem sonu değerleme kuru** | VUK 280 — **GİB tebliğ kuru esas** | GİB yıl sonu tebliği (TCMB alış yedek) | scrape/parametre | ⬜ |
| Reeskont oranı | VUK 281/285 — TCMB **avans** oranı | TCMB reeskont-avans sayfası (RG) | scrape/parametre | ⬜ |
| TÜFE / Yİ-ÜFE | TMS 29 / VUK mük.298 | **EVDS** serileri | REST, api-key | ⬜ anahtar alınacak |
| TMS 19 iskonto | uzun vadeli DİBS verimi | EVDS DİBS/verim serileri | REST, api-key | ⬜ |
| TFRS 16 alternatif borçlanma | TCMB ticari kredi ağırlıklı faiz | EVDS | REST, api-key | ⬜ |
| **WACC / ERP + Türkiye CRP** | TMS 36, TFRS 13 (Big4 pratiği) | **Damodaran (Stern)** + Kroll çaprazı | dosya indir/scrape + manuel | ⬜ 6 aylık ritim |
| Hadler + takvim | yıllık tebliğler | GİB | scrape | ⬜ vergi-parametreleri'ni besler |
| Piyasa referans faizi | TFRS 16/9 kıyas | BIST TLREF | scrape | ⏸ düşük öncelik (EVDS yeter) |

**Kilit bulgular:** (a) TCMB kur verisi anahtarsız XML'le tam çözülüyor — reverse gerekmiyor;
(b) EVDS "tek kapı": TÜFE, DİBS, kredi faizi tek anahtarla — TÜİK/Hazine ayrı adaptör istemez;
(c) İki eksen ayrımı veri katmanında da yaşıyor: vergi ekseni TCMB/GİB (resmi), bağımsız denetim
ekseni Damodaran/Kroll (doktrin) — kaynak kaydındaki `eksen` alanı bunu taşır;
(d) Kroll aboneliksiz tam çekilemez → manuel override asıl yol (zaten ilkemiz).

## 3. Rota (İş listesi J bölümü)
1. **J.1 ✅ Veri iskeleti** — data/veri-kaynaklari.json (bu tur; 9 kaynak, eksen/erisim/kullanim/dogrulandi)
2. **J.2 İngest adaptör sözleşmesi** — crates/ingest: `VeriKaynagi` trait (çek → normalize → arşivle);
   üçlü taşıma tipi `{deger, kaynak, alinma_tarihi}` domain'e uyuyan modül olarak
3. **J.3 TCMB XML adaptörü** — İLK CANLI KAYNAK (anahtarsız, doğrulanmış): analiz sayfasındaki
   deterministik örnek kur serisini gerçek seriyle değiştirir; D′ kur değerlemesinin girdisi
4. **J.4 EVDS adaptörü** — kullanıcı api-key aldıktan sonra: TÜFE (TMS 29), DİBS (TMS 19), kredi faizi (TFRS 16)
5. **J.5 Dönemsel kaynaklar** — Damodaran/Kroll içe aktarma + manuel override UI; TCMB avans + GİB
   tebliğ değerleri için "değişiklik uyarısı + parametre güncelle" akışı
6. **J.6 Kanıt arşivi** — ham yanıt saklama + çalışma kağıdında {deger, kaynak, tarih} gösterimi
7. **J.7 ⏸ Dışa API** — /api/veri/* uçları (kalıcılık + çoklu mükellef sonrası)

## 4. Bağlantı noktaları (mevcut sisteme)
- Analiz "Aylık & kur" → kur_serisi() örnek verisi J.3 ile gerçeğe döner (not zaten ekranda)
- D′ değerleme sihirbazları → kur (VUK 280) + avans oranı (VUK 281) buradan
- Denetim kağıtları (I.4 uzman değerlendirme) → Damodaran/Kroll kıyas değeri buradan
- vergi-parametreleri.json ↔ GİB izleyicisi (J.5) — dogrulandi:false bayrakları otomatik teyide döner
