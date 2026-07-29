# Açık İşler — 28.07.2026

Bu oturumda biriken eksikler. Sıralama **etkiye** göre: en üstteki, ürünün bugün
yapamadığı işi engelliyor; en alttaki iyileştirme.

Her madde için: **ne eksik · neden önemli · nasıl anlaşılır (kanıt)**.

---

## A. BELGE OKUMA — ürünün asıl işi

### A1. Türkçe OCR modeli yok ⛔
Kullanılan model genel Latin alfabesi için eğitilmiş. Taranan belgede
`ÖRNEK TEKSTİL` → `ORNEK TEKSTiL`, `ÇORAP` → `CORAP`, `Kadıköy` → `Kadikoy`.

**Neden önemli:** Tutarlar denklemle korunuyor ama **metin korunmuyor** — metnin aritmetiği
yok. Fişe yazılan cari ünvanı ve mal adı bozuk gidiyor; cari eşleştirme ve Ba/Bs
mutabakatı bunun üstüne kurulamaz.

**Kanıt:** ÖRNEK TEKSTİL faturası taraması — 7/7 sayısal alan doğru, ünvan bozuk.

### A2. Kural itibarı ölçülmüyor ⛔
Yanlış öğrenilmiş bir kural kendiliğinden temizlenmiyor. `isabet`/`deneme` alanları var
ama denklem bir kuralı çürüttüğünde düşürülmüyor.

**Neden önemli:** Bugün bizzat yaşandı — kirli çıkarım döneminde öğrenilmiş bir kural,
kod düzeldikten SONRA bile kirli değer üretti ve puanı en yüksek olduğu için kazandı.
Elle silmek zorunda kaldım.

**Yapılacak:** Denklem bir kuralın ürettiği değeri reddettiğinde `isabet` düşsün;
belli eşiğin altına inen kural kendiliğinden unutulsun.

### A3. Kalem çıkarımı ızgarayı kullanmıyor
`belge_kalem` satır metni üzerinde çalışıyor, hücre ızgarası üzerinde değil.
Sonuç: `"ÇORAP ADET"` — mal adına birim yapışıyor.

**Yapılacak:** `satir_kalem` yerine hücre listesi alan bir sürüm; sütun indeksi bilinince
ad/birim/miktar/fiyat/tutar ayrımı kesinleşir.

### A4. Denklem yalnız ARİTMETİK ⛔ (değerleme raporlarının kapısı)
`belge-parametreleri.json` yalnız `terimler`/`hedef` toplama denklemi destekliyor.
Değerleme raporunda aritmetik yok; doğrulanabilir başka bağıntılar var:

| Tür | Örnek |
|---|---|
| **ARALIK** | ulaşılan değer, gösterilen emsallerin arasında mı |
| **TARIH_SIRASI** | değerleme tarihi ≤ rapor tarihi, dönem içinde |
| **ZORUNLU_BEYAN** | TFRS 13 girdi seviyesi beyan edilmiş mi |
| **DEFTER_KARSILASTIR** | rapordaki değer = deftere alınan tutar |

Sonuncusu en güçlü kozumuz: defter zaten elimizde, belge dışından bağımsız tanık.
**Bu olmadan `belge-evreni.json`'daki 5 değerleme türü okunamaz.**

### A5. UBL-TR XML yolu bağlı değil
e-Fatura'nın XML'i varsa OCR'a **hiç gerek yok** — birebir okunur, hata payı sıfır.
Şu an sadece PDF/görüntü yolu var.

**Neden önemli:** Günlük hacmin en büyük kalemi e-Fatura. En kesin yol bağlanmamış.

### A6. Çok oranlı KDV faturası
Bir faturada %1 + %10 + %20 birlikte olabilir. Şu an tek `kdv` alanı ve tek denklem var;
böyle bir faturada denklem tutmaz ve otomatik doldurma hiç yapılmaz.

### A7. İstisna ve iskonto alanları yok
Tevkifat eklendi; **ihracat istisnası** (601, KDV doğmaz) ve **fatura sonrası iskonto**
(611) fatura akışında yok. Belgede `İSK %` sütunu var, biz okumuyoruz.

### A8. Çok sayfalı belge
Yalnız 1. sayfa OCR'lanıyor ve önizleniyor. Metin katmanlı PDF'te tüm sayfalar okunuyor
ama taranmışta yalnız ilk sayfa. Parmak izi de ilk 60 satıra bakıyor.

