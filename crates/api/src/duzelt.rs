//! TERİM DÜZELTME — bozuk okunan finans terimini en yakın gerçek terime bağlar.
//!
//! ## Neden yalnız TERİM düzeltilir
//!
//! Altı gerçek el yazısı belge okundu ve sapma hep aynı yerde çıktı:
//!
//! | Ne | Sonuç |
//! |---|---|
//! | Sayılar (aritmetiği kapanan belgelerde) | 1889 faturası 14/14 · 1828 eczane hesabı 10/10 — **tam** |
//! | Alan/terim sözcükleri | güvenilir; kapalı bir kümeden gelirler |
//! | **Özel isimler** | belirsiz — "Ann Forbes", "Réty", "Reichmann" |
//!
//! Terim düzeltilebilir çünkü **sonlu bir kümeden** gelir: bir faturada "Matrah" yazar,
//! "Matrih" diye bir muhasebe terimi yoktur. Özel isim öyle değildir — dünyadaki her isim
//! olabilir, dolayısıyla "en yakın isim"e çekmek düzeltme değil TAHRİFTİR. Firma ünvanını
//! "düzeltmek" yanlış cariye kayıt demektir.
//!
//! ## Sayıya ASLA dokunulmaz
//!
//! Bu modülün en sert kuralı. "41.50"yi "47.50"ye çeken bir düzeltici, muhasebe hatasını
//! düzeltmez — **üretir**. Sayıların doğruluğu sözlükten değil, belgenin kendi
//! aritmetiğinden gelir (bkz. `belge_oku`: kalem toplamı ↔ belge tutarı, rakam ↔ yazı).
//!
//! ## Sessizce değiştirmez
//!
//! Motor öneri üretir, uzaklığı ve gerekçesiyle. Eşit yakınlıkta iki aday varsa **hiç öneri
//! vermez** — belirsizken tahmin yürütmek, yanlış düzeltmeden beterdir çünkü kullanıcı
//! kontrol etme gereği duymaz.

use axum::{extract::Query, Json};
use serde::Serialize;
use std::collections::HashMap;

const FINANS: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/finans-sozlugu.json"));

/// Sözlük BİR KEZ ayrıştırılır. Eskiden her `sozluk()` çağrısı JSON'u baştan parse ediyordu
/// ve `karisir()` bunu düzenleme-uzaklığı DP döngüsünün İÇİNDEN çağırıyordu: her karakter
/// karşılaştırması için tam bir JSON ayrıştırması. Ölçüm koşusu 10 dakikada bitmeyince
/// fark edildi — tek bir düzeltme çağrısı bile gereksiz yere yüzlerce kat pahalıydı.
static SOZLUK_ONBELLEK: std::sync::OnceLock<serde_json::Value> = std::sync::OnceLock::new();
fn sozluk() -> &'static serde_json::Value {
    SOZLUK_ONBELLEK.get_or_init(|| serde_json::from_str(FINANS).unwrap_or(serde_json::Value::Null))
}

/// Karışma çiftleri tablosu — (a,b) → bedel. DP döngüsünde milyonlarca kez sorgulanır,
/// bu yüzden JSON'dan bir kez çıkarılıp düz bir haritaya alınır.
static KARISMA: std::sync::OnceLock<HashMap<(char, char), f32>> = std::sync::OnceLock::new();
fn karisma_tablosu() -> &'static HashMap<(char, char), f32> {
    KARISMA.get_or_init(|| {
        let s = sozluk();
        let mut m = HashMap::new();
        // Aksan farkı BEDELSİZ; görsel karışma ucuz (0.4); gerisi tam bedel (1.0).
        for (dizi, bedel) in [("turkce_aksan", 0.0f32), ("gorsel", 0.4)] {
            for p in s["_karistirma_ciftleri"][dizi].as_array().into_iter().flatten() {
                let (Some(x), Some(y)) = (p[0].as_str(), p[1].as_str()) else { continue };
                let (Some(x), Some(y)) = (x.chars().next(), y.chars().next()) else { continue };
                // Aksan girdisi önce yazıldığı için `or_insert` onu korur — daha ucuz olan kazanır.
                m.entry((x, y)).or_insert(bedel);
                m.entry((y, x)).or_insert(bedel);
            }
        }
        m
    })
}

