//! KALEM ÇIKARIMI — faturanın mal/hizmet satırları.
//!
//! ## Sorun
//!
//! Fatura başlığı ve toplamları okunuyordu ama **satır dökümü** okunmuyordu: mal cinsi,
//! miktar, birim fiyat, satır tutarı. Bunlar muhasebede lazım — stok kaydı, KDV oran
//! kırılımı, denetimde örnekleme, Ba/Bs mutabakatı hep satır düzeyinde çalışır.
//!
//! ## Yaklaşım: tabloyu ANLAMAYA çalışmıyoruz, ARİTMETİKLE buluyoruz
//!
//! Sütun başlıklarını tanımaya çalışmak kırılgan: her fatura farklı yazıyor
//! ("Miktar" / "Adet" / "Mik." / "Qty"), sütun sırası değişiyor, bazılarında başlık hiç yok.
//!
//! Bunun yerine **satırın kendi aritmetiğini** arıyoruz: bir satırda üç sayı varsa ve
//! `a × b = c` tutuyorsa, o satır bir kalemdir ve a=miktar, b=birim fiyat, c=tutardır.
//! Bu, düzenden tamamen bağımsızdır — sütun sırası, başlık metni, font, hizalama
//! hiçbirini bilmeye gerek yok.
//!
//! Motorun geri kalanıyla aynı doktrin: **hangi sayının ne olduğunu aritmetik söyler.**
//!
//! ## Yanlış pozitife karşı
//!
//! Rastgele üç sayının çarpım ilişkisi tutturması olasıdır (ör. tarih parçaları).
//! Üç koruma var: (1) tutar sıfırdan büyük olmalı, (2) satırda metin (mal adı) bulunmalı,
//! (3) **Σ satır tutarı = matrah** — asıl kanıt bu. Toplam tutmuyorsa kalemler kabul edilmez.

use serde::Serialize;

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Kalem {
    /// Mal/hizmet adı — satırdaki ilk sayıdan ÖNCEKİ metin.
    pub ad: String,
    /// Miktar. Kesirli olabilir (0,5 kg) — bu yüzden kuruş değil ondalık.
    pub miktar: f64,
    pub birim: String,
    /// Birim fiyat (kuruş).
    pub birim_fiyat: i64,
    /// Satır tutarı (kuruş).
    pub tutar: i64,
    /// Kaçıncı satırdan geldiği — kullanıcıya gösterirken belgede işaretlemek için.
    pub satir: usize,
}

/// Bilinen ölçü birimleri. Miktarın hemen ardındaki sözcük buysa birim olarak alınır.
/// Liste kapalı tutuluyor: her sözcüğü birim saymak, mal adının bir parçasını koparırdı.
const BIRIMLER: &[&str] = &[
    "adet", "ad", "kg", "gr", "g", "ton", "lt", "l", "ml", "m", "m2", "m²", "m3", "m³",
    "cm", "mm", "paket", "pk", "koli", "kutu", "çift", "cift", "düzine", "duzine",
    "saat", "gün", "gun", "ay", "yıl", "yil", "kişi", "kisi", "sefer", "km",
];

/// Satırdaki sayıları (konum + değer) çıkarır. Değer kuruş cinsindendir.
fn sayilar(satir: &str, dil: &str) -> Vec<(usize, usize, i64)> {
    let h: Vec<char> = satir.chars().collect();
    let mut cikti = Vec::new();
    let mut i = 0;
    while i < h.len() {
        if !h[i].is_ascii_digit() { i += 1; continue; }
        let bas = i;
        while i < h.len() && (h[i].is_ascii_digit() || h[i] == '.' || h[i] == ',') { i += 1; }
        let mut son = i;
        while son > bas && (h[son - 1] == '.' || h[son - 1] == ',') { son -= 1; }
        // Yüzdeye bitişik sayı ORAN'dır, tutar değil — kalem aritmetiğine sokulmaz.
        let yuzde = (bas > 0 && h[bas - 1] == '%') || (son < h.len() && h[son] == '%');
        let ham: String = h[bas..son].iter().collect();
        if !yuzde {
            if let Some(v) = crate::belge_oku::sayi_ayristir(&ham, dil) { cikti.push((bas, son, v)); }
        }
        i = son.max(bas + 1);
    }
    cikti
}

