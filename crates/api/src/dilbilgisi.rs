//! DİL BİLGİSİ KATMANI — Türkçe ve İngilizce çekim eklerini çözer.
//!
//! ## Sorun: çekimli biçimleri elle saymak ölçeklenmiyor
//!
//! Belge etiketleri sözlükte kök halinde tutulur ama belgede **çekimli** yazar:
//!
//! | Sözlükte | Belgede görülen |
//! |---|---|
//! | `miktar` | MİKTAR · MİKTARI · MİKTARLARI |
//! | `tutar` | TUTAR · TUTARI · TUTARLAR · TUTARLARI |
//! | `birim fiyat` | BİRİM FİYAT · BİRİM FİYATI · BİRİM FİYATLARI |
//! | `unit price` | UNIT PRICE · UNIT PRICES |
//!
//! Her biçimi sözlüğe elle yazmak hem büyür hem eksik kalır: görülmemiş bir çekim
//! (MİKTARLARI) hiç eşleşmez. Kural yazmak, listeyi uzatmaktan üstündür.
//!
//! ## Türkçe: SONDAN EKLEMELİ dil + ÜNLÜ UYUMU
//!
//! Türkçe eklerin ünlüsü, gövdenin son ünlüsüne uyar. Bu bir süs değil, **ayırt edici
//! bilgi**: uyuma aykırı bir son hece ek OLAMAZ, gövdenin parçasıdır.
//!
//! - **Büyük ünlü uyumu**: kalın ünlüden (a ı o u) sonra kalın, ince ünlüden (e i ö ü)
//!   sonra ince ek gelir. `tutar` (son ünlü *a* → kalın) + iyelik → `tutarı` ✔ ·
//!   `tutari` ✘ olamaz.
//! - **Küçük ünlü uyumu** dar ünlülü eklerde düzlük/yuvarlaklığı belirler:
//!   `a ı` → *ı* · `e i` → *i* · `o u` → *u* · `ö ü` → *ü*.
//!
//! Bu yüzden ek soyarken **uyum sınanır**. Sınanmazsa "TARİH" sözcüğünden sondaki *i*
//! soyulup "TAR" elde edilir ve alakasız eşleşmeler doğar. Uyum sınanınca soyulmaz:
//! *tarih*'in son ünlüsü *i* (ince), ama *i* eki almış bir gövde olsaydı kökün son ünlüsü
//! de ince olurdu — burada asıl ayırt edici, kökün en az 3 harf kalması ve ekin
//! **ünsüzle bitmesi** koşuludur.
//!
//! **Ünsüz yumuşaması** ayrıca ele alınır: `kitap` + *ı* → `kitabı`. Ek soyulunca kalan
//! `kitab` sözlükte yoktur; `p/ç/t/k` geri getirilmelidir. Muhasebe etiketlerinde bu
//! *fiyat → fiyadı* gibi biçimlerde karşımıza çıkar.
//!
//! ## İngilizce: çekim sığ, kural az
//!
//! Çoğul `-s`/`-es`, `-ies → y`; iyelik `'s`. Türkçedeki gibi uyum yok.
//!
//! ## Doktrin: kök ADAY üretir, karar yine eşleşmenindir
//!
//! Bu modül bir kelimenin olası köklerini **çoğaltır**, hangisinin doğru olduğuna karar
//! vermez. Çağıran, adayları sözlükte arar; hiçbiri tutmazsa eşleşme yoktur — uydurma yok.

/// TÜRKÇE'YE ÖZGÜ KÜÇÜK HARFE ÇEVİRME.
///
/// Türkçede i/I çifti diğer dillerden farklıdır: **İ → i** ve **I → ı**. Rust'ın
/// `to_lowercase()` Unicode varsayılanını uygular ve `"İ"` için **birleşik noktalı**
/// `i` + U+0307 üretir. Bu iki şeyi birden bozar:
///
/// 1. Fazladan birleşen karakter, ünlü sınamasını başarısız kılar — "CİNSİ" çözümlenemez.
/// 2. `I → i` katlaması **kalın ünlüyü ince gösterir** ve ünlü uyumu sınaması çöker:
///    "MİKTARLARI" → yanlış küçültmede "miktarlari" olur, ek ünlüsü ince görünür,
///    gövde kalındır, uyum tutmaz ve ek soyulamaz.
///
/// Ünlü uyumu bu katmanın ayırt edici bilgisi olduğu için katlama BURADA yapılamaz.
fn tr_kucult(s: &str) -> String {
    let mut o = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            'İ' => o.push('i'),
            'I' => o.push('ı'),
            _ => o.extend(c.to_lowercase()),
        }
    }
    o
}