/// Karşılaştırma biçimi: küçük harf, aksansız, noktalama yok.
/// `belge_oku::sadelestir` ile aynı kuralı kullanır — iki modül aynı sözcüğü aynı görmeli.
fn sade(s: &str) -> String { crate::belge_oku::sadelestir(s) }

/// İki karakter el yazısında birbirine karışıyor mu? Karışıyorsa değiştirme UCUZDUR.
///
/// **Bu tablonun kaynağı hakkında dürüst not:** karışma çiftleri okunan belgelerden ÖLÇÜLMEDİ,
/// genel bilgiden türetildi. (Önceki sürümde "gözlemden geliyor" yazıyordu — yanlıştı, düzeltildi.)
/// Ölçülmüş olan şey tablonun kendisi değil, tablonun ÜRETTİĞİ sonuç: `olcum.rs` binlerce
/// bozulma üretip kurtarma ve yanlış düzeltme oranlarını sayıyor. Tablo oradan doğrulanıyor.
///
/// Türkçe aksan (ı/i, ş/s, ğ/g, ü/u, ö/o, ç/c) elle doldurulan belgede zaten atlanır —
/// "Odenecek" ile "Ödenecek" aynı sözcüktür, hiç ceza almamalı.
fn karisir(a: char, b: char) -> Option<f32> {
    karisma_tablosu().get(&(a, b)).copied()
}

/// Türkçe çekim ekleri (aksan sadeleştirmesinden SONRAKİ yüzey biçimleriyle: ı→i, ü→u, ö→o).
/// Uzundan kısaya sıralı — soyma en uzun ekten başlamalı, yoksa "larin" için önce "in" soyulur.
///
/// Belge etiketlerinde baskın olan **3. tekil iyelik** ekidir: Türkçe ad tamlaması ikinci
/// sözcüğü çeker — "Fatura Tarihi", "KDV Matrahı", "Genel Toplamı", "Ödenecek Tutarı".
const TR_EKLER: &[&str] = &[
    "sinden", "sindan", "larindan", "lerinden",
    "sinin", "larin", "lerin", "sinde", "sina", "sine", "sini",
    "lari", "leri", "nin", "nun", "den", "dan", "ten", "tan",
    "lar", "ler", "de", "da", "te", "ta", "si", "su",
    "in", "un", "ni", "nu", "ye", "ya", "yi", "yu",
    "i", "u", "e", "a",
];

/// TERİMİN sonuna ek gelmiş mi? Gelmişse ek soyulmuş biçim de aday sayılır.
///
/// Neden yalnız SOYMA, kök bulma değil: gerçek bir biçimbirim çözümleyicisi yazmak bu işin
/// kapsamı dışında ve gereksiz — muhasebe belgesi **dar bir alandır**, terim kümesi kapalıdır.
/// Araştırma da bunu söylüyor: sözlük yaklaşımı Türkçede genelde yetersizdir ama çek/fatura
/// gibi dar alanlarda zorunludur. Bize gereken, kapalı kümeye ek toleransı eklemek.
///
/// **Aşırı soymaya karşı koruma:** kök en az 3 harf kalmalı. Yoksa "Kasa"dan "a" soyulup
/// "Kas" olur ve alakasız eşleşmeler doğar.
fn tr_ek_soy(kelime: &str) -> Vec<String> {
    // TÜM uygulanabilir ekler denenir, yalnız en uzunu değil. Açgözlü soyma yanlıştı:
    // "alicilari" için önce "lari" soyulup "alici" bulunuyordu; oysa doğru kök, yalnız
    // "i" soyularak elde edilen "alicilar" (= Alıcılar) idi. Uzun ek her zaman doğru ek değildir.
    let mut adaylar = vec![kelime.to_string()];
    let mut katman = vec![kelime.to_string()];
    // En fazla iki katman: "tutar-lar-i" gibi. Daha fazlası etiketlerde görülmüyor.
    for _ in 0..2 {
        let mut sonraki = Vec::new();
        for k in &katman {
            for ek in TR_EKLER {
                if let Some(kok) = k.strip_suffix(ek) {
                    // Aşırı soymaya karşı: kök en az 3 harf kalmalı ("Kasa" → "Kas" olmasın).
                    if kok.chars().count() >= 3 && !adaylar.iter().any(|a| a == kok) {
                        adaylar.push(kok.to_string());
                        sonraki.push(kok.to_string());
                    }
                }
            }
        }
        if sonraki.is_empty() { break; }
        katman = sonraki;
    }
    adaylar
}

