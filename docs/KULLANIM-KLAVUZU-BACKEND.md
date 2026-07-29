# Audit-Liners Backend Kullanım Kılavuzu (Eğitim Referansı)

> Amaç: backend API sistemlerini uçtan uca belgeleyen, **eğitim/öğretim** için kullanılan canlı kılavuz.
> Her iş bittiğinde ilgili uç buraya kaydedilir. Taban URL (yerel): `http://127.0.0.1:8787`.
> Kimlik: giriş sonrası `Authorization: Bearer <token>` başlığı. Para daima **kuruş (i64)**; tarih **gg.aa.yyyy**.

## 0. Mimari özet
- **Hexagonal:** bağımlılıksız `domain` çekirdeği (çift taraflı defter kuralları) + `api` (axum) adaptörü.
- **Çoklu mükellef:** "aktif çalışma seti + arşiv swap" — aktif mükellefin defteri bellekte, diğerleri arşivde.
- **Kalıcılık yok (bilinçli):** her şey bellekte; sunucu yeniden başlayınca sıfırlanır. Kalıcılık = DB-per-mükellef (docs/tasarim/coklu-mukellef-db).
- **Yetki 3 eksen:** rol (admin/kullanici) × departman (görünürlük) × kademe (işlem yetkisi) + SoD.
- **Doktrinler:** kesin fiş DEĞİŞMEZ (VUK 217 iptal); iki eksen (vergi ≠ bağımsız denetim); beyanname = kayıt dayanaklı + manuel müdahale izli.

## 1. Kimlik & oturum
| Uç | Yöntem | Amaç | Yetki |
|----|--------|------|-------|
| `/api/giris` | POST | `{kullanici_adi, sifre}` → `{token, kullanici}`; hatalı 401 | herkes |
| `/api/oturum` | GET | Bearer token'ı doğrula → kullanıcı; geçersiz 401 | oturum |
| `/api/cikis` | POST | token'ı düşür | oturum |

Tohum yönetici: **admin / audit2026** (rol admin, departman YONETIM, kademe YONETICI). Parola özeti şu an
std DefaultHasher (GEÇİCİ) — kalıcılıkta argon2/bcrypt. **Kayıt yoktur**; kullanıcıları admin açar.

## 2. Kullanıcı yönetimi (admin)
| Uç | Yöntem | Amaç | Yetki |
|----|--------|------|-------|
| `/api/kullanicilar` | GET | tüm kullanıcılar | admin |
| `/api/kullanici` | POST | `{kullanici_adi, ad, sifre, rol, mukellef_idleri, departman, kademe}` → yeni kullanıcı | admin |
| `/api/departmanlar` | GET | departman kataloğu (→ görünür modüller) | oturum |
| `/api/kademeler` | GET | kademe kataloğu (→ izinli işlemler + onay eşiği) | oturum |

## 3. Mükellef & sektör
| Uç | Yöntem | Amaç | Yetki |
|----|--------|------|-------|
| `/api/mukellefler` | GET | kullanıcının GÖREBİLDİĞİ mükellefler + aktif | oturum (admin=hepsi) |
| `/api/mukellef` | POST | `{unvan, vkn, sektor_kodlari, maliyet_secenegi}` → yeni mükellef | admin |
| `/api/mukellef/:id/aktif` | POST | aktif mükellefi değiştir (swap); yetkisiz 403 | atanmış kullanıcı |
| `/api/sektorler` | GET | 10 sektör kataloğu (üretim/7A-7B/stok-maliyet hesapları/nitelik) | oturum |
| `/api/sablonlar` | GET | fiş şablonları (sektör etiketli — frontend süzer) | oturum |

## 4. Hesap planı & muavin
| Uç | Yöntem | Amaç |
|----|--------|------|
| `/api/hesaplar` | GET | yaprak hesaplar (fiş satırı seçimi) |
| `/api/hesap-plani` | GET | ağaç (seviye/sınıf/grup) |
| `/api/hesap/muavin` | POST | `{ana, alt_kod, ad}` → alt hesap (120.01…) |
| `/api/hesap/:kod/aciklama` | GET | MSUGT resmi tanım + işleyiş + karşı bacak + kapatma |
| `/api/hesap/:kod/kume` · `/api/kume` | GET/POST | açıklama kümeleri (koddan bağımlı öneri) |

## 5. Fiş & defterler ⭐ (çekirdek)
| Uç | Yöntem | Amaç | **YETKİ (Y3)** |
|----|--------|------|----------------|
| `/api/fis` | POST | fiş kesinleştir (denge/kronoloji/mükerrer belge/yaprak kontrolleri) | **kademe "kesinlestir" + onay eşiği**; ELEMAN 403 |
| `/api/fisler?limit` | GET | fiş listesi (özet) | oturum |
| `/api/fis/:id` | GET | fiş detayı (satırlar + dayanak) | oturum |
| `/api/fis/:id/iptal` | POST | VUK 217 ters kayıt `{tarih, gerekce}` | **kademe "iptal"** (müdür/yönetici); diğerleri 403 |
| `/api/mizan` | GET | Kebir\|Kod\|Ad\|Açıklama\|Borç\|Alacak\|Borç Bk\|Alacak Bk |
| `/api/yevmiye?limit` | GET | tarih sıralı maddeler + nakli yekûn |
| `/api/kebir/:kod?limit` | GET | hesap hareket dökümü + yürüyen bakiye |
| `/api/muavin` · `/muavin/hareket/:kod` · `/muavin/txt` | GET | muavin özet / sayfalı hareket / TXT export |

