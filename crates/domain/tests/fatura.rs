//! Fatura → otomatik fiş testleri (e-fatura yaşam döngüsü).

use domain::fatura::{fatura_fisi, Fatura, FaturaDurum, FaturaSatiri, FaturaYonu};
use domain::fis::Tarih;
use domain::Kurus;

fn satis_faturasi(durum: FaturaDurum) -> Fatura {
    Fatura {
        no: "EF-2026-001".into(),
        yon: FaturaYonu::Satis,
        cari_kod: "120.01".into(),
        tarih: Tarih::yeni(2026, 3, 10),
        satirlar: vec![
            FaturaSatiri { aciklama: "Ürün A".into(), hesap_kod: "600.01".into(), miktar: 10, birim_fiyat: Kurus::yeni(10000), kdv_orani: 20, istisna_kodu: None },
            FaturaSatiri { aciklama: "Ürün B (ihracat)".into(), hesap_kod: "601.01".into(), miktar: 5, birim_fiyat: Kurus::yeni(20000), kdv_orani: 20, istisna_kodu: Some("11/1-a".into()) },
        ],
        durum,
        efatura_uuid: None,
        belge_ref: None,
    }
}

#[test]
fn kesinlesmemis_faturadan_fis_uretilmez() {
    assert!(fatura_fisi(&satis_faturasi(FaturaDurum::Taslak)).is_none());
    assert!(fatura_fisi(&satis_faturasi(FaturaDurum::Gonderildi)).is_none());
    assert!(fatura_fisi(&satis_faturasi(FaturaDurum::Reddedildi)).is_none());
}

#[test]
fn satis_fisi_istisnali() {
    let f = satis_faturasi(FaturaDurum::Kesinlesti);
    // matrah: 10×100 + 5×200 = 2.000; KDV yalnız Ürün A: 1.000×%20 = 200; toplam 2.200
    assert_eq!(f.matrah(), 200000);
    assert_eq!(f.toplam_kdv(), 20000);

    let fis = fatura_fisi(&f).unwrap();
    assert_eq!(fis.toplam_borc(), fis.toplam_alacak());
    // 120.01 borç genel toplam
    assert!(fis.satirlar.iter().any(|s| s.hesap_id == "120.01" && s.borc == Kurus::yeni(220000)));
    // KDV oran alt hesabına: 391.20
    assert!(fis.satirlar.iter().any(|s| s.hesap_id == "391.20" && s.alacak == Kurus::yeni(20000)));
    // istisna satırı KDV üretmedi ama açıklamada işaretli
    assert!(fis.satirlar.iter().any(|s| s.hesap_id == "601.01" && s.aciklama.contains("İSTİSNA 11/1-a")));
    // dayanak e-Fatura
    assert_eq!(fis.dayanak.as_ref().unwrap().belge_no, "EF-2026-001");
}

#[test]
fn alis_fisi() {
    let f = Fatura {
        no: "AF-77".into(),
        yon: FaturaYonu::Alis,
        cari_kod: "320.05".into(),
        tarih: Tarih::yeni(2026, 3, 12),
        satirlar: vec![FaturaSatiri { aciklama: "Mal".into(), hesap_kod: "153.01".into(), miktar: 1, birim_fiyat: Kurus::yeni(1000000), kdv_orani: 10, istisna_kodu: None }],
        durum: FaturaDurum::Kesinlesti,
        efatura_uuid: Some("uuid-1".into()),
        belge_ref: Some("arsiv/AF-77.xml".into()),
    };
    let fis = fatura_fisi(&f).unwrap();
    assert_eq!(fis.toplam_borc(), fis.toplam_alacak());
    assert!(fis.satirlar.iter().any(|s| s.hesap_id == "153.01" && s.borc == Kurus::yeni(1000000)));
    assert!(fis.satirlar.iter().any(|s| s.hesap_id == "191.10" && s.borc == Kurus::yeni(100000)));
    assert!(fis.satirlar.iter().any(|s| s.hesap_id == "320.05" && s.alacak == Kurus::yeni(1100000)));
}
