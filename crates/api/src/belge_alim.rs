//! BELGE ALIM AKIŞI — üç yol, tek doğrulama.
//!
//! Bir belge sisteme üç şekilde girebilir. Üçü de **aynı denklem kapısından** geçer;
//! kayıt açma güvencesi hangi yoldan geldiğine bağlı değildir.
//!
//! | Mod | Kim doldurur | Güvence nereden gelir |
//! |---|---|---|
//! | **OTOMATIK** | Metin katmanlı PDF'ten birebir çıkarım | Denklem kanıtı — kanıtlanmayan alan DOLDURULMAZ |
//! | **SECIMLI** | Kullanıcı belge üzerinde sırayla seçer | Her seçim ayrı onaylanır + denklem |
//! | **MANUEL** | Kullanıcı elle yazar | Taramayla ÇELİŞKİ uyarısı + denklem |
//!
//! ## "Dijital PDF'te hata oranı 0" ne demek, ne demek değil
//!
//! Metin katmanından karakter çıkarmak **birebirdir** — orada hata olmaz. Ama hangi sayının
//! "matrah", hangisinin "toplam" olduğuna karar vermek sezgiseldir ve orada sıfır garanti
//! edilemez. Bu yüzden otomatik mod bir şey **iddia etmez**: yalnız belgenin kendi
//! denklemini SAĞLAYAN alan takımını doldurur. Denklem tutmuyorsa hiçbir şey doldurulmaz,
//! akış seçimli moda düşer. Sıfır hata bir temenni değil, yapısal sonuçtur:
//! **kanıtlanmamış hiçbir değer otomatik yazılmaz.**
//!
//! ## Çelişki neden uyarı, neden engel değil
//!
//! Manuel giriş taramayla çelişirse kullanıcı haklı olabilir — tarama yanılmış olabilir.
//! O yüzden ENGEL değil UYARI üretilir; ama sessiz geçilmez, kullanıcı görüp onaylar.
//! Sessizce kullanıcıya uymak da sessizce taramaya uymak kadar yanlıştır.

use axum::{extract::Path, Json};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

const PARAM: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/belge-parametreleri.json"));

static PARAMETRELER: OnceLock<serde_json::Value> = OnceLock::new();
fn parametreler() -> &'static serde_json::Value {
    PARAMETRELER.get_or_init(|| serde_json::from_str(PARAM).unwrap_or(serde_json::Value::Null))
}

/// Oturumlar bellekte tutulur. Kalıcılık kullanıcı kararıyla ertelendi; alım oturumu
/// zaten kısa ömürlüdür (belge yüklenir, alanlar onaylanır, fiş üretilir, oturum biter).
/// Sunucu yeniden başlarsa yarım kalan oturum kaybolur — kullanıcı belgeyi tekrar yükler.
static OTURUMLAR: OnceLock<Mutex<HashMap<String, Oturum>>> = OnceLock::new();
fn oturumlar() -> &'static Mutex<HashMap<String, Oturum>> {
    OTURUMLAR.get_or_init(|| Mutex::new(HashMap::new()))
}

