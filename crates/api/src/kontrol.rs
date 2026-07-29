//! GÜVENLİK KATMANI — kurallı ön kontrol + sonrası değişmez denetimi.
//!
//! Neden var: hataları tek tek avlıyorduk ve her seferinde biri kaçıyordu (dolu faturaya
//! sessiz eşleştirme, zincirleme kayma, ters bakiye, negatif kasa). Bu modül kuralları
//! **numaralandırıp tek yerde** toplar; her mutasyon önce ÖN KONTROL'den geçer, sonra
//! DEĞİŞMEZ denetimiyle doğrulanır.
//!
//! İki katman, iki farklı iş:
//!   · ÖN KONTROL (K-kodları)  → kullanıcı hatasını ENGELLER, ne yapacağını söyler.
//!   · DEĞİŞMEZ  (D-kodları)   → sistem hatasını YAKALAR. Biri bozulursa bu bir BUG'dır.

use crate::simulasyon::Dataset;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize, Clone, Debug)]
pub struct Bulgu {
    pub kod: String,
    pub onem: String,       // ENGEL (işlem yapılamaz) · UYARI (yapılır ama dikkat)
    pub mesaj: String,
    /// Kullanıcı ne yapmalı — "hata var" demek yetmez, çözümü söylenmeli.
    pub cozum: String,
}

impl Bulgu {
    fn engel(kod: &str, mesaj: String, cozum: String) -> Self {
        Bulgu { kod: kod.into(), onem: "ENGEL".into(), mesaj, cozum }
    }
    fn uyari(kod: &str, mesaj: String, cozum: String) -> Self {
        Bulgu { kod: kod.into(), onem: "UYARI".into(), mesaj, cozum }
    }
}

fn tl(k: i64) -> String { format!("{}.{:02}", k / 100, (k % 100).abs()) }

/// gg.aa.yyyy → yyyyaagg
fn tk(t: &str) -> String {
    let p: Vec<&str> = t.split('.').collect();
    if p.len() == 3 { format!("{}{}{}", p[2], p[1], p[0]) } else { String::new() }
}

// ─────────────────────────────────────────────────────────────────────────────
// ÖN KONTROL — manuel tahsis isteği
// ─────────────────────────────────────────────────────────────────────────────