/// Türkçe için ek-duyarlı uzaklık: okunan sözcüğün SON sözcüğünden ek soyulup en iyi eşleşme
/// alınır. Ad tamlamasında ek son sözcüğe gelir ("Fatura Tarihi" → "Tarihi"), bu yüzden yalnız
/// son sözcük soyulur; baştakini soymak "Faturalar Tarihi" gibi biçimleri de bozardı.
fn uzaklik_tr(okunan: &str, terim: &str) -> f32 {
    let mut en_iyi = uzaklik(okunan, terim);
    let parcalar: Vec<&str> = okunan.split(' ').collect();
    let Some((son, bas)) = parcalar.split_last() else { return en_iyi };
    let son_uz = son.chars().count();
    for kok in tr_ek_soy(son) {
        let birlesik = if bas.is_empty() { kok.clone() }
                       else { format!("{} {}", bas.join(" "), kok) };
        // AZ SOYMA DAHA SADIKTIR. Soyulan her harf küçük bir bedel taşır; yoksa "Alıcıları"
        // hem "Alıcılar"a (1 harf soyulmuş) hem "Alıcı"ya (4 harf) sıfır uzaklıkta oturur ve
        // motor sözlük sırasına göre keyfî birini seçer. Bedel, belgeye daha yakın olanı kazandırır.
        let soyulan = son_uz.saturating_sub(kok.chars().count()) as f32;
        en_iyi = en_iyi.min(uzaklik(&birlesik, terim) + soyulan * 0.05);
    }
    en_iyi
}

/// Karışma-duyarlı düzenleme uzaklığı (Levenshtein + yer değiştirme).
///
/// Düz Levenshtein burada yanlış sonuç verir: "Tutar"→"Tutur" ile "Kasa"→"Kesa" ikisi de
/// 1 uzaklıktadır, oysa ilki el yazısında olağan, ikincisi değil. Karışan çiftleri ucuzlatınca
/// gerçekten olası okuma hataları öne çıkar, uzak eşleşmeler elenir.
pub fn uzaklik(a: &str, b: &str) -> f32 {
    let x: Vec<char> = a.chars().collect();
    let y: Vec<char> = b.chars().collect();
    let (n, m) = (x.len(), y.len());
    if n == 0 { return m as f32; }
    if m == 0 { return n as f32; }

    let mut d = vec![vec![0f32; m + 1]; n + 1];
    for i in 0..=n { d[i][0] = i as f32; }
    for j in 0..=m { d[0][j] = j as f32; }
    for i in 1..=n {
        for j in 1..=m {
            let bedel = if x[i - 1] == y[j - 1] { 0.0 }
                        else { karisir(x[i - 1], y[j - 1]).unwrap_or(1.0) };
            d[i][j] = (d[i - 1][j] + 1.0)
                .min(d[i][j - 1] + 1.0)
                .min(d[i - 1][j - 1] + bedel);
            // Bitişik harf yer değiştirmesi (Damerau) — "Matrha" → "Matrah"
            if i > 1 && j > 1 && x[i - 1] == y[j - 2] && x[i - 2] == y[j - 1] {
                d[i][j] = d[i][j].min(d[i - 2][j - 2] + 1.0);
            }
        }
    }
    d[n][m]
}