/// Türkçe ünlüler, kalınlık/incelik ve düzlük/yuvarlaklık ile.
fn unlu(c: char) -> Option<(bool, bool)> {
    // (kalin, yuvarlak)
    match c {
        'a' => Some((true, false)),  'ı' => Some((true, false)),
        'o' => Some((true, true)),   'u' => Some((true, true)),
        'e' => Some((false, false)), 'i' => Some((false, false)),
        'ö' => Some((false, true)),  'ü' => Some((false, true)),
        _ => None,
    }
}

fn son_unlu(s: &str) -> Option<(bool, bool)> {
    s.chars().rev().find_map(unlu)
}

/// Dar ünlü ekin, gövdeye göre olması gereken biçimi (ı/i/u/ü).
fn dar_ek_unlusu(govde: &str) -> Option<char> {
    let (kalin, yuvarlak) = son_unlu(govde)?;
    Some(match (kalin, yuvarlak) {
        (true, false) => 'ı', (true, true) => 'u',
        (false, false) => 'i', (false, true) => 'ü',
    })
}

/// Düz ünlü ekin biçimi (a/e) — büyük ünlü uyumu.
fn duz_ek_unlusu(govde: &str) -> Option<char> {
    son_unlu(govde).map(|(kalin, _)| if kalin { 'a' } else { 'e' })
}

/// Ünsüz yumuşaması geri alınır: `kitab` → `kitap`. Muhasebe etiketlerinde
/// `fiyad-` (< fiyat), `kayd-` (< kayıt) gibi biçimlerde karşımıza çıkar.
fn sertlestir(kok: &str) -> Option<String> {
    let son = kok.chars().last()?;
    let sert = match son { 'b' => 'p', 'c' => 'ç', 'd' => 't', 'g' => 'k', 'ğ' => 'k', _ => return None };
    let mut s: String = kok.chars().take(kok.chars().count() - 1).collect();
    s.push(sert);
    Some(s)
}

/// En az bu kadar harf kalmalı — aşırı soyma alakasız eşleşme üretir.
const EN_KISA_KOK: usize = 3;

/// Bir Türkçe sözcüğün olası köklerini üretir (kendisi dahil).
///
/// Ekler **ünlü uyumu sınanarak** soyulur: gövdeye uymayan bir son hece ek olamaz.
/// Aynı sözcükten birden çok kök çıkabilir (`tutarları` → `tutarlar`, `tutar`);
/// hepsi aday olarak döner, seçimi çağıran yapar.
pub fn tr_kokler(kelime: &str) -> Vec<String> {
    let k = tr_kucult(kelime);
    let mut cikti = vec![k.clone()];
    let mut katman = vec![k];

    // İki katman yeter: "tutar-lar-ı" gibi. Daha derini etiketlerde görülmüyor.
    for _ in 0..2 {
        let mut sonraki = Vec::new();
        for g in &katman {
            for kok in bir_ek_soy(g) {
                if kok.chars().count() >= EN_KISA_KOK && !cikti.contains(&kok) {
                    cikti.push(kok.clone());
                    sonraki.push(kok.clone());
                    // Yumuşamış ünsüz varsa sert biçimi de aday
                    if let Some(s) = sertlestir(&kok) {
                        if !cikti.contains(&s) { cikti.push(s); }
                    }
                }
            }
        }
        if sonraki.is_empty() { break; }
        katman = sonraki;
    }
    cikti
}

