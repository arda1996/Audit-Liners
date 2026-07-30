//! BELGE OKUMA MOTORU — okunan metni SAYIYA çevirir ve belgenin kendi içinde DOĞRULAR.
//!
//! ## İş bölümü — neden bu modül var
//!
//! Görüntüden metin çıkarmayı **model** yapar (görme yeteneği). Bu modül o metni alır ve
//! *deterministik* kısmı yapar: sayıya çevirme, tarih çözme, çapraz doğrulama, fişe eşleme.
//! Ayrım şundan: model "0" ile "O", "1" ile "7", "3" ile "8" karıştırabilir ve bunu
//! **kendinden emin** biçimde yapar. Bir okumanın doğruluğunu OCR güven skoru değil,
//! **belgenin kendi aritmetiği** kanıtlar.
//!
//! ## Güven, skordan değil TUTARLILIKTAN gelir
//!
//! Gerçek belgelerle yapılan sınamada işe yarayan tek ölçüt buydu:
//!   · 1889 tarihli el yazısı faturada 14 kalem okundu → alt toplamlar ve devir tutarı
//!     kuruşuna kadar tuttu. Okuma doğruydu, çünkü **tutmasaydı yanlış olduğunu görecektik**.
//!   · 1999 tarihli makbuzda rakam (102.177,36) ile yazı ("сто две тысячи сто семьдесят семь")
//!     birbirini doğruladı. Bu, muhasebecinin elle yaptığı kontrolün ta kendisidir.
//!
//! Bu yüzden motorun asıl işi **B-kodlu bulgular** üretmektir. Doğrulanamayan alan
//! "okundu" sayılmaz; `guven` düşer ve kullanıcıya SORULUR. Motor asla tahmin yürütmez —
//! projedeki tevkifat/ÖTV oranı ilkesiyle aynı: bilinmiyorsa uydurulmaz.
//!
//! ## Dil kapsamı
//! **Çekirdek: TR + EN** (tam kapsam, tam test). **İkinci halka: DE + FR + RU**
//! (alan etiketi + sayı + yazıyla tutar; ülkeye özgü vergi alanları yok).

use axum::Json;
use serde::{Deserialize, Serialize};

const SOZLUK: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/belge-sozlugu.json"));

fn sozluk() -> serde_json::Value { serde_json::from_str(SOZLUK).unwrap_or(serde_json::Value::Null) }

/// BELGE KODU NORMALİZASYONU — fatura no, seri, ETTN gibi harf+rakam kodları için.
///
/// ## Ölçülen sorun
///
/// 22 belgelik temiz (200 dpi) tarama örnekleminde **15'inde fatura numarası kaybediliyordu.**
/// Sebep tek bir karakterdi: OCR `GIB20090000000001` kodundaki büyük `I` harfini küçük `l`
/// okuyor (`GlB2009...`). Aynı hata `ETU...`, `EGE...` gibi tüm kodlarda tekrarlıyor.
///
/// ## Kural — tahmin değil, ÇIKARIM
///
/// Belge kodları büyük harf + rakamdan oluşur. **Böyle bir kodun içinde küçük `l`
/// bulunamaz** — küçük harf orada zaten geçersizdir. Dolayısıyla `l`'yi `I` yapmak bir
/// tahmin değil, geçersiz bir okumayı geçerli tek karşılığına indirgemektir.
///
/// Aynı gerekçeyle: küçük `o` → `O`. Ama `O` ↔ `0` ayrımına **DOKUNULMAZ** — ikisi de o
/// konumda geçerlidir, hangisi olduğu belgeden çıkmaz ve motor bilmediğini uydurmaz.
///
/// ## Ne zaman uygulanır
///
/// Yalnız **kod görünümlü** dizgilerde: en az 5 karakter, hem harf hem rakam içeriyor ve
/// harflerin çoğunluğu zaten büyük. Bu koşullar olmadan normal metne dokunulmaz — yoksa
/// "Elektrik" gibi bir mal adı "EIektrik" olurdu.
///
/// **İz bırakır:** çağıran, değişiklik olup olmadığını dönen dizgiyi karşılaştırarak anlar
/// ve kullanıcıya bildirir. Sessiz düzeltme, denetimde kabul edilemez (VUK 219).
pub fn kod_normalize(ham: &str) -> String {
    let t = ham.trim();
    let n = t.chars().count();
    if n < 5 { return ham.to_string(); }

    let harf = t.chars().filter(|c| c.is_alphabetic()).count();
    let rakam = t.chars().filter(|c| c.is_ascii_digit()).count();
    if harf == 0 || rakam == 0 { return ham.to_string(); }
    // Harflerin çoğunluğu büyük olmalı — bu bir KOD, cümle değil.
    let buyuk = t.chars().filter(|c| c.is_uppercase()).count();
    if buyuk * 2 < harf { return ham.to_string(); }
    // Boşluk içeren dizgi kod değildir.
    if t.chars().any(|c| c.is_whitespace()) { return ham.to_string(); }

    t.chars().map(|c| match c {
        // Küçük harf bu bağlamda GEÇERSİZ — tek geçerli karşılığına indirgenir.
        'l' => 'I',
        'o' => 'O',
        // 'O' ↔ '0' ve 'S' ↔ '5' ayrımına dokunulmaz: ikisi de geçerlidir, uydurulmaz.
        _ => c.to_uppercase().next().unwrap_or(c),
    }).collect()
}

