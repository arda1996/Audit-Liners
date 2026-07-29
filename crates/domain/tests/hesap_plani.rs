//! TDHP yükleme + koddan türetme doğrulaması.

use domain::hesap::{HesapDogasi, HesapTipi};
use domain::hesap_plani::{csv_yukle, hesap_olustur};

const TDHP: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/tdhp.csv"));

fn bul(kod: &str) -> domain::Hesap {
    hesap_olustur(kod, ad_of(kod)).expect("hesap üretilemedi")
}

// test için ad: doğa kuralı ada (-) bakar; bu yardımcı gerçek adları taklit eder
fn ad_of(kod: &str) -> &'static str {
    match kod {
        "103" => "Verilen Çekler ve Ödeme Emirleri (-)",
        "591" => "Dönem Net Zararı (-)",
        "621" => "Satılan Ticari Mallar Maliyeti (-)",
        _ => "X",
    }
}

#[test]
fn tum_plan_yuklenir() {
    let hesaplar = csv_yukle(TDHP);
    let yaprak = hesaplar.iter().filter(|h| h.yaprak).count();
    // Sabit sayı iddia ETMİYORUZ: plana TFRS/OCI hesapları (ör. 549.90) sonradan eklenebiliyor;
    // sabit sayı, meşru ekleme yapıldığında testi kırıp gerçek bir hata varmış izlenimi veriyordu.
    // Değişmez olan: MSUGT çekirdeği eksiksiz + kod tekilliği + başlıkların yaprak olmaması.
    assert!(yaprak >= 261, "yaprak hesap sayısı MSUGT çekirdeğinin (261) altına düşemez: {yaprak}");

    // Kod tekilliği — mükerrer kod, kayıtların yanlış hesaba düşmesine yol açar.
    let mut kodlar: Vec<&str> = hesaplar.iter().map(|h| h.kod.as_str()).collect();
    kodlar.sort_unstable();
    let tekil = kodlar.len();
    kodlar.dedup();
    assert_eq!(kodlar.len(), tekil, "hesap planında mükerrer kod var");

    // sınıf/grup başlıkları da var (yaprak değil)
    assert!(hesaplar.iter().any(|h| h.kod == "1" && !h.yaprak));
    assert!(hesaplar.iter().any(|h| h.kod == "10" && !h.yaprak));

    // Motorların dayandığı çekirdek hesaplar yaprak (kayıt yapılabilir) olmalı.
    for kod in ["100", "102", "120", "153", "191", "320", "391", "600"] {
        assert!(
            hesaplar.iter().any(|h| h.kod == kod && h.yaprak),
            "{kod} yaprak hesap olarak bulunmalı (tahsis/aktarım motoru buna kayıt atıyor)"
        );
    }
}

#[test]
fn dogalar_dogru_turetilir() {
    // 100 Kasa — aktif, borç, yaprak
    let h = hesap_olustur("100", "Kasa").unwrap();
    assert_eq!(h.tip, HesapTipi::Aktif);
    assert_eq!(h.dogasi, HesapDogasi::Borc);
    assert!(h.yaprak);

    // 320 Satıcılar — pasif, alacak
    let h = hesap_olustur("320", "Satıcılar").unwrap();
    assert_eq!(h.tip, HesapTipi::Pasif);
    assert_eq!(h.dogasi, HesapDogasi::Alacak);

    // 600 Yurtiçi Satışlar — gelir, alacak
    let h = hesap_olustur("600", "Yurtiçi Satışlar").unwrap();
    assert_eq!(h.tip, HesapTipi::Gelir);
    assert_eq!(h.dogasi, HesapDogasi::Alacak);

    // 710 Direkt İlk Madde — maliyet, borç
    let h = hesap_olustur("710", "Direkt İlk Madde ve Malzeme Giderleri").unwrap();
    assert_eq!(h.tip, HesapTipi::Maliyet);
    assert_eq!(h.dogasi, HesapDogasi::Borc);
}

#[test]
fn kontra_hesaplar_ters_doga() {
    // 103 (-) aktif kontra → alacak
    assert_eq!(bul("103").dogasi, HesapDogasi::Alacak);
    assert_eq!(bul("103").tip, HesapTipi::Aktif);

    // 591 (-) pasif kontra → borç
    assert_eq!(bul("591").dogasi, HesapDogasi::Borc);

    // 621 (-) gelir tablosu kontra → borç, gider olarak sınıflanır
    assert_eq!(bul("621").dogasi, HesapDogasi::Borc);
    assert_eq!(bul("621").tip, HesapTipi::Gider);
}

#[test]
fn yansitma_istisnasi_alacak() {
    // 701 yansıtma — adında (-) yok ama ALACAK çalışır
    let h = hesap_olustur("701", "Maliyet Muhasebesi Yansıtma Hesabı").unwrap();
    assert_eq!(h.dogasi, HesapDogasi::Alacak);
    assert_eq!(h.tip, HesapTipi::Maliyet);

    // 710 (yansıtma değil) → borç
    let h = hesap_olustur("710", "Direkt İlk Madde ve Malzeme Giderleri").unwrap();
    assert_eq!(h.dogasi, HesapDogasi::Borc);
}