### A9. Taranmış belgede öğrenme zayıf
OCR her seferinde biraz farklı okuyabilir → parmak izi tutmaz → şablon birikmez.
Parmak izi OCR gürültüsüne dayanıklı hale gelmeli (harf iskeleti yerine daha kaba özet).

### A10. Yakınsama sayacı yok
"Öğrenme birikiyor" bir **tez**, kanıt değil. Kaç belge otomatik doldu, kaç tanesi elle,
hangi şablon kaç kez işe yaradı — ölçülmüyor.

**Bu olmadan A2, A9 ve genel mimarinin işe yarayıp yaramadığı bilinemez.**

### A11. Gerçek belge regresyon seti boş
`data/ornek-belgeler/` boş. Her düzeltme, elle kurduğum sentetik belgelerle sınanıyor.
Gerçek belge + beklenen değer çifti olmadan **isabet oranı ölçülemez**.

---

## B. MUHASEBE DOĞRULUĞU — denetimden kalan açık bulgular

Bunlar üç ajanlı denetimde bulunup **düzeltilmeyen** maddeler. Beşi düzeltildi
(K05 yön, DELETE geri alma, K01 paniği, karma enstrüman, havada uçları) — bunlar kaldı.

### B1. Fişlerin %100'ünde açıklama boş ⛔
`main.rs` açıklamayı hesaplayıp `let _ = ack;` ile atıyor. 11.085/11.085 fişte boş.
Storno ile gerçek tahsilat yevmiyede **ayırt edilemiyor** (VUK 219 / TTK 65).

### B2. Storno fişi yanlış tipleniyor
`fis_tipi()` `k.storno` bayrağını okumuyor; storno Tahsil/Tediye olarak yazılıyor.
Kodun kendi yorumu "storno Mahsup fişidir" diyor.

### B3. Her fişin dayanağı sabit "Fatura"
Açılış, virman, tahsilat, storno dahil. `BelgeTipi::{Makbuz, Dekont, Diger}` domainde var,
kullanılmıyor. VUK 227/229: tahsilat makbuzu fatura değildir.

### B4. Kronoloji koruması olay tarihini eziyor
Defterin son fiş tarihinden eski her aktarım o tarihe kaydırılıyor. Nisan kasa tahsilatı
Aralık'a yazılabiliyor → dönemsellik ve geçici vergi bilançoları bozulur.

### B5. %44 gelecek tarihli fiş
4.845 fiş bugünden ileri tarihli (en ileri 28.12.2026). "Bugünden sonrasına kayıt
yapılamaz" kuralı yok.

### B6. Açılış fişi kodda sabit tapa
102: 60.000.000 TL kodda gömülü. Bu olmadan 102 alacak bakiye veriyor — yani
`denetim_anomali`'nin en yüksek önemli bulgusu sabit bir tutarla bastırılıyor.

### B7. KDV1 matrahı defterden geri hesaplanıyor
`matrah = hesaplanan × 100 / oran`. Tevkifatta yarım matrah + mükerrer beyan;
iade KDV'si indirilecek KDV'ye karışıp hayali alış üretiyor. Matrah belge kaynağından
taşınmalı.

### B8. Banka ekstresi hiç üretilmiyor
`/api/banka` ucu çalışıyor ve `POST /api/banka/ekstre` ile dolduruluyor — ama **simülasyon
ekstre üretmiyor.** `st.banka` boş başlıyor ve kullanıcı elle ekstre yüklemedikçe boş kalıyor;
banka mutabakat ekranı açılışta tamamen boş. Simülasyon ödemeleri zaten üretiyor, karşılık
gelen ekstre satırını da üretmeli — yoksa eşleştirmenin sınanacağı veri yok.

### B9. `/api/banka/:id/esle` doğrulama yapmıyor
Aynı fiş iki zıt yönlü ekstre satırına bağlanabiliyor. `banka_oneri` filtreleri zaten var,
`esle` kullanmıyor.

### B10. K07 kısmi doldurmada devre dışı
Yalnız hedef tamamen manuel doluysa engelliyor; kısmi durumda manuel karar manuel kararı
düşürebiliyor ve kazananı tahsilat tarihi belirliyor.

