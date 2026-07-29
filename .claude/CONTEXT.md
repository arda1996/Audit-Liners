# CONTEXT.md — Kümülatif Proje Beyni

> **Bu dosya bağlam kopmasına karşı asıl çıpadır.** Context dolup özetlenince (compaction)
> veya yeni oturum başlayınca **önce bunu oku** — tüm zinciri buradan kur. Kronoloji değil,
> *biriken durum* tutar: doktrinler → mimari → nerede kaldık → sıradaki.
>
> **Güncelleme kuralı:** Her anlamlı kilometre taşında (yeni doktrin, tamamlanan dilim, yön
> değişimi) ilgili bölümü elle güncelle. Ham tur-tur akış `.claude/sessions/journal.md`'de
> (otomatik, salt-ekleme). Kalıcı tekil gerçekler `~/.claude/.../memory/*.md`'de.
> **Son güncelleme:** 2026-07-18

---

## 0. Kimlik

**Audit-Liners** — Türkiye muhasebe + bağımsız denetim yapan, hem tarayıcıda hem lokalde
çalışan sistem. **Hedef kullanıcı:** SMMM / Mali Müşavir (çok mükellefli).
**Çalışma tarzı (KULLANICI, kalıcı):** "goal komutuyla çalışmıyorum — amacım her noktayı
*birlikte* yapmak ve bana da bir şeyler öğretmen." → Tek atışta bitirme; araştır, anlat,
onun kararıyla ilerle. Sık geri bildirim: frontend'i "çok kötü" bulur, tekrardan hoşlanmaz.

---

## 1. Değişmez Doktrinler (ihlal etme — hepsi kullanıcı kararı)

1. **İki eksen ayrı:** Vergi denetimi/raporlama = **VUK**. Bağımsız denetim/raporlama =
   **TFRS + BDS**. Karıştırma. → [[iki-katmanli-raporlama]]
2. **Para = tamsayı kuruş** (`Kurus` i64 newtype). Float YASAK. Oranlar ppm (milyonda bir),
   ara çarpımlar i128/i128.
3. **UFRS düzeltmeleri deftere ASLA işlenmez** — top-side (WTB üstü) kayıtlar. Defter VUK'ta
   kalır; TFRS ayrı katman.
4. **Her UFRS/beyanname kaydında zorunlu:** dayanak + denetçi notu + değerleme yöntemi.
   Kayıt = referans/dayanak; format otoritesi ayrı; manuel müdahale katmanı şart, izli
   (deftere işlemez). → [[beyanname-doktrini]]
5. **Program muhakemeyi besler, karar denetçinindir.** "Yazılım yardımcı, denetçinin kendisi
   değil." Otomatik hesap → öneri; onay insanda.
6. **Muhasebe hatası görürsen** TMS/TFRS/mevzuattan örnekleyerek uyar. → [[standartlara-gore-uyari]]
7. **Yasal sınırlar (RED):** learnReverse banka otomasyonu KALICI RED. Emlak ilan sitelerinden
   otomatik scraping = yasal risk + savunulamaz kanıt → denetçi manuel getirir. Veri çekme:
   banka-dışı, read-only, transfer yok. → [[site-kesif-ve-learnreverse]] [[degerleme-kanit-doktrini]]
8. **MT940/CAMT.053 ≠ gerçek dekont.** MT940 = eşleştirme yakıtı; gerçek dekont = denetim
   delili. Üçlü mutabakat: dekont ↔ MT940 ↔ fiş. → [[banka-eslestirme-uclu-mutabakat]]
9. **TDHP tam olmalı** — eksik hesap muhasebe bütünlüğüne aykırı.

---

## 2. Mimari Harita (nerede ne var)

- **`crates/domain/`** — bağımlılıksız çekirdek (std+thiserror). `para`(Kurus), `hesap`,
  `fis`, `kurallar`(V1-V9 doğrulama, kesinleştir, storno, açılış), `hesap_plani`. Testli.
- **`crates/api/src/main.rs`** (~3900 satır) — axum, bellekte state, `:8787`. TÜM UFRS/senaryo/
  mizan/beyanname uçları burada. En çok düzenlenen dosya.
- **`web/src/App.tsx`** (~2500 satır) — React18+TS+Vite, HashRouter (react-router-dom@7).
  En çok düzenlenen frontend. `web/src/HesapSecici.tsx`, `web/src/styles.css` (tasarım sistemi).
- **`data/`** — veri-güdümlü çekirdek: `tdhp.csv` (TDHP), `ufrs-calismalar.json` (v6, 18 çalışma
  kataloğu), `ufrs-parametreleri.json` (oranlar — hardcode yasak), `duran-varlik-envanteri.json`
  (37 varlık, 3 seviye), `sektorler.json` (10 sektör).
- **`docs/tasarim/`** — karar dokümanları (ufrs-worksheet, degerleme-veri-mimarisi,
  frontend-routing-plani, gorsel-format-kutuphanesi=kendini güncelleyen, tms1-siniflandirma-icerik…).
