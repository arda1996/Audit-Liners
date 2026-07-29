//! Kredi/Faiz motoru — ödeme planı (amortisman) + faiz + BSMV; **öneri üretir, karar vermez**.
//!
//! Firmalar farklı yöntemlerle kredilenir (annüite / eşit anapara / spot; rotatif/dövizli/leasing
//! sonraki faz). Motor ödeme planını hesaplar ve tahakkuk esasına göre atılması gereken kayıtları
//! [`OnerilenKayit`] olarak sunar (bkz. [`crate::oneri`]); deftere ASLA doğrudan yazmaz.
//!
//! **Para bütünlüğü:** annüite faktörü doğası gereği reel aritmetik ister — f64 yalnız *geçici*
//! faktör/faiz hesabında kullanılır, her tutar **kuruşa yuvarlanır** ve **son taksit artığı kapatır**
//! (Σanapara = anapara TAM eşit). Saklanan/döndürülen her değer tamsayı [`Kurus`]. Float depolanmaz.
//!
//! **Vade dilimi (300/303/400):** motor öneride bulunur + gerekçe taşır (≤12 ay 300; uzun vadede
//! gelecek 12 ay 303, ötesi 400) ama seçimi muhasebeciye bırakır — [`crate::oneri`] felsefesi.

use crate::fis::{Dayanak, BelgeTipi, Fis, FisTipi, Tarih, YevmiyeSatiri};
use crate::oneri::{Guven, OneriTuru, OnerilenKayit};
use crate::para::Kurus;

/// Amortisman (geri ödeme) yöntemi.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OdemeYontemi {
    /// Eşit taksit (annüite) — Türk ticari kredilerinde yaygın; taksit sabit, faiz azalır.
    Annuite,
    /// Eşit anapara (azalan taksit) — anapara her dönem sabit, faiz azaldıkça taksit düşer.
    EsitAnapara,
    /// Spot/bullet — faiz dönemsel ödenir, anapara vade sonunda tek seferde.
    Spot,
}

/// Değişken/kademeli faiz dilimi: `ay` ay boyunca `yillik_bp` uygulanır (ör. ilk 12 ay %5, sonra %3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FaizDilimi {
    pub ay: u32,
    pub yillik_bp: u32,
}

/// Kredi tanımı. Faiz **yıllık baz puan** (bp): 4500 = %45. BSMV bp: 500 = %5 (faiz üzerinden).
#[derive(Debug, Clone)]
pub struct Kredi {
    pub anapara: Kurus,
    pub yillik_faiz_bp: u32,
    pub vade_ay: u32,
    pub baslangic: Tarih,
    pub yontem: OdemeYontemi,
    /// Faiz üzerinden BSMV (6802). Ticari kredi: KKDF %0 (dahil edilmez); BSMV %5 tipiktir.
    pub bsmv_bp: u32,
    /// Boşsa sabit `yillik_faiz_bp`. Doluysa kademeli faiz: her ay ilgili dilimin oranı uygulanır;
    /// diliml toplamı vadeden kısaysa son dilim kalan aylara taşar, uzunsa vadeye kırpılır.
    pub faiz_dilimleri: Vec<FaizDilimi>,
}

impl Kredi {
    /// Her aya (1..=vade_ay) düşen aylık oranı üretir (yıllık bp → aylık f64). Kademeli faizi açar.
    fn aylik_oranlar(&self) -> Vec<f64> {
        let n = self.vade_ay.max(1) as usize;
        if self.faiz_dilimleri.is_empty() {
            return vec![self.yillik_faiz_bp as f64 / 10_000.0 / 12.0; n];
        }
        let mut oranlar = Vec::with_capacity(n);
        for d in &self.faiz_dilimleri {
            let r = d.yillik_bp as f64 / 10_000.0 / 12.0;
            for _ in 0..d.ay {
                if oranlar.len() < n {
                    oranlar.push(r);
                }
            }
        }
        // Dilimler vadeyi doldurmadıysa son dilimin oranıyla tamamla.
        let son = *oranlar.last().unwrap_or(&(self.yillik_faiz_bp as f64 / 10_000.0 / 12.0));
        while oranlar.len() < n {
            oranlar.push(son);
        }
        oranlar
    }
}

