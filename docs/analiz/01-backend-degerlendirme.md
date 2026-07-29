Aşağıda backend değerlendirmesi. Bulgulardaki 33 dökümana dayanarak mevcut mimariyi (Rust domain + axum, TDHP tabanlı, VUK odaklı, tamsayı kuruş) etraflıca inceledim.

---

# Audit-Liners Backend Değerlendirmesi (Muhasebe Standartları Perspektifi)

## 0) Önce Stratejik Çerçeve: VUK mı, TMS mi?

Bu, mimarinin **en kritik kararı** ve mevcut yaklaşımınız doğru zeminde. Net koyuyorum:

**Çekirdek = VUK/TDHP olmalı. TMS bir KATMAN (overlay) olarak eklenmeli, çekirdek değil.**

Gerekçe (bulgulara dayanarak):
- Hedef kitleniz SMMM + KOBI. Bulguların neredeyse tamamı (TMS 33 §2, TMS 27 §7, TMS 28 §18, TMS 39 sektörel not, TMS 26 sektörel not) tam-set TFRS'nin **yalnızca halka açık / KGK denetimine tabi / büyük işletmeler** için zorunlu olduğunu, tipik KOBI'nin BOBİ FRS veya VUK/TDHP uyguladığını söylüyor. TMS 39 notu birebir: "korunma muhasebesi modülü OPSİYONEL/ileri seviye olmalı ve varsayılan kapalı gelmelidir."
- Fakat **hemen hemen HER TMS dökümanının `vuk_tms_farki` alanı** aynı sonuca varıyor: yazılım "VUK değeri" ile "TMS değeri"ni **AYRI saklamalı** ve aradaki farkı **ertelenmiş vergi (TMS 12)** köprüsüyle raporlamalı. Yani çekirdek defter VUK, TMS ise aynı işlemin ikinci bir değerleme katmanı.

Doğru mimari şekli:

```
İşlem (yevmiye) 
   ├── VUK değeri (çekirdek defter — kayıt, mizan, beyanname)   ← ZORUNLU, herkes
   └── TMS değeri (paralel overlay — finansal raporlama)         ← OPSİYONEL, denetim/büyük işletme
        └── Fark = geçici/kalıcı fark → TMS 12 ertelenmiş vergi
```

Bu yüzden aşağıda mimariyi **"dual-value defter"** olarak değerlendiriyorum: bu tek karar 15+ dökümanın ortak talebini karşılıyor.

---

## 1) NEYİ DOĞRU YAPMIŞIZ

| Yapılan | Dayanak (bulgular) | Değerlendirme |
|---|---|---|
| **Para = tamsayı kuruş (i64), FLOAT YOK** | TMS 2, TMS 12, TMS 21 — hepsi hassas ölçüm gerektiriyor; kur/reeskont/amortisman hesapları yuvarlama hatası kaldırmaz | **Kritik doğru karar.** Muhasebede float felakettir. Bunu koruyun. |
| **TDHP 261 hesabı koddan türetme (tip/doğa/yaprak)** | Genel Muhasebe Kitabı: kök gruplar 1-9/0, düzenleyici (-) hesaplar, nazım hesaplar mali tabloya etki etmez | Doğru temel. Hesap kodunun ilk hanesinden grup/karakter türetmek TDHP'nin ruhuna uygun. |
| **Çift taraflı defter, borç=alacak kısıtı** | Genel Muhasebe Kitabı: "her yevmiye maddesinde borç toplamı = alacak toplamı" | Temel denge invariant'ı doğru yerde. |
| **Dönem kapanış/açılış VİRMAN (6→690→692→590/591)** | Ön Büroda Muhasebe: 691→370, 690→692→590/591 zinciri | Kapanış zinciri mevzuata uygun. |
| **Mali tablolar: bilanço aktif=pasif, gelir tablosu** | Genel Muhasebe: "Aktif Toplam = Pasif Toplam her zaman korunmalı" | Temel raporlama doğru. |
| **Cari (120.xx/320.xx), muavin (alt hesap kod-bazlı)** | TMS 24 yazılım etkisi: ilişkili taraf bakiyeleri cari hesap bayrağı üzerinden yakalanmalı | Cari yapısı var; ancak "ilişkili taraf" bayrağı eksik (aşağıda). |
| **VUK odaklı başlama, vergi motorunu erteleme** | Tüm TMS notları VUK'u çekirdek kabul ediyor | Stratejik olarak doğru — TMS'yi önce yapıp KOBI'yi kaçırmamışsınız. |

