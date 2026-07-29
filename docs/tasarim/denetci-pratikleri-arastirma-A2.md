# Denetçi Pratikleri Araştırması — A2: AJE/RJE Kayıt Ekranı ve Kontrol Mekanizmaları

**Tarih:** 2026-07-17 · **Amaç:** UFRS worksheet kayıt sisteminin (otomatik doldurma, yönlendirme, güvenlik/doğrulama) sahadan geçerli fikirlerle iyileştirilmesi.
**KGS formülü (proje doktrini):** Kaynak Otoritesi × Veri Yeterliliği × Zaman Tazeliği (0–1).

---

## Kayıt Ekranı UX Desenleri

### 1. CaseWare Working Papers — Adjusting Entry Worksheet (sektör altın standardı)
**Kaynak:** [Caseware Docs — Adjusting Entry worksheet](https://www.caseware.com/docs/en/desktop/working-papers/engagements/journal-entries/adjusting-entry-worksheet), [Adjusting Journal Entries Interface](https://documentation.caseware.com/2018/WorkingPapers/en/Content/Accounting_and_Assurance/Adjusting_Journal_Entries/r_Adjusting_Journal_Entries_Interface.htm) · **KGS: 0.85** (resmi vendor dokümantasyonu × tam ekran detayı × güncel)

Ekran anatomisi:
- **Meta alanlar:** Kayıt no (dropdown ile gezinilebilir), **Created by** (oluşturan kullanıcı otomatik damgalanır), Period Type (raporlama dönemlerinden seçim), Date (aktif dönemin kapanış tarihi default), Description (satır başına 40 karakter, çok satırlı).
- **6 kayıt tipi, her birinin posting davranışı farklı:**
  1. **Normal adjusting (AJE)** — tüm otomatik dokümanları günceller; dönem geçişinde adjustment sütunundan açılış bakiyesine taşınır.
  2. **Reclassifying (RJE)** — mali tablolara ve leadsheet'lere yansır ama **yasal muhasebe kayıtlarına yansımaz** (bizim VUK-dokunulmaz ilkemizin birebir karşılığı).
  3. **Eliminating** — konsolidasyon eliminasyonları.
  4. **Tax** — vergi bazı düzeltmeleri (ayrı sütun/baz).
  5. **Other basis** — alternatif çerçeve (ikinci GAAP; bizde VUK↔TFRS dual-defter'e denk).
  6. **Prior period** — sadece kümülatif (YTD) tabloları etkiler.
- **+3 "Unrecorded" (kayda alınmamış) varyant: factual / projected / judgmental** — düzeltilmeyen yanlışlıklar (SUD/passed adjustments) ayrı havuzda izlenir, mizana işlenmez ama önemlilik özetine akar.
- **Recurrence:** Recurring (tanımlı aralıkta tekrar, kümülatif sütun takibi) ve **Reversing** (önceki dönem kaydını otomatik ters çevirme). Reversing yalnızca Normal/Reclassifying/Eliminating/Tax/Other basis tiplerinde açılır.
- **Hesap seçimi çok-defterli:** aynı satır Financial / Cash Flow / Leadsheet / Group mapping / Tax code eksenlerine aynı anda maplenebilir.
- **Satır sütunları:** No, Hesap adı, Tutar (opsiyonel Borç/Alacak ayrık), Vergi kodu, Kümülatif, **Annotation** (satır bazında tickmark + not = çalışma kağıdı referansı).
- **Doğrulama:** "Booked in General Ledger" checkbox'ı kaydı ara bakiyelerle sınırlar (açılış bakiyelerini bozamaz); "Calculated" opsiyonu formül tabanlı kayıt + otomatik yeniden hesaplama sağlar.

**Bize uygulanabilirlik:** Tip ayrımı (AJE/RJE/Unrecorded) + reversing + satır bazlı annotation, WS-12 çatı desenine doğrudan taşınabilir. Özellikle RJE'nin "deftere işlenmez, rapora yansır" davranışı Beyanname Doktrini ile aynı felsefe.

### 2. Türkiye pratiği: Mizan-sütunlu dönüşüm çalışma kağıdı (yaygın fiili standart)
**Kaynak:** [CONSIFRS UFRS Uygulaması](https://consifrs.com/cozum-detay/ufrs-uygulamasi) (TR vendor), [Vergi Algı — VUK→UFRS/BOBİ FRS geçişte düzeltme ve sınıflandırma kayıtları](https://vergialgi.com/bagimsiz-denetimde-vuk-finansal-tablolardan-ufrs-bobi-frs-tablolara-geciste-verilen-duzeltme-ve-sin), [DergiPark — Vergi Mizanından BOBİ FRS'ye Dönüşüm Kayıtları](https://dergipark.org.tr/en/download/article-file/928117) · **KGS: 0.65** (vendor tanıtımı + hakemli makale karışımı; ekran detayı orta)

- TR'de fiili desen: **Yasal Mizan → şirket uyarlama/düzeltme sütunları → boş "Audit Düzeltme Kayıtları (AAJE)" sütunu → denetlenmiş UFRS mizanı**. Denetçi farkları ayrı sütunda tutulur; şirket dönüşümü ile denetçi düzeltmesi karışmaz.
- AAJE'ler **Excel'den map edilerek** içeri alınır (denetçinin Excel'de çalışıp sisteme yükleme alışkanlığı güçlü — import yolu şart, elle giriş tek yol olmamalı).
- CONSIFRS yaklaşımı: "**Ayrı bir defter tutulmaz**" — dönüşüm periyodik yapılır, önceki dönem reversal gerektirmez; sınıflandırma/netleştirme/düzeltme kayıtları sistem tarafından üretilir, denetçi bulguları üstüne biner.
- Akademik literatür (DergiPark/ResearchGate) üç dönüşüm yöntemi sayar: **tablo düzeyinde / mizan düzeyinde / ikili muhasebe**; mizan düzeyi en yaygını (bizim seçimimizle uyumlu).
- Kritik saha notu (Vergi Algı ekosistemi): "Tüm fark kayıtlarının **neden** hazırlandığı, ilgili hesaplamalarla beraber dokümante edilmeli ve **sonraki dönemlerde kullanılmak üzere** saklanmalı — denetim dosyasındaki en önemli çalışma kağıdı." → Her kayda zorunlu gerekçe + hesaplama eki + gelecek döneme devir mekanizması.

### 3. Caseware Turkey / Financials — TFRS raporlama akışı
**Kaynak:** [Caseware Turkey — Finansal Raporlama](https://www.caseware.com.tr/finansal.php) · **KGS: 0.7** (yerel vendor, TR pazarına özgü)

- Mizan/düzeltme/kebir importu: Excel, CSV, XBRL, XML, SAF-T.
- Düzeltme girildiğinde **ilgili tüm göstergeler ve dipnotlar otomatik güncellenir** — tek kayıt → çok doküman senkronu; tie-out elle yapılmaz.
- Çıktı: KGK taksonomisine uygun XBRL doğrulaması dahil.

### 4. Fieldguide — ATB (adjusted trial balance) merkezli akış
**Kaynak:** [Fieldguide — Adjusted trial balance](https://www.fieldguide.io/resource-articles/adjusted-trial-balance) · **KGS: 0.7** (modern vendor, kavramsal derinlik iyi, pazarlama içeriği)

- ATB "engagement'ın merkez hub'ı": tüm çalışma kağıtları, yanlışlık özetleri ve mali tablolar **tek kaynaktan** türetilir.
- Kayıt kategorileri net ayrılır: **müşteri kaynaklı** (dönem sonu tahakkuk/amortisman) vs **denetçi önerili** (yönetimin kabul ettiği AJE) vs **reclass** (net kârı etkilemeyen sunum düzeltmesi).
- Excel acısı: "mizan, düzeltmeler ve çalışma kağıtları ayrı dosyalarda yaşar; tie-out her review'da yeniden inşa edilir" — geç düzeltme = çoklu doküman güncelleme + yeniden bağlama.
- Doğrulama çerçevesi: JE popülasyonu bütünlük testi (önceki yıl TB + cari hareket = bakiye yeniden hesabı), ATB düzeyinde borç=alacak, çalışma kağıdı↔mizan çapraz mutabakat.

---

## Güvenlik/Kontrol Mekanizmaları Kataloğu

| Mekanizma | Açıklama | Kaynak | KGS | Bize uygulanabilirlik |
|---|---|---|---|---|
| **Denge zorunluluğu (satır ve fiş düzeyi)** | Fiş kaydedilmeden borç=alacak; ATB düzeyinde de ikinci kontrol | [Fieldguide](https://www.fieldguide.io/resource-articles/adjusted-trial-balance), Caseware docs | 0.85 | ZORUNLU, v1. Kayıt butonu dengesizken pasif; fark rozeti canlı gösterilir |
| **Kayıt tipi → posting davranışı kısıtı** | RJE deftere işlenmez; Unrecorded mizanı hiç etkilemez; Prior period sadece YTD | [Caseware AJE worksheet](https://www.caseware.com/docs/en/desktop/working-papers/engagements/journal-entries/adjusting-entry-worksheet) | 0.85 | ZORUNLU. Tip enum'u motorda posting kuralını belirlesin, UI'da tip değişince davranış önizlemesi |
| **Dosya kilidi / 60 gün arşiv (BDS 230)** | Rapor tarihinden itibaren 60 günde nihai dosya; sonrası silme/değiştirme yasak | [KGK BDS 230](https://kgk.gov.tr/Portalv2Uploads/files/Duyurular/v2/BDS/BDS%20230-Site.pdf), [IFAC ISA 230](https://www.ifac.org/system/files/publications/files/A012%202012%20IAASB%20Handbook%20ISA%20230.pdf) | 0.9 | ZORUNLU (mevzuat). Engagement-düzeyi kilit tarihi; kilit sonrası yalnız "ekleme + gerekçe" (BDS 230 A24 deseni), düzenleme yok |
| **Değişmezlik + storno (append-only)** | Kayıt sonrası edit yok; düzeltme = ters kayıt + yeni kayıt; append-only log + hash zinciri | [Stampli](https://www.stampli.com/resources/immutable-audit-trail/), [ChequeDB](https://chequedb.com/resources/blog/immutable-audit-trails-101-what-financial-compliance-actually-requires), [HubiFi](https://www.hubifi.com/blog/immutable-audit-log-basics) | 0.7 | YÜKSEK. ULID + append-only event log zaten mimaride var; SHA-256 zincirleme düşük maliyetli ek |
| **Maker-checker (dört göz)** | Hazırlayan ≠ onaylayan; onay kaydı kimlik+timestamp+onaylanan durumun snapshot'ı ile bağlanır | [Velt compliance guide](https://velt.dev/blog/financial-audit-trail-compliance-guide), BDS 220/KYS 1 ([KGK KYS 1 rehberi](https://kgk.gov.tr/Portalv2Uploads/files/Duyurular/v2/TDS/TDS_2022_Seti/KYS%201%20%C4%B0LK%20UYGULAMA%20REHBER%C4%B0.pdf)) | 0.75 | YÜKSEK ama tek-kişilik SMMM ofisinde **opsiyonel mod** olmalı (self-review uyarısıyla); çok kullanıcıda zorunlu |
| **Oluşturan damgası (attribution)** | Her fişte "Created by" otomatik, değiştirilemez | Caseware AJE worksheet | 0.85 | ZORUNLU, ucuz |
| **Hesap-sınıf kısıtları** | Belirli hesaplara (OCI, birikmiş kâr, sistem-üretimi ertelenmiş vergi hesapları) elle kayıt engeli/uyarısı | Caseware "Booked in GL" checkbox deseni + genel muhasebe yazılımı pratiği | 0.6 | YÜKSEK. Bizde ek olarak: 570/580, OCI karşılığı hesaplar, motorun sahiplendiği ertelenmiş vergi hesapları elle girişte "motor sahipliği" uyarısı versin |
| **Dönem kilidi (kayıt tarihi kısıtı)** | Kayıt tarihi açık döneme sınırlı; kapalı döneme kayıt = yalnız Prior-period tipi | Caseware period type/date alanları | 0.8 | ZORUNLU. Aktif+arşiv swap desenimizle uyumlu |
| **Düzeltilmeyen yanlışlıklar havuzu (SUD)** | Unrecorded factual/projected/judgmental kayıtları mizana işlemeden biriktir; önemlilikle karşılaştır (BDS 450) | Caseware unrecorded types; Fieldguide "passed adjustments" | 0.8 | YÜKSEK. WS-12'ye "öneri → kabul/red" durumu; reddedilen AJE otomatik SUD'a düşer |
| **JE risk bayrakları** | Yuvarlak tutar, dönem sonu az-belgeli kayıt, top-side, yetkisiz kullanıcı → otomatik işaret | [Yellowbook-CPE](https://yellowbook-cpe.com/testing-journal-entries-in-audits.html), [JofA — JE testing w/ Excel](https://www.journalofaccountancy.com/issues/2021/nov/journal-entry-testing-excel/), [MindBridge](https://www.mindbridge.ai/blog/better-approach-journal-entry-testing/), Fieldguide | 0.7 | ORTA vade. Denetçinin kendi girdiği fişlere de aynı testler uygulanabilir (öz-kalite kontrolü) |
| **Zorunlu gerekçe + hesaplama eki** | Her fark kaydının nedeni + hesaplama dokümante, gelecek döneme saklanır | TR pratiği (Vergi Algı ekosistemi) + BDS 230 "deneyimli denetçi testi" ([ciferi](https://ciferi.com/blog/isa-230-audit-documentation-guide)) | 0.75 | ZORUNLU. Gerekçe alanı boşsa kayıt atılamaz; hesaplama ekine ULID referansı — KGS/kanıt doktriniyle birleşir |
| **Formül-tabanlı (calculated) kayıt** | Tutar elle değil formülle; kaynak değişince otomatik yeniden hesap | Caseware "Calculated" opsiyonu | 0.8 | YÜKSEK. Bizim öz motor zaten bunu hedefliyor; elle override → "manuel müdahale katmanı, izli" (Beyanname Doktrini) |
| **Tek kaynak → çok doküman senkronu** | Düzeltme girilince leadsheet/dipnot/tablo otomatik güncellenir | [Caseware Turkey](https://www.caseware.com.tr/finansal.php), Fieldguide | 0.75 | Mimari ilke; tie-out'u insan değil sistem taşır |

---

## Denetçi Sesleri (düşük KGS, fikir değeri yüksek)

Doğrudan Reddit/forum thread'i yakalanamadı (aramalar sonuçsuz); aşağıdakiler ikincil kaynaklardan damıtılmış saha sinyalleri:

- **"Excel'den kopamama" gerçeği** (KGS ~0.4): TR vendor'ları bile AAJE'yi "Excel'den map ederek" alıyor; eğitim pazarı "Excel'de VUK→TFRS dönüşümü" kursları satıyor ([FİNTEMO](https://fintemo.com/?page_id=110), [LinkedIn/Meltem Aköz](https://tr.linkedin.com/posts/meltemakoz_excelde-tfrs-tms-finansal-tablo-donusumu-activity-7069530438834208768-2GN5)). Ders: kayıt ekranı ne kadar iyi olursa olsun **iki yönlü Excel köprüsü** (şablonlu import + tam export) olmadan denetçi benimsemez.
- **"Yazılım önerir, denetçi karar verir" sınırı** (KGS ~0.4): Fieldguide'ın "auditor-proposed → management accepts" akışı ile CONSIFRS'in "sistem üretir, denetçi bulgusu üstüne biner" modeli aynı noktada buluşuyor: otomatik üretilen dönüşüm kaydı **taslak/öneri** statüsünde doğmalı; denetçi kabul/düzenle/reddet der; kabul edilmeden mizana işlenmemeli. Otomatik doldurma "sessizce postlama" olarak algılanırsa güven kaybı.
- **Geç düzeltme kaskadı en büyük şikâyet** (KGS ~0.35): "Her geç AJE = Word taslağı + Excel destek dosyası + tie-out'u yeniden kontrol" (Fieldguide'ın sahadan aktarımı). En çok zaman kaybettiren an, kayıt atmak değil **kaydın türevlerini güncellemek**.
- **Top-side/dönem-sonu kayıtlara refleks şüphe** (KGS ~0.4): Denetçi kültüründe normal döngü dışı kayıt = ekstra bakış (PCAOB/Yellowbook geleneği). Kendi ürettiğimiz dönüşüm fişleri de "top-side" sınıfındadır; her birinin kaynağını (hangi standart, hangi WS, hangi veri) tek tıkla açıklayabilmek güven inşa eder.
- **Akademik saha araştırması** ([ResearchGate — mizan düzeyinde dönüşüm sorunları](https://www.researchgate.net/publication/369762929)) (KGS ~0.5): mizan düzeyinde dönüşümde en sık sorunlar: hesap eşleme belirsizlikleri, paralel kayıt yükü, VUK-TFRS farklarının dokümantasyon eksikliği — otomatik hesap eşleme önerisi + fark gerekçe zorunluluğu tam bu yaraya denk geliyor.

---

## Öneri Sıralaması (KGS-ağırlıklı yol haritası)

1. **Denge zorunluluğu + kayıt tipi enum'u (AJE/RJE/Unrecorded/PriorPeriod) + tip-bazlı posting kuralı** — KGS 0.85, mevcut WS-12 çatısına en düşük maliyetle oturur; RJE'nin "deftere işlenmez" davranışı dual-defter stratejimizin motor karşılığı.
2. **BDS 230 uyumu: dönem/dosya kilidi + append-only storno + created-by damgası** — KGS 0.9 (mevzuat) + 0.7 (teknik desen); kilit sonrası yalnız gerekçeli ekleme. Mimarideki ULID/event-log ile hash zinciri ucuz.
3. **Zorunlu gerekçe + hesaplama referansı (kanıt bağı)** — KGS 0.75; her fiş → standart maddesi + WS + kaynak veri ULID'i. KGS/kanıt doktrini ile doğal birleşim; "en önemli çalışma kağıdı" saha tespitinin ürünleşmesi.
4. **Öneri-onay akışı (otomatik fiş taslak doğar, denetçi kabul eder) + SUD havuzu** — KGS 0.75; "yazılım önerir, denetçi karar verir" sınırına saygı; reddedilenler düzeltilmemiş yanlışlıklar özetine düşer (BDS 450 çıktısı bedavaya gelir).
5. **Reversing/recurring + calculated (formül-tabanlı) kayıt + tek kaynak-çok doküman senkronu** — KGS 0.8; geç düzeltme kaskadını öldüren özellik seti; öz motorun "elle override izli" ilkesiyle uyumlu.
6. *(Orta vade)* Maker-checker opsiyonel modu, hesap-sınıf kısıtları (OCI/570-580/motor-sahipli hesaplar), JE risk bayrakları, iki yönlü Excel köprüsü.

---

### Kaynak listesi (tam)
- Caseware AJE worksheet: https://www.caseware.com/docs/en/desktop/working-papers/engagements/journal-entries/adjusting-entry-worksheet
- Caseware AJE interface (2018): https://documentation.caseware.com/2018/WorkingPapers/en/Content/Accounting_and_Assurance/Adjusting_Journal_Entries/r_Adjusting_Journal_Entries_Interface.htm
- Caseware Turkey Finansal Raporlama: https://www.caseware.com.tr/finansal.php
- CONSIFRS UFRS Uygulaması: https://consifrs.com/cozum-detay/ufrs-uygulamasi
- Vergi Algı — VUK→UFRS/BOBİ FRS düzeltme/sınıflandırma: https://vergialgi.com/bagimsiz-denetimde-vuk-finansal-tablolardan-ufrs-bobi-frs-tablolara-geciste-verilen-duzeltme-ve-sin
- DergiPark — Vergi mizanından BOBİ FRS'ye dönüşüm: https://dergipark.org.tr/en/download/article-file/928117
- ResearchGate — Mizan düzeyinde dönüşüm sorunları: https://www.researchgate.net/publication/369762929
- KGK BDS 230: https://kgk.gov.tr/Portalv2Uploads/files/Duyurular/v2/BDS/BDS%20230-Site.pdf
- KGK KYS 1 İlk Uygulama Rehberi: https://kgk.gov.tr/Portalv2Uploads/files/Duyurular/v2/TDS/TDS_2022_Seti/KYS%201%20%C4%B0LK%20UYGULAMA%20REHBER%C4%B0.pdf
- IFAC ISA 230: https://www.ifac.org/system/files/publications/files/A012%202012%20IAASB%20Handbook%20ISA%20230.pdf
- ciferi — ISA 230 rehberi: https://ciferi.com/blog/isa-230-audit-documentation-guide
- Fieldguide — Adjusted trial balance: https://www.fieldguide.io/resource-articles/adjusted-trial-balance
- Stampli — Immutable audit trail: https://www.stampli.com/resources/immutable-audit-trail/
- ChequeDB — Immutable audit trails 101: https://chequedb.com/resources/blog/immutable-audit-trails-101-what-financial-compliance-actually-requires
- HubiFi — Immutable audit log basics: https://www.hubifi.com/blog/immutable-audit-log-basics
- Velt — Financial audit trail compliance: https://velt.dev/blog/financial-audit-trail-compliance-guide
- Yellowbook-CPE — Testing journal entries: https://yellowbook-cpe.com/testing-journal-entries-in-audits.html
- Journal of Accountancy — JE testing w/ Excel: https://www.journalofaccountancy.com/issues/2021/nov/journal-entry-testing-excel/
- MindBridge — JE testing: https://www.mindbridge.ai/blog/better-approach-journal-entry-testing/
- muhasebetr — TMS 12 ertelenmiş vergi: https://www.muhasebetr.com/yazarlarimiz/mahmutsemihsekercioglu/004/
- FİNTEMO — Excel'de VUK-TFRS dönüşüm eğitimi: https://fintemo.com/?page_id=110
- KGK FRS Uyumlu Hesap Planı Taslağı: https://www.kgk.gov.tr/Portalv2Uploads/files/Duyurular/v2/TFRS/EK2_Finansal%20Raporlama%20Standartlar%C4%B1na%20Uygun%20Hesap%20Plan%C4%B1%20Tasla%C4%9F%C4%B1.pdf
