# Okuma Motoru — Sorunlar ve Yol Haritası

> Her madde **kanıta** bağlı: ölçüm sayısı, denetim bulgusu ya da `dosya:satır`.
> Kanıtı olmayan madde bu listede yoktur.

---

## 0. Nerede duruyoruz — ve %100 ne demek DEĞİL

Son ölçüm (66 belge, yer gerçeği GİB UBL-TR örnek paketinden **kesin**):

| Zorluk | Sütun | Tam isabet | Alan isabeti |
|---|---|---|---|
| K1-KOLAY (dijital PDF) | 121/121 | 22/22 | %100 |
| K2-ORTA (200 dpi tarama) | 117/117 | 22/22 | %100 |
| K3-ZOR (110 dpi + eğik + gürültü) | 99/106 | 22/22 | %100 |

**Bu %100'ün SÖYLEMEDİKLERİ — hepsi gerçek sınırlar:**

1. **Örneklem 22 faturadan türetildi.** 66 belge var ama 22 kaynak XML'in 3 zorluk
   varyantı. Bağımsız belge sayısı 22.
2. **Bozulma benim ürettiğim.** Gerçek tarayıcı/telefon bozulması daha çeşitli: kırışık,
   kıvrık sayfa, parmak gölgesi, flaş yansıması, perspektif eğrilik. Hiçbiri yok.
3. **Belgeleri ben çizdim.** Gerçek matbaa faturası, entegratör çıktısı, termal fiş
   düzeni yok. Kendi çizdiğim düzeni okumak, kendi sınavımı kendim yazmaktır.
4. **K4 el yazısı katmanı BOŞ.** Tek bir el yazısı belge ölçülmedi.
5. **Yalnız FATURA.** Makbuz, dekont, ekstre, gider pusulası, irsaliye kategorileri boş.
6. **Alan isabeti "değer belgede geçiyor mu"yu ölçüyor**, "doğru alana yazıldı mı"yı değil.
   Daha zayıf bir ölçüt.

Yani: **motor kendi ürettiğim sınavda tam yapıyor.** Bu bir taban çizgisidir, bir başarı
beyanı değil.

---

## 1. ÖLÇÜM AÇIKLARI — önce bunlar, çünkü diğer her şeyi bunlar doğrular

### Ö1. Gerçek belge yok ⛔
Örneklem tamamen sentetik. **Gerçek bir SMMM'nin eline geçen belge hiç ölçülmedi.**

**Yapılacak:** 30-50 gerçek belge, anonimleştirilerek `data/ornek-belgeler/` altına;
her birine elle doğrulanmış `beklenen` değer. Bu, listedeki tüm diğer maddelerin
önceliğini de yeniden sıralayacaktır — şu an neyin gerçekten bozulduğunu bilmiyoruz.

> Kullanıcının kendi indirdiği faturalar bu boşluğu kapatabilir.

### Ö2. Kategori kapsamı %8
`belge-evreni.json` 57 belge türü sayıyor; ölçümde **yalnız FATURA** var.
MAKBUZ, DEKONT, EKSTRE, GIDER_PUSULASI dizinleri boş.

### Ö3. K4 el yazısı hiç yok
Elle doldurulmuş fatura/makbuz Türkiye'de hâlâ yaygın. Düzeltici katman
(`duzelt.rs`) el yazısı için tasarlandı ama **el yazısı belgede hiç ölçülmedi**.

### Ö4. Yakınsama ölçülmüyor
"Öğrenme birikiyor" hâlâ bir **tez**. Aynı şablonun 2., 3., 5. belgesinde isabet
artıyor mu — ölçen bir metrik yok. Kurulması gereken:
- **N'inci belge eğrisi** (aile içi sıraya göre isabet)
- **Aile şişme oranı** = aile sayısı / farklı VKN sayısı (ideal 1-3; 10+ ise parmak izi bozuk)
- **Dokunma oranı** — kaç alan insan eli değmeden geçti

---

## 2. MOTOR AÇIKLARI — ölçümle veya denetimle kanıtlanmış

### M1. Türkçe karakterler OCR'da kayboluyor ⛔
Kullandığımız RapidOCR sözlüğü (`ppocr_keys_v1.txt`) **Çince için**: 12 Türkçe
karakterden 2'sini içeriyor (yalnız ü, Ü). `ŞELALE → SELALE`, `ÇORAP → CORAP`.

