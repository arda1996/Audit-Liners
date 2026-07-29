# Beyanname Form Yapıları — Alan Haritası

> Kaynak: kullanıcının paylaştığı gerçek beyanname görüntüleri (BDP ekranları, kağıt formlar,
> doldurulmuş örnekler) + GİB tebliğleri. Her form için **alanlar → bizdeki karşılığı → eksik**.
>
> ⚠ **Görsellerin bir kısmı eski sürüm** (2005, 2010, 2019, 2024 tarihli). Alan *yapısı* büyük
> ölçüde aynı kalıyor ama **kod listeleri ve oranlar değişiyor**. Aşağıda oran/kod verilen her
> yerde güncel tebliğ teyidi şarttır; teyit edilmemişler işaretli.

---

## 1. KDV BEYANNAMESİ — 1 No.lu (Form 1015A)

Formun asıl iskeleti, doldurulmuş örnekten:

### MATRAH VE VERGİ BİLDİRİMİ
| Form alanı | Bizde | Durum |
|---|---|---|
| Matrah / KDV Oranı / Vergi (oran bazlı satırlar) | `matrahlar[]` — 391 muavin kırılımı | ✅ |
| Matrah Toplamı | Σ matrah | ✅ |
| Hesaplanan Katma Değer Vergisi | `hesaplanan_toplam` | ✅ |
| **Daha Önce İndirim Konusu Yapılan KDV'nin İlavesi** | — | ⬜ **eksik** |
| Toplam Katma Değer Vergisi | | ✅ |

Kağıt form bunu üç tabloya ayırıyor — bizim tek tablomuz bu ayrımı yapmıyor:
- **TABLO-1: TEVKİFAT UYGULANMAYAN İŞLEMLER** (Teslim ve Hizmet Bedeli · KDV Oranı · Hesaplanan KDV)
- **TABLO-2: KISMİ TEVKİFAT UYGULANAN İŞLEMLER** (+ **Tevkifat Oranı** sütunu) ⬜ oranı saklamıyoruz
- **TABLO-3: DİĞER İŞLEMLER**

### İNDİRİMLER
| Form alanı | Bizde | Durum |
|---|---|---|
| Önceki Dönemden Devreden İndirilecek KDV | `onceki_devreden` (190) | ✅ |
| Bu Döneme Ait İndirilecek KDV | `bu_donem_indirilecek` (191) | ✅ |
| **Sorumlu Sıfatıyla Beyan Edilen KDV** (kod **109**) | — | ⬜ **eksik — KDV2 bağlantısı** |
| Yurtiçi Alımlara İlişkin KDV | — | ⬜ ayrıştırılmıyor |
| İndirimler Toplamı | `indirimler_toplam` | ✅ |

### BU DÖNEME AİT İNDİRİLECEK KDV'NİN ORANLARA GÖRE DAĞILIMI
| Form alanı | Bizde | Durum |
|---|---|---|
| KDV Oranı · Alınan Mal ve Hizmete Ait Bedel · KDV Tutarı | 191 muavinleri **var** | 🟡 veri hazır, **çıktıya bağlanmadı** |

Bu, hızlı bir kazanç: `191.01/191.10/191.20` muavinleri zaten oran taşıyor; hesaplanan tarafta
yaptığımız kırılımın aynısı indirim tarafında da yapılabilir.

### SONUÇ HESAPLARI
Ödenmesi Gereken KDV · İade Edilmesi Gereken KDV · Sonraki Döneme Devreden KDV → ✅ (`odenecek`/`devreden`)
**İade Edilmesi Gereken KDV** ayrı bir kalem; bizde yok (devredene çeviriyoruz) ⬜

### İHRACAT / İSTİSNA BÖLÜMÜ
İhraç Kaydıyla Teslim Bedeli Toplamı · Tecil Edilebilir KDV · Yüklenilen KDV ·
İstisna Kapsamına Giren İşlemlere Ait Toplam Teslim ve Hizmet Tutarı · İade Edilebilir KDV
→ Bizde yalnız **istisna matrahı** var ✅; **yüklenilen KDV / tecil / iade** yok ⬜

---

## 2. KDV BEYANNAMESİ — 2 No.lu (Form 1015B) · sorumlu sıfatıyla

BDP ekranı iki tablo gösteriyor — **bu ayrım bizde hiç yok**:

| Bölüm | Sütunlar |
|---|---|
| **TAM TEVKİFAT UYGULANAN İŞLEMLER** | Matrah · KDV Oranı · Vergi |
| **KISMİ TEVKİFAT UYGULANAN İŞLEMLER** | Matrah · KDV Oranı · **Tevkifat Oranı** · Vergi |

Sonuç alanları: Matrah Toplamı · Vergi Toplamı · Toplam KDV Matrahı · **Tevkif Edilen KDV Toplamı** ·
İlave Edilecek KDV · Ödenmesi Gereken KDV.