/// Annüite sabit taksiti (kalan anapara, aylık oran, kalan ay) — kuruş. Oran değişince yeniden çağrılır.
fn annuite_taksit(kalan: i64, r: f64, kalan_ay: u32) -> i64 {
    if kalan_ay == 0 {
        return kalan;
    }
    if r > 0.0 {
        let f = r * (1.0 + r).powi(kalan_ay as i32) / ((1.0 + r).powi(kalan_ay as i32) - 1.0);
        yuvarla(kalan as f64 * f)
    } else {
        (kalan + kalan_ay as i64 - 1) / kalan_ay as i64
    }
}

/// Ödeme planındaki tek taksit satırı — tüm tutarlar tamsayı kuruş.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaksitSatiri {
    pub no: u32,
    pub tarih: Tarih,
    /// Toplam ödeme = anapara + faiz + bsmv.
    pub taksit: Kurus,
    pub anapara: Kurus,
    pub faiz: Kurus,
    pub bsmv: Kurus,
    /// Bu taksit sonrası kalan anapara.
    pub kalan_anapara: Kurus,
}

fn yuvarla(x: f64) -> i64 {
    x.round() as i64
}

/// Bir tarihe `ay` ekle. Gün, hedef ayın son gününü aşarsa son güne kıstırılır (31 Oca +1 ay → 28/29 Şub).
pub fn ay_ekle(t: Tarih, ay: u32) -> Tarih {
    let toplam0 = (t.yil as i64) * 12 + (t.ay as i64 - 1) + ay as i64;
    let yil = (toplam0.div_euclid(12)) as i32;
    let ay_yeni = (toplam0.rem_euclid(12)) as u8 + 1;
    let son = ayin_son_gunu(yil, ay_yeni);
    Tarih::yeni(yil, ay_yeni, t.gun.min(son))
}

