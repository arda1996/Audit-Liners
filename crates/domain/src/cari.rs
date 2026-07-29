//! Cari (müşteri/satıcı) — muavin hesabı üstüne kart. Bakiye/ekstre defterden gelir.
//! Alıcı = 120.xx muavini, Satıcı = 320.xx muavini.

use crate::defter::Defter;
use crate::hesap::Hesap;
use crate::hesap_plani::muavin_olustur;
use crate::para::Kurus;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CariTip {
    Alici,
    Satici,
}

#[derive(Debug, Clone)]
pub struct Cari {
    /// Muavin kod, ör. "120.01".
    pub kod: String,
    pub unvan: String,
    pub vkn_tckn: String,
    pub tip: CariTip,
}

/// Ana hesap (120 Alıcılar / 320 Satıcılar) altında cari kart + muavin hesabını üretir.
/// Muavin adı = cari unvanı. Tip ana hesabın sınıfından türetilir (1→Alıcı, 3→Satıcı).
pub fn cari_olustur(ana: &Hesap, alt_kod: &str, unvan: &str, vkn_tckn: &str) -> (Cari, Hesap) {
    let muavin = muavin_olustur(ana, alt_kod, unvan);
    let tip = if ana.kod.starts_with('1') {
        CariTip::Alici
    } else {
        CariTip::Satici
    };
    let cari = Cari {
        kod: muavin.kod.clone(),
        unvan: unvan.to_string(),
        vkn_tckn: vkn_tckn.to_string(),
        tip,
    };
    (cari, muavin)
}

impl Cari {
    /// Cari bakiye (borç toplamı, alacak toplamı) — defterden.
    pub fn bakiye(&self, defter: &Defter) -> (Kurus, Kurus) {
        defter.hesap_bakiyesi(&self.kod)
    }

    /// Net bakiye, kuruş: borç(+) / alacak(−). Alıcıda (+) = bizden alacaklı→bize borçlu.
    pub fn net_bakiye(&self, defter: &Defter) -> i64 {
        let (b, a) = defter.hesap_bakiyesi(&self.kod);
        b.deger() - a.deger()
    }

    /// Cari ekstre = muavin hesabının defter-i kebiri (hareketler + yürüyen bakiye).
    pub fn ekstre(&self, defter: &Defter) -> Vec<crate::defter::KebirHareket> {
        defter.kebir(&self.kod)
    }
}