`ppocrv5_latin_dict.txt` 12/12 kapsıyor (kendim doğruladım, 1.358 satır).

**Neden hâlâ açık:** model değişimi ~50 MB indirme gerektiriyor.
**Etkisi:** sayısal alanlar denklemle korunuyor, **metin korunmuyor** — cari ünvanı ve
mal adı bozuk gidiyor. `eslestir.rs` kapalı kümede bunu kurtarıyor ama yalnız cari
defterde kayıtlıysa.

### M2. Kural itibarı ölçülmüyor ⛔
`belge_ogren::uygula()` `isabet`/`deneme` alanlarını **hiç okumuyor**; puan sabit 200.
Tek yanlış onay doğru kuralı `*k = Kural{…}` ile **tamamen eziyor**.

**Yaşandı:** kirli çıkarım döneminde öğrenilmiş bir kural, kod düzeldikten sonra bile
en yüksek puanla kazandı; elle silmek zorunda kaldım.

**Teşhis düzeltmesi:** bu bir istatistik değil **köken (provenance)** problemi. Kod
sürümü değişince o sürümde toplanan kanıt geçersizleşir → epoch damgası gerekiyor.

### M3. Öğrenme çapası müşteri adını içeriyor
`ogren()` çapayı değerin bulunduğu satırdan üretiyor:
`"SAYIN: EGE TEKSTİL A.Ş."` → çapa `"sayin ege tekstil a s"`.
**Müşteri adı çapanın içinde**, dolayısıyla farklı müşterili bir sonraki faturada kural
hiç eşleşmiyor. Şablon öğrenmenin pratikte çalışmamasının başlıca sebebi bu olabilir.

### M4. Parmak izi OCR gürültüsüne dayanıksız
Aynı satıcının ikinci faturası yeni şablon sanılıyor → öğrenme birikmiyor.
Ö4 ölçülmeden bunun ne kadar zarar verdiği bilinmiyor.

### M5. Eksi işareti belirteçten düşüyor
`belge_ogren::belirtecler` rakam olmayan karakteri atlayarak başlıyor; `-` ve `(`
kayboluyor. Negatif bakiyeli ekstrede otomatik mod hiç çalışmıyor ve kullanıcı doğru
değeri elle girince **sahte çelişki uyarısı** çıkıyor.
MUTABAKAT'ta denklem `mutlak:true` olduğu için işaret kaybı + mutlak karşılaştırma
birleşiyor: 45.000/12.500 ile 12.500/45.000 aynı "doğrulandı" sonucunu veriyor.

### M6. `etiket_gecer` ölü kod
`grep` yalnız tanımı buluyor. Belgelenen "bozuk glif toleransı" **devrede değil**;
testi başka sebeple geçiyor. Eşanlamlısı olmayan alanlarda tek bozuk karakter etiketi
tamamen kaçırıyor, aday listesi boşalıyor ve `denklemle_sec` denklemi **hiç denemeden**
atlıyor.

### M7. Çok sayfalı taranmış PDF
Yalnız 1. sayfa OCR'lanıyor ve bu **kullanıcıya söylenmiyor**. 6 sayfalık ekstrede
zincir eksik veriyle kuruluyor, kullanıcı tamamının okunduğunu sanıyor.

### M8. Çok oranlı KDV
Bir faturada %1 + %10 + %20 birlikte olabilir. Tek `kdv` alanı ve tek denklem var;
böyle bir faturada denklem tutmuyor ve otomatik doldurma **hiç** yapılmıyor.
UBL-TR bunu `TaxSubtotal` ile modelliyor — bizim şemamızda karşılığı yok.

### M9. Denklem yalnız ARİTMETİK
Değerleme raporunda aritmetik özdeşlik yok. Gereken bağıntı türleri:
`ARALIK` · `TARIH_SIRASI` · `ZORUNLU_BEYAN` · **`DEFTER_KARSILASTIR`**.
Sonuncusu en güçlü kozumuz: defter zaten elimizde, belge dışından bağımsız tanık.

### M10. UBL-TR XML yolu bağlı değil
e-Fatura'nın XML'i varsa **OCR'a hiç gerek yok** — birebir okunur, hata payı sıfır.
Günlük hacmin en büyük kalemi ve bağlanmamış. GİB örnek paketleri artık elimizde
(`data/ornek-belgeler/_ham/`), ayrıştırıcı `scripts/ornek-uret.py` içinde **zaten var**
— yalnız motora taşınacak.