**Özet:** Temel muhasebe altyapısı (double-entry, TDHP türetme, tamsayı para, kapanış) **sağlam ve doğru zeminde**. Sorun eksikliklerde, mimari hatada değil.

---

## 2) NEYİ YANLIŞ / EKSİK YAPMIŞIZ

Her madde somut + dayanak dökümanla.

### 2.1 — Dual-value (VUK/TMS) defter altyapısı YOK → en büyük yapısal eksik
- **Sorun:** Her hesap/varlık tek bir değerle tutuluyor. Oysa 15+ TMS dökümanı "VUK değeri" ve "TMS değeri"ni ayrı saklamayı zorunlu kılıyor.
- **Dayanak:** TMS 12 (`vuk_tms_farki`: "her hesap için 'VUK değeri' ve 'TMS değeri'ni ayrı tutarak"), TMS 16, TMS 2, TMS 19, TMS 40, TMS 41 — hepsi aynı.
- **Etki:** Bu altyapı olmadan ertelenmiş vergi, TMS raporlaması, bağımsız denetim modülü **hiç yapılamaz**. Bu eksik diğer tüm TMS eksiklerinin kök nedeni.

### 2.2 — Dönem sonu DEĞERLEME düzeltmeleri YOK (kendi belirttiğiniz eksik)
Bu "ertelendi" demişsiniz ama muhasebenin çekirdeği bu. Alt kırılım:

- **Amortisman motoru yok** — TMS 16 + VUK farkı: VUK'ta sabit oran/faydalı ömür listesi, TMS'de gerçek faydalı ömür + kalıntı değer. **İki ayrı amortisman defteri (dual-book) zorunlu** (TMS 16 yazılım etkisi: "Her kıymet için VUK ve TMS amortismanı paralel hesaplanmalı"). Ayrıca VUK amortisman tabanı = maliyet (hurda yok), TMS = maliyet − kalıntı değer → sürekli fark.
- **Reeskont yok** — Genel Muhasebe: iç iskonto formülü `(a×n×t)/(36500+n×t)`, izleyen dönem başı zorunlu iptal kaydı. **Alacak reeskontu varsa borç reeskontu da zorunlu** (Vergi Komitesi + KV Örnek: "seçimlik hak değildir", aksi halde lehe reeskont KKEG).
- **Şüpheli alacak karşılığı yok** — VUK: dava/icra/protesto şartı; teminatlı kısma karşılık ayrılmaz. TFRS 9: beklenen kredi zararı (hukuki takip olmadan). İki mantık ayrı.
- **Stok değer düşüklüğü (NGD) yok** — TMS 2: `MIN(maliyet, NGD)`, kalem bazında, iptal maliyetle sınırlı. VUK'ta serbest NGD karşılığı KKEG. TDHP 158.
- **Kur değerleme yok** — TMS 21: sadece **parasal** kalemler kapanış kuruyla; parasal olmayanlar (stok/MDV/avans) işlem tarihi kuruyla sabit. Her hesaba `is_monetary` bayrağı şart.
- **Karşılıklar (TMS 37) yok** — %50 olasılık eşiği, koşullu borç yevmiyeye yazılmaz sadece dipnot, iskonto (vergi öncesi oran).

