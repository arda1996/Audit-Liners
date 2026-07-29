# Değerleme Veri Mimarisi — Kanıt Hiyerarşisi ve Güven Skoru Doktrini

> **Statü:** Düşünce çerçevesi + uzun vadeli yol haritası. KOD YOK — tartışılacak, geliştirilecek, sonra
> parça parça inşa edilecek. Kullanıcı ilkesi: "her noktayı birlikte yapmak."
> **Tarih:** 2026-07-14 · **Kardeş:** [ufrs-worksheet.md](ufrs-worksheet.md), [dis-veri-katmani.md](dis-veri-katmani.md)

---

## 0. Problemi bir cümlede kurmak

Bir değerleme, **dayandığı verinin kalitesi kadar** geçerlidir. Bu yüzden programın görevi "bir sayı
üretmek" değil, **o sayının arkasındaki kanıt zincirini taşınabilir ve sorgulanabilir kılmaktır.**

Bunu neden şimdi konuşuyoruz: **enflasyon muhasebesi uygulanmıyor.** Yani tarihi maliyet, 2019'da
150.000 TL'ye alınan depo binasını hâlâ 150.000 TL gösteriyor. Bu rakam *doğru* (VUK açısından) ama
*gerçek değil*. Bağımsız denetimin işi tam olarak bu boşluğu kapatmak: **gerçeğe uygun değeri
piyasadaki ekonomik aktörlerin fiyatlamasına dayandırmak.** Enflasyon düzeltmesi yokken bu, seçenek
değil zorunluluk hâline geliyor.

---

## 1. Kavramsal omurga: TFRS 13 zaten bir güven hiyerarşisidir

