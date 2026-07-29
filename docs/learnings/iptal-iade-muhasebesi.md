# İptal, İade ve Satış İndirimleri — Muhasebe Kuralı

> Kaynak: MSUGT (Tekdüzen Muhasebe Sistemi Uygulama Genel Tebliği) gelir tablosu formatı ·
> VUK 219/227/229 · KDVK 35 (matrah değişikliği) · GİB e-Belge senaryoları.
> Bu belge, motorun uyması gereken kuralı sabitler. Program bu kuralı ihlal ederse defter yanlıştır.

## 0. En kritik ayrım: İPTAL ≠ İADE

Bu ikisi aynı şey sanılır ve **muhasebe kayıtları tamamen farklıdır**.

| | **İPTAL** | **İADE** |
|---|---|---|
| Ne oldu? | Belge hiç hüküm doğurmadı | Belge hüküm doğurdu, sonra mal geri geldi |
| Teslim | Mal teslim **edilmedi** / hizmet ifa edilmedi | Mal teslim **edildi**, sonra iade |
| Kayıt yöntemi | **Ters kayıt (storno)** | **610 Satıştan İadeler** hesabı |
| 600'e etkisi | Ters çevrilir (hiç olmamış gibi) | **Ters çevrilmez** — brüt satış korunur |
| Gelir tablosu | Brüt satışta görünmez | Brüt satışta görünür, altında indirim olarak düşülür |
| Karşı belge | Yok (belge iptal) | Alıcı **iade faturası** keser (KDV mükellefiyse) |

**Neden 600 ters çevrilmez?** MSUGT gelir tablosu formatı şunu ister:

```
600 Brüt Satışlar
(−) 610 Satıştan İadeler
(−) 611 Satış İskontoları
(−) 612 Diğer İndirimler
= NET SATIŞLAR
```

Brüt satışı azaltmak **bilgi kaybıdır**. Denetim açısından da kritik: **iade oranı** (610/600) bir
risk göstergesidir. Dönem sonunda şişirilip ertesi dönem iade edilen hasılat (kanal doldurma /
channel stuffing) ancak 610 ayrı izlenirse görülür. 600'ü ters çevirirsek bu iz kaybolur.

---

## 1. Satış faturası iptali (mal teslim edilmemiş)

Kayıt yapılmışsa ters kayıtla kapatılır. **Kayıt silinmez** (VUK 219, TTK 65).

Asıl kayıt:
| Hesap | Borç | Alacak |
|---|---:|---:|
| 120 Alıcılar | 12.000 | |
| 600 Yurtiçi Satışlar | | 10.000 |
| 391 Hesaplanan KDV | | 2.000 |

İptal (storno — borç/alacak yer değiştirir):
| Hesap | Borç | Alacak |
|---|---:|---:|
| 600 Yurtiçi Satışlar | 10.000 | |
| 391 Hesaplanan KDV | 2.000 | |
| 120 Alıcılar | | 12.000 |

Net etki sıfır; iki kayıt da defterde durur. Hesaplanan KDV doğmamış olur.

**e-Belge tarafı:** e-Arşiv faturası **8 gün** içinde GİB portalından iptal edilir. e-Fatura'da
**ticari senaryoda** alıcı 8 gün içinde **RET** yanıtı verir; **temel senaryoda** ret mekanizması
yoktur, iptal talebi/harici mutabakat gerekir. Bu yüzden `RED` ve `IPTAL` durumları muhasebede
**aynı sonucu** doğurur: fatura hüküm doğurmaz.

> **DOĞRULANMADI:** 8 günlük sürenin 2026 itibarıyla güncelliği ve e-Fatura iptal portalının
> kapsamı tebliğden teyit edilmeli.

---

## 2. Satıştan iade (mal teslim edilmiş, geri geldi)

Satıcı tarafında, iade faturası alındığında:

| Hesap | Borç | Alacak |
|---|---:|---:|
| **610 Satıştan İadeler (-)** | matrah | |
| 191 İndirilecek KDV | KDV | |
| 120 Alıcılar (veya 100/102) | | toplam |

191 borçlanır çünkü iade faturası bizim için **alış belgesi** gibidir; KDVK 35 gereği matrah
değiştiğinden KDV düzeltilir.

Stok geri döner (aralıksız envanterde eş zamanlı):
| Hesap | Borç | Alacak |
|---|---:|---:|
| 153 Ticari Mallar | maliyet | |
| 621 Satılan Ticari Mallar Maliyeti | | maliyet |