**Tam tevkifat ≠ kısmi tevkifat:** tam tevkifatta KDV'nin **tamamı** alıcıda (10/10);
kısmi tevkifatta 2/10–9/10 arası bölünür. Bizim motorumuz yalnız kısmi tevkifatı ve yalnız
**satış** tarafını modelliyor. KDV2 için **alış tevkifatı** gerekiyor.

### KDV1 ↔ KDV2 zinciri (kritik)
1. Alıcı, tevkif ettiği KDV'yi **KDV2** ile beyan eder ve öder.
2. Ödediği tutarı **KDV1'in indirim bölümünde "109 — Sorumlu Sıfatıyla Beyan Edilerek Ödenen KDV"**
   satırında indirim gösterir; ekler bölümünde bildirim tablosu doldurulur.

Doldurulmuş örnekte bu görünüyor: `Sorumlu Sıfatıyla Beyan Edilen KDV 1.000,00`.

### Tevkifat işlem türü kodları (BDP ekranından)
| Kod | Madde | İşlem |
|---|---|---|
| 406 | 29/2 | Yıl içinde indirimli orana tabi işlemlere ilişkin iade |
| 408 | 11/1-b | Türkiye'de ikamet etmeyenlere iade edilen KDV |
| 409 | 9/1 | İstisnadan vazgeçenlerin hurda/atık tesliminde tevkifat |
| 410 | 9/1 | Yapım işleri + mühendislik-mimarlık hizmetleri |
| 411 | 9/1 | Temizlik hizmeti |
| 412 | 9/1 | Çevre ve bahçe bakım hizmetleri |
| 413 | 9/1 | Özel güvenlik hizmeti |
| 414 | 9/1 | Makine, teçhizat, demirbaş ve taşıt tadil/bakım/onarım |
| 415 | 9/1 | Yemek servis hizmeti |
| 416 | 9/1 | Etüt, plan-proje, danışmanlık, denetim ve benzeri |
| 417 | 9/1 | Hurda metalden elde edilen külçe teslimleri |
| 419 | 9/1 | Bakır, çinko, alüminyum, kurşun ve alaşımları |
| 420 | 9/1 | Yapı denetim hizmetleri |

> **DOĞRULANMADI:** Bu liste **2010 dönemi BDP ekranından** okundu. Kod numaraları ve kapsam
> güncel KDV Genel Uygulama Tebliği ile değişmiş olabilir; **her koda karşılık gelen tevkifat
> oranı da burada YOK** ve tebliğden alınmalıdır. Motor bu yüzden oranı muhasebeciye seçtiriyor,
> kendisi tahmin etmiyor.

> **Tevkifat alt sınırı (2026): KDV dahil 12.000 TL** — bu haddin altındaki işlemde tevkifat
> uygulanmaz. Kaynak ikincil (sektör yayınları), **tebliğden teyit edilmeli**.

---

## 3. GEÇİCİ VERGİ BEYANNAMESİ (Form 1032)

Doldurulmuş örnekten tam satır listesi:

| Form satırı | Bizde | Durum |
|---|---|---|
| Ticari Bilanço Karı veya Zararı | gelir tablosu | ✅ |
| Kanunen Kabul Edilmeyen Giderler (KKEG) | manuel satır | 🟡 kod kataloğu yok |
| Zarar Olsa Dahi İndirilecek İstisna ve İndirimler | — | ⬜ |
| Kar Toplamı / Zarar Toplamı | ✅ | |
| Mahsup Edilecek Geçmiş Yıl Zararları | — | ⬜ |
| İndirime Esas Tutar | ✅ | |
| **İşletmeden Çekilen Enflasyon Düzeltmesi Farkları** | — | ⬜ |
| Safi Geçici Vergi Matrahı | ✅ | |
| KVK 32/A indirimli kurumlar vergisi matrahı / oranı | — | ⬜ teşvik belgeli yatırım |
| KVK Geçici 4/5 Mad. indirimli matrah / oran | — | ⬜ |
| Genel Orana Tabi Geçici Vergi Matrahı | ✅ | |
| **VERGİ BİLDİRİMİ** | | |
| Hesaplanan Geçici Vergi | ✅ | |
| Önceki Dönemlerde Hesaplanan Geçici Vergi | ✅ (kümülatif) | |
| Mahsup Edilecek Yabancı Ülkelerde Ödenen Vergi | — | ⬜ |
| **Mahsup Edilecek Tevkifat Tutarı** | — | ⬜ (stopaj mahsubu) |
| Ödenecek Geçici Vergi | ✅ | |
| **Damga Vergisi** | — | ⬜ beyanname damgası (örnekte 303,10) |

### EK: FATURA BİLGİLERİ TABLOSU
Merkez/Şube · **Matbaa VKN** · Fatura Tarih Seri-Sıra No · **Alıcı VKN** · Tutar (KDV Dahil)

Geçici vergi döneminde kullanılan faturaların **en son fatura bilgileri** beyan ediliyor.
Bizde fatura no/tarih/alıcı VKN **var** ✅; matbaa VKN yok (e-faturada matbaa yok) —
e-belge mükellefinde bu tablonun kapsamı teyit edilmeli.

---

## 4. MUHTASAR BEYANNAME (Form 1003)