/// Tek bir çekim ekini, ÜNLÜ UYUMUNU SINAYARAK soyar.
fn bir_ek_soy(k: &str) -> Vec<String> {
    let mut cikti = Vec::new();
    let n = k.chars().count();
    if n <= EN_KISA_KOK { return cikti; }
    let harf: Vec<char> = k.chars().collect();
    let kes = |say: usize| -> String { harf[..n - say].iter().collect() };

    // ── ÇOĞUL: -lar / -ler ────────────────────────────────────────────────
    if n > 3 {
        let son3: String = harf[n - 3..].iter().collect();
        if son3 == "lar" || son3 == "ler" {
            let govde = kes(3);
            if duz_ek_unlusu(&govde) == son3.chars().nth(1) { cikti.push(govde); }
        }
    }

    // ── İYELİK 3. tekil: ünlüyle biten gövdede -sı/-si/-su/-sü ────────────
    if n > 4 {
        let son2: String = harf[n - 2..].iter().collect();
        if son2.starts_with('s') {
            let govde = kes(2);
            if son_unlu(&govde).is_some()
                && govde.chars().last().map(|c| unlu(c).is_some()).unwrap_or(false)
                && dar_ek_unlusu(&govde) == son2.chars().nth(1)
            { cikti.push(govde); }
        }
    }

    // ── İYELİK / BELİRTME: -ı -i -u -ü (ünsüzle biten gövde) ──────────────
    if let Some(son) = harf.last() {
        if matches!(son, 'ı' | 'i' | 'u' | 'ü') {
            let govde = kes(1);
            let ünsüzle_bitiyor = govde.chars().last().map(|c| unlu(c).is_none()).unwrap_or(false);
            if ünsüzle_bitiyor && dar_ek_unlusu(&govde) == Some(*son) {
                cikti.push(govde.clone());
                if let Some(s) = sertlestir(&govde) { cikti.push(s); }
            }
        }
    }

    // ── DURUM EKLERİ: -da/-de/-ta/-te · -dan/-den/-tan/-ten · -ın/-in/-un/-ün ──
    for (ek_uzunluk, kalip) in [(2usize, "da"), (2, "de"), (2, "ta"), (2, "te"),
                                (3, "dan"), (3, "den"), (3, "tan"), (3, "ten")] {
        if n > ek_uzunluk + EN_KISA_KOK - 1 {
            let son: String = harf[n - ek_uzunluk..].iter().collect();
            if son == kalip {
                let govde = kes(ek_uzunluk);
                if duz_ek_unlusu(&govde) == kalip.chars().nth(1) { cikti.push(govde); }
            }
        }
    }
    if n > 2 {
        let son2: String = harf[n - 2..].iter().collect();
        if matches!(son2.as_str(), "ın" | "in" | "un" | "ün") {
            let govde = kes(2);
            if dar_ek_unlusu(&govde) == son2.chars().next() { cikti.push(govde); }
        }
    }
    cikti
}

/// İngilizce çekim: çoğul ve iyelik. Türkçedeki gibi uyum yoktur, kural sığdır.
pub fn en_kokler(kelime: &str) -> Vec<String> {
    let k = kelime.to_lowercase();
    let mut cikti = vec![k.clone()];
    let n = k.chars().count();
    let kes = |s: &str, say: usize| -> String {
        s.chars().take(s.chars().count() - say).collect()
    };
    if let Some(g) = k.strip_suffix("'s").or_else(|| k.strip_suffix("’s")) {
        if g.chars().count() >= 2 { cikti.push(g.to_string()); }
    }
    if n > 3 && k.ends_with("ies") { cikti.push(kes(&k, 3) + "y"); }
    if n > 3 && k.ends_with("es") { cikti.push(kes(&k, 2)); }
    if n > 2 && k.ends_with('s') && !k.ends_with("ss") { cikti.push(kes(&k, 1)); }
    cikti.dedup();
    cikti
}

/// Bir ÖBEĞİN (çok sözcüklü etiketin) kök adayları.
///
/// Türkçede ad tamlamasında ek **son sözcüğe** gelir: "birim fiyatı", "genel toplamı",
/// "mal hizmet tutarı". Bu yüzden yalnız son sözcük çekimden arındırılır; baştakini
/// soymak "faturalar tarihi" gibi biçimleri bozardı.
pub fn kokler(obek: &str, dil: &str) -> Vec<String> {
    let sozcukler: Vec<&str> = obek.split_whitespace().collect();
    let Some((son, bas)) = sozcukler.split_last() else { return vec![obek.to_string()] };
    let adaylar = if dil == "en" { en_kokler(son) } else { tr_kokler(son) };
    adaylar.into_iter()
        .map(|k| if bas.is_empty() { k } else { format!("{} {}", bas.join(" "), k) })
        .collect()
}

#[cfg(test)]
mod testler {
    use super::*;

    fn var(v: &[String], s: &str) -> bool { v.iter().any(|x| x == s) }

