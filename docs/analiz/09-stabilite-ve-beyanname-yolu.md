# Stabilite Değerlendirmesi ve Beyannameye Giden Yol

> 2026-07-25. Öncelik sırası kullanıcının koyduğu gibi: **önce mevcut muhasebe/banka/fiş sistemini
> çalışır ve stabil hale getir**, sonra beyanname yolunu inşa et, sonra vergi denetimine katkı ver.
> Aşağıdaki bulgular tahmin değil, çalışan sistem üzerinde ölçüldü.

---

# BÖLÜM 1 — Mevcut sistemin durumu (ölçüm)

## 1.1 Çalışan zincir

Şu an uçtan uca çalışan ve testlerle korunan hat:

```
fatura (KDV oranı · iptal · iade · iskonto · tevkifat)
  → tahsilat olayı (cariye ait)
  → tahsis motoru (FIFO / manuel)
  → tahsis defteri (storno'lu, denetim izli)
  → GERÇEK defter (yevmiye → kebir → mizan)
  → KDV1 beyanname taslağı
```

Ölçülen: 90 test geçiyor · mizan Σborç = Σalacak · kasa ekranı ile 100 hesabı tutuyor ·
cari açık bakiye (120+320) simülasyonla birebir · iade oranı %4,11 üretiliyor.

## 1.2 Bulunan kusurlar — öncelik sırasıyla

### K1. Kalıcılık yok (BLOKE EDİCİ)
`AppState`'te **34 alan** var, hiçbiri diske yazılmıyor. Kod taramasında tek bir
`fs::write`/`File::create` yok. Sunucu yeniden başlayınca kaybolan:
yevmiye (11.160 fiş), tahsis defteri, storno zinciri, manuel eşleştirme kararları, şablonlar.

Bu artık kolaylık meselesi değil: **storno zinciri bir denetim izidir.** Kaybolması VUK 219'un
"kayıt silinmez" ilkesini fiilen ihlal eder. Stabilitenin tanımı budur.

### K2. KDV oranı defterde kayboluyor → beyanname matrahı YANLIŞ
Faturalar artık %1 / %10 / %20 / istisna taşıyor. Ama deftere **düz `391` ve `191`** olarak
yazılıyor. KDV1 motoru ise muavin kodundan oran okuyor (`391.20` → %20) ve çözemeyince
**hepsini %20 varsayıyor**.

Ölçülen sonuç (03/2026):
```
matrah 10.714.926,25 · hesaplanan 2.142.985,25 · oran kırılımı: TEK SATIR (%20)
```
Gerçekte o ayın satışları karma orandan oluşuyor. Yani **beyan edilen matrah gerçeği yansıtmıyor.**
KDV1 formu zaten oran bazlı satır ister; düz hesapla bu form doldurulamaz.

**Çözüm:** muavin hesaplar (`391.01/391.10/391.20`, `191.01/191.10/191.20`, istisna için ayrı).
Altyapı hazır — `muavin_olustur` / `POST /api/hesap/muavin` zaten var, kullanılmıyor.

### K3. Fiş tipi her zaman "Mahsup"
`muhasebe_aktar` bütün fişleri `FisTipi::Mahsup` olarak yazıyor. Oysa:
kasaya nakit giriş = **Tahsil fişi**, kasadan nakit çıkış = **Tediye fişi**.
Bu VUK açısından şekil şartıdır ve denetimde fiş tipi tutarsızlığı sorulur.

### K4. 06/2026 beyannamesi boş dönüyor — DOĞRULANMADI
03 ve 09 dolu, **06 sıfır**. Veri mi motor mu, ölçmedim. Araştırılmalı; sessiz veri kaybıysa
ciddidir.

### K5. Aktarım ve storno birim testi yok
`muhasebe_aktar` ve `defteri_senkronize` `AppState` gerektirdiği için yalnız uç düzeyinde
doğrulandı. Tahsis motorunun 26 testi var, bu ikisinin sıfır.

