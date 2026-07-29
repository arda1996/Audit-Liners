# Açık Bankacılık (ÖHVPS) + İş Bankası Sandbox — Asset Analizi

> Kaynak: TCMB ÖHVPS standardı (ohvps.github.io v1.0.2), BKM GEÇİT, İş Bankası developer portal.
> Tarih: 2026-07-19. Durum: **bilgi çıkarımı / karar öncesi** — henüz kod yok.
> İlgili karar: [[site-kesif-ve-learnreverse]] — reverse yerine bu MEŞRU kanal.

## 0. Tek cümlelik cevap
"10 farklı bankada hesap → tek API'den hepsinin ekstresi" **mümkün ve Türkiye bunu resmî olarak
kurdu**: **HBHS (Hesap Bilgisi Hizmeti / AIS)** servisi, **BKM GEÇİT** üzerinden **tek entegrasyonla**
tüm bankalara ulaşır. Ama çağırabilmek için **YÖS lisansı** (ya da lisanslı YÖS ortaklığı) şart.

## 1. Roller (kim kimdir)
| Rol | Açılım | Bizim konumumuz |
|-----|--------|-----------------|
| **HHS** | Hesap Hizmeti Sağlayıcısı (bankalar — İş B., Garanti, vb.) | Veriyi biz DEĞİL, bankalar sunar |
| **YÖS** | Yetkili Ödeme Hizmeti Sağlayıcısı (TPP) | **Bizim olmamız gereken rol** (veya ortak) |
| **GEÇİT** | BKM API Gateway — YÖS↔HHS arasında tek kapı | Tek entegrasyon noktamız |
| **ÖHK** | Ödeme Hizmeti Kullanıcısı (mükellef/müşteri) | Rıza veren taraf |

## 2. İki kanal — hangisi?
- **A) BKM GEÇİT (birleşik):** `secure.api.bkm.com.tr/ohvps/...` — **tek bağlantı = tüm bankalar.**
  Çoklu-banka toplulaştırma (mükellefin 10 bankası) için **doğru yol.** YÖS lisansı gerektirir.
- **B) İş Bankası doğrudan API:** developer.isbank.com.tr (prod) / developer.sandbox.isbank.com.tr (test).
  Yalnız İş B. hesapları. Ayrıca **BaaS / Servis Modeli Bankacılığı** (/baas) sunuyor — "banka lisansı
  olmadan kendi uygulamandan bankacılık hizmeti." Sandbox = geliştirme/deneme için açık.

