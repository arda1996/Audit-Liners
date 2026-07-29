//! Banka ekstresi bütünlük çekirdeği — D.7 (uyuyan modül; API/UI'a iş akışında gerekince bağlanır).
//!
//! Kaynak fikirler: takipsistemiv2 (bakiye zinciri, 3 katmanlı dedup, 2 katmanlı ters işlem tespiti).
//! Mantık düzeltmeleriyle port edildi:
//!   • Para float DEĞİL — kuruş i64 (kaynaktaki en büyük kusur buydu).
//!   • Ters işlem işaretleme TEK sorumlu fonksiyonda ve İDEMPOTENT (kaynaktaki "çift çağrı" bug'ının kökü).
//!   • Dedup anahtarı deterministik dizge (hash çakışma sınıfı yok); aynı-gün aynı-tutar için sıra ayracı.

use crate::fis::Tarih;
use crate::isim::{isim_skoru, tr_katla};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HareketYonu {
    Giris,
    Cikis,
}

#[derive(Debug, Clone)]
pub struct EkstreHareket {
    pub tarih: Tarih,
    pub aciklama: String,
    /// Kuruş, her zaman pozitif; yön ayrı alanda.
    pub tutar: i64,
    /// İşlem sonrası bakiye (kuruş, işaretli — eksi bakiye mümkün).
    pub bakiye_sonrasi: i64,
    pub yon: HareketYonu,
    /// Bankanın işlem/dekont numarası (varsa; Vakıfbank 17 hane gibi unique olanlar dedup'ın 2. katmanı).
    pub referans_no: Option<String>,
    pub karsi_taraf: Option<String>,
    /// TERS/İADE onaylı işaret — yalnız `ters_isle` yazar.
    pub ters_islem: bool,
}

#[derive(Debug, Clone)]
pub struct Ekstre {
    pub iban: String,
    pub banka: String,
    pub donem_bas: Tarih,
    pub donem_son: Tarih,
    pub acilis_bakiye: i64,
    pub kapanis_bakiye: i64,
    /// Kronolojik (eski → yeni) beklenir; parser DESC veriyorsa çevirmek PARSER'ın işi.
    pub hareketler: Vec<EkstreHareket>,
}