/// Karşılaştırma için sadeleştirme: küçük harf + aksan düşürme + noktalama temizliği.
/// Türkçe İ/ı özel: `to_lowercase()` "İ" → "i̇" (birleşik nokta) üretir, eşleşmeyi bozar.
pub fn sadelestir(s: &str) -> String {
    let mut o = String::with_capacity(s.len());
    for c in s.chars() {
        let c = match c {
            // LATİN TANIMA MODELİNİN GETİRDİĞİ YENİ KARIŞMA SINIFI.
            // Latin PP-OCRv5 sözlüğü 502 karakterlik: Türkçe 12/12 kapsıyor ama
            // aksanlı Latin harflerini de üretebiliyor. Ölçüldü: "BİRİM" → "BÍRIM".
            // Í/Ì/Ī Türk alfabesinde YOKTUR; o konumda geçerli tek karşılık İ'dir.
            // Bu bir tahmin değil, geçersiz okumanın tek geçerli karşılığına indirgenmesi.
            'İ' | 'I' | 'Ï' | 'Î' | 'Í' | 'Ì' | 'Ī' | 'Ĭ' | 'Į' => 'i',
            'ı' | 'í' | 'ì' | 'ī' | 'ĭ' | 'į' => 'i',
            'á' | 'à' | 'å' | 'ã' | 'ā' | 'ă' | 'ą' => 'a',
            'Á' | 'À' | 'Å' | 'Ã' | 'Ā' | 'Ă' | 'Ą' => 'a',
            'ó' | 'ò' | 'õ' | 'ō' | 'ŏ' | 'ő' => 'o',
            'Ó' | 'Ò' | 'Õ' | 'Ō' | 'Ŏ' | 'Ő' => 'o',
            'ú' | 'ù' | 'ū' | 'ŭ' | 'ů' | 'ű' => 'u',
            'Ú' | 'Ù' | 'Ū' | 'Ŭ' | 'Ů' | 'Ű' => 'u',
            'ë' | 'ē' | 'ĕ' | 'ė' | 'ę' | 'ě' => 'e',
            'Ë' | 'Ē' | 'Ĕ' | 'Ė' | 'Ę' | 'Ě' => 'e',
            'ñ' | 'ń' | 'ň' => 'n', 'Ñ' | 'Ń' | 'Ň' => 'n',
            'ć' | 'č' | 'ĉ' => 'c', 'Ć' | 'Č' | 'Ĉ' => 'c',
            'ś' | 'š' | 'ŝ' | 'Ś' | 'Š' | 'Ŝ' => 's',
            'ź' | 'ż' | 'ž' => 'z', 'Ź' | 'Ż' | 'Ž' => 'z',
            'ł' => 'l', 'Ł' => 'l', 'đ' => 'd', 'Đ' => 'd',
            'ý' | 'ÿ' => 'y', 'Ý' | 'Ÿ' => 'y',
            'Ş' | 'ş' => 's',
            'Ğ' | 'ğ' | 'ĝ' | 'ġ' | 'ģ' | 'Ĝ' | 'Ġ' | 'Ģ' => 'g',
            'Ü' | 'ü' => 'u',
            'Ö' | 'ö' => 'o', 'Ç' | 'ç' => 'c',
            'Ä' | 'ä' => 'a', 'ß' => 's',
            'É' | 'é' | 'È' | 'è' | 'Ê' | 'ê' => 'e',
            'À' | 'à' | 'Â' | 'â' => 'a',
            'Ô' | 'ô' => 'o', 'Û' | 'û' | 'Ù' | 'ù' => 'u',
            'ё' => 'е',
            _ => c,
        };
        for l in c.to_lowercase() {
            if l.is_alphanumeric() || l == ' ' { o.push(l); }
            else if !o.ends_with(' ') { o.push(' '); }
        }
    }
    o.split_whitespace().collect::<Vec<_>>().join(" ")
}

// ─────────────────────────────────────────────────────────────────────────────
// DİL SEZME
// ─────────────────────────────────────────────────────────────────────────────

/// Bir etiketin belgede geçip geçmediği. Sadeleştirilmiş biçim en az 3 HARF olmalı.
///
/// Uzunluk ham metinde ölçülemez: "№" tek karakterdir ama 3 bayttır, ve sadeleştirmede
/// tamamen kaybolur — `contains("")` her belgede doğru döndüğü için sezici her dile puan
/// veriyordu ve "dil sezilemedi" durumu hiç oluşmuyordu.
fn eslesti(sade_metin: &str, etiket: &str) -> bool {
    let p = sadelestir(etiket);
    p.chars().count() >= 3 && sade_metin.contains(&p)
}

/// Belgedeki etiketlerden dili sezer. Dönen: (dil kodu, kaç etiket eşleşti).
///
/// Eşleşme yoksa **"tr" varsayılmaz** — `("", 0)` döner ve çağıran kullanıcıya sorar.
/// Yanlış dil varsayımı sayı biçimini de yanlış okur (1.234 → TR'de 1234, EN'de 1,234).
pub fn dil_sez(metin: &str) -> (String, usize) {
    let d = sadelestir(metin);
    let s = sozluk();
    let mut en_iyi = (String::new(), 0usize);
    for (kod, _) in s["diller"].as_object().map(|o| o.iter().collect::<Vec<_>>()).unwrap_or_default() {
        let mut puan = 0usize;
        for (_, etiketler) in s["alan_etiketleri"].as_object().into_iter().flatten() {
            for e in etiketler[kod].as_array().into_iter().flatten() {
                if let Some(t) = e.as_str() { if eslesti(&d, t) { puan += 1; } }
            }
        }
        for (_, tur) in s["belge_turleri"].as_object().into_iter().flatten() {
            for a in tur["anahtar"][kod].as_array().into_iter().flatten() {
                if let Some(t) = a.as_str() { if eslesti(&d, t) { puan += 2; } } // tür anahtarı daha ayırt edici
            }
        }
        if puan > en_iyi.1 { en_iyi = (kod.clone(), puan); }
    }
    en_iyi
}

