//! Hesap planı yükleme: `data/tdhp.csv` (kod,ad) → Hesap. Tip/doğa/seviye/yaprak KODDAN türetilir.
//! Kurallar: docs/tasarim/hesap-plani-format.md

use crate::hesap::{Hesap, HesapDogasi, HesapTipi};

/// Sınıf 7 yansıtma hesapları: adlarında "(-)" yok ama ALACAK çalışırlar (istisna).
const YANSITMA_KODLARI: &[&str] = &[
    "701", "711", "721", "731", "741", "751", "761", "771", "781", "798",
];

fn ters(d: HesapDogasi) -> HesapDogasi {
    match d {
        HesapDogasi::Borc => HesapDogasi::Alacak,
        HesapDogasi::Alacak => HesapDogasi::Borc,
    }
}

/// Tek satırdan Hesap üret. Geçersiz kod (boş/sayı değil/sınıf 8) → None.
pub fn hesap_olustur(kod: &str, ad: &str) -> Option<Hesap> {
    let kod = kod.trim();
    let ad = ad.trim();
    let sinif = kod.chars().next()?.to_digit(10)? as u8;

    // Taban doğa (sınıf bazlı)
    let taban = match sinif {
        1 | 2 => HesapDogasi::Borc,
        3 | 4 | 5 => HesapDogasi::Alacak,
        6 => HesapDogasi::Alacak, // gelir tablosu tabanı
        7 => HesapDogasi::Borc,
        9 => HesapDogasi::Borc,
        _ => return None, // 8 serbest / geçersiz
    };

    let duzenleyici = ad.ends_with("(-)");
    let mut dogasi = if duzenleyici { ters(taban) } else { taban };
    if YANSITMA_KODLARI.contains(&kod) {
        dogasi = HesapDogasi::Alacak; // yansıtma istisnası
    }

    let tip = match sinif {
        1 | 2 => HesapTipi::Aktif,
        3 | 4 | 5 => HesapTipi::Pasif,
        6 => {
            if dogasi == HesapDogasi::Alacak {
                HesapTipi::Gelir
            } else {
                HesapTipi::Gider
            }
        }
        7 => HesapTipi::Maliyet,
        9 => HesapTipi::Nazim,
        _ => return None,
    };

    // Standart seedde 3 hane = yaprak (postable). 1-2 hane = sınıf/grup başlığı.
    // Muavin eklenince ana hesap yaprak olmaktan çıkar (HesapPlani yönetir).
    let yaprak = kod.len() == 3 && !kod.contains('.');

    Some(Hesap {
        id: kod.to_string(),
        kod: kod.to_string(),
        ad: ad.to_string(),
        tip,
        dogasi,
        yaprak,
        aktif: true,
    })
}

/// Ana hesabın altına muavin (alt hesap) üretir. Kod = "ana.alt" (ör. 120 + 01 → 120.01).
/// Tip/doğa ana hesaptan miras; muavin daima yaprak. Ana hesap, muavin eklenince
/// yaprak olmaktan çıkmalı (postlamada yalnız yaprağa izin verilir — çağıran/HesapPlani yönetir).
pub fn muavin_olustur(ana: &Hesap, alt_kod: &str, ad: &str) -> Hesap {
    let kod = format!("{}.{}", ana.kod, alt_kod);
    Hesap {
        id: kod.clone(),
        kod,
        ad: ad.to_string(),
        tip: ana.tip,
        dogasi: ana.dogasi,
        yaprak: true,
        aktif: true,
    }
}

/// CSV içeriğini (kod,ad — başlık satırı atlanır) Hesap listesine çevirir.
/// İsimde virgül olabileceği için yalnızca İLK virgülden böler.
pub fn csv_yukle(icerik: &str) -> Vec<Hesap> {
    icerik
        .lines()
        .skip(1) // başlık: kod,ad
        .filter_map(|satir| {
            let (kod, ad) = satir.split_once(',')?;
            hesap_olustur(kod, ad)
        })
        .collect()
}
