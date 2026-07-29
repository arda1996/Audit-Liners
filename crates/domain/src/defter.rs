//! Defter motoru: kesin fişlerden türetilmiş defterler — mizan, defter-i kebir, yevmiye, bakiye.
//! Saklanan tek doğruluk kaynağı Fiş+YevmiyeSatırı; buradakiler hep sorgulanır (saklanmaz).

use std::collections::BTreeMap;

use crate::fis::{Fis, Tarih, YevmiyeSatiri};
use crate::hesap::HesapId;
use crate::kurallar::fis_no;
use crate::para::Kurus;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MizanSatiri {
    pub hesap_id: HesapId,
    pub borc: Kurus,
    pub alacak: Kurus,
    pub borc_bakiye: Kurus,
    pub alacak_bakiye: Kurus,
}

#[derive(Debug, Clone)]
pub struct KebirHareket {
    pub tarih: Tarih,
    /// Yevmiye madde no — kebir↔yevmiye çapraz referansı (yasal format).
    pub yevmiye_no: Option<u64>,
    pub fis_no: String,
    pub aciklama: String,
    /// Karşı bacak hesap kodları (profesyonel muavin/kebir kolonu).
    pub karsi: String,
    pub borc: Kurus,
    pub alacak: Kurus,
    /// Yürüyen bakiye: borç(+) / alacak(−) yönlü, kuruş.
    pub yuruyen_bakiye: i64,
}

/// `kod`, `onek` hesabının altında mı? Sayısal grup (12→120) veya muavin (120→120.01).
fn altinda(kod: &str, onek: &str) -> bool {
    if kod == onek {
        return true;
    }
    if onek.contains('.') {
        // muavin alt-kırılımı: 120.01 → 120.01.001
        kod.starts_with(&format!("{}.", onek))
    } else {
        // sayısal grup: ana-kod (dot öncesi) önek ile başlıyor mu? (120 → 12, 1)
        let ana = kod.split('.').next().unwrap_or(kod);
        ana.starts_with(onek)
    }
}

/// Kesin fişlerin defteri. Türetilmiş görünümleri (mizan/kebir/yevmiye) üretir.
#[derive(Default)]
pub struct Defter {
    pub fisler: Vec<Fis>,
}

impl Defter {
    pub fn yeni() -> Self {
        Self::default()
    }

    pub fn ekle(&mut self, fis: Fis) {
        self.fisler.push(fis);
    }

    /// Tek hesabın (borç toplamı, alacak toplamı).
    pub fn hesap_bakiyesi(&self, hesap_id: &str) -> (Kurus, Kurus) {
        let mut b = 0i64;
        let mut a = 0i64;
        for f in &self.fisler {
            for s in &f.satirlar {
                if s.hesap_id == hesap_id {
                    b += s.borc.deger();
                    a += s.alacak.deger();
                }
            }
        }
        (Kurus::yeni(b), Kurus::yeni(a))
    }

    /// Rollup bakiye: önek altındaki tüm hesapların toplamı. İki hiyerarşiyi birleştirir:
    /// sayısal grup (12 → 120, 121…) ve muavin (120 → 120.01, 120.02…).
    pub fn bakiye_rollup(&self, kod_onek: &str) -> (Kurus, Kurus) {
        let mut b = 0i64;
        let mut a = 0i64;
        for f in &self.fisler {
            for s in &f.satirlar {
                if altinda(&s.hesap_id, kod_onek) {
                    b += s.borc.deger();
                    a += s.alacak.deger();
                }
            }
        }
        (Kurus::yeni(b), Kurus::yeni(a))
    }

    /// Mizan: her hesabın borç/alacak toplamı + borç/alacak bakiyesi (kod sırasıyla).
    pub fn mizan(&self) -> Vec<MizanSatiri> {
        let mut m: BTreeMap<HesapId, (i64, i64)> = BTreeMap::new();
        for f in &self.fisler {
            for s in &f.satirlar {
                let e = m.entry(s.hesap_id.clone()).or_insert((0, 0));
                e.0 += s.borc.deger();
                e.1 += s.alacak.deger();
            }
        }
        m.into_iter()
            .map(|(id, (b, a))| {
                let net = b - a;
                MizanSatiri {
                    hesap_id: id,
                    borc: Kurus::yeni(b),
                    alacak: Kurus::yeni(a),
                    borc_bakiye: Kurus::yeni(net.max(0)),
                    alacak_bakiye: Kurus::yeni((-net).max(0)),
                }
            })
            .collect()
    }

    /// Defter-i kebir: bir hesabın hareketleri (tarih/fiş sırasıyla) + yürüyen bakiye.
    pub fn kebir(&self, hesap_id: &str) -> Vec<KebirHareket> {
        let mut hareketler: Vec<(&Fis, &YevmiyeSatiri)> = Vec::new();
        for f in &self.fisler {
            for s in &f.satirlar {
                if s.hesap_id == hesap_id {
                    hareketler.push((f, s));
                }
            }
        }
        hareketler.sort_by(|x, y| x.0.tarih.cmp(&y.0.tarih).then(x.0.sira.cmp(&y.0.sira)));

        let mut yur = 0i64;
        hareketler
            .into_iter()
            .map(|(f, s)| {
                yur += s.borc.deger() - s.alacak.deger();
                // Karşı bacak: fişte bu satırın TERS tarafındaki hesaplar
                let bu_borc = s.borc.pozitif();
                let mut karsi: Vec<String> = f
                    .satirlar
                    .iter()
                    .filter(|x| x.borc.pozitif() != bu_borc)
                    .map(|x| x.hesap_id.clone())
                    .collect();
                karsi.dedup();
                KebirHareket {
                    tarih: f.tarih,
                    yevmiye_no: f.yevmiye_no,
                    fis_no: fis_no(f).unwrap_or_default(),
                    aciklama: s.aciklama.clone(),
                    karsi: karsi.join(", "),
                    borc: s.borc,
                    alacak: s.alacak,
                    yuruyen_bakiye: yur,
                }
            })
            .collect()
    }

    /// Yevmiye defteri: fişler tarih (sonra seri/sıra) sırasıyla.
    pub fn yevmiye(&self) -> Vec<&Fis> {
        let mut v: Vec<&Fis> = self.fisler.iter().collect();
        v.sort_by(|x, y| {
            x.tarih
                .cmp(&y.tarih)
                .then(x.seri.cmp(&y.seri))
                .then(x.sira.cmp(&y.sira))
        });
        v
    }

    /// Mizan dengeli mi? (toplam borç = toplam alacak — bütünlük kontrolü)
    pub fn dengeli(&self) -> bool {
        let (mut b, mut a) = (0i64, 0i64);
        for f in &self.fisler {
            for s in &f.satirlar {
                b += s.borc.deger();
                a += s.alacak.deger();
            }
        }
        b == a
    }
}
