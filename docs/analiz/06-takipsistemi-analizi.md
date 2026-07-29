# takipsistemiv2 Analizi — Audit-Liners'a Taşınacak Fikirler (2026-07-06)

> Kaynak: github.com/arda1996/takipsistemiv2 (Python FastAPI + React; banka ekstresi takip/mutabakat sistemi).
> Değerlendirme: kod bire bir alınmaz (buglar + Python), **fikir ve alan bilgisi** alınır — kullanıcının talebi de bu.

## Projenin özü
PDF banka ekstresi → parse (15+ TR bankası) → doğrulama (bakiye zinciri, dedup) → kişi/IBAN eşleme →
dış kayıtla katmanlı mutabakat (SuperMatch) → sistem sağlık denetimi → Telegram bildirimi.

## 1. En değerli varlık: banka parser ALAN BİLGİSİ
- 9+ banka parser'ı (ziraat, vakifbank, halkbank, ing, kuveyt_turk, emlak/vakıf katılım, alternatif, tombank…)
  + generic + excel; `BANK_SIGNATURES` ile PDF'ten otomatik banka tanıma.
- `pdfparseyöntemi.md` = süreç fener dokümanı (bizim learnings kalıbının aynısı): bbox-bazlı tablo okuma
  (extract_tables boşluk yutar!), multi-line transaction, DESC→kronolojik çevirme, TR tutar formatı,
  sayfa sonu bölünmesi, İ/I IGNORECASE tuzağı, 3 katmanlı dedup, reversal (TERS/İADE) işaretleme.
- Veri modeli: ParsedStatement{iban, kişi, dönem, opening/closing, quality_score, parse_method(llm fallback!)}
  + ParsedTransaction{tarih, açıklama, tutar, balance_after, tür, receipt_no, sender, raw_counterparty, is_reversal}.

**Bize aktarım (D.5):** Python'ı porta çevirmek DEĞİL — 15 parser dosyasının alan bilgisini **banka profili
JSON'una** damıtmak (motor kod + kural veri ilkemiz): her banka = {imza metinleri, tarih/tutar formatı,
kolon düzeni/bbox ipuçları, sıralama yönü, receipt kuralı}. Tek Rust motoru + profiller. PDF metin çıkarımı:
pdf-extract/lopdf crate; çözümsüz formatlarda "llm/hybrid fallback" fikri korunur.

## 2. Bakiye zinciri doğrulaması → yeni denetim motoru M13
Her işlemde balance_after zinciri opening→closing doğrulanıyor; ekstreler ARASI devamlılık + çakışan dönem
(overlap-aware) kontrolü; HealthCheck sayfası "zincir kopukluğu / bakiye tutarsızlığı / boş ekstre" raporluyor.
**Bize:** M13 "banka bakiye zinciri" motoru — ekstre içi zincir + ekstreler arası devir + defter 102.xx
bakiyesiyle mutabakat. TIC/ETIC programlarına "banka mutabakatı" çalışması (BDS 505 banka teyidinin veri ayağı).

## 3. SuperMatch ilkeleri → eşleştirme motorumuzun v2'si
- Katmanlı: (isim+IBAN) grubu → grup içi KESİN tutar (kuruş toleranssız) → kalanlar KATEGORİZE:
  bizde-eksik / karşıda-eksik / grup-içi-tutar-farkı / isim-yok / IBAN-yok.
- Deterministik: sorted keys + id tie-break (bizim determinizm ilkemizle aynı ruh).
**Bize (D.6):** Mevcut banka önerisi (tek kural: tutar ±3 gün) bu mimariye yükselir; aynı motor cari
mutabakata (müşteri/satıcı ekstresi) da koşar; kategorize artıklar çalışma kağıdına BULGU olarak akar.

## 4. Türkçe isim kanonikleştirme → cari eşleştirme
canonical_name: TR fold (İ→I, Ş→S…), unvan temizliği (DR/SAYIN…), banka payment-suffix strip
("- FAST Anlık Ödeme"), name_match_score (Jaccard benzeri).
**Bize (D.7):** ekstre açıklamasından karşı taraf adı çıkar → cari kart/muavin önerisi (120/320.xx) →
"eşleşmeyen ekstreden taslak fiş" işinin isim ayağı. **Optimize yol:** 2.460 satırlık isim listesi
(turkish_names.py) bize GEREKSİZ — biz ad/soyad ayrıştırmıyoruz, cari adı bütün eşleşir; kanonikleştirme
+ skor yeter (~100 satır Rust).

## 5. Diğer taşınabilir fikirler
- **3 katmanlı dedup** (tx_hash: IBAN+tarih+tür+tutar+bakiye · receipt_no · content-key fallback) →
  ekstre yeniden yükleme mükerrer koruması (bizim mükerrer belge reddinin banka ayağı).
- **is_reversal işareti** → dünkü iade-körlüğü dersinin banka tarafı: TERS/İADE hareketleri işaretle,
  motorlar ayrı ele alsın.
- **quality_score + Sistem Denetimi sayfası** → denetim programına sektör-üstü "veri sağlığı" programı
  (zincir, boş dönem, negatif kasa günleri) — H'de kısmen var, ekstre boyutu eklenir.
- **Telegram bildirimi** → vergi takvimi son gün uyarıları (uzak faz, F).
- **Excel parser** → toplu fiş içe aktarma (B'deki iş) için hazır desen.

## 6. Optimize/kolay yol karşılaştırması
| takipsistemiv2 yolu | Bizim optimize yolumuz |
|---------------------|------------------------|
| Banka başına ayrı .py parser (15 dosya, kod tekrarı) | Tek motor + banka profili JSON (veri-güdümlü; yeni banka = veri girişi) |
| Python + Docker + VPS + SQLite WAL sorunları | Rust tek binary (Tauri sidecar'sız), bellekte→gömülü PG |
| turkish_names.py 2.460 satır sözlük | Kanonikleştirme + benzerlik skoru (~100 satır); sözlüksüz |
| mark_reversals çift çağrı bug'ı (dokümanda itiraf) | Reversal işareti tek yerde: parse sonrası motorda |
| Vevo'ya özel eşleştirme | Genel mutabakat motoru: banka↔defter, cari↔cari, Vevo-benzeri her dış kaynak |

## 7. Entegrasyon sırası (IS-LISTESI D'ye işlendi)
1. D.5 Banka profili JSON + PDF içe aktarma motoru (parser alan bilgisinin damıtılması)
2. D.6 Eşleştirme v2 (katmanlı + kategorize + deterministik) → kağıda bulgu akışı
3. D.7 Ekstre dedup + M13 bakiye zinciri + is_reversal + isim kanonikleştirme→cari önerisi
