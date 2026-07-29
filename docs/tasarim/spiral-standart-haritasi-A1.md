# Spiral Standart Haritası — A1

**Amaç:** UFRS WorkSheet düzeltme kaydı (AJE/RJE) ekranını besleyen, standartlar arası tetikleme + senaryo + doğrulama haritası.
**Kaynak:** KGK resmi TMS çevirileri (PDF, /Users/arda/Downloads/), madde numaraları doğrulandı.
**Konvansiyonlar:** Para = kuruş (tamsayı). Kayıtlar top-side (deftere işlenmez). EV geri izleme TMS 12.61A: K/Z kanalı→691, OCI→522.90 (yeniden değerleme) / 549.90 (aktüeryal), doğrudan özkaynak→570/580. EVV=283, EVY=483. `@hedef` = denetçinin seçtiği hesap.
**Tablo sütunları:** Durum | Bacaklar (B=borç, A=alacak) | Kanal | EV | İstisna/Not.

---

## TMS 12 — Gelir Vergileri (OMURGA — WS-EV)

TMS 12 kendi başına senaryo üretmekten çok, **diğer bütün çalışmaların çıktısını tüketen çatı çalışmadır**. Her AJE bir geçici fark doğurur/kapatır; EV bacağı kaynağın kanalını izler (12.61A "backwards tracing").

| Durum | Bacaklar | Kanal | EV | İstisna/Not |
|---|---|---|---|---|
| İndirilebilir geçici fark → EVV doğuşu (karşılık, değer düşüklüğü, NRV vb.) | B: 283 — A: 691 | kz | EVV | 12.24: gelecekte yeterli vergiye tabi kâr **muhtemel** olmalı; 12.56: her dönem gözden geçirilir |
| Vergiye tabi geçici fark → EVY doğuşu (K/Z kaynaklı, örn. reeskont iptali) | B: 691 — A: 483 | kz | EVY | 12.15: bütün vergiye tabi farklar için zorunlu (istisnalar hariç) |
| OCI kaynaklı EVY (TMS 16/40→16.61 yeniden değerleme artışı) | B: 522.90 — A: 483 | oci_522 | EVY | 12.61A(a) + 12.62(a): vergi de OCI'de |
| OCI kaynaklı EVV/EVY (TMS 19 aktüeryal fark) | B: 283 — A: 549.90 (veya tersi) | oci_549 | fark yönüne göre | 12.61A(a); K/Z'ye asla geri sınıflanmaz |
| Doğrudan özkaynak kaynaklı EV (TMS 8 geçmişe dönük düzeltme) | B/A: 283 veya 483 — karşı: 570/580 | özkaynak | yönüne göre | 12.61A(b) + 12.62A(a) |
| Mali zarar / vergi avantajı devri → EVV | B: 283 — A: 691 | kz | EVV | 12.34-36: kullanılabilirlik kanıtı şart; süre sınırı (TR: 5 yıl) değerlendirilir |
| Vergi oranı değişikliği → EV yeniden ölçümü | mevcut 283/483 düzeltilir — karşı bacak kaynağın kanalına göre 691 **veya** 522.90/549.90 | kaynağa göre | — | 12.47: yasalaşmış/yasalaşması kesine yakın oran; 12.60+12.63: OCI'den doğan EV'nin oran etkisi OCI'de düzeltilir |
| EVV tanınabilirlik iptali (kâr projeksiyonu çöktü) | B: 691 — A: 283 | kz | EVV azaltımı | 12.56 |

**İstisnalar (EV hesaplanMAZ — motor engellemeli):**
- 12.15(a): **Şerefiyenin ilk muhasebeleştirilmesi** → EVY yok.
- 12.15(b) / 12.24: **İlk muhasebeleştirme istisnası** — işletme birleşmesi olmayan ve ne muhasebe ne vergi kârını etkileyen işlemden doğan fark → EV yok.
- 12.39 / 12.44: bağlı ortaklık/iştirak yatırımlarının dış farkları — tersine dönme kontrolü işletmedeyse ve öngörülebilir gelecekte dönmeyecekse EVY yok.
- 12.51: EV ölçümü, defter değerinin **geri kazanılma şekline** (kullanım/satış) uygun oran ve matrahla yapılır.
- 12.53: **EV iskonto edilmez** (hard-stop).

---

## TMS 16 — Maddi Duran Varlıklar (WS-GUD-MDV, WS-AMORT)

