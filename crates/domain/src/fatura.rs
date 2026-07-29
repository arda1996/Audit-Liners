//! Fatura — vergiye giden yolun kapısı. E-fatura yaşam döngüsü:
//! Taslak → Gönderildi (entegratör call) → Kesinleşti (→ OTOMATİK fiş) / Reddedildi.
//! Fiş yalnız KESİNLEŞEN faturadan üretilir; orijinal belge (UBL/PDF) arşivde saklanır.

use crate::fis::{BelgeTipi, Dayanak, Fis, FisTipi, Tarih, YevmiyeSatiri};
use crate::para::Kurus;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaturaYonu {
    /// Satış (giden) — 120.xx borç / 600.xx + 391.xx alacak
    Satis,
    /// Alış (gelen) — 153/7xx + 191.xx borç / 320.xx alacak
    Alis,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaturaDurum {
    Taslak,
    Gonderildi,
    Kesinlesti,
    Reddedildi,
}

#[derive(Debug, Clone)]
pub struct FaturaSatiri {
    pub aciklama: String,
    /// Satırın gelir/gider/stok hesabı (600.xx / 153.xx / 7xx.xx — yaprak).
    pub hesap_kod: String,
    pub miktar: i64,
    pub birim_fiyat: Kurus,
    /// KDV oranı yüzde (0/1/10/20). İstisna varsa dikkate alınmaz.
    pub kdv_orani: u8,
    /// KDVK istisna kodu (ör. "11/1-a ihracat", "13", "17/4-g"). Varsa KDV satırı üretilmez.
    pub istisna_kodu: Option<String>,
}

impl FaturaSatiri {
    pub fn tutar(&self) -> i64 {
        self.miktar * self.birim_fiyat.deger()
    }
    pub fn kdv(&self) -> i64 {
        if self.istisna_kodu.is_some() {
            0
        } else {
            // yarım kuruş yukarı yuvarlama (standart)
            (self.tutar() * self.kdv_orani as i64 + 50) / 100
        }
    }
}

#[derive(Debug, Clone)]
pub struct Fatura {
    pub no: String,
    pub yon: FaturaYonu,
    /// Cari muavin kodu (120.xx alıcı / 320.xx satıcı).
    pub cari_kod: String,
    pub tarih: Tarih,
    pub satirlar: Vec<FaturaSatiri>,
    pub durum: FaturaDurum,
    /// E-fatura sistemindeki benzersiz kimlik (entegratörden döner).
    pub efatura_uuid: Option<String>,
    /// Orijinal belgenin arşiv referansı (UBL XML / PDF).
    pub belge_ref: Option<String>,
}

impl Fatura {
    pub fn matrah(&self) -> i64 {
        self.satirlar.iter().map(|s| s.tutar()).sum()
    }
    pub fn toplam_kdv(&self) -> i64 {
        self.satirlar.iter().map(|s| s.kdv()).sum()
    }
    pub fn genel_toplam(&self) -> i64 {
        self.matrah() + self.toplam_kdv()
    }
}

/// KDV oranını alt hesaba eşler: 391.01 / 391.10 / 391.20 (satış), 191.xx (alış).
fn kdv_hesabi(yon: FaturaYonu, oran: u8) -> String {
    let ana = match yon {
        FaturaYonu::Satis => "391",
        FaturaYonu::Alis => "191",
    };
    format!("{}.{:02}", ana, oran)
}

/// KESİNLEŞEN faturadan otomatik muhasebe fişi (taslak) üretir.
/// Kesinleşmemiş faturadan fiş üretilmez (None) — e-fatura onayı şart.
pub fn fatura_fisi(f: &Fatura) -> Option<Fis> {
    if f.durum != FaturaDurum::Kesinlesti {
        return None;
    }
    let mut s = Vec::new();
    let toplam = Kurus::yeni(f.genel_toplam());

    match f.yon {
        FaturaYonu::Satis => {
            s.push(YevmiyeSatiri::borc(f.cari_kod.clone(), toplam));
            for sat in &f.satirlar {
                let mut satir = YevmiyeSatiri::alacak(sat.hesap_kod.clone(), Kurus::yeni(sat.tutar()));
                satir.aciklama = match &sat.istisna_kodu {
                    Some(k) => format!("{} [İSTİSNA {}]", sat.aciklama, k),
                    None => sat.aciklama.clone(),
                };
                s.push(satir);
            }
            // KDV: oran bazında grupla (aynı oran alt hesabına tek satır)
            for oran in [1u8, 10, 20] {
                let k: i64 = f.satirlar.iter().filter(|x| x.kdv_orani == oran && x.istisna_kodu.is_none()).map(|x| x.kdv()).sum();
                if k > 0 {
                    s.push(YevmiyeSatiri::alacak(kdv_hesabi(f.yon, oran), Kurus::yeni(k)));
                }
            }
        }
        FaturaYonu::Alis => {
            for sat in &f.satirlar {
                let mut satir = YevmiyeSatiri::borc(sat.hesap_kod.clone(), Kurus::yeni(sat.tutar()));
                satir.aciklama = match &sat.istisna_kodu {
                    Some(k) => format!("{} [İSTİSNA {}]", sat.aciklama, k),
                    None => sat.aciklama.clone(),
                };
                s.push(satir);
            }
            for oran in [1u8, 10, 20] {
                let k: i64 = f.satirlar.iter().filter(|x| x.kdv_orani == oran && x.istisna_kodu.is_none()).map(|x| x.kdv()).sum();
                if k > 0 {
                    s.push(YevmiyeSatiri::borc(kdv_hesabi(f.yon, oran), Kurus::yeni(k)));
                }
            }
            s.push(YevmiyeSatiri::alacak(f.cari_kod.clone(), toplam));
        }
    }

    let mut fis = Fis::taslak(FisTipi::Mahsup, f.tarih, s);
    fis.aciklama = format!("Fatura {} ({})", f.no, match f.yon { FaturaYonu::Satis => "satış", FaturaYonu::Alis => "alış" });
    fis.dayanak = Some(Dayanak {
        belge_tipi: BelgeTipi::EFatura,
        belge_no: f.no.clone(),
        belge_tarihi: f.tarih,
    });
    Some(fis)
}
