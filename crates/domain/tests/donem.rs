//! Dönem kapanış/açılış virman testleri.

use domain::defter::Defter;
use domain::donem::{acilis, donem_sonucu, kapanis};
use domain::fis::{Fis, FisDurum, FisTipi, Tarih, YevmiyeSatiri};
use domain::Kurus;

fn b(kod: &str, t: i64) -> YevmiyeSatiri {
    YevmiyeSatiri::borc(kod.to_string(), Kurus::yeni(t))
}
fn a(kod: &str, t: i64) -> YevmiyeSatiri {
    YevmiyeSatiri::alacak(kod.to_string(), Kurus::yeni(t))
}
fn kesin(sira: u64, satirlar: Vec<YevmiyeSatiri>) -> Fis {
    let mut f = Fis::taslak(FisTipi::Mahsup, Tarih::yeni(2026, 12, 31), satirlar);
    f.seri = Some("MAH".to_string());
    f.sira = Some(sira);
    f.durum = FisDurum::Kesin;
    f
}

fn dengeli(f: &Fis) -> bool {
    f.toplam_borc() == f.toplam_alacak()
}

fn kar_defteri() -> Defter {
    let mut d = Defter::yeni();
    // Satış: 100 B 18.000 · 600 A 15.000 · 391 A 3.000
    d.ekle(kesin(1, vec![b("100", 1800000), a("600", 1500000), a("391", 300000)]));
    // SMM: 621 B 10.000 · 153 A 10.000
    d.ekle(kesin(2, vec![b("621", 1000000), a("153", 1000000)]));
    d
}

#[test]
fn donem_sonucu_kar() {
    let d = kar_defteri();
    // gelir 15.000 − gider 10.000 = 5.000 kâr
    assert_eq!(donem_sonucu(&d), 500000);
}

#[test]
fn kapanis_virman_zinciri() {
    let d = kar_defteri();
    let fisler = kapanis(&d, Tarih::yeni(2026, 12, 31), Kurus::yeni(0));
    assert_eq!(fisler.len(), 3, "6→690, 690→692, 692→590");

    // Hepsi dengeli
    for f in &fisler {
        assert!(dengeli(f), "kapanış fişi dengeli olmalı");
    }

    // 1) 600 BORÇ 15.000 + 621 ALACAK 10.000 + 690 ALACAK 5.000
    assert_eq!(fisler[0].toplam_borc(), Kurus::yeni(1500000));
    // 3) 692 → 590 (kâr): 590 alacaklanır
    let f3 = &fisler[2];
    assert!(f3
        .satirlar
        .iter()
        .any(|s| s.hesap_id == "590" && s.alacak == Kurus::yeni(500000)));
}

#[test]
fn kapanis_zarar() {
    let mut d = Defter::yeni();
    // gelir 5.000, gider 8.000 → 3.000 zarar
    d.ekle(kesin(1, vec![b("100", 500000), a("600", 500000)]));
    d.ekle(kesin(2, vec![b("621", 800000), a("153", 800000)]));
    assert_eq!(donem_sonucu(&d), -300000);

    let fisler = kapanis(&d, Tarih::yeni(2026, 12, 31), Kurus::yeni(0));
    let son = fisler.last().unwrap();
    // Zarar: 591 borçlanır
    assert!(son
        .satirlar
        .iter()
        .any(|s| s.hesap_id == "591" && s.borc == Kurus::yeni(300000)));
}

#[test]
fn kapanis_vergi_karsiligi() {
    let d = kar_defteri(); // net (vergi öncesi) kâr = 5.000
    // %25 KV → 1.250 vergi
    let fisler = kapanis(&d, Tarih::yeni(2026, 12, 31), Kurus::yeni(125000));
    assert_eq!(fisler.len(), 4, "6→690, 691/370, 690/691→692, 692→590");
    for f in &fisler {
        assert!(dengeli(f));
    }
    // Vergi karşılığı: 370 Ödenecek vergi alacaklanır
    assert!(fisler.iter().any(|f| f
        .satirlar
        .iter()
        .any(|s| s.hesap_id == "370" && s.alacak == Kurus::yeni(125000))));
    // 590 Dönem net kârı = vergi SONRASI (5.000 − 1.250 = 3.750)
    assert!(fisler.iter().any(|f| f
        .satirlar
        .iter()
        .any(|s| s.hesap_id == "590" && s.alacak == Kurus::yeni(375000))));
}

#[test]
fn acilis_yalniz_bilanco() {
    let mut d = Defter::yeni();
    // 100 B 5.000 · 320 A 5.000 (bilanço) + 600 A 1.000 / 100 B 1.000 (sonuç karışık)
    d.ekle(kesin(1, vec![b("100", 500000), a("320", 500000)]));
    let acilis_fis = acilis(&d, Tarih::yeni(2027, 1, 1));
    assert_eq!(acilis_fis.tip, FisTipi::Acilis);
    // 100 BORÇ, 320 ALACAK (devir); dengeli
    assert!(dengeli(&acilis_fis));
    assert!(acilis_fis
        .satirlar
        .iter()
        .all(|s| s.hesap_id.starts_with('1') || s.hesap_id.starts_with('3')));
}
