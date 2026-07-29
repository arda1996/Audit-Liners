//! Mutabakat motoru v2 — katmanlı, deterministik, kategorize (D.6; uyuyan modül).
//!
//! Kaynak fikir: takipsistemiv2 SuperMatch. Genelleştirilerek ve mantık düzeltmeleriyle port edildi:
//!   • Vevo'ya özel değil — banka↔defter, cari↔cari, her iki-taraflı liste.
//!   • Tie-break id ASC değil ÖNCE TARİH YAKINLIĞI (aynı tutarlı adaylar arasında |Δgün| en küçük
//!     olan eşleşir; eşitlikte id) — kaynaktaki id-tie-break tarihçe uzak kaydı eşleyebiliyordu.
//!   • Kesin tutar ilkesi korunur (kuruş toleransı YOK); isteğe bağlı tarih penceresi parametredir.
//!
//! Çıktı kategorileri çalışma kağıdına bulgu olarak akacak şekilde ayrıştırılır (BDS 505 veri ayağı).

use crate::fis::Tarih;
use crate::isim::kanonik_isim;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct MutabakatKayit {
    pub id: usize,
    pub tarih: Tarih,
    /// Kuruş, pozitif (yön/taraf çağıranın sorumluluğu — motor tek yönlü listeleri karşılaştırır).
    pub tutar: i64,
    /// Karşı taraf adı; None/boş → isimsiz havuz (tutar-esaslı eşleşme).
    pub isim: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtikKategori {
    /// Grup iki tarafta da var; bu kayıt karşı tarafta eksik.
    GrupIciEksik,
    /// Bu isim karşı tarafta hiç yok.
    IsimYok,
}

#[derive(Debug, Default)]
pub struct MutabakatSonuc {
    /// (bizim id, karşı id) — kesin eşleşmeler.
    pub eslesen: Vec<(usize, usize)>,
    pub yalniz_biz: Vec<(usize, ArtikKategori)>,
    pub yalniz_karsi: Vec<(usize, ArtikKategori)>,
}

/// Kaba gün indeksi — yakınlık kıyası için monotonik (takvim hassasiyeti gerekmez).
fn gun_indeksi(t: &Tarih) -> i64 {
    t.yil as i64 * 372 + t.ay as i64 * 31 + t.gun as i64
}

fn grup_anahtari(k: &MutabakatKayit) -> String {
    k.isim.as_deref().map(kanonik_isim).unwrap_or_default()
}

/// Katmanlı mutabakat: kanonik isim grubu → grup içi KESİN tutar eşleşmesi (tarih-yakınlık öncelikli,
/// deterministik) → kategorize artıklar. `pencere_gun`: eşleşme için azami |Δgün| (None = sınırsız).
pub fn mutabakat(
    biz: &[MutabakatKayit],
    karsi: &[MutabakatKayit],
    pencere_gun: Option<i64>,
) -> MutabakatSonuc {
    // Gruplama — BTreeMap: deterministik gezinme (kaynaktaki "sorted keys" ilkesi).
    let mut gb: BTreeMap<String, Vec<&MutabakatKayit>> = BTreeMap::new();
    let mut gk: BTreeMap<String, Vec<&MutabakatKayit>> = BTreeMap::new();
    for k in biz {
        gb.entry(grup_anahtari(k)).or_default().push(k);
    }
    for k in karsi {
        gk.entry(grup_anahtari(k)).or_default().push(k);
    }

    let mut sonuc = MutabakatSonuc::default();
    let bos: Vec<&MutabakatKayit> = Vec::new();

    for (grup, bizler) in &gb {
        let karsilar = gk.get(grup).unwrap_or(&bos);
        if karsilar.is_empty() {
            for k in bizler {
                sonuc.yalniz_biz.push((k.id, ArtikKategori::IsimYok));
            }
            continue;
        }
        // Grup içi kesin tutar eşleşmesi — deterministik: bizimkiler (tarih,id) sıralı gezilir,
        // her biri için karşı adaylar arasından |Δgün| en küçük (eşitlikte id küçük) seçilir.
        let mut b_sirali: Vec<&MutabakatKayit> = bizler.clone();
        b_sirali.sort_by_key(|k| (gun_indeksi(&k.tarih), k.id));
        let mut kalan_karsi: Vec<&MutabakatKayit> = karsilar.clone();
        kalan_karsi.sort_by_key(|k| (gun_indeksi(&k.tarih), k.id));

        for b in b_sirali {
            let secim = kalan_karsi
                .iter()
                .enumerate()
                .filter(|(_, k)| k.tutar == b.tutar)
                .filter(|(_, k)| {
                    pencere_gun.map_or(true, |p| (gun_indeksi(&k.tarih) - gun_indeksi(&b.tarih)).abs() <= p)
                })
                .min_by_key(|(_, k)| ((gun_indeksi(&k.tarih) - gun_indeksi(&b.tarih)).abs(), k.id));
            match secim {
                Some((idx, k)) => {
                    sonuc.eslesen.push((b.id, k.id));
                    kalan_karsi.remove(idx);
                }
                None => sonuc.yalniz_biz.push((b.id, ArtikKategori::GrupIciEksik)),
            }
        }
        for k in kalan_karsi {
            sonuc.yalniz_karsi.push((k.id, ArtikKategori::GrupIciEksik));
        }
    }
    // Karşıda olup bizde grubu hiç olmayanlar
    for (grup, karsilar) in &gk {
        if !gb.contains_key(grup) {
            for k in karsilar {
                sonuc.yalniz_karsi.push((k.id, ArtikKategori::IsimYok));
            }
        }
    }
    sonuc.eslesen.sort_unstable();
    sonuc.yalniz_biz.sort_unstable_by_key(|(id, _)| *id);
    sonuc.yalniz_karsi.sort_unstable_by_key(|(id, _)| *id);
    sonuc
}

#[cfg(test)]
mod test {
    use super::*;

    fn k(id: usize, gun: u8, tutar: i64, isim: Option<&str>) -> MutabakatKayit {
        MutabakatKayit { id, tarih: Tarih::yeni(2026, 4, gun), tutar, isim: isim.map(String::from) }
    }

    #[test]
    fn kesin_eslesme_ve_kategoriler() {
        let biz = vec![
            k(1, 5, 10_000, Some("Ali Veli")),
            k(2, 7, 20_000, Some("Ali Veli")),   // karşıda yok → GrupIciEksik
            k(3, 9, 5_000, Some("Mehmet Demir")), // karşıda isim yok → IsimYok
        ];
        let karsi = vec![
            k(11, 5, 10_000, Some("ALİ VELİ - FAST Anlık Ödeme")), // kanonik aynı grup
            k(12, 8, 7_000, Some("Ayşe Kaya")),                     // bizde isim yok → IsimYok
        ];
        let s = mutabakat(&biz, &karsi, None);
        assert_eq!(s.eslesen, vec![(1, 11)]);
        assert_eq!(s.yalniz_biz, vec![(2, ArtikKategori::GrupIciEksik), (3, ArtikKategori::IsimYok)]);
        assert_eq!(s.yalniz_karsi, vec![(12, ArtikKategori::IsimYok)]);
    }

    #[test]
    fn kurus_toleransi_yok() {
        let s = mutabakat(&[k(1, 5, 10_000, None)], &[k(2, 5, 10_001, None)], None);
        assert!(s.eslesen.is_empty(), "1 kuruş fark = farklı kayıt (kesin tutar ilkesi)");
    }

    #[test]
    fn tarih_yakinlik_tie_break() {
        // Aynı tutarlı iki karşı aday: 20'si (id 11) ve 6'sı (id 12). Bizimki 5'i →
        // id-ASC eşleme (kaynak davranışı) 11'i seçerdi; düzeltilmiş motor tarihçe yakın 12'yi seçmeli.
        let s = mutabakat(&[k(1, 5, 10_000, None)], &[k(11, 20, 10_000, None), k(12, 6, 10_000, None)], None);
        assert_eq!(s.eslesen, vec![(1, 12)]);
    }

    #[test]
    fn pencere_gun_siniri() {
        let s = mutabakat(&[k(1, 5, 10_000, None)], &[k(11, 20, 10_000, None)], Some(3));
        assert!(s.eslesen.is_empty());
        assert_eq!(s.yalniz_biz[0].1, ArtikKategori::GrupIciEksik);
    }

    #[test]
    fn deterministik_girdi_sirasi_farketmez() {
        let biz1 = vec![k(1, 5, 10_000, Some("A B")), k(2, 6, 10_000, Some("A B"))];
        let biz2 = vec![biz1[1].clone(), biz1[0].clone()];
        let karsi = vec![k(11, 6, 10_000, Some("A B")), k(12, 5, 10_000, Some("A B"))];
        let s1 = mutabakat(&biz1, &karsi, None);
        let s2 = mutabakat(&biz2, &karsi, None);
        assert_eq!(s1.eslesen, s2.eslesen);
        assert_eq!(s1.eslesen, vec![(1, 12), (2, 11)], "tarih yakınlığıyla çapraz değil düz eşleşme");
    }
}
