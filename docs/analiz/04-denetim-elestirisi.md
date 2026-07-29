I now have all the grounding I need. Here is the audit report.

---

# DENETİM RAPORU — Audit-Liners Muhasebe Çekirdeği vs. TDHP/VUK/TMS Referans Dokümanları

Kapsam: `crates/domain/src/{donem,mali_tablo,kurallar}.rs`, `data/hesap-kurallari.json`, `docs/tasarim/*`. Format: **Bulgu | Etki | Dayanak | Öneri**. En kritik 12 madde, önem sırasına göre.

---

### 1. Vergiyi doğuran olay / KDV motoru KAYIT tarafında hiç yok — sadece tasarım notu
**Bulgu:** `maliyet-ve-vergi.md` "Vergiyi doğuran olay motoru"nu (KDV = teslim/hizmet ifası anında 191/391; ödeme değil) net tanımlamış, ama `domain`'de tek bir satır kod yok. Fiş üretimi (`fis.rs`/`kurallar.rs`) KDV/stopaj/damga satırını orandan türetmiyor; kullanıcı 191/391'i elle girmek zorunda. `VergiOlayi { tip, matrah, oran, hesap }` yapısı hiç uygulanmamış.
**Etki:** Veresiye satışta bile KDV'nin teslim anında doğduğu kural sisteme gömülü değil → mükellef KDV'yi ödeme anına kaydederse (yaygın hata) yazılım engellemez/uyarmaz. Beyanname temeli çürük.
**Dayanak:** `maliyet-ve-vergi.md` bölüm C; Genel Muhasebe Kitabı ("KDV brüt satışlara dahil edilmez… 191/391/190/360"); TMS-benzeri değil doğrudan KDVK md.10 / VUK md.19.
**Öneri:** `domain`'e `vergi.rs` ekle: işlem tipi → vergi olayı → otomatik KDV satırı (matrah×oran), tevkifat (2 No.lu KDV / kısmi) ve istisna bayrağı. Oranlar config'te (%1/%10/%20), tarih-etkin.

---

### 2. Ödenecek/Devreden KDV mantığı (aylık mahsup) hiç kurulmamış
**Bulgu:** `hesap-kurallari.json` 191→391, 391→191, 190→191/391 kapatma alanlarını doğru tutuyor, ama kodda aylık KDV mahsup rutini yok: dönem sonunda `391 borç / 191 alacak`, kalan borç `360 Ödenecek KDV`, kalan alacak `190 Devreden KDV` üretimi yapılmıyor.
**Etki:** KDV beyannamesi üretilemez; devreden KDV sonraki aya taşınamaz. Bu SMMM için günlük iş — eksikliği ürünü kullanılamaz kılıyor.
**Dayanak:** Ön Büroda Muhasebe & Genel Muhasebe Kitabı: "191 İndirilecek, 391 Hesaplanan, 190 Devreden, 360 Ödenecek". `hesap-kurallari.json` 190 `kapatma: "191/391"`.
**Öneri:** Aylık KDV kapanış virmanı üreten fonksiyon; `min(hesaplanan, indirilecek)` mahsup, artık borç→360, artık alacak→190. Ayrı dönem-içi (aylık) periyot kavramı gerekir.

---

### 3. Dönem sonu değerleme düzeltmeleri tamamen yok — kapanış "ham" mizandan yapılıyor
**Bulgu:** `donem.rs::kapanis()` doğrudan `defter.mizan()`'ı 690'a topluyor. Öncesinde hiçbir dönem sonu düzeltmesi yok: amortisman (257/268), şüpheli alacak karşılığı (129/654), reeskont (122/322/647/657), stok değer düşüklüğü (158), kıdem tazminatı karşılığı (372).
**Etki:** Kâr/zarar YANLIŞ hesaplanır — bunlar giderleşmeden 690'a gidiliyor. `donem_sonucu()` VUK'a göre bile hatalı sonuç verir. Dönemsellik ve ihtiyatlılık ilkesi ihlali.
**Dayanak:** Genel Muhasebe Kitabı (reeskont iç iskonto formülü, amortisman, şüpheli alacak %100/teminat hariç); TMS 16/36/37; `hesap-kurallari.json` bu hesapları içeriyor ama kod kullanmıyor.
**Öneri:** `kapanis()`'ten ÖNCE zorunlu bir "dönem sonu envanter/değerleme" adımı: amortisman planı, reeskont, karşılık motorları. Sıra: değerleme → 7→6 yansıtma → 6→690.

---