- **`.claude/completions/`** — her tamamlanan dilimin kaydı (kronoloji burada; 0 token, auto-load YOK).
- **Hafıza:** `~/.claude/projects/-Users-arda-Desktop-Audit-Liners/memory/` (kalıcı tekil gerçekler).

**Çalıştırma:** backend `cargo run -p api` (:8787) · frontend `cd web && npm run dev` (:5173).

---

## 3. UFRS WorkSheet — Kavramsal Çekirdek (şu anki ana cephe)

**Değer zinciri (Big-4 WTB deseni):** VUK mizan → AJE → Düzeltilmiş → RJE → CF(devir) → TFRS.
Firmalar "insanlar gibi kümülatif ilerler" → dönem kesinleşir, sonraki dönem AJE/RJE yeniden atılır.

**Köken (provenance) modeli:** her hesap satırı kaynak taşır (defter/parametre/formül/sonuç) +
müdahale bayrağı. Manuel müdahale edilen bakiyenin öncesi/sonrası izlenir (`hs_defter/param/formul/sonuc`).

**Senaryo motoru (SON büyük iş — denetçi hesap aramaz, DURUM seçer):**
Standart, her çalışmanın kaydını zaten çizmiştir. Denetçi senaryoyu seçer → sistem bacakları kurar.
- **Ertelenmiş vergi otomatik (TMS 12.61A geri izleme):** kanal ana kaleme göre değişir.
  - K/Z kanalı → **691**; OCI kanalları → **522.90** (yeniden değerleme), **549.90** (aktüeryal).
  - EV varlık **283** (EVV), EV yükümlülük **483** (EVY).
  - `ev_tutar = (tutar × kv_ppm + 500_000) / 1_000_000`. KV oranı `ufrs-parametreleri` (%25 varsayılan).
- **Doğrulanmış örnek:** GUD Artışı 100k → 252 B / 522.90 A + 522.90 B25k / 483 A25k (OCI,
  net fon 75k). GUD Azalışı → 659.90 B / 252 A + 283 B25k / 691 A25k (K/Z). İkisi de dengeli.
- Uç: `POST /api/ufrs/calisma/:id/senaryo`. Veri: `ufrs-calismalar.json` içindeki `senaryolar[]`
  + `ev_hesaplari` haritası. **Şu an sadece WS-GUD-MDV, WS-KIDEM, WS-ECL'de tanımlı.**

**Değerleme kanıt doktrini:** KGS = Kaynak Otoritesi × Veri Yeterliliği × Zaman Tazeliği ×
Emsal Yakınlığı (çarpım). TFRS 13 Seviye 1/2/3. Düşük güven → güven aralığı düşürülür.
→ [[degerleme-kanit-doktrini]]

---

## 4. Şu An Nerede (kümülatif durum, 2026-07-17)

**Ayakta ve doğrulanmış:**
- Dikey dilim: fiş kaydet → mizan uçtan uca (domain kuralları + api + web).
- TDHP tam seed; çoklu mükellef "aktif+arşiv swap"; 10 sektör; kullanıcı giriş/yetki.
- UFRS WorkSheet: 18 çalışma kataloğu, sekmeli ekran, alt-kırılım hesap motoru, WTB formatı +
  kümüle devir, duran varlık envanteri (satır detayı), köken modeli, tasarım sistemi (katman 1),
  HashRouter (routing adım 1-2), hedef hesap akışı, **senaryo/ertelenmiş vergi motoru**.

**En büyük boşluk:** **Kalıcılık YOK** — her şey bellekte (api state). DB-per-mükellef planı
`docs/tasarim/coklu-mukellef-db.md`'de ama uygulanmadı.

---

## 5. Sıradaki (öncelik sırası — kullanıcı onayıyla)

