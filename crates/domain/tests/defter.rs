//! Defter motoru testleri — mizan / kebir / bakiye / yevmiye / muavin rollup.

use domain::defter::Defter;
use domain::fis::{Fis, FisDurum, FisTipi, Tarih, YevmiyeSatiri};
use domain::hesap_plani::{hesap_olustur, muavin_olustur};
use domain::Kurus;

fn b(kod: &str, t: i64) -> YevmiyeSatiri {
    YevmiyeSatiri::borc(kod.to_string(), Kurus::yeni(t))
}
fn a(kod: &str, t: i64) -> YevmiyeSatiri {
    YevmiyeSatiri::alacak(kod.to_string(), Kurus::yeni(t))
}

fn kesin(seri: &str, sira: u64, gun: u8, satirlar: Vec<YevmiyeSatiri>) -> Fis {
    let mut f = Fis::taslak(FisTipi::Mahsup, Tarih::yeni(2026, 3, gun), satirlar);
    f.seri = Some(seri.to_string());
    f.sira = Some(sira);
    f.durum = FisDurum::Kesin;
    f
}

fn ornek_defter() -> Defter {
    let mut d = Defter::yeni();
    d.ekle(kesin("MAH", 1, 1, vec![b("153", 1000000), b("191", 200000), a("100", 1200000)]));
    d.ekle(kesin("MAH", 2, 2, vec![b("100", 1800000), a("600", 1500000), a("391", 300000)]));
    d
}

#[test]
fn mizan_dengeli_ve_dogru() {
    let d = ornek_defter();
    assert!(d.dengeli());

    let m = d.mizan();
    let kasa = m.iter().find(|x| x.hesap_id == "100").unwrap();
    assert_eq!(kasa.borc, Kurus::yeni(1800000));
    assert_eq!(kasa.alacak, Kurus::yeni(1200000));
    assert_eq!(kasa.borc_bakiye, Kurus::yeni(600000));
    assert_eq!(kasa.alacak_bakiye, Kurus::yeni(0));

    let satis = m.iter().find(|x| x.hesap_id == "600").unwrap();
    assert_eq!(satis.alacak_bakiye, Kurus::yeni(1500000));

    let tb: i64 = m.iter().map(|x| x.borc.deger()).sum();
    let ta: i64 = m.iter().map(|x| x.alacak.deger()).sum();
    assert_eq!(tb, ta);
}

#[test]
fn hesap_bakiyesi_dogru() {
    let d = ornek_defter();
    let (borc, alacak) = d.hesap_bakiyesi("100");
    assert_eq!(borc, Kurus::yeni(1800000));
    assert_eq!(alacak, Kurus::yeni(1200000));
}

#[test]
fn kebir_yuruyen_bakiye() {
    let d = ornek_defter();
    let k = d.kebir("100");
    assert_eq!(k.len(), 2);
    assert_eq!(k[0].yuruyen_bakiye, -1200000);
    assert_eq!(k[0].fis_no, "MAH-1");
    assert_eq!(k[1].yuruyen_bakiye, 600000);
    assert_eq!(k[1].fis_no, "MAH-2");
}

#[test]
fn yevmiye_tarih_sirasinda() {
    let d = ornek_defter();
    let y = d.yevmiye();
    assert_eq!(y.len(), 2);
    assert_eq!(y[0].sira, Some(1));
    assert_eq!(y[1].sira, Some(2));
}

// ---- Adım B: Muavin + rollup ----
#[test]
fn muavin_rollup() {
    // 120 Alıcılar altında iki cari muavin
    let alicilar = hesap_olustur("120", "Alıcılar").unwrap();
    let m1 = muavin_olustur(&alicilar, "01", "ABC Ltd.");
    let m2 = muavin_olustur(&alicilar, "02", "XYZ A.Ş.");
    assert_eq!(m1.kod, "120.01");
    assert_eq!(m2.kod, "120.02");
    assert!(m1.yaprak);

    // Muavinlere kayıt: 120.01 B 3.000, 120.02 B 2.000, karşı 100 A 5.000
    let mut d = Defter::yeni();
    d.ekle(kesin("MAH", 1, 1, vec![b("120.01", 300000), b("120.02", 200000), a("100", 500000)]));

    // Tek muavin bakiyesi
    assert_eq!(d.hesap_bakiyesi("120.01"), (Kurus::yeni(300000), Kurus::yeni(0)));
    // Ana hesaba rollup: 120 = 120.01 + 120.02
    assert_eq!(d.bakiye_rollup("120"), (Kurus::yeni(500000), Kurus::yeni(0)));
    // Sınıf rollup'ı da çalışır (1 → tüm 1xx): burada sadece 120.* var
    assert_eq!(d.bakiye_rollup("12"), (Kurus::yeni(500000), Kurus::yeni(0)));
}
