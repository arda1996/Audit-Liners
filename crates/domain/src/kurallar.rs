//! Kayıt kuralları motoru (0.2): doğrulama V1–V7, kesinleştirme, numaralama, iptal (düzeltme kaydı), açılış.
//! Saf — yan etkisiz fonksiyonlar; DB yok. (bkz. docs/tasarim/02-kayit-kurallari.md)

use std::collections::HashMap;

use crate::fis::*;
use crate::hesap::*;
use crate::para::Kurus;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KayitHatasi {
    /// Fiş zaten KESİN — düzenlenemez/yeniden kesinleştirilemez (V: değişmezlik).
    ZatenKesin,
    /// En az 2 satır olmalı (V2).
    YetersizSatir,
    /// Bir satırda ya borç ya alacak > 0; negatif olamaz (V3).
    GecersizTutar,
    /// Hesap mükellefte yok / pasif / yaprak değil (V4).
    GecersizHesap(HesapId),
    /// Dönem açık değil (V5).
    DonemKapali,
    /// Fiş tarihi dönem aralığı dışında (V6).
    TarihDonemDisi,
    /// Σborç ≠ Σalacak (V1).
    DengeBozuk { borc: i64, alacak: i64 },
    /// Tip dayanak zorunlu kılıyor ama dayanak yok (V7).
    DayanakEksik,
}

impl std::fmt::Display for KayitHatasi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KayitHatasi::ZatenKesin => write!(f, "fiş zaten kesin — değiştirilemez"),
            KayitHatasi::YetersizSatir => write!(f, "fiş en az 2 satır içermeli"),
            KayitHatasi::GecersizTutar => write!(f, "satırda ya borç ya alacak > 0 olmalı, negatif olamaz"),
            KayitHatasi::GecersizHesap(id) => write!(f, "geçersiz hesap: {} (yok/pasif/yaprak değil)", id),
            KayitHatasi::DonemKapali => write!(f, "dönem açık değil — kayıt girilemez"),
            KayitHatasi::TarihDonemDisi => write!(f, "fiş tarihi dönem aralığı dışında"),
            KayitHatasi::DengeBozuk { borc, alacak } => {
                write!(f, "denge bozuk: borç {} ≠ alacak {}", borc, alacak)
            }
            KayitHatasi::DayanakEksik => write!(f, "bu fiş tipi için dayanak zorunlu"),
        }
    }
}

impl std::error::Error for KayitHatasi {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DonemDurum {
    Acik,
    Kilitli,
    Kapanmis,
}

#[derive(Debug, Clone)]
pub struct Donem {
    pub durum: DonemDurum,
    pub baslangic: Tarih,
    pub bitis: Tarih,
}

/// Seri başına **boşluksuz** sıra + TÜM fişlerde müteselsil **yevmiye madde no** sayacı.
/// İkisi de yalnızca kesinleştirmede tüketilir.
#[derive(Default)]
pub struct SeriSayac {
    sayaclar: HashMap<String, u64>,
    yevmiye: u64,
}

impl SeriSayac {
    pub fn yeni() -> Self {
        Self::default()
    }

    fn sonraki(&mut self, seri: &str) -> u64 {
        let e = self.sayaclar.entry(seri.to_string()).or_insert(0);
        *e += 1;
        *e
    }

    fn sonraki_yevmiye(&mut self) -> u64 {
        self.yevmiye += 1;
        self.yevmiye
    }
}

/// Doğrulama bağlamı (hesap planı + dönem) — saf, salt-okunur.
pub struct Baglam<'a> {
    pub hesaplar: &'a HashMap<HesapId, Hesap>,
    pub donem: &'a Donem,
}

/// V1–V7 doğrulama. Kesinleştirme ön koşulu; yan etkisiz.
pub fn dogrula(fis: &Fis, ctx: &Baglam) -> Result<(), KayitHatasi> {
    if fis.durum == FisDurum::Kesin {
        return Err(KayitHatasi::ZatenKesin);
    }

    // V2 — en az 2 satır
    if fis.satirlar.len() < 2 {
        return Err(KayitHatasi::YetersizSatir);
    }

    // V3 + V4 — her satır geçerli tutar + geçerli yaprak hesap
    for s in &fis.satirlar {
        let borc_var = s.borc.pozitif();
        let alacak_var = s.alacak.pozitif();
        // tam olarak biri pozitif olmalı (ikisi de ya da hiçbiri olmaz)
        if borc_var == alacak_var {
            return Err(KayitHatasi::GecersizTutar);
        }
        if s.borc.negatif() || s.alacak.negatif() {
            return Err(KayitHatasi::GecersizTutar);
        }
        match ctx.hesaplar.get(&s.hesap_id) {
            Some(h) if h.aktif && h.yaprak => {}
            _ => return Err(KayitHatasi::GecersizHesap(s.hesap_id.clone())),
        }
    }

    // V5 — dönem açık
    if ctx.donem.durum != DonemDurum::Acik {
        return Err(KayitHatasi::DonemKapali);
    }

    // V6 — tarih dönem aralığında
    if fis.tarih < ctx.donem.baslangic || fis.tarih > ctx.donem.bitis {
        return Err(KayitHatasi::TarihDonemDisi);
    }

    // V1 — denge
    let b = fis.toplam_borc();
    let a = fis.toplam_alacak();
    if b != a {
        return Err(KayitHatasi::DengeBozuk {
            borc: b.deger(),
            alacak: a.deger(),
        });
    }

    // V7 — dayanak (tipe göre)
    if fis.tip.dayanak_zorunlu() && fis.dayanak.is_none() {
        return Err(KayitHatasi::DayanakEksik);
    }

    Ok(())
}