#[derive(Serialize, Clone, Debug)]
pub struct Oneri {
    pub okunan: String,
    pub oneri: String,
    pub uzaklik: f32,
    /// 0–100: uzaklığın sözcük uzunluğuna oranından türer.
    pub benzerlik: u8,
    /// Terim bir hesaba karşılık geliyorsa TDHP kodu (ör. "Hesaplanan KDV" → 391).
    pub hesap: Option<String>,
    pub gerekce: String,
}

#[derive(Serialize)]
pub struct DuzeltmeSonucu {
    pub okunan: String,
    /// Tam eşleşme varsa terim buradadır; düzeltmeye gerek yoktur.
    pub tam_eslesme: Option<String>,
    /// Tek ve yeterince yakın aday varsa öneri.
    pub oneri: Option<Oneri>,
    /// Eşit yakınlıkta birden çok aday — SEÇİM YAPILMAZ, kullanıcıya sorulur.
    pub belirsiz: Vec<String>,
    pub durum: String,
    pub aciklama: String,
}

/// Sözcük sayı mı? Sayılara ASLA düzeltme uygulanmaz.
fn sayisal(s: &str) -> bool {
    let r = s.chars().filter(|c| c.is_ascii_digit()).count();
    r > 0 && r * 2 >= s.chars().filter(|c| !c.is_whitespace()).count()
}

/// Bir okumayı sözlükteki terimlerle karşılaştırır.
///
/// `baglam`: belge türü (FATURA/MAKBUZ/…). Verilirse yalnız o türde geçen terimler aday olur —
/// dekontta "Tevkifat" aranmaz. Boşsa tüm terimler aday.
pub fn duzelt(okunan: &str, dil: &str, baglam: &str) -> DuzeltmeSonucu {
    duzelt_ayarli(okunan, dil, baglam, ESIK_CARPANI, MARJ)
}

/// EŞİK — kabul edilecek en büyük uzaklık, sözcük uzunluğunun katı olarak.
/// MARJ — en iyi adayın ikinciden en az bu kadar İYİ olması gerekir; değilse belirsiz sayılır.
///
/// İkisi de `olcum.rs` içindeki tarama ile seçildi, hisle değil: eşik/marj ızgarası taranıp
/// **yanlış düzeltme oranını düşük tutan** en yüksek kurtarma noktası alındı. Muhasebede
/// yanlış düzeltme, düzeltmemekten pahalıdır — kullanıcı kontrol etme gereği duymaz.
pub const ESIK_CARPANI: f32 = 0.34;
pub const MARJ: f32 = 0.30;