### 2.3 — Ertelenmiş vergi (TMS 12) motoru YOK
- **Dayanak:** TMS 12 tüm dökümanın kalbi. "Geçici fark = TMS defter değeri − VUK değeri"; iskonto YASAK (§53); tersine dönme yılı oranı (cari değil).
- **Kritik ayrım:** **Geçici fark** (ertelenmiş vergi doğurur) vs **kalıcı fark/KKEG** (sadece efektif oran mutabakatı, ertelenmiş vergi doğurmaz). Bu ayrım motorda kodlanmalı.
- **Etki:** 2.1 (dual-value) olmadan bu imkânsız; 2.1 olunca bu neredeyse otomatik türeyecek.

### 2.4 — Enflasyon düzeltmesi YOK (kritik ve güncel!)
- **Dayanak:** TMS 29 + VUK mük.298. **İkisi AYRI motor** (TMS 29 `vuk_tms_farki`): tetik eşiği farklı (VUK: 3 yıl >%100 VE son yıl >%10; TMS: takdir), net parasal pozisyon işleyişi farklı.
- **GÜNCEL VERGİ NOTU (KV Örnek 2025):** VUK Geçici 37 ile **2025, 2026, 2027 hesap dönemlerinde enflasyon düzeltmesi YAPILMAZ** (istisna: münhasıran altın/gümüş). Yani bugünün Türkiye'sinde **VUK tarafı şu an düzeltme yapmıyor** — motoru yapın ama varsayılan kapalı, 2028 için hazır olsun.
- **Altyapı gereği:** Her parasal olmayan kaleme **iktisap/işlem tarihi** ve **parasal/parasal-olmayan bayrağı** şart (Yİ-ÜFE endeks tablosu + tarih bazlı düzeltme katsayısı).

### 2.5 — Kıdem tazminatı / bordro / SGK YOK
- **Dayanak:** TMS 19 + Bordrolama S&C + SGK Son Değişiklikler.
- **En keskin VUK-TMS farkı:** VUK'ta kıdem yalnızca **fiilen ödendiğinde** gider (ayrılan karşılık KKEG); TMS 19'da **aktüeryal karşılık** (projected unit credit, iskonto, maaş artışı, ayrılma olasılığı). Aktüeryal kazanç/kayıp → **OCI** (kâra değil).
- **Bordro çekirdeği:** kümülatif GV matrahı, damga vergisi (brüt × 0,00759), SGK tavan (2026: 9 kat = 297.270 TL, katsayı parametrik!), yemek istisnası (2026: günlük 300 TL), engellilik indirimi. Bunların hepsi **yıl-bazlı versiyonlu parametre tablosu** gerektiriyor.

### 2.6 — İlişkili taraf (TMS 24) meta-verisi YOK
- **Dayanak:** TMS 24 yazılım etkisi: cari/kontak kartına `ilişkili_taraf` bayrağı + 7 kategorili ilişki türü enum. "Sadece hesap koduna (131/331) güvenmek YETERSİZ" — 120/320 içindeki ilişkili taraf da yakalanmalı.
- **Çift kullanım:** Hem TMS 24 dipnotu hem KVK 13 transfer fiyatlandırması (KV Örnek) aynı bayrağı kullanır.

### 2.7 — Vergi/KDV motoru YOK (kendi belirttiğiniz eksik) + beyanname köprüsü
- **Dayanak:** Genel Muhasebe + Gelir Vergisi Rehberi + KV Örnek.
- **Ticari kâr → mali kâr köprüsü:** `Mali Kâr = Ticari Kâr + KKEG − İstisnalar − Geçmiş Yıl Zararı`. Bu köprü **beyanname simülasyonu** olarak ayrı çalışma tablosu olmalı; her satır KVK madde referansıyla etiketli.
- **KDV:** binek oto KDV'si maliyete (indirilemez), 191/391/190/360 mahsup, konaklama %8/%18 (sektörel). Tüm oranlar **hardcode DEĞİL** — yıl/tarih-etkin parametre.
- **KKEG kataloğu:** binek oto had aşımı, finansman gider kısıtlaması (%10), örtülü sermaye faiz+kur farkı, reeskont asimetrisi — her biri ayrı KKEG kodu.

