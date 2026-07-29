# Audit-Liners — Proje Durum Raporu

**Tarih:** 14 Temmuz 2026 · **Statü:** Çalışan prototip, mimari oturmuş, kalıcılık öncesi

---

## 1. Bir cümlede proje

**SMMM/Mali Müşavir için, VUK defterinden bağımsız denetim raporuna kadar giden tek program.**
Muhasebe kaydı ile denetim kanıtı arasındaki zinciri kopmadan taşımayı hedefler — Excel'i ortadan
kaldırmadan değil, **Excel'in yapamadığını yaparak**: kaynağı belli, izi tutulan, savunulabilir sayı.

---

## 2. Kurucu doktrinler (her kararı bunlar belirliyor)

| Doktrin | Ne demek | Neden |
|---|---|---|
| **İki eksen** | Vergi raporlaması (VUK) ≠ Bağımsız denetim (TFRS+BDS). İkisi ayrı katman, çapraz bağlı. | Aynı defterden iki farklı gerçek üretilir; karıştırılırsa ikisi de yanlış olur |
| **Deftere sızmaz** | Denetim/beyanname düzeltmeleri VUK defterine ASLA işlenmez; ayrı katmanda yaşar | VUK 217: kesin fiş değişmez. Düzeltme = ayrı kayıt, izli |
| **Manuel müdahale serbest ama izli** | Denetçi her değeri değiştirebilir — ama kim/ne zaman/eski→yeni/gerekçe kaydedilir | BDS 230: deneyimli denetçi kağıdı tek başına okuyup anlayabilmeli |
| **Para = kuruş (i64)** | Float yasak. Çarpma/bölme yalnız açık yuvarlama politikasıyla | Excel'in "penny problem"i denetimde savunulamaz |
| **Dayanak zorunlu** | Her kayıt bir belgeye/hesaplamaya/rapora bağlanır | Dayanaksız kayıt denetim kanıtı değildir |
| **Kanıt Güven Skoru** | Program "doğrudur" demez; "şu kanıtlarla ulaştım, güvenim şu, şurası zayıf" der | Karar denetçinindir — program muhakemeyi besler |

---

## 3. Bugün ne çalışıyor (ölçülmüş)

**Kod:** domain 1.858 satır (15 modül) · API 3.744 satır (**66 uç**) · frontend 3.024 satır (9 bileşen)
· 16 veri kataloğu · 37 tasarım/araştırma dokümanı · **59 test yeşil** · 26 tamamlanmış iş dosyası.

### 3.1 Muhasebe çekirdeği ✅
- Çift taraflı kayıt, denge kontrolü, yaprak hesap zorunluluğu, kronoloji koruması, mükerrer belge reddi
- Ayrı seri fiş no + müteselsil yevmiye no (VUK) · **VUK 217 iptal** (kesin fiş değişmez, ters kayıt)
- TDHP 262 hesap + **sınırsız muavin kırılımı** (`253.01.001` — 3. seviye çalışıyor)
- Defterler: mizan · yevmiye · kebir · muavin (+ TXT dışa aktarım)
- Mali tablolar: bilanço (aktif=pasif) · gelir tablosu · dönem kapanış/açılış virmanı
- 7/A yansıtma (760→761→631, 770→771→632, 780→781→660)
- **Fixture: 19.702 fiş** (e-ticaret 2026, açılış bilançosu + yıl içi + dönem sonu — dengeli)

### 3.2 Vergi ekseni ✅
- KDV: aylık 191↔391 mahsubu · **KDV1 beyanname taslağı** (oran kırılımlı, iade-duyarlı)
- Geçici vergi hesap kağıdı (kümülatif; ticari kâr + KKEG − istisna)
- 10 vergi kanunu parametre kataloğu (2026 değerleri, doğrulanmış)
- **Beyanname doktrini:** muhasebe kaydı = dayanak; GİB kılavuzu = format otoritesi; manuel müdahale
  katmanı deftere işlemez, izli

### 3.3 Denetim ekseni ✅ (bu turun ağırlığı)
- **7 sektör × 32 çalışma programı** (BDS/TMS dayanaklı) + çalışma kağıdı motoru
- **UFRS WorkSheet** — projenin kalbi:
  - **WTB (Working Trial Balance)**: Big-4 değer zinciri `VUK → AJE → Düzeltilmiş → RJE → Devir(CF) → TFRS`,
    sınıf bölümleri + ara toplamlar + sıfır kontrol satırları + **kâr köprüsü**; her tutar delinebilir
  - **18 çalışma** (TMS 16/19/21/23/36/37/40, TFRS 9/15/16, TMS 12 ertelenmiş vergi çatısı…),
    sektöre göre süzülür (üretim 7/A, inşaat YYİO)
  - **Hesaplama motoru** (4 çalışma): Reeskont (alt hesap PV) · ECL (kova matrisi) · Kıdem (aktüerya) ·
    NRV — parametre gir → hesapla → **ara tablo (denetim izi)** → **önerilen kayıt** → forma aktar
  - **Kayıt disiplini:** dayanak + değerleme yöntemi + "neyi baz aldı" + denetçi notu **ZORUNLU**;
    hazırlayan/tarih/dönem sistem atar; silme yok (vazgeç = iz kalır)
  - **Kümüle devir:** dönem kesinleşir → sonraki dönem taşı-bırak CF'leri otomatik (P/L → 570)
- **Duran varlık envanteri (37 varlık):** üretim hattı = 10 bileşen (CNC torna Mazak QT-250MY,
  seri no…), binek = 6 araç (plaka) + tamamlayıcılar. **Kayıt satırları kimlik taşıyor.**

