# Hesap Planı — Kanonik Format & Türetme Kuralları

**Seed:** `data/tdhp.csv` — MSUGT Tek Düzen Hesap Planı'nın tam listesi (8 sınıf, 58 grup, 262 ana hesap).
Kolonlar: `kod,ad`. Diğer her şey **koddan türetilir** (kural-tabanlı, "sınırları çizilmiş sistem").

## Neden sadece kod + ad?
TDHP kapalı ve kurallı bir sistem: hesabın **sınıfı, tipi, doğası, seviyesi** kodundan deterministik
çıkar. Veriyi minimal tutup kuralları `domain`'de tek yerde tanımlıyoruz (tekrarsız, tutarlı).

## Türetme kuralları (domain'de)
| Özellik | Kural |
|---------|-------|
| **Seviye** | kod uzunluğu: 1=sınıf, 2=grup, 3=ana hesap. (Şirket 100.01 açınca 4+ = muavin) |
| **Yaprak (postable)** | Çekirdek seedde **3 hane = yaprak**. Muavin açılınca üst yaprak olmaktan çıkar |
| **Tip** | İlk haneden: 1,2→Aktif · 3,4,5→Pasif · 6→Sonuç(gelir/gider) · 7→Maliyet · 9→Nazım |
| **Doğa (normal bakiye)** | Sınıf tabanı + **(-) çevirme**: Aktif/Maliyet→Borç, Pasif→Alacak, Sınıf 6 tabanı→Alacak; adı **"(-)" ile bitiyorsa doğa ters çevrilir** (düzenleyici/kontra hesap) |

**Düzenleyici (-) örnekleri:** 103, 119, 257 (aktif kontra→Alacak); 501, 580, 591 (pasif kontra→Borç);
610, 621, 632 (gelir kontra→Borç). Bu kural 62 hesabın hepsinde doğru çalışır.

## Bilinen istisna (domain'de özel ele alınacak)
- **Sınıf 7 yansıtma hesapları** (701, 711, 721, 731, 741, 751, 761, 771, 781, 798): adlarında "(-)"
  yok ama **alacak** çalışırlar (maliyet yansıtma). Kural tabanı bunları Borç sanır → domain'de bu
  kodlar için özel "alacak yansıtma" işareti. (Dönem sonunda sıfırlanır; mizan işareti için gerekli.)

## Doğrulama / bütünlük
- ✅ Kod benzersiz, 8 sınıf (8 boş — standart), boşluksuz grup yapısı.
- ⚠️ **Son kontrol:** Bu liste MSUGT + güncel kaynaklardan derlendi; üretim öncesi resmi
  **MSUGT (Muhasebe Sistemi Uygulama Genel Tebliği) / GİB** metniyle bir kez daha karşılaştırılmalı
  (sektörel ek hesaplar ve enflasyon düzeltmesi kalemleri için).

## Kullanım (dikey dilim)
Mükellef açılışında bu seed kopyalanır → mükellefin hesap planı olur. Mükellef alt (muavin) hesap
ekleyebilir; standart 262 hesap silinemez (bütünlük) ama pasifleştirilebilir.