/// Parametreli sürüm — ayar taraması bunu çağırır.
pub fn duzelt_ayarli(okunan: &str, dil: &str, baglam: &str, esik_carpani: f32, marj: f32) -> DuzeltmeSonucu {
    let bos = |durum: &str, aciklama: &str| DuzeltmeSonucu {
        okunan: okunan.to_string(), tam_eslesme: None, oneri: None, belirsiz: Vec::new(),
        durum: durum.into(), aciklama: aciklama.into(),
    };

    if okunan.trim().is_empty() { return bos("BOS", "Okunacak metin yok."); }

    // KURAL 1 — SAYIYA DOKUNULMAZ. Sayının doğruluğu sözlükten değil aritmetikten gelir.
    if sayisal(okunan) {
        return bos("SAYI",
            "Sayılar düzeltilmez. Bir rakamı 'en yakın rakama' çekmek hatayı düzeltmez, ÜRETİR. \
             Sayının doğruluğu sözlükten değil aritmetikten gelir: kalem toplamını belge \
             tutarıyla, rakamı yazıyla karşılaştır (POST /api/belge/coz).");
    }

    let s = sozluk();
    let sade_okunan = sade(okunan);
    let mut adaylar: Vec<(f32, String, Option<String>)> = Vec::new();

    for t in s["terimler"][dil].as_array().into_iter().flatten() {
        let Some(terim) = t["t"].as_str() else { continue };
        // Bağlam süzgeci — dekontta fatura terimi önerilmesin.
        if !baglam.is_empty() {
            let uygun = t["ba"].as_array().into_iter().flatten()
                .any(|b| b.as_str() == Some(baglam));
            if !uygun { continue; }
        }
        let sade_terim = sade(terim);
        // Türkçede ek-duyarlı ölçü kullanılır (belge "Tutar" değil "Tutarı" yazar).
        let u = if sade_terim == sade_okunan { 0.0 }
                else if dil == "tr" { uzaklik_tr(&sade_okunan, &sade_terim) }
                else { uzaklik(&sade_okunan, &sade_terim) };
        adaylar.push((u, terim.to_string(), t["hesap"].as_str().map(String::from)));
    }

    if adaylar.is_empty() {
        return bos("SOZLUK_YOK", "Bu dil/bağlam için sözlükte terim yok.");
    }
    adaylar.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    // TAM EŞLEŞME kararı, TÜM adaylar toplandıktan SONRA verilir.
    // Eskiden ilk yeterince yakın terimde erken dönülüyordu; sözlük sırası kazanıyordu.
    // "Alıcıları" hem "Alıcı"ya hem "Alıcılar"a oturur — listede önce geleni değil,
    // GERÇEKTEN EN YAKIN olanı seçmek gerekir.
    // TAM yalnız fark BİÇİMBİRİMSEL/AKSANSAL ise verilir — harf değişimi varsa bu bir
    // DÜZELTMEdir ve öneri olarak sunulmalıdır. (Eşik olarak "0.5'ten küçük uzaklık" demek
    // yanlıştı: ucuz karışmalar — 'Tevklfat' → 'Tevkifat', bedel 0.4 — sessizce "sözlükte
    // birebir var" sayılıyordu. Kullanıcı düzeltildiğini görmeden onaylardı.)
    let bicimsel_ayni = |terim: &str| -> bool {
        let st = sade(terim);
        if st == sade_okunan { return true; }
        if dil != "tr" { return false; }
        let p: Vec<&str> = sade_okunan.split(' ').collect();
        let Some((son, bas)) = p.split_last() else { return false };
        tr_ek_soy(son).iter().any(|kok| {
            let b = if bas.is_empty() { kok.clone() } else { format!("{} {}", bas.join(" "), kok) };
            b == st
        })
    };
    if bicimsel_ayni(&adaylar[0].1) {
        return DuzeltmeSonucu {
            okunan: okunan.to_string(),
            tam_eslesme: Some(adaylar[0].1.clone()),
            oneri: None, belirsiz: Vec::new(),
            durum: "TAM".into(),
            aciklama: "Terim sözlükte var (Türkçede ek almış biçimiyle olabilir); düzeltme gerekmiyor.".into(),
        };
    }

    // KURAL 2 — EŞİK. Uzaklık, sözcüğün kendi uzunluğuna göre değerlendirilir: 4 harflik
    // sözcükte 2 hata sözcüğü değiştirir, 20 harflikte değiştirmez.
    let uzunluk = sade_okunan.chars().count().max(1) as f32;
    let esik = (uzunluk * esik_carpani).max(1.0);
    let (en_iyi, terim, hesap) = adaylar[0].clone();
    if en_iyi > esik {
        return bos("BILINMIYOR",
            "Sözlükte yeterince yakın terim yok. Bu bir ÖZEL İSİM (firma, kişi, yer) olabilir — \
             özel isimler düzeltilmez, çünkü kapalı bir kümeden gelmezler.");
    }

    // KURAL 2B — KORUNAN SÖZCÜK. Terim olmayan ama belgede MEŞRU bulunan sözcükler
    // (fatura satır kalemleri: nakliye, danışmanlık, temizlik…) terime çekilmemeli.
    // Okuma bir korunan sözcüğe terimden daha yakın YA DA eşit uzaklıktaysa çekimser kalınır.
    //
    // Ölçüm bunu gerekli kıldı: 'nakliye' → 'Bakiye' ve 'masa' → 'Kasa' öneriliyordu.
    // İkisi de gerçek fatura sözcüğüdür; "düzeltilmeleri" veri bozulmasıdır.
    let en_yakin_korunan = sozluk()["korunan"][dil].as_array().into_iter().flatten()
        .filter_map(|k| k.as_str())
        .map(|k| if dil == "tr" { uzaklik_tr(&sade_okunan, &sade(k)) } else { uzaklik(&sade_okunan, &sade(k)) })
        .fold(f32::MAX, f32::min);
    if en_yakin_korunan <= en_iyi {
        return bos("KORUNAN",
            "Okuma, bir finans terimine olduğu kadar (veya daha çok) MEŞRU bir belge sözcüğüne \
             de benziyor — fatura satır kalemleri (nakliye, danışmanlık…) terime çekilmez. \
             Düzeltme yapılmadı.");
    }

    // KURAL 3 — BELİRSİZLİK. Yalnız TAM eşitlik aramak yetmiyordu: 0.4 ile 0.5 uzaklıktaki
    // iki aday da fiilen ayırt edilemez, ama eski kural birincisini seçiyordu. Artık en iyi
    // aday, ikinciden en az MARJ kadar iyi olmalı — değilse seçim yapılmaz.
    let esitler: Vec<String> = adaylar.iter()
        .filter(|(u, _, _)| *u - en_iyi < marj)
        .map(|(_, t, _)| t.clone()).collect();
    if esitler.len() > 1 {
        return DuzeltmeSonucu {
            okunan: okunan.to_string(), tam_eslesme: None, oneri: None,
            belirsiz: esitler,
            durum: "BELIRSIZ".into(),
            aciklama: "Aynı yakınlıkta birden çok terim var. Seçim yapılmadı — \
                       belirsizken tahmin yürütmek, yanlış düzeltmeden beterdir.".into(),
        };
    }

    let benzerlik = (100.0 * (1.0 - en_iyi / uzunluk)).clamp(0.0, 100.0) as u8;
    DuzeltmeSonucu {
        okunan: okunan.to_string(), tam_eslesme: None, belirsiz: Vec::new(),
        oneri: Some(Oneri {
            okunan: okunan.to_string(),
            oneri: terim.clone(),
            uzaklik: (en_iyi * 100.0).round() / 100.0,
            benzerlik,
            hesap: hesap.clone(),
            gerekce: match &hesap {
                Some(h) => format!("'{terim}' finans terimi (TDHP {h}); okunan biçim {en_iyi:.2} uzaklıkta."),
                None => format!("'{terim}' finans terimi; okunan biçim {en_iyi:.2} uzaklıkta."),
            },
        }),
        durum: "ONERI".into(),
        aciklama: "Öneri üretildi. Otomatik uygulanmaz — onaylayan muhasebecidir.".into(),
    }
}

