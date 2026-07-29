//! Dönem kapanış/açılış virman operasyonları. Defter mizanından otomatik kapanış fişleri üretir.
//! Sıra (donem-kapanis-acilis.md, muhasebenet.net ile doğrulandı):
//!  1) Gelir/gider (6, 69x hariç) → 690   2) 690 → 692   3) 692 → 590/591
//! (7→6 yansıtma önceden yapılmış varsayılır; bilanço kapanış fişi ayrı adım.)

use crate::defter::Defter;
use crate::fis::{Fis, FisTipi, Tarih, YevmiyeSatiri};
use crate::para::Kurus;

fn sinif(kod: &str) -> u8 {
    kod.as_bytes().first().map(|b| b.wrapping_sub(b'0')).unwrap_or(99)
}
fn grup(kod: &str) -> &str {
    if kod.len() >= 2 {
        &kod[..2]
    } else {
        kod
    }
}

/// Dönem sonucu (net kâr/zarar, kuruş): Σ(gelir alacak − gider borç) sınıf 6, 69x hariç.
/// Pozitif = kâr, negatif = zarar.
pub fn donem_sonucu(defter: &Defter) -> i64 {
    let mut net = 0i64;
    for ms in defter.mizan() {
        if sinif(&ms.hesap_id) == 6 && grup(&ms.hesap_id) != "69" {
            net += ms.alacak_bakiye.deger() - ms.borc_bakiye.deger();
        }
    }
    net
}

/// Kapanış virman fişleri (taslak). Her biri dengelidir; gözden geçirilip kesinleştirilir.
/// `vergi_karsiligi`: dönem kârı üzerinden hesaplanan kurumlar/gelir vergisi (kuruş).
/// Vergi tutarını dışarıdan alır (mali kâr/KKEG hesabı vergi motorunun işi); 0 = vergi yok.
pub fn kapanis(defter: &Defter, tarih: Tarih, vergi_karsiligi: Kurus) -> Vec<Fis> {
    let mut fisler = Vec::new();
    let mizan = defter.mizan();

    // 1) Gelir/gider (6, 69x hariç) → 690
    let mut satirlar = Vec::new();
    let mut net = 0i64;
    for ms in &mizan {
        if sinif(&ms.hesap_id) == 6 && grup(&ms.hesap_id) != "69" {
            let bb = ms.borc_bakiye.deger();
            let ab = ms.alacak_bakiye.deger();
            if bb > 0 {
                // gider (borç bakiyeli) → kapatmak için ALACAK
                satirlar.push(YevmiyeSatiri::alacak(ms.hesap_id.clone(), Kurus::yeni(bb)));
                net -= bb;
            } else if ab > 0 {
                // gelir (alacak bakiyeli) → kapatmak için BORÇ
                satirlar.push(YevmiyeSatiri::borc(ms.hesap_id.clone(), Kurus::yeni(ab)));
                net += ab;
            }
        }
    }
    if !satirlar.is_empty() {
        if net > 0 {
            satirlar.push(YevmiyeSatiri::alacak("690".to_string(), Kurus::yeni(net)));
        } else if net < 0 {
            satirlar.push(YevmiyeSatiri::borc("690".to_string(), Kurus::yeni(-net)));
        }
        let mut f = Fis::taslak(FisTipi::Kapanis, tarih, satirlar);
        f.aciklama = "Gelir/gider hesaplarının 690'a devri".to_string();
        fisler.push(f);
    }

    // Vergi yalnız kârda ve kârı aşmayacak şekilde uygulanır.
    let vergi = if net > 0 {
        vergi_karsiligi.deger().max(0).min(net)
    } else {
        0
    };

    // 2) Vergi karşılığı: 691 BORÇ / 370 ALACAK (kurumlar/gelir vergisi tahakkuku)
    if vergi > 0 {
        let mut f = Fis::taslak(
            FisTipi::Kapanis,
            tarih,
            vec![
                YevmiyeSatiri::borc("691".to_string(), Kurus::yeni(vergi)),
                YevmiyeSatiri::alacak("370".to_string(), Kurus::yeni(vergi)),
            ],
        );
        f.aciklama = "Dönem kârı vergi karşılığı (691/370)".to_string();
        fisler.push(f);
    }

    // 3) 690 (ve 691) → 692
    if net != 0 {
        let s = if net > 0 {
            let mut v = vec![YevmiyeSatiri::borc("690".to_string(), Kurus::yeni(net))];
            if vergi > 0 {
                v.push(YevmiyeSatiri::alacak("691".to_string(), Kurus::yeni(vergi)));
            }
            v.push(YevmiyeSatiri::alacak("692".to_string(), Kurus::yeni(net - vergi)));
            v
        } else {
            vec![
                YevmiyeSatiri::borc("692".to_string(), Kurus::yeni(-net)),
                YevmiyeSatiri::alacak("690".to_string(), Kurus::yeni(-net)),
            ]
        };
        let mut f = Fis::taslak(FisTipi::Kapanis, tarih, s);
        f.aciklama = "690/691 → 692 devri".to_string();
        fisler.push(f);
    }

    // 4) 692 → 590 (net kâr) / 591 (net zarar) — vergi SONRASI net
    let net_sonuc = net - vergi;
    if net_sonuc > 0 {
        let mut f = Fis::taslak(
            FisTipi::Kapanis,
            tarih,
            vec![
                YevmiyeSatiri::borc("692".to_string(), Kurus::yeni(net_sonuc)),
                YevmiyeSatiri::alacak("590".to_string(), Kurus::yeni(net_sonuc)),
            ],
        );
        f.aciklama = "692 → 590 (dönem net kârı)".to_string();
        fisler.push(f);
    } else if net_sonuc < 0 {
        let mut f = Fis::taslak(
            FisTipi::Kapanis,
            tarih,
            vec![
                YevmiyeSatiri::borc("591".to_string(), Kurus::yeni(-net_sonuc)),
                YevmiyeSatiri::alacak("692".to_string(), Kurus::yeni(-net_sonuc)),
            ],
        );
        f.aciklama = "692 → 591 (dönem net zararı)".to_string();
        fisler.push(f);
    }

    fisler
}

