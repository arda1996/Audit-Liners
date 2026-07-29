//! Türkçe isim kanonikleştirme + benzerlik — cari/karşı taraf eşleştirmenin dil katmanı.
//!
//! Kaynak fikir: takipsistemiv2 (canonical_name + name_match_score). Temizlenerek port edildi:
//! 2.460 satırlık isim sözlüğü ALINMADI (ad/soyad ayrıştırmıyoruz; cari adı bütün eşleşir),
//! regex yerine saf dilim işlemleri (domain bağımlılıksız kalır).

/// Türkçe harfleri ASCII'ye katlar ve büyütür (İ/I ıi tuzağına dayanıklı karşılaştırma).
pub fn tr_katla(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'i' | 'ı' | 'İ' | 'I' | 'î' | 'Î' => 'I',
            'ş' | 'Ş' => 'S',
            'ğ' | 'Ğ' => 'G',
            'ç' | 'Ç' => 'C',
            'ö' | 'Ö' => 'O',
            'ü' | 'ü' | 'Ü' => 'U',
            'â' | 'Â' => 'A',
            'û' | 'Û' => 'U',
            c => c.to_ascii_uppercase(),
        })
        .collect()
}

const UNVANLAR: [&str; 7] = ["DR", "PROF", "MR", "MRS", "SAYIN", "BAY", "BAYAN"];

/// Banka ödeme etiketi kuyruğu mu? ("- FAST Anlık Ödeme", "- EFT…", "- Havale…" vb.)
fn odeme_kuyrugu(kelime: &str) -> bool {
    const ETIKET: [&str; 8] = ["FAST", "ANLIK", "ODEME", "HAVALE", "EFT", "TRANSFER", "GONDERILEN", "ALINAN"];
    ETIKET.iter().any(|e| kelime.starts_with(e))
}

/// Deterministik kanonik isim: TR katlama + unvan temizliği + "- ödeme etiketi" kuyruğu kırpma.
/// Kırpma sonrası tek kelime kalır ve orijinal çok kelimeliyse orijinale döner (aşırı kırpma koruması).
pub fn kanonik_isim(ham: &str) -> String {
    let katli = tr_katla(ham);
    // "-" sonrası ödeme etiketi kuyruğunu at
    let govde = match katli.split(['-', '–', '—']).next() {
        Some(bas) => {
            let kalan = katli[bas.len()..].trim_start_matches(['-', '–', '—', ' ']);
            if !kalan.is_empty() && odeme_kuyrugu(kalan.trim()) { bas } else { &katli }
        }
        None => &katli,
    };
    let kelime_mi = |k: &&str| !k.is_empty() && k.chars().any(char::is_alphanumeric) && !UNVANLAR.contains(k);
    let kelimeler: Vec<&str> = govde
        .split(|c: char| c.is_whitespace() || c == '.' || c == ',')
        .filter(kelime_mi)
        .collect();
    let sonuc = kelimeler.join(" ");
    let orijinal_kelime = katli.split_whitespace().count();
    if sonuc.split_whitespace().count() < 2 && orijinal_kelime >= 2 && govde.len() < katli.len() {
        // aşırı kırpıldı — kuyruğu geri al, yalnız unvan/temizlik uygula
        return katli
            .split(|c: char| c.is_whitespace() || c == '.' || c == ',')
            .filter(kelime_mi)
            .collect::<Vec<_>>()
            .join(" ");
    }
    sonuc
}

/// Kelime-kümesi benzerliği (Jaccard). 1.0 = kanonik formda özdeş.
pub fn isim_skoru(a: &str, b: &str) -> f64 {
    let ka = kanonik_isim(a);
    let kb = kanonik_isim(b);
    if ka.is_empty() || kb.is_empty() {
        return 0.0;
    }
    if ka == kb {
        return 1.0;
    }
    let sa: std::collections::BTreeSet<&str> = ka.split_whitespace().collect();
    let sb: std::collections::BTreeSet<&str> = kb.split_whitespace().collect();
    let kesisim = sa.intersection(&sb).count() as f64;
    let birlesim = sa.union(&sb).count() as f64;
    if birlesim == 0.0 { 0.0 } else { kesisim / birlesim }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn tr_katlama_i_tuzagi() {
        assert_eq!(tr_katla("ilİMDaĞ şçöü"), "ILIMDAG SCOU");
    }

    #[test]
    fn unvan_ve_kuyruk_temizligi() {
        assert_eq!(kanonik_isim("SAYIN Ali VELİ - FAST Anlık Ödeme"), "ALI VELI");
        assert_eq!(kanonik_isim("Dr. Ayşe Yılmaz"), "AYSE YILMAZ");
    }

    #[test]
    fn asiri_kirpma_korumasi() {
        // "-" sonrası etiket DEĞİL, ismin parçası → kuyruk atılmaz
        assert_eq!(kanonik_isim("ALİ - VELİ"), "ALI VELI");
    }

    #[test]
    fn skor() {
        assert_eq!(isim_skoru("Ali Veli", "ALİ VELİ - EFT Transfer"), 1.0);
        assert!(isim_skoru("Ali Veli Kaya", "Ali Kaya") > 0.6);
        assert_eq!(isim_skoru("Ali Veli", "Mehmet Demir"), 0.0);
    }
}