### K6. Sayfalama sınırları belgesiz
`/api/fisler` 300, `/api/yevmiye` 150 satır döndürüyor. Hata değil ama ölçüm yaparken beni
yanılttı; "11.160 fiş var" ile "300 fiş var" arasındaki fark sayfalama. Uç sözleşmesinde yazılmalı.

## 1.3 Banka tarafı eksikleri

| # | Eksik | Etki |
|---|---|---|
| B1 | Gerçek banka bağlantısı yok (simülasyon) | OAuth/açık bankacılık akışı kurulmadı |
| B2 | Eşleşme ipucu tek boyutlu (yalnız ünvan) | Ünvansız hareket maker'a düşüyor; IBAN→cari, açıklamada fatura no, tutar+tarih eşleşmesi eklenmedi |
| B3 | Banka masrafı / BSMV ayrıştırma yok | Parametrede BSMV %5 var, motor yok — masraf satırı 780/653'e ayrılmıyor |
| B4 | Çek/senet portföy takibi yok | 101/121 hesapları hareket görüyor ama vade/tahsil/karşılıksız izlenmiyor |

## 1.4 Fiş tarafı eksikleri

| # | Eksik | Etki |
|---|---|---|
| F1 | Şablonlar kalıcı değil | K1'in alt kümesi |
| F2 | Taslak fiş CRUD yok | Maker-checker'ın "maker" ayağı eksik; şu an ya kesinleşir ya hiç |
| F3 | Toplu fiş girişi yok | 1000 satırlık ekstre tek tek işleniyor |
| F4 | Masraf yeri (masraf_yeri_id) alanı var, kullanılmıyor | 7/A maliyet dağıtımı için gerekli |

---

# BÖLÜM 2 — Türkiye'de şirketlerin verdiği beyannameler

## 2.1 Beyanname haritası

| Beyanname | Dönem | Verilme | Bizdeki kaynak | Durum |
|---|---|---|---|---|
| **KDV1** (1 No.lu) | aylık | ertesi ay 28 | 391/191 muavin + 190/360 | 🟡 taslak var, **oran kırılımı bozuk (K2)** |
| **KDV2** (2 No.lu, sorumlu sıfatıyla) | aylık | ertesi ay 28 | tevkifatlı **alış** | 🔴 alış tevkifatı modellenmedi |
| **Muhtasar ve Prim Hizmet (MUHSGK)** | aylık (veya 3 aylık) | ertesi ay 26 | 360/361 + bordro | 🔴 bordro modülü yok |
| **Geçici vergi** | 3 aylık | dönemi izleyen 2. ayın 17'si | gelir tablosu | 🟡 hesap kağıdı var, KKEG/istisna kataloğu yok |
| **Kurumlar vergisi** | yıllık | Nisan | dönem sonu + geçici mahsup | 🟡 kapanış motoru var |
| **Yıllık gelir vergisi** (şahıs) | yıllık | Mart | GV tarifesi | 🔴 mükellef tipi ayrımı yok |
| **Damga vergisi** | aylık | 26 | 360 | 🔴 |
| **Ba / Bs formları** | aylık | ertesi ay sonu | cari + fatura toplamları | 🟢 **verimiz hazır** |
| **e-Defter berat** | aylık/3 aylık | | yevmiye + kebir XML | 🔴 imza/berat altyapısı yok |

> **DOĞRULANMADI:** Ba/Bs bildirim yükümlülüğünün 2026 itibarıyla kapsamı ve haddi
> (e-fatura/e-defter mükellefleri için istisna getirilip getirilmediği) tebliğden teyit edilmeli.
> Verilme günleri de yıllık tebliğle kayabiliyor — `vergi-parametreleri.json`'a taşınmalı.

## 2.2 Beyannameyi doldurmak için ne gerekiyor