/// Bilanço kapanış fişi: kalan bilanço (1-5) bakiyelerini sıfırlar (aktif ALACAK, pasif BORÇ).
/// Not: kapanis() virmanları DEFTERE eklendikten sonra çağrılmalı (690/692 sıfır, 590 dolu olsun).
pub fn bilanco_kapanis(defter: &Defter, tarih: Tarih) -> Fis {
    let mut satirlar = Vec::new();
    for ms in defter.mizan() {
        let s = sinif(&ms.hesap_id);
        if (1..=5).contains(&s) {
            let bb = ms.borc_bakiye.deger();
            let ab = ms.alacak_bakiye.deger();
            if bb > 0 {
                satirlar.push(YevmiyeSatiri::alacak(ms.hesap_id.clone(), Kurus::yeni(bb)));
            } else if ab > 0 {
                satirlar.push(YevmiyeSatiri::borc(ms.hesap_id.clone(), Kurus::yeni(ab)));
            }
        }
    }
    let mut f = Fis::taslak(FisTipi::Kapanis, tarih, satirlar);
    f.aciklama = "Bilanço hesaplarının kapanışı".to_string();
    f
}

/// Açılış fişi: bilanço bakiyelerini yeni dönemde geri açar (aktif BORÇ, pasif ALACAK).
/// Önceki dönemin (kapanış öncesi) bilanço mizanından üretilir; sonuç hesapları (6,7) gelmez.
pub fn acilis(onceki_defter: &Defter, tarih: Tarih) -> Fis {
    let mut satirlar = Vec::new();
    for ms in onceki_defter.mizan() {
        let s = sinif(&ms.hesap_id);
        if (1..=5).contains(&s) {
            let bb = ms.borc_bakiye.deger();
            let ab = ms.alacak_bakiye.deger();
            if bb > 0 {
                satirlar.push(YevmiyeSatiri::borc(ms.hesap_id.clone(), Kurus::yeni(bb)));
            } else if ab > 0 {
                satirlar.push(YevmiyeSatiri::alacak(ms.hesap_id.clone(), Kurus::yeni(ab)));
            }
        }
    }
    let mut f = Fis::taslak(FisTipi::Acilis, tarih, satirlar);
    f.aciklama = "Açılış (devir)".to_string();
    f
}