/// Bir satır kalem mi? Öyleyse çözümlenmiş halini döndürür.
///
/// `miktar × birim_fiyat = tutar` üçlüsünü arar. Birden çok üçlü tutarsa **en sağdaki
/// tutarı** olan seçilir: fatura tablolarında satır tutarı en sağdaki sütundur.
pub fn satir_kalem(satir: &str, indeks: usize, dil: &str) -> Option<Kalem> {
    let s = sayilar(satir, dil);
    if s.len() < 3 { return None; }

    let mut en_iyi: Option<Kalem> = None;
    for a in 0..s.len() {
        for b in 0..s.len() {
            if b == a { continue; }
            for c in 0..s.len() {
                if c == a || c == b { continue; }
                let (_, _, miktar_k) = s[a];
                let (_, _, fiyat) = s[b];
                let (c_bas, _, tutar) = s[c];
                if tutar <= 0 || fiyat <= 0 || miktar_k <= 0 { continue; }
                // Miktar kuruş olarak okundu (100 → 10000); gerçek miktar /100.
                let miktar = miktar_k as f64 / 100.0;
                let hesap = miktar * fiyat as f64;
                // Yuvarlama payı: kuruş hesabında 1 kuruşluk sapma olağandır.
                if (hesap - tutar as f64).abs() > 1.0 { continue; }
                // TABLO DÜZENİNİN DOĞAL SIRASI: miktar → birim fiyat → tutar, soldan sağa.
                // Bu kural şart: çarpma değişmeli olduğu için `100 × 80,00` ile `80 × 100,00`
                // aynı sonucu verir ve motor miktarla fiyatı ters seçebilir. Konum bunu çözer.
                if s[a].0 >= s[b].0 { continue; }              // miktar, fiyatın SOLUNDA
                if c_bas < s[a].0 || c_bas < s[b].0 { continue; } // tutar en sağda

                let ad = satir[..s[a].0.min(satir.len())].trim()
                    .trim_matches(|c: char| c.is_ascii_digit() || c == '|' || c == '.').trim().to_string();
                if ad.chars().filter(|c| c.is_alphabetic()).count() < 2 { continue; }

                // Miktarın hemen ardındaki sözcük birim olabilir.
                let kuyruk: String = satir.chars().skip(s[a].1).take(12).collect();
                let birim = kuyruk.split_whitespace().next().unwrap_or("")
                    .trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase();
                let birim = if BIRIMLER.contains(&birim.as_str()) { birim } else { String::new() };

                let aday = Kalem { ad, miktar, birim, birim_fiyat: fiyat, tutar, satir: indeks };
                // Birden çok üçlü tutarsa EN SAĞDAKİ tutarı olan seçilir: fatura tablosunda
                // satır tutarı en sağdaki sütundur (ara sütunlar iskonto/KDV olabilir).
                if en_iyi.as_ref().map(|e| tutar > e.tutar).unwrap_or(true) {
                    en_iyi = Some(aday);
                }
            }
        }
    }
    en_iyi
}

#[derive(Serialize, Debug)]
pub struct KalemSonuc {
    pub kalemler: Vec<Kalem>,
    pub toplam: i64,
    /// Kalem toplamı matrahla tutuyor mu? Kalemlerin KABUL ÖLÇÜTÜ budur.
    pub matrahla_tutuyor: Option<bool>,
    pub not: String,
}

/// Belgedeki kalem satırlarını bulur ve matrahla doğrular.
///
/// `matrah` verilirse Σtutar ile karşılaştırılır. **Tutmuyorsa kalemler DÖNDÜRÜLÜR ama
/// `matrahla_tutuyor=false` işaretlenir** — silmek yerine işaretlemek doğru: kullanıcı
/// hangi satırın eksik/fazla olduğunu görüp düzeltebilmeli.
pub fn kalemleri_bul(satirlar: &[&str], dil: &str, matrah: Option<i64>) -> KalemSonuc {
    let mut kalemler: Vec<Kalem> = Vec::new();
    for (i, s) in satirlar.iter().enumerate() {
        if let Some(k) = satir_kalem(s, i, dil) { kalemler.push(k); }
    }
    // Aynı tutarı taşıyan mükerrer satırları ayıkla (ara toplam satırı kalem gibi görünebilir).
    kalemler.dedup_by(|a, b| a.tutar == b.tutar && a.ad == b.ad);

    let toplam: i64 = kalemler.iter().map(|k| k.tutar).sum();
    let matrahla_tutuyor = matrah.map(|m| m == toplam);
    let not = match (kalemler.is_empty(), matrahla_tutuyor) {
        (true, _) => "Kalem satırı bulunamadı — belgede miktar × birim fiyat = tutar ilişkisi kurulamadı.".into(),
        (false, Some(true)) => format!("{} kalem bulundu ve toplamı matrahla TUTUYOR — kalemler doğrulandı.", kalemler.len()),
        (false, Some(false)) => format!(
            "{} kalem bulundu ama toplamı ({}) matrahla uyuşmuyor — bir satır eksik veya fazla okunmuş olabilir.",
            kalemler.len(), toplam / 100),
        (false, None) => format!("{} kalem bulundu; matrah verilmediği için doğrulanamadı.", kalemler.len()),
    };
    KalemSonuc { kalemler, toplam, matrahla_tutuyor, not }
}

