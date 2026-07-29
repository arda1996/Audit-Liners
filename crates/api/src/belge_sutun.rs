//! SÜTUN ADLANDIRMA — kolonun NE OLDUĞUNU bulur.
//!
//! ## İki ayrı problem
//!
//! | Soru | Çözüm | Şablon gerekir mi |
//! |---|---|---|
//! | Sütun **nerede**? | Dikey boşluk koridoru (`belge_konum`) | Hayır — geometrik olgu |
//! | Sütun **ne**? | Başlık satırı + sözlük (bu modül) | Evet — anlam öğrenilir |
//!
//! Bu ayrım kritik: konum her belgede vardır ve tanımaya gerek yoktur; anlam ise yazıya
//! bağlıdır ve her firma başka yazar ("MİKTAR" / "Adet" / "Mik." / "Qty" / "Menge").
//!
//! ## Neden gerekli
//!
//! `belge_kalem` kalemleri **aritmetikle** buluyor: `a × b = c` tutan üçlüyü arıyor.
//! Bu düzenden bağımsız olduğu için güçlü, ama iki zaafı var:
//!
//! 1. **Çarpma değişmelidir** — `100 × 80` ile `80 × 100` aynı sonucu verir; hangisinin
//!    miktar hangisinin fiyat olduğu aritmetikten çıkmaz. Şu an soldan-sağa konum kuralıyla
//!    tahmin ediliyor, bu da sütun sırası değişince kırılır.
//! 2. **Aritmetiği olmayan sütun görünmez** — mal adı, birim, iskonto oranı, KDV oranı.
//!    Kullanıcının "mal cinsi mal adeti gibi kolonlar bizim alanlar bölümümüzde yok"
//!    şikâyeti tam olarak budur.
//!
//! Başlık tanınırsa ikisi de çözülür: miktar hangi sütunsa odur, mal adı hangi sütunsa odur.
//!
//! ## Doktrin korunur: başlık ADAY üretir, hakem yine aritmetiktir
//!
//! Başlık eşleşmesi bir **ipucudur**, kanıt değil. Başlığa göre okunan kalem yine
//! `miktar × birim fiyat = tutar` sınamasından geçer; tutmuyorsa başlık yanlış
//! haritalanmış demektir ve aritmetik kazanır. Tanıma önerir, doğrulama karar verir.

use serde::Serialize;

const SOZLUK: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/belge-sozlugu.json"));

