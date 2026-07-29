# Denetime Giden Yol — sektörel denetim çalışmaları mimarisi

> "Vergiye giden yol"un ikizi. Tek veri kaynağı: **muhasebe kayıtları** (defter/mizan/muavin motorlarımız).
> İlke: denetim çalışması kodda değil **veride** tanımlanır → proaktif değişiklik = JSON düzenlemek.

## 1. Üç katman

| Katman | Nerede | Ne |
|--------|--------|-----|
| **Motor** (kod, Rust) | crates/domain | Generic tarama/analitik fonksiyonları (M1–M12). Sektör bilmez; hesap öneki + parametre alır. |
| **Program** (veri, JSON) | data/denetim-programlari.json | Sektör → çalışma listesi. Her çalışma: hangi motor + hangi hesap önekleri + eşikler + BDS referansı. |
| **Bulgu** (çıktı) | API/UI | Çalışma sonucu: etkilenen fişler/hesaplar, değer↔eşik, ciddiyet, çalışma kağıdı satırı. |

**Proaktif değişiklik mekanizması** (açıklama-kümeleri kalıbı):
- Standart programlar `data/`'da sürümlü (`surum` alanı). Kullanıcı katmanı ayrı tutulur: çalışma **kopyala-düzenle** (eşik/hesap değiştir), **kapat/aç**, **yeni çalışma ekle** (mevcut motorlarla).
- Bulgu, üretildiği çalışmanın sürümüyle saklanır (denetim izi: hangi eşikle bulundu).
- Mükellef profili (E.5) sektör kodu taşır → program otomatik seçilir; çoklu sektörde birleşim.

## 2. Motor kütüphanesi (M1–M12)

| # | Motor | Girdi | Durum |
|---|-------|-------|-------|
| M1 | Oran/trend analitiği (aylık seri, sapma eşiği) | hesap önekleri, eşik % | 🟡 var (14 oran) — seriye genellenecek |
| M2 | Yaşlandırma (120/320/196…) | önek, vade dilimleri | ⬜ |
| M3 | Benford (ilk hane dağılımı) | önek, min tutar | ⬜ |
| M4 | Mükerrer tarama (belge no / tutar+karşı+tarih) | tip | 🟡 var (belge no) — tutar bazlısı eklenecek |
| M5 | Zaman deseni (hafta sonu/gece/dönem sonu yığılma) | pencere | ⬜ |
| M6 | Yuvarlak tutar taraması | önek, yuvarlaklık, min | ⬜ |
| M7 | Doğaya ters bakiye | önek | ✅ var (ters bakiye uyarısı) |
| M8 | Karşı bacak sapması (karsi[] dışı eşleşme) | kural verisi | ⬜ (veri hazır: 83 hesap) |
| M9 | Kesme/cut-off (dönem sınırı ± N gün hareketleri) | önek, pencere | ⬜ |
| M10 | Konsantrasyon (tek cari/tedarikçi payı) | önek, eşik % | ⬜ |
| M11 | Denklik/zincir kontrolü (yansıtma, KDV, maliyet akışı) | zincir tanımı | 🟡 kapanış+KDV motoru var — kontrol modu eklenecek |
| M12 | Süreklilik/boşluk (yevmiye no, kronoloji) | — | ✅ var |

4/12 kısmen-tam hazır → denetim yolu mevcut muhasebe motorlarının üstüne kurulur.

## 3. Sektör programları (popülerlik sırası — SMMM portföyü)

1. **TIC** Ticaret (toptan/perakende) — marj analitiği, stok-SMM zinciri, cari yaşlandırma, kasa/KDV
2. **ETIC** E-ticaret — TIC + POS/aracı platform (108) mutabakatı, iade yoğunluğu, kargo gideri oranı *(fixture'ımız = test alanı)*
3. **HIZ** Hizmet — avans→hasılat dönüşümü, dönemsellik (380/480), personel gideri oranı
4. **URT** Üretim — 7/A yansıtma denkliği, 150→152→620 maliyet akışı, fire, boş kapasite (TMS 2: boş kapasite GÜG'ü stoklanamaz → 680)
5. **INS** İnşaat/taahhüt — 170↔350 proje eşlemesi, hakediş stopajı, tamamlanma tutarlılığı
6. **LOJ** Taşımacılık — araç muavini başına akaryakıt/amortisman makullüğü, aktifleştirme (VUK 272) ayrımı
7. **GID** Restoran/gıda — nakit yoğunluk (Z raporu↔kayıt), fire/zayi KDV düzeltmesi
8. **TAR** Tarım — TMS 41 canlı varlık (faz 2; pazar boşluğu analizimizin hedefi)

Ayrıntılı çalışma listeleri: `data/denetim-programlari.json`.

## 4. BDS bağları (çalışmaların dayanağı)
- **BDS 240** hile riski → yevmiye kayıt testleri (M3/M5/M6), hasılat kesme (M9), nakit yoğunluk
- **BDS 315/330** risk belirleme/yanıt → sektör programının kendisi
- **BDS 505** dış teyit → cari mutabakat çıktıları (D bölümü eşleştirmeyle birleşir)
- **BDS 520** analitik prosedürler → M1/M10/M11
- **BDS 530** örnekleme → bulgu kümesinden örneklem çekimi (faz 2)
- **BDS 570** süreklilik → mevcut 14 oran + banka görünümü buraya bağlanır
- Önkoşul: **kullanıcı+zaman damgası (B)** olmadan BDS 240 kayıt testlerinin "kim/ne zaman" boyutu eksik kalır.

## 5. Uygulama sırası
1. H.1 Veri modeli + JSON yükleme (bu dosya + programlar)
2. H.2 Motor tamamlama (öncelik: M8 karşı bacak — B.8'le ortak, M5, M9, M10, M6)
3. H.3 ETIC programını fixture'da uçtan uca çalıştır (bulgu üretimi)
4. H.4 Denetim sayfası UI: program listesi → çalıştır → bulgu → çalışma kağıdı satırı
5. H.5 Kullanıcı düzenleme katmanı (kopyala-düzenle-kapat + sürüm)
6. H.6 Mükellef sektör ataması (E.5'e bağlanır)
