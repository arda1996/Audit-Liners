//! Cari modülü testleri.

use domain::cari::{cari_olustur, CariTip};
use domain::defter::Defter;
use domain::fis::{Fis, FisDurum, FisTipi, Tarih, YevmiyeSatiri};
use domain::hesap_plani::hesap_olustur;
use domain::Kurus;

fn kesin(sira: u64, satirlar: Vec<YevmiyeSatiri>) -> Fis {
    let mut f = Fis::taslak(FisTipi::Mahsup, Tarih::yeni(2026, 3, 1), satirlar);
    f.seri = Some("MAH".to_string());
    f.sira = Some(sira);
    f.durum = FisDurum::Kesin;
    f
}

#[test]
fn cari_olustur_ve_bakiye() {
    let alicilar = hesap_olustur("120", "Alıcılar").unwrap();
    let (cari, muavin) = cari_olustur(&alicilar, "01", "ABC Ltd.", "1234567890");

    assert_eq!(cari.kod, "120.01");
    assert_eq!(cari.tip, CariTip::Alici);
    assert_eq!(cari.unvan, "ABC Ltd.");
    assert_eq!(muavin.ad, "ABC Ltd.");
    assert!(muavin.yaprak);

    // Veresiye satış: 120.01 B 6.000, 600 A 5.000, 391 A 1.000
    let mut d = Defter::yeni();
    d.ekle(kesin(
        1,
        vec![
            YevmiyeSatiri::borc("120.01".to_string(), Kurus::yeni(600000)),
            YevmiyeSatiri::alacak("600".to_string(), Kurus::yeni(500000)),
            YevmiyeSatiri::alacak("391".to_string(), Kurus::yeni(100000)),
        ],
    ));

    assert_eq!(cari.bakiye(&d), (Kurus::yeni(600000), Kurus::yeni(0)));
    assert_eq!(cari.net_bakiye(&d), 600000); // borç bakiye = bize borçlu
    assert_eq!(cari.ekstre(&d).len(), 1);
}

#[test]
fn satici_tipi() {
    let saticilar = hesap_olustur("320", "Satıcılar").unwrap();
    let (cari, _) = cari_olustur(&saticilar, "05", "XYZ A.Ş.", "9876543210");
    assert_eq!(cari.kod, "320.05");
    assert_eq!(cari.tip, CariTip::Satici);
}
