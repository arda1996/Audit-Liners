//! T1–T11 — 02-kayit-kurallari.md test senaryolarının birebir karşılığı.

use std::collections::HashMap;

use domain::*;

// ---- yardımcılar ----

fn hesap(kod: &str, tip: HesapTipi, dogasi: HesapDogasi, yaprak: bool) -> Hesap {
    Hesap {
        id: kod.to_string(),
        kod: kod.to_string(),
        ad: kod.to_string(),
        tip,
        dogasi,
        yaprak,
        aktif: true,
    }
}

fn standart_hesaplar() -> HashMap<HesapId, Hesap> {
    let mut m = HashMap::new();
    m.insert("100".into(), hesap("100", HesapTipi::Aktif, HesapDogasi::Borc, true)); // Kasa
    m.insert("320".into(), hesap("320", HesapTipi::Pasif, HesapDogasi::Alacak, true)); // Satıcılar
    m.insert("600".into(), hesap("600", HesapTipi::Gelir, HesapDogasi::Alacak, true)); // Satışlar
    m.insert("770".into(), hesap("770", HesapTipi::Gider, HesapDogasi::Borc, true)); // Gider
    m.insert("12".into(), hesap("12", HesapTipi::Aktif, HesapDogasi::Borc, false)); // grup — yaprak DEĞİL
    m
}

fn acik_donem() -> Donem {
    Donem {
        durum: DonemDurum::Acik,
        baslangic: Tarih::yeni(2026, 1, 1),
        bitis: Tarih::yeni(2026, 12, 31),
    }
}

fn ornek_dayanak() -> Dayanak {
    Dayanak {
        belge_tipi: BelgeTipi::Makbuz,
        belge_no: "M-1".to_string(),
        belge_tarihi: Tarih::yeni(2026, 3, 1),
    }
}

fn tarih() -> Tarih {
    Tarih::yeni(2026, 3, 1)
}

fn b(kod: &str, t: i64) -> YevmiyeSatiri {
    YevmiyeSatiri::borc(kod.to_string(), Kurus::yeni(t))
}
fn a(kod: &str, t: i64) -> YevmiyeSatiri {
    YevmiyeSatiri::alacak(kod.to_string(), Kurus::yeni(t))
}

#[test]
fn t1_dengeli_fis_kesinlesir() {
    let hesaplar = standart_hesaplar();
    let donem = acik_donem();
    let ctx = Baglam { hesaplar: &hesaplar, donem: &donem };
    let mut sayac = SeriSayac::yeni();

    let mut fis = Fis::taslak(FisTipi::Tahsil, tarih(), vec![b("100", 1000), a("600", 1000)])
        .ile_dayanak(ornek_dayanak());

    assert!(kesinlestir(&mut fis, &ctx, &mut sayac).is_ok());
    assert_eq!(fis.durum, FisDurum::Kesin);
    assert_eq!(fis_no(&fis).as_deref(), Some("TAH-1"));
}

#[test]
fn t2_denge_bozuk() {
    let hesaplar = standart_hesaplar();
    let donem = acik_donem();
    let ctx = Baglam { hesaplar: &hesaplar, donem: &donem };

    let fis = Fis::taslak(FisTipi::Mahsup, tarih(), vec![b("100", 1000), a("600", 900)]);
    assert_eq!(
        dogrula(&fis, &ctx),
        Err(KayitHatasi::DengeBozuk { borc: 1000, alacak: 900 })
    );
}

#[test]
fn t3_yetersiz_satir() {
    let hesaplar = standart_hesaplar();
    let donem = acik_donem();
    let ctx = Baglam { hesaplar: &hesaplar, donem: &donem };

    let fis = Fis::taslak(FisTipi::Mahsup, tarih(), vec![b("100", 1000)]);
    assert_eq!(dogrula(&fis, &ctx), Err(KayitHatasi::YetersizSatir));
}

#[test]
fn t4_gecersiz_tutar() {
    let hesaplar = standart_hesaplar();
    let donem = acik_donem();
    let ctx = Baglam { hesaplar: &hesaplar, donem: &donem };

    let bozuk = YevmiyeSatiri {
        hesap_id: "100".into(),
        borc: Kurus::yeni(500),
        alacak: Kurus::yeni(500),
        masraf_yeri_id: None,
        aciklama: String::new(),
    };
    let fis = Fis::taslak(FisTipi::Mahsup, tarih(), vec![bozuk, a("600", 500)]);
    assert_eq!(dogrula(&fis, &ctx), Err(KayitHatasi::GecersizTutar));
}

#[test]
fn t5_yaprak_olmayan_hesap() {
    let hesaplar = standart_hesaplar();
    let donem = acik_donem();
    let ctx = Baglam { hesaplar: &hesaplar, donem: &donem };

    let fis = Fis::taslak(FisTipi::Mahsup, tarih(), vec![b("12", 1000), a("600", 1000)]);
    assert_eq!(dogrula(&fis, &ctx), Err(KayitHatasi::GecersizHesap("12".into())));
}

#[test]
fn t6_donem_kapali() {
    let hesaplar = standart_hesaplar();
    let kapali = Donem {
        durum: DonemDurum::Kapanmis,
        baslangic: Tarih::yeni(2026, 1, 1),
        bitis: Tarih::yeni(2026, 12, 31),
    };
    let ctx = Baglam { hesaplar: &hesaplar, donem: &kapali };

    let fis = Fis::taslak(FisTipi::Mahsup, tarih(), vec![b("100", 1000), a("600", 1000)]);
    assert_eq!(dogrula(&fis, &ctx), Err(KayitHatasi::DonemKapali));
}

