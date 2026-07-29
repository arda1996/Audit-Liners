# Banka Veri Hattı — Backend Kurgusu (offline-first)

> Amaç: banka hesap verisini (OAuth, read-only) → muhasebe kaydı dayanağına + kredi/faiz hesabına
> + vergi ve bağımsız denetim çıktısına dönüştüren tam backend. **Test hesabı yokken kurulur**:
> her katman ÖHVPS standardı + mock veri ile offline test edilir; İş B. ticari test hesabı gelince
> yalnız kimlik/clientId takılır. İlke: para hareketi YOK, salt-okunur; deftere kesin yazma YOK (taslak).
> Referans: docs/analiz/07-acik-bankacilik-isbank.md · data/islem-tipleri.json

## 0. Katman haritası (hexagonal — domain saf kalır)
```
[OAuth/Rıza katmanı]  →  [Ingest adaptörü]  →  [Normalize: EkstreHareket]
     (crates/api)          (crates/ingest)          (crates/domain)
                                                          │
        ┌─────────────────────────────────────────────────┼───────────────────────────┐
        ▼                          ▼                        ▼                           ▼
 [Bütünlük doğrulama]      [Sınıflandırıcı]         [Kredi/Faiz motoru]        [Kanıt arşivi]
 zincir/devir/dedup       islem-tipleri.json        kredi.rs (YENİ)           {ham,hash,rıza}
        │                          │                        │                           │
        └──────────────► [Taslak fiş üretici] ◄─────────────┘                           │
                          (maker-checker)                                               │
                                   │                                                    │
                          [Üçlü mutabakat] ◄────────── dekont (kendi arayüz/VakıfBank) ─┘
                                   │
                   ┌───────────────┴───────────────┐
                   ▼                                ▼
          [Vergi ekseni: VUK]              [Bağımsız denetim: TFRS/BDS]
          BSMV/KKDF, dayanak zinciri       reeskont, etkin faiz, kanıt
```

## 1. OAuth / Rıza katmanı (crates/api)
- Akış: mükellef "bankanı bağla" → bankanın KENDİ ekranına redirect → GKD/2FA orada → `code` → token.
- Sakla: `{ mukellef_id, banka_kodu, refresh_token(şifreli), rıza_no, rıza_bitis, kapsam }`.
  **Şifre asla saklanmaz/görülmez.** refresh_token ile access mint → "sürekli açık oturum" YOK.
- Çoklu mükellef: her bağlantı `mukellef_id`'ye bağlı; aktif+arşiv swap deseni ([[sektor-ve-coklu-mukellef]]).
- Mock: sahte token veren yerel "banka" stub → tüm hat account'suz test edilir.