/// Dedup anahtarı — katman 1 (içerik). Aynı gün aynı tutarlı meşru çift işlem `sira` ile ayrışır.
pub fn hareket_anahtari(iban: &str, h: &EkstreHareket, sira: usize) -> String {
    format!(
        "{}|{:02}.{:02}.{:04}|{}|{}|{}|{}",
        iban,
        h.tarih.gun,
        h.tarih.ay,
        h.tarih.yil,
        match h.yon { HareketYonu::Giris => "G", HareketYonu::Cikis => "C" },
        h.tutar,
        h.bakiye_sonrasi,
        sira
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZincirBulgu {
    /// Hareket sırası (0-bazlı); ekstreler arası devir bulgusunda usize::MAX.
    pub sira: usize,
    pub beklenen: i64,
    pub gorulen: i64,
}

/// M13 çekirdeği — ekstre İÇİ bakiye zinciri: açılış + Σ(±tutar) adım adım her `bakiye_sonrasi` ile
/// ve son adım kapanışla tutmalı. Kopukluklar bulgu listesi olarak döner (boş = sağlam).
pub fn zincir_dogrula(e: &Ekstre) -> Vec<ZincirBulgu> {
    let mut bulgular = Vec::new();
    let mut bakiye = e.acilis_bakiye;
    for (i, h) in e.hareketler.iter().enumerate() {
        bakiye += match h.yon {
            HareketYonu::Giris => h.tutar,
            HareketYonu::Cikis => -h.tutar,
        };
        if bakiye != h.bakiye_sonrasi {
            bulgular.push(ZincirBulgu { sira: i, beklenen: bakiye, gorulen: h.bakiye_sonrasi });
            // zinciri bankanın değerinden devam ettir — tek kopukluk N sahte bulgu üretmesin
            bakiye = h.bakiye_sonrasi;
        }
    }
    if bakiye != e.kapanis_bakiye {
        bulgular.push(ZincirBulgu { sira: e.hareketler.len(), beklenen: bakiye, gorulen: e.kapanis_bakiye });
    }
    bulgular
}

/// Ekstreler ARASI devir: önceki kapanış = sonraki açılış olmalı (kronolojik ardışık ekstreler).
pub fn devir_dogrula(onceki: &Ekstre, sonraki: &Ekstre) -> Option<ZincirBulgu> {
    if onceki.kapanis_bakiye != sonraki.acilis_bakiye {
        Some(ZincirBulgu { sira: usize::MAX, beklenen: onceki.kapanis_bakiye, gorulen: sonraki.acilis_bakiye })
    } else {
        None
    }
}

const TERS_IPUCLARI: [&str; 9] =
    ["TERS ", "IPTAL", "IADE", "GERI DONE", "REVERSAL", "REFUND", "CHARGEBACK", "TERS:", "ISLEM IPTAL"];

fn ters_ipucu(aciklama: &str) -> bool {
    let k = tr_katla(aciklama);
    TERS_IPUCLARI.iter().any(|p| k.contains(p))
}

/// 2 katmanlı ters işlem tespiti (kaynak kural: "sadece İADE yazması YETMEZ").
/// Katman 1: açıklamada ipucu. Katman 2: daha önceki hareketlerde aynı tutarlı, TERS YÖNLÜ ve
/// karşı tarafı uyumlu (skor ≥ 0.5 ya da iki tarafta da isim yok) bir orijinal işlem ARANIR.
/// İDEMPOTENT: her çağrıda işaretler sıfırdan hesaplanır (çift çağrı zararsız).
pub fn ters_isle(hareketler: &mut [EkstreHareket]) -> usize {
    for h in hareketler.iter_mut() {
        h.ters_islem = false;
    }
    let mut sayi = 0;
    for i in 0..hareketler.len() {
        if !ters_ipucu(&hareketler[i].aciklama) {
            continue;
        }
        let aday = &hareketler[i];
        let onaylandi = hareketler[..i].iter().any(|o| {
            o.tutar == aday.tutar
                && o.yon != aday.yon
                && match (&o.karsi_taraf, &aday.karsi_taraf) {
                    (Some(a), Some(b)) => isim_skoru(a, b) >= 0.5,
                    (None, None) => true,
                    _ => false,
                }
        });
        if onaylandi {
            hareketler[i].ters_islem = true;
            sayi += 1;
        }
    }
    sayi
}

#[cfg(test)]
mod test {
    use super::*;

    fn h(gun: u8, tutar: i64, bakiye: i64, yon: HareketYonu, aciklama: &str, karsi: Option<&str>) -> EkstreHareket {
        EkstreHareket {
            tarih: Tarih::yeni(2026, 3, gun),
            aciklama: aciklama.into(),
            tutar,
            bakiye_sonrasi: bakiye,
            yon,
            referans_no: None,
            karsi_taraf: karsi.map(String::from),
            ters_islem: false,
        }
    }

    fn ekstre(hareketler: Vec<EkstreHareket>, acilis: i64, kapanis: i64) -> Ekstre {
        Ekstre {
            iban: "TR00".into(), banka: "test".into(),
            donem_bas: Tarih::yeni(2026, 3, 1), donem_son: Tarih::yeni(2026, 3, 31),
            acilis_bakiye: acilis, kapanis_bakiye: kapanis, hareketler,
        }
    }

    #[test]
    fn saglam_zincir_bulgusuz() {
        let e = ekstre(vec![
            h(1, 10_000, 110_000, HareketYonu::Giris, "tahsilat", None),
            h(2, 4_000, 106_000, HareketYonu::Cikis, "ödeme", None),
        ], 100_000, 106_000);
        assert!(zincir_dogrula(&e).is_empty());
    }

    #[test]
    fn kopuk_zincir_tek_bulgu() {
        // 2. hareketin bakiyesi yanlış; zincir bankadan devam eder → sahte ardıl bulgu üremez
        let e = ekstre(vec![
            h(1, 10_000, 110_000, HareketYonu::Giris, "tahsilat", None),
            h(2, 4_000, 105_000, HareketYonu::Cikis, "ödeme", None), // beklenen 106.000
            h(3, 1_000, 106_000, HareketYonu::Giris, "faiz", None),
        ], 100_000, 106_000);
        let b = zincir_dogrula(&e);
        assert_eq!(b.len(), 1);
        assert_eq!(b[0], ZincirBulgu { sira: 1, beklenen: 106_000, gorulen: 105_000 });
    }

    #[test]
    fn kapanis_tutmazsa_bulgu() {
        let e = ekstre(vec![h(1, 10_000, 110_000, HareketYonu::Giris, "x", None)], 100_000, 111_000);
        let b = zincir_dogrula(&e);
        assert_eq!(b.len(), 1);
        assert_eq!(b[0].gorulen, 111_000);
    }

    #[test]
    fn devir_kontrolu() {
        let a = ekstre(vec![], 0, 50_000);
        let mut b = ekstre(vec![], 50_000, 50_000);
        assert!(devir_dogrula(&a, &b).is_none());
        b.acilis_bakiye = 49_000;
        assert_eq!(devir_dogrula(&a, &b).unwrap().beklenen, 50_000);
    }

    #[test]
    fn dedup_anahtari_sira_ile_ayrisir() {
        let x = h(1, 5_000, 105_000, HareketYonu::Giris, "a", None);
        assert_ne!(hareket_anahtari("TR00", &x, 0), hareket_anahtari("TR00", &x, 1));
        assert_eq!(hareket_anahtari("TR00", &x, 0), hareket_anahtari("TR00", &x.clone(), 0));
    }

    #[test]
    fn ters_islem_ipucu_yetmez_orijinal_sart() {
        let mut hs = vec![
            h(5, 7_000, 93_000, HareketYonu::Cikis, "İADE — karşılığı olmayan", Some("Ali Veli")),
        ];
        assert_eq!(ters_isle(&mut hs), 0, "orijinali olmayan İADE işaretlenmemeli");

        let mut hs = vec![
            h(1, 7_000, 107_000, HareketYonu::Giris, "FAST gelen", Some("Ali Veli")),
            h(5, 7_000, 100_000, HareketYonu::Cikis, "FAST İADE", Some("ALİ VELİ - Fast Anlık Ödeme")),
        ];
        assert_eq!(ters_isle(&mut hs), 1);
        assert!(hs[1].ters_islem && !hs[0].ters_islem);
    }

    #[test]
    fn ters_isle_idempotent() {
        let mut hs = vec![
            h(1, 7_000, 107_000, HareketYonu::Giris, "gelen", None),
            h(5, 7_000, 100_000, HareketYonu::Cikis, "İPTAL", None),
        ];
        assert_eq!(ters_isle(&mut hs), 1);
        assert_eq!(ters_isle(&mut hs), 1, "ikinci çağrı sonucu değiştirmemeli (çift çağrı bug'ının ilacı)");
    }
}
