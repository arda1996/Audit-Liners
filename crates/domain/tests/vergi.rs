//! KDV aylık mahsup testleri.

use domain::defter::Defter;
use domain::fis::{Fis, FisDurum, FisTipi, Tarih, YevmiyeSatiri};
use domain::vergi::kdv_mahsup;
use domain::Kurus;

fn b(k: &str, t: i64) -> YevmiyeSatiri { YevmiyeSatiri::borc(k.into(), Kurus::yeni(t)) }
fn a(k: &str, t: i64) -> YevmiyeSatiri { YevmiyeSatiri::alacak(k.into(), Kurus::yeni(t)) }
fn kesin(sira: u64, s: Vec<YevmiyeSatiri>) -> Fis {
    let mut f = Fis::taslak(FisTipi::Mahsup, Tarih::yeni(2026, 3, 15), s);
    f.seri = Some("MAH".into()); f.sira = Some(sira); f.durum = FisDurum::Kesin; f
}

#[test]
fn odenecek_kdv_cikar() {
    // İndirilecek 2.000 < Hesaplanan 3.000 → 1.000 ödenecek (360)
    let mut d = Defter::yeni();
    d.ekle(kesin(1, vec![b("153", 1000000), b("191", 200000), a("100", 1200000)]));
    d.ekle(kesin(2, vec![b("100", 1800000), a("600", 1500000), a("391", 300000)]));

    let f = kdv_mahsup(&d, Tarih::yeni(2026, 3, 31)).unwrap();
    assert_eq!(f.toplam_borc(), f.toplam_alacak());
    assert!(f.satirlar.iter().any(|s| s.hesap_id == "391" && s.borc == Kurus::yeni(300000)));
    assert!(f.satirlar.iter().any(|s| s.hesap_id == "191" && s.alacak == Kurus::yeni(200000)));
    assert!(f.satirlar.iter().any(|s| s.hesap_id == "360" && s.alacak == Kurus::yeni(100000)));
}

#[test]
fn devreden_kdv_cikar() {
    // İndirilecek 2.000 > Hesaplanan 1.200 → 800 devreden (190 borç)
    let mut d = Defter::yeni();
    d.ekle(kesin(1, vec![b("153", 1000000), b("191", 200000), a("100", 1200000)]));
    d.ekle(kesin(2, vec![b("100", 720000), a("600", 600000), a("391", 120000)]));

    let f = kdv_mahsup(&d, Tarih::yeni(2026, 3, 31)).unwrap();
    assert_eq!(f.toplam_borc(), f.toplam_alacak());
    assert!(f.satirlar.iter().any(|s| s.hesap_id == "190" && s.borc == Kurus::yeni(80000)));
    assert!(!f.satirlar.iter().any(|s| s.hesap_id == "360"));
}

#[test]
fn onceki_devreden_dahil() {
    // Önceki aydan 190'da 500 devreden + bu ay 191'de 1.000; hesaplanan 2.000 → ödenecek 500
    let mut d = Defter::yeni();
    d.ekle(kesin(1, vec![b("190", 50000), a("100", 50000)])); // devreden (basitleştirilmiş taşıma)
    d.ekle(kesin(2, vec![b("153", 500000), b("191", 100000), a("100", 600000)]));
    d.ekle(kesin(3, vec![b("100", 1200000), a("600", 1000000), a("391", 200000)]));

    let f = kdv_mahsup(&d, Tarih::yeni(2026, 4, 30)).unwrap();
    assert_eq!(f.toplam_borc(), f.toplam_alacak());
    // eski 190 kapanır (alacak 500), fark 360'a 500
    assert!(f.satirlar.iter().any(|s| s.hesap_id == "190" && s.alacak == Kurus::yeni(50000)));
    assert!(f.satirlar.iter().any(|s| s.hesap_id == "360" && s.alacak == Kurus::yeni(50000)));
}

#[test]
fn alt_hesaplar_tek_tek_kapanir() {
    // KDV alt hesaplarda: 191.10 (%10) 500 + 191.20 (%20) 1.500; 391.20 alacak 2.500
    let mut d = Defter::yeni();
    d.ekle(kesin(1, vec![b("153", 5000000), b("191.10", 50000), b("191.20", 150000), a("100", 5200000)]));
    d.ekle(kesin(2, vec![b("100", 15000000), a("600", 12500000), a("391.20", 250000)]));

    let f = kdv_mahsup(&d, Tarih::yeni(2026, 3, 31)).unwrap();
    assert_eq!(f.toplam_borc(), f.toplam_alacak());
    // her alt hesap kendi satırıyla kapanır
    assert!(f.satirlar.iter().any(|s| s.hesap_id == "391.20" && s.borc == Kurus::yeni(250000)));
    assert!(f.satirlar.iter().any(|s| s.hesap_id == "191.10" && s.alacak == Kurus::yeni(50000)));
    assert!(f.satirlar.iter().any(|s| s.hesap_id == "191.20" && s.alacak == Kurus::yeni(150000)));
    // fark: 2.500 − 2.000 = 500 ödenecek
    assert!(f.satirlar.iter().any(|s| s.hesap_id == "360" && s.alacak == Kurus::yeni(50000)));
}

#[test]
fn hareket_yoksa_none() {
    let d = Defter::yeni();
    assert!(kdv_mahsup(&d, Tarih::yeni(2026, 3, 31)).is_none());
}