Senin anlattığın "kaynağa göre güven derecelendirmesi" fikri, aslında standartta hazır duruyor —
biz onu **işletilebilir hâle getireceğiz**. [TFRS 13](https://www.kgk.gov.tr/Portalv2Uploads/files/Duyurular/v2/TFRS/TFRS_2020/TFRS%2013.pdf)
girdileri üçe ayırır ve şu emri verir: *"gözlemlenebilir girdilerin kullanımını azami seviyeye çıkar,
gözlemlenebilir olmayanları asgariye indir."*

| Seviye | Ne demek | Bizim dünyamızda karşılığı |
|---|---|---|
| **Seviye 1** | Aktif piyasada, aynı varlık için kote fiyat | TCMB döviz kuru · BIST hisse fiyatı · devlet tahvili getirisi |
| **Seviye 2** | Gözlemlenebilir ama dolaylı: benzer varlık, emsal işlem, endeks | Emsal daire satışı · TCMB Konut Fiyat Endeksi · ikinci el araç ilan ortalaması |
| **Seviye 3** | Gözlemlenemeyen girdi: işletmenin kendi varsayımı | DCF projeksiyonu · kıdem devir hızı · ECL temerrüt oranı |

**Kritik içgörü:** Seviye 3'e düşmek yasak değil — ama **bedeli vardır**: daha fazla dipnot açıklaması,
duyarlılık analizi zorunluluğu, ve denetçi için daha yüksek kanıt yükü ([BDS 540](https://www.kgk.gov.tr/Portalv2Uploads/files/Duyurular/v2/BDS/BDS%20540(1).pdf)).
Programımız bu bedeli **otomatik hesaplamalı ve denetçiye göstermeli.**

---

## 2. Benim eklediğim katman: Kanıt Güven Skoru (KGS)

TFRS 13 seviyeyi söyler ama **"ne kadar güvenilir"** sorusuna sayı vermez. Senin "güven aralığı
kümülatif düşmeli" fikrin tam burayı dolduruyor. Önerdiğim model — **dört çarpan**:

```
KGS = Kaynak Otoritesi × Veri Yeterliliği × Zaman Tazeliği × Emsal Yakınlığı
      (0–1)             (0–1)              (0–1)            (0–1)
```

**Neden çarpım, toplam değil?** Çünkü zincir en zayıf halkası kadar sağlamdır — ama tek bir zayıflık
her şeyi sıfırlamamalı. Çarpım her ikisini de sağlar: bir faktör 0,5'e düşerse skor yarılanır, ama
diğer güçlü faktörler hâlâ katkı verir. **Toplasaydık** zayıf kanıtı güçlü kanıtla "telafi etmiş"
olurduk — bu denetimde kabul edilemez.

### 2.1 Kaynak Otoritesi — kim söylüyor?

| Katman | Örnek | Otorite |
|---|---|---|
| **A. Resmî kurum** | TCMB (kur, KFE, reeskont oranı), TÜİK (TÜFE/Yİ-ÜFE), Hazine (tahvil getirisi) | 1,00 |
| **B. Lisanslı/denetlenen kuruluş** | [SPK lisanslı değerleme şirketi](https://spk.gov.tr/kurumlar/gayrimenkul-degerleme-kuruluslari) raporu, BDDK yetkili kuruluş | 0,85–0,95 |
| **C. Kabul görmüş akademik/uluslararası** | [Damodaran ülke risk primi](https://pages.stern.nyu.edu/~adamodar/New_Home_Page/datafile/ctryprem.html) (TR ERP %9,30 · Tem 2026), Big-4 WACC çalışmaları | 0,75–0,85 |
| **D. Sektörel/ticari veri sağlayıcı** | Endeksa, sektör dernekleri, fiyat endeksi yayınlayan platformlar | 0,55–0,75 |
| **E. Piyasa ilanı (asking price)** | Emlak/araç ilan siteleri — **satış değil, talep fiyatı** | 0,35–0,55 |
| **F. İşletme yönetiminin beyanı** | Yönetim tahmini, iç projeksiyon | 0,30–0,50 |

**Tartışmaya açtığım nokta:** E katmanı (ilan siteleri) kritik bir tuzak barındırıyor. **İlan fiyatı ≠
satış fiyatı.** Türkiye'de emlak ilan fiyatı ile gerçekleşen satış arasında tipik olarak pazarlık payı
vardır. Yani ilan verisini ham kullanmak **sistematik olarak yukarı yanlı** bir değerleme üretir —
bu, denetimde "varlıkların şişirilmesi" anlamına gelir ve **denetçiyi yanlış tarafa hataya iter.**
Çözüm önerim: ilan verisi kullanılacaksa **düzeltme katsayısı** (haircut) uygulanmalı ve bu katsayının
kendisi de dayanaklı olmalı (ör. TCMB KFE ile ilan ortalamasının tarihsel farkından kalibre edilmeli).
**Bunu senle tartışmak istiyorum — pazarlık payını nasıl kalibre ederiz?**

### 2.2 Veri Yeterliliği — kaç gözlem?

Tek bir emsal, emsal değildir. Önerim:
- n ≥ 10 benzer gözlem → 1,00
- n = 5–9 → 0,80
- n = 3–4 → 0,60
- n = 1–2 → 0,35 · **ve sistem "yetersiz veri" bayrağı basar**
- n = 0 → Seviye 3'e düş (yönetim varsayımı), duyarlılık analizi **zorunlu**

Ayrıca **dağılımın kendisi** bilgi taşır: gözlemler çok dağınıksa (yüksek varyans) güven düşmeli.
Standart sapma / ortalama (varyasyon katsayısı) > %25 ise skoru cezalandırmayı öneriyorum.

### 2.3 Zaman Tazeliği — ne zaman ölçüldü?

Yüksek enflasyon ortamında **veri hızla bayatlar**. 6 ay önceki emsal satış, bugünün gerçeği değil.
- ≤ 1 ay → 1,00 · ≤ 3 ay → 0,90 · ≤ 6 ay → 0,75 · ≤ 12 ay → 0,55 · > 12 ay → 0,30
- **Ama:** eski veri **endeksle güncellenebilir** (TCMB KFE ile taşınır) → tazelik cezası hafifler,
  bunun yerine "endeksleme belirsizliği" küçük bir ceza olarak eklenir.

### 2.4 Emsal Yakınlığı — ne kadar benzer?

Depo binası için emsal: aynı ilçede mi, aynı m² bandında mı, aynı yaşta mı, aynı yapı sınıfında mı?
Her uyuşmayan boyut skoru düşürür. **Bu, TFRS 13'ün "Seviye 2 → Seviye 3'e kayma" mekanizmasıdır:**
düzeltme (adjustment) ne kadar büyükse, girdi o kadar gözlemlenemez hâle gelir.

---

## 3. Çapraz Doğrulama Zorunluluğu — "tek kaynağa güvenme"

Senin en güçlü sezgin bu: *"resmi kurumların değerlerinin gerçekçiliği sorgulanmalı, hâlâ soru işaretiyse
piyasadaki reel aktörlerin fiyatlaması analiz edilmeli."*

Bunu bir **kural** hâline getiriyorum:

> **Önemli bir değerlemede en az iki bağımsız kaynak karşılaştırılmalı.** Kaynaklar arası sapma
> **%15'i aşarsa** sistem "çelişki bulgusu" üretir ve denetçiden **gerekçe ister**: hangisini neden seçti?

Neden bu kadar önemli: **resmî veri de yanlı olabilir.** Örnek — TCMB Konut Fiyat Endeksi, *kredi
kullanılan* konutların değerleme raporlarından hesaplanır. Yani nakit satışları, lüks segmenti ve
kredi kullanılmayan işlemleri **eksik temsil eder**. Bu, resmi kaynağın bile bir **örneklem yanlılığı**
taşıdığı anlamına gelir. Denetçi bunu bilmeli ve raporlayabilmeli.

**Çelişki raporu** şunu içermeli: her kaynağın değeri, KGS'si, sapmanın büyüklüğü, ve denetçinin
tercih gerekçesi. Bu, [BDS 540](https://www.kgk.gov.tr/Portalv2Uploads/files/Duyurular/v2/BDS/BDS%20540(1).pdf)'ın
"yönetimin varsayımlarını değerlendir" emrinin somut karşılığıdır.

---

## 4. Güven düşünce ne olur? (Sonuç zinciri)

KGS sadece bir rozet değil — **denetim davranışını değiştirmeli**:

| KGS | Anlamı | Sistemin zorladığı davranış |
|---|---|---|
| **≥ 0,85** | Güçlü kanıt | Öneri doğrudan kabul edilebilir; standart dipnot |
| **0,65–0,85** | Kabul edilebilir | **İkinci kaynak zorunlu** (çapraz doğrulama) |
| **0,45–0,65** | Zayıf | + **Duyarlılık analizi zorunlu** (±%10/±%20 senaryo) + genişletilmiş dipnot |
| **< 0,45** | Yetersiz | Değerleme **"tahmin belirsizliği yüksek"** damgası alır; denetçi görüşünü etkileyebilir (BDS 705 — sınırlandırılmış görüş adayı) |

**Bu tabloyu tartışmak istiyorum.** Eşikler benim önerim; denetim pratiğinde bunların firma politikası
olarak belirlenmesi gerekir. Sence eşikler parametrik mi olmalı (her denetim firması kendi eşiğini
belirler), yoksa biz standart bir set mi dayatmalıyız?

---

## 5. İskonto oranı: tek doğru yok, bir *koridor* var

Senin dediğin gibi *"birden çok oran varsa karşılaştırılmalı, en gerçekçi olanı tespit edilmeli."*
Türkiye'de bugün bir iskonto oranı için en az **dört ayrı referans** var:

| Kaynak | Ne verir | Kullanım alanı |
|---|---|---|
| **TCMB politika/avans/reeskont faizi** | %39,75 avans (Tem 2026) | Reeskont (VUK+TFRS 9 kısa vadeli) |
| **Hazine tahvil getirisi (10Y)** | Risksiz oran (nominal) | TMS 19 kıdem iskontosu — standardın istediği "yüksek kaliteli tahvil" |
| **Damodaran ülke risk primi** | TR ERP %9,30 (Tem 2026) | DCF / kullanım değeri (TMS 36) — özkaynak maliyeti |
| **Şirketin fiili borçlanma faizi** | Gerçekleşen kredi maliyeti | TFRS 16 alternatif borçlanma faizi |

**Bunlar birbirini tutmaz — ve tutmaması normaldir**, çünkü farklı riskleri fiyatlarlar. Hata,
"tek doğru oran" aramaktır. Doğru yaklaşım: **her çalışma kendi standardının istediği oranı kullanır**,
ve program **hangi oranın neden seçildiğini** kayda düşer (bu zaten `degerleme_bazi` alanımız).

**Kritik gerçeklik kontrolü (senin "gerçekçilik sorgulanmalı" ilken):** Reel iskonto oranı =
nominal − enflasyon. Türkiye'de nominal %40, enflasyon %35 ise reel oran ~%3-4. Ama enflasyon
*beklentisi* %30 mu %45 mi? **Bu tek varsayım, kıdem tazminatı karşılığını iki katına çıkarabilir.**
Bu yüzden kıdem çalışmasında **duyarlılık analizi pazarlık konusu değil, zorunluluktur.**

---

## 6. Veri toplama katmanı — dürüst kısıtlar

Uzun vadeli hedef: parametrelerin otomatik gelmesi. Ama **yasal ve etik sınırları baştan koymalıyız**,
yoksa inşa ettiğimiz şey çöpe gider:

### Yeşil kuşak — serbestçe kullanılabilir (ÖNCE BUNLAR)
- **TCMB EVDS** — resmî API'si var (kur, KFE, Ticari GM Fiyat Endeksi, faiz serileri). ✅ Birinci hedef.
- **TÜİK** — açık veri portalı (TÜFE, Yİ-ÜFE, sektörel endeksler).
- **Hazine/BİST** — tahvil getirileri, kote fiyatlar.
- **KAP** — halka açık şirketlerin dipnotları (emsal iskonto oranı, ömür varsayımı hazinesi!).
- **SPK lisanslı değerleme şirketleri listesi** — kaynak otoritesi doğrulaması için.

### Sarı kuşak — izinli/lisanslı erişim gerekir
- Ticari veri sağlayıcılar (Endeksa vb.) — **API lisansı** ile.
- Değerleme raporlarının kamuya açık örnekleri — telif/kullanım şartlarına bakılmalı.

### Kırmızı kuşak — DİKKAT (dürüst uyarı)
İlan sitelerinden (sahibinden, hepsiemlak vb.) **otomatik veri çekmek**, bu sitelerin kullanım
şartlarında genellikle **açıkça yasaklanmıştır** ve teknik engellerle korunur. Bunu bilerek inşa
edersek: (a) hukuki risk, (b) sürekli kırılan bir sistem, (c) denetim aracı için **savunulamaz bir
kanıt kaynağı** (mahkemede "izinsiz çektim" diyemezsin) üretmiş oluruz.

**Alternatif yol — ve bence daha güçlüsü:** Denetçi emsal verisini **manuel girer** (zaten çalışma
kağıdına ekran görüntüsü/link koyarak), program da bunu **yapılandırılmış emsal kartı** olarak saklar:
`{kaynak, ilan/satış, tarih, konum, m², fiyat, link, ekran görüntüsü}`. Program **hesaplamayı ve
KGS'yi** yapar, **veriyi denetçi getirir**. Bu hem yasal, hem BDS 500 anlamında daha sağlam (denetçi
kanıtı bizzat görmüş ve değerlendirmiş olur). İleride resmi/lisanslı API'ler eklendikçe otomasyon artar.

**Bu, benim en önemli itirazım/katkım — tartışalım.** Otomatik çekim cazip ama denetim aracında
**kanıtın kaynağının savunulabilirliği**, otomasyondan daha değerli.

---

## 7. Parametre de bir kayıt gibi dayanaklı olmalı

Muhasebe kaydında dayanak istiyoruz (fatura, dekont). **Aynı disiplini parametreye uygulayalım:**

```
Parametre Kartı:
  anahtar        : "tcmb_avans_faiz"
  deger          : 39,75%
  kaynak         : TCMB — Resmî Gazete, Temmuz 2026
  kaynak_katmani : A (resmî kurum)
  alinma_tarihi  : 14.07.2026
  gecerlilik     : 01.07.2026 – 31.12.2026
  kgs            : 1,00
  dogrulayan     : (denetçi adı) / otomatik
  cakisan_kaynak : —
```

Bu kart, kayda `degerleme_bazi` olarak akar. **Bugün elimizde `ufrs-parametreleri.json` var ama
sadece değer + kaynak metni tutuyor — KGS, geçerlilik tarihi, çakışma alanı yok.** İlk somut adım bu
dosyayı bu şemaya yükseltmek olabilir.

---

## 8. Yol haritası — küçük adımlar, her biri tek başına değerli

Kullanıcı ilkesi: *"tek seferde bir yapı kurarsak her yerini tekrar yapmamız gerekecek."* Katılıyorum.
Bu yüzden **her adım tek başına çalışan bir parça** olacak:

| # | Adım | Neden bu sırada | Bağımlılık |
|---|---|---|---|
| **D1** | **Parametre kartı şeması** — `ufrs-parametreleri.json` → KGS + geçerlilik + kaynak katmanı | En ucuz, en temel; her şey buna dayanacak | — |
| **D2** | **KGS motoru** — 4 çarpanlı skor hesabı + eşiklere göre zorunluluk (duyarlılık/ikinci kaynak) | Tek başına test edilebilir saf fonksiyon | D1 |
| **D3** | **TCMB EVDS bağlantısı** — kur + KFE + faiz serileri (resmî API, yeşil kuşak) | Seviye 1 veri; kur değerlemesi (WS-KUR) anında gerçek olur | D1 |
| **D4** | **Emsal kartı** — denetçinin girdiği emsal verisi (yapılandırılmış) + emsal yakınlık skoru | GUD çalışmalarının (WS-GUD-MDV/YAGM) gerçek girdisi | D2 |
| **D5** | **Çelişki raporu** — çok kaynaklı karşılaştırma, %15 sapma bulgusu | Çapraz doğrulama kuralının hayata geçmesi | D2, D3, D4 |
| **D6** | **Duyarlılık analizi motoru** — ±%10/±%20 senaryo tablosu (kıdem, DCF, ECL) | Düşük KGS'nin zorunlu kıldığı çıktı; dipnota da gider | D2 |
| **D7** | **İskonto oranı koridoru** — 4 kaynak yan yana + seçim gerekçesi | Hangi oran neden seçildi, kayda düşsün | D1, D3 |
| **D8** | **KAP dipnot madenciliği** — halka açık şirketlerin varsayımları (emsal ömür/iskonto/temerrüt) | Seviye 2 kalibrasyon hazinesi; uzun vadeli | D1 |
| **D9** | **Lisanslı veri sağlayıcı entegrasyonu** (sarı kuşak) | Ticari anlaşma gerektirir — en sona | D3, D4 |

**D1–D2 birlikte bir hafta sürmez ve hemen değer üretir** (her kayıt KGS taşımaya başlar).
**D3 en yüksek getirili tek adım** (kur değerlemesi bugün elle giriliyor, resmî API'yle gerçek olur).

---

## 9. Açık sorular — birlikte tartışacaklarımız

1. **İlan fiyatı → satış fiyatı düzeltme katsayısı** nasıl kalibre edilir? (Bence: TCMB KFE ile
   ilan ortalamasının tarihsel sapmasından; ama veri var mı?)
2. **KGS eşikleri** firma politikası mı olmalı, biz mi dayatmalıyız?
3. **Enflasyon beklentisi** kimden alınır? (TCMB Piyasa Katılımcıları Anketi mi, TÜİK gerçekleşen mi,
   şirketin kendi bütçesi mi?) — Kıdem karşılığı buna aşırı duyarlı.
4. **Makine/ekipman emsali** gayrimenkulden zor: ikinci el sanayi makinesi piyasası şeffaf değil.
   Burada maliyet yaklaşımı (yenileme maliyeti − yıpranma) daha savunulabilir olabilir. Kabul mü?
5. **KGS düşük olduğunda denetçi görüşü** ne kadar etkilenmeli? Bu, programın haddini aşma riski
   taşır — görüş denetçinindir, program sadece **bayrak kaldırmalı**. Sınırı nereye çekiyoruz?

---

## 10. Neden bu iş bu kadar önemli (ve neden acele etmemeliyiz)

Bu çalışmalar **doğrudan bağımsız denetçi raporunun unsurlarıdır.** Bir GUD kaydı yanlışsa, bilanço
yanlıştır; bilanço yanlışsa denetçi görüşü yanlıştır; ve bunun sorumluluğu — KGK denetiminde,
mahkemede — **denetçinin üzerindedir.**

Bu yüzden buradaki her satır, "çalışıyor" olmaktan fazlasını gerektiriyor: **savunulabilir** olmalı.
Program bir denetçiye "şu değer doğrudur" dememeli; **"şu değere şu kanıtlarla ulaştım, güven skorum
şu, şurası zayıf"** demeli. Karar denetçinindir — program onun **muhakemesini besler ve izini tutar.**

*Bu doküman canlı bir taslaktır. Tartıştıkça büyüyecek.*

### Kaynaklar
[TFRS 13 (KGK)](https://www.kgk.gov.tr/Portalv2Uploads/files/Duyurular/v2/TFRS/TFRS_2020/TFRS%2013.pdf) ·
[BDS 500 Denetim Kanıtları](https://kgk.gov.tr/Portalv2Uploads/files/Duyurular/v2/BDS/BDS%20500.pdf) ·
[BDS 540 Muhasebe Tahminleri](https://www.kgk.gov.tr/Portalv2Uploads/files/Duyurular/v2/BDS/BDS%20540(1).pdf) ·
[BDS 620 Uzman Çalışmalarının Kullanılması](https://kgk.gov.tr/Portalv2Uploads/files/Duyurular/v2/BDS/BDS_620.pdf) ·
[SPK Gayrimenkul Değerleme Kuruluşları](https://spk.gov.tr/kurumlar/gayrimenkul-degerleme-kuruluslari) ·
[TCMB EVDS](https://evds3.tcmb.gov.tr/) ·
[TCMB Konut Fiyat Endeksi metaveri](https://tcmb.gov.tr/wps/wcm/connect/b4628fa9-11a7-4426-aee6-dae67fc56200/KFE-Metaveri.pdf?MOD=AJPERES) ·
[Damodaran Ülke Risk Primleri](https://pages.stern.nyu.edu/~adamodar/New_Home_Page/datafile/ctryprem.html)
