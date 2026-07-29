# TMS 1 Çerçevesinde SINIFLANDIRMA (Reclassification / RJE) Çalışması — Denetim Pratiğindeki Gerçek İçerik

Bu belge, UFRS WorkSheet modülündeki "Sınıflandırma" çalışmasının mevcut tek kalemlik ("UV kredinin KV kısmı") içeriğinin neden yetersiz olduğunu ve denetim pratiğinde bu çalışmanın gerçekte neyi kapsadığını, kaynaklı olarak ortaya koyar.

---

## 1. Sınıflandırma NEYİ AMAÇLAR?

VUK'a dayalı Tek Düzen Hesap Planı (TDHP) mizanı, **vergi matrahı** mantığıyla tutulur; hesap numarası bir kez atandığında (ör. bir kredi "400 Banka Kredileri" olarak açıldığında) yıl içinde vade yaklaşsa bile genellikle aynı hesapta kalır. TFRS finansal durum tablosu ise **kullanıcıya karar-yararlı bilgi** verecek biçimde, likidite ve vade esasına göre kalemleri **cari (dönen/kısa vadeli)** ve **cari olmayan (duran/uzun vadeli)** olarak ayrı sunmayı zorunlu kılar (TMS 1, §60–76).

Kritik nokta: **Sınıflandırma tutarı değiştirmez, sunumu düzeltir.** Bir varlığın/borcun ölçüm değeri (kuruş bazında) aynı kalır; yalnızca finansal tablodaki **satırı** (gösterim yeri) değişir. Bu nedenle bu kayıtlar **kâr/zararı etkilemez** ve **tutar-nötrdür** (borç bacağı = alacak bacağı, net özkaynak etkisi sıfır).

TMS 1'in gerekçesi: "İşletme sermayesi olarak sürekli devir daim eden net varlıkların, uzun vadeli faaliyetlerde kullanılan varlıklardan ayrıştırılması" kullanıcıya likidite/işletme sermayesi analizinde fayda sağlar ([KGK TMS 1, §60–68](https://www.kgk.gov.tr/Portalv2Uploads/files/Duyurular/v2/TMS/TMS_1_Finansal%20Tablolar%C4%B1n%20Sunulu%C5%9Fu.pdf); [ACCAFIN — Cari/Cari Olmayan kavramları](https://www.accafin.com/muhasebe/ifrs-ias-ufrs-tms-usgaap/tms-1-finansal-tablolarin-sunulusu-nda-cari-ve-uzun-donem-kavramlari)).

**Sınıflandırma ≠ Düzeltme (AJE):** Ölçüm/tanıma düzelten AJE'ler (reeskont, değer düşüklüğü, kıdem karşılığı hesaplaması, ertelenmiş verginin *tutarının* hesabı) kâr/zararı veya net varlığı değiştirir. Sınıflandırma (RJE) yalnızca **zaten doğru ölçülmüş** bir tutarı doğru satıra taşır. Denetçi bu ikisini ayrı çalışma kağıtlarında tutmalıdır.

---

## 2. TİPİK SINIFLANDIRMA KALEMLERİ (TDHP → TFRS)

Aşağıdaki her kalem bağımsız bir RJE satırıdır. "Borç/Alacak" bacakları **sunum satırları** arasında yapılır (dual-defter/çalışma kağıdı katmanında; TDHP defterine işlenmez). Tutarlar kuruş (i64).

