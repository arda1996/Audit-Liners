# UFRS Kayıt Atma Sistemi — Teşhis Raporu (A3)

**Tarih:** 2026-07-17 · **Kapsam:** salt teşhis, kod değişikliği yok.
**İncelenen:** `crates/api/src/main.rs` (UFRS uçları 2800-3680), `web/src/App.tsx` (360-520, 1949-2465), `web/src/HesapSecici.tsx`, `data/ufrs-calismalar.json`, `data/hesap-kurallari.json`, `data/hesap-aciklamalari.json`.

---

## A. DARBOĞAZ TEŞHİSİ — sistem neden döngüde kalıyor?

**Desenin adı: "ÜÇ YARIM YOL" (üç paralel yardım hattı, hiçbiri uçtan uca tamam).**

Kayıt atmanın üç yardım mekanizması var ve üçü birbirinden bağımsız, birbiriyle çelişen, kapsamı yarım hatlar:

| Hat | Kapsam | Ne üretir | Eksiği |
|---|---|---|---|
| **① Hesapla motoru** (`main.rs:3337-3570`) | 18 çalışmanın **5'i** (RECLASS, REESKONT, ECL, KIDEM, NRV) | Tam satırlar + tutar + yöntem + baz taslağı | **Ertelenmiş vergi bacağı YOK** |
| **② Senaryo motoru** (`main.rs:3584-3668`) | 18 çalışmanın **3'ü** (GUD-MDV 2, KIDEM 2, ECL 1 = 5 senaryo) | Kayıt şekli + **EV bacağı dahil** | **Tutar üretmez** (elle girilir), hedef doğrulanmaz |
| **③ Kayıt çipleri** (`kayit_hesaplari`, App.tsx:2236-2262) | 18 çalışmanın **18'i** | Sadece hesap kodu satıra düşürür | Yön/tutar/karşı bacak önermez |

Döngünün mekaniği:

1. **Kapsama boşluğu:** 13/18 çalışmada (AMORT, K16, KUR, HASILAT, GUD-YAGM, BORCLANMA, DEGERDUS, KARSILIK, URETIM, YYIO, EV, ENFLASYON, NRV-dışılar) ne motor ne senaryo var — denetçi tamamen elle kayıt kurmak zorunda; sistemin vaadi ("hesap seçince otomatik dolsun") sadece 5 çalışmada, o da yarım tutuyor.
2. **Çelişki:** aynı olay için iki hat farklı kayıt üretiyor. Ör. WS-ECL: motor `654.90 / 129.90` önerir (EV'siz, `main.rs:3476`), senaryo `654.90 / @hedef + EV bacağı` kurar (`ufrs-calismalar.json` ecl_artis). Denetçi hangisinin "doğru ve tam" olduğunu her seferinde yeniden düşünüyor → mimari her denemede yeniden tartışmaya açılıyor.
3. **`@hedef` anlam kayması (en sinsi hata):** UI hedefi "bu kayıt HANGİ hesap İÇİN?" diye sunuyor (App.tsx:2082 "hesabı seç — kayıt onun için atılacak"), senaryo motoru ise `@hedef`'i "kaydın bu bacağı HANGİ hesaba İŞLENECEK" olarak kullanıyor (`main.rs:3617`). WS-ECL'de girdi tablosu 120/121/128/129 gösterir; denetçi doğal olarak 120'yi seçer → senaryo **doğrudan 120'yi alacaklandırır** (alacağın kendisini siler), oysa doğru bacak 129.90 karşılıktır — çipler de bunu söylüyor (`alacak: 129.90`). Aynı şey KIDEM'de: hedef 77x seçilirse karşılık 770'e alacak yazılır. **Katalogdaki çip verisi ile senaryo motoru aynı dosyada birbiriyle çelişiyor.**
4. **Güvenlik ağı yokluğu döngüyü kapatamıyor:** backend yanlış-yönlü, kapsam-dışı, mükerrer, aşırı-tutarlı kaydı kabul ettiği için (bkz. B) hata WTB'de geç fark ediliyor; "sistem güvenilmez" hissi → hattı yeniden kurma kararı → yine üç yarım yol → döngü.

**Kök neden sınıflandırması:** %50 **veri modeli** (`@hedef` tek anlamla iki işi yapamaz; senaryo bacakları "hedefin karşı bacağı"nı ayrı alan olarak taşımıyor), %30 **mimari** (üç yardım hattı tek boru hattında birleşmemiş; `hesap-kurallari.json`'daki karşı-bacak/yön bilgisi UFRS hattına hiç bağlanmamış), %20 **UX** (iki giriş noktası — çalışma formu ve WTB penceresi — aynı state'i paylaşıp farklı alan setleri gösteriyor; pencere hattı yapısal olarak hiç başarılı olamaz, bkz. D1).

---

## B. EKSİK DOĞRULAMA ENVANTERİ (backend)

`ufrs_kayit_ekle` (`main.rs:2911-3009`) **var olan** kontroller: oturum (2918), tür AJE/RJE (2921), ≥2 satır (2924), satır tek-taraf pozitif (2931), TDHP hesap varlığı + otomatik muavin (2938-2960), borç=alacak dengesi (2961-2965), dayanak_ref/denetçi notu/yöntem/baz boş-olamaz (2966-2977), kesin dönem kilidi (2984).

**OLMAYAN kontroller:**

| # | Eksik kontrol | Referans | Etki |
|---|---|---|---|
| B1 | **Hesap-yön (doğa) uygunluğu:** `st.kurallar` (hesap-kurallari.json, 83 hesabın B/A doğası + karşı bacakları) state'e yükleniyor (`main.rs:3762`) ama UFRS hattında **hiç okunmuyor**; sadece `hesap_aciklama` bilgi ucunda (`main.rs:733`). Kasa'yı alacaklandırıp eksiye düşüren, karşılığı borçlandırıp aktifleştiren kayıt sessizce geçer. | 2938-2960'ta yok | yüksek |
| B2 | **RJE kâr-nötrlük denetimi:** RJE "tutar-nötr sınıflama" tanımlı (2754) ama RJE'nin 6xx/7xx bacağı içermediği doğrulanmıyor; kâr etkileyen RJE atılabilir → WTB kâr köprüsü (3120-3125, sadece AJE sayar) sessizce yanlışlanır. | 2921-2923 sonrası yok | yüksek |
| B3 | **Çalışma-kapsam uyumu:** `kaynak_ws` varlığı bile doğrulanmıyor (`ufrs_ws_devir` bilinmeyen ws için sessizce TASI_BIRAK döner, 2806-2811); satır hesaplarının çalışmanın `hesaplar` önekleriyle veya `kayit_hesaplari` listesiyle ilişkisi hiç kontrol edilmiyor. WS-KIDEM'den 600'e kayıt atılabilir; var olmayan ws'e kayıt "hayalet" olur (çalışmalar sekmesindeki sayaç 2826 onu hiç göstermez, ama defterde ve WTB'de yaşar). | 2911-3009'da yok | orta-yüksek |
| B4 | **Mükerrer kayıt kilidi:** aynı ws + aynı satır seti + aynı dönem için tekrar kayıt engeli/uyarısı yok; "Formu doldur → Kaydı at" iki kez çalıştırılırsa çift düzeltme oluşur. TERS_CEVIR çalışmalarında (REESKONT/ECL/NRV) cari dönemde ikinci hesaplama + ikinci kayıt = çift karşılık. | 2978-3007'de yok | yüksek |
| B5 | **Maker-checker / kayıt-düzeyi onay:** `durum` yalnız `onerildi`/`vazgecildi` (2770); "onaylandı" durumu, ikinci imza, kademe eşiği yok. `onerildi` kayıt **doğrudan** WTB'ye akar (3080). Dönem kesinleştirme Y3 ister (3152) ama tek tek kayıtlar hiç gözden geçirilmeden kesinleşir — 3159'daki yorum bunu "girişte dengeliydi" diye geçiştiriyor. | 3146-3163 | yüksek |
| B6 | **Tutar sınırı / önemlilik eşiği:** hiçbir üst sınır, mizan-bakiyesi-aşımı veya önemlilik karşılaştırması yok. 252'nin bakiyesi 1 TL iken 1 milyar TL GUD artışı atılabilir. Senaryo ucunda tutar sadece `>0` (3596). | 2961 civarı, 3596 | orta |
| B7 | **Katalog-değer doğrulaması:** `dayanak_tur` (2995), `degerleme_yontemi` (2998) serbest metin kabul ediliyor — `dayanak_turleri`/`degerleme_yontemleri` kataloğuyla ve çalışmanın kendi yöntem listesiyle karşılaştırılmıyor. "asdf" geçerli dayanak türüdür. `standart` da istemciden gelen serbest metin (2990). | 2894-2908 | orta |
| B8 | **Senaryo ucunda state yok:** `ufrs_senaryo` imzasında `State` extractor'ı yok (3584-3586) → hedef hesabın varlığı, çalışma kapsamı, bakiyesi, yönü **hiçbir şekilde** doğrulanamıyor; `serbest` işareti "ilk karakter rakam mı" sezgisiyle atanıyor (3610). Uydurma "999" hedefi TDHP kodu sanılır. | 3584-3610 | yüksek |
| B9 | **Dönem-içi tarih/dönem tutarlılığı:** kayıt dönemi tek aktif yıldan türetiliyor (2801-2803, `st.donem.baslangic.yil`); kayıt tarihi `bugun_str()` — geriye dönük dönem seçme imkânı yok ama dönem değiştirilince eski dönemin açık kayıtları da o dönemle görünür kalır; kayıt üstünde dönem-kilit dışında dönem-atama doğrulaması yok. | 2983, 3003 | düşük-orta |
| B10 | **Devirde yetki ve onay:** `ufrs_devir` yalnız oturum ister (3188-3190) — kesinleştirme Y3 iken toplu CF üretimi herkese açık; ayrıca üretilen CF'ler `onerildi` doğar ve incelemesiz WTB'ye akar. TEKRARLA davranışı TASI_BIRAK ile aynı kola düşer (3218) — "her dönem yeniden atılmalı" kaydı sessizce taşı-bırak CF olur, yeniden üretim hatırlatması yalnız TERS_CEVIR'e var (3221-3222). | 3182-3256 | orta |
| B11 | **Kâr köprüsü asimetrisi:** `vuk_kar` yalnız sınıf 6'yı sayar (3119), `aje_kar_etkisi` 6 VE 7 ile başlayanları sayar (3123) ve **serbest TFRS kalemlerini** (adı harfle başlayan P/L kalemleri, ör. "Kullanım Hakkı Amortismanı") tamamen atlar → köprü tutmayabilir, kontrol satırı bunu yakalamaz. | 3119-3125 | orta |
| B12 | **Numara üretimi dönemsiz:** `sira` tüm dönemler + vazgeçilenler üzerinden önek sayımı (2982) — çakışma yok ama numara dönem bilgisi taşımıyor; vazgeçilen numara "boşluk" olarak dosyada iz bırakır (kabul edilebilir, belgelenmeli). | 2978-2988 | düşük |