/// Bir alım parametresinin belgede görülebilecek ETİKET EŞANLAMLILARI.
///
/// Alan kodları (`unvan`, `kdv`, `toplam`…) ile sözlükteki anahtarlar (`karsi_taraf`,
/// `vergi_tutari`…) birebir aynı değil; eşleme burada tutulur. Sözlük dil bazlıdır,
/// dolayısıyla İngilizce bir faturada "Bill To", Türkçede "Sayın" aranır.
pub fn alan_esanlamlilari(alan_kodu: &str, dil: &str) -> Vec<String> {
    let anahtar = match alan_kodu {
        "unvan" => "karsi_taraf",
        "vkn" => "vergi_no",
        "belge_no" => "belge_no",
        "tarih" => "tarih",
        "matrah" | "brut" => "matrah",
        "kdv" => "vergi_tutari",
        // Tevkifatın KENDİ etiketleri var; genel KDV etiketlerini paylaşırsa "KDV tutarı"
        // satırını kapar ve fiş yanlış kurulur (cariye eksik borç, satıcıda sıfır KDV).
        "tevkifat" => "tevkifat",
        "toplam" | "net" | "kapanis" => "toplam",
        "yaziyla" => "tutar_yaziyla",
        "iban" => "iban",
        // Ekstre / mutabakat / gider pusulası alanları. Bunlar `belge-parametreleri.json`'da
        // TANIMLIYDI ama etiket karşılıkları yoktu: tarama bu alanları hiç aramıyor, aday
        // listesi boş kalıyor ve `denklemle_sec` denklemi HİÇ DENEMEDEN atlıyordu. Yani
        // DEKONT zinciri ve MUTABAKAT yön denklemi kâğıt üzerinde vardı, fiilen çalışmıyordu.
        // `sozluk_denetim::her_parametre_alaninin_etiketi_var` bu boşluğu artık test ediyor.
        "acilis" => "acilis",
        "giris" => "giris",
        "cikis" => "cikis",
        "stopaj" => "stopaj",
        "borc_bakiye" => "borc_bakiye",
        "alacak_bakiye" => "alacak_bakiye",
        "net_bakiye" => "net_bakiye",
        "iskonto" => "iskonto",
        "otv" => "otv",
        "vade" => "vade",
        "para_birimi" => "para_birimi",
        "vergi_dairesi" => "vergi_dairesi",
        "seri" => "seri",
        "sira" => "sira",
        "irsaliye_no" => "irsaliye_no",
        "sevk_tarihi" => "sevk_tarihi",
        "odeme_sekli" => "odeme_sekli",
        "banka_adi" => "banka_adi",
        "hesap_no" => "hesap_no",
        "ettn" => "ettn",
        _ => return Vec::new(),
    };
    // ── SINIRDA SADELEŞTİRME ─────────────────────────────────────────────────
    //
    // Çağıran taraf etiketi **sadeleştirilmiş** metinde arıyor (`sade.contains(e)`).
    // Sözlük ise doğal yazımda tutuluyor. İkisi eşleşmiyordu ve sonuç sessiz kayıptı:
    // ölçüldüğünde 221 girdinin **48'i (%22) hiç eşleşemiyordu** — "seri sıra no",
    // "düzenleme tarihi", "numéro de facture", "rechnungs-nr", "счет №" gibi Türkçe
    // karakter, aksan veya noktalama içeren her girdi ölüydü.
    //
    // Sadeleştirme burada, TEK YERDE yapılır. Böylece sözlük her dilde **doğal yazımıyla**
    // yazılabilir (ünlü aksanı, ß, kiril, tire, °) ve yazan kişinin kodlama kuralını
    // bilmesi gerekmez. Bütünlük testi (`sozluk_butunlugu`) bu sözleşmeyi korur.
    let mut cikti: Vec<String> = sozluk()["alan_etiketleri"][anahtar][dil].as_array()
        .into_iter().flatten()
        .filter_map(|x| x.as_str().map(sadelestir))
        .filter(|s| s.chars().count() >= 2) // "n°" → "n": tek harf her yere uyar, elenir
        .collect();
    // UZUNDAN KISAYA: "fatura tarihi" ile "tarih" ikisi de uyuyorsa daha ÖZGÜL olan
    // önce gelmeli. Sıraya güvenmeyen çağıran da var (`max_by_key`), ama güvenen için
    // doğru sıra budur.
    cikti.sort_by(|a, b| b.chars().count().cmp(&a.chars().count()).then(a.cmp(b)));
    cikti.dedup();
    cikti
}

/// Belge türünü anahtar kelimelerden çözer. Bulunamazsa boş döner — uydurulmaz.
pub fn belge_turu(metin: &str, dil: &str) -> String {
    let d = sadelestir(metin);
    let s = sozluk();
    let mut en_iyi = (String::new(), 0usize);
    for (tur, veri) in s["belge_turleri"].as_object().into_iter().flatten() {
        for a in veri["anahtar"][dil].as_array().into_iter().flatten() {
            if let Some(t) = a.as_str() {
                let t = sadelestir(t);
                if !t.is_empty() && d.contains(&t) && t.len() > en_iyi.1 {
                    en_iyi = (tur.clone(), t.len()); // en UZUN eşleşme kazanır ("gider pusulası" > "fatura")
                }
            }
        }
    }
    en_iyi.0
}

// ─────────────────────────────────────────────────────────────────────────────
// SAYI OKUMA — yerel biçim
// ─────────────────────────────────────────────────────────────────────────────