### 2.8 — Kalıcılık (Postgres) YOK
- Bellek-içi API ile denetim izi (audit trail), çok dönemli veri, geriye dönük düzeltme (TMS 8), enflasyon karşılaştırmalı yeniden düzeltme imkânsız. **TMS 8** çok dönemli açılış/kapanış bakiyesi saklama zorunlu kılıyor.

### 2.9 — İştirak/bağlı ortaklık sınıflandırması YOK
- **Dayanak:** Genel Muhasebe + TMS 27/28. Oy hakkı eşiği: <%10 menkul kıymet, %10-50 iştirak (242), >%50 bağlı ortaklık (245). TMS 28 özkaynak yöntemi (VUK maliyet vs TMS özkaynak → ertelenmiş vergi). SMMM KOBI için genelde maliyet yöntemi yeterli ama eşik sınıflandırması çekirdek.

### 2.10 — TMS 8 (politika/tahmin/hata) ayrımı YOK
- **Dayanak:** TMS 8. Üç senaryo tamamen farklı işlenmeli: politika→geriye dönük, tahmin→ileriye yönelik, hata→geriye dönük yeniden düzenleme. Amortisman ömür değişimi = tahmin (ileriye), en sık karışan hata bu.

---

## 3) VERİ MODELİNE / İŞLEMEYE EKLENECEKLER — ÖNCELİK SIRASIYLA

Öncelik = "KOBI çekirdeği önce, TMS overlay sonra" ilkesine göre.

### FAZ 0 — Temel altyapı (bunlar olmadan hiçbir şey ilerlemez)
1. **Kalıcılık (Postgres)** + **denetim izi** (immutable audit trail: kim/ne zaman/hangi belge). Silme yok, ters kayıt.
2. **Dual-value alan yapısı:** her hesap/varlık kaydına `vuk_deger` + `tms_deger` (i64 kuruş). TMS başta null geçilebilir ama şema hazır olmalı. → *Bu tek karar 15 dökümanı açar.*
3. **Çok dönemli (multi-period) yapı:** her hesabın dönem-açılış ve dönem-kapanış bakiyesi ayrı (TMS 8 için de gerekli).
4. **Hesap meta-verisi genişletme:** `is_monetary` (TMS 21/29), `is_related_party` + `iliski_turu` enum (TMS 24), `iktisap_tarihi` (TMS 29 endeksleme), düzenleyici(-)/nazım bayrakları.

### FAZ 1 — VUK dönem sonu değerlemesi (KOBI'nin gerçek ihtiyacı)
5. **Amortisman motoru (dual-book):** VUK amortismanı (sabit oran/liste, hurdasız) + TMS amortismanı (faydalı ömür, kalıntı değer). MDV kartı: maliyet bileşenleri, `kullanıma_hazır_tarihi` (amortisman başlangıcı), komponent desteği (opsiyonel). Fark → geçici fark tablosu.
6. **Reeskont motoru:** iç iskonto formülü, alacak-borç **simetri zorlaması** (asimetride lehe gider otomatik KKEG), izleyen dönem başı otomatik iptal kaydı.
7. **Şüpheli alacak karşılığı:** VUK (dava/icra şartı, teminatlı hariç, TDHP 128/129); TFRS 9 ECL alanı şema olarak.
8. **Stok değer düşüklüğü (TMS 2):** `MIN(maliyet, NGD)`, kalem bazında, TDHP 158, iptal tavanı (maliyeti aşamaz). Maliyet yöntemi enum {ÖZEL, FIFO, AĞIRLIKLI_ORTALAMA} — **LIFO YOK**.
9. **Kur değerleme motoru:** sadece parasal yabancı para kalemleri kapanış kuruyla (646/656); parasal olmayanları otomatik dışla. Kur tablosu (tarih, para, kur tipi).