**Fiş kesinleştirme kuralları (domain):** Σborç=Σalacak · yaprak hesap · dönem içi tarih · kronoloji (geriye
tarih reddi) · mükerrer belge reddi · ayrı seri + müteselsil yevmiye no. Kesin fiş DEĞİŞMEZ → düzeltme yalnız
iptal (md.217). **Kademe (Y3):** yalnız "kesinlestir" yetkili kademe post edebilir; SORUMLU onay eşiği (500.000 TL),
MÜDÜR sınırsız; iptal yalnız MÜDÜR/YÖNETİCİ. *(Not: maker≠checker tam SoD, taslak CRUD + kullanıcı damgası gelince.)*

## 6. Mali tablolar & analiz
| Uç | Yöntem | Amaç |
|----|--------|------|
| `/api/bilanco` | GET | aktif=pasif; dönem kârı öz kaynağa dahil |
| `/api/gelir-tablosu` | GET | 60 gelir / 62 SMM / 63 faaliyet gid. / 66 finansman / 691 vergi (2-haneli gruba göre gelir-gider ayrımı) |
| `/api/analiz?ay&kiyas` | GET | 14 oran + banka görünümü + kebir bilanço + GT + aylık kur (tx vs tx-1) |

## 7. Vergi yolu (G ekseni)
| Uç | Yöntem | Amaç |
|----|--------|------|
| `/api/kdv` · `/api/kdv-mahsup` | GET/POST | KDV pozisyonu / ay sonu mahsup fişi (191↔391→360/190) |
| `/api/kdv-beyanname?ay` · `/duzenle` | GET/POST | KDV1 taslağı (oran kırılımlı) + manuel müdahale satırı (deftere işlemez) |
| `/api/gecici-vergi?ceyrek` · `/duzenle` | GET/POST | kümülatif geçici vergi kağıdı (ticari kâr + KKEG − istisna) × oran − mahsup |
| `/api/vergi-parametreler` | GET | 10 kanunluk had/oran kataloğu (2026; dogrulandi bayraklı) |

## 8. Denetim (H ekseni — bağımsız denetim)
| Uç | Yöntem | Amaç |
|----|--------|------|
| `/api/denetim/programlar` | GET | 7 sektör × çalışma programı |
| `/api/denetim/kagit/:id` | GET | çalışma kağıdı (ilgili hesaplar defterden + M-motor testleri + MSUGT nitelik) |
| `/api/denetim/kagit/:id/not` · `/duzenle` | POST | SMMM sonuç notu / Excel-benzeri kağıt düzenlemeleri (deftere işlemez, izli) |

## 8a. Duran varlık envanteri (kayıt detayı)
"Kayıt en ince ayrıntısına kadar açıklanmalı" ilkesi: 253.01 Üretim hattı tek satır değil —
onu oluşturan 10 bileşen (CNC torna, robot kaynak, konveyör…) ayrı hesap + kimlik kartı;
254.01 Binek 6 ayrı araç (plaka/marka) + tamamlayıcıları (lastik seti, araç takip).

| Uç | Yöntem | Amaç |
|----|--------|------|
| `/api/varlik/:kod` | GET | Varlık kartları: `253.01` → 10 bileşen; `254.01.001` → tek araç. Her varlıkta marka_model, seri_plaka, edinim, maliyet, vuk_omur, tfrs_omur, tamamlayici_of + defter bakiyesi |

Veri: **data/duran-varlik-envanteri.json** (37 varlık). Hesap kodu 3 seviyeli: `253.01.001`
(kebir.grup.varlık). `.9xx` = tamamlayıcı (ana varlıkla birlikte değerlenir/elden çıkar).
**Satır açıklaması** (`YevmiyeSatiri.aciklama`) fiş satırında zorunlu detay taşır — açılış fişinde
varlık kimliği, bordroda kesinti dökümü, kirada stopaj gerekçesi. UFRS çalışma girdi tablosu
kimliği + VUK/TFRS ömrünü gösterir (WS-AMORT varlık bazında ömür farkı hesaplayabilir).

## 8b. UFRS WorkSheet (I ekseni — VUK→TFRS dönüşüm; görev #26)
Model: **top-side entries** — deftere ASLA işlemez; denetçi çalışmalarından doğan AJE/RJE, WTB'ye akar.
GUD (gerçeğe uygun değer) ekseni esas: her kayıtta **dayanak** (ekspertiz/piyasa/hesaplama/sözleşme/
aktüeryal/teyit) + **denetçi notu** zorunlu. TMS 29 katalogda UYUYAN (şu an uygulanmıyor).