---

## 3. ARAYÜZ AÇIKLARI — kullanıcının bildirdiği

### U1. Okunan değer yerinde düzeltilemiyor
Motor `"HaticeAYDIN IrsalyeTanhi:10.02.2006"` okuduğunda kullanıcı bunu kırpıp
"sadece şu kısım doğru" diyemiyor.
**Ayrıca:** düzeltme öğrenmeye de gitmiyor — `ogren()` değeri belgede `contains()` ile
arıyor, kullanıcının düzelttiği metin belgede aynen geçmediği için eşleşme tutmuyor ve
**en değerli düzeltme çöpe gidiyor**.

### U2. Belgeye özgü alan eklenemiyor
Belgede olup şablonda olmayan alanı (Seri, Sıra, İrsaliye No, Vergi Dairesi) kullanıcı
ekleyemiyor.

### U3. Otomatik onaylanan alana geri dönülemiyor
Otomatik mod denklem alanlarını `onaylandi=true` yapıyor; arayüz ilk onaysız alana
gidiyor. Yanlış otomatik onaylanmış bir değere **bir daha uğranmıyor**.

### U4. Güven sinyali sınırda düşüyor
`aday_sayisi` arka uçta üretiliyor, arayüz tipinde yok. `Aday.gerekce`, `Kural.isabet`
hiç ekrana gelmiyor. Kullanıcı bir değerin nereden geldiğini göremiyor.

### U5. Kalem tablosu düzenlenemiyor
Salt okunur. `matrahla_tutuyor=false` çıkarsa kullanıcının yapabileceği tek şey baştan
başlamak.

---

## 4. SORUN OLMAYAN — üzerine gitmeyelim

Ölçüldü ve iyi çalışıyor; buraya yatırım yapmak israf olur:

- **Sütun konumu tespiti** (bant ayrımı + koridor) — 3 katmanda da başlık bulunamayan
  belge yok.
- **Sütun adlandırma** — başlık bulunduğunda hiç yanlış eşleşme yok (121/121, 117/117).
- **Dil bilgisi katmanı** — çekimli biçimler kuralla çözülüyor; sözlükten 7 elle yazılmış
  çekim silindi, testler geçmeye devam etti.
- **Ölçek çapası** — ondalık ayraç hatası (100 kat) kapatıldı ve regresyon testi var.
- **Sözlük bütünlüğü** — 6 test, sessiz çürümeyi engelliyor.

---

## 5. SIRA

| # | İş | Neden bu sırada | Maliyet |
|---|---|---|---|
| 1 | **Ö1 gerçek belge örneklemi** | Diğer her maddenin önceliğini bu belirler; şu an neyin bozulduğunu bilmiyoruz | ½ gün + kullanıcı belgeleri |
| 2 | **M10 UBL-TR XML yolu** | Günlük hacmin en büyüğü, hata payı SIFIR, ayrıştırıcı zaten yazılmış | ½ gün |
| 3 | **Ö4 yakınsama sayacı** | Öğrenme iddiasının tek kanıtı; M2/M3/M4'ün etkisi ancak bununla görülür | 1 gün |
| 4 | **M3 çapa düzeltmesi** | Öğrenmenin çalışmama sebebi büyük ihtimalle bu, ve ucuz | ½ gün |
| 5 | **M2 kural itibarı** | Yaşanmış zarar; öğrenme büyüdükçe risk artar | 1 gün |
| 6 | **U1 yerinde düzeltme + öğrenmeye besleme** | En değerli kullanıcı girdisi şu an çöpe gidiyor | 2 gün |
| 7 | **M1 Türkçe OCR sözlüğü** | Metin kalitesi; ~50 MB indirme onayı bekliyor | ½ gün |
| 8 | **M5 eksi işareti** | Ekstre/mutabakat yolunu açar | 2 saat |
| 9 | **M7 çok sayfa** | Ekstrede sessiz veri kaybı | ½ gün |
| 10 | **M8 çok oranlı KDV** | UBL-TR `TaxSubtotal` şeması hazır | 1 gün |

**M6, M9, U2-U5** bunlardan sonra.