/// Manuel eşleştirme isteğini kurallardan geçirir. ENGEL varsa işlem YAPILMAMALIDIR.
/// `manuel`: mevcut manuel kararlar (tahsilat_id → fatura_no)
pub fn tahsis_on_kontrol(
    d: &Dataset,
    manuel: &HashMap<String, String>,
    tahsilat_id: &str,
    fatura_no: &str,
) -> Vec<Bulgu> {
    let mut b = Vec::new();

    let Some(t) = d.tahsilatlar.iter().find(|x| x.id == tahsilat_id) else {
        b.push(Bulgu::engel("K01", format!("Tahsilat bulunamadı: {tahsilat_id}"),
            "Referans numarasını kontrol et; banka hareketi listesinden seç.".into()));
        return b;
    };
    let Some(f) = d.faturalar.iter().find(|x| x.no == fatura_no) else {
        b.push(Bulgu::engel("K02", format!("Fatura bulunamadı: {fatura_no}"),
            "Fatura numarasını kontrol et; cari seçip açık fatura listesinden seç.".into()));
        return b;
    };

    // K03 — Hükümsüz belgeye tahsis edilemez: ortada borç yoktur.
    if !f.gecerli {
        b.push(Bulgu::engel("K03",
            format!("{fatura_no} hüküm doğurmayan bir belge ({}) — cari borç yaratmaz.", f.edurum),
            "Taslak faturayı önce gönder; iptal/reddedilmiş faturaya tahsilat bağlanamaz.".into()));
    }

    // K04 — Cari uyumu. Ünvansız girişte cariyi maker BEYAN eder; beyanı kontrol edecek
    // bir gerçek yok, o yüzden bu kural yalnız ünvan varken işler.
    if !t.firma.is_empty() && f.karsi != t.firma {
        b.push(Bulgu::engel("K04",
            format!("Cari uyuşmuyor — tahsilat: {} · fatura: {}", t.firma, f.karsi),
            "Aynı cariye ait bir fatura seç. Yanlış cariye tahsis, iki carinin de bakiyesini bozar.".into()));
    }

    // K05 — Yön uyumu. CARİDEN FARKLI OLARAK yön maker beyanı değil, EKSTRENİN OLGUSUDUR:
    // para ya girmiştir ya çıkmıştır. Bu yüzden ünvan olsun olmasın HER ZAMAN kontrol edilir.
    //
    // Eskiden K05 de K04 ile birlikte `if !t.firma.is_empty()` bloğunun içindeydi. Havadaki
    // kayıtların TAMAMINDA ünvan boş olduğu için, kural tam da manuel eşleştirmenin kullanıldığı
    // her durumda kapalıydı: banka GİRİŞİ bir alış faturasına bağlanabiliyor ve fiş
    // `320 borç / 102 ALACAK` yazılıyordu — giren para çıkmış gibi kaydediliyordu.
    if f.yon != t.yon {
        b.push(Bulgu::engel("K05",
            format!("Yön uyuşmuyor — banka hareketi: {} · fatura: {}", t.yon, f.yon),
            "Gelen para satış faturasına, giden para alış faturasına bağlanır. \
             Yön ekstreden gelir; değiştirilemez.".into()));
    }

    // K06 — Net borcu sıfır olan fatura (tamamı iade/iskonto edilmiş).
    if f.gecerli && f.net_borc == 0 {
        b.push(Bulgu::engel("K06",
            format!("{fatura_no} net borcu sıfır — tamamı iade veya iskonto edilmiş."),
            "Bu faturada kapatılacak borç yok. Başka bir açık fatura seç.".into()));
    }

    // K07 — Hedef başka bir MANUEL kararla dolu. Manuel, manueli DÜŞÜRMEZ (maker kendi
    // kararını sessizce ezmesin). Hangi eşleştirmeyi kaldırması gerektiğini SÖYLE.
    let doldurdugu: Vec<&String> = manuel.iter()
        .filter(|(k, v)| v.as_str() == fatura_no && k.as_str() != tahsilat_id)
        .map(|(k, _)| k).collect();
    if !doldurdugu.is_empty() {
        let manuel_tutar: i64 = f.tahsisler.iter()
            .filter(|x| x.kaynak == "MANUEL" && doldurdugu.iter().any(|d| **d == x.tahsilat_id))
            .map(|x| x.tutar).sum();
        if manuel_tutar >= f.net_borc {
            b.push(Bulgu::engel("K07",
                format!("{fatura_no} tamamen MANUEL kararlarla dolu ({} TL) — manuel karar manuel kararı düşüremez.",
                    tl(manuel_tutar)),
                format!("Önce şu manuel eşleştirmeyi kaldır: {}. (DELETE /api/tahsis/manuel/{})",
                    doldurdugu.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", "),
                    doldurdugu[0])));
        }
    }

    // K08 — Zaten bu faturaya bağlıysa işlem gereksiz.
    if manuel.get(tahsilat_id).map(|x| x == fatura_no).unwrap_or(false) {
        b.push(Bulgu::uyari("K08",
            format!("{tahsilat_id} zaten {fatura_no} faturasına manuel bağlı."),
            "Değişiklik gerekmiyor. Kaldırmak için DELETE kullan.".into()));
    }

    // K09 — Fatura tahsilattan SONRAKİ tarihliyse: doğmamış borç ödenmiş olur.
    if !tk(&f.tarih).is_empty() && !tk(&t.tarih).is_empty() && tk(&f.tarih) > tk(&t.tarih) {
        b.push(Bulgu::uyari("K09",
            format!("Fatura tarihi ({}) tahsilattan ({}) SONRA — henüz doğmamış borç kapatılıyor.", f.tarih, t.tarih),
            "Avans tahsilatı ise doğru olabilir; değilse tarihleri kontrol et.".into()));
    }

    // K10 — Tahsilat faturanın borcundan büyük: aşan kısım başka yere gidecek.
    if f.gecerli && f.net_borc > 0 && t.tutar > f.kalan && f.kalan > 0 {
        b.push(Bulgu::uyari("K10",
            format!("Tahsilat ({} TL) faturanın kalanından ({} TL) büyük.", tl(t.tutar), tl(f.kalan)),
            "Aşan tutar aynı carinin diğer açık faturalarına FIFO ile dağıtılacak; kalanı avans olur.".into()));
    }

    b
}

