# Gider Hesapları (7'li Sınıf) ve KKEG — Muhasebe Kuralı

> Kaynak: MSUGT · GVK 40/41/68 · KVK 10/11 · VUK 262/313 · KDVK 30.
> Veri: `data/gider-katalogu.json` (33 gider türü, 9 yansıtma kuralı) · Motor: `crates/api/src/gider.rs`
> Uçlar: `GET /api/gider/katalog?sektor=X` · `POST /api/gider/oner`

## 1. Üç seçenek — hangi hesap grubunu kullanacağız?

Sektörün `maliyet_secenegi` alanı belirler (`data/sektorler.json`):

| Seçenek | Kim | Hesaplar |
|---|---|---|
| **7/A** gider yeri esaslı | Üretim, inşaat, enerji, tarım | 710 · 720 · 730 · 740 · 750 · 760 · 770 · 780 |
| **7/B** gider çeşidi esaslı | Hizmet, lojistik, gıda/konaklama (küçük işletme) | 790–797, yansıtma 798 → 799 |
| **yok** | Ticaret, e-ticaret, serbest meslek | 7'li kullanılmaz — gider **doğrudan 6xx**'e |

> **DOĞRULANMADI:** 7/A zorunluluk haddi yıllık tebliğle belirlenir; parametre dosyasına alınmalı.

**"yok" seçeneğinde hesap nasıl bulunur?** 7/A hesabının **yansıtma hedefi** kullanılır:
760→**631**, 770→**632**, 780→**660**, 750→**630**. Ticarette mal alışı gider değil **stok**tur
(153), satılınca 621'e geçer — motor bunu uyarıyla söyler.

## 2. Yansıtma zinciri (dönem sonu)

| Gider | Yansıtma | Hedef |
|---|---|---|
| 710 · 720 · 730 | 711 · 721 · 731 | **151** Yarı mamul → 152 → 620 |
| 740 Hizmet üretim maliyeti | 741 | **622** Satılan hizmet maliyeti |
| 750 Ar-Ge | 751 | **630** |
| 760 Pazarlama satış dağıtım | 761 | **631** |
| 770 Genel yönetim | 771 | **632** |
| 780 Finansman | 781 | **660** |
| 790–797 (7/B) | 798 | **799** → oradan 6xx/15x |

## 3. KKEG — Kanunen Kabul Edilmeyen Gider

**MSUGT'de ayrı bir "KKEG hesabı" YOKTUR.** Gider normal hesabına yazılır, KKEG kısmı
**`.99` muavininde** izlenir (ör. `770.99`, `793.99`).

Kritik: **KKEG beyannamede matraha GERİ EKLENİR; yevmiyede ters kayıt YAPILMAZ.**
Defter ticari kârı gösterir, beyanname mali kârı. (Beyanname doktrini.)

### Üç KKEG davranışı

| Tür | Nasıl | Örnek |
|---|---|---|
| **TAMAMI** | Giderin tümü KKEG | Vergi cezası, gecikme zammı (KVK 11/1-d) · ÖİV (6802/39) · alkol-tütün reklamı (GVK 41/7) |
| **ORANSAL** | Sabit oranda | Binek oto yakıt/bakım: **%70 indirilebilir, %30 KKEG** (GVK 40/5) |
| **HADDİ AŞAN** | Haddi aşan kısım | Binek oto kirası (GVK 40/1), amortismanı (GVK 40/7) |
| **KISMEN** | Olaya göre değişir — motor **tahmin yürütmez** | Finansman gider kısıtlaması, bağış limiti, indirilemeyen KDV |

### Binek otomobil kısıtının SEKTÖREL istisnası
GVK 40/1 parantez içi hükmü: **faaliyeti binek otomobil kiralama veya çeşitli şekillerde
işletme olanlarda kısıt UYGULANMAZ.** Araç kiralama şirketi, taksi işletmesi, sürücü kursu
tam gider yazar. Motor bunu `arac_isletmecisi` bayrağıyla ayırt eder.

## 4. Sektörel gider örnekleri

| Sektör | Ayırt edici gider | Hesap |
|---|---|---|
| Üretim (URT) | Hammadde, direkt işçilik, fabrika GÜG | 710 · 720 · 730 |
| Hizmet (HIZ) | Danışman/yazılımcı ücreti → hizmet üretim maliyeti | 740 (7/A) · 791/793 (7/B) |
| E-ticaret (ETIC) | **Platform komisyonu**, kargo | 631 (maliyet yok) |
| İnşaat (INS) | Yıllara yaygın inşaat maliyeti — **gider yazılmaz**, 17x'te bekletilir | 170–178 |
| Serbest meslek (SRB) | Mesleki gider — **tahsil esası**, ödenmemiş gider yazılamaz (GVK 68) | 770/793 |
| Lojistik (LOJ) | Yakıt, araç bakım — **ticari araçta binek kısıtı YOK** | 740 · 793 |

## 5. Motorun garantileri (testle korunuyor)

1. `indirilebilir + KKEG = gider tutarı` — her zaman, her türde. Para kaybolmaz.
2. Fiş satırlarının toplamı da gider tutarını verir.
3. Had bilinmiyorsa **KKEG uydurulmaz** — uyarı verilir (tevkifat ve ÖTV oranlarındaki ilkeyle aynı).
4. Sektöre uygun olmayan gider türü uyarı üretir.
5. Maliyet muhasebesi olmayan sektörde üretim hesabı (710/720/730) önerilmez.

## 6. Eksikler

- Binek oto hadleri (kira, ÖTV+KDV, amortisman) `vergi-parametreleri.json`'a alınmalı — **şu an yok**.
- Finansman gider kısıtlaması oranı doğrulanmadı.
- 7/A zorunluluk haddi doğrulanmadı.
- Katalog 33 gider türü kapsıyor; nadir kalemler (5746 Ar-Ge indirimi ayrıntısı, yatırım teşviki,
  yenileme fonu) eklenmedi.
- KKEG muavinleri henüz **otomatik açılmıyor** — `.99` kodu öneriliyor ama hesap planında yaratılmıyor.
