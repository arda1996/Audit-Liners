//! Vergi modülü — "Vergiye giden yol"un ilk halkası: aylık KDV mahsubu.
//! KDV alt hesaplarda izlenir (191.10, 191.20, 391.01…); mahsup TÜM alt kırılımı
//! tarar ve her yaprak hesabı TEK TEK kapatır. Dayanak: MSUGT 191/391/190/360.

use crate::defter::Defter;
use crate::fis::{Fis, FisTipi, Tarih, YevmiyeSatiri};
use crate::para::Kurus;

/// `kod`, `onek` hesabının kendisi veya alt kırılımı mı? (191 → 191, 191.10, 191.10.01…)
fn altinda(kod: &str, onek: &str) -> bool {
    kod == onek || kod.starts_with(&format!("{}.", onek))
}

/// Önek altındaki her hesabın net bakiyesini (borç−alacak) hesap bazında toplar.
fn alt_bakiyeler(defter: &Defter, onek: &str) -> Vec<(String, i64)> {
    defter
        .mizan()
        .into_iter()
        .filter(|m| altinda(&m.hesap_id, onek))
        .map(|m| (m.hesap_id, m.borc_bakiye.deger() - m.alacak_bakiye.deger()))
        .filter(|(_, net)| *net != 0)
        .collect()
}

/// Aylık KDV mahsup fişi (taslak); KDV hareketi yoksa None.
/// 191.* ve 190.* (devreden) borç bakiyeleri ile 391.* alacak bakiyeleri hesap hesap kapatılır;
/// fark 360 (ödenecek) veya 190 (yeni devreden) olur.
pub fn kdv_mahsup(defter: &Defter, tarih: Tarih) -> Option<Fis> {
    let ind = alt_bakiyeler(defter, "191"); // borç bakiyeli beklenir
    let dev = alt_bakiyeler(defter, "190");
    let hes = alt_bakiyeler(defter, "391"); // alacak bakiyeli beklenir (net negatif)

    let toplam_ind: i64 = ind.iter().map(|(_, n)| n.max(&0)).sum::<i64>()
        + dev.iter().map(|(_, n)| n.max(&0)).sum::<i64>();
    let toplam_hes: i64 = hes.iter().map(|(_, n)| (-n).max(0)).sum();

    if toplam_ind == 0 && toplam_hes == 0 {
        return None;
    }

    let mut s = Vec::new();
    // 391.* kapat (borç), 191.* ve 190.* kapat (alacak) — her yaprak ayrı satır
    for (kod, net) in &hes {
        if *net < 0 {
            s.push(YevmiyeSatiri::borc(kod.clone(), Kurus::yeni(-net)));
        }
    }
    for (kod, net) in ind.iter().chain(dev.iter()) {
        if *net > 0 {
            s.push(YevmiyeSatiri::alacak(kod.clone(), Kurus::yeni(*net)));
        }
    }
    // Fark
    if toplam_hes > toplam_ind {
        s.push(YevmiyeSatiri::alacak("360".into(), Kurus::yeni(toplam_hes - toplam_ind)));
    } else if toplam_ind > toplam_hes {
        s.push(YevmiyeSatiri::borc("190".into(), Kurus::yeni(toplam_ind - toplam_hes)));
    }

    let mut f = Fis::taslak(FisTipi::Mahsup, tarih, s);
    f.aciklama = "Aylık KDV tahakkuku (191/391 mahsup, alt hesaplar dahil)".into();
    Some(f)
}
