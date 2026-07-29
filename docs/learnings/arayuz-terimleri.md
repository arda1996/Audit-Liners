# Muhasebe Yazılımı Arayüz Terimleri (buton/menü sözlüğü)

Türk muhasebe yazılımlarının (Paraşüt, Mikro, Logo, Bizimhesap, Luca) ortak arayüz dili.
Amaç: kendi UI'mızda **tutarlı, tanıdık** terimler kullanmak. "Bizde?" = hangi fazda.
Kaynaklar: parasut.com, mikro.com.tr, bizimhesap.com (arayüz incelemesi).

## Genel aksiyon butonları
| Buton | Anlamı | Bizde? |
|-------|--------|--------|
| Yeni / Yeni kayıt | Boş kayıt aç | ✅ şimdi |
| Kaydet | Taslak olarak sakla | ✅ şimdi (Taslak) |
| Kaydet ve yeni | Kaydet, hemen yeni aç | ✅ şimdi |
| Kesinleştir / Onayla | Kaydı kilitle (değişmez) | ✅ şimdi (domain `kesinlestir`) |
| Vazgeç / İptal | Değişikliği bırak | ✅ şimdi |
| Sil | Kaydı sil (yalnız taslak) | ✅ şimdi |
| Düzenle | Mevcut kaydı aç | ✅ şimdi |
| Kopyala / Çoğalt | Benzer kayıt üret | ⏳ sonra |
| Yazdır / Önizleme | Çıktı | ⏳ sonra |
| Dışa aktar (Excel/PDF) | Veriyi indir | ⏳ raporlama |
| Ara / Filtrele / Listele | Kayıt bul | ✅ liste ekranı |
| Yenile | Listeyi tazele | ✅ |

## Fiş işlemleri (çekirdek)
| Terim | Anlamı | Bizde? |
|-------|--------|--------|
| Tahsil / Tediye / Mahsup / Açılış / Kapanış fişi | Fiş tipleri | ✅ şimdi |
| Satır ekle / Satır sil | Yevmiye satırı | ✅ şimdi |
| Borç / Alacak | Kayıt yönü | ✅ şimdi |
| Denge / Bakiye | Σborç=Σalacak | ✅ şimdi |
| Fiş no / tarih / açıklama | Başlık alanları | ✅ şimdi |
| Evrak / Belge no | Dayanak | ✅ şimdi |
| İptal | Kesin fişi ters kayıtla iptal (tekn. "storno") | ✅ şimdi (domain `iptal_fisi`) |
| Virman | Hesaplar arası bakiye aktarımı (kapanışın temeli) | ✅ dönem kapanışında |

## Cari hesap (sonraki faz)
| Terim | Anlamı | Bizde? |
|-------|--------|--------|
| Cari kart | Müşteri/tedarikçi tanımı | ⏳ |
| Cari hareket / ekstre | Hesap dökümü | ⏳ |
| Mutabakat | Karşılıklı bakiye onayı | ⏳ |
| Tahsilat / Ödeme | Para giriş/çıkış | ✅ (tahsil/tediye fişi) |
| Virman | Hesaplar arası aktarım | ⏳ |

## Defterler & raporlar (raporlama modülü)
| Terim | Anlamı | Bizde? |
|-------|--------|--------|
| Yevmiye defteri | Tarih sıralı kayıtlar | ✅ yakında |
| Defter-i kebir / Büyük defter | Hesap bazlı hareket | ✅ yakında |
| Mizan (geçici/kesin) | Hesap borç/alacak/bakiye özeti | ✅ var |
| Muavin defter | Alt hesap dökümü | ⏳ |
| Bilanço / Gelir tablosu | Mali tablolar | ⏳ raporlama |
| Envanter | Dönem sonu sayım | ⏳ |

## Dönem
| Terim | Anlamı | Bizde? |
|-------|--------|--------|
| Hesap dönemi | Mali yıl | ✅ (model) |
| Açılış / Kapanış | Dönem başı/sonu | ✅ (model) |
| Devir | Bakiyeyi sonraki döneme taşı | ✅ (model) |
| Dönem kilidi | Kapalı döneme kayıt yok | ✅ (kural) |

## Sonraki fazlar (terim rezervi)
Stok: Stok kartı · Stok hareketi · Depo · Sayım.
e-Dönüşüm: e-Fatura · e-Arşiv · e-İrsaliye · e-Defter · e-SMM · GİB.
Vergi: KDV beyannamesi · Muhtasar · Geçici vergi · Tevkifat.

## Karar: MVP buton seti (masaüstü)
**Fiş ekranı:** Yeni · Kaydet (taslak) · Kesinleştir · Vazgeç · Sil · Satır ekle/sil · İptal.
**Sol menü (navigasyon):** Fişler · Yevmiye defteri · Defter-i kebir · Mizan · Hesap planı · (Cari, Raporlar, Dönem — sonra).
**Liste ekranı:** Yeni · Ara/Filtrele · Düzenle · Yenile.
Dil ilkesi: cümle düzeni, kısa, fiil-önce ("Yeni fiş", "Kesinleştir"). ALLCAPS yok.