### B11. Avans deftere girmiyor
1.642.806 TL; 340/159 hesapları yalnız uyarı metninde. Banka ekstresinde para var,
102'ye girmiyor.

### B12. Tevsik bulgusu denetim ucuna akmıyor
137 ihlal, 9.619.857 TL — `/api/denetim/anomali` bunu hiç görmüyor. Ayrıca kontrol satır
bazlı; tebliğ haddi **aynı gün aynı kişiyle işlemlerin toplamına** uygulanır (27 grup kaçıyor).

### B13. Kalemler fişe bağlı değil
Kalem tablosu okunuyor ama matraha toplanıp bırakılıyor. Stok kaydı (153 kalem bazlı) ve
KDV oran kırılımı satır düzeyinde çalışır.

### B14. Ölü kod: `ekstre.rs` + `mutabakat.rs`
410 satır; export ediliyor, hiçbir uçtan çağrılmıyor. Bakiye zinciri doğrulama, 3 katmanlı
dedup, ters işlem tespiti yazılmış ama bağlanmamış — B8/B9 tam da bunlara ihtiyaç duyuyor.

### B15. `/api/tahsis/defter` sayfalanamıyor
`offset` yok, max 1000; defterde 6.396 kayıt. Denetim izi ucu için ciddi eksik.

### B16. UI'da manuel eşleştirmeyi geri alma butonu yok
DELETE ucu artık doğru çalışıyor (bu oturumda düzeltildi) ama arayüzde çağıran yok.

---

## C. KANIT — iddia ettiğimiz ama ölçmediğimiz

### C1. Belge okuma isabeti ölçülmedi
Bugüne kadarki her rakam ya **belge-içi tutarlılık** ya **sentetik bozulma**.
Gerçek ölçüm için 30-50 gerçek belge + elle doğrulanmış beklenen değer gerekiyor.

### C2. Türkçe el yazısı örneği yok
6 el yazısı belge okundu, hiçbiri Türkçe değil. Ürünün birincil dilinde kanıt yok.

### C3. n=6 istatistik değil
Doğrulanabilir 6 belgede 0 hata var; üçler kuralıyla gerçek hata oranının %95 üst sınırı
hâlâ **~%40**. "Güvenilir" demek için yeterli değil.

### C4. Bozulma modeli varsayım
Düzeltici ölçümündeki (%92,2 kurtarma) bozulmalar benim ürettiğim model;
gerçek görme hatalarının dağılımı farklı olabilir.

---

## D. ALTYAPI

### D1. Kalıcılık yok — kullanıcı kararıyla ertelendi
Sunucu durumu tamamen bellekte. Kod değiştiğinde `manuel_tahsis` ve **tüm storno geçmişi
sıfırlanıyor** — VUK 219 iz kaydı kalıcı değil. Alım oturumları da bellekte.

**Not:** Öğrenilen şablonlar diske yazılıyor (`data/ogrenilen-sablonlar.json`, git dışı) —
yaşayan katmanın gereği. Ama defterin kendisi kalıcı değil.

### D2. `rapidocr-onnxruntime` bağımlılığı
Bu oturumda kuruldu (~200 MB: onnxruntime, opencv, numpy). Sistemin taranmış belge
okuması buna bağlı. Kaldırılması istenirse görüntü zinciri devre dışı kalır.

---

## Önerilen sıra

| # | İş | Neden önce |
|---|---|---|
| 1 | **A10** yakınsama sayacı | Diğer her şeyin işe yarayıp yaramadığı ancak bununla görülür |
| 2 | **A2** kural itibarı | Bugün zarar verdi; öğrenme büyüdükçe risk artar |
| 3 | **A1** Türkçe OCR | Sayılar korunuyor, kirli olan tek şey metin |
| 4 | **A5** UBL-TR XML | Günlük hacmin en büyük kalemi, hata payı sıfır yol |
| 5 | **B1-B3** fiş açıklama/tip/dayanak | VUK 219 izlenebilirlik — denetimde sorulur |
| 6 | **A4** denklem türleri | Değerleme raporlarının kapısı |
| 7 | **A11 + C1** gerçek regresyon seti | Ölçüm olmadan ilerleme iddia edilemez |

**A3, A6-A9, B4-B16** bunlardan sonra.
