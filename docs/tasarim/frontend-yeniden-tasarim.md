# Frontend Yeniden Tasarım — Araştırma & Yön (2026-07-09)

> Kullanıcı: "her şey darma dağın, butonlar gereksiz, daha korelasyonu yüksek ve profesyonel bir yapı."
> Bu tur yalnız YÖN; uygulama sonraki turlarda aşamalı. Kaynak: ERP/muhasebe SaaS UI araştırması + 5-ajan
> eleştirisi (2026-07-08) + arayuz-terimleri.md.

## Mevcut sorunlar (ekran + eleştiriden)
1. **Üst bar savruk:** Rehber · Firma·Çıkış · Mükellef · Dönem · +Yeni kayıt — hepsi AYNI boyutta chip,
   hiyerarşi yok; hangisi birincil belli değil (ekran görüntüsündeki kırmızı kutu).
2. **ÇİFT breadcrumb:** başlık altında "Kayıt → fişler → yevmiye…" VE sağda "Kayıt → Yevmiye → Kebir…" —
   ikisi de aynı şeyi söylüyor, biri gereksiz.
3. **Tutarsız kart/tipografi:** .card-hd yok (14+ inline kopya), 12+ font boyutu, ₺/tarih/negatif tutarsız
   (5-ajan bulguları — hâlâ açık).
4. **Bağlam belirsiz:** "hangi mükellef üzerinde çalışıyorum" birincil olmalı ama küçük bir chip.

## Araştırma çıktısı — profesyonel desenler (ERP/fintech)
- **Global bar 3 bölge:** sol = workspace/bağlam anahtarı; orta = sayfa başlığı + TEK breadcrumb; sağ =
  yalnız gerçekten global aksiyonlar (avatar menüsü, yardım). "Salesforce App Launcher" kalıbı: kalıcı
  global bar + ⌘K komut paleti her şeye saniyede erişim.
- **Mükellef = birincil bağlam:** Slack/Linear "workspace switcher" gibi SOL ÜSTTE belirgin — kullanıcı
  önce "hangi firma" seçer, sonra çalışır. Küçük chip değil, kimliğin parçası.
- **Progressive disclosure:** özet önce (dashboard KPI şeridi 4-6), tıkla → detay. Bilgi aşırı yükü
  kullanıcıların %46,7'sini vuruyor — yoğunluğu katmanla.
- **Veri tablosu standardı:** sticky başlık (position:sticky), satır 40px (yoğun) / 48px (rahat), metin
  sola / sayı sağa / durum rozeti ortala. Muhasebe defterleri için kritik.
- **F/Z tarama deseni:** büyük sayı = anında dikkat; öncelikli öğe sol-üst.

## Yeni yapı — hedef (uygulanacak spec)
**Global bar (yeni):**
- SOL: 🏢 Mükellef anahtarı (belirgin, unvan + sektör rozeti) — birincil bağlam.
- ORTA: sayfa başlığı + TEK breadcrumb (alttakini kaldır).
- SAĞ: Dönem chip · ⌘K ara · yardım (?) · avatar menüsü (kullanıcı adı/rol/çıkış — tek dropdown'a topla).
- "+ Yeni kayıt" birincil aksiyon olarak sağda kalır ama tek vurgulu düğme.

**Tasarım sistemi (5-ajan ertelenenleri):**
- Tek `.card-hd` (başlık kabı) · 5 basamaklı tipografi skalası (lbl/body/emphasis/h3/h2) · tablo override'ları
  kaldır · tek negatif kuralı (renk + işaret) · ₺/gg.aa.yyyy her yerde.
- Renk: kırmızı-beyaz marka (logo) ile uyumlu; kırmızı yalnız vurgu/tehlike, ink birincil.

**Sayfa iskeleti:** sidebar (240px, mevcut iyi) + global bar (yeni) + içerik grid (CSS Grid auto-fill kart).

## Sıra (uygulama, sonraki turlar)
1. Global bar sadeleştirme (tek breadcrumb, mükellef workspace switcher, avatar menü) — en görünür kazanım.
2. Tasarım sistemi (.card-hd + tipografi + tablo standardı + negatif/para/tarih tutarlılığı).
3. Dashboard progressive disclosure + tablo sticky başlık.
4. Yetkilendirmeyle entegrasyon: departmana göre modül görünürlüğü (bkz. yetkilendirme-departman-hiyerarsi.md).