/// GET /api/belge/duzelt?metin=Matrih&dil=tr&baglam=FATURA
pub async fn terim_duzelt(Query(q): Query<HashMap<String, String>>) -> Json<DuzeltmeSonucu> {
    Json(duzelt(
        q.get("metin").map(String::as_str).unwrap_or(""),
        q.get("dil").map(String::as_str).unwrap_or("tr"),
        q.get("baglam").map(String::as_str).unwrap_or(""),
    ))
}

/// GET /api/belge/terimler?dil=tr&baglam=FATURA — sözlüğün kendisi (arayüzde tamamlama için).
pub async fn terim_listesi(Query(q): Query<HashMap<String, String>>) -> Json<serde_json::Value> {
    let dil = q.get("dil").map(String::as_str).unwrap_or("tr");
    let baglam = q.get("baglam").map(String::as_str).unwrap_or("");
    let s = sozluk();
    let liste: Vec<serde_json::Value> = s["terimler"][dil].as_array().into_iter().flatten()
        .filter(|t| baglam.is_empty()
            || t["ba"].as_array().into_iter().flatten().any(|b| b.as_str() == Some(baglam)))
        .cloned().collect();
    Json(serde_json::json!({
        "dil": dil, "baglam": baglam, "adet": liste.len(), "terimler": liste,
        "not": "Sayılar bu sözlükle düzeltilmez; doğrulukları belgenin kendi aritmetiğinden gelir.",
    }))
}

#[cfg(test)]
mod testler;
#[cfg(test)]
mod olcum;   // İstatistiksel ölçüm — kurtarma, yanlış düzeltme, uydurma oranları + ayar taraması.