// ─────────────────────────────────────────────────────────────────────────────
// DEĞİŞMEZ DENETİMİ — mutasyon sonrası. Biri bozulursa BUG'dır, sessiz geçilmez.
// ─────────────────────────────────────────────────────────────────────────────

/// Sistemin her zaman doğru olması gereken kuralları. Boş dönmeli.
pub fn degismezler(d: &Dataset, defter_borc: i64, defter_alacak: i64) -> Vec<Bulgu> {
    let mut b = Vec::new();

    // D01 — PARA KORUNUMU: hiçbir tahsilat tutarından fazla dağıtılamaz.
    let mut dagitim: HashMap<&str, i64> = HashMap::new();
    for f in &d.faturalar {
        for t in &f.tahsisler { *dagitim.entry(t.tahsilat_id.as_str()).or_insert(0) += t.tutar; }
    }
    for t in &d.tahsilatlar {
        let dg = dagitim.get(t.id.as_str()).copied().unwrap_or(0);
        if dg > t.tutar {
            b.push(Bulgu::engel("D01",
                format!("{} tutarından fazla dağıtıldı: {} > {}", t.id, tl(dg), tl(t.tutar)),
                "Tahsis motorunda para yaratılıyor — dağıtım sınırı (kalan_fatura) kontrol edilmeli.".into()));
        }
        if dg != t.dagitilan {
            b.push(Bulgu::engel("D02",
                format!("{} dagitilan alanı gerçek dağıtımla uyuşmuyor: {} ≠ {}", t.id, tl(t.dagitilan), tl(dg)),
                "Motor `dagitilan` alanını yazarken tahsisleri saymıyor.".into()));
        }
    }

    // D03 — AŞIRI TAHSİLAT: fatura kalanı negatif olamaz.
    for f in &d.faturalar {
        let toplam: i64 = f.tahsisler.iter().map(|x| x.tutar).sum();
        if toplam > f.net_borc {
            b.push(Bulgu::engel("D03",
                format!("{} net borcundan fazla tahsis aldı: {} > {}", f.no, tl(toplam), tl(f.net_borc)),
                "Faturaya borcundan fazla para yazılıyor.".into()));
        }
        if f.kalan < 0 {
            b.push(Bulgu::engel("D04", format!("{} kalanı negatif: {}", f.no, tl(f.kalan)),
                "kalan = net_borc − tahsil hesabı bozuk.".into()));
        }
        // D05 — Hükümsüz belge tahsis alamaz.
        if !f.gecerli && !f.tahsisler.is_empty() {
            b.push(Bulgu::engel("D05",
                format!("{} hükümsüz ({}) olduğu halde {} tahsis almış", f.no, f.edurum, f.tahsisler.len()),
                "Geçersiz belge FIFO kuyruğuna girmemeli ve manuel tahsis kabul etmemeli.".into()));
        }
    }

    // D07 — YÖN BÜTÜNLÜĞÜ: gelen para satış, giden para alış faturasını kapatır.
    // Bozulursa fiş yanlış hesaba yazılır (banka girişi 102 ALACAK'a) — mizan yine denk
    // görünür, ama hangi varlığın arttığı yanlış olur ve banka mutabakatı tutmaz.
    let yonler: HashMap<&str, &str> = d.tahsilatlar.iter()
        .map(|t| (t.id.as_str(), t.yon.as_str())).collect();
    for f in &d.faturalar {
        for ts in &f.tahsisler {
            if let Some(&ty) = yonler.get(ts.tahsilat_id.as_str()) {
                if ty != f.yon {
                    b.push(Bulgu::engel("D07",
                        format!("{} ({}) yönü {} olan {} hareketine bağlanmış", f.no, f.yon, ty, ts.tahsilat_id),
                        "Yön denetimi (K05) atlanmış. Gelen para satış, giden para alış faturasına bağlanır.".into()));
                }
            }
        }
    }

    // D06 — ÇİFT TARAFLI KAYIT: defter dengesi.
    if defter_borc != defter_alacak {
        b.push(Bulgu::engel("D06",
            format!("Defter dengesiz: borç {} ≠ alacak {}", tl(defter_borc), tl(defter_alacak)),
            "Bir fiş tek bacaklı yazılmış; Σborç = Σalacak bozulmuş (VUK 219).".into()));
    }

    b
}

#[cfg(test)]
mod testler {
    use super::*;
    use crate::simulasyon::dataset;