### 4. ENFLASYON DÜZELTMESİ yok — mali tablolar Türkiye'de yanıltıcı
**Bulgu:** Ne kod ne veri modeli enflasyon düzeltmesini biliyor. Parasal/parasal-olmayan ayrımı (hesap bazlı bayrak) yok; Yİ-ÜFE endeks tablosu, düzeltme katsayısı, 698 Enflasyon Düzeltme Hesabı, net parasal pozisyon kâr/zararı yok.
**Etki:** **VUK Geçici 37 ile 2025-2027 hesap dönemlerinde vergisel enflasyon düzeltmesi YAPILMAYACAK** — yani bu üç yıl için "eksikliğimiz" pratikte VUK ile aynı hizada (istisna: münhasıran altın/gümüş). ANCAK: (a) TMS 29'a göre finansal raporlama için düzeltme takdiri devam eder (son 3 yıl kümülatif >%100 eşiği Türkiye'de aşıldı); (b) 2028+ için VUK düzeltmesi geri dönebilir; (c) hesap bazlı parasal/parasal-olmayan altyapısı yoksa hiçbir zaman uygulanamaz.
**Dayanak:** TMS 29 (parasal kalem düzeltilmez, parasal-olmayan Yİ-ÜFE ile düzeltilir); Kurumlar Vergisi Örnek 2025 (VUK Geçici 37: 2025-2027 düzeltme yapılmaz).
**Öneri:** Şimdi kod yazmaya gerek yok ama hesap planına `parasal: bool` alanı ve `iktisap_tarihi`/endeks alanları ekle (altyapı). 2028 riskini `docs/`'a not düş; kullanıcıya "mali tablolar tarihi maliyet, enflasyon düzeltmesiz" uyarısı göster.

---

### 5. Mali tablonun hangi standartta olduğu belirsiz — TMS/VUK karışıklık riski
**Bulgu:** `mali_tablo.rs` TDHP grup kodlarından (60/62/63…) tek bir bilanço+gelir tablosu üretiyor. Bu **VUK/TDHP tablosu**, ama kodda/dokümanda açık etiket yok. Ertelenmiş vergi, diğer kapsamlı gelir, yeniden değerleme fonu gibi TMS kavramları hiç yok; "hangi defter" ayrımı (VUK değeri vs TMS değeri) yapılmamış.
**Etki:** Kullanıcı bu tabloyu TFRS raporu sanabilir. Neredeyse tüm referans dokümanlar "yazılım VUK defteri ile TMS değerini AYRI saklamalı" diyor — tek değerli model bu ayrımı yapısal olarak imkânsız kılıyor.
**Dayanak:** Her TMS dokümanının `vuk_tms_farki` bölümü ("çift defter / ertelenmiş vergi köprüsü zorunlu"); Ön Büroda Muhasebe (VUK vs TMS/TFRS iki ayrı hesap planı).
**Öneri:** Mali tabloya açık `standart: VUK` etiketi. Orta vadede her hesap/varlık için VUK-değeri ve TMS-değeri alanları; ertelenmiş vergi köprüsü (283/293/483/493) için geçici fark tablosu.

---

### 6. Kapanış virman sırası doğru AMA vergi karşılığı (690→370, 691) adımı atlanmış
**Bulgu:** `donem.rs`'in sırası (6→690, 690→692, 692→590/591) tasarım dokümanıyla uyumlu. Ancak `donem-kapanis-acilis.md` adım 2 "Vergi karşılığı (kâr varsa): 690 BORÇ / 370 ALACAK, 691 üzerinden" diyor; kodda bu adım **yok** — yorumda "vergi karşılığı 0 — vergi motoru sonra eklenecek" yazıyor ve 690 doğrudan 692'ye tam tutar aktarılıyor.
**Etki:** Kurumlar vergisi tahakkuku hiç kaydedilmiyor → 692 net kâr, vergi öncesi kârla eşit çıkıyor → 590'a brüt kâr taşınıyor. Ödenecek vergi (370) bilançoda görünmüyor. `donem_sonucu()` de 69x'i dışlayıp aynı hatayı sürdürüyor.
**Dayanak:** `donem-kapanis-acilis.md` adım 2-3; Ön Büroda Muhasebe (691→370, 690/691/692 zinciri); `hesap-kurallari.json` 691 `karsi:["370","690","692"]`.
**Öneri:** `kapanis()`'e adım 2 ekle: kâr>0 ise mali kâr üzerinden KV hesapla (min. asgari KV kontrolü ileride), `691 borç / 370 alacak` + `690 borç / 691,692 alacak`. Vergi motoru gelene kadar oranı parametre al.

---