## 2b. İş Bankası Sandbox — GERÇEK API ENVANTERİ (2026-07-19 canlı keşif)
> developer.sandbox.isbank.com.tr — site-keşif + render ile çıkarıldı. Katalog PUBLIC;
> **operasyon detayı (GET/POST + request/response) LOGIN-ARKASI** ("Giriş Yapmalısınız —
> API detayını görebilmek için giriş yapınız"). Şema için kullanıcı girişi + yakalama şart.
> Not: Bunlar İş Bankası'nın KENDİ developer/BaaS API'leri (İş B. hesapları) — ÖHVPS/GEÇİT'ten ayrı.

**6 kategori:** Krediler · Para Aktarma · Veri Paylaşımı · Ödemeler · Hesaplar · Kartlar.
(Para Aktarma / Ödemeler = para hareketi → **kapsam dışı**, girmedik.)

**► Hesaplar (bizim çekirdek):**
| API | Sürüm | Op | Ne yapar | Bizde yeri |
|-----|-------|----|----------|-----------|
| **Hesap Hareketleri API** | V1.2.9 | 1 | Hesap hareketlerini otomatik izleme/raporlama, 3. parti entegrasyon = **EKSTRE** | **D.6/D.7 mutabakat girdisi** |
| **Hesap Bilgileri API** | V2.1.8 | 2 | Müşteri hesaplarını + detaylarını listeler (Accounts) | Hesap/bakiye görünümü · slug `/tr/all-apis/accounts` |
| Hesap Bilgi Sorgulama API | V1.0.0 | 1 | **"İş ortağı firma ile iş birliği kapsamında"** açılan özel hesap bilgileri | ← canlı için **ortaklık/sözleşme** gerektiğinin kanıtı |
| Birikimli Mevduat Hesabı API | V1.1.4 | 2 | Hesap açılış (kapsam dışı) | — |
| Sanal IBAN Yönetimi API | V1.1.6 | 1 | Alt işyeri sanal IBAN (ACH) | ileride tahsilat eşleştirme |

**► Veri Paylaşımı (kur/piyasa + yardımcı):**
Döviz Kurları API **V1.1.10** · Dar Marjlı Döviz V1.0.1 · Kur Çevrimi V1.1.7 · IBAN Doğrulama V1.1.6 ·
Müşteri Bilgileri V1.2.6 · Anonim POS Verisi · Banka/Şube Liste · Bildirim V2.1.7 · Bölgeler ·
DijiKolay (+Finansman) · En Yakın ATM/Şube · HGS (×10: bakiye/geçiş/ekstre/ödeme/satış/sorgu/ürün) · Sadakat Puan.

**Lisans sorusuna dair kanıt:** "Hesap Bilgi Sorgulama" açıklaması canlı erişimin **İş Bankası ile iş
ortaklığı/sözleşme** kapsamında olduğunu gösteriyor — bu, ulusal YÖS lisansından farklı, **bankayla
ticari anlaşma** kapısı. Yani geliştirme serbest; canlıda İş B.'nin onaylı-uygulama/ortaklık süreci var.

## 2c. Portal backend yapısı (giriş yapılmış oturumla çıkarıldı — 2026-07-19)
> IBM API Connect tabanlı. Katalog+ürün meta'sı oturumla `/sandbox/api/*` altından okunuyor.
> Operasyon swagger'ı (GET yolu + request/response + hata kodları) operasyon açılınca çekiliyor.
> NOT: oturum Bearer token + çerezleri yakalanan trafikte görünür — hassas, paylaşılmamalı.

**Portal iç uçları:**
- `GET /sandbox/api/session` — oturum durumu (giriş doğrulama)
- `GET /sandbox/api/groupinfo` — 6 grup: Krediler=21 · Para Aktarma=22 · Veri Paylaşımı=23 · Ödemeler=24 · **Hesaplar=25** · Kartlar=26
- `GET /sandbox/api/products/groupinfo/{grupId}` — gruptaki ürünler *(frontend bug: id yerine `[object Object]` → 400)*
- `GET /sandbox/api/products/{productId}` — ürün detayı (plan, rateLimit, api ref'i)
- `GET /sandbox/api/apis/{apiId}` — API meta (oaiVersion, state)

**Hesaplar grubu ürünleri (gerçek ID'ler):**
| Ürün (name) | Sürüm | productId | apiId | Op |
|-------------|-------|-----------|-------|----|
| **account-transactions** (Hesap Hareketleri/ekstre) | 1.2.9 | 2f9385e2-e1dc-4457-8507-30d49e4fa182 | deposit-transactions `e4da04c3-c683-48ee-896f-0da98d578ab9` | 1 |
| **accounts** (Hesap Bilgileri) | 2.1.8 | a073927e-dbe5-43c0-90de-fb2e1a34348c | — | 2 |
| account-management (Birikimli Mevduat) | 1.1.4 | d49194a0-48d7-4e2c-8968-bf6df46a895d | — | 2 |
| deposit-query-product (Hesap Bilgi Sorgulama) | 1.0.0 | 7e3a2dcd-e9a6-41a1-97ff-511ba1800a23 | — | 1 |
| virtual-iban-management-product (Sanal IBAN) | 1.1.6 | dc8d303a-d73d-40ed-a5bd-ef3ce084f9f7 | — | 1 |

**Hesap Hareketleri (ekstre) planı:** "Limited" — **50 istek / 1 saniye** (İş B.'nin kendi API planı;
ÖHVPS'nin 4/gün kotasından farklı — bu bankanın doğrudan developer planı).

**KALAN:** operasyon swagger'ı (GET/POST yolu + request/response şeması + hata kodları) — operasyon
açılınca çekiliyor; yakalamak için kullanıcının o operasyonu açması yeter (aşağıda).

## 3. HBHS/AIS endpoint'leri (ÖHVPS standardı) — TAM İSTEDİĞİMİZ ŞEY
URI deseni: `[hhs-prefix]/ohvps/hbh/s{versiyon}/{kaynak}`

**Rıza yönetimi (consent):**
- `POST /hesap-bilgisi-rizasi` — rıza oluştur (→ ÖHK bankada GKD/2FA ile onaylar)
- `GET  /hesap-bilgisi-rizasi/{RizaNo}` — rıza durumu
- `DELETE /hesap-bilgisi-rizasi/{RizaNo}` — rızayı iptal et

**Hesap & işlem verisi (read-only):**
- `GET /hesaplar` — erişilebilir tüm hesaplar
- `GET /hesaplar/{hspRef}` — hesap detayı
- `GET /hesaplar/{hspRef}/bakiye` — anlık bakiye
- `GET /hesaplar/{hspRef}/islemler` — **işlem/ekstre geçmişi ← dekont/mutabakatın kaynağı**

> ÖEBH (ödeme emri / para gönderme) endpoint'leri VAR ama **kapsam dışı** — biz yalnız `hbh` (bilgi).

## 4. Yetkilendirme akışı — senin tarif ettiğin akışın meşru hali
- **Client Credentials** (clientId/clientSecret → access token): sistemsel, kullanıcısız çağrılar.
- **Authorization Code + GKD** (Güçlü Kimlik Doğrulama = **senin dediğin 2FA**): ÖHK bankanın
  arayüzüne yönlenir, onaylar, `code` → `access token` → `X-Access-Token` başlığıyla veri çekilir.
  **Tam da "call at → 2FA → veri" akışı — ama bankanın kendi rıza ekranından, forge değil.**

**Kritik başlıklar:** `X-Request-ID` (idempotency), `X-Group-ID` (tek rıza akışı), `X-ASPSP-Code`
(HHS), `X-TPP-Code` (YÖS), `PSU-Initiated` (E=kullanıcı önünde / H=sistem-otomatik).

## 5. ⚠️ HIZ LİMİTLERİ — ürün tasarımını belirler
Sistem-başlatan (PSU-Initiated=**H**, kullanıcı önünde değil) çağrılarda günlük kota:
| Kaynak | Bireysel | Kurumsal |
|--------|----------|----------|
| Hesap listesi | 4/gün | 4/gün |
| Bakiye | 24/gün | 24/gün |
| **İşlem/ekstre** | **4/gün** | **12/saat** |
| Rıza durumu | 4/gün | 4/gün |

→ **Sürekli poll YASAK/kotalı.** Bu, daha önce konuştuğumuz **"talep anında çek + yerel snapshot'ta
çalış"** modelini standart ZORUNLU kılıyor. Kullanıcı önündeyken (PSU-Initiated=E) limitler farklı.
`X-RateLimit-Remaining` / `X-RateLimit-Reset` başlıkları izlenmeli.

## 6. Teknik standartlar
JSON / HTTPS TLS 1.2+ · UTF-8 · ISO 8601 zaman (`yyyy-MM-dd'T'HH:mm:ssXXX`) · ISO 4217 para kodu ·
sürüm `/s{major}.{minor}/` (çoklu sürüm bir süre paralel).

## 7. Audit-Liners'a bağlanışı (asset → nerede kullanılır)
| ÖHVPS asset | Audit-Liners'ta yeri |
|-------------|----------------------|
| `GET /hesaplar/{ref}/islemler` | **D.6/D.7 mutabakat motoru** girdisi (ekstre satırı → fiş eşleştirme) |
| Rıza (consent) akışı | Yeni: mükellef-banka rıza yönetimi UI + rıza saklama |
| `{deger, kaynak, alinma_tarihi}` | Zaten var — ekstre satırı bu üçlüyle kanıt olur (BDS 500) |
| Hız limiti / snapshot | "Talep anında çek" doktrini (session keep-alive tartışması) |
| Çoklu banka (GEÇİT) | Çoklu-mükellef + çoklu-hesap toplulaştırma vizyonu |

## 8. AÇIK SORULAR (login/araştırma gerekir)
1. **YÖS lisansı:** başvuru şartları, sermaye, süre? Ya da **lisanslı YÖS ile ortaklık** modeli?
   (En kritik iş kararı — canlıya çıkışın önkoşulu.)
2. İş Bankası sandbox: onboarding adımları, clientId/secret alma, hangi API paketleri açık?
   → **Login gerektiriyor; kullanıcı `~/.site-kesif-chrome` profilinde giriş yapmalı.**
3. `islemler` yanıtının tam şema alanları (tarih, tutar, karşı hesap, açıklama, referans) —
   D.6 eşleştirme anahtarlarına birebir map için.
4. Sandbox'ta test verisi/örnek ekstre var mı? (Gerçek veriye gerek kalmadan D.6'yı besleyebilir.)
5. GKD (2FA) akışının sandbox'taki taklidi nasıl? Redirect URL / callback yapısı.

## 8b. SENTEZ (2 ajan taraması, 2026-07-19) — bulgular + iş listesi

### Banka bağlanabilirliği
- **Developer API + ekstre + açık sandbox:** İş Bankası, Garanti BBVA, Yapı Kredi (OAuth2), VakıfBank,
  DenizBank, Kuveyt Türk (en olgun), Albaraka, Akbank(mock). Kayıt yeterli; canlı = banka sözleşmesi.
- **Portalı yok ama GEÇİT'te zorunlu:** Ziraat, Halkbank, TEB, ING TR. ÖHVPS gereği 35 HHS + 9 ödeme
  kuruluşu GEÇİT'te; FAST katılımcıları için **31.12.2025 sertifikasyon zorunlu** → 2026'da tüm sektör.
- **Dekont:** ⭐ **VakıfBank "Dekont Sorgula (getReceipt)" — işlem bazında PDF/metin dekont** (envanterdeki
  TEK doğrulanmış dekont API'si). GEÇİT standardında dekont YOK (yalnız işlem satırı, 12 ay geriye,
  maskeli karşı IBAN/unvan). → Dekont için: VakıfBank özel API **veya** mükelleften belge akışı korunur.
- **Yol kararı:** MVP = İş B. + VakıfBank; ürün = **(g) bendi HBHS lisanslı ödeme kuruluşuyla ORTAKLIK**
  (rıza/veri onlarda, eşleştirme/denetim bizde) — YÖS lisansının ağır yükü olmadan GEÇİT erişimi.

### Ekstre → fiş/vergi (kod gerçeği)
- **Domain hazır, API takılı değil:** `ekstre.rs` (EkstreHareket, zincir/devir doğrula, dedup, ters_isle)
  + `mutabakat.rs` ÖHVPS `islemler`'e birebir map olur; ama `api/main.rs:1441 BankaHareket` naif
  (referans/karşı taraf/bakiye yok) ve `banka_oneri` yalnız **var olan fişle** eşliyor — "ekstreden fiş
  ÜRET" akışı (senin %90 dayanağın) hiç yok. `mutabakat`/`ekstre` "uyuyan modül" → bağlanacak.
- **Banka mahsup fişi dayanak boşluğu:** `fis.rs:31` yalnız Tahsil/Tediye'de dayanak zorunlu → banka
  mahsup fişi bugün **dayanaksız kesinleşebilir**; ekstre-kaynaklıda dayanak (dekont/satır ref) zorunlu olmalı.

### ⚠️ Mevzuat düzeltmesi (kalıcı uyar-talimatı gereği)
- **"EFT masrafı üzerinde ÖİV" YANLIŞ.** ÖİV (6802 md.39) **telekom** vergisidir; banka ekstresinde oluşmaz.
  Banka işlemindeki ikinci kesinti **KKDF** (tüketici kredisi faizi %15, vadeli ithalat %6, **ticari kredi %0**).
  SMMM mükellefinde KKDF tipik olarak **ithalat transferinde** görülür → maliyet (VUK 262).
- **BSMV (6802) ≠ KDV:** banka masrafındaki BSMV **191'e ASLA gitmez** (indirilemez maliyet); masrafla aynı
  gider hesabına (770/780). Denetim kuralı: banka fişinde 191 varsa **otomatik bulgu**.
- **Damga çifte-gider riski:** ekstredeki GİB/damga ödemesi **gider değil**, 360 borç kapama (tahakkuk fişi
  zaten gideri yazmıştı) → ekstre satırı "borç kapama" önerir, ikinci gider yazmaz.

### Üçlü mutabakat & kanıt
- API ekstresi = MT940 halefi = bankanın **yapısal beyanı**; mükellemin elinden geçmediği için PDF
  yüklemeden **BDS 500 açısından daha güvenilir** (KGS'de yüksek skor). Dekont hâlâ ayrı belge.
- `fis.rs:74 Dayanak` genişlemeli: `kaynak, alinma_tarihi, hash, ekstre_satir_ref` (+ rıza_no, X-Request-ID).
- **%90 dayanak iddiası kalıcılık (F.2 🔒) olmadan oturum-ömürlü** — bu bağımlılık unutulmamalı.

### D bölümüne önerilen iş kalemleri (öncelik/bağımlılık)
| # | İş | Önc | Bağ |
|---|----|-----|-----|
| D.8 | ÖHVPS/İş B. `islemler`→`EkstreHareket` adaptörü (crates/ingest): şema yakalama, kota disiplini (12/saat), ham JSON+hash | 1 | J.2, sandbox |
| D.9 | `data/islem-tipleri.json` — açıklama→işlem tipi anahtar-kelime taksonomisi (veri-esaslı) | 1 | — |
| D.10 | Taslak fiş üretici (tip→şablon); `hesap-kurallari.json` 102 `karsi` genişletme (642/646/656/770/780/335/360/361); maker-checker | 1 | D.9, B.7 |
| D.11 | BSMV/KKDF/damga/stopaj ayrıştırıcı (ayrı+gömülü satır); stopaj oranları + kambiyo binde 2 doğrula → vergi-parametreleri.json | 2 | D.9 |
| D.12 | `Dayanak` genişletme + banka mahsup fişte dayanak zorunluluğu | 2 | — |
| D.13 | Üçlü mutabakat raporu (`mutabakat()`×2 + fark matrisi: kayıt dışı/dayanaksız/belge eksik) | 2 | D.8, D.12 |
| D.14 | Denetim kuralları (191-BSMV bayrağı, taksit/faiz "ayrıştırma bekliyor", damga çifte-gider koruması) | 3 | D.10, D.11 |
| D.15 | `BankaHareket`→`EkstreHareket` taşı + `banka_oneri`'yi `mutabakat()` motoruna bağla (uyuyan modülü uyandır) | 1 | — |

## 9. Kaynaklar
- ÖHVPS standardı: https://ohvps.github.io/v1.0.2/
- BKM DSSP/GEÇİT: https://bkm.com.tr/en/products-and-services/data-sharing-services-in-the-field-of-payment/
- TCMB ÖHVPS API standartları PDF (v1.0.0)
- İş Bankası: https://developer.isbank.com.tr/ · sandbox: https://developer.sandbox.isbank.com.tr/ · BaaS: /baas
- Open Banking Tracker (İş B.): https://www.openbankingtracker.com/provider/türkiye-iş-bankası