### KDV1 — en yakın hedef
Formun istediği, bizde olması gereken:
1. **Oran bazlı matrah/KDV satırları** → muavin hesap (K2). *Bu tek başına en kritik iş.*
2. **İstisna kapsamındaki teslimler ayrı satır** → 601 ve `istisna_kod` zaten üretiliyor ✓
3. **Tevkifat uygulanan işlemler ayrı satır** → `tevkifat` alanı var ✓, satıra bağlanmadı
4. **İade edilen KDV düzeltmesi** → 610/611 kayıtlarında 191 borcu var ✓
5. **Önceki dönemden devreden (190)** → zincir çalışıyor ✓
6. **Ödenecek/devreden sonucu (360/190)** → mahsup motoru çalışıyor ✓

Yani KDV1'in **%70'i hazır**; eksik olan oran kırılımı ve iki özel satır.

### Ba/Bs — en hızlı kazanç
Bir cariye ait, haddi aşan alım (Ba) ve satım (Bs) toplamlarının aylık bildirimi.
Bizde **cari bazlı fatura toplamları zaten var** (`/api/simulasyon/firmalar` deseni).
Gereken: dönem filtresi + had eşiği + VKN alanı. Neredeyse hiç yeni motor gerekmiyor.

### MUHSGK — en uzak
Bordro modülü (F2) olmadan mümkün değil. Stopajlar (kira/serbest meslek) hâlihazırda 360'a
yazılabiliyor ama ücret tarafı yok.

---

# BÖLÜM 3 — Önerilen sıra

## Faz 1 — STABİLİTE (önce bu, kullanıcının önceliği)
1. **Kalıcılık** (K1). Dosya tabanlı başla (JSON snapshot + append-only journal), sonra SQLite.
   Kapsam: fişler, tahsis defteri, manuel tahsis, şablonlar, sayaçlar, aktarılan anahtarları.
2. **KDV muavin kırılımı** (K2). `391.01/391.10/391.20`, `191.*`; simülasyon bu kodlara yazsın.
   Bu hem defter doğruluğu hem beyanname ön şartı — iki fazın kesişimi, o yüzden Faz 1'de.
3. **Fiş tipi doğruluğu** (K3). Tahsil/Tediye/Mahsup ayrımı.
4. **06/2026 anomalisi** (K4) araştırılsın.
5. **Aktarım + storno birim testleri** (K5).