fn ayin_son_gunu(yil: i32, ay: u8) -> u8 {
    match ay {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (yil % 4 == 0 && yil % 100 != 0) || yil % 400 == 0 {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

/// Ödeme planı (amortisman tablosu) üretir. Σanapara = anapara (son satır artığı kapatır).
/// Kademeli faizi (`faiz_dilimleri`) destekler: annüitede oran değiştiğinde sabit taksit,
/// kalan anapara ve kalan vade için yeniden hesaplanır (bankaların uyguladığı yöntem).
pub fn odeme_plani(k: &Kredi) -> Vec<TaksitSatiri> {
    let n = k.vade_ay.max(1);
    let p = k.anapara.deger();
    let oranlar = k.aylik_oranlar(); // her aya düşen aylık oran
    let bsmv_oran = k.bsmv_bp as f64 / 10_000.0;
    let esit_anapara = p / n as i64;
    let mut plan = Vec::with_capacity(n as usize);
    let mut kalan = p;
    let mut taksit_sabit = 0i64;
    let mut onceki_oran = f64::NAN;

    for i in 1..=n {
        let son = i == n;
        let r = oranlar[(i - 1) as usize];
        let faiz = yuvarla(kalan as f64 * r);
        let (anapara, taksit_anapara_faiz) = match k.yontem {
            OdemeYontemi::Annuite => {
                if son {
                    (kalan, kalan + faiz)
                } else {
                    // Oran değiştiyse sabit taksiti kalan bakiye + kalan vade için yeniden hesapla.
                    if r != onceki_oran {
                        taksit_sabit = annuite_taksit(kalan, r, n - (i - 1));
                        onceki_oran = r;
                    }
                    let ana = (taksit_sabit - faiz).clamp(0, kalan);
                    (ana, ana + faiz)
                }
            }
            OdemeYontemi::EsitAnapara => {
                let ana = if son { kalan } else { esit_anapara.min(kalan) };
                (ana, ana + faiz)
            }
            OdemeYontemi::Spot => {
                let ana = if son { kalan } else { 0 };
                (ana, ana + faiz)
            }
        };
        let bsmv = yuvarla(faiz as f64 * bsmv_oran);
        kalan -= anapara;
        plan.push(TaksitSatiri {
            no: i,
            tarih: ay_ekle(k.baslangic, i),
            taksit: Kurus::yeni(taksit_anapara_faiz + bsmv),
            anapara: Kurus::yeni(anapara),
            faiz: Kurus::yeni(faiz),
            bsmv: Kurus::yeni(bsmv),
            kalan_anapara: Kurus::yeni(kalan),
        });
    }
    plan
}

/// Kredinin vade sınıfı — kullanım anındaki orijinal vadeye göre (bilgi/gerekçe amaçlı).
/// ≤12 ay: kısa (300). >12 ay: uzun (400) + gelecek 12 ay 303'e devreder (dönem sonu, muhasebeci kararı).
pub fn vade_gerekce(k: &Kredi) -> String {
    if k.vade_ay <= 12 {
        "Vade ≤ 12 ay → kısa vadeli (300). Muhasebeci onayına tabi.".to_string()
    } else {
        format!(
            "Vade {} ay > 12 → uzun vadeli (400); gelecek 12 aya isabet eden anapara dönem sonunda 303'e \
             devreder (kısa vadeye aktarım). Vade dilimi/hesap seçimi muhasebeci kararıdır.",
            k.vade_ay
        )
    }
}

fn banka_dekont(no: &str, tarih: Tarih) -> Dayanak {
    Dayanak { belge_tipi: BelgeTipi::Dekont, belge_no: no.to_string(), belge_tarihi: tarih }
}

/// Verilen ödeme planından (hesaplanmış VEYA bankadan yüklenmiş) tahakkuk esasına göre önerilen
/// kayıtları üretir: kullanım + her taksit. Plan kaynağından bağımsız çalışır → yükleme yolu ile
/// program hesaplaması aynı motoru kullanır. `plan_kaynak` gerekçeye/izlenebilirliğe yazılır
/// (ör. "program hesaplaması" / "banka planı yüklendi").
pub fn oneriler_plandan(
    anapara: Kurus,
    baslangic: Tarih,
    vade_ay: u32,
    plan: &[TaksitSatiri],
    kredi_no: &str,
    plan_kaynak: &str,
) -> Vec<OnerilenKayit> {
    let mut cikti = Vec::new();
    let ana_hesap = if vade_ay <= 12 { "300" } else { "400" };
    let vade_not = if vade_ay <= 12 {
        format!("Vade ≤ 12 ay → kısa vadeli (300). Plan kaynağı: {plan_kaynak}. Muhasebeci onayına tabi.")
    } else {
        format!(
            "Vade {vade_ay} ay > 12 → uzun vadeli (400); gelecek 12 aya isabet eden anapara dönem \
             sonunda 303'e devreder. Plan kaynağı: {plan_kaynak}. Vade dilimi/hesap seçimi muhasebeci kararıdır."
        )
    };

    // 1) Kredi kullanımı (anapara girişi): 102 borç / 300|400 alacak
    let kullanim_fis = Fis::taslak(
        FisTipi::Mahsup,
        baslangic,
        vec![
            YevmiyeSatiri::borc("102".to_string(), anapara),
            YevmiyeSatiri::alacak(ana_hesap.to_string(), anapara),
        ],
    )
    .ile_dayanak(banka_dekont(kredi_no, baslangic));
    cikti.push(
        OnerilenKayit::yeni(
            OneriTuru::KrediKullanim,
            baslangic,
            format!("Kredi kullanımı ({kredi_no})"),
            vade_not,
            kullanim_fis,
            Guven::Orta,
        )
        .ile_kaynak(kredi_no),
    );

    // 2) Her taksit: 300|400 (anapara) + 780 (faiz) + 780 (BSMV) borç / 102 alacak
    for t in plan {
        let mut satirlar = Vec::new();
        if t.anapara.pozitif() {
            satirlar.push(YevmiyeSatiri::borc(ana_hesap.to_string(), t.anapara));
        }
        if t.faiz.pozitif() {
            satirlar.push(YevmiyeSatiri::borc("780".to_string(), t.faiz)); // finansman gideri (faiz)
        }
        if t.bsmv.pozitif() {
            satirlar.push(YevmiyeSatiri::borc("780".to_string(), t.bsmv)); // faiz BSMV'si (780.BSMV)
        }
        satirlar.push(YevmiyeSatiri::alacak("102".to_string(), t.taksit));
        let fis = Fis::taslak(FisTipi::Mahsup, t.tarih, satirlar)
            .ile_dayanak(banka_dekont(kredi_no, t.tarih));
        cikti.push(
            OnerilenKayit::yeni(
                OneriTuru::KrediTaksit,
                t.tarih,
                format!("Kredi taksiti #{} ({kredi_no})", t.no),
                "Tahakkuk esası: taksit ödemesi. Faiz+BSMV 780'e; anapara vade sınıfına göre 300/303/400 \
                 (muhasebeci kararı). BSMV indirilemez (191'e gitmez)."
                    .to_string(),
                fis,
                Guven::Yuksek,
            )
            .ile_kaynak(kredi_no),
        );
    }
    cikti
}

/// Program hesaplamasından öneriler (parametrik yol). Yükleme yolu için bkz. [`oneriler_plandan`].
pub fn oneriler(k: &Kredi, kredi_no: &str) -> Vec<OnerilenKayit> {
    oneriler_plandan(
        k.anapara,
        k.baslangic,
        k.vade_ay,
        &odeme_plani(k),
        kredi_no,
        "program hesaplaması",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kredi(yontem: OdemeYontemi) -> Kredi {
        Kredi {
            anapara: Kurus::yeni(100_000_00), // 100.000 TL
            yillik_faiz_bp: 4800,             // %48 → aylık %4
            vade_ay: 12,
            baslangic: Tarih::yeni(2026, 1, 15),
            yontem,
            bsmv_bp: 500, // %5
            faiz_dilimleri: vec![],
        }
    }

    #[test]
    fn anapara_tam_kapanir_annuite() {
        let plan = odeme_plani(&kredi(OdemeYontemi::Annuite));
        assert_eq!(plan.len(), 12);
        let toplam_anapara: i64 = plan.iter().map(|t| t.anapara.deger()).sum();
        assert_eq!(toplam_anapara, 100_000_00, "Σanapara = anapara tam eşit olmalı");
        assert_eq!(plan.last().unwrap().kalan_anapara, Kurus::SIFIR, "son satır kalanı 0");
    }

    #[test]
    fn anapara_tam_kapanir_esit_anapara() {
        let plan = odeme_plani(&kredi(OdemeYontemi::EsitAnapara));
        let toplam: i64 = plan.iter().map(|t| t.anapara.deger()).sum();
        assert_eq!(toplam, 100_000_00);
        // Eşit anapara: ilk taksit son taksitten büyük (faiz azalır).
        assert!(plan[0].taksit.deger() > plan[11].taksit.deger());
    }

    #[test]
    fn spot_anapara_vade_sonunda() {
        let plan = odeme_plani(&kredi(OdemeYontemi::Spot));
        for t in &plan[..11] {
            assert_eq!(t.anapara, Kurus::SIFIR, "spot: ara taksitlerde anapara yok");
            assert!(t.faiz.pozitif(), "spot: her ay faiz var");
        }
        assert_eq!(plan[11].anapara, Kurus::yeni(100_000_00), "anapara son ay");
    }

    #[test]
    fn faiz_ve_bsmv_azalir_annuite() {
        let plan = odeme_plani(&kredi(OdemeYontemi::Annuite));
        // Faiz kalan anapara üzerinden → ilk ay > son ay.
        assert!(plan[0].faiz.deger() > plan[11].faiz.deger());
        // BSMV = faiz × %5 (ilk ay: faiz 4000.00 TL → 200.00 TL BSMV)
        assert_eq!(plan[0].bsmv.deger(), (plan[0].faiz.deger() as f64 * 0.05).round() as i64);
    }

    #[test]
    fn ilk_ay_faizi_dogru() {
        // Aylık %4 × 100.000 = 4.000 TL faiz ilk ay.
        let plan = odeme_plani(&kredi(OdemeYontemi::Annuite));
        assert_eq!(plan[0].faiz, Kurus::yeni(4_000_00));
    }

    #[test]
    fn ay_ekle_ay_sonu_kistirma() {
        // 31 Ocak + 1 ay = 28 Şubat (2026 artık yıl değil).
        assert_eq!(ay_ekle(Tarih::yeni(2026, 1, 31), 1), Tarih::yeni(2026, 2, 28));
        // 15 Aralık + 2 ay = 15 Şubat sonraki yıl.
        assert_eq!(ay_ekle(Tarih::yeni(2026, 12, 15), 2), Tarih::yeni(2027, 2, 15));
    }

    #[test]
    fn oneriler_kullanim_ve_taksit() {
        let k = kredi(OdemeYontemi::Annuite);
        let o = oneriler(&k, "KRD-2026-001");
        assert_eq!(o.len(), 13); // 1 kullanım + 12 taksit
        assert_eq!(o[0].tur, OneriTuru::KrediKullanim);
        // Her önerilen fiş dengeli olmalı (Σborç = Σalacak).
        for x in &o {
            assert_eq!(
                x.onerilen_fis.toplam_borc(),
                x.onerilen_fis.toplam_alacak(),
                "önerilen fiş dengeli olmalı"
            );
        }
    }

    #[test]
    fn uzun_vade_400_onerilir() {
        let mut k = kredi(OdemeYontemi::Annuite);
        k.vade_ay = 18;
        let o = oneriler(&k, "KRD-2026-002");
        // Kullanım fişinde uzun vade → 400 alacak.
        let alacak_hesap = &o[0].onerilen_fis.satirlar[1].hesap_id;
        assert_eq!(alacak_hesap, "400");
        assert!(o[0].gerekce.contains("303"), "uzun vadede 303 devri gerekçede anılmalı");
    }

    #[test]
    fn degisken_faiz_dilimleri() {
        // İlk 6 ay %60 (aylık %5), sonraki 6 ay %36 (aylık %3) — kademeli faiz.
        let mut k = kredi(OdemeYontemi::EsitAnapara);
        k.yillik_faiz_bp = 0; // kullanılmaz (dilimler dolu)
        k.faiz_dilimleri = vec![
            FaizDilimi { ay: 6, yillik_bp: 6000 },
            FaizDilimi { ay: 6, yillik_bp: 3600 },
        ];
        let plan = odeme_plani(&k);
        // Eşit anapara: her ay 100.000/12 anapara sabit.
        let toplam: i64 = plan.iter().map(|t| t.anapara.deger()).sum();
        assert_eq!(toplam, 100_000_00);
        // 1. ay faizi: kalan 100.000 × %5 = 5.000 TL; 7. ay artık %3 oranına düşmüş olmalı.
        assert_eq!(plan[0].faiz, Kurus::yeni(5_000_00));
        // 7. ayda oran %3 → faiz, aynı kalanda %5 olsaydından düşük.
        let kalan6 = plan[5].kalan_anapara.deger();
        assert_eq!(plan[6].faiz, Kurus::yeni((kalan6 as f64 * 0.03).round() as i64));
    }

    #[test]
    fn dilim_vadeden_kisa_ise_son_dilim_tasar() {
        // Tek dilim 4 ay verilmiş ama vade 12 ay → kalan 8 ay son dilimin oranıyla.
        let mut k = kredi(OdemeYontemi::EsitAnapara);
        k.faiz_dilimleri = vec![FaizDilimi { ay: 4, yillik_bp: 2400 }]; // aylık %2
        let plan = odeme_plani(&k);
        assert_eq!(plan.len(), 12);
        // 1. ay faizi 100.000 × %2 = 2.000 TL.
        assert_eq!(plan[0].faiz, Kurus::yeni(2_000_00));
    }

    #[test]
    fn yukleme_yolu_ayni_motoru_kullanir() {
        // Muhasebeci bankanın planını yükledi (elle kurulmuş 2 satırlık plan).
        let plan = vec![
            TaksitSatiri {
                no: 1,
                tarih: Tarih::yeni(2026, 2, 15),
                taksit: Kurus::yeni(52_100_00),
                anapara: Kurus::yeni(50_000_00),
                faiz: Kurus::yeni(2_000_00),
                bsmv: Kurus::yeni(100_00),
                kalan_anapara: Kurus::yeni(50_000_00),
            },
            TaksitSatiri {
                no: 2,
                tarih: Tarih::yeni(2026, 3, 15),
                taksit: Kurus::yeni(51_050_00),
                anapara: Kurus::yeni(50_000_00),
                faiz: Kurus::yeni(1_000_00),
                bsmv: Kurus::yeni(50_00),
                kalan_anapara: Kurus::SIFIR,
            },
        ];
        let o = oneriler_plandan(
            Kurus::yeni(100_000_00),
            Tarih::yeni(2026, 1, 15),
            6,
            &plan,
            "KRD-YUK-001",
            "banka planı yüklendi",
        );
        assert_eq!(o.len(), 3); // 1 kullanım + 2 taksit
        assert!(o[0].gerekce.contains("banka planı yüklendi"));
        for x in &o {
            assert_eq!(x.onerilen_fis.toplam_borc(), x.onerilen_fis.toplam_alacak());
        }
    }
}