---

## C. OTOMATİK DOLDURMA BOŞLUKLARI — hesap seçilince ne gelmeli de gelmiyor?

| # | Boşluk | Kaynak veri (hazır ama bağlanmamış) | Bugünkü davranış |
|---|---|---|---|
| C1 | **Karşı bacak önerisi:** hesap seçilince `hesap-kurallari.json` `karsi` listesi (83 hesap, ör. 129→[654,128], 257→[770,730,760]) form satırına önerilmiyor. Veri backend'de yüklü (`main.rs:3762`, `hesap_aciklama` ucu 726-751 zaten servis ediyor) — UFRS formu bu ucu **hiç çağırmıyor** (App.tsx UFRS bölümünde `aciklama` fetch'i yok). | hesap-kurallari.json `karsi`, `kapatma`, `doga` | Çipler statik listeden gelir; hedefe göre daralmaz. Serbest modda hiç öneri yok. |
| C2 | **Yön önerisi:** hesabın `doga` (B/A) ve `borc`/`alacak` açıklamaları formda hangi sütuna yazılacağını söyleyebilirdi; bugün denetçi çipi tıklayınca hesap satıra düşer ama **borç mu alacak mı boş kalır** (App.tsx:2249-2254 — çip BORÇ/ALACAK grubunda dursa da satıra sadece kod yazar, tutar sütunu seçilmez). | kayit_hesaplari zaten borç/alacak gruplu | Grup bilgisi görselde var, satıra taşınmıyor. |
| C3 | **Tutar önerisi:** girdi tablosunda hesabın net bakiyesi var (2879-2881) ama hedef seçilince senaryo tutar alanına bakiye/fark **ön-doldurulmuyor** (senaryoTutar boş başlar, App.tsx:387). Motoru olan 5 çalışmada motor tutarı biliyor; senaryolu 3 çalışmada tutar tamamen elle. Motor + senaryo hiç konuşmuyor: motorun FARK'ı senaryonun tutarına akmıyor. | `ufrs_hesapla` FARK satırı; girdi_hesaplar.net | Denetçi tutarı elle kopyalıyor — hata kaynağı. |
| C4 | **Ertelenmiş vergi:** senaryo hattında otomatik (3628-3655), **motor hattında yok** (3385-3568'in hiçbir kolu EV bacağı üretmez, WS-EV ayrı çalışma). Aynı ekonomik olayda iki farklı tamlık düzeyi. | ufrs-calismalar.json `ev_hesaplari` + `ev_kanal` | Motor önerisini kullanan denetçi EV'yi unutmaya teşvik ediliyor. |
| C5 | **Dayanak türü / yöntem ön-seçimi kısmi:** `dayanak_onerisi` ve ilk `degerleme_yontemi` çalışma açılınca set ediliyor (App.tsx:423, 427) — iyi; ama **dayanak_ref** ve **degerleme_bazi** için şablon üretimi yalnız motor/senaryo çıktısında var; 13 elle-çalışmada boş placeholder. `hesap-aciklamalari.json` (tanım+işleyiş metinleri) not taslağına hiç akmıyor. | degerleme_bazi_taslagi, not_taslagi | Elle hatta her metin sıfırdan. |
| C6 | **Varlık kimliği → kayıt bağlantısı:** girdi tablosu varlık kimliği/VUK-TFRS ömrünü gösteriyor (2882-2886) ama hedef seçilince bu bilgi açıklama/baz alanına taşınmıyor; "en ince ayrıntıyla açıklama" ilkesi elle kopyaya kalıyor. | varlik envanteri | Görsel süs olarak kalıyor. |

---

## D. STATE TUTARLILIK RİSKLERİ (frontend)

| # | Risk | Referans | Senaryo |
|---|---|---|---|
| D1 | **"+ Kayıt (pencere)" hattı yapısal çıkmaz:** WTB'deki modal form (App.tsx:2418-2459) **degerleme_yontemi/degerleme_bazi alanlarını hiç göstermiyor**; `acUfrsWs` ukBaz'ı boşaltır (428). Backend baz'ı zorunlu kılar (2975). Sonuç: pencereden atılan her kayıt "değerlemenin neyi baz aldığı zorunlu" hatasıyla düşer ve kullanıcının dolduracağı alan ekranda YOK. Çalışma seçilmemişse `ufrsKayitGonder` sessizce return eder (483-484) — buton hiçbir şey yapmaz, mesaj yok. **Kullanıcının "hep aynı darboğaz" algısının en somut adayı.** | App.tsx:2418-2459, 483, 428 | Her pencere denemesi başarısız. |
| D2 | **Çalışma değişiminde yarım reset:** `acUfrsWs` (418-430) hedef/senaryo/hesap-önerisini sıfırlar ama **ukSatirlar, ukAciklama, ukDayanakRef, ukNot sıfırlanmaz** → önceki çalışmanın satırları yeni `kaynak_ws` altında gönderilebilir (B3 ile birleşince yanlış çalışmaya temiz görünümlü kayıt). | App.tsx:418-430, 489 | KIDEM satırları ECL çalışması adına kaydedilir. |
| D3 | **hedefSec satır ezme:** hedef seçimi "ilk boş satıra, yoksa 0. satıra" yazar (392-396) — dolu formda hedef değiştirmek 0. satırın hesabını sessizce ezer; art arda iki hedef denemesi iki ayrı satıra iki hedef bırakır (ilki bayat kalır). | App.tsx:389-397 | Bayat hedef bacağı dengeli ama yanlış kayıtla sonuçlanır. |
| D4 | **serbest bayrağı iki ayrı sezgiyle atanıyor:** backend senaryo `!ilk_karakter_rakam` (main.rs:3610), frontend gönderimde `/^\d{3}(\.[\w-]+)?$/` (App.tsx:432). Üç seviyeli gerçek muavin "252.90.1" frontend regex'ine uymaz → **serbest=true** gönderilir → doğrulama atlanır, WTB'de kebir 252 yerine "T" sınıfı ayrı satır olur; aynı tutar iki farklı satırda izlenebilir. | App.tsx:432, 485; main.rs:3082-3087 | WTB kırılımı sessizce bölünür. |
| D5 | **oneriFormaAktar onaysız tam ezme + çift gönderim:** öneri formu tek tıkla tüm alanları ezer (460-471), mevcut yarım giriş uyarısız kaybolur; başarılı kayıttan sonra `ufrsHesap` (öneri paneli) **temizlenmez** (492-496 sadece form alanlarını temizler) → "Formu doldur →" yeniden tıklanabilir → B4 ile mükerrer kayıt. | App.tsx:460-471, 492-496 | Çift AJE. |
| D6 | **Hata gösterimi tutarsız/sessiz:** kayıt hatası `ukMesaj`'da metin olarak gösterilir (497 — iyi) ama `ufrsVazgec` yanıtı hiç kontrol etmez (500-503, kesin dönem/CF reddi kullanıcıya görünmez — liste değişmeyince "buton bozuk" algısı); `wtbKirilim` catch'i sessiz (479); devir onay sorusu yok (512-519, kesinleştirmede var 506). | App.tsx:500-503, 512-519 | Sessiz başarısızlık = güven kaybı. |
| D7 | **İki "②" bloğu:** hem senaryosu hem motoru olan çalışmada (KIDEM, ECL) iki ayrı kart aynı "②" numarasıyla yan yana (2108, 2135) ve ikisi de `ufrsHesapMesaj`/`ufrsHesap`'ı paylaşır — motor sonucu ekrandayken senaryo çalıştırmak sonucu değiştirir, kullanıcı hangi hattın çıktısına baktığını kaybeder. | App.tsx:2105-2156, 408-409 | A'daki çelişkinin UI yüzü. |
| D8 | **senaryoTutar/Oran kalıcılığı:** çalışma değişince `senaryoTutar` sıfırlanır (419) ama senaryo kodu değişince tutar kalır; `senaryoOran` hiç sıfırlanmaz — önceki mükellefin/dönemin KV oranı sonrakine taşınabilir. | App.tsx:386-388, 419 | Yanlış oranla EV bacağı. |

---

## Özet yargı

Sistem "kayıt atılamıyor" diye değil, **"kayıt üç yarım yoldan atılıyor"** diye döngüde: yardım hatları (çip/senaryo/motor) tek boru hattında birleşmemiş, `@hedef` veri modeli iki anlamı tek alanda taşıyor, `hesap-kurallari.json`'daki karşı-bacak/yön bilgisi UFRS hattına hiç bağlanmamış (atıl), ve backend'de yön/kapsam/mükerrer/onay/önemlilik kontrolleri olmadığı için hatalar geç patlıyor. Ayrıca WTB'deki "+ Kayıt (pencere)" hattı zorunlu alanı hiç göstermediği için **yapısal olarak asla başarılı olamıyor** — "hep aynı darboğaz" hissinin en somut kaynağı budur.
