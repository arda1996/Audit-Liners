---
name: muhasebe-kontrol
description: Bu projede muhasebe kaydı, KDV/ÖTV hesabı, beyanname veya defter etkileyen HER değişiklikten önce ve sonra uygulanacak doğruluk kuralları. Fiş üretimi, tahsis/eşleştirme, iptal-iade, vergi hesabı, mizan/bilanço veya beyanname kodu yazarken; "bu kayıt doğru mu", "neden ters bakiye var", "KDV neden tutmuyor" sorularında kullan.
---

# Muhasebe Doğruluk Kuralları — projenin biriktirdiği doktrin

Bu kurallar tartışma sonucu değil, **hata bulunarak** öğrenildi. Her biri bir kez ihlal edildi ve
defteri bozdu. Kod yazmadan önce oku; yazdıktan sonra doğrula.

## 1. İPTAL ≠ İADE (en sık karıştırılan)

| | İPTAL | İADE |
|---|---|---|
| Olay | Belge hüküm doğurmadı (teslim yok) | Teslim oldu, mal geri geldi |
| Kayıt | **Ters kayıt (storno)** | **610 Satıştan İadeler** |
| 600'e etki | Ters çevrilir | **Ters ÇEVRİLMEZ** |

600'ü ters çevirmek bilgi kaybıdır: gelir tablosu `600 − 610 − 611 − 612 = net satış` ister ve
**iade oranı (610/600) bir denetim göstergesidir** (dönem sonu şişirme / kanal doldurma).
Satıştan iadede **191 BORÇLANIR** (KDVK 35 matrah düzeltmesi); alıştan iadede **191 ALACAKLANIR**.

## 2. KAYIT SİLİNMEZ (VUK 219 / TTK 65)

Bir kayıt yanlışsa **silinmez, karalanmaz** — `IPTAL` işaretlenir ve karşısına borç/alacağı yer
değiştirmiş **storno** açılır. Asıl + storno net sıfır eder, **ikisi de defterde kalır**.
Düzeltme kaydı **olayın tarihine değil, kaydın yapıldığı tarihe** atılır (yevmiye geriye tarihlenemez);
belgenin kendi tarihi `dayanak`ta korunur, iz kopmaz.

## 3. ÖTV → KDV zinciri

**ÖTV, KDV matrahına DAHİLDİR** (ÖTVK 11 · KDVK 24/b):
`bedel → ÖTV → KDV matrahı = bedel + ÖTV → KDV`
KDV'yi yalnız bedelden hesaplamak eksik beyandır. ÖTV alışta **maliyete** girer (VUK 262), 191'e yazılmaz.

## 4. KDV muavinleri ARDIŞIK

`391.01, 391.02, 391.03…` — TDHP alt kırılım kuralı. **Oran koddan okunmaz**, katalogdan çözülür
(`hesaplama::muavin_orani`). `391.20` yazmak yanlıştır.
**%8 ve %18 tarihî orandır** — 10.07.2023'te %10 ve %20 oldu; eski tarihli belgede geçerli,
yeni tarihli fişte uyarı verilmeli.

## 5. Tahsilat cariye aittir, faturaya değil

Para cariye girer; hangi faturayı kapattığını **tahsis motoru** belirler:
donmuş (fişlenmiş) → manuel (maker) → **FIFO** (en eski açık fatura, **TBK 100**).
- Tahsilat kendinden **sonraki tarihli** faturayı kapatamaz (doğmamış borç ödenmez).
- **Kayda geçmiş tahsis DONDURULUR** — yeniden hesaplanırsa tek düzeltme yüzlerce storno üretir.
- Karşı taraf **ünvanı yoksa otomatik eşleşme YAPILMAZ** — yanlış cariye tahsis yanlış bakiyedir.

## 6. Hükümsüz belge kayıt doğurmaz

`TASLAK` → hiç kayıt yok (henüz belge değil).
`IPTAL/RED` → tahakkuk + storno çifti; **cari bakiye doğurmaz**, tahsilat tahsis edilemez.
Cari açık bakiye = `toplam − tevkifat − iade − iskonto − tahsilat` (asla negatif değil).

## 7. Ters bakiye = anomali, sebebi sorulmalı