### K-1. Uzun vadeli borcun kısa vadeli kısmı (vade < 12 ay)
- **TDHP kaynağı:** 400 Banka Kredileri, 405 Çıkarılmış Tahviller, 407/408, 421 Borç Senetleri (UV)
- **TFRS hedefi:** "Uzun vadeli borçlanmaların kısa vadeli kısımları" (kısa vadeli yükümlülük). TDHP karşılığı **303 Uzun Vadeli Kredilerin Anapara Taksitleri ve Faizleri** hesabıdır — TDHP bunu zaten öngörür ama uygulamada çoğu firma yıl içi transferi ihmal eder.
- **Tetikleyici veri:** Kredi geri ödeme planı / itfa tablosu; bilanço tarihinden itibaren 12 ay içinde vadesi gelen anapara + tahakkuk faizi.
- **RJE:** Borç 400 (UV borçlanmalar) / Alacak 303 (UV borçlanmaların KV kısmı). Kâr/zarar etkisi: **yok.**
- **Kaynak:** ([JAFAS — VUK'tan UFRS'ye dönüşüm](https://www.kgk.gov.tr/Portalv2Uploads/files/Duyurular/v2/TMS/TMS_1_Finansal%20Tablolar%C4%B1n%20Sunulu%C5%9Fu.pdf); TMS 1 §69–72 vade esası)

### K-2. İlişkili taraf alacak/borçlarının ayrıştırılması (TMS 24 kesişimi)
- **TDHP kaynağı:** 131/231 Ortaklardan Alacaklar, 132/232 İştiraklerden, 133/233 Bağlı Ortaklıklardan Alacaklar; 331 Ortaklara Borçlar, 332/333 İştirak/Bağlı Ortaklık; ayrıca **120 Alıcılar / 320 Satıcılar içindeki ilişkili taraf bakiyeleri**.
- **TFRS hedefi:** "İlişkili taraflardan ticari alacaklar" ve "İlişkili taraflardan ticari olmayan alacaklar" (ve borç tarafı) ayrı satırlarda. TFRS, ilişkili taraf bakiyelerini üçüncü taraflardan **ayrı gösterir** (TMS 24 dipnot + TMS 1 sunum).
- **Tetikleyici veri:** İlişkili taraf listesi + cari hesap analizi; 120/320 içindeki ilişkili taraf mükelleflerinin bakiye dökümü. Ayrıca **ticari (mal/hizmet)** mi **ticari olmayan (finansman/temettü)** mi ayrımı.
- **RJE (örnek):** Borç "İlişkili taraflardan ticari alacaklar" / Alacak 120 (üçüncü taraf ticari alacaklar) — ilişkili kısım kadar.
- **Kaynak:** ([bilgi.edu.tr — TMS 1 alt sınıflar: müşterilerden, ilişkili taraflardan, peşin ödemeler](https://openaccess.bilgi.edu.tr/server/api/core/bitstreams/734cc404-765c-497f-8f1b-81f7835095df/content))

### K-3. Ticari olmayan alacak/borçların ticariden ayrılması
- **TDHP kaynağı:** 136 Diğer Çeşitli Alacaklar, 236 (UV); 126/226 Verilen Depozito ve Teminatlar; personel avansları, vergi iadesi alacağı.
- **TFRS hedefi:** "Ticari alacaklar" yalnızca mal/hizmet satışından doğanları içerir; kalanlar "Diğer alacaklar". Depozito/teminatlar genelde "Diğer alacaklar" (uzun vadeli ise duran).
- **Tetikleyici veri:** Alacağın doğuş niteliği (fatura/satış mı, personel/idari mi).
- **RJE:** Borç "Diğer alacaklar" / Alacak 120 (yanlışlıkla ticaride toplanmış tutar kadar).

### K-4. Verilen/alınan sipariş avanslarının ayrıştırılması
- **TDHP kaynağı:** 159 Verilen Sipariş Avansları, 259 (UV verilen avanslar); 340 Alınan Sipariş Avansları, 440 (UV).
- **TFRS hedefi:** **Verilen avanslar** → stok/hizmet peşinatı ise "Peşin ödenmiş giderler" veya niteliğine göre "Diğer dönen varlıklar" (nakit çıkışına dönmeyeceği için **ticari alacak değildir**). **Alınan avanslar** → mal/hizmet borcu olduğundan (TFRS 15 sonrası) "Ertelenmiş gelirler / Sözleşme yükümlülükleri", ticari borç değil.
- **Tetikleyici veri:** Avansın hangi sipariş/sözleşmeye ait olduğu, teslim vadesi (12 ay içi/dışı).
- **RJE:** Borç "Peşin ödenmiş giderler" / Alacak 159; ve Borç 340 / Alacak "Ertelenmiş gelirler".

### K-5. Peşin ödenen giderler / gelecek dönem gelir-giderleri
- **TDHP kaynağı:** 180 Gelecek Aylara Ait Giderler, 280 Gelecek Yıllara Ait Giderler; 380 Gelecek Aylara Ait Gelirler, 480 Gelecek Yıllara Ait Gelirler; 181/281 Gelir Tahakkukları.
- **TFRS hedefi:** 180 → "Peşin ödenmiş giderler" (dönen), 280 → "Peşin ödenmiş giderler" (duran); 380/480 → "Ertelenmiş gelirler" (kısa/uzun vadeli). Vade 12 ayı aşan kısım **duran/uzun vadeliye** taşınır.
- **Tetikleyici veri:** Giderin/gelirin ilişkin olduğu dönem takvimi (ör. 3 yıllık peşin kira → 12 ay dönen, kalan duran).
- **RJE:** 180 içinden 12 ayı aşan kısım Borç "Peşin ödenmiş giderler (uzun vadeli)" / Alacak "Peşin ödenmiş giderler (kısa vadeli)".

### K-6. Nakit ve nakit benzerleri tanımı (TMS 7)
- **TDHP kaynağı:** 100 Kasa, 101 Alınan Çekler, 102 Bankalar, 103 Verilen Çekler (-), 108 Diğer Hazır Değerler, 110/111/112 Menkul kıymetler.
- **TFRS hedefi:** "Nakit ve nakit benzerleri" = kasa + **vadesiz mevduat** + edinim tarihinden itibaren **≤ 3 ay** vadeli, değer riski önemsiz likit yatırımlar. **Vadesi 3 aydan uzun mevduatlar nakit benzeri DEĞİLDİR** → "Finansal yatırımlar"a taşınır. **Bloke/teminat mevduat** serbest kullanılamadığından nakitten çıkarılıp "Diğer alacaklar/finansal varlıklar"a alınır. İleri vadeli **alınan çekler (101)** genelde nakit değil, ticari alacaktır.
- **Tetikleyici veri:** Mevduat vade dökümü; blokaj/teminat şartı; çeklerin vade tarihi.
- **RJE:** Borç "Finansal yatırımlar" / Alacak 102 (3 aydan uzun vadeli mevduat kadar); Borç "Ticari alacaklar" / Alacak 101 (vadeli çekler).
- **Kaynak:** ([KGK TMS 7 §6–8](https://www.kgk.gov.tr/Portalv2Uploads/files/DynamicContentFiles/T%C3%BCrkiye%20Muhasebe%20Standartlar%C4%B1/TMSTFRS2018Seti/TMS/TMS%207%20Nakit%20Ak%C4%B1%C5%9F%20Tablosu%20Kurul%20Karar%C4%B1(2).pdf); [ACCAFIN — bloke mevduatların sunumu](https://www.accafin.com/muhasebe/ifrs-ias-ufrs-tms-usgaap/tms-7-nakit-akis-tablosu-kapsaminda-bloke-mevduat))

### K-7. Şüpheli alacak karşılığının netleştirme/sunumu
- **TDHP kaynağı:** 128 Şüpheli Ticari Alacaklar, 129 Şüpheli Ticari Alacaklar Karşılığı (-).
- **TFRS hedefi:** Ticari alacaklar **net** (brüt alacak – zarar karşılığı) tek satırda sunulur; TDHP'de 128 (brüt) ile 129 (karşılık) ayrı görünür. Sunumda karşılık ilgili alacak satırından **düşülür**. (Not: karşılık *tutarının* VUK ≠ TFRS 9 beklenen kredi zararı farkı **AJE**'dir; buradaki iş yalnızca net gösterimdir.)
- **Tetikleyici veri:** Karşılık hareket tablosu; 128/129 bakiyeleri.
- **RJE:** Sunum netleştirmesi — 128 ve 129, "Ticari alacaklar" satırında net gösterilir.
- **Kaynak:** ([bilgi.edu.tr — alacaklar ve karşılıklar alt sınıfları](https://openaccess.bilgi.edu.tr/server/api/core/bitstreams/734cc404-765c-497f-8f1b-81f7835095df/content))

### K-8. Stok ↔ duran varlık arası sınıflama (TFRS 5 — Satış amaçlı elde tutulan)
- **TDHP kaynağı:** 252/253/254 Maddi duran varlıklar (satışa çıkarılan); nadiren 15x stok.
- **TFRS hedefi:** Kullanımı durdurulup **aktif olarak satışa** konu, satışı 12 ay içinde yüksek olası duran varlık, ayrı **"Satış amaçlı elde tutulan duran varlıklar"** satırında (dönen varlık grubu altında ayrı) gösterilir; amortisman durur. Stoklarla karıştırılmamalıdır.
- **Tetikleyici veri:** Yönetim satış kararı, aktif pazarlama, satış olasılığı ≥ 12 ay kriteri.
- **RJE:** Borç "Satış amaçlı elde tutulan duran varlıklar" / Alacak 252 (net defter değeri).
- **Kaynak:** ([KGK TFRS 5](https://www.kgk.gov.tr/Portalv2Uploads/files/DynamicContentFiles/T%C3%BCrkiye%20Muhasebe%20Standartlar%C4%B1/TMSTFRS2018Seti/TFRS/TFRS_5_2018.pdf))

### K-9. Ertelenmiş vergi varlık/yükümlülük netleştirmesi (TMS 12)
- **TDHP kaynağı:** TDHP'de tek düzen karşılığı yok; dönüşüm çalışmasında hesaplanan ertelenmiş vergi varlığı (EVV) ve yükümlülüğü (EVY).
- **TFRS hedefi:** TMS 12 §74 — aynı vergi otoritesi ve yasal mahsup hakkı varsa EVV ve EVY **netleştirilerek** tek kalemde ("Ertelenmiş vergi varlığı" *veya* "yükümlülüğü", **her zaman duran/uzun vadeli**) sunulur; asla dönen varlık değildir.
- **Tetikleyici veri:** Geçici farklar tablosu; aynı vergi idaresi/mahsup hakkı kontrolü.
- **RJE:** Netleştirme — EVV ve EVY karşılıklı mahsup, kalan net bakiye tek satır.
- **Kaynak:** ([muhasebetr — TMS 12 ertelenmiş vergi](https://www.muhasebetr.com/yazarlarimiz/mahmutsemihsekercioglu/004/))

### K-10. Yatırım amaçlı gayrimenkul ayrımı (TMS 40 — sınıflama bacağı)
- **TDHP kaynağı:** 250 Arazi ve Arsalar, 252 Binalar (kira geliri/değer artışı amaçlı tutulanlar).
- **TFRS hedefi:** Kullanım/üretim değil, **kira geliri veya değer artışı** amaçlı gayrimenkul, MDV'den ayrılıp **"Yatırım amaçlı gayrimenkuller"** satırına taşınır. (Ölçüm modeli seçimi ayrı AJE; buradaki iş sınıflama bacağıdır.)
- **Tetikleyici veri:** Gayrimenkulün kullanım amacı; kira sözleşmeleri.
- **RJE:** Borç "Yatırım amaçlı gayrimenkuller" / Alacak 252.

### K-11 (ek). Kıdem tazminatı karşılığının vadeye göre ayrımı
- **TDHP kaynağı:** 372 Kıdem Tazminatı Karşılığı (KV), 472 (UV).
- **TFRS hedefi:** "Çalışanlara sağlanan faydalara ilişkin karşılıklar" — 12 ay içi ödenmesi beklenen kısım kısa, kalan uzun vadeli. (Aktüeryal *ölçüm* farkı AJE'dir.)
- **Tetikleyici veri:** Aktüer raporu, tahmini ödeme takvimi.

---

## 3. Bu kayıtlar RJE mi? Belgeleme ve dayanak

**Evet — hepsi RJE'dir (Reclassifying Journal Entry).** Ortak özellikler:
- **Kâr/zararı etkilemez** (gelir tablosu satırlarına dokunmaz).
- **Tutar-nötrdür** (borç = alacak; net varlık ve özkaynak sabit).
- Sadece **finansal durum tablosu satırları arasında** taşıma yapar (bir istisna: gelir tablosu içi yeniden sınıflamalar da olabilir; bu proje bilanço odaklı).

**Denetimde dayanak (audit evidence):** Her RJE satırı bir **kanıta** bağlanmalıdır:
- K-1: Kredi **itfa/geri ödeme planı**, banka sözleşmesi
- K-2: **İlişkili taraf listesi** + cari hesap mutabakatı
- K-4/K-5: **Sözleşme**, teslim/dönem takvimi
- K-6: **Mevduat vade dökümü**, blokaj yazısı, çek vade listesi
- K-8: Yönetim **satış kararı** tutanağı, pazarlama kanıtı
- K-9: **Geçici farklar tablosu**, mahsup hakkı analizi

Her satır **bağımsız kabul/ret** edilebilmelidir: denetçi bir satırın gerekçesini yetersiz bulursa yalnızca onu reddeder, diğerleri etkilenmez.

---

## 4. ÖNERİLEN ÇALIŞMA KAĞIDI YAPISI (denetçinin göreceği)

Denetçi, sınıflandırma çalışmasını **kalem kalem, her satır bağımsız onaylanabilir** bir tablo olarak görmek ister. Mevcut tek-kalemlik ekranın yerine önerilen yapı:

### 4.1 Üst özet şeridi
```
TMS 1 — SINIFLANDIRMA (RJE)     Mükellef: ...   Dönem: 2025
Toplam RJE satırı: 11   |   Onaylı: 8   |   Bekleyen: 2   |   Reddedilen: 1
Net kâr/zarar etkisi: 0,00 ₺  ✓ (nötr)   |   Bilanço denge: DENGE ✓
```
(Tutar-nötrlük ve denge her an görünür; RJE mantığının otomatik kontrolü.)

### 4.2 Kalem kalem RJE tablosu
Her satır bir sınıflandırma; genişletilebilir (accordion) — açılınca dayanak ve bacaklar görünür.

| # | Sınıflandırma | Kaynak (TDHP) | Hedef (TFRS satırı) | Tutar (₺) | Dayanak | Durum |
|---|---------------|---------------|---------------------|-----------|---------|-------|
| K-1 | UV kredinin KV kısmı | 400 | UV borçl. KV kısmı (303) | 1.250.000,00 | İtfa planı | ✅ Onaylı |
| K-2 | İlişkili taraf ayrımı | 120 | İlişkili tar. tic. alacak | 480.000,00 | Cari analiz | ⏳ Bekliyor |
| K-6 | Vadeli mevduat (>3 ay) | 102 | Finansal yatırımlar | 900.000,00 | Vade dökümü | ✅ Onaylı |
| … | … | … | … | … | … | … |

### 4.3 Satır detay paneli (açılınca)
```
K-1  Uzun vadeli kredinin kısa vadeli kısmı
─────────────────────────────────────────────
Bacaklar (RJE — kâr/zarar etkisi yok):
   BORÇ    400 Banka Kredileri (UV)          1.250.000,00
   ALACAK  303 UV Kredi KV Kısmı             1.250.000,00
Gerekçe:  X Bankası kredisinin 12 ay içinde vadesi
          gelen 5 taksiti (itfa planı ekli).
Dayanak:  📎 kredi_itfa_plani_2025.pdf
Standart: TMS 1 §69-72
[ Onayla ]  [ Reddet ]  [ Not ekle ]
```

### 4.4 Tasarım/UX önerileri (App.tsx bileşen diline uygun)
- Her RJE satırı bir **card**; durum için **pill** (ok=yeşil onaylı, warn=sarı bekleyen, kırmızı #E23A32 reddedilen).
- Kaynak↔hedef eşlemesi **görsel ok/akış** (soldan TDHP hesabı → sağda TFRS satırı) ile gösterilsin; salt metin liste "eziyet" şikâyetinin kaynağı.
- Hesap seçiminde mevcut **HesapSecici combo** yeniden kullanılabilir.
- Bacak tutarları **.num/.mono** ile sağ hizalı; kuruş (i64) → 2 hane format.
- Üstte kalıcı **denge/nötrlük rozeti** (RJE'nin tanımı gereği her zaman 0 olmalı; ihlalinde kırmızı uyarı).
- Sidebar'dan tıklanınca **/worksheet/tms-1/siniflandirma** gibi ayrı URL açılmalı (kullanıcının URL routing şikâyeti); her RJE satırına **deep-link** (ör. `#K-1`).
- Toplu işlem: "Tüm dayanağı olanları onayla", filtre (onaylı/bekleyen), dışa aktar (denetim dosyası PDF).

---

## Özet
Mevcut "sadece UV kredinin KV kısmı" içeriği, gerçek sınıflandırma çalışmasının **11 kaleminden yalnızca 1'idir**. TMS 1 sınıflandırması; nakit tanımı (TMS 7), ilişkili taraf ayrımı (TMS 24), avans/ertelenmiş gelir (TFRS 15), satış amaçlı varlık (TFRS 5), ertelenmiş vergi netleştirmesi (TMS 12) ve vade bazlı cari/duran ayrımını kapsayan; **kâr/zarar-nötr, tutar-nötr, dayanağa bağlı, satır satır onaylanabilir** bir RJE seti olarak modellenmelidir.

## Kaynaklar
- [KGK — TMS 1 Finansal Tabloların Sunuluşu (§60–76 cari/cari olmayan ayrımı)](https://www.kgk.gov.tr/Portalv2Uploads/files/Duyurular/v2/TMS/TMS_1_Finansal%20Tablolar%C4%B1n%20Sunulu%C5%9Fu.pdf)
- [KGK — TMS 7 Nakit Akış Tablosu (nakit ve nakit benzerleri, ≤3 ay)](https://www.kgk.gov.tr/Portalv2Uploads/files/DynamicContentFiles/T%C3%BCrkiye%20Muhasebe%20Standartlar%C4%B1/TMSTFRS2018Seti/TMS/TMS%207%20Nakit%20Ak%C4%B1%C5%9F%20Tablosu%20Kurul%20Karar%C4%B1(2).pdf)
- [KGK — TFRS 5 Satış Amaçlı Elde Tutulan Duran Varlıklar](https://www.kgk.gov.tr/Portalv2Uploads/files/DynamicContentFiles/T%C3%BCrkiye%20Muhasebe%20Standartlar%C4%B1/TMSTFRS2018Seti/TFRS/TFRS_5_2018.pdf)
- [KGK — Finansal Raporlama Standartlarına Uygun Hesap Planı Taslağı (TDHP→TFRS eşlemesi)](https://www.kgk.gov.tr/Portalv2Uploads/files/Duyurular/v2/TFRS/EK2_Finansal%20Raporlama%20Standartlar%C4%B1na%20Uygun%20Hesap%20Plan%C4%B1%20Tasla%C4%9F%C4%B1.pdf)
- [ACCAFIN — TMS 1'de "Cari" ve "Cari Olmayan" kavramları](https://www.accafin.com/muhasebe/ifrs-ias-ufrs-tms-usgaap/tms-1-finansal-tablolarin-sunulusu-nda-cari-ve-uzun-donem-kavramlari)
- [ACCAFIN — TMS 7 kapsamında bloke mevduatların sunumu](https://www.accafin.com/muhasebe/ifrs-ias-ufrs-tms-usgaap/tms-7-nakit-akis-tablosu-kapsaminda-bloke-mevduat)
- [İstanbul Bilgi Üniv. — TMS 1 alt sınıflar (müşteri/ilişkili taraf/peşin ödeme ayrımı)](https://openaccess.bilgi.edu.tr/server/api/core/bitstreams/734cc404-765c-497f-8f1b-81f7835095df/content)
- [muhasebetr — TMS 12 Ertelenmiş Vergi](https://www.muhasebetr.com/yazarlarimiz/mahmutsemihsekercioglu/004/)
- [DergiPark — Vergi Mizanından FRS'ye Dönüşüm Kayıtları](https://dergipark.org.tr/en/download/article-file/928117)

Not: `jafas.org` üzerindeki VUK→TFRS makalesi, alakasız bir hosta (`spmbjabar.com`) yönlendirdiği için güvenlik gereği takip edilmedi; ilgili içerik diğer KGK/akademik kaynaklardan doğrulandı.