/// Metindeki tutarı KURUŞ olarak döndürür. Yerel biçime göre ayraçları çözer.
///
/// Tuzak: "1.234" TR/DE'de **1234**, EN'de **1.234**'tür. Bu yüzden dil bilinmeden
/// sayı okunamaz; dil boşsa güvenli kural uygulanır (son ayraçtan sonra tam 2 hane
/// varsa ondalık, yoksa binlik).
pub fn sayi_ayristir(ham: &str, dil: &str) -> Option<i64> {
    let s = sozluk();
    let (binlik, ondalik) = match s["diller"][dil]["sayi_bicimi"].as_object() {
        Some(b) => (
            b.get("binlik").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            b.get("ondalik").and_then(|x| x.as_str()).unwrap_or("").to_string(),
        ),
        None => (String::new(), String::new()),
    };

    // Rakam ve ayraç dışındaki her şeyi at (para simgesi, harf, boşluk türleri).
    let mut t: String = ham.chars()
        .filter(|c| c.is_ascii_digit() || *c == '.' || *c == ',' || *c == '-'
                    || *c == ' ' || *c == '\u{00A0}' || *c == '\u{202F}')
        .collect();
    let eksi = t.trim_start().starts_with('-');
    t = t.replace(['\u{00A0}', '\u{202F}'], " ");
    if binlik == " " { t = t.replace(' ', ""); } else { t = t.replace(' ', ""); }
    let t = t.trim_matches('-').to_string();
    if t.is_empty() || !t.chars().any(|c| c.is_ascii_digit()) { return None; }

    // Tam kısımdaki HER ayraç atılır. `replace(binlik)` yetmiyordu: elle doldurulmuş
    // belgelerde yazan kişi yerel binlik ayracını kullanmayabiliyor (Rus makbuzunda tutar
    // "102.177,36" yazılmıştı — ondalık virgül yerel, ama binlik nokta değil boşluk olmalıydı).
    let yalniz_rakam = |s: &str| s.chars().filter(|c| c.is_ascii_digit()).collect::<String>();

    // ── ÖLÇEK ÇAPASI — dilden BAĞIMSIZ ────────────────────────────────────────
    //
    // NEDEN VAR: `matrah + KDV = toplam` **DOĞRUSALDIR**. Üç terim de aynı katsayıyla
    // bozulursa denklem HÂLÂ TUTAR. Yani ondalık ayraç yanlış çözülürse motor 100 kat
    // büyük tutarı "belgenin kendi aritmetiğiyle kanıtlandı" diye onaylar ve otomatik
    // doldurur. Ölçek, denklemin sabitleyemediği tek büyüklüktür — burada sabitlenir.
    //
    // KURAL: binlik ayracından sonra **her zaman tam 3 hane** gelir. Bir sayının son
    // grubu 1-2 haneliyse o ayraç binlik OLAMAZ, ondalıktır. Bu bir tahmin değil,
    // basamak gruplamasının tanımı — hangi dil olursa olsun geçerli.
    //
    // İki gerçek hatayı birden kapatır:
    //   "8.000.00"  TR bağlamında  → 800.000 TL yerine 8.000,00 TL   (OCR virgülü nokta okudu)
    //   "8,000.00"  TR bağlamında  → 800.000 TL yerine 8.000,00 TL   (İngiliz biçimi TR belgede)
    let gruplar: Vec<&str> = t.split(['.', ',']).collect();
    let olcek_capasi = gruplar.len() > 1 && {
        let son = gruplar[gruplar.len() - 1];
        (son.len() == 1 || son.len() == 2)
            && son.chars().all(|c| c.is_ascii_digit())
            && gruplar[1..gruplar.len() - 1].iter().all(|g| g.len() == 3)
            && !gruplar[0].is_empty() && gruplar[0].len() <= 3
            && gruplar.iter().all(|g| g.chars().all(|c| c.is_ascii_digit()))
    };

    let (tam, kesir) = if olcek_capasi {
        let i = t.rfind(['.', ','])?;
        (yalniz_rakam(&t[..i]), t[i + 1..].to_string())
    } else if !ondalik.is_empty() && t.contains(&ondalik) {
        // Yerel ondalık ayracının SONUNCUSU gerçek ayraçtır; öncekiler binliktir.
        let i = t.rfind(&ondalik)?;
        let (a, b) = t.split_at(i);
        let b = &b[ondalik.len()..];
        // Aynı ayraç birden çok kez geçiyorsa hiçbiri ondalık DEĞİLDİR (iki ondalık nokta olmaz)
        // — hepsi binliktir. Tek geçiyorsa yerel kural neyse odur.
        // (Önce "ayraçtan sonra 2'den çok hane varsa binliktir" diyordum; bu EN'de "1.234"ü
        //  1234 okuyordu — oysa İngilizcede nokta ondalıktır, sayı 1,234'tür. 1000 kat hata.)
        if t.matches(&ondalik as &str).count() > 1 || b.is_empty()
            || !b.chars().all(|c| c.is_ascii_digit()) {
            (yalniz_rakam(&t), String::new())
        } else {
            (yalniz_rakam(a), b.to_string())
        }
    } else {
        // Ondalık ayracı yok. Dil bilinmiyorsa güvenli kural: son ayraçtan sonra tam 2 hane
        // varsa ondalık say; bu, "12.50" gibi tek ayraçlı yazımı doğru okur.
        let son = t.rfind(['.', ',']);
        match son {
            Some(i) if dil.is_empty() && t.len() - i - 1 == 2 && t.matches(['.', ',']).count() == 1 =>
                (yalniz_rakam(&t[..i]), t[i + 1..].to_string()),
            _ => (yalniz_rakam(&t), String::new()),
        }
    };

    let tam: i64 = tam.parse().ok()?;
    let kurus: i64 = match kesir.len() {
        0 => 0,
        1 => kesir.parse::<i64>().ok()? * 10,
        _ => kesir[..2].parse().ok()?,
    };
    let v = tam.checked_mul(100)?.checked_add(kurus)?;
    Some(if eksi { -v } else { v })
}

// ─────────────────────────────────────────────────────────────────────────────
// YAZIYLA TUTAR — beş dil
// ─────────────────────────────────────────────────────────────────────────────
// Muhasebede tutar hem rakamla hem yazıyla yazılır ve İKİSİ TUTMAK ZORUNDADIR.
// Tutmuyorsa ya okuma yanlıştır ya belge tahrif edilmiştir — her iki halde de durulur.
// Bu, tek bir OCR skorunun asla veremeyeceği bir kanıttır.

