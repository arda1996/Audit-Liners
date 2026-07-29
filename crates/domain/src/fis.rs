//! Fiş (voucher) + Yevmiye Satırı (journal line) — saklanan tek doğruluk kaynağı.

use crate::hesap::HesapId;
use crate::para::Kurus;

pub type FisId = u64;
pub type MasrafYeriId = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FisTipi {
    Acilis,
    Tahsil,
    Tediye,
    Mahsup,
    Kapanis,
}

impl FisTipi {
    /// Ayrı seri kodu (her tip kendi sırasını tutar).
    pub fn seri_kodu(&self) -> &'static str {
        match self {
            FisTipi::Tahsil => "TAH",
            FisTipi::Tediye => "TED",
            FisTipi::Mahsup => "MAH",
            FisTipi::Acilis => "AC",
            FisTipi::Kapanis => "KAP",
        }
    }

    /// Dayanak zorunlu mu? Tahsil/Tediye zorunlu; Mahsup esnek; Açılış/Kapanış muaf.
    pub fn dayanak_zorunlu(&self) -> bool {
        matches!(self, FisTipi::Tahsil | FisTipi::Tediye)
    }

    /// Sistemsel (dayanak gerekmez, kaynağı dönem mekanizması) mi?
    pub fn sistemsel(&self) -> bool {
        matches!(self, FisTipi::Acilis | FisTipi::Kapanis)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FisDurum {
    Taslak,
    Kesin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tarih {
    pub yil: i32,
    pub ay: u8,
    pub gun: u8,
}

impl Tarih {
    pub fn yeni(yil: i32, ay: u8, gun: u8) -> Self {
        Tarih { yil, ay, gun }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BelgeTipi {
    Fatura,
    EFatura,
    Makbuz,
    Dekont,
    PosFisi,
    Sozlesme,
    Bordro,
    Beyanname,
    Diger,
}

#[derive(Debug, Clone)]
pub struct Dayanak {
    pub belge_tipi: BelgeTipi,
    pub belge_no: String,
    pub belge_tarihi: Tarih,
}

#[derive(Debug, Clone)]
pub struct YevmiyeSatiri {
    pub hesap_id: HesapId,
    pub borc: Kurus,
    pub alacak: Kurus,
    pub masraf_yeri_id: Option<MasrafYeriId>,
    pub aciklama: String,
}

impl YevmiyeSatiri {
    pub fn borc(hesap_id: HesapId, tutar: Kurus) -> Self {
        YevmiyeSatiri {
            hesap_id,
            borc: tutar,
            alacak: Kurus::SIFIR,
            masraf_yeri_id: None,
            aciklama: String::new(),
        }
    }

    pub fn alacak(hesap_id: HesapId, tutar: Kurus) -> Self {
        YevmiyeSatiri {
            hesap_id,
            borc: Kurus::SIFIR,
            alacak: tutar,
            masraf_yeri_id: None,
            aciklama: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Fis {
    pub tip: FisTipi,
    pub durum: FisDurum,
    pub tarih: Tarih,
    pub satirlar: Vec<YevmiyeSatiri>,
    pub dayanak: Option<Dayanak>,
    /// Kesinleştirmede, zorunlu olmayan tipte dayanak yoksa true işaretlenir (denetim raporlar).
    pub dayanaksiz: bool,
    pub seri: Option<String>,
    pub sira: Option<u64>,
    /// Yevmiye madde numarası — TÜM fişlerde müteselsil (VUK). Kesinleştirmede atanır.
    pub yevmiye_no: Option<u64>,
    /// İptal kaydı ise, iptal edilen kaynak fişin id'si.
    pub iptal_kaynak_id: Option<FisId>,
    /// Bu fiş iptal edildi mi (ters kayıtla)?
    pub iptal: bool,
    pub aciklama: String,
}

impl Fis {
    pub fn taslak(tip: FisTipi, tarih: Tarih, satirlar: Vec<YevmiyeSatiri>) -> Self {
        Fis {
            tip,
            durum: FisDurum::Taslak,
            tarih,
            satirlar,
            dayanak: None,
            dayanaksiz: false,
            seri: None,
            sira: None,
            yevmiye_no: None,
            iptal_kaynak_id: None,
            iptal: false,
            aciklama: String::new(),
        }
    }

    /// Builder: dayanak ekle.
    pub fn ile_dayanak(mut self, d: Dayanak) -> Self {
        self.dayanak = Some(d);
        self
    }

    pub fn toplam_borc(&self) -> Kurus {
        self.satirlar.iter().map(|s| s.borc).sum()
    }

    pub fn toplam_alacak(&self) -> Kurus {
        self.satirlar.iter().map(|s| s.alacak).sum()
    }
}