### 7. `690` grubu `donem_sonucu()` ve gelir tablosunda tutarsız işleniyor
**Bulgu:** `donem_sonucu()` sınıf 6'yı toplarken `grup=="69"` olanı dışlıyor (doğru). Ama `gelir_tablosu()` yalnızca 60-68'i topluyor, 69'u hiç kullanmıyor — yani `gelir_tablosu().donem_kari` vergi karşılığını (691) hiçbir zaman göremez. Kod yorumu "`donem_kari == donem_sonucu(defter)` olmalıdır" diyor ama 691 devredevreye girince ikisi ayrışır.
**Etki:** Vergi karşılığı eklendiği an gelir tablosu net kârı ≠ 692 net kârı; iki fonksiyon çelişir. Sessiz tutarsızlık.
**Dayanak:** Ön Büroda Muhasebe gelir tablosu basamakları ("Dönem Kârı − Vergi Karşılığı = Dönem Net Kârı"); madde 6 ile bağlı.
**Öneri:** Gelir tablosuna `vergi_karsiligi` (691) ve `donem_net_kari` satırları ekle; `donem_kari` = vergi öncesi, `donem_net_kari` = 692 ile mutabık olsun.

---

### 8. Bordro / SGK / stopaj tamamen yok
**Bulgu:** Ne domain'de ne veri modelinde ücret bordrosu, SGK primi, gelir vergisi stopajı, damga vergisi (bordro), muhtasar var. 360 Ödenecek Vergi ve Fonlar ve 335/770 hesapları planda var ama besleyen hiçbir motor yok.
**Etki:** SMMM'nin en yoğun aylık işlerinden biri (bordro + muhtasar + SGK bildirimi) kapsam dışı. Kıdem tazminatı gideri, SGK işveren payı gibi kalemler dönem giderine hiç girmiyor → kâr yine hatalı.
**Dayanak:** Bordrolama S&C ve SGK Son Değişiklikler dokümanları (2026 parametreleri, brütten nete, kümülatif GV matrahı, damga 0,00759).
**Öneri:** Ayrı `bordro` modülü + yıl-bazlı parametre tablosu (asgari ücret, SPEK tavan, prim oranları, GV dilimleri). Bordrodan 335/360/770 yevmiye fişi üretimi.

---

### 9. Kıdem tazminatı karşılığı (TMS 19) ve ertelenmiş vergi (TMS 12) yok
**Bulgu:** `hesap-kurallari.json` 372 Kıdem Tazminatı Karşılığı hesabını içeriyor (`karsi:["770","100"]`) ama aktüeryal hesaplama motoru, iskonto, OCI'ye aktüeryal fark ayrımı yok. Ertelenmiş vergi varlığı/borcu (283/293/483/493) hesap planında bile yok.
**Etki:** TMS raporlaması iddia edilirse eksik; VUK tarafında da kıdem karşılığı KKEG olduğundan geçici fark doğar ama bu fark izlenmiyor → ertelenmiş vergi köprüsü kurulamaz.
**Dayanak:** TMS 19 (öngörülen yükümlülük yöntemi, iskonto, aktüeryal fark OCI); TMS 12 (VUK ile TMS farkı → geçici fark → ertelenmiş vergi); Bordrolama S&C `vuk_tms_farki`.
**Öneri:** TMS iddiası varsa TMS 19 aktüeryal modül; yoksa en azından kıdem karşılığını KKEG olarak işaretle ve geçici fark tablosuna yaz.

---

