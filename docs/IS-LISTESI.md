# İŞ LİSTESİ — Audit-Liners (kalıcı takip dosyası)

> **Kural:** Her iş bittiğinde bu dosya güncellenir. Parametreler:
> **Durum:** ✅ tamam · 🔨 devam · ⏸ sonraya · 🔒 bloklu (önce bağımlılığı bitmeli) · ⬜ bekliyor
> **Bağımlılık:** bu işten önce bitmesi gerekenler. **Not:** neyi tamamladık / neyi bilinçli erteledik.
> Son güncelleme: 2026-07-04
>
> **Durum özeti (2026-07-04 ajan denetimi):** 39 test yeşil (domain 11 + api 5 + fixture/senaryo 14) · tsc temiz · ~25 API ucu çalışıyor · fixture 20k kayıt Q4 denk (Aktif=Pasif 9.870.293,38).
> Ajan doğrulaması: Taslak CRUD **yok** (`Fis::taslak` yalnız iç kurucu, anında kesinleştiriliyor), fiş arama **yok** (yalnız `?limit`), karşı bacak motoru **yok** (karsi[] verisi görüntüleme amaçlı), kullanıcı/zaman damgası **yok**, dönem yönetimi UI **yok** (fiş tipindeki Açılış/Kapanış seçeneği sihirbaz değildir), cari kart UI **yok**, kalıcılık **yok** (tümü bellekte, 2026 hardcode).

> **🎯 ODAKLANMIŞ GÖREV LİSTESİ (2026-07-09, kullanıcı: "dağıldık, listeye koy, sırasıyla yap"):**
> Bu tur ARAŞTIRMA yapıldı (kod yok). Sıra: Y1 muhasebe kayıt düzeltmeleri (analiz/07) → Y2 yetkilendirme
> veri modeli (tasarim/yetkilendirme-departman-hiyerarsi) → Y3 yetki backend+UI → Y4 frontend global bar
> (tasarim/frontend-yeniden-tasarim) → Y5 tasarım sistemi. TaskList #20-24. Frontend "daha sonra" (kullanıcı);
> yetki "önce araştır" (kullanıcı) — ikisi de araştırıldı, kodlama GO bekliyor.