## 2. Ingest adaptörü (crates/ingest — YENİ crate)
- `trait VeriKaynagi { fn hesaplar(); fn islemler(hsp, baslangic, bitis) -> Vec<EkstreHareket>; }`
- İlk impl: `IsbankHbhs` (ÖHVPS `/hesaplar/{ref}/islemler`). Banka PDF (D.5) ile ortak arayüz.
- **Kota disiplini:** sistem-çağrısı bireysel 4/gün, kurumsal 12/saat; snapshot + son çekim kaydı.
- **Ham yanıt + hash** saklanır (BDS 500 kanıt) → kanıt arşivi.
- Şema notu: İş B. `islemler` alanları henüz yakalanmadı (07-...md açık soru #3) → adaptör önce
  ÖHVPS v2.0.0 standardına yazılır (islem no, referans, tutar, bakiye, zaman, kanal, B/A, karşı IBAN/unvan).

## 3. Normalize + bütünlük (crates/domain — HAZIR, bağlanacak)
- `EkstreHareket` (ekstre.rs) ← adaptör çıktısı.
- `zincir_dogrula` (işlem kaçırma → bakiye zinciri kırılırsa yakalanır), `devir_dogrula` (dönem sınırı),
  `hareket_anahtari` (mükerrer → dedup). → **iki risk (kaçırma/mükerrer) burada matematiksel çözülür.**

## 4. Sınıflandırıcı + taslak fiş üretici (D.9→D.10)
- `data/islem-tipleri.json` → açıklama (tr_katla normalize) → tip → şablon + vergi kuralı + güven.
- `guven=yuksek` → otomatik taslak; `orta/dusuk` → SMMM onay kuyruğu (maker-checker).
- `hesap-kurallari.json` 102 `karsi` genişletme: 642/646/656/770/780/335/360/361 (bugün yok).
- Çıktı: **taslak** fiş (dayanak = ekstre satır ref + hash) → mevcut fiş motoruna.

## 5. Kredi/Faiz motoru (crates/domain/kredi.rs — YENİ, hesap gerektirmez)
> Hem vergi (VUK reeskont md.281/285, dönem sonu faiz tahakkuku) hem bağımsız denetim (TFRS 9 etkin
> faiz/itfa edilmiş maliyet) için gerekli. Saf domain, tamsayı kuruş.
- **Girdi:** anapara, faiz oranı, vade, ödeme sıklığı, başlangıç; yöntem: **eşit taksit (annüite)** / eşit anapara / spot.
- **Çıktı — ödeme planı (amortisman tablosu):** her taksit → {tarih, taksit, anapara, faiz, BSMV, kalan bakiye}.
  Bu tablo, ekstredeki "KREDİ TAKSİT" tek satırının **anapara/faiz ayrışmasını** çözer (D.11 "ayrıştırma bekliyor" bayrağını kaldırır).
- **Faiz tahakkuku:** gün-bazlı işlemiş faiz (dönem sonu 381/181 tahakkuk; VUK).
- **BSMV:** kredi faizinde %5 → 780; ticari kredide **KKDF %0** (tüketici %15).
- **Reeskont (VUK 281/285):** senetli alacak/borç ve vadeli için iç iskonto → 642/657 · 122/322.
- **Etkin faiz (TFRS 9):** işlem maliyetleri dahil EIR ile itfa edilmiş maliyet — denetim overlay (I.2 düzeltme katmanı).
- **Fişleştirme:** plan → taksit tarihinde otomatik taslak (300 anapara + 780 faiz+BSMV / 102).

## 6. Üçlü mutabakat + kanıt (D.12→D.13)
- `Dayanak` genişletme (fis.rs:74): `kaynak, alinma_tarihi, hash, ekstre_satir_ref, rıza_no`.
- Banka mahsup fişinde dayanak zorunlu (fis.rs:31 istisnası kapatılır).
- `mutabakat()` ×2 (dekont↔ekstre, ekstre↔fiş) + fark matrisi: kayıt dışı / dayanaksız / belge eksik.
- Dekont: VakıfBank getReceipt (kendi hesabı) VEYA **"dekont yerine geçmez" etiketli kendi arayüzümüz**.

## 7. İki eksen çıktısı
- **Vergi (VUK):** BSMV≠KDV bayrağı (191'de banka masrafı → bulgu), tamlık kanıtı (zincir), dayanak zinciri.
- **Bağımsız denetim (TFRS/BDS):** etkin faiz/reeskont overlay, API ekstresi = BDS 500 güçlü kanıt (KGS yüksek).

## 8. Fazlama (build order — hesap gerektirmeyenler önce)
| Faz | İş | Hesap gerekir mi |
|-----|----|------------------|
| P1 | **kredi.rs** (ödeme planı + faiz tahakkuku + BSMV/reeskont) + testler | Hayır ✅ |
| P2 | islem-tipleri.json olgunlaştır + hesap-kurallari 102 karsi genişlet + sınıflandırıcı (D.9/D.10) | Hayır ✅ |
| P3 | crates/ingest + VeriKaynagi trait + mock kaynak + zincir/dedup bağla (D.8/D.15) | Hayır (mock) ✅ |
| P4 | Dayanak genişletme + üçlü mutabakat (D.12/D.13) | Hayır ✅ |
| P5 | OAuth/rıza katmanı + IsbankHbhs impl | Test hesabı gelince |
| P6 | UI: banka bağla, taslak onay kuyruğu, mutabakat kağıdı, dekont arayüzü | Kısmen |

**Kritik bağımlılık:** kalıcılık (F.2 🔒) — kanıt arşivi/rıza kalıcı olmadan bellekte sınırlı; P5 öncesi düşünülmeli.
