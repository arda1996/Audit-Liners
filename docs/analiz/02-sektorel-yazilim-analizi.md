Bu bir analitik/yazılım stratejisi görevi; skill gerektirmiyor. Doğrudan analizi üretiyorum.

# Türkiye Muhasebe Yazılımları — Sektörel Analiz ve Audit-Liners Konumlandırması

## 1. Başlıca Yazılımlar: Ölçek ve Sektör Uygunluğu

| Yazılım | Ölçek | Dağıtım | Güçlü Olduğu Sektör/Segment | TMS/TFRS Desteği |
|---|---|---|---|---|
| **Logo (Tiger/Go/j-Platform)** | KOBİ → Kurumsal | On-prem + Bulut | Ticaret, Üretim (MRP), dağıtım | Kısmi (Tiger 3 Enterprise'da sınırlı) |
| **Mikro (Fly/Jump/Run)** | Küçük → Orta | On-prem ağırlıklı | Ticaret, perakende, KOBİ üretim | Zayıf/yok |
| **Netsis (Logo)** | Orta → Kurumsal | On-prem + Bulut | Üretim (gelişmiş MRP-II), ihracat, çok şirketli | Orta (konsolidasyon modülü) |
| **Paraşüt** | Mikro/Küçük, freelancer | SaaS (bulut) | E-ticaret, hizmet, serbest meslek, ön muhasebe | Yok (VUK/e-belge odaklı) |
| **Luca / Luca-Net (TÜRMOB)** | SMMM büroları | Bulut | **Mükellef bürosu / çoklu şirket beyanname** | Yok (VUK/beyanname) |
| **Zirve** | Küçük → Orta, SMMM | On-prem + Bulut | SMMM bürosu, ticaret, tarım (müstahsil) | Zayıf |
| **Dia** | KOBİ | Full SaaS | Ticaret, üretim, hizmet, çok şubeli | Zayıf |
| **Akınsoft (Wolvox)** | Mikro/Küçük | On-prem | Sektörel dikey paketler (kafe, oto, kuaför) | Yok |
| **ETA (SQL/V.8)** | Küçük → Orta | On-prem | Ticaret, SMMM bürosu | Zayıf |
| **Uyumsoft** | Orta → Kurumsal | Bulut + on-prem | e-Dönüşüm entegratörü, üretim, kamu | Orta |
| **BizimHesap** | Mikro | SaaS | Esnaf, ön muhasebe, basit fatura | Yok |
| **Nilvera** | Tüm ölçekler (entegratör) | SaaS/API | **e-Fatura/e-Arşiv/e-İrsaliye entegrasyonu (altyapı)** | Yok — belge katmanı |

**Ana gözlem:** Türkiye pazarının neredeyse tamamı **VUK/vergi matrahı + e-Dönüşüm** ekseninde. Hiçbir yaygın paket **TMS/TFRS ölçümünü çekirdeğe** almıyor; ertelenmiş vergi, dual defter (VUK + TMS köprüsü) ve denetim iş akışları ya yok ya "kurumsal ek modül" olarak zayıf.

## 2. Sektör Bazında Gereksinim & Uygun Yazılım Matrisi

| Sektör / Segment | Kritik Özellik & Standartlar | Uygun Mevcut Yazılım(lar) | Pazar Boşluğu |
|---|---|---|---|
| **Ticaret** | Stok (FIFO/ort. ma., TMS 2), NGD/değer düşüklüğü 158, e-fatura, KDV | Logo, Mikro, Dia, ETA | TMS 2 NGD + değer düşüklüğü karşılığı otomasyonu yok |
| **Üretim** | Maliyetlendirme (7/A-7/B), normal kapasite GÜG dağıtımı, komponent amortisman (TMS 16), atıl kapasite gideri | **Netsis, Logo Tiger** (MRP güçlü) | GÜG'ün "normal kapasite" ayrımı + TMS maliyet motoru zayıf |
| **Hizmet / Ön muhasebe** | Basit cari-fatura, TFRS 15, SMM makbuzu | Paraşüt, BizimHesap, Dia | Yeterli — düşük TMS ihtiyacı |
| **İnşaat (yıllara yaygın)** | 170/350 hakediş, TMS 23 borçlanma maliyeti aktifleştirme, onerous sözleşme (TMS 37), TMS 40 | Netsis, Logo (proje modülü) | Özellikli varlık aktifleştirme oranı + değer düşüklüğü otomasyonu yok |
| **Tarım (TMS 41)** | Canlı varlık GUD-SM, taşıyıcı bitki/hayvan ayrımı, hasat→stok geçişi, dual VUK-maliyet | **Hiçbiri** (Zirve sadece müstahsil %2) | **Tam boşluk** — GUD değerleme + dual defter |
| **Finansal kuruluş** | TFRS 9 sınıflama, TMS 32/39 hedge, nakit akış netleştirme (TMS 7), BDDK/SPK raporlama | Kurumsal ERP + özel (banka içi) | KOBİ ölçeğinde erişilebilir çözüm yok |
| **Serbest meslek / SMMM-büro** | Çoklu mükellef, toplu beyanname, e-defter berat, bordro/SGK | **Luca, Zirve, ETA, Paraşüt** | Beyanname güçlü, TMS/denetim köprüsü yok |
| **Bağımsız denetim** | KGK/BDS uyumu, ilişkili taraf (TMS 24), örneklem, çalışma kağıtları, rotasyon/bağımsızlık kontrolü, denetim izi | **Hiçbir yerli yaygın paket** (Excel + import araçları hakim) | **Büyük boşluk** — BDS iş akışı + otomatik testler |

## 3. Audit-Liners Konumlandırması

### Doldurabileceğimiz Boşluk
Pazardaki tüm oyuncular **VUK/e-belge** katmanında sıkışmış. Bulgu setinin tamamında tekrar eden çekirdek ihtiyaç: **VUK envanteri ile TMS ölçüm değerinin AYRI saklanması + ertelenmiş vergi (TMS 12) köprüsü**. Bunu native veri modeline koyan yerli çözüm yok.

**Ayrıştırıcı çekirdek: Dual-defter + Ertelenmiş Vergi Motoru**
- Her hesap/varlık için `vuk_degeri` ve `tms_degeri` paralel alanları → geçici/kalıcı fark otomatik üretimi (TMS 2, 12, 16, 19, 36, 37, 38, 40, 41'in tümü bunu şart koşuyor).
- Otomatik KKEG ↔ istisna ↔ ertelenmiş vergi ayrımıyla **ticari kâr → mali kâr köprüsü** (KV/GV örnek çalışmalarındaki matrah akışını motorlaştırma).

### Rekabet Edebileceğimiz Segmentler (öncelik sırasıyla)
1. **Bağımsız denetim (mavi okyanus):** BDS/KGK iş akışı, ilişkili taraf (TMS 24) tespiti, bağımsızlık/rotasyon kontrolü, otomatik analitik testler (reeskont simetrisi, anormal yevmiye, hasılata dayalı amortisman uyarısı), değişmez denetim izi. Yerli yaygın rakip **yok**.
2. **TFRS raporlayan KOBİ + SMMM danışmanlık:** VUK defterinden TMS finansal tablo + dipnot + ertelenmiş vergi üretimi. Luca/Zirve'nin bittiği yerde başlıyoruz.
3. **Tarım (TMS 41):** Niş ama tamamen boş; GUD-SM değerleme + dual defter ile tek yerli çözüm olma fırsatı.

### Bizi Ayrıştıran Özellikler
- **Standart-farkındalıklı kural motoru:** LIFO'yu engelleyen, sınıf-bazı toplu NGD indirgemesini bloklayan, şerefiye değer düşüklüğü iptalini yasaklayan, reeskont simetrisi zorlayan, hasılata dayalı amortismanı uyaran gömülü validasyonlar (bulgulardaki "yapılmaması gerekenler" doğrudan kural setine dönüşür).
- **Otomatik dipnot + ertelenmiş vergi mutabakatı** (rakiplerde manuel/Excel).
- **Denetim izi + AI anomali tespiti** (mesleki etik dosyasındaki hesap verebilirlik/değiştirilemezlik ilkesiyle uyumlu).

**Stratejik sonuç:** ERP/ön muhasebe pazarında (Logo/Mikro/Paraşüt) baştan rekabet etme; onların e-belge/VUK verisini **tüketen üst katman** ol — "TMS/TFRS + Ertelenmiş Vergi + Bağımsız Denetim" katmanı. Ayrıştırıcı = **dual-defter köprüsü + standart-farkındalıklı denetim kuralları**.