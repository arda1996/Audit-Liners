# TFRS Raporlama — Kalem Sınıflandırması (tasnif) Öğrenmesi

> Kaynak: Kiler Holding 31.12.2025 bağımsız denetim raporu (docs/analiz/05). Kullanıcının kilit tespiti:
> TFRS satırlarının verisi **muhasebe kayıtlarının niteliğinden** (açıklama + faturanın dayanağı) gelir.
> Tasnif haritası: `data/tfrs-tasnif.json` (veri-güdümlü, I.1).

## Ana ders: TDHP kodu TEK BAŞINA yetmez
VUK mali tablosunda 120 = "Alıcılar" tek satır. TFRS'de aynı 120 bakiyesi **kaydın niteliğine göre**
iki-üç ayrı sunum satırına bölünür. Bir bakiyenin hangi TFRS satırına gideceğini belirleyen 4 boyut:

1. **Vade** — kısa (dönen) / uzun (duran). Hesap sınıfı verir AMA reclass var: "uzun vadeli kredinin
   kısa vadeli kısmı" kredi itfa planındaki vade tarihinden çıkar, hesap kodundan değil.
2. **İlişki** — ilişkili taraf / ilişkili olmayan. Kaynak: karşı **cari kartının ilişkili taraf bayrağı**
   (henüz yok — I.5 alan ihtiyacı). Rapor bunu her ticari/diğer alacak-borç satırında ikiye böler.
3. **Nitelik** — ticari / diğer / finans sektörü / müşteri sözleşmesi. Kaynak: hesap + **açıklama** +
   **faturanın dayanağı** (mal-hizmet ticari mi, avans mı, finansman mı). Ör. 340 avans → TFRS 15
   "müşteri sözleşmelerinden doğan yükümlülük"; 335 ortaklara borç → "diğer borçlar - ilişkili".
4. **Ölçüm** — maliyet / gerçeğe uygun değer / itfa edilmiş maliyet. Bu ham TDHP bakiyesinden değil,
   **TFRS düzeltme katmanından** (I.2) gelir: TMS 40 GUD, TFRS 9, TFRS 16 kullanım hakkı, TMS 2 NRV.

## Rapordaki somut örnekler (tersine)
- "Ticari alacaklar → İlişkili taraflardan / İlişkili olmayan taraflardan" (2 satır, tek 12x kökünden)
- "Peşin ödenmiş giderler → İlişkili / İlişkili olmayan" (180 dönemsellik + ilişki)
- "Müşteri sözleşmelerinden doğan varlıklar/yükümlülükler" (TFRS 15 — 17x/340, açıklama+sözleşme)
- "Uzun vadeli kredilerin kısa vadeli kısmı" (vade reclass)
- "Kullanım hakkı varlıkları / Ertelenmiş vergi" — TDHP'de karşılığı YOK, düzeltme katmanından doğar

## Programa yansıması (rota)
- **I.1 (bu doküman + tfrs-tasnif.json):** TDHP → TFRS satır haritası, bölen boyutlar + kaynak nitelik. ✅ başladı
- **I.5 alan ihtiyacı:** cari kartta ilişkili taraf bayrağı; muavin/açıklama nitelik etiketi; kredide vade tarihi.
- **I.2 düzeltme katmanı:** ölçüm boyutunu (GUD/NRV/kıdem/ertelenmiş vergi) dayanaklı üretir; VUK defterine işlemez.
- **I.3 TFRS tam set:** bu haritayı mizana uygulayıp Kiler formatında karşılaştırmalı tablo üretir.
- **TFRS raporlama SAYFASI:** analiz sayfasından ayrı; nitelik-temelli tasnifi gösterir, ham hesabı ve
  düzeltmeyi yan yana koyar (denetim izi). "Yavaş yavaş" — önce tasnif haritası, sonra sayfa.