| Uç | Yöntem | Amaç | Yetki |
|----|--------|------|-------|
| `/api/ufrs/calismalar` | GET | çalışma kataloğu (data/ufrs-calismalar.json) + kayıt sayıları | herkes |
| `/api/ufrs/calisma/:id` | GET | tanım + defterden girdi hesapları (önek) + çalışmanın kayıtları | herkes |
| `/api/ufrs/kayit` | POST | yeni AJE/RJE — denge + tek-taraf + **alt kırılım** (serbest değilse yaprak şart; muavin açılmış anaya RED; `ana.alt` deseninde alt yoksa OTOMATİK muavin) + dayanak + not + **degerleme_yontemi + degerleme_bazi (ZORUNLU — kaydın gerekçesi)**; hazırlayan+tarih+dönem+devir SİSTEM | **oturum** |
| `/api/ufrs/calisma/:id/hesapla` | POST | `{parametreler}` → defterden otomatik veri çekip hesapla → `{ara_tablo (denetim izi), uyarilar, onerilen (satırlar+açıklama+dayanak+not_taslağı)}`. Motor: WS-REESKONT (alt hesap PV), WS-ECL (kova matrisi), WS-KIDEM (basit aktüerya), WS-NRV (kalem NRV). Kuruş i64 + ppm; kayıt ATMAZ | herkes |
| `/api/ufrs/kayit/:no/vazgec` | POST | silme YOK — durum=vazgecildi (iz kalır) | oturum |
| `/api/ufrs/kayitlar` | GET | kayıt defteri (JE register) | herkes |
| `/api/ufrs/wtb` | GET | WTB — Big-4 değer zinciri: VUK→AJE→Düzeltilmiş→RJE→CF(devir)→TFRS; sınıf gruplaması + satır başına kayıt ref'leri (delinebilir tutar) + kontrol satırları + kâr köprüsü + dönem/kesinlik | herkes |
| `/api/ufrs/kesinlestir` | POST | aktif dönemi kilitle — kesin döneme kayıt/vazgeç yok; devir kaynağı olur | **kademe "kesinlestir"** |
| `/api/ufrs/devir` | POST | önceki KESİN dönemin AJE'lerini taşı-bırak kuralıyla aktif döneme CF getirir (P/L→570, bilanço aynen; TASIMA_YOK/YENIDEN_URET atlanır; TERS_CEVIR yeniden ölçüm görevi düşer; çift devir kilidi) | oturum |

Numaralama: `AJE-16-01` (standart no gömülü), devir fişleri `CF-001`. Kayıtta `donem` + `devir`
davranışı (katalogdan) sistem atar. Katalog `sektorler` etiketli — uygunluk `uygun` bayrağıyla döner. Her çalışmada **`kayit_hesaplari`**
(borç/alacak bacaklarının STABİL hesap seti — sektör değişse de değişmez; frontend çip olarak sunar) +
**`degerleme_yontemleri`** (uygulanabilir ölçüm yöntemleri; kök `degerleme_yontemleri` kataloğundan).
KÜMÜLE İLKE: firmalar kesintisiz ilerler; kesin dönemin TFRS pozisyonu CF katmanıyla yeni dönemin
açılışına kurulur. Kayıt satırı: TDHP kodu VEYA serbest TFRS kalem adı.
Kâr köprüsü: VUK dönem sonucu (6'lı sınıf net) + AJE 6/7'li satır etkisi = TFRS dönem sonucu.

## 9. Banka & belge (D — eşleştirme)
| Uç | Yöntem | Amaç |
|----|--------|------|
| `/api/banka/ekstre` · `/api/banka` · `/:id/oneri` · `/:id/esle` | POST/GET | ekstre yükle → kayıt önerisi (±3 gün) → eşle |
| `/api/belge` · `/api/belgeler` · `/:id/oneri` · `/:id/esle` | POST/GET | gelen belge kutusu → fişe bağla (±5 gün) |
| `/api/fatura*` | — | fatura yaşam döngüsü (PARK — e-dönüşümle) |

## 10. Yardımcı
| Uç | Yöntem | Amaç |
|----|--------|------|
| `/api/ornek-veri` | POST | boş deftere 20.000 kayıtlık e-ticaret fixture (deterministik) |

## 11. Yetki matrisi (Y2/Y3)
**Departman → görünür modüller** (data/departmanlar.json): MUHASEBE(muhasebe/banka/belgeler/hesaplar) ·
VERGI(vergi/analiz) · BORDRO(bordro/muhtasar) · CARI(cari/banka/belgeler) · DENETIM(denetim/analiz) · YONETIM(*).

**Kademe → işlemler + onay eşiği** (data/kademeler.json):
| Kademe | İşlemler | Eşik |
|--------|----------|------|
| ELEMAN | taslak_gir | — (kesinleştiremez) |
| SORUMLU | taslak_gir, kesinlestir | 500.000 TL |
| MUDUR | + iptal, donem_kapat | sınırsız |
| YONETICI | * | sınırsız |

**Zorlama noktaları:** `/api/fis` (kesinleştir + eşik), `/api/fis/:id/iptal` (iptal); frontend NAV departmana
göre süzülür, aksiyon düğmeleri kademeye göre gizlenir.