static VERI: std::sync::OnceLock<serde_json::Value> = std::sync::OnceLock::new();
fn sozluk() -> &'static serde_json::Value {
    VERI.get_or_init(|| serde_json::from_str(SOZLUK).unwrap_or(serde_json::Value::Null))
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct Sutun {
    /// Izgaradaki sütun indeksi.
    pub indeks: usize,
    /// Belgede yazan başlık metni, olduğu gibi.
    pub ham: String,
    /// Kanonik anlam: SIRA · MAL_ADI · BIRIM · MIKTAR · BIRIM_FIYAT · ISKONTO ·
    /// KDV_ORANI · KDV_TUTARI · TUTAR. Tanınmadıysa boş.
    pub anlam: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct BaslikSatiri {
    /// Izgarada kaçıncı satır başlık.
    pub satir: usize,
    pub sutunlar: Vec<Sutun>,
    /// Kaç sütunun anlamı çözüldü.
    pub taninan: usize,
}

impl BaslikSatiri {
    /// Bir anlamın hangi sütunda olduğunu verir.
    pub fn sutun(&self, anlam: &str) -> Option<usize> {
        self.sutunlar.iter().find(|s| s.anlam == anlam).map(|s| s.indeks)
    }
}

/// Başlık metnini kanonik anlama eşler.
///
/// **En UZUN eşleşme kazanır.** Bu şart: "KDV ORANI" hem `kdv orani`ya (KDV_ORANI) hem
/// `kdv`ye (KDV_ORANI) uyar; "BİRİM FİYAT" hem `birim fiyat`a (BIRIM_FIYAT) hem `birim`e
/// (BIRIM) uyar. Kısa eşleşme kazansaydı birim fiyat sütunu "birim" sanılırdı.
pub fn anlam_coz(baslik: &str) -> String {
    let sade = crate::belge_oku::sadelestir(baslik);
    let sade = sade.trim();
    if sade.is_empty() { return String::new(); }

    let mut en_iyi = (String::new(), 0usize);
    let Some(tablo) = sozluk()["sutun_basliklari"].as_object() else { return String::new() };
    for (anlam, diller) in tablo {
        for (_dil, liste) in diller.as_object().into_iter().flatten() {
            for a in liste.as_array().into_iter().flatten() {
                let Some(t) = a.as_str() else { continue };
                if t.is_empty() || !sade.contains(t) { continue; }
                if t.len() > en_iyi.1 { en_iyi = (anlam.clone(), t.len()); }
            }
        }
    }
    en_iyi.0
}

/// Bir hücre sayısal mı? (tutar, miktar, oran — başlık satırı ayırt etmek için)
fn sayisal(h: &str) -> bool {
    let r: String = h.chars().filter(|c| c.is_ascii_digit()).collect();
    !r.is_empty() && h.chars().filter(|c| c.is_alphabetic()).count() <= 2
}

/// Izgarada kalem tablosunun BAŞLIK SATIRINI bulur.
///
/// Ölçüt üç katlı, hepsi birden aranır:
/// 1. Satırda en az 3 hücre var (tek sütunlu bir metin satırı başlık değildir).
/// 2. Hücrelerin hiçbiri sayısal değil (başlıkta rakam olmaz).
/// 3. En az 2 hücrenin anlamı sözlükten çözülüyor.
///
/// Birden çok aday varsa **en çok tanınan** kazanır; eşitlikte üstteki (fatura başlık
/// satırı tablonun tepesindedir, alttaki "TOPLAM" bloğu değil).
pub fn basliklari_bul(izgara: &[Vec<String>]) -> Option<BaslikSatiri> {
    let mut en_iyi: Option<BaslikSatiri> = None;

    for (i, satir) in izgara.iter().enumerate() {
        if satir.len() < 3 { continue; }
        if satir.iter().any(|h| sayisal(h)) { continue; }

        let sutunlar: Vec<Sutun> = satir.iter().enumerate()
            .map(|(j, h)| Sutun { indeks: j, ham: h.trim().to_string(), anlam: anlam_coz(h) })
            .collect();
        let taninan = sutunlar.iter().filter(|s| !s.anlam.is_empty()).count();
        if taninan < 2 { continue; }

        // Aynı anlam iki sütuna düşerse başlık yanlış okunmuştur — daha az tanınan
        // ama tutarlı bir satır, daha çok tanınan ama çelişkili satırdan iyidir.
        let mut anlamlar: Vec<&str> = sutunlar.iter()
            .filter(|s| !s.anlam.is_empty()).map(|s| s.anlam.as_str()).collect();
        anlamlar.sort_unstable();
        let n = anlamlar.len();
        anlamlar.dedup();
        if anlamlar.len() != n { continue; }

        let aday = BaslikSatiri { satir: i, sutunlar, taninan };
        if en_iyi.as_ref().map(|e| aday.taninan > e.taninan).unwrap_or(true) {
            en_iyi = Some(aday);
        }
    }
    en_iyi
}

#[cfg(test)]
mod testler {
    use super::*;

    fn iz(satirlar: &[&[&str]]) -> Vec<Vec<String>> {
        satirlar.iter().map(|s| s.iter().map(|x| x.to_string()).collect()).collect()
    }

    /// Türk faturasının klasik kalem tablosu başlığı.
    #[test]
    fn turkce_fatura_basligi_cozulur() {
        let g = iz(&[
            &["ÖRNEK TEKSTİL"],
            &["CİNSİ", "BİRİM", "MİKTAR", "BİRİM FİYAT", "KDV %", "TUTAR"],
            &["ÇORAP", "ADET", "200", "2,50", "8", "500,00"],
        ]);
        let b = basliklari_bul(&g).expect("başlık bulunamadı");
        assert_eq!(b.satir, 1);
        assert_eq!(b.sutun("MAL_ADI"), Some(0));
        assert_eq!(b.sutun("BIRIM"), Some(1));
        assert_eq!(b.sutun("MIKTAR"), Some(2));
        assert_eq!(b.sutun("BIRIM_FIYAT"), Some(3), "sütunlar: {:?}", b.sutunlar);
        assert_eq!(b.sutun("KDV_ORANI"), Some(4));
        assert_eq!(b.sutun("TUTAR"), Some(5));
    }

    /// **En uzun eşleşme şart.** "BİRİM FİYAT" kısa eşleşmeyle "BİRİM" sanılırsa
    /// miktar ile fiyat yer değiştirir ve kalem tutarı yanlış hesaplanır.
    #[test]
    fn birim_fiyat_birim_sanilmaz() {
        assert_eq!(anlam_coz("BİRİM FİYAT"), "BIRIM_FIYAT");
        assert_eq!(anlam_coz("BİRİM"), "BIRIM");
        assert_eq!(anlam_coz("KDV ORANI"), "KDV_ORANI");
        assert_eq!(anlam_coz("KDV TUTARI"), "KDV_TUTARI");
    }

    /// Aynı belge İngilizce gelirse de çözülmeli — Türkçe ve İngilizce eşit öncelikli.
    #[test]
    fn ingilizce_baslik_cozulur() {
        let g = iz(&[
            &["DESCRIPTION", "UNIT", "QTY", "UNIT PRICE", "VAT %", "AMOUNT"],
            &["SOCKS", "PCS", "200", "2.50", "8", "500.00"],
        ]);
        let b = basliklari_bul(&g).expect("başlık yok");
        assert_eq!(b.sutun("MAL_ADI"), Some(0));
        assert_eq!(b.sutun("MIKTAR"), Some(2));
        assert_eq!(b.sutun("BIRIM_FIYAT"), Some(3));
        assert_eq!(b.sutun("TUTAR"), Some(5));
    }

    /// Rakam içeren satır başlık DEĞİLDİR — kalem satırı başlık sanılmamalı.
    #[test]
    fn kalem_satiri_baslik_sanilmaz() {
        let g = iz(&[&["ÇORAP", "ADET", "200", "2,50", "8", "500,00"]]);
        assert!(basliklari_bul(&g).is_none(), "kalem satırı başlık sanıldı");
    }

    /// Toplam bloğu (2 hücre) başlık sanılmamalı.
    #[test]
    fn toplam_blogu_baslik_sanilmaz() {
        let g = iz(&[&["GENEL TOPLAM", "3.456,00"], &["NET TUTAR", "3.200,00"]]);
        assert!(basliklari_bul(&g).is_none(), "toplam bloğu başlık sanıldı: {:?}", basliklari_bul(&g));
    }

    /// OCR başlığı bozarsa tanınmayan sütun BOŞ anlamla döner — uydurulmaz.
    #[test]
    fn taninmayan_sutun_uydurulmaz() {
        let g = iz(&[
            &["C1NS1", "B1R1M", "MİKTAR", "TUTAR"],
            &["ÇORAP", "ADET", "200", "500,00"],
        ]);
        let b = basliklari_bul(&g).expect("başlık yok");
        assert_eq!(b.sutun("MIKTAR"), Some(2));
        assert_eq!(b.sutun("TUTAR"), Some(3));
        // "C1NS1" ve "B1R1M" bozuk — anlamları BOŞ kalmalı, yakın bir anlama zorlanmamalı.
        assert_eq!(b.sutunlar[0].anlam, "", "bozuk başlığa anlam uyduruldu");
        assert_eq!(b.sutunlar[1].anlam, "");
    }

    /// Aynı anlam iki sütuna düşerse o satır başlık kabul edilmez.
    #[test]
    fn celiskili_baslik_reddedilir() {
        let g = iz(&[&["TUTAR", "TUTAR", "MİKTAR"]]);
        assert!(basliklari_bul(&g).is_none(), "çelişkili başlık kabul edildi");
    }
}