/// Bir dil için kelime→değer tablosu. `(kelime, değer, tür)`; tür: 0=birim, 1=yüz, 2=çarpan.
fn kelime_tablosu(dil: &str) -> Vec<(&'static str, i64, u8)> {
    match dil {
        "tr" => vec![
            ("sifir",0,0),("bir",1,0),("iki",2,0),("uc",3,0),("dort",4,0),("bes",5,0),
            ("alti",6,0),("yedi",7,0),("sekiz",8,0),("dokuz",9,0),
            ("on",10,0),("yirmi",20,0),("otuz",30,0),("kirk",40,0),("elli",50,0),
            ("altmis",60,0),("yetmis",70,0),("seksen",80,0),("doksan",90,0),
            ("yuz",100,1),
            ("bin",1_000,2),("milyon",1_000_000,2),("milyar",1_000_000_000,2),
        ],
        "en" => vec![
            ("zero",0,0),("one",1,0),("two",2,0),("three",3,0),("four",4,0),("five",5,0),
            ("six",6,0),("seven",7,0),("eight",8,0),("nine",9,0),("ten",10,0),
            ("eleven",11,0),("twelve",12,0),("thirteen",13,0),("fourteen",14,0),("fifteen",15,0),
            ("sixteen",16,0),("seventeen",17,0),("eighteen",18,0),("nineteen",19,0),
            ("twenty",20,0),("thirty",30,0),("forty",40,0),("fifty",50,0),
            ("sixty",60,0),("seventy",70,0),("eighty",80,0),("ninety",90,0),
            ("hundred",100,1),
            ("thousand",1_000,2),("million",1_000_000,2),("billion",1_000_000_000,2),
        ],
        // Almanca birleşik yazılır: "siebenundsiebzig" = sieben+und+siebzig. Uzun-eşleşmeli
        // parçalayıcı bunu kendiliğinden çözer; "und" yok sayılır, iki parça toplanır.
        "de" => vec![
            ("null",0,0),("eins",1,0),("ein",1,0),("zwei",2,0),("drei",3,0),("vier",4,0),
            ("funf",5,0),("sechs",6,0),("sieben",7,0),("acht",8,0),("neun",9,0),("zehn",10,0),
            ("elf",11,0),("zwolf",12,0),("dreizehn",13,0),("vierzehn",14,0),("funfzehn",15,0),
            ("sechzehn",16,0),("siebzehn",17,0),("achtzehn",18,0),("neunzehn",19,0),
            ("zwanzig",20,0),("dreissig",30,0),("vierzig",40,0),("funfzig",50,0),
            ("sechzig",60,0),("siebzig",70,0),("achtzig",80,0),("neunzig",90,0),
            ("hundert",100,1),
            ("tausend",1_000,2),("millionen",1_000_000,2),("million",1_000_000,2),
            ("milliarden",1_000_000_000,2),("milliarde",1_000_000_000,2),
        ],
        // Fransızca 70/80/90 toplama-çarpma karışımıdır: soixante-dix=60+10,
        // quatre-vingts=4×20, quatre-vingt-dix=4×20+10. "quatre vingt" özel kuralla çarpılır.
        "fr" => vec![
            ("zero",0,0),("un",1,0),("une",1,0),("deux",2,0),("trois",3,0),("quatre",4,0),
            ("cinq",5,0),("six",6,0),("sept",7,0),("huit",8,0),("neuf",9,0),("dix",10,0),
            ("onze",11,0),("douze",12,0),("treize",13,0),("quatorze",14,0),("quinze",15,0),
            ("seize",16,0),("vingt",20,0),("vingts",20,0),("trente",30,0),("quarante",40,0),
            ("cinquante",50,0),("soixante",60,0),
            ("cent",100,1),("cents",100,1),
            ("mille",1_000,2),("million",1_000_000,2),("millions",1_000_000,2),
            ("milliard",1_000_000_000,2),("milliards",1_000_000_000,2),
        ],
        // Rusça'da sayı sözcükleri çekimlidir (тысяча/тысячи/тысяч, один/одна, два/две).
        // Hepsi ayrı girdi olarak tutulur — kök kesmek "сто"yu bozardı.
        "ru" => vec![
            ("ноль",0,0),("один",1,0),("одна",1,0),("одно",1,0),("два",2,0),("две",2,0),
            ("три",3,0),("четыре",4,0),("пять",5,0),("шесть",6,0),("семь",7,0),
            ("восемь",8,0),("девять",9,0),("десять",10,0),
            ("одиннадцать",11,0),("двенадцать",12,0),("тринадцать",13,0),("четырнадцать",14,0),
            ("пятнадцать",15,0),("шестнадцать",16,0),("семнадцать",17,0),("восемнадцать",18,0),
            ("девятнадцать",19,0),
            ("двадцать",20,0),("тридцать",30,0),("сорок",40,0),("пятьдесят",50,0),
            ("шестьдесят",60,0),("семьдесят",70,0),("восемьдесят",80,0),("девяносто",90,0),
            ("сто",100,1),("двести",200,1),("триста",300,1),("четыреста",400,1),
            ("пятьсот",500,1),("шестьсот",600,1),("семьсот",700,1),("восемьсот",800,1),("девятьсот",900,1),
            ("тысяча",1_000,2),("тысячи",1_000,2),("тысяч",1_000,2),
            ("миллион",1_000_000,2),("миллиона",1_000_000,2),("миллионов",1_000_000,2),
            ("миллиард",1_000_000_000,2),("миллиарда",1_000_000_000,2),("миллиардов",1_000_000_000,2),
        ],
        _ => vec![],
    }
}

/// Yok sayılacak bağlaç/gürültü sözcükleri (dil bazlı).
fn atlanacak(dil: &str) -> &'static [&'static str] {
    match dil {
        "tr" => &["yalniz","yaziyla","tl","turk","lirasi","lira","kurus","ve"],
        "en" => &["and","only","say","dollars","dollar","cents","cent","pounds","euros"],
        "de" => &["und","euro","cent","in","worten"],
        "fr" => &["et","euros","euro","centimes","centime","la","somme","de"],
        "ru" => &["и","рублей","рубль","рубля","копеек","копейки","копеек","прописью"],
        _ => &[],
    }
}