### FAZ 2 — Vergi motoru + beyanname köprüsü (KOBI için kritik gelir kaynağı)
10. **Ticari kâr → mali kâr köprüsü** (beyanname simülasyonu): KKEG kataloğu (seed data), istisna/indirim, geçmiş yıl zararı (5 yıl FIFO mahsup), her satır KVK madde referanslı.
11. **KDV motoru:** 191/391/190/360, indirilemez KDV→maliyet kuralı, tüm oranlar **yıl-bazlı parametre**.
12. **Ertelenmiş vergi (TMS 12):** dual-value farkından geçici/kalıcı ayrımı, tersine dönme yılı oranı, **iskonto YASAK**. (5 numaradan otomatik beslenir.)

### FAZ 3 — Bordro/SGK (ayrı ama zorunlu modül)
13. **Bordro çekirdeği:** brütten nete, kümülatif GV matrahı, damga vergisi, SGK/işsizlik, **yıl-bazlı versiyonlu parametre tablosu** (asgari ücret, GV dilimleri, SGK tavan katsayısı, yemek/engellilik istisnaları).
14. **Kıdem tazminatı:** VUK (ödendiğinde gider, karşılık KKEG) + TMS 19 (aktüeryal, OCI). Aktüeryal varsayım parametre tablosu (dönem versiyonlu).

### FAZ 4 — TMS tam-set overlay (yalnızca denetim/büyük işletme, varsayılan KAPALI)
15. **Enflasyon düzeltmesi:** VUK mük.298 motoru (2025-2027 **kapalı**, Geçici 37) + TMS 29 motoru (ayrı). Yİ-ÜFE tablosu, tarih bazlı katsayı, net parasal pozisyon.
16. **İştirak/bağlı ortaklık:** oy hakkı eşiği sınıflandırma + TMS 28 özkaynak yöntemi.
17. **TMS 8 üç-senaryo motoru:** politika/tahmin/hata ayrı işleme yolları.
18. **Dipnot üretici + bağımsız denetim modülü** (ilişkili taraf, karşılık hareket tabloları, TMS 24/37 açıklamaları).

---

## Kritik Noktalarda Net Duruş (özet cevaplar)

- **Çekirdek hangisi?** → **VUK/TDHP çekirdek, TMS overlay.** Ama dual-value şemasını FAZ 0'da kur ki TMS'yi sonradan söküp takmak yerine yalnızca doldurman gerekssin.
- **TMS nereye girer?** → Bağımsız denetime tabi / KGK kapsamı / büyük işletmeler. Modül **varsayılan kapalı**, müşteri profiline `raporlama_cercevesi` bayrağıyla açılır (VUK / BOBİ FRS / tam TFRS).
- **Enflasyon düzeltmesi:** İki ayrı motor. VUK tarafı **2025-2027 yasal olarak durdurulmuş** (Geçici 37) — motoru yaz, kapalı bırak. TMS 29 ayrı.
- **Ertelenmiş vergi:** Dual-value farkından türer; **iskonto asla yapma**; geçici/kalıcı ayrımı motorda zorunlu.
- **Kıdem tazminatı:** VUK = ödeme esası (karşılık KKEG), TMS 19 = aktüeryal + OCI. İki katman.
- **Amortisman:** İki paralel defter (VUK sabit oran/hurdasız vs TMS faydalı ömür/kalıntı değerli) — en yaygın geçici fark kaynağı, FAZ 1'de öncelikli.

**Tek cümlelik yol haritası:** FAZ 0'daki dual-value + kalıcılık + meta-veri altyapısını kurun; bu tek yatırım, ertelenmiş vergiden enflasyon düzeltmesine, kıdem tazminatından TMS raporlamasına kadar bulgulardaki tüm TMS taleplerini "veri var, sadece hesaplama ekle" seviyesine indirger.