Bakiye hesabın doğasına aykırıysa açıklanabilir bir sebebi olmalıdır:

| Hesap | Ters bakiye ne demek |
|---|---|
| 100 Kasa alacak | Olmayan para ödenmiş → kayıt dışı hasılat / eksik virman |
| 102 Banka alacak | Hesapta olmayan para → KMH ise **300 Banka Kredileri**'ne alınmalı |
| 120 Alıcılar alacak | Fazla tahsilat → avanssa **340** |
| 320 Satıcılar borç | Fazla ödeme → avanssa **159** |
| 153 Stok alacak | **Negatif stok** — olmayan mal satılmış; alış faturası eksik |

Kontrol: `GET /api/denetim/anomali`

## 8a. Gider hesapları ve KKEG

Sektörün `maliyet_secenegi` hesabı belirler: **7/A** (gider yeri: 710/760/770…) ·
**7/B** (gider çeşidi: 790–797) · **yok** (ticaret/serbest meslek → doğrudan 6xx).
Ticarette mal alışı gider değil **stoktur** (153 → satılınca 621).

**KKEG için ayrı ana hesap YOKTUR** — gider normal hesabına, KKEG kısmı **`.99` muavinine**.
Üç davranış: TAMAMI (vergi cezası) · ORANSAL (binek oto %30) · HADDİ AŞAN (binek kira/amortisman).
Sektörel istisna: faaliyeti **araç kiralama/işletme** olanda binek kısıtı uygulanmaz (GVK 40/1).

Had bilinmiyorsa **KKEG uydurma** — uyar. `indirilebilir + KKEG = tutar` her zaman tutmalı.
Ayrıntı: `docs/learnings/gider-hesaplari-ve-kkeg.md` · `POST /api/gider/oner`

## 8. Beyanname doktrini

Beyan düzeltmesi **deftere işlemez**. Defter **ticari kârı**, beyanname **mali kârı** gösterir.
**KKEG** gider hesabına normal yazılır, ayrı muavinde/nazımda izlenir, beyannamede matraha
**geri eklenir** — yevmiyede ters kayıt yapılmaz.
İstisna teslim matrahı beyan edilir ama **hesaplanan KDV'ye eklenmez**.

## 9. Oranlar koda gömülmez

Hepsi `data/vergi-parametreleri.json` → `hesaplama` bölümünden okunur. Yeni oran eklenince
kod değişmez. **Belirsiz oranı motor tahmin etmez** — kullanıcıya seçtirir
(tevkifat türü oranları, ÖTV oranları bilerek boş).

## 10. Her fiş dengeli, her tutar kuruş

`Σborç = Σalacak` — tek fişte ve toplamda. Tutarlar **i64 kuruş**, float yok.
Brütten KDV ayrıştırmada `matrah + KDV = brüt` **her zaman** tutmalı (yuvarlama kaçağı olmamalı).

---

## 11. GÜVENLİK KATMANI — kural ekle, ad-hoc kontrol yazma

Kurallar tek yerde: `crates/api/src/kontrol.rs`. **Handler içine dağınık `if` yazma** —
her yeni kural oraya, numaralı olarak eklenir.

**İki katman, iki farklı iş:**
- **ÖN KONTROL (K-kodları)** → kullanıcı hatasını ENGELLER. Her bulgu `mesaj` + **`cozum`**
  taşır; "hata var" demek yetmez, ne yapacağı söylenir.
  `ENGEL` → işlem yapılmaz · `UYARI` → yapılır ama bildirilir.
- **DEĞİŞMEZ (D-kodları)** → sistem hatasını YAKALAR. Biri bozulursa bu bir **BUG**'dır,
  sessiz geçilmez; yanıtta `degismez_ihlalleri` olarak döner.

Mevcut kurallar: K01 tahsilat yok · K02 fatura yok · K03 hükümsüz belge · K04 cari uyuşmaz ·
K05 yön uyuşmaz · K06 net borç sıfır · K07 manuel dolu (+hangi eşleştirme kaldırılmalı) ·
K08 zaten bağlı · K09 fatura tahsilattan sonraki tarihli · K10 tutar borçtan büyük.
D01 para korunumu · D02 dagitilan tutarlılığı · D03 aşırı tahsilat · D04 negatif kalan ·
D05 hükümsüz belge tahsis almış · D06 defter dengesi.