/// Kesinleştir: doğrula → dayanaksız işaretle → seri/sıra ata → durumu KESİN yap.
pub fn kesinlestir(
    fis: &mut Fis,
    ctx: &Baglam,
    sayac: &mut SeriSayac,
) -> Result<(), KayitHatasi> {
    dogrula(fis, ctx)?;

    // Zorunlu olmayan tipte dayanak yoksa bilgi amaçlı işaretle (denetim raporlar).
    if fis.dayanak.is_none() && !fis.tip.sistemsel() {
        fis.dayanaksiz = true;
    }

    let seri = fis.tip.seri_kodu().to_string();
    let sira = sayac.sonraki(&seri);
    fis.seri = Some(seri);
    fis.sira = Some(sira);
    fis.yevmiye_no = Some(sayac.sonraki_yevmiye());
    fis.durum = FisDurum::Kesin;
    Ok(())
}

/// Fiş numarası gösterimi, ör. "TAH-1". Kesinleşmemişse None.
pub fn fis_no(fis: &Fis) -> Option<String> {
    match (&fis.seri, fis.sira) {
        (Some(s), Some(n)) => Some(format!("{}-{}", s, n)),
        _ => None,
    }
}

/// İptal fişi (düzeltme kaydı): KESİN fişin ters kaydı — VUK md.217.
/// Kurallar (mevzuat):
/// - Tarih = hatanın FARK EDİLDİĞİ tarih (`iptal_tarihi`); kaynak tarihe geri dönülmez.
/// - Açıklama standardı: kaynak fiş no + VUK 217 referansı + gerekçe.
/// - Dayanak = kaynak fişin kendisi (dayanaksız İŞARETLENMEZ).
/// - Kayıt silinmez; ters kayıtla iptal edilir, iz korunur.
pub fn iptal_fisi(
    kaynak: &Fis,
    kaynak_id: FisId,
    kaynak_no: &str,
    iptal_tarihi: Tarih,
    gerekce: &str,
) -> Fis {
    let satirlar = kaynak
        .satirlar
        .iter()
        .map(|s| YevmiyeSatiri {
            hesap_id: s.hesap_id.clone(),
            borc: s.alacak, // borç ↔ alacak ters çevrilir
            alacak: s.borc,
            masraf_yeri_id: s.masraf_yeri_id,
            aciklama: format!("İptal: {}", s.aciklama),
        })
        .collect();

    let mut f = Fis::taslak(FisTipi::Mahsup, iptal_tarihi, satirlar);
    f.iptal_kaynak_id = Some(kaynak_id);
    f.aciklama = format!(
        "{} nolu fişin VUK md.217 gereğince ters kayıtla düzeltme (iptal) kaydıdır — {}",
        kaynak_no, gerekce
    );
    f.dayanak = Some(crate::fis::Dayanak {
        belge_tipi: crate::fis::BelgeTipi::Diger,
        belge_no: format!("Kaynak fiş: {}", kaynak_no),
        belge_tarihi: kaynak.tarih,
    });
    f
}

/// Açılış fişi: önceki dönem kapanış bakiyelerinden (bilanço hesapları) otomatik devir.
/// Girdi: (hesap_id, borç_bakiye, alacak_bakiye).
pub fn acilis_fisi(bakiyeler: &[(HesapId, Kurus, Kurus)], tarih: Tarih) -> Fis {
    let satirlar = bakiyeler
        .iter()
        .map(|(id, b, a)| YevmiyeSatiri {
            hesap_id: id.clone(),
            borc: *b,
            alacak: *a,
            masraf_yeri_id: None,
            aciklama: "Devir".to_string(),
        })
        .collect();

    Fis::taslak(FisTipi::Acilis, tarih, satirlar)
}