#[test]
fn t7_dayanak_tipe_gore() {
    let hesaplar = standart_hesaplar();
    let donem = acik_donem();
    let ctx = Baglam { hesaplar: &hesaplar, donem: &donem };
    let mut sayac = SeriSayac::yeni();

    let tediye = Fis::taslak(FisTipi::Tediye, tarih(), vec![b("770", 1000), a("100", 1000)]);
    assert_eq!(dogrula(&tediye, &ctx), Err(KayitHatasi::DayanakEksik));

    let mut mahsup = Fis::taslak(FisTipi::Mahsup, tarih(), vec![b("100", 1000), a("600", 1000)]);
    assert!(kesinlestir(&mut mahsup, &ctx, &mut sayac).is_ok());
    assert!(mahsup.dayanaksiz);
}

#[test]
fn t8_bosluksuz_numaralama() {
    let hesaplar = standart_hesaplar();
    let donem = acik_donem();
    let ctx = Baglam { hesaplar: &hesaplar, donem: &donem };
    let mut sayac = SeriSayac::yeni();

    let mut f1 = Fis::taslak(FisTipi::Tahsil, tarih(), vec![b("100", 1000), a("600", 1000)])
        .ile_dayanak(ornek_dayanak());
    let mut f2 = Fis::taslak(FisTipi::Tahsil, tarih(), vec![b("100", 2000), a("600", 2000)])
        .ile_dayanak(ornek_dayanak());

    kesinlestir(&mut f1, &ctx, &mut sayac).unwrap();
    kesinlestir(&mut f2, &ctx, &mut sayac).unwrap();

    assert_eq!(fis_no(&f1).as_deref(), Some("TAH-1"));
    assert_eq!(fis_no(&f2).as_deref(), Some("TAH-2"));
    // Yevmiye madde no: TÜM fişlerde müteselsil (VUK)
    assert_eq!(f1.yevmiye_no, Some(1));
    assert_eq!(f2.yevmiye_no, Some(2));

    let mut f3 = Fis::taslak(FisTipi::Mahsup, tarih(), vec![b("100", 500), a("600", 500)]);
    kesinlestir(&mut f3, &ctx, &mut sayac).unwrap();
    assert_eq!(fis_no(&f3).as_deref(), Some("MAH-1")); // seri ayrı
    assert_eq!(f3.yevmiye_no, Some(3)); // yevmiye müteselsil devam
}

#[test]
fn t9_kesin_fis_degismez() {
    let hesaplar = standart_hesaplar();
    let donem = acik_donem();
    let ctx = Baglam { hesaplar: &hesaplar, donem: &donem };
    let mut sayac = SeriSayac::yeni();

    let mut fis = Fis::taslak(FisTipi::Tahsil, tarih(), vec![b("100", 1000), a("600", 1000)])
        .ile_dayanak(ornek_dayanak());
    kesinlestir(&mut fis, &ctx, &mut sayac).unwrap();

    assert_eq!(dogrula(&fis, &ctx), Err(KayitHatasi::ZatenKesin));
}

#[test]
fn t10_iptal_ters_kayit() {
    let kaynak = Fis::taslak(FisTipi::Tahsil, tarih(), vec![b("100", 1000), a("600", 1000)]);
    let fark_tarihi = Tarih::yeni(2026, 7, 15); // hata bu tarihte fark edildi
    let st = iptal_fisi(&kaynak, 42, "TAH-1", fark_tarihi, "mükerrer kayıt");

    assert_eq!(st.tip, FisTipi::Mahsup);
    assert_eq!(st.iptal_kaynak_id, Some(42));
    // VUK 217: iptal fişi FARK EDİLME tarihiyle atılır, kaynak tarihle DEĞİL
    assert_eq!(st.tarih, fark_tarihi);
    // Açıklama standardı: kaynak no + VUK 217 + gerekçe
    assert!(st.aciklama.contains("TAH-1") && st.aciklama.contains("217") && st.aciklama.contains("mükerrer"));
    // Dayanak = kaynak fiş (dayanaksız işaretlenmemeli)
    assert!(st.dayanak.as_ref().unwrap().belge_no.contains("TAH-1"));
    assert_eq!(st.toplam_borc(), kaynak.toplam_alacak());
    assert_eq!(st.toplam_alacak(), kaynak.toplam_borc());
    assert_eq!(st.toplam_borc(), st.toplam_alacak());
}

#[test]
fn t11_acilis_devri() {
    let hesaplar = standart_hesaplar();
    let donem27 = Donem {
        durum: DonemDurum::Acik,
        baslangic: Tarih::yeni(2027, 1, 1),
        bitis: Tarih::yeni(2027, 12, 31),
    };
    let ctx = Baglam { hesaplar: &hesaplar, donem: &donem27 };
    let mut sayac = SeriSayac::yeni();

    let bakiyeler = vec![
        ("100".to_string(), Kurus::yeni(5000), Kurus::SIFIR),
        ("320".to_string(), Kurus::SIFIR, Kurus::yeni(5000)),
    ];
    let mut acilis = acilis_fisi(&bakiyeler, Tarih::yeni(2027, 1, 1));

    assert_eq!(acilis.tip, FisTipi::Acilis);
    assert_eq!(acilis.toplam_borc(), acilis.toplam_alacak());
    assert!(kesinlestir(&mut acilis, &ctx, &mut sayac).is_ok());
    assert_eq!(fis_no(&acilis).as_deref(), Some("AC-1"));
    assert!(!acilis.dayanaksiz);
}