    /// Muhasebe belgelerinde en sık görülen çekim: 3. tekil iyelik.
    #[test]
    fn iyelik_eki_soyulur() {
        assert!(var(&tr_kokler("miktarı"), "miktar"));
        assert!(var(&tr_kokler("tutarı"), "tutar"));
        assert!(var(&tr_kokler("birimi"), "birim"));
        assert!(var(&tr_kokler("toplamı"), "toplam"));
        assert!(var(&tr_kokler("oranı"), "oran"));
    }

    /// **ÜNLÜ UYUMU AYIRT EDİCİDİR.** Uyuma aykırı bir son hece ek OLAMAZ.
    /// Sınanmazsa "tarih" → "tar" olur ve alakasız eşleşmeler doğar.
    #[test]
    fn unlu_uyumuna_aykiri_ek_soyulmaz() {
        // "tutar" kalın gövde; ince ek alamaz → "tutari" bir çekim değildir.
        assert!(!var(&tr_kokler("tutari"), "tutar"), "uyuma aykırı ek soyuldu");
        // "birim" ince gövde; kalın ek alamaz.
        assert!(!var(&tr_kokler("birimı"), "birim"), "uyuma aykırı ek soyuldu");
    }

    /// Çoğul + iyelik birlikte: "tutarları" → "tutarlar" → "tutar".
    #[test]
    fn cogul_ve_iyelik_birlikte_cozulur() {
        let k = tr_kokler("tutarları");
        assert!(var(&k, "tutarlar"), "{k:?}");
        assert!(var(&k, "tutar"), "{k:?}");
    }

    /// Ünsüz yumuşaması geri alınır: `fiyadı` → `fiyat`.
    #[test]
    fn unsuz_yumusamasi_geri_alinir() {
        assert!(var(&tr_kokler("fiyadı"), "fiyat"), "{:?}", tr_kokler("fiyadı"));
    }

    /// **AŞIRI SOYMAYA KARŞI.** Kök en az 3 harf kalmalı.
    #[test]
    fn asiri_soyma_engellenir() {
        for k in tr_kokler("adı") { assert!(k.chars().count() >= 2, "çok kısa kök: {k}"); }
        assert!(!var(&tr_kokler("kdv"), "kd"));
    }

    /// Öbekte ek SON sözcüğe gelir — ad tamlaması kuralı.
    #[test]
    fn obekte_son_sozcuk_soyulur() {
        let k = kokler("birim fiyatı", "tr");
        assert!(var(&k, "birim fiyat"), "{k:?}");
        let k2 = kokler("mal hizmet tutarı", "tr");
        assert!(var(&k2, "mal hizmet tutar"), "{k2:?}");
    }

    #[test]
    fn ingilizce_cogul_ve_iyelik() {
        assert!(var(&en_kokler("quantities"), "quantity"));
        assert!(var(&en_kokler("prices"), "price"));
        assert!(var(&en_kokler("boxes"), "box"));
        assert!(var(&kokler("unit prices", "en"), "unit price"));
    }
}

#[cfg(test)]
mod tr_kucultme_testleri {
    use super::*;
    fn var(v: &[String], s: &str) -> bool { v.iter().any(|x| x == s) }

    /// **Türkçe i/I çifti.** `to_lowercase()` "İ" için birleşik noktalı karakter üretir
    /// ve "I"yı "i" yapar — ikisi de ünlü uyumu sınamasını çökertir.
    #[test]
    fn buyuk_harfli_baslik_cozulur() {
        assert!(var(&tr_kokler("CİNSİ"), "cins"), "{:?}", tr_kokler("CİNSİ"));
        assert!(var(&tr_kokler("MİKTARLARI"), "miktar"), "{:?}", tr_kokler("MİKTARLARI"));
        assert!(var(&tr_kokler("TUTARI"), "tutar"));
        assert!(var(&tr_kokler("BİRİMİ"), "birim"));
        assert!(var(&tr_kokler("ADEDİ"), "adet"), "{:?}", tr_kokler("ADEDİ"));
    }

    /// Büyük harfte de ünlü uyumu KORUNUR: "I" kalın, "İ" incedir.
    #[test]
    fn buyuk_harfte_de_uyum_sinanir() {
        // "TUTARI" (kalın gövde + kalın ek) çözülür; "TUTARİ" bir çekim DEĞİLDİR.
        assert!(var(&tr_kokler("TUTARI"), "tutar"));
        assert!(!var(&tr_kokler("TUTARİ"), "tutar"), "uyuma aykırı ek soyuldu");
    }
}
