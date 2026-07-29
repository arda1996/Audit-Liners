Bu döküman setinin gerektirdiği frontend ihtiyaçlarını, mevcut Audit-Liners ekranlarıyla çakışmaları da işaretleyerek önceliklendirilmiş TO-DO listesi olarak çıkarıyorum.

# Audit-Liners — Frontend TO-DO (Dökümanlardan Türetilmiş)

Faz tanımları: **F1** = mevcut VUK/TDHP defterini kullanılabilir SMMM ürünü yapan çekirdek; **F2** = vergi/beyanname + dönem sonu değerleme; **F3** = TMS/denetim/ertelenmiş vergi + enflasyon; **F4** = sektörel + e-dönüşüm + etik/kayyımlık.

| # | Ekran/Özellik | Ne | Neden (döküman) | Öncelik | Faz |
|---|---|---|---|---|---|
| 1 | **Firma/Mükellef Profili** | Mükellef tipi (şahıs/sermaye şirketi/finans), NACE kodu + imalat bayrağı, sanayi sicil, teşvik belgesi, mali yıl, raporlama çerçevesi (VUK/TMS), halka açık bayrağı, deprem bölgesi | KV Örnek 2025, Gelir Vergisi Rehberi, SGK, TMS 27/33 (kapsam eşiği); tüm hesaplama motorlarını dallandırır | Yüksek | F1 |
| 2 | **Parametre/Oran Yönetimi (yıl-etkin)** | KV/GV oranları, GV dilimleri (ücret ayrı set), KDV oranları, asgari ücret, SGK tavan/oran, binek oto hadleri, YDO, engellilik indirimi, damga vergisi — hepsi yıl bazlı config UI | KV Örnek, GV Rehberi, Bordro, SGK (tüm oranlar "hardcode edilmemeli") | Yüksek | F1 |
| 3 | **Dönem Sonu Değerleme Sihirbazı — Amortisman** | Sabit kıymet kartı (maliyet bileşenleri, kullanıma_hazır tarih, VUK+TMS çift defter, komponent), yöntem (doğrusal/azalan/üretim), kalıntı değer, plan üretimi, KKEG aşım hesabı (binek oto) | TMS 16, VUK; Genel Muhasebe; KV Örnek (binek oto) | Yüksek | F2 |
| 4 | **Değerleme — Reeskont** | Senet/çek listesi, iç iskonto formülü (36500), alacak-borç simetri zorunluluğu kontrolü + KKEG uyarısı, dönem başı otomatik iptal kaydı, YP senette önce kur | TMS 39/VUK 281-285; Genel Muhasebe; KV Örnek | Yüksek | F2 |
| 5 | **Değerleme — Şüpheli Alacak** | 120/128 aktarımı, teminatlı kısım hariç karşılık, VUK şartı (dava/icra) vs TFRS 9 ECL ayrımı, 129/654/644 kaydı | TMS 37, Genel Muhasebe, VUK | Yüksek | F2 |
| 6 | **Değerleme — Kur** | Parasal/parasal olmayan bayrağı (hesap bazlı), kapanış kuru değerleme, sadece parasal kalemler, 646/656 kaydı, TCMB kur tablosu | TMS 21; Genel Muhasebe | Orta | F2 |
| 7 | **Değerleme — Karşılıklar** | Karşılık tablosu (garanti/dava/yeniden yapılandırma/onerous), olasılık >%50 sınıflandırıcı (karşılık/koşullu borç/açıklama), beklenen değer, iskonto (unwinding), hareket tablosu | TMS 37 | Orta | F2 |
| 8 | **KDV Beyanname Ekranı** | 191/391/190/360 takibi, indirilecek/hesaplanan/devreden, oran satır bazlı (konaklama %8/%18), indirilemez KDV→maliyet (binek oto), aylık beyanname çıktısı | Ön Büro, Genel Muhasebe, KV Örnek | Yüksek | F2 |
| 9 | **Muhtasar Beyanname** | Stopaj kalemleri (ücret GV, kira %20, kâr payı %15, serbest meslek), damga vergisi, aylık beyan | Bordro, GV Rehberi, KV Örnek | Yüksek | F2 |
| 10 | **Bordro/Ücret Ekranı** | Brütten nete (SGK %14/işsizlik %1/GV/DV), kümülatif GV matrahı, istisnalar (yemek/yol/çocuk/aile), PEK tavan taşıma, personel tipleri (emekli/part-time/yabancı/kapıcı), puantaj (fazla mesai %50/%25), zorunlu alan validasyonu | Bordrolama, SGK Son Değişiklikler | Yüksek | F2 |
| 11 | **Kurumlar Vergisi Beyanname/Matrah** | Ticari kâr→+KKEG→−istisna→−geçmiş zarar→−indirim→matrah zinciri (her satır madde referanslı); KKEG kataloğu, iştirak/taşınmaz istisna (FIFO+fon), örtülü sermaye/TF/KEYK, indirimli KV, nakit sermaye, uyumlu mükellef %5, yurtiçi asgari KV | KV Örnek 2025, Vergi Komitesi | Yüksek | F2 |
| 12 | **Gelir Vergisi Beyanname** | 7 gelir unsuru toplama, beyan sınırları (GVK 86), Geçici 67 beyan-dışı, kâr payı %50 istisna, GMSİ konut istisnası, değer artış Yİ-ÜFE endeksleme, indirimler, %5 uyumlu | Gelir Vergisi Rehberi 2025 | Orta | F2 |
| 13 | **Geçici Vergi** | Dönemsel (3/6/9/12) matrah, finansman gider kısıtlaması %10 KKEG, mahsup takibi | KV Örnek, GV Rehberi | Orta | F2 |
| 14 | **Finansal Tablolar + DİPNOTLAR** | Mevcut bilanço/gelir tablosuna DİPNOT üretici ekle: muhasebe politikaları, TMS 8/10/16/19/36/37 zorunlu açıklamaları, hareket tabloları, kâr dağıtım tablosu (yasal yedek I/II tertip) | TMS 1/8/10/16/19/24/36/37; Ön Büro (kâr dağıtım) | Yüksek | F3 |
| 15 | **Ertelenmiş Vergi Modülü (TMS 12)** | Her hesap için VUK değeri vs TMS değeri, geçici/kalıcı fark ayrımı, ertelenmiş vergi varlık/borç, iskonto YASAK, oran = ters dönme yılı, efektif oran mutabakatı | TMS 12 (ve tüm standartların vuk_tms_farkı) | Yüksek | F3 |
| 16 | **Enflasyon Düzeltmesi Ekranı** | Parasal/parasal olmayan sınıflama, Yİ-ÜFE endeks tablosu, düzeltme katsayısı, iktisap tarihi bazlı düzeltme, net parasal pozisyon K/Z, karşılaştırmalı yeniden düzeltme; VUK Geçici 37 (2025-2027 durdurma) bilgilendirmesi | TMS 29; KV Örnek (VUK Geç.37) | Orta | F3 |
| 17 | **TMS Muhasebe Politikası/Değişiklik İş Akışı** | Değişiklik türü enum (politika/tahmin/hata), geriye dönük vs ileriye yönelik motor, çok dönemli açılış bakiyesi, dipnot şablonu | TMS 8 | Düşük | F3 |
| 18 | **Dönem Sonrası Olaylar (Subsequent Events)** | Onay tarihi + olay penceresi, ADJUSTING/NON_ADJUSTING sınıflama, temettü kontrolü, going concern bayrağı, dipnot | TMS 10 | Düşük | F3 |
| 19 | **İlişkili Taraf Modülü** | Cari kartına ilişkili taraf bayrağı + 7 kategori enum, işlem/bakiye kaydı, kilit yönetici ücreti (5 kategori), dipnot üretici, KVK 13 TF köprüsü | TMS 24, KV Örnek | Orta | F3 |
| 20 | **Sektörel — İnşaat Hakediş** | 170 yıllara yaygın inşaat, hakediş bazlı, TMS 23 borçlanma maliyeti aktifleştirme (özellikli varlık), asgari işçilik (SGK) | Genel Muhasebe, TMS 23, SGK | Orta | F4 |
| 21 | **Sektörel — Tarım Canlı Varlık** | GUD−satış maliyeti değerleme, hasat→stok, taşıyıcı bitki/canlı varlık ayrımı, devlet teşviki (koşullu/koşulsuz), hareket tablosu | TMS 41, TMS 20 | Düşük | F4 |
| 22 | **Sektörel — Üretim Maliyet (7/A-7/B)** | Fonksiyon/çeşit maliyet seçimi, normal kapasite/atıl kapasite GÜG ayrımı, standart-fiili varyans, stok NGD değerlemesi | TMS 2, Genel Muhasebe | Orta | F4 |
| 23 | **Yatırım Amaçlı Gayrimenkul** | GUD vs maliyet politikası, GUD değişimi K/Z, transfer iş akışı, sınıf ayrımı (252 vs yatırım amaçlı) | TMS 40 | Düşük | F4 |
| 24 | **Nakit Akış Tablosu** | Faaliyet sınıfı (esas/yatırım/finansman) mapping, doğrudan+dolaylı yöntem, kur etkisi ayrı satır, mutabakat tie-out | TMS 7 | Orta | F3 |
| 25 | **Denetim Modülü + BDS Kontrolleri** | Analitik testler (bağımsızlık beyanı, rotasyon, çıkar çatışması, KGK bildirim akışı), anomali/hile testleri, YMM tasdik eşiği kontrolleri | Mesleki Etik, KV Örnek (YMM eşik) | Düşük | F4 |
| 26 | **Etik/Bağımsızlık İş Akışı** | RBAC gizlilik, erişim logu, müşteri kabul kontrol listesi, aynı müşteri denetim+danışmanlık engeli, veri paylaşım onay kaydı | Mesleki Etik | Düşük | F4 |
| 27 | **Kayyımlık Dosya Modülü** | Görev tipi (temsil/denetim/yönetim) + tipe göre RBAC yetki profili, mahkeme bilgisi, 6 adımlı workflow, değiştirilemez audit trail, mahkeme raporu | Kayyımlık | Düşük | F4 |
| 28 | **Emeklilik Planı Raporlaması (opsiyonel)** | Net varlık tablosu, GUD değerleme, aktüeryal yükümlülük, %5 yoğunlaşma açıklaması — sadece plan tipi mükelleflerde aktif | TMS 26 | Düşük | F4 |
| 29 | **E-Dönüşüm** | e-Fatura/e-Arşiv/e-İrsaliye/e-Defter entegrasyonu, 10 yıl arşiv, belge tipleri (müstahsil makbuzu, gider pusulası, serbest meslek makbuzu) | Genel Muhasebe, Ön Büro, GV/KV | Orta | F4 |
| 30 | **Ara Dönem Raporlama** | Çift sütun (dönemsel + YTD), efektif yıllık vergi oranı yöntemi, karşılaştırmalı | TMS 34 | Düşük | F3 |