/// Yazıyla yazılmış tutarı TAM SAYI olarak çözer (kuruş değil — tam kısım).
///
/// Almanca birleşik sözcükleri (`einhundertsiebenundsiebzig`) uzun-eşleşmeli parçalayıcıyla
/// açar. Çözülemeyen bir parça kalırsa `None` döner: **yarım okuma, yanlış okumadan iyidir**.
pub fn yaziyla_ayristir(ham: &str, dil: &str) -> Option<i64> {
    let tablo = kelime_tablosu(dil);
    if tablo.is_empty() { return None; }
    // Rusça sadeleştirmede Kiril korunur; diğerlerinde aksan düşer.
    let metin = if dil == "ru" { ham.to_lowercase().replace('ё', "е") } else { sadelestir(ham) };
    let metin = metin.replace('-', " ");

    // Sözcükleri parçala; her sözcüğü uzun-eşleşmeli olarak alt parçalara ayır (DE için şart).
    let mut parcalar: Vec<(i64, u8)> = Vec::new();
    for kelime in metin.split_whitespace() {
        if atlanacak(dil).contains(&kelime) { continue; }
        if kelime.chars().all(|c| c.is_ascii_digit()) { continue; } // rakam varsa yazı bölümü değil
        let mut kalan = kelime;
        'dis: while !kalan.is_empty() {
            // ÖNCE sayı sözcüğü, SONRA bağlaç. Sıra ters olduğunda Fransızca "deux"un
            // başındaki "de" bağlaç sanılıp kırpılıyor ve sayı hiç okunamıyordu.
            // En UZUN eşleşme: "sieben" ile "siebzig" karışmasın diye.
            let mut en_iyi: Option<(&str, i64, u8)> = None;
            for (k, v, t) in &tablo {
                if kalan.starts_with(k) && en_iyi.map(|(e, _, _)| k.len() > e.len()).unwrap_or(true) {
                    en_iyi = Some((k, *v, *t));
                }
            }
            if let Some((k, v, t)) = en_iyi {
                parcalar.push((v, t));
                kalan = &kalan[k.len()..];
                continue;
            }
            // Sayı değilse bağlaç olabilir — Almanca "siebenUNDsiebzig" sözcük İÇİNDE geçer.
            if let Some(a) = atlanacak(dil).iter().find(|a| kalan.starts_with(**a)) {
                kalan = &kalan[a.len()..];
                continue;
            }
            if parcalar.is_empty() { return None; }
            break 'dis; // sözcüğün kuyruğu tanınmadı (çekim eki) — buraya kadarını al
        }
    }
    if parcalar.is_empty() { return None; }

    // Biriktirici: birim topla · yüz çarp · çarpanda blok kapat.
    let (mut toplam, mut gecerli) = (0i64, 0i64);
    let mut onceki_birim = 0i64;
    for (v, t) in parcalar {
        match t {
            1 => { // yüz (ve RU'da "двести" gibi hazır yüzlükler)
                gecerli = if v >= 200 { gecerli + v } else if gecerli == 0 { 100 } else { gecerli * 100 };
            }
            2 => { // bin / milyon / milyar — bloğu kapat
                toplam += if gecerli == 0 { 1 } else { gecerli } * v;
                gecerli = 0;
            }
            _ => {
                // Fransızca "quatre vingt" = 4 × 20 (toplama değil çarpma).
                if dil == "fr" && v == 20 && onceki_birim == 4 { gecerli = gecerli - 4 + 80; }
                else { gecerli += v; }
            }
        }
        onceki_birim = v;
    }
    Some(toplam + gecerli)
}

// ─────────────────────────────────────────────────────────────────────────────
// ÇÖZÜM — çapraz doğrulama
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Serialize, Clone, Debug)]
pub struct BelgeBulgu {
    pub kod: String,
    pub onem: String,   // ENGEL (kayda geçmez) · UYARI (geçer, işaretli) · BILGI
    pub mesaj: String,
    pub cozum: String,
}
impl BelgeBulgu {
    fn yeni(kod: &str, onem: &str, mesaj: String, cozum: &str) -> Self {
        BelgeBulgu { kod: kod.into(), onem: onem.into(), mesaj, cozum: cozum.into() }
    }
}

/// Modelin görüntüden okuduğu HAM alanlar — hepsi belgede yazdığı gibi, metin olarak.
/// Okunamayan alan boş bırakılır; **tahmin edilmez**.
#[derive(Deserialize, Default)]
pub struct BelgeGirdi {
    #[serde(default)] pub metin: String,          // belgenin tüm okunan metni (dil/tür sezme için)
    #[serde(default)] pub dil: String,            // biliniyorsa; boşsa sezilir
    #[serde(default)] pub belge_no: String,
    #[serde(default)] pub tarih: String,
    #[serde(default)] pub karsi_taraf: String,
    #[serde(default)] pub vergi_no: String,
    #[serde(default)] pub matrah: String,
    #[serde(default)] pub vergi_tutari: String,
    #[serde(default)] pub vergi_orani: String,
    #[serde(default)] pub toplam: String,
    #[serde(default)] pub tutar_yaziyla: String,
    #[serde(default)] pub para_birimi: String,
    /// Kalem tutarları (varsa) — toplamla mutabakat için.
    #[serde(default)] pub kalemler: Vec<String>,
    /// Belgeden okunan ALAN ETİKETLERİ (ör. "Matrih", "Genel Toolam"). Motor bunları
    /// sözlükle karşılaştırıp kanonik terime bağlar; böylece belgeyi kim yüklerse yüklesin
    /// aynı alan aynı adla ve aynı hesap koduyla gelir.
    #[serde(default)] pub etiketler: Vec<String>,
}

#[derive(Serialize)]
pub struct BelgeCozum {
    pub dil: String,
    pub dil_puani: usize,
    pub belge_turu: String,
    pub belge_no: String,
    pub tarih: String,
    pub karsi_taraf: String,
    pub vergi_no: String,
    pub matrah: Option<i64>,
    pub vergi_tutari: Option<i64>,
    pub toplam: Option<i64>,
    pub yaziyla_tutar: Option<i64>,
    pub kalem_toplami: Option<i64>,
    pub para_birimi: String,
    /// 0–100. Yalnız DOĞRULANMIŞ alanlar yükseltir; doğrulanamayan alan puan getirmez.
    pub guven: u8,
    pub bulgular: Vec<BelgeBulgu>,
    pub deftere_hazir: bool,
    pub not: String,
    /// Okunan etiketlerin kanonik karşılıkları — düzeltme motorundan gelir.
    pub etiket_cozumu: Vec<serde_json::Value>,
}

fn tl(k: i64) -> String { format!("{}.{:02}", k / 100, (k % 100).abs()) }