#[cfg(test)]
mod testler {
    use super::*;

    /// Konum motorundan çıkan gerçekçi fatura tablosu.
    const TABLO: &[&str] = &[
        " Sıra  Mal/Hizmet          Miktar   Birim Fiyat        Tutar",
        " 1     Pamuk iplik          100 kg     80,00           8.000,00",
        " 2     Boya malzemesi        25 adet   120,00          3.000,00",
        " 3     Nakliye hizmeti        1        500,00            500,00",
        "                                    Mal Hizmet Toplam Tutarı     11.500,00",
        "                                    Hesaplanan KDV %20            2.300,00",
        "                                    Vergiler Dahil Toplam        13.800,00",
    ];

    #[test]
    fn kalem_satiri_aritmetikle_bulunur() {
        // Sütun başlığını TANIMADAN, yalnız miktar × fiyat = tutar ilişkisiyle.
        let k = satir_kalem(TABLO[1], 1, "tr").expect("kalem bulunamadı");
        assert_eq!(k.miktar, 100.0);
        assert_eq!(k.birim_fiyat, 8_000, "birim fiyat kuruş olarak yanlış");
        assert_eq!(k.tutar, 800_000);
        assert!(k.ad.contains("Pamuk iplik"), "mal adı yanlış: {}", k.ad);
        assert_eq!(k.birim, "kg", "birim okunmadı");
    }

    #[test]
    fn baslik_ve_toplam_satirlari_kalem_sayilmaz() {
        // Başlıkta sayı yok; toplam satırlarında çarpım ilişkisi yok.
        assert!(satir_kalem(TABLO[0], 0, "tr").is_none(), "başlık satırı kalem sanıldı");
        for i in 4..7 {
            assert!(satir_kalem(TABLO[i], i, "tr").is_none(),
                "toplam satırı kalem sanıldı: {}", TABLO[i]);
        }
    }

    #[test]
    fn oran_kalem_aritmetigine_girmez() {
        // "Hesaplanan KDV %20   2.300,00" — %20 bir çarpan değildir.
        assert!(satir_kalem("Hesaplanan KDV %20   11.500,00   2.300,00", 0, "tr").is_none(),
            "yüzde oranı miktar sanılıp sahte kalem üretildi");
    }

    #[test]
    fn kalem_toplami_matrahla_dogrulanir() {
        let s = kalemleri_bul(TABLO, "tr", Some(1_150_000));
        assert_eq!(s.kalemler.len(), 3, "kalem sayısı yanlış: {:?}",
            s.kalemler.iter().map(|k| &k.ad).collect::<Vec<_>>());
        assert_eq!(s.toplam, 1_150_000);
        assert_eq!(s.matrahla_tutuyor, Some(true), "{}", s.not);
        assert!(s.not.contains("TUTUYOR"));
    }

    #[test]
    fn eksik_kalem_matrah_kontrolunde_yakalanir() {
        // Bir satır okunamamış gibi davran — toplam tutmamalı ve BU BİLDİRİLMELİ.
        let eksik: Vec<&str> = TABLO.iter().enumerate()
            .filter(|(i, _)| *i != 2).map(|(_, s)| *s).collect();
        let s = kalemleri_bul(&eksik, "tr", Some(1_150_000));
        assert_eq!(s.matrahla_tutuyor, Some(false), "eksik kalem yakalanmadı");
        assert!(s.not.contains("uyuşmuyor"), "kullanıcıya sorun bildirilmeli: {}", s.not);
        // Kalemler SİLİNMEZ — kullanıcı hangisinin eksik olduğunu görebilmeli.
        assert_eq!(s.kalemler.len(), 2);
    }

    #[test]
    fn kesirli_miktar_okunur() {
        // 2,5 kg × 40,00 = 100,00
        let k = satir_kalem("Boya   2,5 kg   40,00   100,00", 0, "tr").expect("kesirli miktar okunmadı");
        assert_eq!(k.miktar, 2.5);
        assert_eq!(k.tutar, 10_000);
    }

    #[test]
    fn adsiz_satir_kalem_sayilmaz() {
        // Mal adı olmayan satır (yalnız sayılar) kalem değildir — yanlış pozitif koruması.
        assert!(satir_kalem("  1   2   2  ", 0, "tr").is_none());
    }
}