| Durum | Bacaklar | Kanal | EV | İstisna/Not |
|---|---|---|---|---|
| GUD artışı (ilk kez) | B: @hedef (250-257) — A: 522.90 | oci_522 | EVY | 16.39; 16.36: **sınıfın tamamı** yeniden değerlenir |
| GUD artışı — önceki K/Z azalışını geri çevirme | B: @hedef — A: 649.90 (önceki gider tutarı kadar) + A: 522.90 (aşan kısım) | kz + oci_522 karma | EVY (bacak bazında) | 16.39 son cümle (SİMETRİ): gelir yazılan kısım, daha önce K/Z'ye giden azalışı **aşamaz** |
| GUD azalışı (önceki artış yok) | B: 659.90 — A: @hedef | kz | EVV | 16.40 ilk cümle |
| GUD azalışı — önceki 522 bakiyesi varken | B: 522.90 (bakiye kadar) + B: 659.90 (aşan kısım) — A: @hedef | oci_522 + kz karma | bacak bazında | 16.40 (SİMETRİ): OCI'ye yazılan azalış, o varlığın 522 bakiyesini **aşamaz** |
| Amortisman farkı (faydalı ömür / kıst / bileşen) | B: 632/730.90 veya A: 257 iptali | kz | yönüne göre EVV/EVY | 16.43-47 bileşen amortismanı; 16.51: ömür+kalıntı değer her yıl gözden geçirilir (tahmin değişikliği → TMS 8.36, ileriye dönük) |
| 522 → geçmiş yıl kârına transfer (kullanım/çıkış) | B: 522.90 — A: 570 | özkaynak-içi | EV yok (522'deki net-vergi tutar taşınır) | 16.41: transfer **K/Z üzerinden yapılamaz** (hard-stop); tutar = YD amortismanı − maliyet amortismanı farkı |
| Yeniden değerlenmiş varlığın satışı | 522 bakiyesi 570'e; satış K/Z'si YD defter değerine göre | kz + özkaynak-içi | EVY kapanışı | 16.41 + 16.71 |

**Not:** 16.42 açıkça TMS 12'ye yönlendirir — GUD artış senaryosu seçildiğinde EV bacağı otomatik ve zorunlu.

---

## TMS 36 — Varlıklarda Değer Düşüklüğü (WS-DEGERDUS)

| Durum | Bacaklar | Kanal | EV | İstisna/Not |
|---|---|---|---|---|
| Değer düşüklüğü — yeniden değerlenMEmiş varlık | B: 654/659.90 — A: 257.90 (birikmiş değer düşüklüğü) veya @hedef | kz | EVV | 36.59-61; sonrası amortisman düzeltilir (36.63) |
| Değer düşüklüğü — yeniden değerlenmiş varlık | B: 522.90 (o varlığın YD fazlası kadar) + B: 659.90 (aşan) — A: @hedef | oci_522 + kz karma | bacak bazında | 36.60-61: YD azalışı gibi işlenir → **TMS 16.40 simetrisi devreye girer** |
| Değer düşüklüğü iptali — normal varlık | B: @hedef — A: 644/649.90 | kz | EVV kapanışı | 36.114; **TAVAN 36.117**: değer düşüklüğü hiç olmasaydı oluşacak (amortisman düşülmüş) defter değeri aşılamaz |
| Değer düşüklüğü iptali — yeniden değerlenmiş varlık | B: @hedef — A: 649.90 (önceki K/Z zararı kadar) + A: 522.90 (kalan) | kz + oci_522 | bacak bazında | 36.119-120 |
| NYB (nakit yaratan birim) zararı dağıtımı | önce şerefiye, sonra varlıklara oransal | kz | EVV | 36.104; varlık bazında taban: GUD−satış maliyeti / KD / sıfır (36.105) |
| Şerefiye değer düşüklüğü | B: 659.90 — A: 261/şerefiye | kz | **EV YOK** | 12.15(a) + **36.124: şerefiye iptali YASAK** |

**Girdi kuralları:** 36.9-14 göstergeler; 36.55 iskonto oranı **vergi öncesi**; KD hesabında gelecekteki yeniden yapılandırma/iyileştirme çıkışları hariç (36.44).

---

## TMS 37 — Karşılıklar, Koşullu Borç ve Varlıklar (WS-KARSILIK)

| Durum | Bacaklar | Kanal | EV | İstisna/Not |
|---|---|---|---|---|
| Karşılık ayrılması (dava, GSM, iade, çevre) | B: 659/689.90 — A: 379/479 | kz | EVV (VUK'ta gider değil) | 37.14 üç koşul: mevcut yükümlülük + muhtemel çıkış + güvenilir tahmin. Kıdem tazminatı BURADA DEĞİL → TMS 19 |
| İskonto etkisi (uzun vadeli karşılık) | karşılık bugünkü değerle ölçülür; dönemsel çözülme B: 661.90 — A: 479 | kz | EVV artışı | 37.45; oran **vergi öncesi** (37.47) |
| Karşılık iptali/azaltımı | B: 379/479 — A: 644.90 | kz | EVV kapanışı | 37.59: her dönem gözden geçirme; yükümlülük kalktıysa iptal zorunlu |
| Ekonomik açıdan dezavantajlı sözleşme | B: 659.90 — A: 379 | kz | EVV | 37.66; önce sözleşmeye tahsisli varlıklarda TMS 36 testi (37.69) |
| Tazminat/rücu varlığı | B: 136.90 — A: 649.90 | kz | ayrı değerlendirilir | 37.53: **ayrı varlık**, karşılıktan netleştirilmez; tavan = karşılık tutarı |

**Yasaklar:** 37.63 gelecek faaliyet zararına karşılık YASAK. 37.36 en iyi tahmin; beklenen değer/en muhtemel sonuç yöntemi belgelenir. Koşullu borç (muhtemel değil) → kayıt YOK, sadece dipnot (37.27-28).

---

## TMS 19 — Çalışanlara Sağlanan Faydalar (WS-KIDEM)

| Durum | Bacaklar | Kanal | EV | İstisna/Not |
|---|---|---|---|---|
| Kıdem karşılığı ilk kuruluş / cari hizmet maliyeti | B: 632/770.90 — A: 472/372 | kz | EVV | 19.57 adımlar; PUC yöntemi (19.67) |
| Faiz maliyeti (net tanımlanmış fayda borcu üzerinden) | B: 661.90 — A: 472 | kz | EVV artışı | 19.123: dönem başı iskonto oranı |
| Aktüeryal kazanç/kayıp (yeniden ölçüm) | B/A: 472 — karşı: 549.90 | oci_549 | yönüne göre EVV/EVY | 19.120(c)+19.122+19.127-128: **OCI zorunlu**, K/Z'ye geri sınıflama YASAK |
| Geçmiş hizmet maliyeti (plan değişikliği) | B: 632.90 — A: 472 | kz | EVV | 19.103: derhal gider |
| Ödeme/fesih (dönem içi tazminat ödemeleri) | B: 472/372 — A: karşılık çözümü | kz | EVV azalışı | fiili ödeme VUK'ta gider → geçici fark kapanır |
| İzin karşılığı, ikramiye tahakkuku (kısa vadeli) | B: 770/632 — A: 372.90 | kz | EVV | 19.11-16; iskonto edilmez |

---

## TMS 21 — Kur Değişiminin Etkileri (WS-KUR)

| Durum | Bacaklar | Kanal | EV | İstisna/Not |
|---|---|---|---|---|
| Parasal kalem kapanış kuru düzeltmesi (102/120/320/300 döviz) | B/A: @hedef — karşı: 646.90 / 656.90 | kz | genelde EV yok (TR'de kur farkı vergisel olarak da tanınır); değerleme farkı varsa EVV/EVY | 21.23(a) kapanış kuru; 21.28 K/Z |
| Parasal olmayan kalem — tarihi kur kontrolü | hatalı çevrim iptali: B/A: @hedef — karşı: 646/656 iptal | kz | yönüne göre | 21.23(b): stok/MDV/avanslar **tarihî kurla kalır** (hard-stop: kapanış kuruyla çevrilemez) |
| GUD'la ölçülen parasal olmayan kalem kur farkı | GUD kazancının kanalını izler | kaynak kanal (oci_522 veya kz) | kaynağa göre | 21.30: kazanç OCI'deyse kur etkisi de OCI'de |
| Yurtdışı işletmede net yatırım kur farkı (konsolide) | B/A: yatırım — karşı: 549/çevrim farkları özkaynak | oci (çevrim farkı) | 12.39 istisnası değerlendirilir | 21.32; solo tabloda K/Z, konsolidede OCI |

---

## TMS 23 — Borçlanma Maliyetleri (WS-BORCLANMA)

| Durum | Bacaklar | Kanal | EV | İstisna/Not |
|---|---|---|---|---|
| Özellikli varlığa faiz aktifleştirme (TFRS gereği) | B: 258/151/@hedef — A: 660.90 iptali | kz (gider iptali) | EVY | 23.8 zorunlu; 23.12: özel kredi = fiilen katlanılan − geçici plasman geliri |
| Genel borçlanma — aktifleştirme oranı | aynı şekil; tutar = ort. harcama × oran | kz | EVY | 23.14: aktifleştirilen tutar **dönem borçlanma maliyetini aşamaz** (hard-stop) |
| VUK'ta aktifleştirilmiş faizin TFRS'de giderleştirilmesi (özellikli değil / tamamlanma sonrası) | B: 660.90 — A: @hedef (253/258) | kz | EVV | 23.22-25? — aktifleştirme, varlık amaçlanan kullanıma **hazır olduğunda durur** (23.22); sonrası faiz gider |
| Aktifleştirmeye ara verme | uzayan duraklamada faiz gider | kz | EVV | 23.20-21 |
| Kur farkının faiz düzeltmesi sayılan kısmı | sınırlı aktifleştirme | kz | yönüne göre | 23.6(e): kur farkı **yalnızca faiz maliyeti düzeltmesi niteliğindeki kısmı** kadar aktifleştirilir → TMS 21 bağlantısı |

---

## TMS 2 — Stoklar (WS-NRV, WS-URETIM)

| Durum | Bacaklar | Kanal | EV | İstisna/Not |
|---|---|---|---|---|
| NRV karşılığı (maliyet > NGD) | B: 654.90 — A: 158 | kz | EVV (VUK 278 takdir komisyonu yoksa KKEG) | 2.9 + 2.28: kalem bazında; NGD = tahmini satış − tamamlanma − satış giderleri (2.6) |
| NRV iptali | B: 158 — A: 644.90 | kz | EVV kapanışı | 2.33: iptal **önceki karşılık tutarını aşamaz** (hard-stop) |
| Maliyet dışı unsur ayıklama (anormal fire, depolama, genel yönetim, satış gid.) | B: 632/659.90 — A: 151/152 | kz | EVV | 2.16: bu kalemler stok maliyetine **giremez** (hard-stop) |
| Sabit GÜG normal kapasite düzeltmesi | boş kapasite → B: 680.90 — A: 151/152 | kz | EVV | 2.13: düşük kapasitede birim yük artırılamaz |
| Vade farkının stoktan ayrıştırılması | B: @hedef stok azalt — A: 780/finansman | kz | yönüne göre | 2.18: uzun vadeli alımda fark finansman gideridir |
| 7/A akış kontrolü (711/721/731 yansıtma tutarlılığı) | RJE — tutar kaydırma | — | — | sektör çekirdeği: üretimde 71x zorunlu |

---

## TMS 40 — Yatırım Amaçlı Gayrimenkuller (WS-GUD-YAGM)

| Durum | Bacaklar | Kanal | EV | İstisna/Not |
|---|---|---|---|---|
| GUD modeli — değer artışı | B: @hedef (252.90/YAG) — A: 649.90 | **kz** (522 KULLANILAMAZ) | EVY | 40.35: GUD farkı **oluştuğu dönemde K/Z'ye** (hard-stop: OCI yasak) |
| GUD modeli — değer azalışı | B: 659.90 — A: @hedef | kz | EVV | 40.35 |
| GUD modelinde amortisman iptali | B: 257 — A: 730/632 iptal | kz | EVY | GUD modelinde amortisman ayrılmaz (hard-stop) |
| Sahibi kullanımından YAG'a transfer (GUD'a geçiş) | fark **TMS 16.39-40 gibi**: artış → 522.90, azalış → 659 | oci_522 / kz | EVY / EVV | 40.61-62: transfer günü farkı TMS 16 rejimiyle işlenir — tek OCI istisnası budur |
| YAG'dan sahibi kullanımına / stoğa transfer | transfer günü GUD = yeni tayin edilmiş maliyet | — | devam eden EV taşınır | 40.57: transfer yalnız **kullanım değişikliği kanıtıyla** (hard-stop) |

---

## TMS 8 — Muhasebe Politikaları, Tahmin Değişiklikleri ve Hatalar

| Durum | Bacaklar | Kanal | EV | İstisna/Not |
|---|---|---|---|---|
| Politika değişikliği — geçmişe dönük | B/A: ilgili bilanço hesabı — karşı: 570/580 | doğrudan özkaynak | 12.61A(b): EV de 570/580'e | 8.19, 8.22; ölçülemiyorsa en erken uygulanabilir dönemden (8.23-25) |
| Geçmiş yıl hatası düzeltmesi | B/A: ilgili hesap — karşı: 570/580 | doğrudan özkaynak | aynı kanal | 8.42; **8.46: hata cari dönem K/Z içinde düzeltilemez** (hard-stop) |
| Tahmin değişikliği (ömür, kalıntı, karşılık oranı) | yalnız cari + gelecek dönem etkisi | kz | yönüne göre | 8.36-38: **ileriye dönük**; geçmişe dönük uygulanamaz (hard-stop) |

**Karar kuralı (form için):** hata mı tahmin mi? Bilgi o tarihte mevcut ve elde edilebilir idiyse = hata (570/580); değilse = tahmin (cari K/Z).

---

## TMS 10 — Raporlama Döneminden Sonraki Olaylar

| Durum | Bacaklar | Kanal | EV | İstisna/Not |
|---|---|---|---|---|
| Düzeltme gerektiren olay (dava sonucu, müşteri iflası, NRV kanıtı, hile) | ilgili çalışmaya YÖNLENDİRİR: TMS 37 karşılık / TFRS 9 ECL / TMS 2 NRV / TMS 36 | ilgili çalışmanın kanalı | ilgili çalışmaya göre | 10.8-9: dönem sonu koşulunun kanıtı → düzelt |
| Düzeltme gerektirmeyen olay (dönem sonrası piyasa/kur düşüşü, yangın) | kayıt YOK; önemliyse dipnot | — | — | 10.10-11 (hard-stop: kayıt önerilmez) |
| Dönem sonrası ilan edilen temettü | borç kaydı İPTAL edilir | RJE | — | 10.12-13: dönem sonunda yükümlülük değildir (hard-stop) |
| Süreklilik varsayımı çöküşü | tüm tablolar esastan değişir — uyarı | — | — | 10.14-16: düzeltmeyle çözülmez, muhasebe esası değişir |

---

## TMS 28 — İştiraklerdeki Yatırımlar (Özkaynak Yöntemi)

| Durum | Bacaklar | Kanal | EV | İstisna/Not |
|---|---|---|---|---|
| İştirak K/Z payı | B: 242/243 — A: 640.90 (zararsa tersi: 659.90/A: 242) | kz | 12.39/44 istisnası değerlendirilir | 28.10; zarar payı **defter değerini sıfırın altına indiremez** (28.38-39, hard-stop) |
| İştirakten temettü | B: 132/102 — A: 242 (defter değerinden düşülür) | — | — | 28.10: **temettü gelir yazılamaz** (hard-stop; VUK 640 kaydı iptal edilir → RJE) |
| İştirak OCI payı | B/A: 242 — karşı: 549.90 | oci_549 | kaynağa göre | 28.10 son cümle |
| Maliyet→özkaynak yöntemine geçiş (TFRS açılışı) | B: 242 — A: 570 (birikmiş pay) + cari yıl 640 | özkaynak + kz | değerlendirilir | geçiş etkisi açılış özkaynağına |
| İştirakte değer düşüklüğü göstergesi | tek tutar olarak TMS 36 testi | kz | EVV | 28.40-42: şerefiye ayrıştırılmaz, iptal mümkün |

---

## İkincil standartlar (kısa)

- **TMS 20 Devlet Teşvikleri:** gelir yaklaşımı zorunlu — teşvik **doğrudan özkaynağa yazılamaz** (20.12, hard-stop). Varlığa bağlı teşvik: ertelenmiş gelir (B: 102/136 — A: 382/482.90, kz'ye sistematik çözüm) veya varlıktan indirim. EV: erteleme farkı → EVV/EVY.
- **TMS 38 Maddi Olmayan Duran Varlıklar:** araştırma gideri aktifleştirilemez (38.54, hard-stop); geliştirme yalnız 38.57'nin 6 koşuluyla aktifleşir; **işletme içi yaratılan şerefiye/marka/müşteri listesi aktifleştirilemez** (38.48, 38.63, hard-stop). VUK'ta aktifleştirilmiş Ar-Ge'nin TFRS'de giderleştirilmesi: B: 630.90 — A: 263 → kz, EVV. Yeniden değerleme yalnız aktif piyasa varsa (38.75) — TR'de nadir, formda uyarı.
- **TMS 29 Yüksek Enflasyon (WS-ENFLASYON, uyuyan):** uygulanıyorsa **tüm diğer çalışmalardan önce** parasal olmayan kalemler endekslenir; net parasal pozisyon kazancı/kaybı → 648/658, kz. Endeksleme farkının EV etkisi 12.61A'ya göre kaynak kanalını izler.
- **TMS 32 Finansal Araçlar — Sunum:** bileşik aracın özkaynak bileşeni ilk muhasebeleştirmede ayrıştırılır → doğrudan özkaynak; EV: 12.23 + 12.62A(b). Geri alınan paylar özkaynaktan düşülür, K/Z yazılamaz (32.33, hard-stop).
- **TMS 41 Tarımsal Faaliyetler:** canlı varlık GUD−satış maliyeti; fark **K/Z'ye** (41.26, 522 kullanılamaz — TMS 16 istisnası yalnız taşıyıcı bitkilerde). Sektör profili tarımsa aktifleşir.
- **TMS 24 / 26 / 27 / 33 / 34:** kayıt üretmez — sunum/açıklama katmanı. TMS 24 → WS-RECLASS ilişkili taraf ayrımını besler. TMS 33 EPS **tüm kâr etkili çalışmalar bittikten sonra** hesaplanır. TMS 7 nakit akış tablosu türetilir, AJE almaz.
- **TMS 39:** TFRS 9 ile ikame — WS-REESKONT/WS-ECL zaten TFRS 9 üzerine kurulu; TMS 39 yalnız makro korunma kalıntısı için.

---

## Spiral Bağlantı Grafiği

### Tetikleme listesi (X → Y: X'teki işlem Y'yi tetikler)

| Tetikleyen | Tetiklenen | Mekanizma |
|---|---|---|
| TMS 16 GUD artış/azalış | TMS 12 | 16.42 → 12.61A: OCI kanalında EVY/EVV |
| TMS 16 yeniden değerleme | TMS 36 | yeniden değerlenmiş varlığın değer düşüklüğü 36.60-61'e göre 16.40 rejimiyle işlenir |
| TMS 36 değer düşüklüğü/iptali | TMS 16.39-40 | simetri: OCI↔K/Z dağılımı varlığın 522 geçmişine bakar |
| TMS 36 zarar/iptal | TMS 16.43+36.63/121 | sonraki amortisman yeni defter değerinden yeniden hesaplanır → WS-AMORT |
| TMS 36, 37, 19, 2, 40, 21, 23, 8 | TMS 12 | her geçici fark → WS-EV (çatı) |
| TMS 21 kur farkı (özellikli varlık kredisi) | TMS 23 | 23.6(e): kur farkının yalnız faiz-düzeltmesi kısmı aktifleşir |
| TMS 23 aktifleştirme | TMS 16 / TMS 2 / TMS 38 | maliyet tabanı değişir → amortisman ve NRV testi yeniden |
| TMS 40 transfer (kullanımdan YAG'a) | TMS 16.39-40 | 40.61-62: transfer günü farkı yeniden değerleme rejimi |
| TMS 40 GUD farkı | TMS 12 | kz kanalında EVY/EVV |
| TMS 19 aktüeryal fark | TMS 12 | 12.61A: oci_549 kanalında EV |
| TMS 10 düzeltme gerektiren olay | TMS 37 / TMS 2 / TMS 36 / TFRS 9 | dönem sonrası kanıt ilgili ölçüm çalışmasını yeniden açar |
| TMS 8 hata/politika | tüm çalışmalar | açılış bakiyeleri (570/580) değişir → cari dönem çalışmaları yeni açılıştan başlar |
| TMS 37 dezavantajlı sözleşme | TMS 36 | 37.69: önce sözleşme varlıklarında değer düşüklüğü testi |
| TMS 2 NRV kanıtı (dönem sonrası satış) | TMS 10.9 | düzeltme gerektiren olay örneği |
| TMS 28 zarar payı sıfırlama | TMS 37 | yasal/zımni yükümlülük varsa fazla zarar için karşılık |
| TMS 29 (aktifse) | HEPSİ | endeksleme diğer tüm ölçümlerin girdisini değiştirir |

### Önerilen çalışma sırası (topolojik)

```
0. TMS 29 (yalnız aktifse — her şeyden önce)
1. TMS 8  → açılış düzeltmeleri (hata/politika, 570/580)     [WS: açılış]
2. TMS 10 → dönem sonrası olay taraması (diğerlerine girdi)
3. TMS 21 → kur düzeltmeleri (parasal kalemler)               [WS-KUR]
4. TMS 23 → aktifleştirme (maliyet tabanını sabitler)         [WS-BORCLANMA]
5. TMS 2 / TMS 16-AMORT / TMS 38 → maliyet esaslı ölçümler    [WS-NRV, WS-URETIM, WS-AMORT]
6. TMS 16-GUD / TMS 40 → yeniden değerleme / GUD              [WS-GUD-MDV, WS-GUD-YAGM]
7. TMS 36 → değer düşüklüğü (GUD SONRASI; 522 geçmişi hazır)  [WS-DEGERDUS]
8. TMS 19 / TMS 37 / TFRS 9 → karşılıklar                     [WS-KIDEM, WS-KARSILIK, WS-ECL]
9. TMS 28 → özkaynak yöntemi (iştirakin kendi TFRS'i bitmiş olmalı)
10. TMS 12 → ERTELENMİŞ VERGİ — EN SON, çatı                  [WS-EV]
11. TMS 33 → EPS (kâr kesinleşince); TMS 1/24 → sunum/reclass [WS-RECLASS]
```

**Döngü uyarısı:** 16↔36 döngü değil, sıra bağımlılığıdır (önce GUD, sonra değer düşüklüğü; iptalde 522 geçmişi sorgulanır). 21↔23 için: önce kur farkı hesaplanır, 23.6(e) sınırı aktifleştirmede uygulanır. WS-EV'in "TEKRARLA"/yeniden üretimi: herhangi bir 1-9 çalışması değişirse WS-EV geçersiz sayılıp yeniden koşulmalı (motor: kirli-bayrak).

---

## Doğrulama Kuralları Kataloğu

| # | Kural | Standart md. | Tip |
|---|---|---|---|
| V01 | Yeniden değerleme artışı doğrudan gelire (K/Z) yazılamaz — yalnız önceki K/Z azalışını geri çevirdiği tutar kadar gelir olur | TMS 16.39 | hard-stop |
| V02 | Yeniden değerleme azalışının OCI'ye (522) yazılan kısmı, o varlığın 522 bakiyesini aşamaz | TMS 16.40 | hard-stop |
| V03 | 522'den geçmiş yıl kârlarına transfer K/Z üzerinden yapılamaz (649 bacaklı transfer engellenir) | TMS 16.41 | hard-stop |
| V04 | Yeniden değerleme sınıf bazında yapılır — aynı sınıftan tek varlık seçilirse uyar | TMS 16.36 | uyarı |
| V05 | Değer düşüklüğü iptali, zarar hiç olmasaydı oluşacak (amortismanlı) defter değerini aşamaz | TMS 36.117 | hard-stop |
| V06 | Şerefiye değer düşüklüğü iptal edilemez | TMS 36.124 | hard-stop |
| V07 | Şerefiye ilk muhasebeleştirmesinden EVY hesaplanamaz | TMS 12.15(a) | hard-stop |
| V08 | İlk muhasebeleştirme istisnası: ne muhasebe ne vergi kârını etkileyen işlem farkına EV kurulamaz | TMS 12.15(b), 12.24 | hard-stop |
| V09 | Ertelenmiş vergi iskonto edilemez | TMS 12.53 | hard-stop |
| V10 | EVV, gelecekte yeterli vergiye tabi kâr muhtemel değilse tanınamaz; dayanak (projeksiyon) zorunlu | TMS 12.24, 12.56 | uyarı + dayanak zorunlu |
| V11 | EV oranı: raporlama günü yasalaşmış/kesine yakın oran; geri kazanım şekline uygun | TMS 12.47, 12.51 | uyarı |
| V12 | OCI kaynaklı kalemin EV'si K/Z (691) kanalına yazılamaz; K/Z kaynaklının EV'si 522/549'a yazılamaz | TMS 12.61A | hard-stop |
| V13 | Karşılık iskonto oranı vergi ÖNCESİ olmalı | TMS 37.47 | hard-stop |
| V14 | Gelecekteki faaliyet zararları için karşılık ayrılamaz | TMS 37.63 | hard-stop |
| V15 | Rücu/tazminat varlığı karşılıktan netleştirilemez; tavanı karşılık tutarıdır | TMS 37.53 | hard-stop |
| V16 | Muhtemel olmayan yükümlülüğe (koşullu borç) kayıt atılamaz — dipnot önerilir | TMS 37.27 | hard-stop |
| V17 | Aktüeryal kazanç/kayıp K/Z'ye yazılamaz; OCI'de birikenler K/Z'ye geri sınıflanamaz | TMS 19.120(c), 19.122 | hard-stop |
| V18 | Kıdem tazminatı TMS 37 çalışmasında değil TMS 19 çalışmasında işlenir | TMS 19.8, 37.5 | hard-stop (yönlendirme) |
| V19 | Parasal olmayan kalem (stok, MDV, alınan/verilen avans) kapanış kuruyla çevrilemez | TMS 21.23(b) | hard-stop |
| V20 | GUD'la ölçülen parasal olmayan kalemin kur etkisi, GUD kazancının kanalını izlemek zorunda | TMS 21.30 | hard-stop |
| V21 | Genel borçlanmadan aktifleştirilen tutar, dönemde katlanılan toplam borçlanma maliyetini aşamaz | TMS 23.14 | hard-stop |
| V22 | Varlık amaçlanan kullanıma hazır olduktan sonra faiz aktifleştirilemez | TMS 23.22 | hard-stop |
| V23 | Kur farkının yalnız faiz-düzeltmesi niteliğindeki kısmı aktifleştirilebilir | TMS 23.6(e) | uyarı (hesap kağıdı zorunlu) |
| V24 | NRV karşılığı iptali önceki karşılık tutarını aşamaz | TMS 2.33 | hard-stop |
| V25 | Anormal fire, depolama, genel yönetim ve satış giderleri stok maliyetine dahil edilemez | TMS 2.16 | hard-stop |
| V26 | Sabit GÜG dağıtımı normal kapasiteye göre; düşük kapasitede birim maliyet şişirilemez | TMS 2.13 | uyarı |
| V27 | YAG GUD farkı OCI'ye (522) yazılamaz — K/Z zorunlu | TMS 40.35 | hard-stop |
| V28 | GUD modelindeki YAG'a amortisman ayrılamaz | TMS 40.33-35 | hard-stop |
| V29 | YAG transferi yalnız kanıtlanmış kullanım değişikliğiyle yapılır | TMS 40.57 | hard-stop (dayanak zorunlu) |
| V30 | Sahibi kullanımından YAG'a geçiş farkı TMS 16.39-40 rejimiyle işlenir (tek OCI istisnası) | TMS 40.61-62 | motor kuralı |
| V31 | Geçmiş yıl hatası cari dönem K/Z'sinde düzeltilemez — 570/580 zorunlu | TMS 8.42, 8.46 | hard-stop |
| V32 | Tahmin değişikliği geçmişe dönük uygulanamaz — yalnız cari+gelecek | TMS 8.36-38 | hard-stop |
| V33 | Raporlama sonrası ilan edilen temettü dönem sonunda borç kaydedilemez | TMS 10.12-13 | hard-stop |
| V34 | Düzeltme gerektirmeyen dönem sonrası olaya kayıt atılamaz (dipnot önerilir) | TMS 10.10 | hard-stop |
| V35 | Özkaynak yönteminde iştirak temettüsü gelir yazılamaz — defter değerinden düşülür | TMS 28.10 | hard-stop |
| V36 | İştirak zarar payı yatırımın defter değerini (uzun vadeli paylar dahil) sıfırın altına indiremez | TMS 28.38-39 | hard-stop |
| V37 | Araştırma safhası giderleri ve işletme içi şerefiye/marka aktifleştirilemez | TMS 38.54, 38.48, 38.63 | hard-stop |
| V38 | Devlet teşviki doğrudan özkaynağa yazılamaz — gelir yaklaşımı zorunlu | TMS 20.12 | hard-stop |
| V39 | Değer düşüklüğü KD hesabında iskonto oranı vergi öncesi olmalı | TMS 36.55 | hard-stop |
| V40 | Değer düşüklüğü/iptal sonrası amortisman yeni defter değerinden yeniden hesaplanır (WS-AMORT tetiklenir) | TMS 36.63, 36.121 | motor kuralı |
| V41 | WS-EV, 1-9 sırasındaki herhangi bir çalışma değiştiğinde geçersiz sayılır ve yeniden üretilir | TMS 12 (çatı deseni) | motor kuralı (kirli-bayrak) |
| V42 | Her AJE'de dayanak + denetçi notu zorunlu; GUD ölçümlerinde SPK lisanslı ekspertiz beklenir | proje kuralı + TFRS 13 hiyerarşisi | hard-stop (form) |

---
*Hazırlayan: A1 analiz oturumu, 2026-07-17. Kaynak PDF metin dökümleri: scratchpad/tms/ (geçici).*