## A. Muhasebe çekirdeği (domain — Rust, 11 test suite yeşil)
| İş | Durum | Bağımlılık | Not |
|----|-------|-----------|-----|
| Çift taraflı kayıt kuralları (V1-V7, denge, yaprak, dönem) | ✅ | — | 11 kural testi |
| Ayrı seri fiş no + müteselsil yevmiye madde no (VUK) | ✅ | — | TAH-1/MAH-1 ayrı; yevmiye 1,2,3… kesintisiz |
| İptal (düzeltme kaydı) — VUK md.217 | ✅ | — | Fark edilme tarihi + gerekçe zorunlu + dayanak=kaynak fiş; "storno" ibaresi kaldırıldı |
| TDHP 261 hesap koddan türetme + muavin (sınırsız kırılım) | ✅ | — | (-) kontra çevirme + 7x1 yansıtma istisnası; rollup kod önekiyle |
| Defter motoru: mizan/kebir/yevmiye/muavin + karşı hesap + B/A | ✅ | — | Kebir↔yevmiye çapraz referanslı |
| Cari (120.xx/320.xx kart + bakiye/ekstre) | ✅ | — | UI kartı henüz yok → E.4 |
| Dönem kapanış/açılış virman (6→690→691/370→692→590/591) | ✅ | — | Vergi karşılığı dahil (denetim m.6 fix); UI'sı yok → C.2 |
| KDV aylık mahsubu (alt hesapları tarar; 360/190) | ✅ | — | "Motorlar önek altını tarar" kuralı |
| Mali tablolar (bilanço aktif=pasif; GT 691+net kâr satırlı) | ✅ | — | Denetim m.7 fix |
| Fatura → otomatik fiş (istisna kodlu, KDV oran alt hesabına) | ✅ | — | E-fatura yaşam döngüsüyle PARK — bkz. F.1 |
| **Dönem sonu değerleme** (amortisman/reeskont/şüpheli alacak/kur/karşılık) | ⬜ | — | Görev #4; kâr doğruluğu için kritik (denetim m.3) |
| **Çoklu mükellef (izolasyon)** | 🔨 | — | 2026-07-09 ODAK: **"aktif çalışma seti + arşiv swap" deseni** — 45 handler'ın hiçbirine dokunmadan çoklu mükellef. AppState mutable alanları = aktif mükellefin defteri; diğerleri arsiv'de. GET/POST /api/mukellefler·mukellef·mukellef/:id/aktif·sektorler. Test: m1(19657 fiş)↔m2(URT,0 fiş) izolasyon+kayıp-yok ✓. Frontend header mükellef seçici (unvan/VKN/sektör + yeni mükellef, 10 sektör). KALAN: (a) sektörün hesap davranışına bağlanması — sektöre göre stok/maliyet/şablon; (b) çoklu DÖNEM (hâlâ tek 2026); (c) kalıcılıkta mükellef=tablo/şema |
| **Y2 Yetkilendirme veri modeli (departman + kademe + SoD)** | ✅ | — | 2026-07-09: data/departmanlar.json (6 departman → görünür modüller) + data/kademeler.json (4 kademe: ELEMAN/SORUMLU/MUDUR/YONETICI → izinli işlemler + onay eşiği; SORUMLU 500k TL). Kullanici'ye departman+kademe alanı; /api/departmanlar·kademeler uçları; kullanıcı formu + tablosu departman/kademe gösterir. Test: veli BORDRO/SORUMLU ✓. **Y3 ✅ (2026-07-09):** kademe zorlaması — /api/fis kesinleştir "kesinlestir" yetkisi + onay eşiği (SORUMLU 500k TL), /api/fis/:id/iptal "iptal" yetkisi (müdür/yönetici). Frontend: NAV departmana göre süzülür (vergici→dashboard/vergi/analiz; eleman→muhasebe modülleri), Kesinleştir/İptal düğmeleri kademeye göre gizli ("Bu kademe kesinleştiremez"). Test: ELEMAN 403, SORUMLU 400k✓/600k✗, MÜDÜR sınırsız+iptal✓. **Backend Kullanım Kılavuzu (eğitim):** docs/KULLANIM-KLAVUZU-BACKEND.md — tüm uçlar + yetki matrisi. **KALAN (SoD tam):** maker≠checker taslak CRUD (B.7) + kullanıcı/zaman damgası gelince |
| **Kullanıcı girişi + yetki (RBAC)** | 🔨 | — | 2026-07-09: giriş ekranı (logo + kayıt YOK), oturum token, admin/kullanıcı rolleri, kullanıcı→mükellef ataması. Uçlar: /api/giris·cikis·oturum·kullanicilar·kullanici. Kullanıcı yalnız atandığı mükellefleri görür+geçebilir (Ayşe m2, m1→403; /kullanicilar→403 test ✓). Admin "Yönetim" paneli: kullanıcı listesi + oluştur (rol + mükellef checkbox). Seed admin: **admin / audit2026** (değiştirilmeli). Logo: web/public/logo.svg (çizgisel kırmızı-beyaz: defter çizgileri + denetim tik'i). ⚠ Parola özeti std DefaultHasher — kalıcılıkta argon2/bcrypt olacak. KALAN: parola değiştirme/reset, kullanıcı düzenle/sil, ledger uçlarına da token (şu an mükellef-swap seviyesinde korumalı) |
| **Sektör kataloğu (Türkiye reel sektörleri)** | ✅ | — | 2026-07-09: data/sektorler.json — NACE Rev.2 tabanlı 10 sektör (TIC/ETIC/URT/INS/HIZ/LOJ/GID/TAR/ENR/SRB), her biri: uretim bayrağı, 7A/7B/yok maliyet seçeneği, stok/maliyet/hasılat hesapları, özel hesaplar, nitelik açıklaması, denetim programı bağı. KİLİT: aynı mal üreticide 150 ilk madde ↔ tüccarda 153 ticari mal; üretim maliyet hesapları (71-78) yalnız üretim/hizmette. Kaynak: NACE + 7/A-7/B hadleri araştırması |
| **Stok miktar takibi + envanter** (adet × birim maliyet, FIFO/ortalama) | ⬜ | — | YENİ TESPİT (2026-07-04): sistem tutar-bazlı; VUK envanter "saymak-ölçmek-tartmak-değerlemek" ister — miktarsız envanter listesi ve doğru SMM üretilemez |
| **TMS overlay (dual-defter)** — VUK çekirdek + TMS düzeltme katmanı | ⬜ | A.11 | YENİ TESPİT: strateji kararlaştırıldı (33-doc analizi, pazar farklılaştırıcı) ama hiç kod yok; TMS19 kıdem, TMS12 ertelenmiş vergi, TMS2 NRV buradan |
| Nazım hesaplar (9) — teminat/kefalet/emanet takibi | ⬜ | — | YENİ TESPİT: TDHP sınıf 9 hiç işlenmedi; bilanço dışı ama denetimde sorulur |
| 7/B seçeneği (gider çeşidi esası — küçük işletme) | ⏸ | — | YENİ TESPİT: yalnız 7/A akışı test edildi; hedef kitle SMMM'nin küçük mükellefleri 7/B kullanır |
| Vergi motoru (vergiyi doğuran olay; KKEG/mali kâr köprüsü) | ⏸ | A.11 | Kullanıcı kararı: muhasebe bitince |
| Enflasyon düzeltmesi altyapısı (parasal bayrak + iktisap tarihi) | ⏸ | A.12 | VUK Geç.37: 2025-27 uygulanmıyor; alan hazırlığı yeter |

## B. Kayıt deneyimi (UI — Muhasebe tek sayfası)
| İş | Durum | Bağımlılık | Not |
|----|-------|-----------|-----|
| Muhasebe tek sayfa + pill alt-sekmeler (bütünlük) | ✅ | — | Kayıt·Fişler·Yevmiye·Muavin·Mizan; **Kebir sekmesi kaldırıldı** → mizandan "Kebir →" ile girilir, "← Mizan" ile dönülür |
| Kebir kolonu + aranabilir hesap seçici (tam liste) + satırdan alt hesap | ✅ | — | |
| Açıklama = hesap koduna bağımlı küme + kullanıcı ekleme | ✅ | — | Kural verisinden otomatik türetme; nadir hesaplar hesap adına düşer |
| Fiş şablonları — **sektöre göre** + ± fark doldur + ters bakiye uyarısı | ✅ | — | 2026-07-09: data/sablonlar.json (24 şablon, sektör etiketli) + GET /api/sablonlar; frontend aktif mükellefin sektörüne göre süzer. **Hesap DÜZELTMELERİ:** üreticide mal alışı 150 (tüccarda 153); ücret tahakkuku SGK işveren payı satırı eklendi (5 satır); SMM (621/153) + satış iadesi (610) şablonları eklendi; üretim zinciri (150→710→711→151→620), hizmet (740→622), inşaat (170/350 hakediş+stopaj). Preview: ETIC→153, URT→150 doğrulandı |
| Kronoloji koruması (geriye tarihli kayıt reddi) | ✅ | — | API'de; iptal fişleri VUK 217 tarihiyle |
| Mükerrer belge (tip+no) reddi | ✅ | — | |
| **Taslak CRUD** (kaydet/düzenle/sil → sonra kesinleştir) | ⬜ | — | En öncelikli eksik; şu an tek yol doğrudan kesinleştirme |
| **Karşı bacak uyarısı** (karsi[] verisinden olağandışı eşleşme) | ⬜ | — | Veri hazır (83 hesap), motor yok |
| **Fiş arama/filtre** (tarih/hesap/tutar/no) | ⬜ | — | 19k fixture'da acil; API `?limit` var, arama yok |
| Kullanıcı + zaman damgası (denetim izi alanları) | ⬜ | — | Domain Fiş'e olusturan/olusturma eklenecek |
| **Toplu fiş içe aktarma** (Excel/CSV — Luca/Zirve'den geçiş) | ⬜ | B.7 | YENİ TESPİT: SMMM'nin programa geçiş yolu; taslak olarak alınıp kontrol→kesinleştirme akışına girmeli |
| Fiş yazdırma / PDF çıktısı | ⬜ | — | YENİ TESPİT: kâğıt imza-dosyalama pratiği sürüyor |
| **Kayıt akışı kolaylık turu** (5-ajan eleştirisi) | ✅ | — | 2026-07-08: TR virgül parse (1.234,56→123456 kuruş, sessiz veri hatası kapandı), decimal input, kesinleştir sonrası reset+belge no otomatik artış+odak+çift-tık koruması, form boş açılış, gg.aa.yyyy tarih, HesapSecici klavye nav (↑↓/Enter), boş defter "Örnek yükle" bandı, Sidebar ölü öğeler "Yakında"+disabled, kontrast. Detay: completions/2026-07-08 |
| **UI kalan (5-ajan)**: prompt→modal · jargon tooltip · .card-hd birleştirme+tipografi skala · aria-live/label · tx-1 açık dil · Rapor&Vergi grup bölme | ⬜ | — | Bilinçle ertelendi; günlük kullanım kritikleri önce yapıldı |
| **Fiş satırı sürükle-sırala + canlı hesap** | ✅ | — | 2026-07-09: kayıt formunda ⠿ tutamakla satır sürüklenir, sıra değişir, toplam borç/alacak+denge CANLI güncellenir (satirlar→useMemo). Preview'da takas doğrulandı. **STANDART SINIRI:** yalnız GİRİLMEKTE olan fişin satırları; KESİN fişler sürüklenip yeniden sıralanamaz (VUK: yevmiye maddeleri müteselsil+kronolojik; kesin fiş değişmez) — taslak fiş sıralaması B.7 taslak CRUD ile gelecek |
| Form kontrol/buton tasarım dili (kullanıcı geri bildirimi: "biçimsiz") | ✅ | — | 2026-07-05: tüm select'ler özel chevron'lu (native görünüm kapatıldı), 36px tek ritim, hover+focus halkası; number input spinner'ları gizli + sağa hizalı tabular rakam; ± / ✕ satır butonları `.ikon-btn` (✕ hover kırmızı); buton etkileşimleri (hover kalkış, active bastırma, focus-visible halka); dashboard takvim kartı vergi sayfasıyla senkronlandı (KDV 26→28) ve tıklanabilir → Vergi/Takvim. Claude Design bağlantısı bu ortamda yetkilendirilemedi — yerel fener (Genel Bakış.dc.html) token'larıyla ilerlendi |
| KDV yardımcısı (matrah girilince oran satırı otomatik) | ⏸ | — | Şablonlarla kısmen karşılanıyor |

## C. Defterler & dönem (görünür çıktılar)
| İş | Durum | Bağımlılık | Not |
|----|-------|-----------|-----|
| Yevmiye (belge+madde toplamı+nakli yekûn) / Kebir (yevmiye no+karşı hesap+B/A) / Muavin (özet→tembel→sayfalı, TXT export) / Mizan (kebir-muavin seviye + kebir filtresi + Kebir→ iniş) | ✅ | — | Büyük veri "lite" kalıbı: özet-önce + sayfalama (108.01: 19k hareket, 45ms/sayfa) |
| **Dönem yönetimi UI** (kapanış/açılış sihirbazı, dönem kilidi) | ⬜ | — | Motor hazır (A.7); fixture ile uçtan uca yıl kapanışı testi yapılacak |
| Diğer defterlere TXT/Excel export | ⏸ | — | Muavin TXT kalıbı kopyalanacak; Excel → rust_xlsxwriter |
| **Mali tablo setinin tamamlanması**: nakit akış + öz kaynak değişim tablosu | ⬜ | — | YENİ TESPİT: TMS 1/34 tam set sayar (bilanço, GT, öz kaynak değişim, nakit akış, dipnot) — bizde yalnız 2/5 var; MSUGT ek tabloları: satışların maliyeti tablosu, kâr dağıtım tablosu |
| **e-Defter (GİB XML)** — yasal yevmiye/kebir çıktısı | 🔒 | F.2, A.12 | YENİ TESPİT: "vergiye giden yol"un yasal ucu; berat/imza gerekir, kalıcılık önkoşul |
| Bilanço/GT ekranları | ✅ | — | Dipnotlar → ileri faz (TMS) |
| **Mali tablolar tek sayfa — sunum esaslarına göre** (analiz içinde "Mali tablolar" sekmesi) | ✅ | — | PDF yöntemi uygulandı: **hesap tipi bilanço** (aktif–pasif KARŞILIKLI, T şekli — Genel Muh. kitabı Şekil 2.3) + grup ara başlıkları (10 HAZIR DEĞERLER…); **karşılaştırmalı kolonlar tx-1/tx** (MSUGT); GT **dönem başından kümülatif** + Δ% (TMS 34). Aktif=Pasif denklik fixture'da doğrulandı (9.870.293,38). Eski ayrı Bilanço/GT sekmeleri kaldırıldı. **TMS 34 tam kuralı BEKLİYOR (A.12 çoklu dönem):** ara dönem bilançosu *önceki YIL SONU* ile, GT *önceki yılın AYNI ara dönemi* ile karşılaştırılır — tek 2026 verisiyle şimdilik tx-1 çeyreği kullanılıyor |
| **Finansal analiz sayfası v2** (sekmeli: Oranlar · Mali tablolar · Aylık&Kur) | ✅ | — | 14 doktrin oranı + **banka/kredibilite görünümü** her kartta (kredi tahsis pratiği); **kebir bazlı bilanço** (aktif/pasif, taraf toplamları); **dönem + kıyas dönemi filtresi** (ay bazlı) → oranlar Δ rozetli, bilanço/GT kıyas kolonlu (Δ%); **karşılaştırma yorum motoru** (yükseliş/düşüş/yatay + işletme sürekliliği + sektör bağlamı + **kur bağlamı** USD-EUR/TL, USD/EUR); yatay-seyir eşiği (%1) float gürültüsünü susturur. Bilanço/GT nav'dan analiz içine taşındı. SONRAYA: **gerçek kur → TCMB EVDS** (şimdilik deterministik örnek seri), sektörel eşik şablonları, yatay analiz yıllar arası (çoklu dönem A.12 bekliyor), trend grafiği, banka skorlama ağırlıklı toplam puan |

## D. Eşleştirme & belge akışı
| İş | Durum | Bağımlılık | Not |
|----|-------|-----------|-----|
| Banka ekstresi alanı (CSV yapıştır → yükle → öneri → eşle) | ✅ | — | Kural: 102* aynı tutar ±3 gün |
| Gelen belge kutusu (e-Fatura/e-Arşiv) + kayda bağlama | ✅ | — | Kural: fiş toplamı ±5 gün; e-dönüşüm bu kutuyu dolduracak |
| Eşleşmeyen ekstre satırından otomatik TASLAK fiş önerisi | ⬜ | B.7 | Taslak sistemi olmadan yapılamaz |
| **Eşleştirme ↔ fatura modülü birleşimi** | ⏸ | F.1 | Kullanıcı notu: önceki fatura çalışmasındaki yapılar (fatura_fisi, durum makinesi, arşiv) buradan alınacak |
| **D.5 Banka PDF içe aktarma — profil motoru** | 🔨 | — | 2026-07-06: **data/banka-profilleri.json HAZIR** (20 banka imzası; ziraat+vakifbank format detaylı, kalanı dogrulandi:false — örnek PDF'le teyit edilecek; 6 genel kural: bbox, multi-line, İ/I, DESC çevirme, ters-işlem tek sorumlu, 3 katman dedup). KALAN: PDF metin çıkarıcı (pdf-extract) + profil yorumlayıcı motor |
| **D.6 Eşleştirme motoru v2 — SuperMatch ilkeleri** | 🔨 | — | 2026-07-06: **domain/mutabakat.rs HAZIR + 5 test** (uyuyan modül — API/UI'a bilinçli bağlanmadı, iş akışında gerekince). Katmanlı (kanonik isim grubu → KESİN tutar), deterministik (BTreeMap+sıralı), kategorize artıklar (GrupIciEksik/IsimYok), pencere_gun parametreli. MANTIK DÜZELTMESİ: id tie-break yerine tarih-yakınlığı |
| **D.7 Ekstre bütünlüğü** | 🔨 | D.5 | 2026-07-06: **domain/ekstre.rs + domain/isim.rs HAZIR + 11 test** (uyuyan). zincir_dogrula (M13 çekirdeği — kopukluk bulgular, tek kopukluk sahte ardıl üretmez), devir_dogrula, hareket_anahtari (dedup, sıra ayraçlı), ters_isle (2 katman: ipucu + orijinal işlem onayı; İDEMPOTENT — kaynak projedeki çift çağrı bug'ının ilacı), kanonik_isim+isim_skoru (sözlüksüz ~100 satır). MANTIK DÜZELTMESİ: float para → kuruş i64. KALAN: 102.xx defter mutabakatına bağlama, cari önerisi UI |

## E. Diğer modüller
| İş | Durum | Bağımlılık | Not |
|----|-------|-----------|-----|
| KDV ekranı | ✅ | — | Kullanıcı: alt hesaplara taşındıkça sadeleşecek |
| 20k e-ticaret fixture (`POST /api/ornek-veri`) | ✅ | — | Deterministik, kronolojik; yansıtma dersi: 7'ler kapatılmadan bilanço denk çıkmaz. **2026-07-09 Y1 düzeltmeleri:** (F1) kira ödemesine GVK 94 %20 stopajı (360'a); (F2) tam bordro — SGK işveren payı (770.02 B + 361 A); gelir/gider ayrımı motorda teyit (60 gelir/62 SMM/63 faaliyet gid); yansıtma OTOMATIK düzeldi (770→771→632=3.072.000 işveren payı dahil, 361=648.000). Bilanço denk (FARK 0), 53 test yeşil. sablonlar.json'a kira-stopajlı + gider-yansıtma şablonları eklendi |
| Hesap işleyiş verisi 262'ye tamamlama + UI işleyiş paneli | 🔨 | — | 83 kurallı + 205 MSUGT açıklaması var; kalan ~65 nadir |
| Cari kartlar + ekstre ekranı | ⬜ | — | Domain hazır (A.6) |
| Firma/Mükellef profili + yıl-etkin parametreler | ⬜ | A.12 | Analiz F1 önkoşulu: oranlar hardcode edilmemeli |
| Bordro/SGK, Muhtasar, beyannameler | ⏸ | E.5, A.13 | Analiz F2 |
| Muhasebe Öğren (oyunlaştırılmış) | 🔒 | tüm çekirdek | Görev #18; içerik hazır (MSUGT açıklamaları) |
| **Spotlight öğretici rehber (ilk kullanım turu)** | ✅ | — | 2026-07-06: web/src/Rehber.tsx — sayfa bazlı adımlar VERİDEN (REHBER kaydı; adım eklemek = veri düzenlemek); hedef öğe karartma+turuncu halkayla vurgulanır (tıklamayı ENGELLEMEZ — kullanıcı anlatılan düğmeye basabilir), hedefsiz adım ortada kart; İleri/Geri/✕; sayfa turu bitince "Sıradaki sayfa →" (dashboard→muhasebe→…→hesaplar yönlendirme akışı); ilk açılışta otomatik (localStorage 'rehber-gezildi'), başlıktaki ❔ Rehber düğmesiyle her an aç/kapat. 8 sayfa, 21 adım; 12 data-rehber çapası. **2026-07-06 ek — kısayollar:** Sonlandır düğmesi (kartta, kırmızı) + Space/→/Enter=ileri, ←=geri, Esc=sonlandır; kart altında kısayol ipucu satırı; form alanı odaklıyken kısayollar devre dışı (tur etkileşimi engellemediğinden yazarken Space yutulmaz — preview'da doğrulandı: input odaklı Space adım değiştirmiyor, body odaklı değiştiriyor). **2026-07-06 bug fix (kullanıcı raporu: "rehber sonlanınca beyaz ekran"):** çok adımlı sayfadan az adımlıya "Sıradaki sayfa" geçişinde, adım-sıfırlama effect'i koşmadan önceki render'da eski indeks yeni listeyi aşıp `adimlar[4]=undefined` → React çöküyordu; indeks artık her render'da listeye kıstırılıyor (adimIdx) + adım yoksa güvenli null. Repro: Muhasebe 5/5 → Banka(1 adım) beyaz ekran → fix sonrası Banka 1/1 + Sonlandır sonrası ekran dolu ✓. KALAN: alt sekme hedefleri (ör. geçici vergi kartı yalnız o sekmede) için sekme-otomatik-geçiş |

## F. Altyapı
| İş | Durum | Bağımlılık | Not |
|----|-------|-----------|-----|
| E-fatura entegrasyonu (call→kesinleşme→otomatik fiş→arşiv) | ⏸ | muhasebe A-Z | PARK (kullanıcı kararı); kod hazır ve test edilmiş: domain/fatura.rs + api uçları |
| **Kalıcılık** (gömülü Postgres + db crate, hexagonal) | 🔒 | muhasebe A-Z | Kullanıcı kararı: muhasebe bitmeden GEÇİLMEZ; kullanıcı kümeler/eşleştirmeler şu an bellekte |
| Tauri paketleme + CI (Win .msi / Mac .dmg) | ⏸ | F.2 | src-tauri derleniyor; sidecar + installer kaldı |
| Denetim modülü (BDS, örnekleme, hash zinciri) | 🔨 | F.2 | Ayrıştırıcı özellik (mavi okyanus). Mimari kuruldu → **H bölümü** (denetime giden yol); önemlilik hesabı + BDS 530 örnekleme + hash zinciri H'nin faz 2'si |

## J. Dış veri katmanı — "program devasa bir API" (2026-07-08 kuruldu)
> Doktrin: veri toplayabilir + dışa sunabilir program; ham yanıt = denetim kanıtı; her dış değer
> {deger, kaynak, alinma_tarihi} üçlüsüyle taşınır; manuel override her zaman var. Mimari + kaynak
> eşlemesi: docs/tasarim/dis-veri-katmani.md · kaynak kaydı: data/veri-kaynaklari.json

| İş | Durum | Bağımlılık | Not |
|----|-------|-----------|-----|
| J.1 Veri kaynak iskeleti (9 kaynak: TCMB×4, EVDS, Damodaran, Kroll, GİB, TLREF) | ✅ | — | TCMB günlük+arşiv XML CANLI doğrulandı (anahtarsız — reverse gerekmedi: USD 46.6337 bülten 2026/122; arşiv 02.01.2026 → 42.8810). Eksen alanı iki katmanlı raporlama doktrinine bağlı: vergi=TCMB/GİB resmi, bağımsız denetim=Damodaran/Kroll doktrin |
| J.2 İngest adaptör sözleşmesi (crates/ingest — VeriKaynagi trait + üçlü taşıma tipi) | ⬜ | — | Banka PDF'i ile aynı arayüz (D.5 ile ortak) |
| **J.2b Site keşif MCP sunucusu** (endpoint çıkarma + çağrı taklit) | ✅ | — | 2026-07-08: tools/site-kesif-mcp (Node+TS+playwright-core, CDP ile KENDİ Chrome'una bağlanır — girişli kaynaklar için gerçek oturum). 8 araç: chrome_baslat/baglan · hedef_ac · yakalananlar (json/xml önce, teklenmiş) · cagri_detay (+şema özeti) · cagri_tekrarla (call/callback taklit, çerez korunur) · profil_uret (→veri-kaynaklari.json taslağı) · temizle. .mcp.json'a kayıtlı; stdio dumanlı test 8 araç ✓. Yetki ilkesi README'de (yetkili/kamuya açık kaynak, auth-aşma/kazıma değil). Keşif ucu → J.2 ingest adaptörünün girdisi |
| J.3 TCMB XML adaptörü — ilk canlı kaynak | ⬜ | J.2 | Analizdeki örnek kur serisini gerçeğe çevirir; D′ kur değerlemesinin girdisi (VUK 280 işlem kuru; dönem sonu kuru GİB TEBLİĞİNDEN — TCMB yedek) |
| J.4 EVDS adaptörü (TÜFE→TMS29, DİBS→TMS19, kredi faizi→TFRS16) | 🔒 | J.2 + api-key | Kullanıcı EVDS'ten ücretsiz anahtar alacak; TEK KAPI: TÜİK/Hazine ayrı adaptör istemez |
| J.5 Dönemsel kaynaklar: Damodaran/Kroll içe aktarma + manuel override UI; TCMB avans + GİB tebliğ değişiklik uyarısı | ⬜ | J.2 | Kroll aboneliksiz tam çekilemez → manuel katman asıl yol; 6 aylık ritim |
| J.6 Kanıt arşivi (ham yanıt saklama + kağıtta kaynak gösterimi) | 🔒 | F.2 kalıcılık | BDS 500; kalıcılık gelmeden bellekte sınırlı |
| J.7 Dışa API (/api/veri/*) — program veri SAĞLAYICISI olur | ⏸ | J.3, F.2 | Devasa API vizyonunun dışa dönük yüzü |

## H. Denetime giden yol — sektörel denetim çalışmaları (2026-07-04 kuruldu)
> Mimari: **motor (kod) / program (veri) / bulgu (çıktı)** — bkz. docs/tasarim/denetime-giden-yol.md.
> Proaktif değişiklik: programlar data/denetim-programlari.json'da sürümlü; kullanıcı katmanı kopyala-düzenle/kapat/ekle (açıklama-kümeleri kalıbı).

| İş | Durum | Bağımlılık | Not |
|----|-------|-----------|-----|
| H.1 Veri modeli + sektör programları JSON | ✅ | — | 7 sektör, 32 çalışma yazıldı (TIC 9 · ETIC 4+devralır · HIZ 4 · URT 5 · INS 4 · LOJ 4 · GID 2); TAR (TMS 41) faz 2 |
| H.2 Motor kütüphanesi M1–M12 tamamlama | ⬜ | — | 4/12 hazır (M7 ters bakiye ✅, M12 kronoloji ✅, M4 belge-mükerrer 🟡, M1 oranlar 🟡). Öncelik: M8 karşı bacak (B.8 ile ortak), M5 zaman deseni, M9 cut-off, M10 konsantrasyon, M2 yaşlandırma |
| H.3 ETIC programını fixture'da uçtan uca çalıştır | ⬜ | H.2 | 20k e-ticaret fixture = doğal test alanı; bulgu üretimi doğrulanır |
| H.4 Denetim sayfası UI (program → çalıştır → bulgu → çalışma kağıdı) | 🔨 | — | 2026-07-04: **Çalışmalar sayfası + çalışma kağıdı üretimi ÇALIŞIYOR** — sidebar "Denetim" nav'ı; sektör pill'leri (devralma: ETIC=TIC+4); çalışma listesi → tıkla → BDS 230 formatlı kağıt: ref no/dönem/dayanak/amaç başlığı, **ilgili hesaplar defterden önek taramasıyla otomatik tespit** (ör. ETIC-01→102.01+108.01, 19k hareket), M7+M1 testleri koşuyor (diğer motorlar "planlı" görünür), MSUGT hesap nitelikleri, SMMM sonuç notu (kaydet, bellekte), TXT indirme (BOM'lu). API: GET /api/denetim/programlar · GET /api/denetim/kagit/:id · POST /api/denetim/kagit/:id/not. KALAN: bulgudan fişe iniş, kağıt durumu (hazırlandı/incelendi/onaylandı), Excel/PDF. **2026-07-04 ek: Excel-benzeri düzenlenebilir grid** — satır no + A-G kolon başlıkları; kağıt varsayılan 🔒 KİLİTLİ açılır ("güvenlikli ama istenildiğinde erişilebilir"), "düzenlemeyi aç" ile hücreler (Borç/Alacak/Not) yazılabilir; **müdahale deftere ASLA işlemez** — sistem değeri saklanır, sarı hücre + "sistem: X" izi + ↺ geri al; manuel satır ekleme (mavi, M rozeti, × sil); TOPLAM düzeltmelerle otomatik; "Kaydet ve kilitle" → POST /api/denetim/kagit/:id/duzenle (bellekte, serbest şema); TXT'de (M)/(D) işaretleri. Hedef: Excel'i tamamen ikame. **2026-07-05 ek: kayıtlara iniş** — kağıtta hesap koduna tıkla (▸/▾) → bakiyeyi oluşturan hareketler (sayfalı, muavin lite kalıbı: tarih/yevmiye no/fiş no/karşı hesap/**belge dayanağı**/yürüyen bakiye) → harekete tıkla → fiş detayı (satırlar + dayanak) → **"Fişi iptal et (VUK 217 düzeltme)"**: gerekçe + fark edilme tarihi zorunlu; kesin fiş DEĞİŞTİRİLMEZ, ters kayıt kesilir; kağıt/mizan kendiliğinden güncellenir (fixture testi: MAH-19571 iptali → 600.01 borç +1.228,22, hareket 11.479). Hareket ucuna fis_id+belge alanları eklendi |
| H.5 Kullanıcı düzenleme katmanı (proaktif değişiklik) | ⬜ | H.4 | Çalışma kopyala-düzenle (eşik/hesap), kapat/aç, yeni ekle; bulgu üretildiği sürümle saklanır |
| H.6 Mükellef sektör ataması → otomatik program seçimi | 🔒 | E.5 | Firma profili (NACE kodu) önkoşul; çoklu sektör birleşimi |
| H.7 Faz 2: önemlilik + BDS 530 örnekleme + BDS 505 teyit mektubu + TAR sektörü | ⏸ | H.4 | Teyit, D eşleştirme yapılarıyla birleşir |

## I. Bağımsız denetim raporlama hattı (2026-07-05 — Kiler raporu tersine analizinden)
> Kaynak: docs/analiz/05-kiler-tersine-analiz.md. Rapor anatomisi: görüş + dayanak + KAM + BDS 600 diğer
> hususlar + sorumluluklar + TTK 402/398 · TMS 29'lu karşılaştırmalı TAM SET (5 tablo) · 32 dipnot.

| İş | Durum | Bağımlılık | Not |
|----|-------|-----------|-----|
| I.1 Tasnif haritası (TDHP → TFRS sunum kalemi kural tablosu) | 🔨 | — | 2026-07-09: **data/tfrs-tasnif.json BAŞLADI** — Kiler raporunun kalem yapısından 29 bilanço + 11 GT satırı, her satırda `ayrim` (vade/ilişki/nitelik/ölçüm bölen boyutları) + `kaynak_nitelik` (hangi kayıt özelliği belirler). KİLİT DERS: TDHP kodu tek başına yetmez, kaydın niteliği (açıklama+fatura dayanağı+cari ilişki bayrağı+vade) belirler. docs/learnings/tfrs-tasnif.md. KALAN: haritayı mizana uygulayan motor + TFRS raporlama sayfası (I.3) |
| I.2 Düzeltme kayıt katmanı (TFRS overlay — kağıda bağlı, dayanaklı) | ⬜ | I.1 | Dual-defter kararının kağıt hali; VUK defterini ASLA değiştirmez. Düzeltme tipleri: TMS 29 endeksleme, TMS 40 GUD, TFRS 15 ilerleme, TMS 19 aktüeryal, TMS 2 NRV, TMS 12 ertelenmiş vergi (geçici fark envanteri = iki defterin farkı) |
| I.3 TFRS tam set üretimi (5 tablo + dipnot iskeleti, karşılaştırmalı) | 🔒 | I.2 | Kiler raporu şablon; mevcut mali tablolar sayfası VUK setidir, bu ayrı set |
| I.4 Yeni kağıt tipleri: uzman raporu değerlendirme (BDS 620), NRV karşılaştırma, endeksleme, eliminasyon, teyitler (BDS 505: banka/cari/avukat), bilanço sonrası olay taraması | ⬜ | H.4 | Kağıt altyapısı (grid+iniş) hazır, tipler eklenecek |
| I.5 Alan ihtiyaçları: cari kartta ilişkili taraf bayrağı; fişte parasal bayrak + iktisap tarihi | ⬜ | — | TMS 29 + Dipnot 26 gereksinimi; A bölümü enflasyon hazırlığıyla birleşir |
| I.6 Denetçi raporu metin üretimi (görüş şablonları: olumlu/sınırlı/olumsuz/görüş vermekten kaçınma + KAM blokları) | ⏸ | I.3 | BDS 700/701/705 şablonları |

> **⚖️ İKİ EKSEN KARARI (2026-07-05, kalıcı):** Raporlama sistemi ikiye bölünür — **G = Vergi denetimi ve
> raporlama** (VUK ekseni: muhasebe → cari çalışmalar → VUK mali tablolar → beyanname → vergi; kağıtlar
> doğrudan vergiyle ilgili) ve **H+I = Bağımsız denetim ve raporlama** (TFRS+BDS ekseni: tasdikli beyanname
> sonrası paralel süreç; gerçeğe uygun değer; denetçi beyanı test eder). Birbirine bağımlı, tamamen farklı
> yapılar. Referans: memory/iki-katmanli-raporlama + docs/analiz/05-kiler-tersine-analiz.md

## G. Vergi denetimi ve raporlama — vergi & beyanname yolu (2026-07-04)
> **🐛 BULUNAN VE DÜZELTİLEN HATA (2026-07-05 test taraması):** KDV1 taslağı 391/191'in yalnız tek yön
> hareketini okuyordu → **satış/alış İADELERİNE KÖRDÜ** (iade fişi defterde 391'i düşürür, taslak düşmezdi
> — devlete FAZLA beyan riski). Testler yakalamadı çünkü fixture'da hiç iade yok (mutlu yol). Fix: net
> hareket (alacak−borç / borç−alacak), sistem mahsup fişi hariç. Canlı kanıt: 200 TL iade → taslak −200 TL ✓,
> Kasım regresyonsuz ✓. DERS + İŞ: fixture'a iade/düzeltme senaryoları eklenmeli (⬜) ve beyanname
> handler'larına entegrasyon testi yazılmalı (⬜).
> **Beyanname doktrini (2026-07-05, kalıcı — hafızada):** dayanak = muhasebe kayıtları (her beyan satırı
> kaynak referanslı); format = GİB Beyanname Düzenleme Kılavuzu; manuel müdahale katmanı şart (deftere
> işlemez, M rozetli, dayanak alanlı). Mimari: docs/tasarim/beyanname-hatti.md. Uygulandı: KDV1 taslağına
> manuel satır katmanı (ilave matrah/indirim, dayanak zorunlu görünür) + tüm uçlara kaynak_notlari alanı.
| İş | Durum | Bağımlılık | Not |
|----|-------|-----------|-----|
| KDV beyannamesi (KDV1) taslak üretimi | ✅ | — | 2026-07-05: **"Vergi" sayfası kuruldu** (nav'da KDV → Vergi; KDV görünümü içine taşındı). GET /api/kdv-beyanname?ay=N: ay içi 391 muavin kırılımından oran bazlı matrahlar (oran = muavin kodundan), 191 indirim, 190 devir zinciri (önceki ay defterinden), ödenecek→360 / devreden→190. Fixture Aralık: ödenecek 25.256,31 ✓. KALAN: beyanname XML/GİB format, tevkifatlı işlemler, istisna satırları |
| Geçici vergi hesap kağıdı + taslağı | ✅ | — | 2026-07-05: GET /api/gecici-vergi?ceyrek — **kümülatif** (KVK mük.120): ticari kâr (GT dönem kârı) + KKEG (689* otomatik tarama + manuel satırlar) − istisna/indirim (manuel) = matrah × %25 − önceki dönem hesaplanan mahsubu = ödenecek. Manuel satırlar bellekte (deftere işlemez). Q4 fixture: 6.311.836,39 kâr → 380.004,30 ödenecek (Q3 mahsuplu) ✓. **Çift yönlü köprü:** mali tablolar GT ↔ geçici vergi kağıdı. Vergi takvimi sekmesi (genel süreler, parametreleşecek). KALAN: oran/istisna kodları firma parametresine (E.5), KKEG'nin nazımdan (950/951) otomatik beslenmesi, beyanname formatı |
| Geçici vergi beyannamesi (GİB format) | ⏸ | A.11 | Hesap kağıdı hazır; değerleme düzeltmeleri (amortisman vs.) girmeden beyan matrahı eksik kalır — D′ bekliyor |
| Muhtasar ve prim hizmet beyannamesi | ⏸ | E.6 | Bordro önkoşul |
| KKEG takibi (689/istisna kodları → mali kâr köprüsü) | ⬜ | — | Ticari kâr ≠ mali kâr; vergi karşılığı şu an ticari kârdan |
| Ba/Bs bildirimi | ⏸ | — | E-belge kapsamı daraldı ama yükümlülük sürüyor |
| **Vergi parametreleri altyapısı (10 kanun)** | ✅ | — | 2026-07-05: data/vergi-parametreleri.json (sürümlü, dogrulandi bayraklı, 2026 tebliğ değerleri: GV dilimleri 190k/400k/1M/5,3M, KV %25, VUK hadleri 12k/12k/25k, YD %25,49, damga oranları); GET /api/vergi-parametreler; Vergi sayfası "Parametreler" sekmesi; **geçici vergi oranı artık hardcode değil buradan okunuyor**. Harita: docs/learnings/vergi-kanunlari-haritasi.md |
| VUK hadlerinin motorlara bağlanması | ⬜ | G.7 | Fatura sınırı (12k) → kayıt uyarısı; şüpheli alacak haddi (25k) → TIC-03/M2 eşiği; amortisman sınırı (12k) → D′ doğrudan gider kuralı |
| Mükellef tipi (şahıs GV tarifesi / kurum KV) | 🔒 | E.5 | GVK 193 gereği: şahıs işletmesinde geçici vergi GV tarifesiyle; firma profili önkoşul |
| 6183 gecikme zammı hesaplayıcı | ⬜ | — | Aylık %4,5 (parametrede, teyit bayraklı); takvim sekmesine "bugün ödersen" hesabı |
| KDV tevkifatı (KDVK 9, 2 no.lu beyanname) + BSMV ayrıştırma (6802) | ⏸ | F.1 | Tevkifat fatura modülüyle; BSMV banka ekstresi eşleştirmesine masraf önerisi olarak |
| ÖTV (4760) sektörel modül · VİV (7338) kapsam dışı | ⏸ / ✖ | — | ÖTV alış maliyetine girer (VUK 262) kuralı şablonlara not edildi |

---

## 🎯 D — DEĞERLEME VERİ MİMARİSİ (uzun vadeli stratejik hedef)

> Doktrin: [docs/tasarim/degerleme-veri-mimarisi.md](tasarim/degerleme-veri-mimarisi.md)
> **Neden:** Enflasyon muhasebesi uygulanmıyor → tarihi maliyet gerçeği yansıtmıyor → GUD'u piyasadaki
> ekonomik aktörlerin fiyatlamasına dayandırmak ZORUNLU. Bağımsız denetim geleceğe yönelik tahmin içerir.
> **İlke:** Program sayı üretmez; sayının arkasındaki KANIT ZİNCİRİNİ taşınabilir ve sorgulanabilir kılar.
> Karar denetçinindir — program muhakemeyi besler, bayrak kaldırır, izi tutar.

**Kanıt Güven Skoru (KGS) = Kaynak Otoritesi × Veri Yeterliliği × Zaman Tazeliği × Emsal Yakınlığı**
(çarpım — zayıf halka güçlüyle telafi edilemez). TFRS 13 Seviye 1/2/3 hiyerarşisinin işletilebilir hâli.

| # | Adım | Getiri | Bağımlılık |
|---|------|--------|-----------|
| **D1** | Parametre kartı şeması (KGS + geçerlilik + kaynak katmanı + çakışma) | Temel — her şey buna dayanır | — |
| **D2** | KGS motoru (4 çarpan + eşiğe göre zorunluluk: 2. kaynak / duyarlılık) | Saf fonksiyon, test edilebilir | D1 |
| **D3** | **TCMB EVDS bağlantısı** (kur + KFE + faiz — resmî API, yasal) | **En yüksek getirili tek adım**: WS-KUR gerçek olur | D1 |
| **D4** | Emsal kartı (denetçi girer, yapılandırılmış: kaynak/tarih/konum/m²/fiyat/link) | GUD çalışmalarının gerçek girdisi | D2 |
| **D5** | Çelişki raporu (çok kaynak, >%15 sapma = bulgu + gerekçe zorunlu) | Çapraz doğrulama kuralı | D2-D4 |
| **D6** | Duyarlılık analizi motoru (±%10/±%20 senaryo — kıdem/DCF/ECL) | Düşük KGS'nin zorunlu çıktısı; dipnota gider | D2 |
| **D7** | İskonto oranı koridoru (TCMB avans / Hazine tahvil / Damodaran ERP / fiili borçlanma) | Hangi oran neden seçildi — kayda düşer | D1, D3 |
| **D8** | KAP dipnot madenciliği (emsal ömür/iskonto/temerrüt varsayımları) | Seviye 2 kalibrasyon hazinesi | D1 |
| **D9** | Lisanslı veri sağlayıcı entegrasyonu (sarı kuşak — ticari anlaşma) | Otomasyon derinleşir | D3, D4 |

**⚠ DÜRÜST KISIT (kırmızı kuşak):** İlan sitelerinden (sahibinden/hepsiemlak) otomatik veri çekme
kullanım şartlarında yasaklı + teknik engelli + **denetimde savunulamaz kanıt**. Doğru yol: denetçi
emsal veriyi manuel getirir (link + ekran görüntüsüyle), program yapılandırır + KGS'ler + hesaplar.
BDS 500 anlamında da bu daha sağlam: denetçi kanıtı bizzat görmüş ve değerlendirmiş olur.

**⚠ İLAN FİYATI ≠ SATIŞ FİYATI:** Ham ilan verisi sistematik yukarı yanlı → varlık şişirir → denetçiyi
yanlış tarafa hataya iter. Düzeltme katsayısı (haircut) şart ve o da dayanaklı olmalı. **Kalibrasyon
yöntemi TARTIŞILACAK.**

**Açık sorular (birlikte tartışılacak):** ilan→satış haircut kalibrasyonu · KGS eşikleri firma politikası mı ·
enflasyon beklentisi kaynağı (kıdem buna aşırı duyarlı) · makine emsali yoksa maliyet yaklaşımı mı ·
KGS düşükken denetçi görüşüne müdahale sınırı nerede (program haddini aşmamalı).
