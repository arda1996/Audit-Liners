# Kayıt Sistemi Yeniden Kuruluş — Sentez Planı (A1+A2+A3)

**Tarih:** 2026-07-17 · **Girdi raporları:** `spiral-standart-haritasi-A1.md` (45 senaryo, V01-V42 kural kataloğu),
`denetci-pratikleri-arastirma-A2.md` (13 kontrol mekanizması, KGS'li), `kayit-sistemi-teshis-A3.md` (Üç Yarım Yol teşhisi).
**Karar bekliyor:** faz onayı kullanıcıda — "her noktayı birlikte" ilkesi.

---

## 1. Döngünün adı kondu: ÜÇ YARIM YOL

Üç bağımsız rapor aynı noktada birleşti. Sistem "kayıt atılamıyor" diye değil, **kayıt üç yarım
yoldan atılıyor** diye döngüde:

| Hat | Kapsam | Üretir | Üretmez |
|---|---|---|---|
| Hesapla motoru | 5/18 | tutar + satır + baz | **EV bacağı** |
| Senaryo motoru | 3/18 | kayıt şekli + EV | **tutar**, hedef doğrulama |
| Statik çipler | 18/18 | hesap kodu | yön, tutar, karşı bacak |

Üstüne üç yapısal hata:
1. **`@hedef` anlam kayması:** UI "kayıt hangi hesap İÇİN" der, motor "bacak hangi hesaba İŞLENİR"
   diye kullanır → ECL'de 120 seçen denetçi karşılık (129.90) yerine alacağın kendisini siler.
2. **`hesap-kurallari.json` atıl:** 83 hesabın doğa/karşı-bacak/kapatma bilgisi yüklü ama UFRS
   hattı hiç okumuyor.
3. **"+ Kayıt (pencere)" yapısal çıkmaz:** modal, backend'in zorunlu kıldığı değerleme-bazı alanını
   hiç göstermiyor → o hattan her kayıt hata alır; çalışma seçili değilse buton sessizce ölür.
   **"Hep aynı darboğaz" hissinin en somut kaynağı.**

**Sonuç:** sorun tek tek özellik eksikliği değil; üç hattın TEK BORU HATTINDA birleşmemesi.
Yeni özellik eklemek döngüyü büyütür — önce birleştirme.

---

## 2. Hedef mimari: TEK BORU HATTI (senaryo omurga)

```
Denetçi akışı (her çalışmada AYNI):
1. ÇALIŞMA      → spiral sıraya göre (TMS 29→8→10→21→23→2/16A→16G/40→36→19/37/ECL→28→12→33/1)
2. SENARYO      → durum seç (45 senaryo kataloğu, A1); hesap arama YOK
3. HEDEF        → senaryonun İZİN VERDİĞİ hesaplardan seç (öneklerle süzülür); karşı bacaklar
                  senaryodan gelir — @hedef sadece "ölçülen varlık/borç" bacağını doldurur
4. TUTAR        → motoru olan çalışmada motor FARK'ı önerir (girdi bakiyesinden); yoksa girdi
                  tablosundaki net bakiye ön-dolar; elle değişiklik = izli müdahale
5. EV           → senaryonun ev_kanal/ev_yon'una göre otomatik bacak (12.15 istisnaları hariç);
                  motor hattı da EV üretir (bugünkü eksik kapanır)
6. DOĞRULAMA    → V01-V42 kural motoru: hard-stop'lar kaydı keser, uyarılar gerekçeyle geçilir
7. TASLAK       → kayıt "önerildi" doğar; denetçi KABUL/DÜZENLE/RED der
                  RED → SUD havuzu (BDS 450 düzeltilmemiş yanlışlıklar özeti bedavaya)
                  KABUL → WTB'ye akar (yalnız kabul edilen akar — bugün önerildi direkt akıyor, yanlış)
```

**Veri modeli düzeltmesi (kök):** senaryo bacaklarında `@hedef` yalnız ölçüm bacağında kullanılır;
`hedef_onekleri` alanı eklenir (örn. ECL: `["12"]` değil `["128","129"]` DEĞİL — hedef 120/121 ALACAK
değil; ECL'de hedef "hangi alacak portföyü İÇİN" bilgisine döner, bacak sabittir: 654.90/129.90).
Yani senaryoya iki ayrı kavram: **`konu`** (hangi kalem için — raporlama/izleme) ve **`bacaklar`**
(hangi hesaplara — sabit veya @hedef). Çipler kataloğu senaryolardan TÜRETİLİR (ayrı el-bakımlı
liste kalkar → çelişki biter).

---

## 3. Güvenlik katmanları (KGS-ağırlıklı, A2+A3 birleşik)

| Katman | İçerik | KGS | Kaynak |
|---|---|---|---|
| G1 Denge + tip kuralı | borç=alacak (var ✅) + **RJE kâr-nötrlüğü** (B2) + tip-bazlı posting (RJE deftere işlemez ✅ felsefe var, motor kuralı eksik) | 0.85 | Caseware + A3-B2 |
| G2 Standart kuralları | **V01-V42 motoru** — 32 hard-stop, madde numaralı hata mesajı ("TMS 16.40: OCI'ye yazılan azalış 522 bakiyesini aşamaz") | 0.9 | A1 (KGK resmi çeviri) |
| G3 Hesap-yön doğrulama | `hesap-kurallari.json` doğa kontrolü UFRS hattına bağlanır (B1); motor-sahipli hesaplara (283/483/522.90/549.90/691) elle giriş uyarısı | 0.6-0.75 | A3-B1 + Caseware |
| G4 Kapsam + mükerrer | çalışma-hesap kapsam uyumu (B3), aynı ws+dönem+senaryo mükerrer kilidi (B4), kaynak_ws varlık doğrulaması | — | A3 |
| G5 Onay akışı | taslak→kabul/red; kabul edilmeyen WTB'ye akmaz (B5); tek kişilik ofiste self-review uyarı modu | 0.75 | A2 + BDS 220 |
| G6 BDS 230 kilidi | dönem kilidi (var ✅) + rapor tarihi sonrası 60 gün nihai dosya; sonrası salt-ekleme + gerekçe | 0.9 | KGK BDS 230 (mevzuat) |
| G7 İz + damga | created-by damgası, append-only + storno (mimaride var), senaryo ucuna State bağlanması (B8) | 0.85 | A2 + A3-B8 |

Ertelenecekler (orta vade): önemlilik eşiği (B6 — önemlilik modülü yokken anlamlı değil),
JE risk bayrakları, maker-checker zorunlu modu, Excel köprüsü, hash zinciri.

---

## 4. Fazlar (onay bekliyor)

**FAZ 0 — Kanamayı durdur (küçük, hemen):**
- "+ Kayıt (pencere)" modalına yöntem/baz alanları + çalışma-seçilmedi mesajı (D1)
- Çalışma değişiminde tam form reset (D2); hedef değişiminde satır ezme düzeltmesi (D3)
- `serbest` bayrağı tek sezgiye indirilir (D4); başarılı kayıtta öneri paneli temizlenir (D5)
- `ufrs_senaryo` ucuna State + hedef doğrulama (B8)

**FAZ 1 — Veri modeli birleşmesi (kök çözüm):**
- Senaryo şeması v7: `konu` ↔ `bacaklar` ayrımı, `hedef_onekleri`, çipler senaryodan türetilir
- A1 kataloğundan kalan 15 çalışmaya senaryolar yazılır (45 senaryo; JSON işi, kod değil)
- Motor + senaryo birleşir: motor FARK üretir → senaryo tutarına akar → EV bacağı her iki hatta

**FAZ 2 — Kural motoru (G2+G3+G4):**
- V01-V42 backend'e (veri-güdümlü kural tablosu; hard-stop/uyarı ayrımı; madde numaralı mesaj)
- hesap-kurallari.json doğa kontrolü + RJE kâr-nötrlüğü + kapsam/mükerrer kilitleri

**FAZ 3 — Onay akışı + SUD (G5):**
- durum makinesi: önerildi → kabul/red; yalnız kabul WTB'ye; red → SUD havuzu (BDS 450 özeti)
- 522 geçmişi varlık-bazında izlenir (16.39-40/36.60-61 simetri senaryoları için ön şart)

**FAZ 4 — Spiral yönlendirme:**
- Çalışma listesi topolojik sıraya göre durum gösterir (bekliyor/hazır/kirli)
- Tetikleme grafı: WS değişince bağımlı WS'lere "kirli" bayrağı (WS-EV en son, otomatik derleme)

---

## 5. Açık sorular (kullanıcıya)

1. Faz sırası uygun mu — FAZ 0'dan mı başlayalım, yoksa doğrudan FAZ 1 veri modeline mi?
2. ECL gibi çalışmalarda `konu` (hangi portföy için) raporlamada mı kalsın, muavin kırılımına mı insin (129.90.120 gibi)?
3. SUD havuzu FAZ 3'te mi, yoksa önemlilik modülüyle birlikte sonraya mı?