/// Ham okumayı çözer ve belgenin kendi içinde tutarlı olup olmadığını sınar.
pub fn coz(g: &BelgeGirdi) -> BelgeCozum {
    let mut b: Vec<BelgeBulgu> = Vec::new();

    // 1) DİL — sayı biçimi buna bağlı, önce bu çözülmeli.
    let (dil, puan) = if !g.dil.is_empty() { (g.dil.clone(), usize::MAX) } else { dil_sez(&g.metin) };
    if dil.is_empty() {
        b.push(BelgeBulgu::yeni("B01", "ENGEL",
            "Belgenin dili sezilemedi — hiçbir alan etiketi tanınmadı.".into(),
            "Dili elle seç (tr/en/de/fr/ru). Dil bilinmeden '1.234' TR'de 1.234, EN'de 1,234 okunur."));
    } else if !sozluk()["diller"][&dil]["cekirdek"].as_bool().unwrap_or(false) {
        b.push(BelgeBulgu::yeni("B09", "BILGI",
            format!("{} ikinci halka dil — alan ve tutar okunur, ülkeye özgü vergi alanları çözülmez.",
                sozluk()["diller"][&dil]["ad"].as_str().unwrap_or(&dil)),
            "Türkçe/İngilizce belgelerde tam kapsam vardır; bu belgede vergi alanlarını gözden geçir."));
    }

    let tur = belge_turu(&g.metin, &dil);
    if tur.is_empty() {
        b.push(BelgeBulgu::yeni("B02", "UYARI",
            "Belge türü anlaşılamadı (fatura mı, makbuz mu, dekont mu).".into(),
            "Türü elle seç — hangi fişin doğacağı buna bağlıdır."));
    }

    let sy = |s: &str| if s.trim().is_empty() { None } else { sayi_ayristir(s, &dil) };
    let matrah = sy(&g.matrah);
    let vergi = sy(&g.vergi_tutari);
    let toplam = sy(&g.toplam);
    let yaziyla = if g.tutar_yaziyla.trim().is_empty() { None }
                  else { yaziyla_ayristir(&g.tutar_yaziyla, &dil).map(|x| x * 100) };

    let kalemler: Vec<i64> = g.kalemler.iter().filter_map(|k| sayi_ayristir(k, &dil)).collect();
    let kalem_toplami = if kalemler.is_empty() { None } else { Some(kalemler.iter().sum()) };
    if kalemler.len() < g.kalemler.len() {
        b.push(BelgeBulgu::yeni("B08", "UYARI",
            format!("{} kalemin {}'i sayıya çevrilemedi.", g.kalemler.len(), g.kalemler.len() - kalemler.len()),
            "Okunamayan kalemleri elle gir; toplam mutabakatı ancak o zaman kanıt olur."));
    }

    // 2) RAKAM ↔ YAZI — belgenin kendi içindeki en güçlü kanıt.
    let mut yazi_dogrulandi = false;
    if let (Some(t), Some(y)) = (toplam, yaziyla) {
        // Yazıyla tutar genelde yalnız TAM kısmı söyler; kuruş ayrı yazılır.
        if y == t || y == t - (t % 100) {
            yazi_dogrulandi = true;
            b.push(BelgeBulgu::yeni("B00", "BILGI",
                format!("Rakam ({}) ile yazıyla tutar birbirini doğruladı.", tl(t)),
                "Bu alan çapraz doğrulandı; elle kontrol gerekmez."));
        } else {
            b.push(BelgeBulgu::yeni("B03", "ENGEL",
                format!("Rakam ({}) ile yazı ({}) UYUŞMUYOR.", tl(t), tl(y)),
                "Ya okuma hatalı ya belge tahrif edilmiş. Belgeyi büyütüp iki alanı da elle doğrula; \
                 düzelmezse kayda geçirme."));
        }
    }

    // 3) KALEM TOPLAMI ↔ BELGE TOPLAMI — 1889 faturasında okumayı kanıtlayan kontrol buydu.
    let mut kalem_dogrulandi = false;
    if let (Some(k), Some(t)) = (kalem_toplami, toplam) {
        let hedef = if matrah.is_some() && vergi.is_some() { matrah.unwrap() } else { t };
        if k == hedef {
            kalem_dogrulandi = true;
            b.push(BelgeBulgu::yeni("B00", "BILGI",
                format!("{} kalemin toplamı ({}) belgedeki tutara oturdu.", kalemler.len(), tl(k)),
                "Kalem okumaları aritmetikle kanıtlandı."));
        } else {
            b.push(BelgeBulgu::yeni("B04", "UYARI",
                format!("Kalem toplamı {} ≠ belge tutarı {} (fark {}).", tl(k), tl(hedef), tl(k - hedef)),
                "Bir kalem yanlış okunmuş veya atlanmış olabilir; farkı veren kalemi ara."));
        }
    }

    // 4) MATRAH + VERGİ = TOPLAM
    if let (Some(m), Some(v), Some(t)) = (matrah, vergi, toplam) {
        if m + v != t {
            b.push(BelgeBulgu::yeni("B05", "UYARI",
                format!("Matrah ({}) + vergi ({}) = {} ≠ toplam ({}).", tl(m), tl(v), tl(m + v), tl(t)),
                "Üç alandan biri yanlış okunmuş. Belgede iskonto/tevkifat varsa fark onu gösteriyor olabilir."));
        }
        // Oran tutarlılığı — beyan edilen oran varsa hesapla karşılaştır.
        if let Some(o) = g.vergi_orani.chars().filter(|c| c.is_ascii_digit()).collect::<String>().parse::<i64>().ok() {
            if o > 0 && m > 0 {
                let beklenen = m * o / 100;
                if (beklenen - v).abs() > 100 { // 1 TL yuvarlama payı
                    b.push(BelgeBulgu::yeni("B06", "UYARI",
                        format!("%{o} oranla matrahtan {} çıkar, belgede {} yazıyor.", tl(beklenen), tl(v)),
                        "Oran veya tutar yanlış okunmuş olabilir; belgedeki oranı doğrula."));
                }
            }
        }
    }

    // 5) ZORUNLU ALANLAR — VUK 227: belgede bulunması gerekenler okunamadıysa kayıt doğmaz.
    let mut eksik: Vec<&str> = Vec::new();
    if g.tarih.trim().is_empty() { eksik.push("tarih"); }
    if toplam.is_none() { eksik.push("tutar"); }
    if !tur.is_empty() {
        for z in sozluk()["belge_turleri"][&tur]["zorunlu_alanlar"].as_array().into_iter().flatten() {
            if z.as_str() == Some("karsi_taraf") && g.karsi_taraf.trim().is_empty() {
                eksik.push("karşı taraf");
            }
        }
    }
    if !eksik.is_empty() {
        b.push(BelgeBulgu::yeni("B07", "ENGEL",
            format!("Zorunlu alan okunamadı: {}.", eksik.join(", ")),
            "Eksik alanı elle gir. Belgesiz/eksik belgeyle kayıt yapılamaz (VUK 227)."));
    }

    // 5B) ETİKET ÇÖZÜMÜ — okunan alan adlarını kanonik terime bağla.
    // Motorun "tüm belge ekleme yollarının çekirdeği" olmasının anlamı budur: belgeyi kim
    // yüklerse yüklesin, aynı alan aynı adla VE aynı TDHP hesabıyla çıkar. Düzeltme sessiz
    // değildir; her biri sonuçta görünür ve şüpheli olan bulguya dönüşür.
    let mut etiket_cozumu: Vec<serde_json::Value> = Vec::new();
    for e in &g.etiketler {
        let d = crate::duzelt::duzelt(e, &dil, &tur);
        etiket_cozumu.push(serde_json::json!({
            "okunan": e, "durum": d.durum,
            "terim": d.tam_eslesme.clone().or_else(|| d.oneri.as_ref().map(|o| o.oneri.clone())),
            "hesap": d.oneri.as_ref().and_then(|o| o.hesap.clone()),
            "benzerlik": d.oneri.as_ref().map(|o| o.benzerlik),
            "belirsiz": d.belirsiz,
        }));
        match d.durum.as_str() {
            "ONERI" => b.push(BelgeBulgu::yeni("B10", "UYARI",
                format!("'{e}' etiketi sözlükte yok; '{}' olabilir.",
                    d.oneri.as_ref().map(|o| o.oneri.as_str()).unwrap_or("")),
                "Öneriyi onayla ya da doğru terimi elle gir — otomatik uygulanmaz.")),
            "BELIRSIZ" => b.push(BelgeBulgu::yeni("B11", "UYARI",
                format!("'{e}' etiketi için birden çok aday var: {}.", d.belirsiz.join(", ")),
                "Hangi alan olduğunu seç; motor belirsizken tahmin yürütmez.")),
            _ => {}
        }
    }

    // 6) GÜVEN — yalnız DOĞRULANMIŞ olan puan getirir. Okunmuş ama doğrulanmamış alan
    //    güven vermez; bu, "emin görünen yanlış okuma"ya karşı tek savunmadır.
    let mut guven: i32 = 0;
    if !dil.is_empty() { guven += 10; }
    if !tur.is_empty() { guven += 10; }
    if toplam.is_some() { guven += 15; }
    if !g.tarih.trim().is_empty() { guven += 10; }
    if !g.karsi_taraf.trim().is_empty() { guven += 10; }
    if yazi_dogrulandi { guven += 25; }   // en güçlü kanıt
    if kalem_dogrulandi { guven += 20; }
    if matrah.is_some() && vergi.is_some() && toplam.is_some()
        && matrah.unwrap() + vergi.unwrap() == toplam.unwrap() { guven += 15; }
    for x in &b {
        if x.onem == "ENGEL" { guven -= 40; } else if x.onem == "UYARI" { guven -= 10; }
    }
    let guven = guven.clamp(0, 100) as u8;

    let engel = b.iter().any(|x| x.onem == "ENGEL");
    BelgeCozum {
        dil: dil.clone(),
        dil_puani: if puan == usize::MAX { 0 } else { puan },
        belge_turu: tur,
        belge_no: g.belge_no.trim().to_string(),
        tarih: g.tarih.trim().to_string(),
        karsi_taraf: g.karsi_taraf.trim().to_string(),
        vergi_no: g.vergi_no.trim().to_string(),
        matrah, vergi_tutari: vergi, toplam,
        yaziyla_tutar: yaziyla,
        kalem_toplami,
        para_birimi: g.para_birimi.trim().to_uppercase(),
        guven,
        deftere_hazir: !engel && guven >= 60,
        etiket_cozumu,
        not: if engel {
            "Belge kayda geçirilemez — ENGEL bulgusu var. Önce çözümü uygula.".into()
        } else if guven < 60 {
            "Okuma doğrulanamadı. Muhasebeci onaylamadan deftere işlenmemeli.".into()
        } else {
            "Okuma belgenin kendi aritmetiğiyle doğrulandı; fiş taslağı üretilebilir.".into()
        },
        bulgular: b,
    }
}

