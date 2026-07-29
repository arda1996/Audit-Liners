//! Mali tablo testleri — bilanço denkliği + gelir tablosu dönem kârı.

use domain::defter::Defter;
use domain::donem::donem_sonucu;
use domain::fis::{Fis, FisDurum, FisTipi, Tarih, YevmiyeSatiri};
use domain::mali_tablo::{bilanco, gelir_tablosu};
use domain::Kurus;

fn b(kod: &str, t: i64) -> YevmiyeSatiri {
    YevmiyeSatiri::borc(kod.to_string(), Kurus::yeni(t))
}
fn a(kod: &str, t: i64) -> YevmiyeSatiri {
    YevmiyeSatiri::alacak(kod.to_string(), Kurus::yeni(t))
}
fn kesin(sira: u64, satirlar: Vec<YevmiyeSatiri>) -> Fis {
    let mut f = Fis::taslak(FisTipi::Mahsup, Tarih::yeni(2026, 6, 1), satirlar);
    f.seri = Some("MAH".to_string());
    f.sira = Some(sira);
    f.durum = FisDurum::Kesin;
    f
}

fn defter() -> Defter {
    let mut d = Defter::yeni();
    // Sermaye konulması: 102 Banka B 100.000 · 500 Sermaye A 100.000
    d.ekle(kesin(1, vec![b("102", 10000000), a("500", 10000000)]));
    // Peşin satış: 100 B 18.000 · 600 A 15.000 · 391 A 3.000
    d.ekle(kesin(2, vec![b("100", 1800000), a("600", 1500000), a("391", 300000)]));
    // SMM: 621 B 10.000 · 153 A 10.000
    d.ekle(kesin(3, vec![b("621", 1000000), a("153", 1000000)]));
    d
}

#[test]
fn bilanco_denk() {
    let bil = bilanco(&defter());
    // Aktif = Pasif (dönem kârı öz kaynağa dahil)
    assert_eq!(bil.aktif_toplam, bil.pasif_toplam);
    assert_eq!(bil.donem_kari, 500000); // 15.000 − 10.000
}

#[test]
fn gelir_tablosu_donem_kari() {
    let d = defter();
    let gt = gelir_tablosu(&d);
    assert_eq!(gt.brut_satislar, 1500000);
    assert_eq!(gt.satislarin_maliyeti, 1000000);
    assert_eq!(gt.brut_satis_kari, 500000);
    // Gelir tablosu dönem kârı = dönem sonucu
    assert_eq!(gt.donem_kari, donem_sonucu(&d));
    assert_eq!(gt.donem_kari, 500000);
    // Vergi karşılığı yokken net kâr = vergi öncesi kâr
    assert_eq!(gt.vergi_karsiligi, 0);
    assert_eq!(gt.donem_net_kari, 500000);
}