#[derive(Serialize, Clone, Debug)]
pub struct Alan {
    pub kod: String,
    pub ad: String,
    pub tip: String,
    pub sira: u32,
    pub zorunlu: bool,
    pub ipucu: String,
    /// Kullanıcının seçtiği/yazdığı ham metin (belgede yazdığı gibi).
    pub ham: String,
    /// Sayıya/tarihe çevrilmiş hali. TUTAR için kuruş.
    pub deger: Option<i64>,
    /// Otomatik taramanın bulduğu değer — manuel girişle karşılaştırmak için saklanır.
    pub tarama: Option<i64>,
    pub tarama_ham: String,
    /// Alanın ETİKETİ belgede açıkça bulundu mu? Metin alanlarının otomatik doldurulma
    /// dayanağı budur — sayısal alanlarda dayanak denklemdir.
    pub etiket_bulundu: bool,
    /// Belgede kaç farklı aday bulundu. 1'den çoksa değer BELİRSİZDİR: doldurulur ama
    /// otomatik ONAYLANMAZ — kullanıcı hangisi olduğunu görmeli. (Bir faturada iki VKN
    /// vardır: satıcının ve alıcının. Hangisinin "karşı taraf" olduğu yöne bağlıdır.)
    pub aday_sayisi: usize,
    pub onaylandi: bool,
    /// Bu alanı kim doldurdu: OTOMATIK · SECIM · MANUEL
    pub kaynak: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct Oturum {
    pub id: String,
    pub belge_turu: String,
    pub belge_turu_ad: String,
    pub dil: String,
    pub mod_: String,
    pub alanlar: Vec<Alan>,
    /// Belgeden çıkarılan ham metin (otomatik mod ve çelişki kontrolü için).
    pub metin: String,
    pub tamamlandi: bool,
}

#[derive(Serialize, Clone, Debug)]
pub struct AlimBulgu {
    pub kod: String,
    pub onem: String,
    pub mesaj: String,
    pub cozum: String,
}
fn bulgu(kod: &str, onem: &str, mesaj: String, cozum: &str) -> AlimBulgu {
    AlimBulgu { kod: kod.into(), onem: onem.into(), mesaj, cozum: cozum.into() }
}

fn tl(k: i64) -> String { format!("{}.{:02}", k / 100, (k % 100).abs()) }

// ─────────────────────────────────────────────────────────────────────────────
// Değer çözümleme — tipe göre
// ─────────────────────────────────────────────────────────────────────────────

/// Ham metni alan tipine göre sayıya çevirir. TUTAR → kuruş, TARIH → yyyyaagg, diğeri None.
/// Çözemezse None — **tahmin edilmez**.
fn deger_coz(tip: &str, ham: &str, dil: &str) -> Option<i64> {
    match tip {
        "TUTAR" => crate::belge_oku::sayi_ayristir(ham, dil),
        "TARIH" => {
            let r: String = ham.chars().filter(|c| c.is_ascii_digit()).collect();
            // gg.aa.yyyy → yyyyaagg. Yalnız 8 haneli tam tarih kabul edilir; kısa biçim
            // (2 haneli yıl) belirsizdir ve muhasebede belirsiz tarih kabul edilemez.
            if r.len() == 8 { format!("{}{}{}", &r[4..8], &r[2..4], &r[0..2]).parse().ok() } else { None }
        }
        // SECENEK: sayısal karşılığı yok; geçerlilik `secenek_gecerli` ile ayrıca sınanır.
        "SECENEK" => None,
        "VKN" => {
            let r: String = ham.chars().filter(|c| c.is_ascii_digit()).collect();
            if r.len() == 10 || r.len() == 11 { r.parse().ok() } else { None }
        }
        _ => None,
    }
}

/// Belge türünün denklemlerini sınar. Eksik alan varsa o denklem ATLANMAZ, "eksik" diye
/// bildirilir — sessizce geçmek, denklemin hiç olmamasıyla aynı şeydir.
fn denklem_kontrol(o: &Oturum) -> (Vec<AlimBulgu>, usize, usize) {
    let mut b = Vec::new();
    let (mut tutan, mut toplam) = (0usize, 0usize);
    let tur = &parametreler()["belge_turleri"][&o.belge_turu];
    let deger = |kod: &str| -> Option<i64> {
        o.alanlar.iter().find(|a| a.kod == kod).and_then(|a| a.deger)
    };
    let ham = |kod: &str| -> String {
        o.alanlar.iter().find(|a| a.kod == kod).map(|a| a.ham.clone()).unwrap_or_default()
    };

    for d in tur["denklemler"].as_array().into_iter().flatten() {
        toplam += 1;
        let ad = d["ad"].as_str().unwrap_or("denklem");
        let hedef_kod = d["hedef"].as_str().unwrap_or("");
        let Some(hedef) = deger(hedef_kod) else {
            b.push(bulgu("A20", "BILGI", format!("{ad} — '{hedef_kod}' henüz girilmedi, doğrulanamadı."),
                "Alan doldurulunca denklem kendiliğinden sınanır."));
            continue;
        };

        // ── RAKAM ↔ YAZI — toplama denkleminden YAPISAL OLARAK farklı ────────────
        //
        // Toplama denklemleri DOĞRUSALDIR: her terim aynı katsayıyla bozulursa denklem
        // hâlâ tutar (bkz. belge_oku ölçek çapası). Yazıyla yazılan tutar ise ölçeği
        // MUTLAK olarak sabitler — "Üçbindörtyüzellialtı" 345.600 okunamaz.
        //
        // Bu yüzden makbuzun tek denklemi budur ve aynı zamanda faturada da en güçlü
        // ikinci kanıttır. Daha önce yalnız `belge_oku::coz` içinde çağrılıyordu; alım
        // akışı bu korumadan tamamen yoksundu ve `yaziyla` alanı METIN olduğu için
        // hiçbir sınamadan geçmeden onaylanıyordu.
        if d["tur"].as_str() == Some("YAZIYLA") {
            let kaynak_kod = d["kaynak"].as_str().unwrap_or("");
            let h = ham(kaynak_kod);
            if h.trim().is_empty() {
                b.push(bulgu("A20", "BILGI", format!("{ad} — '{kaynak_kod}' henüz girilmedi, doğrulanamadı."),
                    "Alan doldurulunca denklem kendiliğinden sınanır."));
                continue;
            }
            // BİRİM: `yaziyla_ayristir` LİRA döndürür ("bin lira" → 1000), alan değerleri
            // ise KURUŞ'tur. `belge_oku::coz` de aynı dönüşümü yapıyor (`x * 100`).
            // Yazıyla kuruş yazımı ("... lira elli kuruş") bu yolda ÇÖZÜLMEZ — o durumda
            // denklem tutmaz ve kullanıcı uyarılır; sessizce yanlış eşleşmektense doğrudur.
            match crate::belge_oku::yaziyla_ayristir(&h, &o.dil).and_then(|x| x.checked_mul(100)) {
                None => b.push(bulgu("A23", "UYARI",
                    format!("{ad} — yazıyla yazılan tutar çözülemedi: \"{}\".", h.trim()),
                    "Yazı okunamadığı için rakamla çapraz doğrulama YAPILAMADI. Belgedeki \
                     yazıyı olduğu gibi girdiğinden emin ol; çözülemezse bu belge \
                     doğrulanmamış sayılır.")),
                Some(y) if y != hedef => b.push(bulgu("A21", "ENGEL",
                    format!("{ad} TUTMUYOR: yazıyla {} ≠ rakamla {}.", tl(y), tl(hedef)),
                    "Rakam ile yazı çelişiyor. Bu, tahrifatın ve okuma hatasının en güçlü \
                     göstergesidir — hangisinin doğru olduğu belgeden kontrol edilmeli."),),
                Some(_) => {
                    tutan += 1;
                    b.push(bulgu("A00", "BILGI",
                        format!("{ad} doğrulandı ({}).", tl(hedef)),
                        "Yazı ile rakam örtüşüyor — ölçek mutlak olarak sabitlendi."));
                }
            }
            continue;
        }
        let mut sol = 0i64;
        let mut eksik = false;
        for t in d["terimler"].as_array().into_iter().flatten() {
            let kod = t["kod"].as_str().unwrap_or("");
            match deger(kod) {
                Some(v) => sol += v * t["isaret"].as_i64().unwrap_or(1),
                None => { eksik = true; break; }
            }
        }
        if eksik {
            b.push(bulgu("A20", "BILGI", format!("{ad} — tüm alanlar girilmedi, doğrulanamadı."),
                "Kalan alanları doldur."));
            continue;
        }
        // `mutlak`: yön farkı önemli değil, tutar aynı olmalı (mutabakatta net bakiye
        // borç ya da alacak yönlü yazılabilir).
        let uyar = if d["mutlak"].as_bool().unwrap_or(false) { sol.abs() != hedef.abs() } else { sol != hedef };
        if uyar {
            b.push(bulgu("A21", "ENGEL",
                format!("{ad} TUTMUYOR: hesaplanan {} ≠ belgedeki {} (fark {}).",
                    tl(sol), tl(hedef), tl(sol - hedef)),
                "Alanlardan biri yanlış okunmuş veya yanlış seçilmiş. Belgeye dönüp \
                 farkı veren alanı düzelt — denklem tutmadan kayıt açılmaz."));
        } else {
            tutan += 1;
            b.push(bulgu("A00", "BILGI", format!("{ad} doğrulandı ({}).", tl(hedef)),
                "Bu alan takımı belgenin kendi aritmetiğiyle kanıtlandı."));
        }
    }
    (b, tutan, toplam)
}

// ─────────────────────────────────────────────────────────────────────────────
// Oturum kurma
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct BaslaGirdi {
    pub belge_turu: String,
    #[serde(default)] pub dil: String,
    /// OTOMATIK · SECIMLI · MANUEL
    #[serde(default)] pub mod_: String,
    /// Metin katmanlı PDF'ten çıkarılmış ham metin (varsa).
    #[serde(default)] pub metin: String,
    /// Oturum kimliği — çağıran verir (deterministik test için). Boşsa metinden türetilir.
    #[serde(default)] pub oturum_id: String,
}

/// POST /api/belge/alim/basla — parametre listesini SIRAYLA döner, oturum açar.
pub async fn alim_basla(Json(g): Json<BaslaGirdi>) -> Json<serde_json::Value> {
    let tur = &parametreler()["belge_turleri"][&g.belge_turu];
    if tur.is_null() {
        return Json(serde_json::json!({
            "hata": format!("bilinmeyen belge türü: {}", g.belge_turu),
            "gecerli": parametreler()["belge_turleri"].as_object()
                .map(|o| o.keys().cloned().collect::<Vec<_>>()).unwrap_or_default(),
        }));
    }
    let dil = if g.dil.is_empty() {
        let (d, _) = crate::belge_oku::dil_sez(&g.metin);
        if d.is_empty() { "tr".to_string() } else { d }
    } else { g.dil.clone() };

    let mut alanlar: Vec<Alan> = tur["parametreler"].as_array().into_iter().flatten().map(|p| Alan {
        kod: p["kod"].as_str().unwrap_or("").into(),
        ad: p["ad"].as_str().unwrap_or("").into(),
        tip: p["tip"].as_str().unwrap_or("METIN").into(),
        sira: p["sira"].as_u64().unwrap_or(0) as u32,
        zorunlu: p["zorunlu"].as_bool().unwrap_or(false),
        ipucu: p["ipucu"].as_str().unwrap_or("").into(),
        ham: String::new(), deger: None, tarama: None, tarama_ham: String::new(),
        etiket_bulundu: false, aday_sayisi: 0,
        onaylandi: false, kaynak: String::new(),
    }).collect();
    alanlar.sort_by_key(|a| a.sira); // kullanıcı SIRAYLA seçecek — sıra sözleşmenin parçası

    let id = if g.oturum_id.is_empty() { oturum_id(&g.metin, &g.belge_turu) }
             else { g.oturum_id.clone() };

    let mod_ = if g.mod_.is_empty() { "SECIMLI".to_string() } else { g.mod_.clone() };
    let mut o = Oturum {
        id: id.clone(), belge_turu: g.belge_turu.clone(),
        belge_turu_ad: tur["ad"].as_str().unwrap_or("").into(),
        dil, mod_: mod_.clone(), alanlar, metin: g.metin.clone(), tamamlandi: false,
    };

    // Metin varsa HER MODDA taranır. Otomatik modda doldurmak için, diğerlerinde
    // kullanıcının girdisiyle karşılaştırmak için — çelişki uyarısı buradan doğar.
    let bulunan = tara(&mut o);
    let otomatik_sonuc = if mod_ == "OTOMATIK" { Some(otomatik_doldur(&mut o)) } else { None };

    oturumlar().lock().unwrap().insert(id.clone(), o.clone());
    Json(serde_json::json!({
        "oturum": o,
        "aktif_alan": sonraki_alan(&o),
        "tarama_bulunan": bulunan,
        "otomatik": otomatik_sonuc,
        "mod_bilgi": parametreler()["modlar"][&mod_],
        "tur_uyari": tur_uyarisi(&g.metin, &o.dil, &g.belge_turu),
    }))
}

/// Oturum kimliği — belgenin İÇERİĞİNDEN ve türünden türer.
///
/// Eskiden `metin.len() * 31 + belge_turu.len()` idi ve iki yönden çakışıyordu:
/// tür adları aynı uzunlukta olabiliyor ("MAKBUZ" ve "FATURA" ikisi de 6 harf), metin
/// uzunluğu da içeriği ayırt etmiyor. Sonuç: aynı belgeyi farklı türle açmak **eski
/// oturumun üstüne yazıyordu** — kullanıcı türü düzeltmek isterse onayları sessizce
/// kayboluyordu. İçeriği karıştırmak bu ailenin tamamını kapatır.
fn oturum_id(metin: &str, tur: &str) -> String {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    metin.hash(&mut h);
    0u8.hash(&mut h); // ayraç: "ab"+"c" ile "a"+"bc" aynı özeti vermesin
    tur.hash(&mut h);
    format!("ALM-{:x}", h.finish())
}

/// **A12 — BELGE TÜRÜ UYUŞMAZLIĞI.**
///
/// `belge_turu` istemciden gelir; motor onu doğru varsayardı. Oysa belgenin kendisi ne
/// olduğunu SÖYLÜYOR ve `belge_oku::belge_turu()` bunu zaten okuyabiliyordu — iki bilgi
/// hiç karşılaştırılmıyordu.
///
/// Sonucu ağırdı: kullanıcı MAKBUZ açtığında belgede kocaman "FATURA" yazsa bile alan
/// kümesi makbuzunki oluyordu (unvan/tarih/toplam/yazıyla). Fatura için zorunlu olan
/// **VKN, belge no, matrah, KDV** alanları hiç sorulmuyor, `matrah + KDV = toplam`
/// denklemi hiç kurulamıyor ve belge doğrulanmamış halde kayda gidiyordu.
///
/// **Sessizce DÜZELTMEZ, uyarır.** Tür değişimi alan kümesini değiştirir; bu kullanıcının
/// kararıdır (motor karar vermez doktrini). Ayrıca tespit de yanılabilir: "fatura" sözcüğü
/// bir irsaliyenin ya da dekontun içinde de geçebilir.
fn tur_uyarisi(metin: &str, dil: &str, secilen: &str) -> serde_json::Value {
    if metin.trim().is_empty() { return serde_json::Value::Null; }
    let sezilen = crate::belge_oku::belge_turu(metin, dil);
    if sezilen.is_empty() || sezilen == secilen { return serde_json::Value::Null; }
    let s = &parametreler()["belge_turleri"][&sezilen];
    if s.is_null() { return serde_json::Value::Null; }

    let eksik: Vec<String> = s["parametreler"].as_array().into_iter().flatten()
        .filter(|p| p["zorunlu"].as_bool().unwrap_or(false))
        .filter_map(|p| p["ad"].as_str().map(|x| x.to_string()))
        .collect();
    serde_json::json!({
        "kod": "A14",
        "secilen": secilen,
        "sezilen": sezilen,
        "sezilen_ad": s["ad"],
        "sorulmayan_zorunlu_alanlar": eksik,
        "mesaj": format!(
            "Belgede \"{}\" yazıyor ama alım \"{}\" olarak açıldı. Alan kümesi yanlış olabilir \
             — tür değiştirilmezse bu belgenin zorunlu alanları hiç sorulmaz ve denklemi kurulamaz.",
            s["ad"].as_str().unwrap_or(&sezilen), secilen),
        "not": "Motor türü kendiliğinden DEĞİŞTİRMEZ: alan kümesini değiştirmek kullanıcının \
                kararıdır ve tespit de yanılabilir (bir irsaliyenin içinde de 'fatura' geçebilir).",
    })
}

/// Sırada onay bekleyen ilk alan — arayüz kullanıcıyı buna yönlendirir.
fn sonraki_alan(o: &Oturum) -> Option<Alan> {
    o.alanlar.iter().find(|a| !a.onaylandi).cloned()
}

// ─────────────────────────────────────────────────────────────────────────────
// Tarama — metin katmanından aday değerler
// ─────────────────────────────────────────────────────────────────────────────

/// Satırdaki etiket, aradığımız alan etiketiyle eşleşiyor mu?
///
/// Tam metin eşleşmesi tek başına yetmiyor: PDF'lerde gömülü font eşlemesi bozulabiliyor ve
/// "KDV tutarı" metin katmanında "KDV tutar·" olarak çıkabiliyor. Tam eşleşme arayan tarama
/// böyle bir satırı hiç görmüyor, alan boş kalıyor ve otomatik mod gereksiz yere düşüyordu.
///
/// Bu yüzden önce tam eşleşme denenir; olmazsa `duzelt::uzaklik` ile — el yazısı/OCR
/// karışmalarını ucuzlatan aynı ölçüyle — etiketin BAŞ KISMI karşılaştırılır. Eşik dardır
/// (etiket uzunluğunun beşte biri): amaç bozuk karakteri tolere etmek, alanları karıştırmak değil.
fn etiket_gecer(satir: &str, etiket: &str) -> bool {
    let sade = crate::belge_oku::sadelestir(satir);
    if sade.contains(etiket) { return true; }
    // Yalnız etiket bölümü (':' öncesi) karşılaştırılır — değerin içeriği eşleşmeyi kirletmesin.
    let Some((bas, _)) = satir.split_once(':') else { return false };
    let bas = crate::belge_oku::sadelestir(bas);
    if bas.is_empty() { return false; }
    let esik = (etiket.chars().count() as f32 * 0.2).max(1.0);
    crate::duzelt::uzaklik(&bas, etiket) <= esik
}

/// Belge metnini tarayıp her alan için ADAY değer bulur ve DENKLEMLE seçim yapar.
///
/// Eski tarama tek düzen varsayıyordu (`Etiket: değer`) ve ilk eşleşmeyi alıyordu.
/// Gerçek belgede bu üç şekilde çöküyordu: değer alt satırda kalıyor, sağa yaslı tutar
/// kaçıyor, adresteki "No:12" fatura numarası sanılıyordu. Artık her alan için TÜM adaylar
/// çıkarılır ve aralarından **belgenin kendi denklemini sağlayan** takım seçilir.
///
/// Denklem burada yalnız doğrulama değil, AYIRT ETME aracıdır: 8.000 + 1.600 = 9.600
/// tutuyorsa o üçlü doğrudur; başka bir kombinasyon tutmuyorsa yanlıştır. Bu, hiçbir
/// düzen kuralının veremeyeceği bir kesinlik verir.
fn tara(o: &mut Oturum) -> usize {
    if o.metin.trim().is_empty() { return 0; }
    let dil = o.dil.clone();
    let satirlar: Vec<&str> = o.metin.lines().collect();

    // 1) ÖĞRENİLMİŞ ŞABLON — bu düzeni daha önce gördük mü?
    let sablon_id = crate::belge_ogren::parmak_izi(&o.metin);
    let alan_tipleri: Vec<(String, String)> =
        o.alanlar.iter().map(|a| (a.kod.clone(), a.tip.clone())).collect();
    let ogrenilen = crate::belge_ogren::uygula(&sablon_id, &satirlar, &alan_tipleri);

    // 2) HER ALAN İÇİN TÜM ADAYLAR
    let mut adaylar: HashMap<String, Vec<crate::belge_ogren::Aday>> = HashMap::new();
    for a in o.alanlar.iter() {
        let mut etiketler: Vec<String> = vec![crate::belge_oku::sadelestir(&a.ad)];
        for e in crate::belge_oku::alan_esanlamlilari(&a.kod, &dil) {
            let s = crate::belge_oku::sadelestir(&e);
            if !s.is_empty() && !etiketler.contains(&s) { etiketler.push(s); }
        }
        etiketler.retain(|e| e.chars().count() >= 3); // "no" gibi kısa etiket adreste de geçer
        let mut liste = crate::belge_ogren::adaylar(&satirlar, &etiketler, &a.tip);
        // Öğrenilmiş değer varsa en öne konur — ama yine denklemden geçecek.
        if let Some(d) = ogrenilen.get(&a.kod) {
            liste.insert(0, crate::belge_ogren::Aday {
                deger: d.clone(), strateji: "öğrenilmiş şablon".into(), satir: 0, puan: 200,
                gerekce: format!("bu düzen daha önce görüldü ({sablon_id})") });
        }
        adaylar.insert(a.kod.clone(), liste);
    }

    // 3) DENKLEMLE AYIRT ETME — denklem alanlarında adayları deneyip tutan takımı seç.
    let secilen = denklemle_sec(o, &adaylar, &dil);

    // 4) Sonucu alanlara yaz.
    let mut bulunan = 0;
    for a in o.alanlar.iter_mut() {
        let Some(liste) = adaylar.get(&a.kod) else { continue };
        let secim = secilen.get(&a.kod).cloned()
            .or_else(|| liste.first().map(|x| x.deger.clone()));
        let Some(d) = secim else { continue };
        a.etiket_bulundu = true;
        a.tarama_ham = d.clone();
        a.tarama = deger_coz(&a.tip, &d, &dil);
        // Belirsizlik ölçüsü: ANLAMLI olarak farklı kaç değer var?
        // Ham metinle saymak sahte belirsizlik üretiyordu — aynı ünvan iki stratejiden
        // birkaç boşluk farkıyla çıkıp "2 aday" sanılıyordu. Sayısal alanda çözülmüş
        // değer, metinde sadeleştirilmiş biçim karşılaştırılır.
        let mut farkli: Vec<String> = liste.iter()
            .map(|x| match deger_coz(&a.tip, &x.deger, &dil) {
                Some(v) => v.to_string(),
                None => crate::belge_oku::sadelestir(&x.deger),
            })
            .filter(|x| !x.is_empty()).collect();
        farkli.sort(); farkli.dedup();
        a.aday_sayisi = farkli.len().max(1);
        bulunan += 1;
    }
    bulunan
}

/// Denklem alanlarının adaylarını deneyip belgenin aritmetiğini SAĞLAYAN takımı bulur.
///
/// Kombinasyon sayısı küçüktür (alan başına birkaç aday), bu yüzden kaba kuvvet yeterli
/// ve en güvenilir yol. Tutan takım bulunamazsa hiçbir şey zorlanmaz — puanı yüksek aday
/// kullanılır ve denklem zaten "tutmadı" diyerek otomatik doldurmayı engeller.
fn denklemle_sec(o: &Oturum, adaylar: &HashMap<String, Vec<crate::belge_ogren::Aday>>, dil: &str)
    -> HashMap<String, String>
{
    let mut sonuc = HashMap::new();
    let tur = &parametreler()["belge_turleri"][&o.belge_turu];
    for d in tur["denklemler"].as_array().into_iter().flatten() {
        let hedef_kod = d["hedef"].as_str().unwrap_or("").to_string();
        let mut kodlar: Vec<String> = d["terimler"].as_array().into_iter().flatten()
            .filter_map(|t| t["kod"].as_str().map(String::from)).collect();
        let isaretler: Vec<i64> = d["terimler"].as_array().into_iter().flatten()
            .map(|t| t["isaret"].as_i64().unwrap_or(1)).collect();
        kodlar.push(hedef_kod.clone());

        // Her alan için en fazla 6 aday dene — gerçek belgede bu fazlasıyla yeter.
        let secenekler: Vec<Vec<i64>> = kodlar.iter().map(|k| {
            adaylar.get(k).map(|v| v.iter().take(6)
                .filter_map(|x| crate::belge_oku::sayi_ayristir(&x.deger, dil))
                .collect::<Vec<_>>()).unwrap_or_default()
        }).collect();
        if secenekler.iter().any(|v| v.is_empty()) { continue; }

        let mutlak = d["mutlak"].as_bool().unwrap_or(false);
        let mut indis = vec![0usize; kodlar.len()];
        let mut bulundu = false;
        'ara: loop {
            let mut sol = 0i64;
            for (n, _) in isaretler.iter().enumerate() { sol += secenekler[n][indis[n]] * isaretler[n]; }
            let hedef = secenekler[kodlar.len() - 1][indis[kodlar.len() - 1]];
            let tutar = if mutlak { sol.abs() == hedef.abs() } else { sol == hedef };
            if tutar && hedef != 0 {
                for (n, k) in kodlar.iter().enumerate() {
                    if let Some(v) = adaylar.get(k) {
                        // Seçilen sayısal değere karşılık gelen HAM metni geri bul.
                        if let Some(a) = v.iter().take(6).find(|x|
                            crate::belge_oku::sayi_ayristir(&x.deger, dil) == Some(secenekler[n][indis[n]])) {
                            sonuc.insert(k.clone(), a.deger.clone());
                        }
                    }
                }
                bulundu = true;
                break 'ara;
            }
            // Sonraki kombinasyon
            let mut p = 0;
            loop {
                if p >= indis.len() { break 'ara; }
                indis[p] += 1;
                if indis[p] < secenekler[p].len() { break; }
                indis[p] = 0; p += 1;
            }
        }
        let _ = bulundu;
    }
    sonuc
}

/// OTOMATİK MOD — yalnız DENKLEMLE KANITLANMIŞ alan takımını doldurur.
///
/// Kanıt yoksa hiçbir şey yazılmaz. "Muhtemelen doğrudur" diye doldurmak, sıfır hata
/// hedefiyle bağdaşmaz: kullanıcı dolu bir alanı kontrol etmez.
fn otomatik_doldur(o: &mut Oturum) -> serde_json::Value {
    // Önce adayları geçici olarak yerleştir.
    for a in o.alanlar.iter_mut() {
        // Sayısal alan: değer çözülmüş olmalı. Metin alanı: etiketin belgede bulunmuş olması yeter.
        if a.tarama.is_some() || (a.tip == "METIN" && a.etiket_bulundu) {
            a.deger = a.tarama;
            a.ham = a.tarama_ham.clone();
            a.kaynak = "OTOMATIK".into();
        }
    }
    let (mut bulgular, tutan, toplam) = denklem_kontrol(o);
    let kanitli = toplam > 0 && tutan == toplam;

    if !kanitli {
        // KANIT YOK → geri al. Yarım doldurulmuş form, boş formdan tehlikelidir.
        for a in o.alanlar.iter_mut() {
            if a.kaynak == "OTOMATIK" {
                a.deger = None; a.ham = String::new(); a.kaynak = String::new();
            }
        }
        return serde_json::json!({
            "doldurma": "YAPILMADI",
            "sebep": if toplam == 0 {
                "Bu belge türünde doğrulayıcı denklem yok — otomatik doldurma kanıtlanamaz."
            } else {
                "Belgenin kendi denklemi taramayla tutmadı; kanıtlanmamış değer yazılmaz."
            },
            "denklem_tutan": tutan, "denklem_toplam": toplam,
            "oneri": "SECIMLI moda geç: alanları belge üzerinde sırayla seçip onayla.",
            "bulgular": bulgular,
        });
    }
    // Kanıtlandı → ama HER alan değil. Yalnız denklemin PARÇASI olan alanlar ya da
    // belgede tek adayı bulunanlar onaylı sayılır. Birden çok aday varsa değer
    // doldurulur ama onay kullanıcıya bırakılır: denklem "8.000+1.600=9.600" üçlüsünü
    // kanıtlar, faturadaki İKİ VKN'den hangisinin karşı taraf olduğunu kanıtlamaz.
    let denklem_alanlari: std::collections::HashSet<String> = {
        let tur = &parametreler()["belge_turleri"][&o.belge_turu];
        let mut k = std::collections::HashSet::new();
        for d in tur["denklemler"].as_array().into_iter().flatten() {
            if let Some(h) = d["hedef"].as_str() { k.insert(h.to_string()); }
            for t in d["terimler"].as_array().into_iter().flatten() {
                if let Some(c) = t["kod"].as_str() { k.insert(c.to_string()); }
            }
        }
        k
    };
    let mut belirsiz: Vec<String> = Vec::new();
    for a in o.alanlar.iter_mut() {
        if a.kaynak != "OTOMATIK" { continue; }
        if denklem_alanlari.contains(&a.kod) || a.aday_sayisi <= 1 { a.onaylandi = true; }
        else { belirsiz.push(format!("{} ({} aday)", a.ad, a.aday_sayisi)); }
    }
    if !belirsiz.is_empty() {
        bulgular.push(bulgu("A22", "UYARI",
            format!("Belgede birden çok aday bulunan alanlar onaylanmadı: {}.", belirsiz.join(", ")),
            "Doldurulan değeri kontrol edip onayla. Örnek: faturada iki VKN vardır \
             (satıcının ve alıcının); hangisinin karşı taraf olduğu fatura yönüne bağlıdır."));
    }
    serde_json::json!({
        "doldurma": "YAPILDI",
        "sebep": "Tüm denklemler taramayla tuttu — değerler belgenin kendi aritmetiğiyle kanıtlandı.",
        "denklem_tutan": tutan, "denklem_toplam": toplam,
        "bulgular": bulgular,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Değer girme (seçim veya manuel) + onay
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct DegerGirdi {
    pub oturum_id: String,
    pub kod: String,
    pub ham: String,
    /// SECIM (belge üzerinden) · MANUEL (elle yazıldı)
    #[serde(default)] pub kaynak: String,
}

/// POST /api/belge/alim/deger — bir alana değer yazar ve ONAY BEKLETİR.
///
/// Onay ayrı adımdır: kullanıcı seçtiğini görmeden ilerlememeli. Seçimli akışın tamamı
/// "seç → gör → onayla" üçlüsüdür; onayı seçimin içine gömmek kontrolü ortadan kaldırır.
pub async fn alim_deger(Json(g): Json<DegerGirdi>) -> Json<serde_json::Value> {
    let mut kilit = oturumlar().lock().unwrap();
    let Some(o) = kilit.get_mut(&g.oturum_id) else {
        return Json(serde_json::json!({ "hata": "oturum bulunamadı" }));
    };
    let dil = o.dil.clone();
    let Some(i) = o.alanlar.iter().position(|a| a.kod == g.kod) else {
        return Json(serde_json::json!({ "hata": format!("alan yok: {}", g.kod) }));
    };

    let tip = o.alanlar[i].tip.clone();
    let cozulen = deger_coz(&tip, &g.ham, &dil);
    let mut bulgular: Vec<AlimBulgu> = Vec::new();

    // SECENEK alanı yalnız tanımlı değerlerden birini alabilir — serbest metin kabul edilirse
    // "SATIŞ" yerine "satis fatuası" yazılır ve fiş yanlış yöne kurulur.
    if tip == "SECENEK" {
        let kod = o.alanlar[i].kod.clone();
        let secenekler: Vec<String> = parametreler()["belge_turleri"][&o.belge_turu]["parametreler"]
            .as_array().into_iter().flatten()
            .find(|p| p["kod"].as_str() == Some(kod.as_str()))
            .and_then(|p| p["secenekler"].as_array().cloned()).unwrap_or_default()
            .iter().filter_map(|x| x.as_str().map(String::from)).collect();
        let g_up = g.ham.trim().to_uppercase();
        if !secenekler.iter().any(|x| x.to_uppercase() == g_up) {
            bulgular.push(bulgu("A13", "ENGEL",
                format!("'{}' geçerli bir seçenek değil.", g.ham),
                &format!("Şunlardan biri olmalı: {}", secenekler.join(" · "))));
        }
    }

    // METIN ve SECENEK'in sayısal karşılığı YOKTUR — bu bir hata değil, tasarımdır.
    // (SECENEK bu listede unutulmuştu ve "Fatura yönü" alanı hep reddediliyordu;
    //  geçerliliği A13 ayrıca sınıyor.)
    if cozulen.is_none() && !g.ham.trim().is_empty() && tip != "METIN" && tip != "SECENEK" {
        bulgular.push(bulgu("A10", "ENGEL",
            format!("'{}' bir {} olarak okunamadı.", g.ham, tip),
            "Değeri belgede yazdığı gibi gir. Tarih 8 haneli olmalı (gg.aa.yyyy), \
             tutarda binlik/ondalık ayracı belgeye uygun olmalı."));
    }

    // ÇELİŞKİ — manuel giriş taramayla uyuşmuyorsa sessiz geçilmez.
    let tarama = o.alanlar[i].tarama;
    if let (Some(t), Some(c)) = (tarama, cozulen) {
        if t != c {
            let (a, b) = if tip == "TUTAR" { (tl(t), tl(c)) } else { (t.to_string(), c.to_string()) };
            bulgular.push(bulgu("A11", "UYARI",
                format!("Girdiğin değer ({b}) belgeden okunanla ({a}) ÇELİŞİYOR."),
                "İkisinden biri yanlış. Belgeye bakıp doğrula: tarama yanılmış olabilir, \
                 senin okuman da. Onaylarsan senin değerin geçerli olur."));
        }
    }

    let engel = bulgular.iter().any(|x| x.onem == "ENGEL");
    if !engel {
        let a = &mut o.alanlar[i];
        a.ham = g.ham.clone();
        a.deger = cozulen;
        a.kaynak = if g.kaynak.is_empty() { "SECIM".into() } else { g.kaynak.clone() };
        a.onaylandi = false; // ONAY AYRI ADIM
    }

    let (denklem, tutan, toplam) = denklem_kontrol(o);
    bulgular.extend(denklem);
    let alan = o.alanlar[i].clone();
    let ozet = serde_json::json!({
        "alan": alan, "onay_bekliyor": !engel,
        "denklem_tutan": tutan, "denklem_toplam": toplam,
        "bulgular": bulgular,
        "not": if engel { "Değer kabul edilmedi." } else { "Değeri kontrol et ve onayla." },
    });
    Json(ozet)
}

#[derive(Deserialize)]
pub struct OnayGirdi { pub oturum_id: String, pub kod: String }

/// POST /api/belge/alim/onayla — kullanıcı gördü ve onayladı; sıradaki alana geç.
pub async fn alim_onayla(Json(g): Json<OnayGirdi>) -> Json<serde_json::Value> {
    let mut kilit = oturumlar().lock().unwrap();
    let Some(o) = kilit.get_mut(&g.oturum_id) else {
        return Json(serde_json::json!({ "hata": "oturum bulunamadı" }));
    };
    let Some(i) = o.alanlar.iter().position(|a| a.kod == g.kod) else {
        return Json(serde_json::json!({ "hata": format!("alan yok: {}", g.kod) }));
    };
    if o.alanlar[i].zorunlu && o.alanlar[i].ham.trim().is_empty() {
        return Json(serde_json::json!({
            "hata": format!("'{}' zorunlu alan, boş onaylanamaz", o.alanlar[i].ad) }));
    }
    o.alanlar[i].onaylandi = true;
    let sonraki = sonraki_alan(o);
    let (bulgular, tutan, toplam) = denklem_kontrol(o);
    Json(serde_json::json!({
        "onaylandi": o.alanlar[i].kod, "sonraki_alan": sonraki,
        "denklem_tutan": tutan, "denklem_toplam": toplam, "bulgular": bulgular,
    }))
}

/// POST /api/belge/alim/tamamla — tüm alanlar onaylı mı, denklemler tutuyor mu?
pub async fn alim_tamamla(Json(g): Json<OnayGirdi>) -> Json<serde_json::Value> {
    let mut kilit = oturumlar().lock().unwrap();
    let Some(o) = kilit.get_mut(&g.oturum_id) else {
        return Json(serde_json::json!({ "hata": "oturum bulunamadı" }));
    };
    let mut bulgular: Vec<AlimBulgu> = Vec::new();
    let eksik: Vec<String> = o.alanlar.iter()
        .filter(|a| a.zorunlu && (a.ham.trim().is_empty() || !a.onaylandi))
        .map(|a| a.ad.clone()).collect();
    if !eksik.is_empty() {
        bulgular.push(bulgu("A12", "ENGEL",
            format!("Zorunlu alanlar eksik veya onaysız: {}.", eksik.join(", ")),
            "Her zorunlu alan doldurulup ONAYLANMALI (VUK 227 — eksik belgeyle kayıt yapılmaz)."));
    }
    let (denklem, tutan, toplam) = denklem_kontrol(o);
    bulgular.extend(denklem);

    // ── DOĞRULAMA DURUMU — motor yapmadığı şeyi YAPTIM DEMEZ ──────────────────
    //
    // Eskiden `tamamlandi = !engel` idi ve yanıt her durumda "denklemler tutuyor"
    // diyordu. Oysa eksik terim ENGEL değil BILGI üretir: bir DEKONT'ta yalnız kapanış
    // bakiyesi girilirse zincir hiç sınanmaz, MAKBUZ'da ise denklem hiç tanımlı değildi.
    // Her iki durumda da motor `denklem_tutan: 0` olduğu halde "denklemler tutuyor"
    // yazıyordu — doğrulanmamış belge doğrulanmış gibi görünüyordu.
    //
    // TAMAMLANMAYI ENGELLEMİYORUZ: elle giriş kullanıcının beyanıdır ve makbuz gibi
    // türlerde aritmetik özdeşlik gerçekten yoktur. Ama durum AÇIKÇA işaretlenir ki
    // hem kullanıcı hem denetim izi neyin kanıtlandığını, neyin kanıtlanmadığını görsün.
    let dogrulama = if toplam == 0 { "DOGRULANMADI" }
                    else if tutan == toplam { "DOGRULANDI" }
                    else if tutan == 0 { "DOGRULANMADI" }
                    else { "KISMEN" };
    if dogrulama != "DOGRULANDI" {
        bulgular.push(bulgu("A24", "UYARI",
            format!("Bu belge DOĞRULANMADI: {tutan}/{toplam} denklem sınandı."),
            "Belge kayda alınabilir ama aritmetik kanıtı yoktur — değerler yalnız \
             kullanıcı beyanına dayanır. Fişte ve denetim izinde bu şekilde görünür."));
    }

    let engel = bulgular.iter().any(|x| x.onem == "ENGEL");
    o.tamamlandi = !engel;

    // ── ÖĞRENME — kesinleşen her giriş motoru besler ────────────────────────────
    // Kullanıcı alanları onaylayıp tamamladığında, HER ONAYLANMIŞ DEĞERİN belgede
    // nerede durduğu çıkarılıp bu düzenin şablonuna yazılır. Manuel giriş de öğretir:
    // muhasebeci elle yazdığı değer belgede geçiyorsa, motor "demek ki bu alan burada
    // duruyormuş" der ve aynı şablondaki bir sonraki belgeyi kendi doldurur.
    //
    // Öğrenme yalnız TAMAMLANMIŞ oturumdan yapılır: yarıda bırakılmış, denklemi tutmayan
    // bir oturumdan kural çıkarmak yanlışı kalıcılaştırırdı.
    let mut ogrenilen = 0usize;
    let mut sablon_id = String::new();
    if o.tamamlandi && !o.metin.trim().is_empty() {
        sablon_id = crate::belge_ogren::parmak_izi(&o.metin);
        let onaylar: Vec<(String, String)> = o.alanlar.iter()
            .filter(|a| a.onaylandi && !a.ham.trim().is_empty())
            .map(|a| (a.kod.clone(), a.ham.clone())).collect();
        let ad = o.alanlar.iter().find(|a| a.kod == "unvan")
            .map(|a| a.ham.clone()).unwrap_or_default();
        ogrenilen = crate::belge_ogren::ogren(&sablon_id, &ad, &o.belge_turu, &o.metin, &onaylar);
    }

    Json(serde_json::json!({
        "oturum_id": o.id, "tamamlandi": o.tamamlandi,
        "ogrenme": { "sablon_id": sablon_id, "yeni_kural": ogrenilen,
            "not": if ogrenilen > 0 {
                "Bu düzen öğrenildi — aynı şablondaki sonraki belge kendiliğinden dolacak."
            } else if o.tamamlandi { "Bu düzen zaten biliniyordu; kurallar tazelendi." }
            else { "Oturum tamamlanmadığı için öğrenme yapılmadı." } },
        "denklem_tutan": tutan, "denklem_toplam": toplam,
        "dogrulama": dogrulama,
        "alanlar": o.alanlar,
        "bulgular": bulgular,
        "not": if engel { "Belge kayda alınamaz — ENGEL bulgusu var." }
               else if dogrulama == "DOGRULANDI" {
                   "Tüm alanlar onaylı ve denklemler tutuyor; fiş taslağı üretilebilir." }
               else if dogrulama == "KISMEN" {
                   "Alanlar onaylı ama denklemlerin bir kısmı sınanamadı — fiş taslağı \
                    üretilebilir, ancak belge KISMEN doğrulanmıştır." }
               else {
                   "Alanlar onaylı ama HİÇBİR denklem sınanamadı — bu belgenin aritmetik \
                    kanıtı yoktur, değerler yalnız kullanıcı beyanına dayanır." },
    }))
}

/// GET /api/belge/alim/:id — oturumun güncel durumu.
pub async fn alim_durum(Path(id): Path<String>) -> Json<serde_json::Value> {
    let kilit = oturumlar().lock().unwrap();
    match kilit.get(&id) {
        Some(o) => {
            let (bulgular, tutan, toplam) = denklem_kontrol(o);
            Json(serde_json::json!({
                "oturum": o, "aktif_alan": sonraki_alan(o),
                "denklem_tutan": tutan, "denklem_toplam": toplam, "bulgular": bulgular }))
        }
        None => Json(serde_json::json!({ "hata": "oturum bulunamadı" })),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// FATURA → FİŞ — belgeden muhasebe kaydına
// ─────────────────────────────────────────────────────────────────────────────
// Bu, sistemin asıl amacıdır: belge, kaydın DAYANAĞIDIR (VUK 227). Alım oturumu
// tamamlanmadan fiş üretilmez; üretilen fiş de doğrudan deftere yazılmaz — kullanıcı
// görür, `/api/fis` ile kesinleştirir. Yetki ve onay eşiği orada zaten uygulanıyor;
// burada tekrarlamak iki ayrı doğruluk kaynağı yaratırdı.

/// POST /api/belge/alim/fis-taslagi — tamamlanmış FATURA oturumundan yevmiye fişi taslağı.
///
/// Kayıt yönü belgeye göre kurulur (`docs/learnings/iptal-iade-muhasebesi.md`):
/// · **SATIŞ:** 120 Alıcılar borç (toplam) / 600 Yurtiçi Satışlar alacak (matrah) / 391 KDV alacak
/// · **ALIŞ:** 153 Ticari Mallar borç (matrah) / 191 İndirilecek KDV borç / 320 Satıcılar alacak
///
/// KDV oranı alan olarak sorulmaz, **matrahtan türetilir** — sorulsaydı kullanıcı yanlış
/// girebilir ve tutarla çelişebilirdi. Türetilen oran katalogda yoksa muavin kodu boş kalır
/// ve uyarı verilir; motor oran UYDURMAZ (tevkifat/ÖTV oranlarındaki ilkeyle aynı).
pub async fn alim_fis_taslagi(Json(g): Json<OnayGirdi>) -> Json<serde_json::Value> {
    let kilit = oturumlar().lock().unwrap();
    let Some(o) = kilit.get(&g.oturum_id) else {
        return Json(serde_json::json!({ "hata": "oturum bulunamadı" }));
    };
    if o.belge_turu != "FATURA" {
        return Json(serde_json::json!({
            "hata": format!("fiş taslağı şu an yalnız FATURA için üretiliyor (bu oturum: {})", o.belge_turu) }));
    }
    // KALEMLER — mal cinsi / miktar / birim fiyat / satır tutarı.
    // Fişi değiştirmezler (matraha toplanırlar) ama stok, KDV oran kırılımı ve denetim
    // örneklemesi satır düzeyinde çalışır; ayrıca Σtutar = matrah bağımsız bir kanıttır.
    let kalem_sonuc = {
        let satirlar: Vec<&str> = o.metin.lines().collect();
        let matrah = o.alanlar.iter().find(|a| a.kod == "matrah").and_then(|a| a.deger);
        crate::belge_kalem::kalemleri_bul(&satirlar, &o.dil, matrah)
    };

    let (denklem, tutan, toplam_d) = denklem_kontrol(o);
    let eksik: Vec<String> = o.alanlar.iter()
        .filter(|a| a.zorunlu && (a.ham.trim().is_empty() || !a.onaylandi))
        .map(|a| a.ad.clone()).collect();
    if !eksik.is_empty() || tutan != toplam_d {
        return Json(serde_json::json!({
            "hata": "fiş üretilemez — oturum tamamlanmadı",
            "eksik_alanlar": eksik, "denklem_tutan": tutan, "denklem_toplam": toplam_d,
            "bulgular": denklem,
        }));
    }

    let al = |kod: &str| o.alanlar.iter().find(|a| a.kod == kod);
    let sayi = |kod: &str| al(kod).and_then(|a| a.deger).unwrap_or(0);
    let metin = |kod: &str| al(kod).map(|a| a.ham.clone()).unwrap_or_default();

    let (matrah, kdv, toplam) = (sayi("matrah"), sayi("kdv"), sayi("toplam"));
    // TEVKİFAT (KDVK 9): KDV'nin bir kısmını alıcı sorumlu sıfatıyla vergi dairesine öder,
    // SATICIYA ÖDEMEZ. Bu yüzden cariye yazılan tutar `toplam − tevkifat`, satıcıda
    // hesaplanan KDV ise yalnız tevkifat DIŞI paydır. Karıştırılırsa hem cari bakiye hem
    // KDV beyanı yanlış olur. (docs/learnings/iptal-iade-muhasebesi.md §6)
    let tevkifat = sayi("tevkifat").max(0);
    let satis = metin("yon").to_uppercase().starts_with("SATIS")
        || metin("yon").to_uppercase().starts_with("SATIŞ");

    let mut uyarilar: Vec<AlimBulgu> = Vec::new();
    // Oranı matrahtan türet ve katalogla doğrula.
    let oran = if matrah > 0 { (kdv as f64 * 100.0 / matrah as f64).round() as i64 } else { 0 };
    let muavin = crate::hesaplama::muavin_kodu(oran);
    if kdv > 0 && muavin.is_empty() {
        uyarilar.push(bulgu("A30", "UYARI",
            format!("Türetilen KDV oranı %{oran} katalogda yok — muavin kodu belirlenemedi."),
            "Tutarları kontrol et. Oran gerçekten farklıysa KDV muavinini elle seç; \
             motor katalogda olmayan oran için kod uydurmaz."));
    }
    let kdv_hesap = |ana: &str| if muavin.is_empty() { ana.to_string() } else { format!("{ana}.{muavin}") };

    let sat = |hesap: String, borc: i64, alacak: i64, ack: &str| serde_json::json!({
        "hesap": hesap, "borc": borc, "alacak": alacak, "aciklama": ack, "serbest": false });

    if tevkifat > kdv {
        uyarilar.push(bulgu("A32", "ENGEL",
            format!("Tevkifat ({}) hesaplanan KDV'den ({}) büyük olamaz.", tl(tevkifat), tl(kdv)),
            "Tevkifat, KDV'nin bir PAYIDIR. Belgedeki tevkifat tutarını kontrol et."));
    }
    // Cariye yazılacak tutar: tevkif edilen KDV satıcıya ödenmez.
    let cari_tutar = (toplam - tevkifat).max(0);
    let satici_kdv = (kdv - tevkifat).max(0);

    let karsi = metin("unvan");
    let mut satirlar: Vec<serde_json::Value> = Vec::new();
    if satis {
        satirlar.push(sat("120".into(), cari_tutar, 0, &karsi));
        satirlar.push(sat("600".into(), 0, matrah, "Yurtiçi satış"));
        if satici_kdv > 0 {
            satirlar.push(sat(kdv_hesap("391"), 0, satici_kdv,
                &format!("Hesaplanan KDV %{oran}{}", if tevkifat > 0 { " (tevkifat dışı pay)" } else { "" })));
        }
    } else {
        satirlar.push(sat("153".into(), matrah, 0, "Ticari mal alışı"));
        if satici_kdv > 0 { satirlar.push(sat(kdv_hesap("191"), satici_kdv, 0, &format!("İndirilecek KDV %{oran}"))); }
        // ALICI tarafında tevkifat: sorumlu sıfatıyla beyan edilecek KDV borcu doğar (360).
        // 2 No.lu KDV beyannamesiyle ödenir; satıcıya ödenmez.
        if tevkifat > 0 {
            satirlar.push(sat(kdv_hesap("191"), tevkifat, 0, "Sorumlu sıfatıyla indirilecek KDV"));
            satirlar.push(sat("360".into(), 0, tevkifat, "Sorumlu sıfatıyla ödenecek KDV (2 No.lu beyanname)"));
        }
        satirlar.push(sat("320".into(), 0, cari_tutar, &karsi));
    }

    // Fiş kendi içinde denk olmalı — denklem zaten tuttuğu için burada bozulamaz,
    // ama sessiz varsayım yerine ölçülür (defterdeki her fiş için geçerli kural).
    let b: i64 = satirlar.iter().filter_map(|s| s["borc"].as_i64()).sum();
    let a: i64 = satirlar.iter().filter_map(|s| s["alacak"].as_i64()).sum();
    if b != a {
        uyarilar.push(bulgu("A31", "ENGEL",
            format!("Üretilen fiş dengesiz: borç {} ≠ alacak {}.", tl(b), tl(a)),
            "Bu bir motor hatasıdır; tutarları kontrol et ve bildir."));
    }

    // Tarih yyyyaagg olarak saklı; fiş ucu gg.aa.yyyy bekliyor.
    let t = sayi("tarih").to_string();
    let tarih = if t.len() == 8 { format!("{}.{}.{}", &t[6..8], &t[4..6], &t[0..4]) } else { String::new() };

    // ── ENGEL VARSA FİŞ ÜRETİLMEZ ─────────────────────────────────────────────
    //
    // Eskiden A31 (fiş dengesiz) ve A32 (tevkifat > KDV) ENGEL üretiyor ama fonksiyon
    // fişi YİNE DE döndürüyordu — üstelik "Taslak. Deftere yazmak için /api/fis ucuna
    // gönderilir" notuyla. `bulgular` dizisini süzmeyen bir istemci dengesiz fişi
    // doğrudan deftere yollayabiliyordu.
    //
    // Motor "hayır" diyorsa çıktısını da geri çekmeli: ENGEL varken `fis` alanı hiç
    // üretilmez. Bulgular yine döner ki kullanıcı neyin yanlış olduğunu görsün.
    if uyarilar.iter().any(|x| x.onem == "ENGEL") {
        return Json(serde_json::json!({
            "hata": "Fiş üretilemedi — ENGEL bulgusu var.",
            "fis": serde_json::Value::Null,
            "denge": { "borc": b, "alacak": a, "denk": b == a },
            "bulgular": uyarilar,
            "not": "ENGEL giderilmeden bu belgeden fiş üretilemez; taslak bilerek \
                    döndürülmemiştir (VUK 219 — hatalı kayıt deftere girmemelidir).",
        }));
    }

    Json(serde_json::json!({
        "fis": {
            "tip": "Mahsup",
            "tarih": tarih,
            "aciklama": format!("{} faturası — {} ({})", if satis { "Satış" } else { "Alış" }, karsi, metin("belge_no")),
            "satirlar": satirlar,
            "dayanak_tur": "Fatura",
            "dayanak_no": metin("belge_no"),
        },
        "ozet": { "yon": if satis { "SATIS" } else { "ALIS" }, "matrah": matrah, "kdv": kdv,
                  "toplam": toplam, "kdv_orani": oran, "kdv_muavin": muavin,
                  "tevkifat": tevkifat, "cari_tutar": cari_tutar },
        "denge": { "borc": b, "alacak": a, "denk": b == a },
        "kalemler": kalem_sonuc,
        "bulgular": uyarilar,
        "not": "Taslak. Deftere yazmak için /api/fis ucuna gönderilir; yetki ve onay eşiği orada uygulanır.",
    }))
}

/// GET /api/belge/alim/sablonlar — arayüzün belge türü + parametre listesini alacağı uç.
pub async fn alim_sablonlar() -> Json<serde_json::Value> {
    Json(parametreler().clone())
}

#[cfg(test)]
mod testler;