**Yeni bir hata bulduğunda:** önce onu yakalayan kuralı `kontrol.rs`'e ekle, sonra düzelt.
Böylece aynı hata bir daha sessizce geçmez.

---

## Değişiklik sonrası DOĞRULAMA sırası

```bash
cargo test --workspace                      # 117 test — hepsi geçmeli
curl -s localhost:8787/api/kontrol/saglik   # saglikli: true olmalı (değişmez + ters bakiye)
curl -s localhost:8787/api/simulasyon/ozet  # denge_ok: true
curl -s localhost:8787/api/mizan            # Σborç = Σalacak
```

Beyanname değiştiyse ayrıca: 12 ayın hepsi dolu mu, oran kırılımı en az 2 satır mı,
ödenecek ve devreden aynı anda pozitif değil mi.

## Performans — pahalı yolu her istekte tekrarlama

Uçlar bir ara **10+ saniye** sürüyordu: her istek `gorunum()` çağırıyor, o da 2200 faturayı
LCG ile sıfırdan üretip FIFO dağıtımını baştan yapıyordu.

İki önbellek çözdü (10,80 sn → 0,02 sn):
1. **`HAM_ONBELLEK`** (`OnceLock`) — `ham()` deterministiktir (sabit tohum, girdisi yok),
   bir kez üretilir. En büyük kazanç buradaydı.
2. **`gorunum_onbellek`** (AppState) — sürüm damgalı. Dağıtımı etkileyen her mutasyonda
   **`surum_arttir(&mut st)`** çağrılır ve önbellek düşer.

⚠ **Yeni bir mutasyon yazarken `surum_arttir` çağırmayı unutma** — yoksa ekran bayat veri gösterir.
Şu an çağıranlar: manuel tahsis ekle/sil, `defteri_senkronize` (kayıt değiştiyse).

Bir de: **11 bin fişten Defter nesnesi kurma** (`ay_defteri`) sırf bakiye lazımsa pahalıdır
(~0,7 sn). Yaprak hesapta rollup gereksizdir — tek geçişte `HashMap` toplamı yeter.

Ölçmeden iyileştirme yapma:
```bash
for u in "/api/simulasyon/faturalar?limit=50" "/api/kontrol/saglik"; do
  /usr/bin/time -p curl -s -o /dev/null "localhost:8787$u"; done
```

## E2E — arayüz ve defter BİRLİKTE test edilir

`web/e2e/` altında Playwright paketi var (`cd web && npm run e2e`). Ayırt edici yanı:
her UI eyleminden sonra **`muhasebeBozulmadi(request)`** çağrılır — yani test yalnız
"buton çalıştı" demez, **para korundu mu, mizan denk mi, ters bakiye doğdu mu** diye sorar.

Yeni bir ekran/akış eklerken E2E testine şunu ekle: eylemi yap, sonra `muhasebeBozulmadi`.
Mutasyon yapmayan (yalnız okuyan) ekranlarda da çağır — okuma defteri bozmamalı.

**İlk koşuşta bir hata buldu:** Havada ekranı `durum=acik` ile fatura listeliyordu, ama
tamamı iade edilmiş faturanın da tahsilatı yoktur — net borcu sıfır olduğu için güvenlik
katmanı reddediyordu (K06). Arayüz, reddedilecek faturayı öneriyordu.
Çözüm: `tahsis_edilebilir=1` filtresi (`gecerli && kalan > 0`).
**Ders: eşleştirme ekranları "açık" değil "tahsis edilebilir" listelemeli.**

API ayakta olmalı (`cargo run -p api`); vite sunucusunu Playwright kendi başlatır.

## Test yazarken

**Varlık değil DAVRANIŞ ölç.** "Her ayda kayıt var mı" testi geçti ama aylar arası 17 kat
çarpıklığı kaçırdı (LCG düşük-bit hatası). Dağılım, oran, değişmez ölç — sabit sayı iddia etme
(hesap planı testi meşru ekleme yüzünden kırılıyordu).