### 10. Kapanış ve bilanço/açılış fişleri arasında sıra bağımlılığı sessiz — yanlış çağrı sırası yanlış tablo üretir
**Bulgu:** `bilanco_kapanis()` yorumu "kapanis() virmanları DEFTERE eklendikten SONRA çağrılmalı" diyor, ama bu bir konvansiyon; kod bunu zorlamıyor. `acilis()` ise "önceki dönemin **kapanış öncesi** bilanço mizanından" üretiliyor — yani 590/591 daha işlenmemişken. İki fonksiyon farklı defter-durumu varsayıyor ve tip sistemi bunu ayırt etmiyor.
**Etki:** Yanlış sırada çağrılırsa (ör. bilanco_kapanis'i kapanış virmanı eklenmeden çağırmak) 690/692 hâlâ dolu → dengesiz/yanlış kapanış fişi. Açılış, dönem kârı 590'a taşınmadan üretilirse öz kaynak eksik devreder.
**Dayanak:** `donem-kapanis-acilis.md` adım 4-5 ("kapanis virmanları eklendikten sonra 590 dolu olsun"); Genel Muhasebe Kitabı süreç sırası (kesin mizan → kapanış).
**Öneri:** Defter durumunu tipe göm (`AcikDefter`/`KapanmisDefter` ayrı tipler) veya kapanış akışını tek orkestratör fonksiyonda zincirle; ara durum çağrısını derleme/çalışma zamanı hatası yap.

---

### 11. `hesap-kurallari.json` — bazı `karsi`/`kapatma` bilgileri eksik/yanıltıcı
**Bulgu:** İki somut tutarsızlık:
- **372 Kıdem Tazminatı Karşılığı** `karsi:["770","100"]`. Karşılık ayrılırken karşı bacak gider hesabı **654 Karşılık Giderleri** (veya 630/660) olmalı; 372'nin tipik karşı bacağı 770 değil 654'tür. 770 (fonksiyonel gider) doğrudan 372'yi karşılamaz — arada 654 var. Ayrıca ödeme bacağı için 100/102 doğru ama karşılık AYIRMA bacağı eksik.
- **600 Yurtiçi Satışlar** `karsi` listesinde `391` var (doğru, KDV bacağı) ama **101 Alınan Çekler** ve **121 Alacak Senetleri** yok; peşin/çekli/senetli satış yaygın karşı bacaklar. Kapsam eksikliği öneri motorunu zayıflatır.
- **654 Karşılık Giderleri** `karsi:["129","158","690"]` — 372 (kıdem) burada yok; 654↔372 ilişkisi iki yönden de kopuk.
**Etki:** Otomatik karşı-hesap önerisi ve doğrulama yanlış/eksik öneri verir; kıdem tazminatı karşılık fişi doğru karşı bacakla üretilemez.
**Dayanak:** Genel Muhasebe Kitabı (karşılık: 654 borç / 129,158,372 alacak); TMS 19/37 (karşılık gideri kâr/zararda).
**Öneri:** 372 `karsi`'ya 654 ekle (karşılık ayırma); 654 `karsi`'ya 372 ekle; 600 `karsi`'ya 101,121 ekle. Çift yönlü tutarlılığı bir test ile doğrula (A hesabının karsi'sinde B varsa mantıksal ters ilişki gözden geçirilsin).

---

### 12. Tevkifat (kısmi KDV / stopaj) ve damga vergisi hiç modellenmemiş
**Bulgu:** Tasarım C bölümü tevkifatı (2 No.lu KDV, kısmi KDV) ve stopajı (360) listeliyor ama ne 360'ı besleyen tevkifat motoru ne 2 No.lu KDV kavramı var. Damga vergisi (kâğıdın düzenlenmesiyle doğar) hiç yok.
**Etki:** Serbest meslek, kira, inşaat gibi tevkifatlı işlemler doğru fiş üretemez; muhtasar beyanname temeli yok. Örtülü sermaye/TF stopajı gibi ileri senaryolar tamamen kapsam dışı.
**Dayanak:** `maliyet-ve-vergi.md` bölüm C; Kurumlar Vergisi Örnek 2025 (örtülü sermaye stopajı, 2 No.lu KDV tevkifatı); Bordrolama (stopaj 360).
**Öneri:** Vergi motoruna (madde 1) tevkifat alt-kuralı: matrahın tevkif oranına düşen kısmı `360`'a, sorumlu sıfatıyla 2 No.lu KDV ayrı beyan. Damga için "kâğıt düzenlendi" olayına bağlı 0,00759 satırı.

---

## Özet Öncelik Sırası
1. **Acil (kâr yanlış hesaplanıyor):** Madde 3 (değerleme düzeltmeleri), Madde 6 (vergi karşılığı virmanı), Madde 7 (gelir tablosu/donem_sonucu tutarsızlığı).
2. **Yapısal (temel eksik):** Madde 1-2 (KDV/vergiyi doğuran olay + ödenecek/devreden), Madde 12 (tevkifat/damga), Madde 8 (bordro/SGK).
3. **Standart/altyapı:** Madde 5 (VUK/TMS ayrımı + etiket), Madde 4 (enflasyon altyapısı — kod değil, hesap alanları), Madde 9 (kıdem/ertelenmiş vergi).
4. **Veri/tutarlılık:** Madde 11 (`hesap-kurallari.json` karsi/kapatma düzeltmeleri), Madde 10 (kapanış sıra bağımlılığını tipe gömme).

**En kritik tek bulgu:** Dönem sonu değerleme + vergi karşılığı olmadan `kapanis()`/`donem_sonucu()` ham mizandan kâr üretiyor — bu, mali tablonun temel doğruluğunu bozan ve zincirin en başındaki hatadır (Madde 3+6+7 birlikte).

İlgili dosyalar: `/Users/arda/Desktop/Audit-Liners/crates/domain/src/donem.rs` (satır 23-31 `donem_sonucu`, 34-104 `kapanis`), `/Users/arda/Desktop/Audit-Liners/crates/domain/src/mali_tablo.rs` (satır 64-95 `gelir_tablosu`), `/Users/arda/Desktop/Audit-Liners/data/hesap-kurallari.json` (kayıt 372, 600, 654), `/Users/arda/Desktop/Audit-Liners/docs/tasarim/maliyet-ve-vergi.md` (bölüm C — uygulanmamış).