### 3.4 Yetki ve çoklu mükellef ✅
- Giriş (kayıt yok — admin açar) · rol × **departman** (görünürlük) × **kademe** (işlem + onay eşiği)
- SORUMLU 500k TL'ye kadar kesinleştirir; iptal yalnız MÜDÜR/YÖNETİCİ; NAV departmana göre süzülür
- Çoklu mükellef: "aktif çalışma seti + arşiv swap" (45 handler değişmeden)

---

## 4. Ne eksik (dürüst envanter)

| Eksik | Etkisi | Öncelik |
|---|---|---|
| **Kalıcılık yok** | Her şey bellekte — sunucu ölünce veri gider | 🔴 En kritik |
| **Taslak CRUD yok** | Kayıt anında kesinleşiyor; maker≠checker (tam SoD) kurulamıyor | 🔴 |
| Kalan 14 çalışmanın hesap motoru | GUD/amortisman/kiralama elle giriliyor | 🟡 |
| WS-EV otomatik derleme | Ertelenmiş vergi çatısı elle besleniyor | 🟡 |
| Değerleme veri katmanı (D1-D9) | Parametreler elle; KGS yok | 🟡 uzun vade |
| Frontend tasarım sistemi | Tutarsız; Y4/Y5 bekliyor | 🟢 |
| Stok miktar takibi | Tutar-bazlı; VUK envanteri miktar ister | 🟢 |
| e-Dönüşüm (entegratör) | Fatura modülü parkta | 🟢 |

---

## 5. Nereye gidiyoruz — üç ufuk

### Yakın (haftalar): **temeli sağlamlaştır**
1. **Kalıcılık** (gömülü Postgres + `db` crate, hexagonal) — mükellef başına şema/DB
2. **Taslak CRUD + kullanıcı/zaman damgası** → tam SoD (kaydı giren ≠ onaylayan)
3. Kalan çalışmaların hesap motorları (önce **WS-AMORT** — varlık envanteri hazır, VUK↔TFRS ömür farkı
   araç araç hesaplanabilir)

### Orta (aylar): **denetimi tamamla**
4. **WS-EV çatısı otomatik derlensin** (her çalışma `ev_etkisi` yayınlar → ertelenmiş vergi kendi kendini kurar)
5. TFRS mali tablo üretimi + dipnot yayın alanları (her çalışma dipnotunu besler)
6. **Değerleme veri mimarisi D1-D3**: parametre kartı (KGS) → KGS motoru → **TCMB EVDS bağlantısı**
   (resmî API — kur değerlemesi elle girilmekten kurtulur)
7. Çalışma kağıdı hesap motoru (formül dili, bağımlılık grafiği — Excel'in yerine geçen katman)

### Uzak (yıl): **kanıt ekonomisi**
8. Emsal kartı + çelişki raporu + duyarlılık analizi (KGS < eşik → zorunlu)
9. İskonto oranı koridoru (TCMB / Hazine / Damodaran / fiili borçlanma — tek doğru yok)
10. KAP dipnot madenciliği (halka açık şirketlerin varsayımları = kalibrasyon hazinesi)
11. Tauri paketleme (masaüstü) + e-dönüşüm

---

## 6. Nasıl geliştiriyoruz (çalışma yöntemi)

**Kullanıcı ilkesi:** *"Goal komutuyla çalışmıyorum — her noktayı birlikte yapmak istiyorum."*

- **Küçük, tek başına değerli adımlar.** Tek seferde büyük yapı kurmuyoruz; kurarsak her yerini
  tekrar yapmamız gerekir.
- **Önce araştır, sonra kodla.** Big-4 formatı, BDS metodolojisi, OSS motorlar — hepsi ajanlarla
  araştırıldı, lisansları adversarial doğrulandı, sonra uygulandı.
- **Veri-güdümlü, kod-değişmez.** Sektör/şablon/departman/kademe/çalışma/parametre → hepsi JSON.
  Yeni sektör = yeni kod değil, yeni satır.
- **Her iş bittiğinde:** completion dokümanı + kullanım kılavuzu güncellemesi + test.
- **Dürüstlük kuralı:** Çalışmayan şeye "çalışıyor" demeyiz. Sınırları (basit aktüeryal model,
  ilan≠satış fiyatı, scraping yasal riski) açıkça yazarız.

---

## 7. Neden bu proje zor — ve neden değerli

Buradaki çalışmalar **doğrudan bağımsız denetçi raporunun unsurlarıdır.** Bir GUD kaydı yanlışsa
bilanço yanlıştır; bilanço yanlışsa denetçi görüşü yanlıştır; ve sorumluluk — KGK denetiminde,
mahkemede — **denetçinin üzerindedir.**

Bu yüzden her satır "çalışıyor" olmaktan fazlasını gerektiriyor: **savunulabilir** olmalı.

Ve şu an bu iş özellikle kritik: **enflasyon muhasebesi uygulanmıyor.** 2019'da 150.000 TL'ye alınan
depo binası defterde hâlâ 150.000 TL — rakam *doğru* ama *gerçek değil*. Bu boşluğu kapatmak, gerçeğe
uygun değeri piyasanın fiyatlamasına dayandırmak, tam olarak bağımsız denetimin işidir.

Program bu işi yapmaz — **denetçi yapar.** Program onun muhakemesini besler, hesabını yapar,
izini tutar, ve zayıf noktada bayrağı kaldırır.

---

*İlgili: [ufrs-worksheet.md](tasarim/ufrs-worksheet.md) · [degerleme-veri-mimarisi.md](tasarim/degerleme-veri-mimarisi.md) ·
[calisma-kagitlari-hesap-motoru.md](tasarim/calisma-kagitlari-hesap-motoru.md) · [KULLANIM-KLAVUZU-BACKEND.md](KULLANIM-KLAVUZU-BACKEND.md) · [IS-LISTESI.md](IS-LISTESI.md)*