## Mevcut Ekranlarla Çakışma / Genişletme Notları

- **Fiş Girişi (mevcut):** #4–#7 değerleme sihirbazları ürettikleri kayıtları bu fişe otomatik yazacak — sihirbaz = fiş üreteci. #10 bordro tahakkuku da fiş üretir. Yeni ekran değil, "otomatik fiş kaynağı" katmanı.
- **Hesap Planı + Muavin (mevcut):** #1 (parasal/parasal-olmayan bayrağı — TMS 21/29), #15 (VUK değeri vs TMS değeri alan çifti), #19 (ilişkili taraf bayrağı) bu ekrana **kolon eklentisi** olarak girer. Yeni ekran değil, hesap kartı genişletmesi.
- **Mizan (mevcut):** #14 dipnot ve #24 nakit akış bu veriyi tükettiği için mizan değişmez; sadece "kesin mizandan tablo üretimi" akışına #14/#24 eklenir.
- **Mali Tablolar/Bilanço-Gelir (mevcut):** #14 doğrudan bunun üstüne dipnot + kâr dağıtım tablosu ekler; #15 ertelenmiş vergi bilanço/gelir tablosuna yeni satırlar getirir. **En yüksek çakışma burada** — mevcut tablo motoru dipnot ve TMS-katmanı için genişletilmeli.
- **Resmi İşleyiş Paneli (mevcut):** #11/#12/#8/#9 beyanname ekranlarındaki "madde referansı/dayanak" gösterimi mevcut MSUGT açıklama panelinin aynı UX kalıbıyla yapılabilir — yeniden kullanılabilir bileşen.

## Önerilen Sıra
F1: #1, #2 (temel altyapı — tüm hesaplamaların önkoşulu). F2: #10, #8, #9, #11 → sonra #3, #4, #5 (SMMM'nin günlük işi: bordro+beyanname+değerleme). F3: #14, #15 (dipnot + ertelenmiş vergi — TMS'e giriş kapısı). F4: sektörel + e-dönüşüm + etik/kayyımlık.

En kritik iki bağımlılık: **#1 Firma Profili** ve **#2 Parametre Yönetimi** — bunlar olmadan hiçbir vergi/bordro ekranı doğru sonuç üretemez, o yüzden F1'de ve Yüksek.