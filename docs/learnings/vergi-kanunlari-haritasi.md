# Vergi Kanunları → Program Yüzeyi Haritası (2026)

> Kullanıcının verdiği 10 kanunluk çerçeveye göre eksik analizi (2026-07-05).
> Parametreler: `data/vergi-parametreleri.json` (sürümlü, dogrulandi bayraklı — kod hardcode etmez).
> 2026 değerleri 588 No.lu VUK Tebliği (RG 31.12.2025) ve yeniden değerleme %25,49'a göre.

## Kanun → programda ne var / ne eksik

| Kanun | Programa dokunan yüzü | Durum |
|-------|----------------------|-------|
| **GVK 193** | Şahıs işletmesi mükellef tipi (GV tarifesiyle geçici vergi/yıllık beyan); stopajlar (kira/SM %20) muhtasara | ⬜ Mükellef tipi ayrımı yok — geçici vergi kağıdı KV varsayıyor. Stopaj kayıtları atılabiliyor (360), muhtasar beslemesi E.6'da |
| **KVK 5520** | %25 oran, geçici vergi zinciri, istisnalar (5/1-a), KKEG (md.11), uyumlu mükellef %5 | ✅ Geçici vergi kağıdı çalışıyor; oran artık parametre dosyasından. ⬜ İstisna/KKEG kod kataloğu (şimdilik manuel satır) |
| **KDVK 3065** | Oranlar, indirim (29), mahsup, zayi (30), tevkifat (9) | ✅ Mahsup motoru + KDV1 taslağı. ⬜ Tevkifatlı fatura (2 no.lu KDV), iade süreçleri |
| **ÖTV 4760** | Liste bazlı, sektörel (akaryakıt/otomotiv/içecek) | ⏸ Sektörel modül — ÖTV alış maliyetine girer kuralı (VUK 262) şablonlara eklenebilir |
| **6802 (BSMV/ÖİV)** | Banka masraflarındaki BSMV ayrıştırma; finans sektörü mükellefi BSMV beyannamesi | ⬜ Ekstre eşleştirmede masraf/BSMV satırı önerisi yok |
| **Damga 488** | Sözleşme/ücret/beyanname damgası; bordroda binde 7,59; beyanname maktu damgaları | ⬜ Bordro (E.6) ile gelir; beyanname damgası vergi takvimi kartına eklendi |
| **Harçlar 492** | Gider/maliyet ayrımı: tapu harcı → gayrimenkul maliyeti (VUK 270), yargı harcı → gider | ⬜ Hesap kuralları verisine harç yönlendirmesi eklenebilir |
| **VİV 7338** | SMMM ticari rutini dışı | ✖ Kapsam dışı (bilinçli) |
| **VUK 213** | ANAYASA: defter/kayıt düzeni ✅, iptal md.217 ✅, belge düzeni ✅, kronoloji ✅; **değerleme md.258+ ⬜ (D′)**; hadler (fatura 12k, amortisman 12k, şüpheli 25k) | 🔨 En kritik eksik değerleme motoru; hadler parametre dosyasında, motorlara bağlanacak |
| **6183** | Gecikme zammı %4,5/ay, tecil; geç ödeme maliyeti | ⬜ Geç ödeme hesaplayıcı (takvim + tutar → zam) |

## Derin öğrenme özeti — programı şekillendiren kurallar
1. **VUK her şeyin usulüdür:** değerleme ölçüleri (md.258+: maliyet bedeli esas, borsa rayici, mukayyet değer,
   itibari değer, emsal bedel) D′ değerleme sihirbazlarının hukuki iskeleti. Amortisman (md.313+): 12.000 TL
   altı doğrudan gider; faydalı ömür listeleri (333 No.lu tebliğ ekleri) oran kataloğu olacak.
2. **Şüpheli alacak (VUK 323):** dava/icra şartı; 25.000 TL altı küçük alacaklar protesto/yazıyla yeterli —
   TIC-03 denetim çalışması eşiği bu haddan okumalı.
3. **Geçici vergi = KV oranı** üzerinden kümülatif; yıllıkta mahsup (KVK 32). Şahıslarda GV tarifesi işler —
   mükellef tipi (E.5) olmadan şahıs işletmesi desteklenemez.
4. **KDV tevkifatı (KDVK 9):** belirli hizmetlerde alıcı 2 no.lu KDV beyannamesiyle sorumlu — fatura modülü
   canlanınca tevkifat oran tablosu gerekir.
5. **6183 vergi DAİRESİNİN hukuku:** program tahakkuk sonrası ödeme takibinde gecikme zammını gösterirse
   SMMM'ye "bugün ödersen X" değeri sunar — takvim kartının doğal uzantısı.
6. **Damga pratiği:** her beyannamenin kendi maktu damgası tahakkuk fişine gider (770/360) — beyanname
   üretimi yapılınca otomatik satır.

## Sıradaki bağlantılar (öncelik)
1. D′ değerleme (VUK 258+/313+/323/280/281) — hadler hazır, motor yok
2. Mükellef tipi (E.5) → GV/KV tarife seçimi
3. VUK hadlerinin motorlara bağlanması (fatura sınırı uyarısı, şüpheli eşiği, amortisman sınırı)
4. 6183 gecikme zammı hesaplayıcı (küçük, takvime eklenir)
