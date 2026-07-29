# Beyanname Hattı — Doktrin ve Mimari

> Kalıcı doktrin (2026-07-05, hafızada: beyanname-doktrini):
> **Dayanak = muhasebe kayıtları · Format = GİB Beyanname Düzenleme Kılavuzu · Manuel müdahale = şart, izli, deftere işlemez.**

## Üç katman

```
[1] MUHASEBE KAYITLARI (tek doğruluk kaynağı)
      │  otomatik eşleme — her satır KAYNAK referanslı
      ▼     ör. "391.10 muavini — 12/2026 alacak hareketi"
[2] BEYANNAME TASLAĞI (kılavuz formatında satırlar)
      │  + SMMM MANUEL MÜDAHALESİ (yön/tutar/dayanak alanlı,
      │    M rozetli, deftere ASLA işlemez, taslakta izli)
      ▼
[3] BEYAN (GİB formatı — ileride XML/BDP paketi)
```

## Uygulama durumu
| Beyanname | Otomatik eşleme | Manuel katman | Kaynak referansı | Kılavuz formatı |
|-----------|----------------|---------------|------------------|-----------------|
| KDV1 | ✅ 391/191/190 motorundan | ✅ ilave matrah / indirim-istisna satırları (dayanak alanlı) | ✅ satır + kaynak_notlari | 🔶 kılavuz satır kodları eşlenecek (data-driven satır tanımı) |
| Geçici vergi | ✅ GT + 689 taraması | ✅ KKEG/istisna satırları | ✅ kaynak_notlari | 🔶 |
| Muhtasar / yıllık GV-KV / damga | ⏸ | kalıp hazır (vergi_duzenlemeler) | — | ⏸ |

## Kurallar (her yeni beyanname ekranı için)
1. Otomatik satır **kaynaksız olamaz** — hesap öneki + dönem + hareket türü yazılır; tıklanınca muavine
   inilebilmeli (kağıttaki iniş kalıbı buraya da gelecek).
2. Manuel satır **dayanak ister** (fiş/belge/kanun md.) ve sistem satırlarından görsel olarak ayrılır (M rozeti).
3. Satır yapısı kılavuzdan: `data/beyanname-formatlari.json` (ileride) — kılavuz güncellenince veri değişir, kod değişmez.
4. Beyan edilen her rakam → taslak → kayıt zinciri geri yürünebilir (vergi incelemesine hazır iz).

## Sonraki adımlar
- Kılavuz satır kodlarının data dosyasına dökülmesi (KDV1 önce), taslak → BDP/XML çıktısı
- Beyanname taslağından muavine iniş (kagit drill-down kalıbının kopyası)
- Manuel müdahalelere kullanıcı+zaman damgası (B işi ile birlikte)
