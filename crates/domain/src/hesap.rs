//! Hesap (account) — TDHP'ye göre. Kayıt yalnızca yaprak (muavin) hesaba yapılır.

/// Hesap kimliği = kod (metin). Muavin kodları (120.01) sayı olamaz; rollup kod-önekiyle yapılır.
pub type HesapId = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HesapDogasi {
    Borc,
    Alacak,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HesapTipi {
    Aktif,
    Pasif,
    Gelir,
    Gider,
    Maliyet,
    Nazim,
}

impl HesapTipi {
    /// Gider/maliyet hesabı mı? (masraf yeri boyutu bu hesaplarda anlamlı — sınıf 6 gider, 7)
    pub fn gider_maliyet(&self) -> bool {
        matches!(self, HesapTipi::Gider | HesapTipi::Maliyet)
    }
}

#[derive(Debug, Clone)]
pub struct Hesap {
    pub id: HesapId,
    pub kod: String,
    pub ad: String,
    pub tip: HesapTipi,
    pub dogasi: HesapDogasi,
    /// Yaprak (muavin) hesap mı? Yalnızca yaprağa kayıt yapılabilir.
    pub yaprak: bool,
    pub aktif: bool,
}