| Form alanı | Bizde | Durum |
|---|---|---|
| **Ödemelerin Tür Kodu** (tablo) | — | ⬜ **kod kataloğu yok** |
| Ödemelerin Gayrisafi Tutarı | — | ⬜ |
| Gelir Vergisi Kesintisi Tutarı | 360'a yazılıyor 🟡 | ayrıştırılmıyor |
| Ücret Öze. Yap. Tevkifat / 5084 terkin | — | ⬜ bordro gerekir |
| KVK 34 uyarınca mahsup edilecek tevkifat | — | ⬜ |
| Asgari geçim indiriminden mahsup | — | ⬜ (AGİ kaldırıldı — teyit) |
| **Çalıştırılan Kişi Sayısı** (aylık/3 aylık) | — | ⬜ bordro |
| Tevkifata İlişkin Damga Vergisi | — | ⬜ |
| Muhtasar Beyannamesine Ait Damga Vergisi | parametre var ✅ | bağlanmadı |

**Sonuç:** Muhtasar, bordro modülü olmadan üretilemez. Ama **kira ve serbest meslek stopajı**
bizde hesaplanabiliyor (hesap makinesi ✅) — bunlar muhtasarın bir bölümünü besleyebilir.

---

## 5. KURUMLAR VERGİSİ BEYANNAMESİ (Form 1010)

TABLO-1 Kurumun kimlik/adres · TABLO-2 Dar mükellef temsilcisi · TABLO-3 Bağlı işyerleri
(Şube/Çıkış Yeri/Ajanlık/İmalat Yeri/Satış Yeri/Saat) · TABLO-4 Geçmiş yıl zararları ·
sekmeler: Gelir İdaresi · Gl. Yıl Zararları · Kazanç ve İlave · Vergi-Mali-Damga Bil. · Düzenleme Bil. · Ekler

Bizde: dönem sonu kapanış motoru ✅, geçici vergi mahsubu 🟡, **mükellef kimlik bilgileri eksik**
(VKN/ticaret sicil/vergi dairesi alanları `MukellefProfil`'de kısmi) ⬜

---

## 6. FORM BA / BS — **YÜRÜRLÜKTE DEĞİL**

Form alanları: Sıra No · Soyadı/Adı veya Unvanı · **Ülkesi** · Vergi Kimlik Numarası ·
**TC Kimlik Numarası** · Belge Sayısı · Mal ve Hizmetlerin Toplam Bedeli (KDV Hariç)

Bizdeki mutabakat raporunda: unvan ✅ · VKN ✅ · belge sayısı ✅ · tutar (KDV hariç) ✅
**Eksik: Ülke ve TC Kimlik No** (gerçek kişi carilerde TCKN kullanılır) ⬜

> Ba/Bs bildirimi **565 Sıra No.lu VUK GT ile 25.09.2024'te kaldırıldı.** Form yapısı burada
> yalnız **çapraz kontrol** ve eski dönem düzeltmeleri için referans olarak duruyor.

---

## 7. YENİ TESPİT EDİLEN — haritada olmayan formlar

| Form | Ne | Bizdeki durum |
|---|---|---|
| **Döviz Gelirleri Beyan Formu** | Firma · doğrudan kazanılan/devredilen/devralınan döviz gelirleri; devir muvafakatname, gümrük beyanname, fatura bilgileri | ⬜ **haritada yoktu** — ihracatçı mükellefte gerekir |
| **BSMV (6802)** | 2834 sayılı CK: kambiyo satışlarında BSMV; bankacılık dışı kurumlara satışlar | 🟡 parametrede oran var (%5), motor yok |
| **ÖTV Beyannamesi** | Liste bazlı, sektörel | ⏸ bilinçli kapsam dışı |
| **Damga Vergisi Beyannamesi** | Sürekli damga mükellefiyeti | 🟡 hesap makinesi var ✅, beyanname yok |

---

## 8. Öncelik önerisi (bu analizden çıkan)

**Hızlı kazançlar (veri hazır, yalnız çıktıya bağlanacak):**
1. **İndirilecek KDV'nin oranlara göre dağılımı** — 191 muavinleri var, hesaplanan tarafındaki
   kırılımın aynısı. Neredeyse bedava.
2. **KDV1'de tevkifatlı/tevkifatsız ayrımı** — TABLO-1 / TABLO-2 ayrımı; `tevkifat` alanı var.
3. **Ba/Bs mutabakatına Ülke + TCKN alanı** — cari kartına eklenecek.

**Orta (yeni modelleme gerekir):**
4. **Alış tevkifatı → KDV2 + KDV1'e 109 satırı.** Zincirin kapanması için şart.
5. **Tevkifat oranının saklanması** — şu an yalnız tutar var, oran yok; TABLO-2 oranı istiyor.
6. **Geçici vergi: damga vergisi + tevkifat mahsubu satırları.**

**Uzak (başka modül gerekir):**
7. Muhtasar (bordro), KV beyannamesi kimlik blokları, ihracat istisna/iade bölümü (yüklenilen KDV).
