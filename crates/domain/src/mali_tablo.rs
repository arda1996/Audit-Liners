//! Mali tablolar: mizandan bilanço ve gelir tablosu türetimi (TDHP sınıf/grup eşlemesiyle).
//! Tutarlar kuruş. Bilanço dönem kârını öz kaynağa ekleyerek dengelenir (kapanış öncesi de geçerli).

use crate::defter::Defter;
use crate::donem::donem_sonucu;

/// Önek altındaki net bakiye (borç(+) / alacak(−)), kuruş.
fn net(defter: &Defter, onek: &str) -> i64 {
    let (b, a) = defter.bakiye_rollup(onek);
    b.deger() - a.deger()
}

#[derive(Debug, Clone)]
pub struct Bilanco {
    pub donen_varliklar: i64,  // sınıf 1
    pub duran_varliklar: i64,  // sınıf 2
    pub aktif_toplam: i64,
    pub kisa_vadeli_yk: i64,   // sınıf 3
    pub uzun_vadeli_yk: i64,   // sınıf 4
    pub oz_kaynaklar: i64,     // sınıf 5 (dönem kârı hariç)
    pub donem_kari: i64,       // dönem net sonucu (öz kaynağa dahil)
    pub pasif_toplam: i64,
}

/// Bilanço. Aktif (1,2) borç bakiyeli, pasif (3,4,5) alacak bakiyeli. Dönem kârı öz kaynağa eklenir.
pub fn bilanco(defter: &Defter) -> Bilanco {
    let donen = net(defter, "1");
    let duran = net(defter, "2");
    let kvyk = -net(defter, "3");
    let uvyk = -net(defter, "4");
    let oz = -net(defter, "5");
    let kar = donem_sonucu(defter);
    Bilanco {
        donen_varliklar: donen,
        duran_varliklar: duran,
        aktif_toplam: donen + duran,
        kisa_vadeli_yk: kvyk,
        uzun_vadeli_yk: uvyk,
        oz_kaynaklar: oz,
        donem_kari: kar,
        pasif_toplam: kvyk + uvyk + oz + kar,
    }
}

#[derive(Debug, Clone)]
pub struct GelirTablosu {
    pub brut_satislar: i64,       // 60
    pub satis_indirimleri: i64,   // 61
    pub net_satislar: i64,
    pub satislarin_maliyeti: i64, // 62
    pub brut_satis_kari: i64,
    pub faaliyet_giderleri: i64,  // 63
    pub faaliyet_kari: i64,
    pub diger_olagan_gelir: i64,  // 64
    pub diger_olagan_gider: i64,  // 65
    pub finansman_giderleri: i64, // 66
    pub olagan_kar: i64,
    pub olagandisi_gelir: i64,    // 67
    pub olagandisi_gider: i64,    // 68
    /// Vergi ÖNCESİ dönem kârı (690 karşılığı).
    pub donem_kari: i64,
    /// 691 Dönem kârı vergi ve diğer yasal yükümlülük karşılıkları (-).
    pub vergi_karsiligi: i64,
    /// Vergi SONRASI net kâr (692 ile mutabık olmalı).
    pub donem_net_kari: i64,
}

/// Gelir tablosu (özet). `donem_kari` == `donem_sonucu(defter)` olmalıdır.
pub fn gelir_tablosu(defter: &Defter) -> GelirTablosu {
    let brut = -net(defter, "60");
    let indirim = net(defter, "61");
    let net_satis = brut - indirim;
    let smm = net(defter, "62");
    let brut_kar = net_satis - smm;
    let faal_gid = net(defter, "63");
    let faal_kar = brut_kar - faal_gid;
    let dig_gelir = -net(defter, "64");
    let dig_gider = net(defter, "65");
    let finansman = net(defter, "66");
    let olagan = faal_kar + dig_gelir - dig_gider - finansman;
    let od_gelir = -net(defter, "67");
    let od_gider = net(defter, "68");
    let donem = olagan + od_gelir - od_gider;
    let vergi = net(defter, "691"); // 691 borç çalışır (kapanışta 692'ye devredilir)
    GelirTablosu {
        brut_satislar: brut,
        satis_indirimleri: indirim,
        net_satislar: net_satis,
        satislarin_maliyeti: smm,
        brut_satis_kari: brut_kar,
        faaliyet_giderleri: faal_gid,
        faaliyet_kari: faal_kar,
        diger_olagan_gelir: dig_gelir,
        diger_olagan_gider: dig_gider,
        finansman_giderleri: finansman,
        olagan_kar: olagan,
        olagandisi_gelir: od_gelir,
        olagandisi_gider: od_gider,
        donem_kari: donem,
        vergi_karsiligi: vergi,
        donem_net_kari: donem - vergi,
    }
}