    fn veri() -> Dataset { dataset(&HashMap::new()) }

    /// Temiz sistemde HİÇBİR değişmez bozulmamalı.
    #[test]
    fn degismezler_temiz_sistemde_bos() {
        let d = veri();
        let (mut b, mut a) = (0i64, 0i64);
        for f in &d.faturalar {
            for fis in [&f.tahakkuk, &f.iptal_fisi, &f.iade_fisi, &f.iskonto_fisi, &f.smm_fisi, &f.iade_stok_fisi, &f.odeme].into_iter().flatten() {
                b += fis.satirlar.iter().map(|s| s.borc).sum::<i64>();
                a += fis.satirlar.iter().map(|s| s.alacak).sum::<i64>();
            }
        }
        let bulgular = degismezler(&d, b, a);
        assert!(bulgular.is_empty(), "değişmez ihlali: {:?}",
            bulgular.iter().map(|x| format!("{} {}", x.kod, x.mesaj)).collect::<Vec<_>>());
    }

    /// Dengesiz defter D06 ile yakalanmalı — sessiz geçilmemeli.
    #[test]
    fn dengesiz_defter_yakalanir() {
        let d = veri();
        let bulgular = degismezler(&d, 100, 90);
        assert!(bulgular.iter().any(|x| x.kod == "D06"), "denge ihlali yakalanmadı");
    }

    /// Yanlış cariye eşleştirme K04 ile ENGELLENMELİ.
    #[test]
    fn yanlis_cari_engellenir() {
        let d = veri();
        let t = d.tahsilatlar.iter().find(|x| !x.firma.is_empty()).unwrap();
        let f = d.faturalar.iter().find(|x| x.karsi != t.firma && x.gecerli).unwrap();
        let b = tahsis_on_kontrol(&d, &HashMap::new(), &t.id, &f.no);
        assert!(b.iter().any(|x| x.kod == "K04" && x.onem == "ENGEL"), "yanlış cari engellenmedi: {b:?}");
    }

    /// Hükümsüz belgeye tahsis K03 ile ENGELLENMELİ.
    #[test]
    fn hukumsuz_belgeye_tahsis_engellenir() {
        let d = veri();
        let f = d.faturalar.iter().find(|x| !x.gecerli && x.yon == "SATIS").unwrap();
        let t = d.tahsilatlar.iter().find(|x| x.firma == f.karsi && x.yon == f.yon).unwrap();
        let b = tahsis_on_kontrol(&d, &HashMap::new(), &t.id, &f.no);
        assert!(b.iter().any(|x| x.kod == "K03" && x.onem == "ENGEL"), "hükümsüz belge engellenmedi: {b:?}");
    }

    /// Manuel dolu faturada K07 ENGEL vermeli VE hangi eşleştirmenin kaldırılacağını söylemeli.
    #[test]
    fn manuel_dolu_fatura_cozum_onerir() {
        let mut manuel = HashMap::new();
        // Önce bir manuel karar kur ve o dağıtımla veriyi üret.
        let d0 = veri();
        let t1 = d0.tahsilatlar.iter()
            .find(|x| !x.firma.is_empty() && x.yon == "SATIS").unwrap().clone();
        let hedef = d0.faturalar.iter()
            .find(|f| f.karsi == t1.firma && f.yon == t1.yon && f.gecerli && f.net_borc > 0 && f.net_borc <= t1.tutar)
            .map(|f| f.no.clone());
        let Some(hedef) = hedef else { return }; // uygun senaryo yoksa atla
        manuel.insert(t1.id.clone(), hedef.clone());
        let d = dataset(&manuel);

        // Şimdi BAŞKA bir tahsilat aynı faturaya bağlanmak istesin.
        let t2 = d.tahsilatlar.iter()
            .find(|x| x.firma == t1.firma && x.yon == t1.yon && x.id != t1.id).unwrap();
        let b = tahsis_on_kontrol(&d, &manuel, &t2.id, &hedef);
        let k07 = b.iter().find(|x| x.kod == "K07");
        assert!(k07.is_some(), "manuel dolu fatura K07 vermedi: {b:?}");
        let k07 = k07.unwrap();
        assert_eq!(k07.onem, "ENGEL");
        assert!(k07.cozum.contains(&t1.id), "çözüm hangi eşleştirmenin kaldırılacağını söylemeli: {}", k07.cozum);
    }
}
