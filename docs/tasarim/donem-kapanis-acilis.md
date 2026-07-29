# Dönem Kapanış / Açılış + Virman (tam yapı)

Eksikti — kullanıcı vurguladı: "kapama işlemlerinde bazı hesapların bakiyeleri **virmanlanıyor**."
Burası dönem döngüsünün kalbi. Tümü **otomatik üretilen virman/mahsup fişleri**dir
(hesaplar arası bakiye aktarımı = virman). Domain'de `kapanis()` / `acilis()` operasyonları üretecek.

## Terim
- **Virman:** bir hesabın bakiyesini başka hesaba aktarma (ör. 600 → 690). Kapanışın temel hareketi.
- **İptal** (eski "storno"): kesin fişin ters kaydı. (Kapanışla karıştırılmaz.)

## Kapanış — sıra (muhasebenet.net örneğiyle doğrulandı)

### 0) Maliyet (7) → gelir tablosu (6) devri
- Maliyet hesapları (710–780) **yansıtma** hesapları (711–781) aracılığıyla ilgili 6 hesabına aktarılır;
  sonra 7 ve yansıtma hesapları ters kayıtla kapatılır. (7/A akışı — `maliyet-ve-vergi.md`.)

### 1) Sonuç hesaplarını 690'a topla (gelir/gider virmanı)
- **Gelir** hesapları (60, 64, 67…) alacak bakiyeli → kapatmak için **BORÇ**, karşı **690 ALACAK**.
- **Gider/maliyet sonuç** hesapları (621, 622, 631, 632, 660, 68…) borç bakiyeli → **ALACAK**, karşı **690 BORÇ**.
- Sonuç: **690 Dönem Kârı/Zararı** bakiyesi = faaliyet sonucu.

### 2) Vergi karşılığı (kâr varsa)
- `690 BORÇ` / `370 Dönem Kârı Vergi ve Diğer Yasal Yük. Karşılıkları ALACAK` (vergi tahakkuku) — `691` üzerinden izlenir.

### 3) 690 → 692 Dönem Net Kârı/Zararı
- `690 BORÇ (kâr) · 691 ALACAK (vergi karş.) · 692 ALACAK (net kâr)` — net sonuç 692'de toplanır.

### 4) 692 → 590 / 591 (öz kaynağa taşı)
- **Kâr:** `692 BORÇ · 590 Dönem Net Kârı ALACAK`.
- **Zarar:** `591 Dönem Net Zararı (-) BORÇ · 692 ALACAK`.

### 5) Bilanço hesaplarını kapat (formal kapanış fişi)
- Kalan **aktifler ALACAK**, **pasifler BORÇ** ile sıfırlanır → Σborç=Σalacak. Dönem `KAPANMIŞ`.

## Açılış (yeni dönem) — kapanışın tersi
- Bilanço bakiyeleri ters kayıtla geri açılır: **aktifler BORÇ, pasifler ALACAK** → `acilis_fisi` (devir).
- Sonuç hesapları (6,7) açılışa **gelmez** (dönemsel, sıfırdan başlar). Sadece bilanço (1-5) devreder.

## Domain tasarımı (uygulanacak)
```
kapanis(baglam) -> Vec<Fis>   // adım 1-5'in virman/kapanış fişleri (taslak), gözden geçirilip kesinleştirilir
acilis(onceki_kapanis_bakiyeleri) -> Fis   // mevcut acilis_fisi'nin genişletilmişi
```
- Girdi: dönem mizanı (hesap bazlı bakiyeler). Çıktı: dengeli virman fişleri.
- Kural: yalnız **bilanço (1-5)** hesapları devreder; **sonuç (6,7)** kapanışta sıfırlanır.
- Her üretilen fiş yine V1–V9'dan geçer (denge zorunlu) → tutarlılık garanti.

## Neden önemli
Bu olmadan dönem kapatılamaz, kâr/zarar öz kaynağa taşınamaz, sonraki dönem açılışı üretilemez.
"Tam yapı"nın olmazsa olmazı; muavin/cari ve kalıcılıkla birlikte gerçek kullanılabilirliğin parçası.