**⭐ ANA CEPHE: Kayıt sistemi yeniden kuruluşu** — 2026-07-17 üç paralel ajan araştırması bitti:
- **Teşhis ("Üç Yarım Yol"):** hesapla motoru (5/18, EV'siz) + senaryo motoru (3/18, tutarsız) +
  statik çipler (18/18, yönsüz) tek boru hattında birleşmemiş; `@hedef` anlam kayması (UI "hangi
  hesap için" ↔ motor "hangi hesaba işlenir" — ECL'de 120 seçilirse alacak silinir!);
  `hesap-kurallari.json` atıl; WTB "+ Kayıt (pencere)" yapısal çıkmaz (baz alanı formda yok).
- **Raporlar:** `docs/tasarim/kayit-sistemi-teshis-A3.md` (B1-B12 doğrulama envanteri, C1-C6
  doldurma boşlukları, D1-D8 state riskleri) · `spiral-standart-haritasi-A1.md` (45 senaryo,
  V01-V42 kural kataloğu madde numaralı, topolojik çalışma sırası: 29→8→10→21→23→2/16A→16G/40→
  36→19/37/ECL→28→**12 en son**→33/1) · `denetci-pratikleri-arastirma-A2.md` (CaseWare tip modeli,
  BDS 230 60-gün kilidi, SUD havuzu, taslak-onay akışı; KGS'li).
- **Sentez planı:** `docs/tasarim/kayit-sistemi-sentez-plani.md` — 5 faz. **FAZ 0 + FAZ 1 BİTTİ
  (2026-07-18, doğrulandı):** pencere formu düzeltildi (ilk başarılı pencere kaydı), senaryo ucu
  hedef doğrulamalı, katalog **v7 = 60 senaryo / 16 çalışma** (`hedef_onekleri`, `tutar_kaynak`,
  `kurallar[]` V-kodları, `ev_kanal:"yok"`, `cift_tutar`+`pay`+bacak `kanal`); motor karma simetri
  senaryolarında tutarı böler (ana/ikinci/kalan) ve **EV'yi kanal bazında** kurar (12.61A bacak
  bazında — curl doğrulandı: 100k/30k azalış → OCI'de 7,5k + K/Z'de 17,5k EV, dengeli); simetri
  sınırı ihlali madde referanslı RED; kapsam dışı hedef = uyarı (karar denetçinin). ECL/NRV'de
  karşılık bacağı SABİT (@hedef anlam hatası çözüldü). TDHP'ye 648/658/698 eklendi.
  Ayrıntı: `.claude/completions/2026-07-17-faz0-faz1-senaryo-v7.md`.
  **FAZ 2 çekirdeği BİTTİ (2026-07-18):** B1 doğa uyarısı (hesap-kurallari.json artık UFRS
  hattında okunuyor; KuralRow'a ad+doga eklendi), B2 RJE kâr-nötrlüğü RED, B3 hayalet-ws RED,
  B4 mükerrer 409 (vazgeç sonrası serbest), B7 katalog-değer RED. **Kalıcı test cihazları:**
  `tools/senaryo-tarama.py` (60/60, T1-T7) + `tools/kayit-dogrulama-testi.py` (9/9, K1-K8) —
  her değişiklikte koş (API açıkken; kayıt testi taze state ister).
  **Sıradaki:** V01-V42'nin tamamı motor kuralı → FAZ 3 (taslak→kabul/red + SUD + 522 geçmişi,
  çipler senaryodan türesin) → FAZ 4 (spiral sıra + kirli bayrak + WS-EV derleme).
  Katalogsuz: WS-ISTIRAK/WS-TESVIK/WS-MODV/TMS 8 açılış.

**Frontend dönüşümü (Figma katalog, 2026-07-18):** kullanıcının Figma "Atelyé" buton/açılır-sayfa
kataloğu analiz edildi → `docs/tasarim/figma-buton-katalogu.md` (tokenlar, 11 buton deseni, 5 açılır
desen, DM Mono veri dili, tek animasyon imzası cubic-bezier(0.22,1,0.36,1)) + dönüşüm planı
`docs/tasarim/frontend-donusum-plani-figma.md` (F1 token/font swap → F2 butonlar → F3 modal/drawer/
toast → F4 UFRS inline→utility → F5 kalan). Beyaz tema esas (krem zemin ALINMAZ, --bg #FAF9F7).
KARAR (kullanıcı): beyaz tema + **bordo #6B1E2E birincil**. **F1+F2 UYGULANDI (2026-07-18):**
token seti + self-host fontlar (Jost/DM Mono/Playfair) + 2px köşe + sıcak gölge + bordo butonlar +
kare mono rozetler (ayrıntı: `.claude/completions/2026-07-18-atelye-tema-f1-f2.md`). Kalan: F3
(modal/drawer/toast) → F4 (UFRS inline→utility) → F5 (kalan ekranlar + dark cila). Kapalı-dünya
hesap seçimi de bitti (standart kapsamı dışı hesap seçilemez + backend RED; TMS 16 → 23 hesap).

Sonrası: net gösterim kontrolü (522 vergiden net mi) · routing adım 3-7 (`/ufrs/c/:calismaId`) ·
TanStack Table · değerleme D1-D9 (KGS motoru, TCMB EVDS) ·
**kalıcılık (en büyük)** + taslak CRUD + tam SoD.

---

## 6. Süreklilik Sistemi (bu dosyanın çalışma şekli)

- **CONTEXT.md** (bu dosya) — elle küratörlü kümülatif beyin. Kopmaya karşı ilk okunacak.
- **`.claude/sessions/journal.md`** — Stop hook her turda tek satır *ekler* (salt-ekleme,
  otomatik). Ham breadcrumb; CONTEXT güncellenmeden arası burada durur.
- **`.claude/sessions/snapshot.md`** — son turun anlık görüntüsü (değişen dosya + son istek/özet).
- **Enjeksiyon:** oturum/gün başında `user-prompt-inject-snapshot.sh` CONTEXT başlığı + son
  journal satırlarını + snapshot'ı bağlama verir.
- **memory/*.md** — kalıcı tekil gerçekler ve doktrinler (MEMORY.md indeks).