/// POST /api/belge/coz — okunan ham alanları çözer ve doğrular.
pub async fn belge_coz(Json(g): Json<BelgeGirdi>) -> Json<BelgeCozum> { Json(coz(&g)) }

/// GET /api/belge/diller — desteklenen diller ve kapsamları.
pub async fn belge_diller() -> Json<serde_json::Value> {
    let s = sozluk();
    let mut liste: Vec<serde_json::Value> = Vec::new();
    for (kod, d) in s["diller"].as_object().into_iter().flatten() {
        liste.push(serde_json::json!({
            "kod": kod, "ad": d["ad"],
            "cekirdek": d["cekirdek"],
            "kapsam": if d["cekirdek"].as_bool().unwrap_or(false) {
                "Tam — alan etiketleri, sayı biçimi, yazıyla tutar, vergi alanları"
            } else {
                "Alan etiketleri, sayı biçimi, yazıyla tutar (ülkeye özgü vergi alanları yok)"
            },
            "sayi_ornegi": d["sayi_bicimi"]["ornek"],
        }));
    }
    liste.sort_by_key(|x| (!x["cekirdek"].as_bool().unwrap_or(false), x["kod"].as_str().unwrap_or("").to_string()));
    Json(serde_json::json!({ "diller": liste, "belge_turleri": s["belge_turleri"] }))
}

#[cfg(test)]
mod testler;