## Faz 2 — BEYANNAMEYE GİDEN YOL
6. KDV1 tam form (istisna + tevkifat satırları, oran kırılımı Faz 1'den gelir).
7. **Ba/Bs** — veri hazır, hızlı kazanç, çapraz kontrol için de temel.
8. KDV2 (alış tevkifatı modellensin).
9. Beyanname çıktısı: GİB'in beklediği format (BDP paketi / XML) — araştırma gerekir.

## Faz 3 — VERGİ DENETİMİ MODÜLÜ
Buranın çekirdeği **mutabakat**tır; beyan ile defterin birbirini tutması:
10. **Beyan ↔ defter mutabakatı**: beyan edilen hesaplanan KDV = defterdeki 391 net hareketi mi?
    Fark varsa bulgu. (Vergi incelemesinin ilk sorduğu şey budur.)
11. **Ba/Bs çapraz kontrolü**: bizim Bs'imiz karşı tarafın Ba'sıyla tutmalı — tutmuyorsa
    sahte/muhteviyatı itibariyle yanıltıcı belge riski.
12. **Tevsik ihlali raporu** — zaten üretiliyor ✓ (152 hareket bulgulandı), rapora bağlanmalı.
13. **Negatif kasa kontrolü** — motorda var ✓ (şu an 0 gün), bulgu listesine girmeli.
14. KDV indirim reddi riskleri: KDVK 30 (zayi mal), belgesiz gider, KKEG ayrımı.

---

## Not — neden bu sıra

Beyanname, defterin **türevi**dir. Defter yanlışsa beyanname de yanlış olur ve hata vergi
dairesine gider. Şu an K2 tam olarak bunu yapıyor: karma oranlı satışları tek orandan beyan
ediyoruz. Kalıcılık (K1) olmadan da hiçbir beyan taslağı bir sonraki güne kalmıyor.

Bu yüzden Faz 1 bitmeden Faz 2'ye geçmek, yanlış temele bina dikmek olur.

---

# EK — Faz 2 araştırma bulguları (25.07.2026)

## Ba/Bs KALDIRILDI — planın düzeltilmesi

Yukarıda Ba/Bs'i "en hızlı kazanç" diye işaretlemiş ve haddini doğrulanmamış bırakmıştım.
Doğrulama sonucu: **Ba/Bs bildirim zorunluluğu 565 Sıra No.lu VUK Genel Tebliği ile
25.09.2024 tarihinde tamamen kaldırıldı.** Gerekçe, GİB'in alım/satım verisine e-Fatura ve
e-Arşiv üzerinden zaten doğrudan erişebilmesi; ayrı bildirim uyum maliyeti olmaktan çıktı.

Motoru silmedim, **çerçevesini değiştirdim**. Ürettiği veri hâlâ üç işe yarıyor:
1. **İç mutabakat** — cari bazlı aylık alım/satım toplamı, cari ekstresiyle karşılaştırılır.
2. **Çapraz kontrol** — bizim satışımız karşı tarafın alışıyla tutmalı; tutmuyorsa sahte veya
   muhteviyatı itibariyle yanıltıcı belge riski. Faz 3'ün (vergi denetimi) temeli budur.
3. **Düzeltme** — 25.09.2024 öncesi dönemler için hâlâ gerekebilir.

Arayüzde de beyanname gibi sunulmuyor; sekme adı "Cari mutabakat" ve tepesinde
"bu bir beyanname değildir" uyarısı var. Vergi takviminden de Ba/Bs satırı çıkarıldı.

## KDV1 ↔ KDV2 bağlantısı (109 kodu)

Tevkifatta iki beyanname birbirine bağlanır:
1. Alıcı, tevkif ettiği KDV'yi **2 No.lu KDV beyannamesi** ile beyan eder ve öder.
2. Ödediği bu tutarı **1 No.lu KDV beyannamesinin indirim bölümünde "109 — Sorumlu Sıfatıyla
   Beyan Edilerek Ödenen KDV"** satırında indirim olarak gösterir; buna bağlı bildirim tablosu
   ekler bölümünde doldurulur.

Bizde şu an tevkifat yalnız **satış** tarafında modellendi (satıcı olarak 391'e kısmi yazıyoruz).
**Alış tevkifatı yok** — dolayısıyla KDV2 üretilemiyor ve KDV1'de 109 satırı doğmuyor.
Sıradaki iş bu: alış faturasına tevkifat alanı + KDV2 taslağı + KDV1'e 109 indirim satırı.

## Faz 2 güncel durum

| İş | Durum |
|---|---|
| KDV1 oran kırılımı | ✅ (Faz 1'de muavinle çözüldü) |
| KDV1 istisna teslim bölümü | ✅ matrah beyan edilir, hesaplanana eklenmez |
| KDV1 tevkifat bölümü | ✅ beyan edilen / tevkif edilen ayrımı |
| Cari mutabakat (eski Ba/Bs) | ✅ beyanname olarak değil, mutabakat raporu olarak |
| Frontend — KDV1 yeni bölümler | ✅ |
| Frontend — Cari mutabakat sekmesi | ✅ |
| **KDV2 + KDV1'e 109 satırı** | ⬜ alış tevkifatı modellenmeli |
| **GİB gönderim formatı (BDP/XML)** | ⬜ araştırılmadı — beyanname ÜRETİMİ ile GÖNDERİMİ ayrı işler |

> **DOĞRULANMADI:** e-Beyanname/BDP paket formatı ve gönderim protokolü hiç araştırılmadı.
> Şu an ürettiğimiz "taslak"tır; GİB'e gönderilebilir bir paket değildir. Bu ayrımı kullanıcıya
> arayüzde de söylemek gerekir (başlıkta "TASLAK" ibaresi duruyor).