**Kısmi iade** aynı mantıkla, iade edilen tutar üzerinden yapılır.

---

## 3. Alıştan iade (biz satıcıya mal iade ettik)

| Hesap | Borç | Alacak |
|---|---:|---:|
| 320 Satıcılar | toplam | |
| 153 Ticari Mallar | | matrah |
| 191 İndirilecek KDV | | KDV |

191 **alacaklanır** — indirilecek KDV azalır. (Not: MSUGT'de 610'un alış karşılığı bir hesap yoktur;
alıştan iade doğrudan stok ve KDV'den düşülür.)

---

## 4. Satış iskontosu (611) ve diğer indirimler (612)

Fatura **üzerinde** gösterilen iskonto matrahı zaten azaltır, ayrı kayıt gerekmez.
Fatura sonrası yapılan (ciro primi, erken ödeme iskontosu) iskonto **611**'e yazılır:

| Hesap | Borç | Alacak |
|---|---:|---:|
| 611 Satış İskontoları (-) | matrah | |
| 191 İndirilecek KDV | KDV | |
| 120 Alıcılar | | toplam |

---

## 5. KDV oranları ve istisna

| Oran | Nerede |
|---|---|
| %20 | genel oran |
| %10 | gıda, tekstil, ilaç vb. indirimli |
| %1 | temel gıda, gazete/dergi vb. |
| %0 / istisna | **ihracat (601 Yurtdışı Satışlar)**, KDVK 11/12 |

**İhracat kaydı:** 120 (veya 601 üzerinden) matrah kadar; **391 doğmaz**.
e-Belgede KDV = 0 ise **KDV muafiyet/istisna sebep kodu ZORUNLUDUR** (UBL-TR `TaxExemptionReason`).

---

## 6. Tevkifatlı satış (KDVK 9 — sorumlu sıfatıyla)

Belirli hizmetlerde KDV'nin bir kısmını alıcı sorumlu sıfatıyla beyan eder.
Örnek: 1.000 matrah, %20 KDV = 200, tevkifat oranı 5/10 → 100 satıcıda, 100 alıcıda.

Satıcı kaydı:
| Hesap | Borç | Alacak |
|---|---:|---:|
| 120 Alıcılar | 1.100 | |
| 600 Yurtiçi Satışlar | | 1.000 |
| 391 Hesaplanan KDV (tevkifat dışı pay) | | 100 |

Alıcı, tevkif ettiği 100'ü **2 No.lu KDV beyannamesi** ile beyan eder.

---

## 7. Tevsik zorunluluğu (nakit tahsilat sınırı)

VUK mükerrer 257 / 459 sıra no.lu VUK Genel Tebliği: belirli haddi aşan tahsilat ve ödemeler
**banka/PTT/finans kurumu** aracılığıyla yapılmak zorundadır. Nakit yapılırsa **özel usulsüzlük
cezası** doğar (hem satıcıya hem alıcıya).

Bu, muhasebe kaydını değil **uyum bulgusunu** etkiler: haddi aşan nakit tahsilat bir denetim
bulgusudur ve programda uyarı üretmelidir.

> **DOĞRULANMADI:** 2026 haddi. `data/vergi-parametreleri.json` içinde sürümlü tutulmalı,
> koda gömülmemeli. Şu an motorda **7.000 TL** varsayımıyla bulgu üretiliyor; tebliğden teyit şart.

---

## 8. Motorun uyması gereken özet kurallar

1. `edurum ∈ {IPTAL, RED}` olan fatura **cari bakiye doğurmaz**; tahakkuk + storno çifti yazılır.
2. İptal edilmiş faturaya **tahsilat tahsis edilemez** (ortada borç yok).
3. `TASLAK` fatura **hiç kayıt üretmez** — henüz belge değildir.
4. İade, 600'ü ters çevirmez; **610** kullanılır ve **191 borçlanır**.
5. İade tutarı fatura toplamını aşamaz.
6. Cari açık bakiye = toplam − iade − iskonto − tahsilat.
7. KDV = 0 ise istisna sebep kodu zorunlu.
8. Haddi aşan nakit tahsilat → uyum bulgusu (kayıt geçerli, ama bulgulanır).

---

# EK — ÖTV, KDV ve KKEG ilişkisi (25.07.2026)

## 1. ÖTV → KDV zinciri: **ÖTV, KDV matrahına DAHİLDİR**

Dayanak: **ÖTVK 11** ve **KDVK 24/b**. Hesap sırası bozulursa KDV eksik hesaplanır.

```
satış bedeli
  → ÖTV = bedel × ÖTV oranı   (veya maktu: litre/kg birim vergisi)
  → KDV MATRAHI = bedel + ÖTV      ← kritik nokta
  → KDV = (bedel + ÖTV) × KDV oranı
  → toplam
```

**Yaygın hata:** KDV'yi yalnız bedel üzerinden hesaplamak. 100.000 bedel · %50 ÖTV · %20 KDV'de
doğru KDV **30.000**'dir (150.000 matrahtan), 20.000 değil — 10.000 eksik beyan doğar.
Motorda bu bir testle kilitlendi (`otv_kdv_matrahina_dahildir`).

### ÖTV listeleri ve vergileme yöntemi
| Liste | Kapsam | Yöntem |
|---|---|---|
| I | Petrol ürünleri, doğalgaz, madeni yağ | **maktu** (litre/kg) |
| II | Kara/hava/deniz taşıtları | **nispi** (matrah dilimli) |
| III | Alkollü içecek, tütün, kolalı gazoz | **karma** (nispi + asgari maktu) |
| IV | Dayanıklı tüketim, lüks ürün | nispi |

> **ORAN MOTORDA YOK.** ÖTV oranları ürün bazında belirlenir ve sık değişir; motor oranı
> kullanıcıdan alır, tahmin yürütmez. (Aynı ilke tevkifat oranlarında da uygulandı.)

### Alış tarafı
**ÖTV, alışta MALİYETE girer** (VUK 262) — indirilecek bir vergi değildir; KDV gibi 191'e yazılmaz.
Yani alınan malın stok/duran varlık maliyeti ÖTV dahil tutarla oluşur.

## 2. İndirilemeyen KDV (KDVK 30)

Bazı KDV indirilemez: zayi olan mal, binek oto alış KDV'si (istisnalar hariç), KKEG'e ilişkin
harcamaların KDV'si. İndirilemeyen KDV **gider veya maliyete** yazılır — 191'de bırakılamaz.
Bu tutarın bir kısmı ayrıca **KKEG** olabilir.

## 3. KKEG — Kanunen Kabul Edilmeyen Gider (KVK 11 / GVK 41)

**Nasıl izlenir (yaygın uygulama):**
- Gider, ilgili gider hesabına **normal şekilde** yazılır (770, 760, 632…).
- KKEG olan kısım **ayrı muavinde** izlenir (ör. `770.KKEG`) veya **nazım hesapta** (950/951).
- MSUGT'de "KKEG hesabı" diye ayrı bir ana hesap **yoktur** — muavin/nazım ile izlenir.

**Beyanda ne olur:**
KKEG, ticari kârdan düşülmüştür ama **mali kâra geri EKLENİR**. Geçici vergi ve kurumlar vergisi
beyannamesinde "Kanunen Kabul Edilmeyen Giderler" satırında matraha eklenir.
**Yevmiyede ters kayıt YAPILMAZ** — defter ticari kârı gösterir, beyanname mali kârı.

Bu, "beyanname doktrini"nin bir örneği: beyan düzeltmesi deftere işlemez, beyannamede yapılır.

**Tipik KKEG kalemleri:** binek oto gider/amortisman kısıtını aşan kısım · indirilemeyen KDV
(KDVK 30) · vergi cezası, gecikme zammı, para cezaları · kanuna aykırı bağış ve yardımlar ·
örtülü sermaye faizi ve transfer fiyatlandırması farkı.

> **DOĞRULANMADI:** Binek oto gider/amortisman kısıt tutarları yıllık tebliğle değişir;
> `vergi-parametreleri.json`'a alınıp sürümlenmeli. Şu an motorda bu kısıt YOK.

## 4. Programda durum

| Konu | Durum |
|---|---|
| ÖTV → KDV zinciri hesabı | ✅ `POST /api/hesapla/otv` + 3 test |
| ÖTV listeleri / yöntem bilgisi | ✅ parametre dosyasında |
| ÖTV oranları | ⬜ bilerek yok — kullanıcı girer |
| ÖTV'nin maliyete girmesi (VUK 262) | ⬜ fiş şablonuna bağlanmadı |
| İndirilemeyen KDV → gider/maliyet | ⬜ |
| KKEG muavin/nazım izleme | ⬜ |
| KKEG'in beyannameye eklenmesi | 🟡 geçici vergide manuel satır var, kod kataloğu yok |
| Binek oto kısıtları | ⬜ parametre yok |
