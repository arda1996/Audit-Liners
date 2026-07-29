# Çalışma Kağıdı Hesap Motoru — Tasarım (Excel-esnek + kuralcı)

> Kaynak: 8 ajanlık araştırma workflow'u (2026-07-12) — mevcut kod analizi, BDS 230 metodolojisi,
> OSS motor taraması, Excel hesaplama bilimi, sürükle-bırak etkileşim modeli, UFRS worksheet pratiği.
> Kardeş doküman: [ufrs-worksheet.md](ufrs-worksheet.md) — bu motorun ilk büyük tüketicisi.

## 0. Problem ve hedef

SMMM/denetçi bugün çalışma kağıdı hesaplarını Excel'de yapıyor. Excel'in gücü: grid + formül +
serbest yerleşim. Excel'in denetimde kabul edilemez yanları: **kaynaksız rakam, kopan referans
(#BAŞV!), izsiz değişiklik, float para, izsiz koruma**. Hedef: Excel'in gücünü alıp kusurlarını
tip sisteminde ve kural katmanında **yapısal olarak imkânsız** kılmak.

Mevcut kodun durumu (acımasız envanter — ajan raporu, dosya:satır'lı):
- Hücre modeli yok: `"kod:alan"` string-anahtarlı override haritası (main.rs:2271, App.tsx:346)
- Formül dili yok: tek "formüller" TSX'e gömülü `borc-alacak` ve `reduce` (App.tsx:1656,1686)
- Bağımlılık grafı yok; kağıtlar arası referans yok; sunucu hesaplamadan habersiz
- `duzenle` ucu doğrulamasız JSON blob'u **izsiz** üzerine yazıyor; oturum bile istemez (main.rs:2464)
- domain'de kağıt kavramı hiç yok; M1/M7 motorları handler içine gömülü (main.rs:2346-2400)

## 1. Temel mimari kararlar

### K1 — Referans modeli: koordinat DEĞİL, kimlik + rol tabanlı
Excel'in 30 yıllık evrimi yönü gösteriyor: A1 → adlandırılmış aralık → `Tablo[Kolon]` → spill `A1#`.
Konum-tabanlı adres frajil; kimlik-tabanlı taşınmaya dayanıklı (Grist/Airtable/Notion kanıtı).
- Her blok/kolon/adlandırılmış değer **ULID** taşır; formül AST'si ID saklar, editör **ad gösterir**.
- Satır-içi formül `[@Borç] - [@Alacak]` (satır-göreli rol); tek değerler `Önemlilik`, `KV_Orani` gibi ad.
- Ham `$A$1` sözdizimi HİÇ sunulmaz. Sürükle-bırak salt yerleşim işlemi olur; hesap grafı değişmez;
  **#BAŞV! sınıfı hata tasarım gereği yok**. Silme anında ters-kenarlar bilinir → "3 formül bağlı,
  silinemez" önleyici ret (Excel'in 'sil, sonra #REF! göm' davranışı yasak).

### K2 — Değer tipleri: para floata değmez (motor çekirdeğinde)
```
Deger = Para(Kurus/i64)             // toplama-çıkarma kayıpsız
      | Oran(i64, ölçek 10⁻⁶)      // yüzde/katsayı/endeks — para tipinden ayrı
      | Sayi(i64) | Metin | Tarih
      | Hata { tur, kaynak_id }     // yayılırken KÖKENİNİ taşır
```
- Çarpma/bölme yalnız **açık yuvarlama politikasıyla** derlenir: `carp_yuvarla(oran, YarimYukari)` —
  yuvarlamasız para çarpımı tip sisteminde yok ("penny problem"in ilacı).
- `dagit(tutar, oranlar) -> Vec<Kurus>` **en-büyük-kalan** primitifi: parçaların toplamı DAİMA ana
  tutara eşit (invariant testli). Excel'de kullanıcı hilesi olan şey bizde motor primitifi.
- i64 kuruş 2^53'e kadar (~90 trilyon TL) f64'te kayıpsız — ama motor içi para asla f64'e inmez.

### K3 — Yeniden hesap: dirty-marking + gerçek topolojik sıra
Excel modeli: bağımlılık ağacı → calculation chain → yalnız kirli hücreler. Bizim ölçek (10³–10⁵
hücre) için Excel'in "kendini optimize eden zincir" karmaşası gereksiz: değişen düğümün ileri
erişilebilirlik kümesini kirlet, **Kahn topolojik sırası** ile yalnız kirlileri hesapla. Deterministik.
- **Uçucu fonksiyon YASAK** (NOW/RAND/OFFSET/INDIRECT sınıfı) — determinizmi ve statik grafı bozar.
  Tarih gerekiyorsa girildiği anda değere donan eylem (izde "sistem tarihi eklendi").
- **Döngü: ekleme anında DFS ile KESİN RET** — iteratif mod denetlenemez (yakınsama garantisiz).
  Meşru ihtiyaç (faiz kapitalizasyonu) kapalı-form özel fonksiyonla: `FAIZ_KAPITALIZE(...)`.
- Hesap zinciri kanıt olarak saklanır: "bu değer şu sırayla şu girdilerden" — Excel'in yapamadığı
  hesap izlenebilirliği (BDS 230 ruhu).
- Golden-test rejimi (McCullough dersi): her fonksiyon referans vakalarla CI'da; aynı girdi →
  bit-eşit çıktı. i64 toplama birleşmeli olduğundan paralellik float'tan güvenli.

### K4 — Blok modeli: tipli bölge + blok-içi ızgara (serbest A1 yok)
Kağıt = sıralı **tipli blok** listesi (Notion içerik-dizisi modeli; taşıma = dizide ID kaydırma):

| Blok tipi | İçerik | Koruma |
|---|---|---|
| `kimlik` | müşteri, dönem, endeks, hazırlayan/inceleyen+tarih | sistem yazar, kilitli |
| `kaynak-veri` | defterden/mizandan sorgu (canlı) | **daima salt-okunur** |
| `hesap-şeridi` | korumalı formüller (şablondan) | yalnız şablon yetkisi değiştirir |
| `test-tablosu` | denetçi girdi kolonları + formül kolonları karışık | girdi=izli, formül=korumalı |
| `manuel-müdahale` | serbest satırlar | serbest ama **gerekçe zorunlu + izli** |
| `sonuç-kanaat` | zorunlu sonuç metni + tickmark lejantı | boşken kağıt kapanamaz |
| `imza` | hazırlayan/inceleyen imza zinciri | imza sonrası kağıt kilitli |

Bir blok = dinamik dizi (spill): "1 formül = 1 çıktı bölgesi"; kaynak büyüyünce genişler; çakışma
açık hata (sessiz üzerine yazma yok). Bloğa referans `Blok#` bütün-bölge biçiminde.

### K5 — Koruma: Excel'in tersi — varsayılan kilitli, istisna tanımlı ve İZLİ
Üç kademe: **kilitli** (API reddeder) / **izli** (girilir; kim+ne zaman+eski→yeni+gerekçe olayı) /
**serbest** (not alanları). Grist dersi: **S-izni ayrı** — veri girme ≠ yapı/formül değiştirme.
Override, değerin üzerine yazmak değil ayrı katman: `hesaplanan + override(kim,zaman,eski→yeni,gerekçe)`;
görünürde override kazanır, fark daima raporlanabilir. Formül gizleme yok — tersine her hesaplanan
değerin formülü+girdileri tek tıkla açılır (kalıcı trace precedents).

### K6 — Olay-günlüklü komutlar; undo = ters-olay EKLEMEK
Her eylem Rust API'de tanımlı Op (`BlokTasi{blok_id, hedef}`, `HucreYaz{hucre_id, eski, yeni}`).
Append-only olay günlüğü = denetim izi = undo yığını (silme yok; geri alma da izde görünür).
Kural katmanı yürütücünün ÖNÜNDE: kilitli alana yazan op yığına giremeden reddedilir.
Kesinleşen kağıtta değişiklik yalnız "iptal + yeni sürüm" (VUK 217 felsefesinin kağıt karşılığı).

## 2. Teknoloji seçimleri (lisans-doğrulamalı kısa liste)

| Katman | Seçim | Lisans | Gerekçe |
|---|---|---|---|
| **Formül motoru** | **İnce öz motor** (crates/domain/kagit) | bizim | Hiçbir hazır motor kuruş-i64 değil (hepsi f64); kuralcı katman (kilit/iz/blok) hiçbirinde yok — zaten yazacağımız kısım orası. Gerçek ihtiyaç ~40-60 fonksiyon, 400 değil |
| Parser tabanı | formualizer-parse (MIT/Apache) devşir VEYA kendi Pratt parser | MIT | parser sıkıcı %30; MIT fork güvenli |
| Referans motor | IronCalc (Rust, MIT/Apache, ~4k★, aktif) | MIT/Apache-2.0 ✅doğrulandı | fonksiyon semantiği + xlsx round-trip referansı; 1 haftalık spike ile i64↔f64 köprüsü test edilebilir (fallback planı). Bakım: 2 çekirdek geliştirici (Nicolás Hatcher + Daniel Gonzalez Albo) — bus-factor riski hafif ama var |
| Semantik referans | Apache POI test takımı, formula.js, LibreOffice broadcast mimarisi | — | kod alınmaz, davranış alınır |
| **Grid UI** | **Glide Data Grid** (canvas, React, MIT) | MIT | motor backend'de → grid "aptal ve hızlı" olmalı; kuralları grid'e biz dayatırız (Handsontable ücretli, AG Grid spreadsheet özellikleri Enterprise, HyperFormula GPLv3 → elendi) |
| **Sürükle-bırak** | **@atlaskit/pragmatic-drag-and-drop** | Apache-2.0 | Atlassian bakımı; sanal grid dostu; **eylem menüsü deseni** (⋮ → Yukarı taşı / Bölüme taşı) = erişilebilir + her taşıma loglanabilir komut (dnd-kit bakımı belirsizleşti) |
| xlsx içe | calamine | MIT | SMMM'nin mevcut Excel kağıtlarını içe alma |
| xlsx dışa | rust_xlsxwriter | MIT/Apache | kağıt/rapor dışa aktarımı |

**Lisans ilkesi (Quadratic dersi):** Mart 2026'da Quadratic kapalı kaynağa döndü, repo silindi
(zaten hiç OSI-açık değildi — kendi "source available" lisansıydı); Handsontable 2019'da açık
kaynağı bıraktı ($999-1.299/gel/yıl — doğrulandı). Ürün kalbine giren her bağımlılık
**fork-edilebilir permissive (MIT/Apache)** olmalı; GPL/source-available kabul edilemez.

**Adversarial lisans doğrulaması sonuçları (ayrı ajan, kaynak URL'li):** HyperFormula GPLv3+ticari
✅teyit; AG Grid'de range selection/pano/Excel export/pivot Enterprise ✅teyit (resmi tablo);
IronCalc MIT/Apache + 300+ fonksiyon + aktiflik ✅teyit. **Univer B-planı ZAYIFLADI:** çekirdek
Apache-2.0 doğru AMA xlsx içe/dışa aktarma, yazdırma, sunucu tarafı hesap ve "Advanced Formula
Engine" da Pro'da (ticari) — SMMM ürününde xlsx vazgeçilmez olduğundan Univer ancak kendi Rust
dosya hattımızla (calamine+rust_xlsxwriter) köprülenirse düşünülebilir. Glide Data Grid önerisi
rakip lisans engelleri doğrulanınca güçlendi. IronCalc'ın NGI fon iddiası teyit edilemedi
(karar girdisi yapılmayacak).

## 3. Kuralcı katman — programlaşan 35 kural (BDS 230 + BDY + ticari yazılım desenleri)

Araştırmanın tam listesi altı grupta (A kimlik/üst veri · B durum makinesi/kilit · C hücre/formül ·
D TB/lead/yevmiye · E parametreler · F iz/inceleme). Kritik olanlar:

- **A:** hazırlayan+tarih sistem atar (elle değişmez); gözden geçiren ≠ hazırlayan (SoD);
  amaç/kaynak/yapılan iş/**sonuç** zorunlu — sonuç boşken kağıt kapanamaz; her ek `kaynak` etiketli
  {PBC, denetçi, üçüncü taraf, sistem}.
- **B:** yaşam döngüsü `taslak → hazırlandı → incelendi → onaylandı → kilitli`; imza sonrası içerik
  değişirse **imza otomatik düşer**; rapor tarihinden sonra 60 gün assembly sayacı (BDS 230);
  kilitli dosyada silme yok, yalnız izli ek; saklama **10 yıl** (BDY — parametrik ama altına inilemez).
- **C:** üç hücre tipi (sistem/formül/girdi) görsel olarak ayırt edilir; tickmark tanımlı lejanttan
  seçilir (serbest işaret yok), kim/ne zaman taşır; çapraz referans tiplenmiş bağ — kaynak değişince
  hedef "güncel değil" uyarısı; kağıtlar arası değer taşıma YALNIZ referansla.
- **D:** tek gerçek kaynak mizan; mapping'den lead otomatik; eşlenmemiş hesap varken "tamam" yok;
  AJE/RJE merkezi günlükte, doğduğu kağıda çapraz referanslı, {önerildi/kabul/vazgeçildi} durumlu;
  **deftere asla otomatik işlenmez**; vazgeçilenler düzeltilmemiş yanlışlıklar özetine (SUM) akar;
  Σ(lead) = mali tablo kalemi denkliği bozuksa dosya kapanamaz; roll-forward: onaylı finaller → PY sütunu.
- **E:** önemlilik kağıdı kök parametre (OM/PM/CTT diğer kağıtlara referansla akar; revizede türeyen
  kağıtlar "yeniden değerlendir" bayrağı alır); örneklemede seed+yöntem+ID listesi (yeniden üretilebilir);
  analitikte |fark|>eşik iken açıklamasız sonuç yazılamaz; varsayım blokları sürümlü (eski silinmez).
- **F:** her hücre değişikliği append-only günlüğe; açık inceleme notu varken dosya kapanamaz;
  kapanış kontrol listesi motor üretir (imzasız kağıt, kopuk referans, bekleyen AJE… sıfırlanmadan bitmez).

## 4. Uygulama fazları

| Faz | İçerik | Çıktı |
|---|---|---|
| **F0** | domain crate `kagit` modülü: Deger tipleri, blok/hücre modeli, Op+olay günlüğü, koruma kademesi | test edilebilir çekirdek (mevcut M1/M7 motorları handler'dan buraya taşınır) |
| **F1** | formül dili: parser (TR fonksiyon adları + EN alias), AST(ID-tabanlı), bağımlılık grafı, Kahn recalc, ~40 fonksiyon + `dagit`/`carp_yuvarla` | golden-test'li motor |
| **F2** | API: kağıt CRUD yeniden — oturum+yetki (Y3 katmanına bağlanır!), izli düzenleme, şablon/örnek sürümleme; mevcut `duzenle` ucunun izsiz-blob modeli emekli edilir | uçlar + KULLANIM-KLAVUZU güncellenir |
| **F3** | Grid UI: Glide Data Grid + pragmatic-dnd + eylem menüsü; blok render; üç hücre tipinin görsel dili; override rozetli gösterim | çalışma kağıdı sayfası v2 |
| **F4** | **UFRS WorkSheet kataloğu** bu motorun üstünde ([ufrs-worksheet.md](ufrs-worksheet.md)) | WS-MAP→WS-EQ hattı |
| **F5** | xlsx içe/dışa (calamine + rust_xlsxwriter), dipnot yayın alanları, kapanış kontrol listesi | dosya seviyesi bütünlük |

**Spike (F1 öncesi, ~1 hafta):** IronCalc'ı crate olarak bağla; (i) i64→f64→i64 round-trip mizan
toplamlarında bit-eşit mi, (ii) artımlı recalc granülerliği, (iii) kilit katmanı dışarıdan sarılıyor mu.
Sonuç olumluysa F1'de "öz motor"un fonksiyon değerlendirmesi IronCalc'a delegasyonla hızlanabilir;
olumsuzsa tam öz motor (plan değişmez, süre değişir).

## 5. Tamlık kritiğinin kapattığı boşluklar (bağımsız eleştirmen ajanın bulguları → kararlar)

**5.1 Kalıcılık + değişmezlik köprüsü (motor seçiminden önce gelir).** BDS 230 "kilit sonrası yalnız
izli ek" + BDY 10 yıl ↔ bugün her şey süreç ölünce kaybolan HashMap'te. Karar: F0'da olay günlüğü
**append-only event store** olarak tasarlanır (K6 zaten bunu kuruyor); kalıcılığa (#12 görev,
DB-per-mükellef) geçişte bu günlük tablo olarak birebir taşınır; dosya kapanışında (assembly)
günlüğün **hash zinciri** mühürlenir — "kilitli dosya değişmedi" iddiası kriptografik olarak
gösterilebilir. Undo, iz ile çelişmez: undo = ters-olay eklemek (K6), silme yok.

**5.2 Defter-provenance: formül dili defter verisine birinci sınıf bağlanır.** ISA 230 "test edilen
kalemlerin ayırt edici özellikleri" şartı hücre→hücre referansıyla karşılanamaz. Karar: domain
fonksiyonları formül dilinin çekirdeğinde — `MIZAN("600";"borç")`, `KEBIR("102.01")`,
`FIS(19542)`, `MUAVIN("120";yaş>90)`. Bunların bağımlılığı "defter dönem X" düğümüne bağlanır:
deftere yeni fiş kesinleşince o dönemin abonesi hücreler kirlenir (defter→graf invalidation).
Test satırları fiş/satır ID listesi taşır (örneklem yeniden bulunabilir). Bulgular serbest metin
değil yapılandırılmış nesne: `{motor, fis_id[], tutar, esik, karar}`.

**5.3 Türkçe formül yerelleştirmesi — parser seviyesinde, sonradan eklenemez.** SMMM `ETOPLA`,
`DÜŞEYARA` yazar; argüman ayracı `;`, ondalık ayracı `,`. Karar: parser TR-birincil tasarlanır —
argüman ayracı `;`, ondalık `,` (binlik `.`); fonksiyon tablosu çift adlı (ETOPLA=SUMIF,
EĞER=IF, DÜŞEYARA=VLOOKUP…) iki ad da kabul, gösterim kullanıcı tercihli. `1,5` sayı mı iki
argüman mı belirsizliği `;` ayracı sayesinde yok. IronCalc locale desteğinin TR kapsamı spike'ta
doğrulanacak (yoksa kendi lexer'ımız zaten TR-birincil).

**5.4 Tip semantiği tablosu (denetimde savunulabilirliğin özü).** Formül düzeyinde tip kuralları:
`Para+Para=Para` · `Para−Para=Para` · `Para×Oran=Para(yuvarlama politikası ZORUNLU parametre)` ·
`Para÷Para=Oran` · `Para÷Sayı=Para(yuvarlama+kalan raporu)` · `ORTALAMA(Para…)=Para(politikalı)` ·
`Oran×Oran=Oran` · `Para×Para=DERLEME HATASI` (anlamsız) · `Para+Oran=DERLEME HATASI`.
KDV/stopaj gibi vergi hesapları `carp_yuvarla(oran, YarimYukari)` standardında; farklı politika
isteyen kağıt bunu parametre bloğunda açıkça beyan eder (izli). Bu tablo F1'de tip denetleyicisine
birebir kodlanır; i64↔f64 köprüsü mü fork mu kararı bu tablonun IronCalc'ta ifade edilebilirliğine bağlı.

**5.5 xlsx stratejisi = ürün kararı.** İçe: müşteri mizan/muavin xlsx'i WS-MAP eşleme sihirbazından
geçer (calamine). Dışa: **değer + formül metni + iz manifesti** birlikte — kağıdın xlsx çıktısında
ayrı bir "İz" sayfası (hazırlayan/inceleyen, override listesi, olay özeti, hash) bulunur; böylece
Excel'e çıkış delil zincirini koparmaz, KGK incelemesine teslim formatı da budur. Excel'e çıkan
kopya "türev kopya" damgası taşır — otorite her zaman programdaki kağıttır.

**5.6 Performans bütçesi + hesabın yeri.** Hedefler: 10k hücreli kağıtta artımlı recalc **<50ms**,
tam recalc **<500ms**; 100k muavin satırı taraması (domain fonksiyonu) **<1s**; tuş-başına UI
gecikmesi **<16ms** (hesap async, optimistic görüntüleme). Hesabın yeri: **otorite backend (axum,
Tauri'de aynı süreç)** — tek doğruluk kaynağı; WASM-frontend ikilemesi ancak ölçüm gecikme sorunu
gösterirse ve yalnız önizleme (authoritative olmayan) olarak eklenir. Bu hedefler F1 benchmark
suite'ine yazılır (criterion).

**5.7 Test/doğrulama stratejisi.** (a) **Oracle testleri**: aynı formül seti LibreOffice/Excel'de
hesaplanır, sonuçlar golden dosyada — motor bit-eşit (para) / ε-eşit (oran) tutmalı; (b) **gerçek
kağıt korpusu**: fixture defterden üretilen amortisman/yaşlandırma/önemlilik kağıtları golden;
(c) **property-based** (proptest): `dagit()` toplam-koruma, i64 taşma sınırları, yuvarlama
birleşme; (d) **fuzz**: formül parser'ına (cargo-fuzz); (e) determinizm: paralel recalc sıra
bağımsızlığı. "Motor doğru hesaplıyor" iddiası kanıtsızsa BDS 230 anlatısı çöker — CI kapısı.

*(Kritiğin 8. bulgusu — TMS 29 çift endeksleme, TMS 12 otomatik türetme, TFRS 16/TMS 19 motorları —
[ufrs-worksheet.md](ufrs-worksheet.md)'de zaten tasarlandı; eleştirmen kırpılmış özet gördüğü için
göremedi. Yine de oradaki §3.1'e VUK↔TMS29 fark mutabakat tablosu netliği eklendi.)*

## 6. Excel'den alınan / bilinçle reddedilen

**Alınan:** dirty-marking artımlı topolojik recalc · kopyala-vs-taşı ikili semantiği (taşıma kimlik
korur, kopya deseni çoğaltır) · spill/blok modeli · iki katmanlı koruma fikri · calculation chain'i
kalıcılaştırma.
**Reddedilen:** double ile para · uçucu fonksiyonlar · iteratif döngü · koordinat-tabanlı frajil
referans · izsiz koruma · izsiz override · formül gizleme (Hidden) · "precision as displayed".
