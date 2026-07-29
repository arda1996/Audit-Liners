//! Kapsamlı SİMÜLASYON üreteci — bütünleşik, DENGELİ (Σborç=Σalacak), ~200M TL hacim, 5000+ kayıt.
//! e-Fatura (satış/alış) → tahakkuk fişi → ödeme enstrümanı (BANKA / NAKİT / ÇEK / SENET / AÇIK) →
//! ödeme fişi + ilgili hareket. Nakit ve çek/senet, ticaretin banka-dışı gerçeğini temsil eder.
//! İzole modül; AppState'e bağımsız; deterministik (sabit seed, random/Date::now YOK).

use axum::{extract::Query, Json};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Cari kartı: ünvan + VKN. **Ba/Bs formları VKN olmadan düzenlenemez** — bildirim
/// karşı tarafın vergi kimliğiyle eşleşir. (VKN'ler kurgusaldır, gerçek mükellefe ait değildir.)
const CARILER: &[(&str, &str)] = &[
    ("BEYAZ MAĞAZACILIK A.Ş.", "1120340560"), ("GÜNEŞ İPLİK SAN. LTD.", "2230451670"),
    ("MERT PERAKENDE A.Ş.", "3340562780"),    ("DEMİR İNŞAAT A.Ş.", "4450673890"),
    ("EGE GIDA PAZARLAMA LTD.", "5560784900"),("FİLİZ AMBALAJ SAN.", "6670895010"),
    ("ACME TEKSTİL A.Ş.", "7780906120"),      ("YILDIZ NAKLİYAT LTD.", "8891017230"),
    ("KARDELEN KOZMETİK A.Ş.", "9901128340"), ("ANADOLU DEMİR ÇELİK", "1012239450"),
    ("MAVİ DENİZ TURİZM", "2123340560"),      ("ALTIN TARIM ÜRÜNLERİ", "3234451670"),
];

/// Ünvandan VKN çözer (Ba/Bs ve e-belge için). Bulunamazsa boş döner.
pub fn vkn(unvan: &str) -> String {
    CARILER.iter().find(|(a, _)| *a == unvan).map(|(_, v)| v.to_string()).unwrap_or_default()
}

pub const BANKA_AD: &str = "Türkiye İş Bankası";
pub const BANKA_HESAP: &str = "SIM-0064-01";
pub const BANKA_IBAN: &str = "TR30 0064 0000 0000 1000";

/// Ödeme enstrümanı — banka-dışı (nakit/çek/senet) ticaretin gerçeği.
#[derive(Serialize, Clone, Copy, PartialEq)]
enum Enstruman { Banka, Nakit, Cek, Senet, Acik }

impl Enstruman {
    fn ad(&self) -> &'static str {
        match self { Enstruman::Banka => "BANKA", Enstruman::Nakit => "NAKIT", Enstruman::Cek => "CEK", Enstruman::Senet => "SENET", Enstruman::Acik => "ACIK" }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FisSat { pub hesap: String, pub borc: i64, pub alacak: i64 }
#[derive(Serialize, Clone)]
pub struct Fis { pub no: String, pub tip: String, pub tarih: String, pub aciklama: String, pub satirlar: Vec<FisSat> }
#[derive(Serialize, Clone)]
pub struct FaturaSim {
    pub no: String, pub tarih: String, pub yon: String, pub karsi: String,
    pub matrah: i64, pub kdv: i64, pub toplam: i64,
    pub enstruman: String, pub odendi: bool,
    pub tahsil: i64, pub kalan: i64, // kalan = net_borc − tahsil (cari açık bakiye)

    // ── KDV ve istisna ────────────────────────────────────────────────────
    pub kdv_oran: i64,         // 20 genel · 10 indirimli · 1 temel gıda · 0 istisna
    pub istisna_kod: String,   // KDV=0 ise ZORUNLU (UBL-TR TaxExemptionReason)
    pub tevkifat: i64,         // KDVK 9: alıcının sorumlu sıfatıyla beyan ettiği KDV payı

    // ── Geçerlilik ve indirimler ──────────────────────────────────────────
    /// TASLAK/IPTAL/RED ise false: belge hüküm doğurmaz, cari bakiye yaratmaz.
    pub gecerli: bool,
    /// Satıştan iade (610) — mal teslim edilmiş, sonra geri gelmiş.
    pub iade_matrah: i64, pub iade_kdv: i64,
    /// Fatura sonrası iskonto (611) — ciro primi / erken ödeme.
    pub iskonto_matrah: i64, pub iskonto_kdv: i64,
    /// Tahsis motorunun kapatacağı gerçek borç: toplam − iade − iskonto (geçersizse 0).
    pub net_borc: i64,

    /// Bu faturayı kapatan tahsis(ler) — hangi tahsilat, ne kadar, FIFO mu manuel mi.
    pub tahsisler: Vec<TahsisSim>,
    pub ebelge_tip: String, // E_FATURA | E_ARSIV (karşı taraf e-Fatura mükellefi mi → otomatik yönlendirme)
    pub edurum: String,     // e-belge durumu (ödeme durumundan AYRI): TASLAK/GONDERILDI/KABUL/RED/IPTAL/RAPORLANDI/ALINDI/ONAY_BEKLIYOR
    /// TASLAK faturada YOK — henüz belge değil, kayıt üretmez.
    pub tahakkuk: Option<Fis>,
    /// IPTAL/RED ise tahakkuku kapatan ters kayıt (VUK 219: silme yok, storno var).
    pub iptal_fisi: Option<Fis>,
    /// İade varsa 610 kaydı (satış) / 320-153-191 (alış). 600 TERS ÇEVRİLMEZ.
    pub iade_fisi: Option<Fis>,

    // ── Satılan malın maliyeti (aralıksız envanter) ───────────────────────
    /// Bu satışla stoktan çıkan malın MALİYETİ. Hasılat (600) ile maliyet (621) aynı
    /// dönemde eşleşmezse gelir tablosu brüt kârı anlamsızdır (dönemsellik · eşleştirme ilkesi).
    pub maliyet: i64,
    /// İade edilen malın maliyeti — stoka geri döner (153 borç / 621 alacak).
    pub iade_maliyet: i64,
    /// Satışta stok çıkışı: 621 borç / 153 alacak. Alışta yoktur (alış stoğu ARTIRIR).
    pub smm_fisi: Option<Fis>,
    /// Satıştan iadede malın stoka dönüşü. Yalnız mal iadesinde; hizmet iadesinde stok yoktur.
    pub iade_stok_fisi: Option<Fis>,
    /// İskonto varsa 611 kaydı.
    pub iskonto_fisi: Option<Fis>,
    pub odeme: Option<Fis>,
}

fn lcg(s: &mut u64) -> u64 {
    *s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    // YÜKSEK BİTLER — 2^64 modüllü LCG'de DÜŞÜK bitler zayıftır: bit 0'ın periyodu 2, bit 1'in 4.
    // `lcg() % 12` düşük bitleri kullandığı için ay dağılımı tek/çift aylara çöküyordu
    // (defterde tek aylar ~1750, çift aylar ~100 kayıt). Aynı zafiyet KDV oranı, enstrüman ve
    // e-durum seçimlerini de çarpıtıyordu. 31 bitlik üst pencere tüm kullanımlara yeter.
    *s >> 33
}
fn borc(h: &str, t: i64) -> FisSat { FisSat { hesap: h.to_string(), borc: t, alacak: 0 } }
fn alacak(h: &str, t: i64) -> FisSat { FisSat { hesap: h.to_string(), borc: 0, alacak: t } }

/// Bir tahsilat/ödeme olayı — faturaya DEĞİL cariye aittir. Hangi faturayı kapattığı
/// dağıtım (tahsis) aşamasında belirlenir: FIFO ya da muhasebecinin manuel eşleştirmesi.
#[derive(Serialize, Clone)]
pub struct TahsilatSim {
    pub id: String,
    /// Karşı taraf ünvanı. BOŞ ise ekstrede ünvan yok → otomatik eşleşme YAPILAMAZ, maker'a düşer.
    pub firma: String,
    pub yon: String, pub tarih: String,
    pub tutar: i64, pub enstruman: String,
    /// Faturalara dağıtılan kısım; tutar − dagitilan = avans (karşılığı fatura yok).
    pub dagitilan: i64,
    /// OTOMATIK (motor eşledi) · MANUEL (maker kararı) · BEKLIYOR (maker müdahalesi şart)
    pub durum: String,
    /// BEKLIYOR ise sebebi — maker ne yapacağını bilsin.
    pub sebep: String,
    /// Paranın hangi bankada/hesapta olduğu — havada bakiye uyarısı bankayı göstermeli.
    pub banka_ad: String,
    pub hesap_id: String,
    pub iban: String,
    /// Maker "fatura henüz kesilmedi" dediyse: beklenen fatura firması + hatırlatma.
    pub fatura_bekleniyor: bool,
    pub bekleyen_firma: String,
}

/// Bir tahsilatın bir faturaya dağıtılan parçası. `kaynak`: FIFO (otomatik) | MANUEL (muhasebeci).
#[derive(Serialize, Clone)]
pub struct TahsisSim {
    pub tahsilat_id: String, pub tarih: String, pub tutar: i64,
    pub kaynak: String, pub enstruman: String,
}

pub struct Dataset { pub faturalar: Vec<FaturaSim>, pub tahsilatlar: Vec<TahsilatSim> }

/// Geriye dönük uyum — manuel eşleştirme olmadan saf FIFO sonucu.
pub fn uret() -> Vec<FaturaSim> { dataset(&HashMap::new()).faturalar }

/// Defterde AKTİF fişi olan tahsisler. Bunlar DONMUŞTUR: yeniden hesaplanmaz.
/// Muhasebe kaydı bir kez atıldıktan sonra geçmiş dağıtım kendiliğinden değişemez —
/// yoksa tek bir düzeltme, sonraki tüm tahsilatları zincirleme kaydırıp yüzlerce
/// gereksiz storno üretirdi. Yalnız serbest bırakılan tahsilat yeniden dağıtılır.
pub type Donmus = HashMap<String, Vec<(String, i64)>>; // tahsilat_id → [(fatura_no, tutar)]

pub fn dataset(manuel: &HashMap<String, String>) -> Dataset {
    dataset_ex(manuel, &Donmus::new())
}

/// FIFO TAHSİS MOTORU — muhasebede cari hesap kapatma sırası "ilk giren ilk çıkar":
/// tahsilat, o carinin EN ESKİ açık faturasından başlayarak kapatır (TBK 100: borçlu hangi
/// borcu ödediğini belirtmemişse muaccel/en eski borç önce kapanır). Muhasebecinin MANUEL
/// eşleştirmesi bu karineyi ezer: manuel tahsisler önce sabitlenir, kalan para FIFO akar —
/// yani manuel eşleştirilen tahsilatın ÖNCEKİ otomatik eşleşmesi kendiliğinden kalkar.
/// `manuel`: tahsilat_id → fatura_no
pub fn dataset_ex(manuel: &HashMap<String, String>, donmus: &Donmus) -> Dataset {
    let (faturalar, tahsilatlar) = ham();
    tahsis_uygula(faturalar, tahsilatlar, manuel, donmus)
}

/// Tahsis motorunun SAF çekirdeği — üreteçten bağımsız. `dataset_ex` sabit simülasyon
/// verisiyle çağırır; testler kendi kurdukları veri setleriyle çağırır.
///
/// Motoru üreteçten ayırmasaydık her senaryoyu ancak 2228 kayıtlık tek bir veri kümesinin
/// içinde şans eseri bulabildiğimiz kadar sınayabilirdik — aykırı durumlar (ünvansız hareket,
/// tamamı iade edilmiş fatura, aşan tahsilat) orada ya hiç yok ya da tek örnekle temsil ediliyor.
pub fn tahsis_uygula(
    mut faturalar: Vec<FaturaSim>,
    mut tahsilatlar: Vec<TahsilatSim>,
    manuel: &HashMap<String, String>,
    donmus: &Donmus,
) -> Dataset {
    // Fatura indeksleri: (firma, yon) → en eskiden yeniye sıralı fatura indeksleri
    let mut kova: HashMap<(String, String), Vec<usize>> = HashMap::new();
    for (i, f) in faturalar.iter().enumerate() {
        if !f.gecerli { continue; } // hükümsüz belge FIFO kuyruğuna girmez
        kova.entry((f.karsi.clone(), f.yon.clone())).or_default().push(i);
    }
    for v in kova.values_mut() {
        v.sort_by(|&a, &b| tarih_key(&faturalar[a].tarih).cmp(&tarih_key(&faturalar[b].tarih))
            .then(faturalar[a].no.cmp(&faturalar[b].no))); // eşit tarihte belge no ile kararlı sıra
    }

    // Kapatılacak tutar NET BORÇ: toplam − iade − iskonto. Geçersiz belgede (TASLAK/IPTAL/RED)
    // sıfırdır — hüküm doğurmayan faturaya tahsilat tahsis edilemez.
    let mut kalan_fatura: Vec<i64> = faturalar.iter().map(|f| f.net_borc).collect();
    let mut tahsisler: Vec<Vec<TahsisSim>> = vec![Vec::new(); faturalar.len()];
    let mut fatura_no_idx: HashMap<String, usize> = HashMap::new();
    for (i, f) in faturalar.iter().enumerate() { fatura_no_idx.insert(f.no.clone(), i); }

    // Tahsilatları tarih sırasına koy (FIFO tahsilat tarafında da kronolojik).
    let mut sira: Vec<usize> = (0..tahsilatlar.len()).collect();
    sira.sort_by(|&a, &b| tarih_key(&tahsilatlar[a].tarih).cmp(&tarih_key(&tahsilatlar[b].tarih))
        .then(tahsilatlar[a].id.cmp(&tahsilatlar[b].id)));

    let mut yaz = |fi: usize, t: &TahsilatSim, tutar: i64, kaynak: &str,
                   kalan_fatura: &mut Vec<i64>, tahsisler: &mut Vec<Vec<TahsisSim>>| {
        kalan_fatura[fi] -= tutar;
        tahsisler[fi].push(TahsisSim {
            tahsilat_id: t.id.clone(), tarih: t.tarih.clone(), tutar,
            kaynak: kaynak.into(), enstruman: t.enstruman.clone(),
        });
    };

    let mut kullanilan: Vec<i64> = vec![0; tahsilatlar.len()];

    // Manuel kararların HEDEF faturaları — yalnız buralarda donmuş tahsis düşürülür.
    // Başka faturalara dokunulmaz; aksi halde tek düzeltme tüm FIFO zincirini kaydırır
    // (bir kez oldu: 50 tahsis yer değiştirdi, yüzlerce gereksiz storno üretiyordu).
    let manuel_hedefler: std::collections::HashSet<&String> = manuel.values().collect();

    // 0) DONMUŞ tahsisler — deftere geçmiş dağıtım korunur. İSTİSNA: manuel kararın hedefi
    //    olan faturada yer AÇILIR (o tahsis düşürülür, parası serbest kalıp yeniden dağıtılır).
    for &ti in &sira {
        let Some(paylar) = donmus.get(&tahsilatlar[ti].id) else { continue };
        let t = tahsilatlar[ti].clone();
        for (fno, tutar) in paylar {
            if manuel_hedefler.contains(fno) { continue; } // maker'a yer aç
            let Some(&fi) = fatura_no_idx.get(fno) else { continue };
            let pay = (*tutar).min(kalan_fatura[fi]).min(t.tutar - kullanilan[ti]);
            if pay > 0 {
                let kaynak = if manuel.get(&t.id).map(|x| x == fno).unwrap_or(false) { "MANUEL" } else { "FIFO" };
                yaz(fi, &t, pay, kaynak, &mut kalan_fatura, &mut tahsisler);
                kullanilan[ti] += pay;
            }
        }
    }

    // 1) MANUEL — maker'ın kararı, hedef faturada açılan yere yazılır. Hedef başka bir
    //    tahsilatla kapanmışsa o tahsis yukarıda düşürülmüştür; burası artık boş bulur.
    for &ti in &sira {
        let Some(hedef_no) = manuel.get(&tahsilatlar[ti].id) else { continue };
        let Some(&fi) = fatura_no_idx.get(hedef_no) else { continue };
        if !faturalar[fi].gecerli { continue; }
        let t = tahsilatlar[ti].clone();
        if !t.firma.is_empty() && faturalar[fi].karsi != t.firma { continue; }
        // Yön HER ZAMAN denetlenir (K05 ile aynı gerekçe): cariyi maker beyan edebilir,
        // ama paranın girip çıktığını ekstre söyler. Ünvansız diye yön serbest bırakılırsa
        // banka girişi alış faturasına bağlanıp 102 ALACAK'a yazılır.
        if faturalar[fi].yon != t.yon { continue; }
        let pay = (t.tutar - kullanilan[ti]).min(kalan_fatura[fi]);
        if pay > 0 {
            yaz(fi, &t, pay, "MANUEL", &mut kalan_fatura, &mut tahsisler);
            kullanilan[ti] += pay;
        }
    }

    // 2) OTOMATİK EŞLEŞME — kalan para FIFO akar, o carinin en eski AÇIK faturasından başlayarak.
    // ÖN KOŞUL: karşı taraf ünvanı bilinmeli. Ünvan yoksa hangi cariye ait olduğu bilinemez →
    // otomatik eşleşme yapılmaz, maker kuyruğuna düşer (yanlış cariye tahsis = yanlış bakiye).
    for &ti in &sira {
        let t = tahsilatlar[ti].clone();
        if t.firma.is_empty() { continue; } // ünvansız → otomatik kapalı
        let mut arta = t.tutar - kullanilan[ti];
        if arta <= 0 { continue; }
        let Some(idxler) = kova.get(&(t.firma.clone(), t.yon.clone())) else { continue };
        for &fi in idxler {
            if arta <= 0 { break; }
            if kalan_fatura[fi] <= 0 { continue; }
            // Fatura henüz doğmamışsa o parayı kapatamaz (tahsilat tarihinden sonraki fatura).
            if tarih_key(&faturalar[fi].tarih) > tarih_key(&t.tarih) { continue; }
            let pay = arta.min(kalan_fatura[fi]);
            yaz(fi, &t, pay, "FIFO", &mut kalan_fatura, &mut tahsisler);
            arta -= pay;
            kullanilan[ti] += pay;
        }
    }
    // Durum ataması — maker kuyruğu buradan doğar.
    for (ti, t) in tahsilatlar.iter_mut().enumerate() {
        t.dagitilan = kullanilan[ti];
        let elde = manuel.contains_key(&t.id);
        (t.durum, t.sebep) = if elde && t.dagitilan > 0 {
            ("MANUEL".into(), String::new())
        } else if t.firma.is_empty() {
            ("BEKLIYOR".into(), "Ekstrede karşı taraf ünvanı yok — otomatik eşleşme yapılamaz; firma seçip fatura eşle".into())
        } else if t.dagitilan == 0 {
            ("BEKLIYOR".into(), format!("{} carisinde eşleşecek açık fatura bulunamadı — avans olabilir", t.firma))
        } else if t.dagitilan < t.tutar {
            ("OTOMATIK".into(), format!("Kısmen dağıtıldı — {} kuruş açıkta (avans)", t.tutar - t.dagitilan))
        } else {
            ("OTOMATIK".into(), String::new())
        };
    }

    // 3) Dağıtım sonucunu faturalara yaz: tahsil/kalan + ödeme fişi tahsislerden türer.
    for (i, f) in faturalar.iter_mut().enumerate() {
        let ts = std::mem::take(&mut tahsisler[i]);
        let tahsil: i64 = ts.iter().map(|x| x.tutar).sum();
        f.tahsil = tahsil;
        f.kalan = f.net_borc - tahsil;
        f.odendi = f.gecerli && f.kalan == 0;
        // Faturayı fiilen hangi enstrüman kapattı — açıksa AÇIK, birden çoksa KARMA.
        // Tek enstrüman varsayımı yanlıştı: bir fatura kısmen çekle, kısmen havaleyle kapanabilir
        // (veride her 3 faturadan 1'i böyle). "İlk tahsisin enstrümanı" alınıp TÜM tutar o hesaba
        // yazıldığında, bankaya giren para çek portföyünde (101) görünüyordu.
        let enstrumanlar: Vec<String> = {
            let mut v: Vec<String> = Vec::new();
            for x in &ts { if !v.contains(&x.enstruman) { v.push(x.enstruman.clone()); } }
            v // ilk görülme sırası — tahsis sırası kronolojik olduğu için kararlı
        };
        f.enstruman = match enstrumanlar.len() {
            0 => "ACIK".into(),
            1 => enstrumanlar[0].clone(),
            _ => "KARMA".into(),
        };
        f.odeme = if tahsil > 0 {
            let satis = f.yon == "SATIS";
            let hesap = |e: &str| match e {
                "NAKIT" => "100",
                "CEK" => if satis { "101" } else { "103" },
                "SENET" => if satis { "121" } else { "321" },
                _ => "102",
            };
            // Her enstrüman KENDİ hesabına, kendi tutarıyla. Karşı bacak (120/320) tektir.
            let mut bacaklar: Vec<(String, i64)> = Vec::new();
            for e in &enstrumanlar {
                let tutar: i64 = ts.iter().filter(|x| &x.enstruman == e).map(|x| x.tutar).sum();
                bacaklar.push((hesap(e).to_string(), tutar));
            }
            let sat = if satis {
                let mut v: Vec<FisSat> = bacaklar.iter().map(|(h, t)| borc(h, *t)).collect();
                v.push(alacak("120", tahsil));
                v
            } else {
                let mut v: Vec<FisSat> = vec![borc("320", tahsil)];
                v.extend(bacaklar.iter().map(|(h, t)| alacak(h, *t)));
                v
            };
            Some(Fis {
                no: format!("ODM-{}", f.no), tip: "Mahsup".into(),
                tarih: ts.last().map(|x| x.tarih.clone()).unwrap_or_else(|| f.tarih.clone()),
                aciklama: format!("{} — {} {} ({} tahsis, {})", f.karsi,
                    if f.kalan > 0 { "kısmi" } else { "tam" },
                    if satis { "tahsilat" } else { "ödeme" }, ts.len(),
                    if ts.iter().any(|x| x.kaynak == "MANUEL") { "manuel eşleştirme" } else { "FIFO" }),
                satirlar: sat,
            })
        } else { None };
        f.tahsisler = ts;
    }

    Dataset { faturalar, tahsilatlar }
}

/// Ham üretimin ÖNBELLEĞİ. `ham()` deterministiktir (sabit tohum, girdisi yok) — sonucu
/// her çağrıda yeniden üretmek gereksizdi ve tüm uçları yavaşlatıyordu (~1,5 sn/çağrı).
/// Bir kez üretilir, sonra klonlanır; klon LCG+format! üretiminden çok daha ucuzdur.
static HAM_ONBELLEK: std::sync::OnceLock<(Vec<FaturaSim>, Vec<TahsilatSim>)> = std::sync::OnceLock::new();

fn ham() -> (Vec<FaturaSim>, Vec<TahsilatSim>) {
    HAM_ONBELLEK.get_or_init(ham_uret).clone()
}

/// Ham üretim — faturalar (henüz tahsis edilmemiş) + tahsilat olayları. Deterministik.
fn ham_uret() -> (Vec<FaturaSim>, Vec<TahsilatSim>) {
    let hedef = 200_000_000_00i64; // 200M TL, kuruş
    let mut s = 0x2026_1453u64;
    let mut hacim = 0i64;
    let mut liste = Vec::new();
    let mut tahsilatlar: Vec<TahsilatSim> = Vec::new();
    let mut i = 0u64;
    while hacim < hedef {
        i += 1;
        let satis = lcg(&mut s) % 2 == 0;
        // TUTAR DAĞILIMI — ticaretin gerçeği: çok sayıda küçük, az sayıda büyük fatura.
        // Hepsi 15k+ olursa her nakit hareket tevsik haddini aşar ve bulgu anlamsızlaşır.
        let matrah = match lcg(&mut s) % 100 {
            0..=41 => 300_00 + (lcg(&mut s) % 5_500_00) as i64,       // 300–5.800 TL (perakende)
            42..=79 => 6_000_00 + (lcg(&mut s) % 44_000_00) as i64,   // 6k–50k (toptan)
            _ => 50_000_00 + (lcg(&mut s) % 200_000_00) as i64,       // 50k–250k (proje/dönemsel)
        };

        // KDV ORANI — tek oran gerçekçi değil. %20 genel · %10 indirimli (gıda/tekstil/ilaç) ·
        // %1 temel gıda · ihracat istisnası (KDVK 11/12, KDV doğmaz, 601'e yazılır).
        let (kdv_oran, ihracat) = if satis {
            match lcg(&mut s) % 100 {
                0..=61 => (20, false), 62..=81 => (10, false), 82..=90 => (1, false), _ => (0, true),
            }
        } else {
            match lcg(&mut s) % 100 { 0..=66 => (20, false), 67..=86 => (10, false), _ => (1, false) }
        };
        let kdv = matrah * kdv_oran / 100;
        let toplam = matrah + kdv;
        // KDV=0 ise istisna sebep kodu ZORUNLU (UBL-TR TaxExemptionReason) — boş bırakılamaz.
        let istisna_kod = if kdv_oran == 0 { "301 — Mal ihracatı (KDVK 11/1-a)".to_string() } else { String::new() };

        // TEVKİFAT (KDVK 9) — belirli hizmetlerde KDV'nin bir kısmını alıcı sorumlu sıfatıyla
        // beyan eder (2 No.lu KDV). Satıcıda 391'e yalnız tevkifat DIŞI pay yazılır.
        let tevkifat = if satis && kdv > 0 && lcg(&mut s) % 100 < 8 { kdv / 2 } else { 0 };

        hacim += toplam;
        let karsi = CARILER[(lcg(&mut s) % CARILER.len() as u64) as usize].0.to_string();
        let ay = 1 + (lcg(&mut s) % 12) as u32;
        let gun = 1 + (lcg(&mut s) % 28) as u32;
        let tarih = format!("{:02}.{:02}.2026", gun, ay);

        // ÖDEME ARACI TUTARLA İLİŞKİLİDİR. Küçük tutarda nakit olağan; haddi aşan tutarda
        // banka/çek/senet zorunlu (VUK mük. 257). Yine de az sayıda işletme haddi aşarak nakit
        // yapar — tevsik ihlali senaryosu gerçekçi oranda (~%6) doğsun diye bilerek üretiliyor.
        let ens = if toplam <= TEVSIK_HADDI {
            match lcg(&mut s) % 100 {
                0..=51 => Enstruman::Nakit, 52..=76 => Enstruman::Banka,
                77..=87 => Enstruman::Cek, 88..=95 => Enstruman::Senet, _ => Enstruman::Acik,
            }
        } else {
            match lcg(&mut s) % 100 {
                0..=57 => Enstruman::Banka, 58..=71 => Enstruman::Cek,
                72..=83 => Enstruman::Senet, 84..=89 => Enstruman::Nakit, _ => Enstruman::Acik,
            }
        };
        // e-BELGE TİPİ: karşı taraf e-Fatura mükellefiyse e-Fatura, değilse e-Arşiv (509 VUK otomatik yönlendirme).
        // Gelen (alış) daima e-Fatura kanalıyla düşer (e-Arşiv gelen kutusuna gelmez).
        let efatura_muk = lcg(&mut s) % 100 < 55;
        let ebelge_tip = if !satis { "E_FATURA" } else if efatura_muk { "E_FATURA" } else { "E_ARSIV" };
        // e-BELGE DURUMU (ödeme/tahsilat durumundan bağımsız — GİB zarf durumu):
        let edurum = if satis {
            if ebelge_tip == "E_ARSIV" { "RAPORLANDI" } // e-Arşiv karşı onaya girmez, GİB'e raporlanır
            else { match lcg(&mut s) % 100 { 0..=4 => "TASLAK", 5..=14 => "GONDERILDI", 15..=89 => "KABUL", 90..=96 => "RED", _ => "IPTAL" } }
        } else {
            // Gelen: temel senaryo otomatik ALINDI; ticari senaryo 7 gün onay penceresi.
            match lcg(&mut s) % 100 { 0..=49 => "ALINDI", 50..=74 => "ONAY_BEKLIYOR", 75..=92 => "KABUL", _ => "RED" }
        };

        // KDV MUAVİNİ — oran defterde taşınmazsa KDV1 beyannamesi matrahı tek orandan geri
        // hesaplar ve yanlış beyan doğar. Kod biçimi hesaplama modülüyle aynı: 391.20, 191.01…
        // Muavin kodu ARDIŞIKTIR (391.01, 391.02…), oranın kendisi DEĞİLDİR — katalogdan çözülür.
        let mk = crate::hesaplama::muavin_kodu(kdv_oran);
        let k391 = format!("391.{mk}");
        let k191 = format!("191.{mk}");

        let (yon, no) = if satis { ("SATIS", format!("ACM2026{:07}", i)) } else { ("ALIS", format!("ALS2026{:07}", i)) };

        // GEÇERLİLİK — TASLAK henüz belge değil (hiç kayıt üretmez); IPTAL/RED hüküm doğurmaz
        // (tahakkuk yazılır, hemen ters kayıtla kapatılır — VUK 219: silme yok).
        let taslak = edurum == "TASLAK";
        let hukumsuz = edurum == "IPTAL" || edurum == "RED";
        let gecerli = !taslak && !hukumsuz;

        // Tahakkuk fişi. Tevkifatlı satışta 391'e yalnız tevkifat DIŞI pay yazılır;
        // ihracatta (kdv=0) KDV satırı hiç doğmaz ve hasılat 601'e gider.
        let tahakkuk = if taslak { None } else {
            let sat = if satis {
                let hasilat = if ihracat { "601" } else { "600" };
                // Tevkifatlı faturada alıcı, tevkif edilen KDV'yi SATICIYA ödemez; sorumlu
                // sıfatıyla 2 No.lu KDV beyannamesiyle vergi dairesine öder. Bu yüzden cariye
                // yalnız "ödenecek tutar" (toplam − tevkifat) borç yazılır.
                let mut v = vec![borc("120", toplam - tevkifat), alacak(hasilat, matrah)];
                if kdv - tevkifat > 0 { v.push(alacak(&k391, kdv - tevkifat)); }
                v
            } else {
                let mut v = vec![borc("153", matrah)];
                if kdv > 0 { v.push(borc(&k191, kdv)); }
                v.push(alacak("320", toplam));
                v
            };
            Some(Fis { no: format!("MAH-{}", i), tip: "Mahsup".into(), tarih: tarih.clone(),
                aciklama: format!("{} faturası tahakkuku — {}{}", yon, karsi,
                    if tevkifat > 0 { " (tevkifatlı)" } else if ihracat { " (ihracat istisnası)" } else { "" }),
                satirlar: sat })
        };

        // İPTAL/RED → ters kayıt. Borç ve alacak yer değiştirir, net etki sıfır, ikisi de defterde.
        let iptal_fisi = if hukumsuz {
            tahakkuk.as_ref().map(|t| Fis {
                no: format!("IPT-{}", i), tip: "Mahsup".into(), tarih: tarih.clone(),
                aciklama: format!("İPTAL — {} fişinin ters kaydı ({}); belge hüküm doğurmadı", t.no,
                    if edurum == "RED" { "alıcı reddetti" } else { "fatura iptal edildi" }),
                satirlar: t.satirlar.iter().map(|x| FisSat { hesap: x.hesap.clone(), borc: x.alacak, alacak: x.borc }).collect(),
            })
        } else { None };

        // SATIŞTAN İADE — yalnız GEÇERLİ faturada olur (mal teslim edilmiş, sonra geri gelmiş).
        // 600 TERS ÇEVRİLMEZ: 610'a yazılır (MSUGT gelir tablosu · iade oranı denetim göstergesi).
        let (iade_matrah, iade_kdv) = if gecerli && lcg(&mut s) % 100 < 6 {
            let oran = if lcg(&mut s) % 100 < 40 { 100 } else { 10 + (lcg(&mut s) % 50) as i64 };
            let im = matrah * oran / 100;
            (im, im * kdv_oran / 100)
        } else { (0, 0) };
        let iade_fisi = if iade_matrah > 0 {
            let t = iade_matrah + iade_kdv;
            let sat = if satis {
                // Satıcıda: 610 + 191 borç / 120 alacak (191 borçlanır — KDVK 35 matrah düzeltmesi)
                let mut v = vec![borc("610", iade_matrah)];
                if iade_kdv > 0 { v.push(borc(&k191, iade_kdv)); }
                v.push(alacak("120", t));
                v
            } else {
                // Alıştan iade: 320 borç / 153 + 191 alacak (indirilecek KDV azalır)
                let mut v = vec![borc("320", t), alacak("153", iade_matrah)];
                if iade_kdv > 0 { v.push(alacak(&k191, iade_kdv)); }
                v
            };
            Some(Fis { no: format!("IAD-{}", i), tip: "Mahsup".into(), tarih: tarih.clone(),
                aciklama: format!("{} iadesi — {} ({}%)", if satis { "Satıştan" } else { "Alıştan" }, karsi, iade_matrah * 100 / matrah),
                satirlar: sat })
        } else { None };

        // ── SATILAN MALIN MALİYETİ (621) — aralıksız envanter ────────────────────────
        // Hasılat kaydedilirken maliyeti de kaydedilmezse gelir tablosu %100 brüt marj gösterir
        // ve dönem kârı iki katına çıkar; kurumlar/geçici vergi matrahı da o kadar şişer.
        // MSUGT: satışta `621 borç / 153 alacak`, iadede tersi (`docs/learnings/iptal-iade-muhasebesi.md §2`).
        //
        // Maliyet oranı LCG akışından ÇEKİLMEZ — `i`'nin karıştırılmasından türetilir. LCG'den
        // çekilseydi sonraki tüm rastgele seçimler kayar ve mevcut veri kümesi baştan değişirdi.
        // Bant %55–66 (brüt marj %34–45); projeksiyon kodundaki ~%40 marj varsayımıyla uyumlu.
        let maliyet_oran = 55 + (((i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 33) % 12) as i64;
        // Hizmet satışında stok yoktur — yalnız mal satışında SMM doğar.
        let mal_satisi = satis && gecerli;
        let maliyet = if mal_satisi { matrah * maliyet_oran / 100 } else { 0 };
        let smm_fisi = if maliyet > 0 {
            Some(Fis { no: format!("SMM-{}", i), tip: "Mahsup".into(), tarih: tarih.clone(),
                aciklama: format!("Satılan ticari mallar maliyeti — {karsi} (maliyet oranı %{maliyet_oran})"),
                satirlar: vec![borc("621", maliyet), alacak("153", maliyet)] })
        } else { None };

        // İade edilen malın maliyeti stoka GERİ DÖNER. İade oranı matrah üzerinden hesaplandığı
        // için maliyet de aynı oranla döner — kısmi iadede kısmi maliyet.
        let iade_maliyet = if maliyet > 0 && iade_matrah > 0 { maliyet * iade_matrah / matrah } else { 0 };
        let iade_stok_fisi = if iade_maliyet > 0 {
            Some(Fis { no: format!("IAS-{}", i), tip: "Mahsup".into(), tarih: tarih.clone(),
                aciklama: format!("Satıştan iade — malın stoka dönüşü ({karsi})"),
                satirlar: vec![borc("153", iade_maliyet), alacak("621", iade_maliyet)] })
        } else { None };

        // FATURA SONRASI İSKONTO (611) — ciro primi / erken ödeme iskontosu.
        let (iskonto_matrah, iskonto_kdv) = if gecerli && iade_matrah == 0 && lcg(&mut s) % 100 < 5 {
            let im = matrah * (2 + (lcg(&mut s) % 8) as i64) / 100; // %2–9
            (im, im * kdv_oran / 100)
        } else { (0, 0) };
        let iskonto_fisi = if iskonto_matrah > 0 && satis {
            let t = iskonto_matrah + iskonto_kdv;
            let mut v = vec![borc("611", iskonto_matrah)];
            if iskonto_kdv > 0 { v.push(borc(&k191, iskonto_kdv)); }
            v.push(alacak("120", t));
            Some(Fis { no: format!("ISK-{}", i), tip: "Mahsup".into(), tarih: tarih.clone(),
                aciklama: format!("Satış iskontosu — {}", karsi), satirlar: v })
        } else { None };
        let (iskonto_matrah, iskonto_kdv) = if iskonto_fisi.is_some() { (iskonto_matrah, iskonto_kdv) } else { (0, 0) };

        // NET BORÇ — tahsis motorunun kapatacağı gerçek tutar.
        // Geçersiz belgede sıfır: ortada borç yok, tahsilat tahsis edilemez.
        // Faturanın "ödenecek tutar"ı = toplam − tevkifat (tevkif edilen KDV satıcıya ödenmez).
        // Tam iade bu tutarı sıfırlar; tevkifat da alıcı tarafında düzeltileceği için altına
        // düşemez — negatif cari borç ekonomik olarak anlamsızdır, sıfırda durur.
        let tahsil_edilebilir = toplam - tevkifat;
        let net_borc = if gecerli {
            (tahsil_edilebilir - (iade_matrah + iade_kdv) - (iskonto_matrah + iskonto_kdv)).max(0)
        } else { 0 };

        // Bu faturanın DOĞURDUĞU tahsilat — net borcu aşamaz. Geçersiz belge tahsilat doğurmaz.
        let tahsilat_tutar = if !gecerli || ens == Enstruman::Acik {
            0
        } else if lcg(&mut s) % 100 < 25 {
            let oran = 30 + (lcg(&mut s) % 55) as i64; // %30–84 kısmi
            net_borc * oran / 100
        } else {
            net_borc
        };

        // Tahsilat olayı cariye kaydedilir (faturaya değil) — tahsisi motor yapar.
        if tahsilat_tutar > 0 {
            tahsilatlar.push(TahsilatSim {
                id: format!("THS-{}", i), firma: karsi.clone(), yon: yon.into(), tarih: tarih.clone(),
                tutar: tahsilat_tutar, enstruman: ens.ad().into(), dagitilan: 0,
                durum: String::new(), sebep: String::new(),
                banka_ad: if ens == Enstruman::Banka { BANKA_AD.into() } else { String::new() },
                hesap_id: if ens == Enstruman::Banka { BANKA_HESAP.into() } else { String::new() },
                iban: if ens == Enstruman::Banka { BANKA_IBAN.into() } else { String::new() },
                fatura_bekleniyor: false, bekleyen_firma: String::new(),
            });
        }

        liste.push(FaturaSim {
            no, tarih, yon: yon.into(), karsi, matrah, kdv, toplam,
            enstruman: ens.ad().into(), odendi: false, tahsil: 0, kalan: net_borc,
            kdv_oran, istisna_kod, tevkifat,
            gecerli, iade_matrah, iade_kdv, iskonto_matrah, iskonto_kdv, net_borc,
            ebelge_tip: ebelge_tip.into(), edurum: edurum.into(),
            tahakkuk, iptal_fisi, iade_fisi, iskonto_fisi, odeme: None, tahsisler: Vec::new(),
            maliyet, iade_maliyet, smm_fisi, iade_stok_fisi,
        });
    }
    // ÜNVANSIZ GİRİŞLER — ekstrede karşı taraf ünvanı yok. Otomatik eşleşme yapılamaz
    // (hangi cariye ait olduğu bilinmiyor); maker'ın firma seçip fatura eşlemesi gerekir.
    // Eşleşmediği sürece "havada bakiye": tamlık ihlali / kayıt dışı hasılat riski (VUK, BDS 500).
    let mut s2 = 0x2026_4841u64;
    for n in 0..30 {
        let tutar = 5_000_00 + (lcg(&mut s2) % 95_000_00) as i64;
        let ay = 1 + (lcg(&mut s2) % 12) as u32;
        let gun = 1 + (lcg(&mut s2) % 28) as u32;
        tahsilatlar.push(TahsilatSim {
            id: format!("THS-H{:04}", n + 1),
            firma: String::new(), // ünvan YOK → otomatik eşleşme kapalı
            yon: "SATIS".into(),
            tarih: format!("{:02}.{:02}.2026", gun, ay),
            tutar, enstruman: "BANKA".into(), dagitilan: 0,
            durum: String::new(), sebep: String::new(),
            banka_ad: BANKA_AD.into(), hesap_id: BANKA_HESAP.into(), iban: BANKA_IBAN.into(),
            fatura_bekleniyor: false, bekleyen_firma: String::new(),
        });
    }

    (liste, tahsilatlar)
}

#[derive(Serialize)]
pub struct Ozet {
    pub fatura_sayisi: usize,
    pub fis_sayisi: usize,
    pub yevmiye_satiri: usize,
    pub hacim_tl: i64,
    pub toplam_borc: i64,
    pub toplam_alacak: i64,
    pub denge_ok: bool,
    pub enstruman: HashMap<String, usize>,
    pub acik_bakiye_tl: i64,
}

/// GET /api/simulasyon/ozet — kayıt sayısı, hacim, borç/alacak dengesi, enstrüman dağılımı.
pub async fn ozet(axum::extract::State(state): axum::extract::State<crate::Shared>) -> Json<Ozet> {
    let d = { let st = state.lock().unwrap(); crate::gorunum(&st) };
    let liste = d.faturalar.clone();
    let mut borc = 0i64;
    let mut alacak = 0i64;
    let mut fis = 0usize;
    let mut satir = 0usize;
    let mut ens: HashMap<String, usize> = HashMap::new();
    let mut acik = 0i64;
    let mut hacim = 0i64;
    for f in &liste {
        hacim += f.toplam;
        *ens.entry(f.enstruman.clone()).or_insert(0) += 1;
        acik += f.kalan; // kısmi dahil gerçek açık kalem
        // Faturanın ürettiği TÜM fişler: tahakkuk · iptal (storno) · iade (610) · iskonto (611) · ödeme
        let fisler = [&f.tahakkuk, &f.iptal_fisi, &f.iade_fisi, &f.iskonto_fisi, &f.smm_fisi, &f.iade_stok_fisi, &f.odeme];
        for fis_ref in fisler.into_iter().flatten() {
            fis += 1;
            for sr in &fis_ref.satirlar {
                satir += 1;
                borc += sr.borc;
                alacak += sr.alacak;
            }
        }
    }
    Json(Ozet {
        fatura_sayisi: liste.len(),
        fis_sayisi: fis,
        yevmiye_satiri: satir,
        hacim_tl: hacim / 100,
        toplam_borc: borc / 100,
        toplam_alacak: alacak / 100,
        denge_ok: borc == alacak,
        enstruman: ens,
        acik_bakiye_tl: acik / 100,
    })
}

/// GET /api/simulasyon/faturalar — SUNUCU-TARAFLI arama/filtre/sıralama + sayfalama {toplam, faturalar}.
/// Parametreler: ara (no/firma) · firma (tam) · yon (SATIS/ALIS) · durum (acik/odendi) · sirala
/// (tarih_desc/tarih_asc/tutar_desc) · offset · limit. Filtre TÜM veri üstünde, sonra sayfalanır.
pub async fn faturalar(axum::extract::State(state): axum::extract::State<crate::Shared>, Query(q): Query<HashMap<String, String>>) -> Json<serde_json::Value> {
    let d = { let st = state.lock().unwrap(); crate::gorunum(&st) };
    let mut liste = d.faturalar.clone();
    if let Some(a) = q.get("ara").filter(|s| !s.is_empty()) {
        let a = a.to_lowercase();
        liste.retain(|f| f.no.to_lowercase().contains(&a) || f.karsi.to_lowercase().contains(&a));
    }
    if let Some(firma) = q.get("firma").filter(|s| !s.is_empty()) {
        liste.retain(|f| &f.karsi == firma);
    }
    if let Some(yon) = q.get("yon").filter(|s| !s.is_empty()) {
        liste.retain(|f| &f.yon == yon);
    }
    if let Some(e) = q.get("enstruman").filter(|s| !s.is_empty()) {
        // Karma kapanan fatura ("KARMA") her iki enstrümanın süzgecinde de görünmeli:
        // banka süzgeci "banka parasıyla kapanan faturalar" demektir, "yalnız bankayla" değil.
        liste.retain(|f| &f.enstruman == e || f.tahsisler.iter().any(|t| &t.enstruman == e));
    }
    if let Some(mn) = q.get("tutar_min").and_then(|s| s.parse::<i64>().ok()) {
        liste.retain(|f| f.toplam >= mn); // kuruş
    }
    if let Some(mx) = q.get("tutar_max").and_then(|s| s.parse::<i64>().ok()) {
        liste.retain(|f| f.toplam <= mx);
    }
    if let Some(b) = q.get("tarih_bas").filter(|s| !s.is_empty()) {
        let k = tarih_key(b);
        liste.retain(|f| tarih_key(&f.tarih) >= k);
    }
    if let Some(b) = q.get("tarih_bit").filter(|s| !s.is_empty()) {
        let k = tarih_key(b);
        liste.retain(|f| tarih_key(&f.tarih) <= k);
    }
    // FATURA SAYFASI sekmesi: giden (e-Fatura satış) · gelen (alış) · earsiv · taslak.
    if let Some(sekme) = q.get("sekme").filter(|s| !s.is_empty()) {
        match sekme.as_str() {
            "giden" => liste.retain(|f| f.yon == "SATIS" && f.ebelge_tip == "E_FATURA" && f.edurum != "TASLAK"),
            "gelen" => liste.retain(|f| f.yon == "ALIS"),
            "earsiv" => liste.retain(|f| f.ebelge_tip == "E_ARSIV" && f.edurum != "TASLAK"),
            "taslak" => liste.retain(|f| f.edurum == "TASLAK"),
            _ => {}
        }
    }
    // e-belge durumu (GİB zarf durumu) filtresi
    if let Some(ed) = q.get("edurum").filter(|s| !s.is_empty()) {
        liste.retain(|f| &f.edurum == ed);
    }
    if let Some(tp) = q.get("ebelge_tip").filter(|s| !s.is_empty()) {
        liste.retain(|f| &f.ebelge_tip == tp);
    }
    // TAHSİS EDİLEBİLİR: eşleştirme ekranları bunu kullanmalı. "durum=acik" YETMEZ —
    // tamamı iade/iskonto edilmiş fatura da tahsil==0'dır ama net borcu sıfır olduğu için
    // tahsis KABUL ETMEZ (K06). Arayüz onu önerirse kullanıcı reddedilen bir işlem yapar.
    if q.get("tahsis_edilebilir").map(|s| s == "1").unwrap_or(false) {
        liste.retain(|f| f.gecerli && f.kalan > 0);
    }
    match q.get("durum").map(|s| s.as_str()) {
        Some("acik") => liste.retain(|f| f.tahsil == 0),
        Some("kismi") => liste.retain(|f| f.tahsil > 0 && f.kalan > 0),
        Some("odendi") => liste.retain(|f| f.kalan == 0),
        _ => {}
    }
    match q.get("sirala").map(|s| s.as_str()) {
        Some("tarih_desc") => liste.sort_by(|a, b| tarih_key(&b.tarih).cmp(&tarih_key(&a.tarih))),
        Some("tarih_asc") => liste.sort_by(|a, b| tarih_key(&a.tarih).cmp(&tarih_key(&b.tarih))),
        Some("tutar_desc") => liste.sort_by(|a, b| b.toplam.cmp(&a.toplam)),
        _ => {}
    }
    let toplam = liste.len();
    let acik_toplam: i64 = liste.iter().map(|f| f.kalan).sum();
    let offset: usize = q.get("offset").and_then(|x| x.parse().ok()).unwrap_or(0);
    let limit: usize = q.get("limit").and_then(|x| x.parse().ok()).unwrap_or(50).min(200);
    let sayfa: Vec<&FaturaSim> = liste.iter().skip(offset).take(limit).collect();
    Json(serde_json::json!({ "toplam": toplam, "acik_toplam": acik_toplam, "faturalar": sayfa }))
}

#[derive(Serialize)]
pub struct FirmaOzet { pub firma: String, pub adet: usize, pub acik: i64, pub toplam: i64 }

/// GET /api/simulasyon/firmalar — "aynı firmaları tek seferde": her firma için adet + açık + toplam.
pub async fn firmalar(axum::extract::State(state): axum::extract::State<crate::Shared>) -> Json<Vec<FirmaOzet>> {
    let d = { let st = state.lock().unwrap(); crate::gorunum(&st) };
    let mut m: HashMap<String, (usize, i64, i64)> = HashMap::new();
    for f in d.faturalar.iter() {
        let e = m.entry(f.karsi.clone()).or_insert((0, 0, 0));
        e.0 += 1;
        e.2 += f.toplam;
        e.1 += f.kalan; // firma açık bakiyesi = Σ kalan (mali önemlilik)
    }
    let mut v: Vec<FirmaOzet> = m.into_iter().map(|(firma, (adet, acik, toplam))| FirmaOzet { firma, adet, acik, toplam }).collect();
    v.sort_by(|a, b| b.acik.cmp(&a.acik));
    Json(v)
}

/// GET /api/simulasyon/tahsilatlar — cari bazlı tahsilat olayları + dağıtım durumu.
/// `firma` ve `yon` ile süzülür; `acik=1` yalnız tam dağıtılmamışları (avans/artık) getirir.
pub async fn tahsilatlar(
    axum::extract::State(state): axum::extract::State<crate::Shared>,
    Query(q): Query<HashMap<String, String>>,
) -> Json<serde_json::Value> {
    let d = { let st = state.lock().unwrap(); crate::gorunum(&st) };
    let manuel_n = { state.lock().unwrap().manuel_tahsis.len() };
    let mut liste = d.tahsilatlar.clone();
    if let Some(f) = q.get("firma").filter(|s| !s.is_empty()) { liste.retain(|t| &t.firma == f); }
    if let Some(y) = q.get("yon").filter(|s| !s.is_empty()) { liste.retain(|t| &t.yon == y); }
    if q.get("acik").map(|s| s == "1").unwrap_or(false) { liste.retain(|t| t.dagitilan < t.tutar); }
    liste.sort_by(|a, b| tarih_key(&a.tarih).cmp(&tarih_key(&b.tarih)));
    let toplam = liste.len();
    let avans: i64 = liste.iter().map(|t| t.tutar - t.dagitilan).sum();
    let limit: usize = q.get("limit").and_then(|x| x.parse().ok()).unwrap_or(100).min(500);
    liste.truncate(limit);
    Json(serde_json::json!({ "toplam": toplam, "avans": avans, "manuel_sayisi": manuel_n, "tahsilatlar": liste }))
}

/// GET /api/simulasyon/fatura-sekmeler — Fatura sayfası sekme rozetleri (adet + açık tutar).
pub async fn fatura_sekmeler(axum::extract::State(state): axum::extract::State<crate::Shared>) -> Json<serde_json::Value> {
    let d = { let st = state.lock().unwrap(); crate::gorunum(&st) };
    let liste = d.faturalar.clone();
    let say = |p: &dyn Fn(&FaturaSim) -> bool| -> (usize, i64) {
        liste.iter().filter(|f| p(f)).fold((0, 0), |(n, a), f| (n + 1, a + f.kalan))
    };
    let (g_n, g_a) = say(&|f| f.yon == "SATIS" && f.ebelge_tip == "E_FATURA" && f.edurum != "TASLAK");
    let (gel_n, gel_a) = say(&|f| f.yon == "ALIS");
    let (ea_n, ea_a) = say(&|f| f.ebelge_tip == "E_ARSIV" && f.edurum != "TASLAK");
    let (t_n, _) = say(&|f| f.edurum == "TASLAK");
    let onay_bekleyen = liste.iter().filter(|f| f.edurum == "ONAY_BEKLIYOR").count();
    Json(serde_json::json!({
        "giden":  { "adet": g_n,   "acik": g_a },
        "gelen":  { "adet": gel_n, "acik": gel_a },
        "earsiv": { "adet": ea_n,  "acik": ea_a },
        "taslak": { "adet": t_n },
        "onay_bekleyen": onay_bekleyen,
    }))
}

// ── Banka feed (Banka.tsx HareketlerYanit şekliyle uyumlu) ───────────────────
#[derive(Serialize)]
pub struct SimIslem {
    pub islem_no: String, pub referans_no: String, pub tarih: String,
    pub yon: &'static str, pub tutar: i64, pub aciklama: String,
    pub karsi_unvan: Option<String>, pub karsi_iban: Option<String>,
    pub kanal: &'static str, pub bakiye_sonrasi: i64,
}
#[derive(Serialize)]
pub struct SimHareketler {
    pub hesap_id: String, pub banka_ad: String, pub iban: String, pub para_birimi: String,
    pub acilis_bakiye: i64, pub kapanis_bakiye: i64, pub islemler: Vec<SimIslem>,
}

fn tarih_key(t: &str) -> String {
    let p: Vec<&str> = t.split('.').collect();
    if p.len() == 3 { format!("{}{}{}", p[2], p[1], p[0]) } else { t.to_string() }
}

/// İş Bankası hesap hareketleri: fatura ödemeleri (SIMREF, belgeli) + "havada" girişler (belgesiz).
fn banka_feed(d: &Dataset) -> (Vec<SimIslem>, i64, i64) {
    // Banka hareketi = TAHSİLAT olayı (faturaya değil cariye ait). Hangi faturayı kapattığı
    // tahsis motorunun sonucudur; muhasebeci bu satırdan manuel eşleştirme yapabilir.
    let mut hareketler: Vec<(String, SimIslem)> = d.tahsilatlar.iter().cloned()
        .into_iter()
        .filter(|t| t.enstruman == "BANKA")
        .enumerate()
        .map(|(i, t)| {
            let satis = t.yon == "SATIS";
            let unvansiz = t.firma.is_empty();
            (
                tarih_key(&t.tarih),
                SimIslem {
                    islem_no: format!("0064-{:06}", i + 1),
                    referans_no: t.id.clone(), // THS-n → tahsis motoruna bağlanır
                    tarih: t.tarih.clone(),
                    yon: if satis { "GIRIS" } else { "CIKIS" },
                    tutar: t.tutar,
                    aciklama: if unvansiz {
                        "HAVALE GELEN - AÇIKLAMASIZ (ünvan yok)".to_string()
                    } else {
                        format!("{} - {}", if satis { "HAVALE GELEN" } else { "EFT GIDEN" }, t.firma)
                    },
                    karsi_unvan: if unvansiz { None } else { Some(t.firma.clone()) },
                    karsi_iban: None,
                    kanal: if satis { "SUBE/İNTERNET" } else { "İNTERNET" },
                    bakiye_sonrasi: 0,
                },
            )
        })
        .collect();

    hareketler.sort_by(|a, b| a.0.cmp(&b.0));
    let acilis = 5_000_000_00i64;
    let mut bakiye = acilis;
    let islemler: Vec<SimIslem> = hareketler
        .into_iter()
        .map(|(_, mut h)| {
            if h.yon == "GIRIS" { bakiye += h.tutar; } else { bakiye -= h.tutar; }
            h.bakiye_sonrasi = bakiye;
            h
        })
        .collect();
    (islemler, acilis, bakiye)
}

/// GET /api/simulasyon/banka — İş Bankası hesap hareketleri (belgeli + havada).
pub async fn banka(axum::extract::State(state): axum::extract::State<crate::Shared>) -> Json<SimHareketler> {
    let d = { let st = state.lock().unwrap(); crate::gorunum(&st) };
    let (islemler, acilis, kapanis) = banka_feed(&d);
    Json(SimHareketler {
        hesap_id: "SIM-0064-01".into(),
        banka_ad: "Türkiye İş Bankası".into(),
        iban: "TR30 0064 0000 0000 1000".into(),
        para_birimi: "TRY".into(),
        acilis_bakiye: acilis,
        kapanis_bakiye: kapanis,
        islemler,
    })
}

#[derive(Serialize)]
pub struct HavadaYanit { pub adet: usize, pub toplam: i64, pub hareketler: Vec<SimIslem> }

/// GET /api/simulasyon/havada — "HAVADA BAKİYE": belgesiz banka girişleri (tamlık/kayıt dışı bulgusu).
pub async fn havada(axum::extract::State(state): axum::extract::State<crate::Shared>) -> Json<HavadaYanit> {
    let d = { let st = state.lock().unwrap(); crate::gorunum(&st) };
    // "Havada" = faturaya BAĞLANMAMIŞ para. Ölçüt `dagitilan == 0` DEĞİLDİR: kısmen dağıtılan
    // bir havalenin artan kısmı da havadadır. Eski ölçütle maker, 49.000 TL'lik ünvansız bir
    // girişin 370 TL'sini eşleştirerek kalemi listeden tümüyle düşürebiliyordu — kontrolün
    // kendisi devre dışı bırakılabiliyordu. `/api/havada` (Havada sayfası) zaten doğru
    // ölçütü kullanıyordu; bu uç (Banka sayfası) onunla çelişiyordu.
    let acik: std::collections::HashMap<String, i64> = d.tahsilatlar.iter()
        .filter(|t| t.dagitilan < t.tutar)
        .map(|t| (t.id.clone(), t.tutar - t.dagitilan)).collect();
    let (islemler, _, _) = banka_feed(&d);
    let hareketler: Vec<SimIslem> = islemler.into_iter()
        .filter(|i| i.yon == "GIRIS" && acik.contains_key(&i.referans_no))
        // Gösterilen tutar hareketin tamamı değil, HAVADA KALAN kısmıdır.
        .map(|mut i| { i.tutar = acik[&i.referans_no]; i })
        .collect();
    let toplam = hareketler.iter().map(|x| x.tutar).sum();
    Json(HavadaYanit { adet: hareketler.len(), toplam, hareketler })
}

// ─────────────────────────────────────────────────────────────────────────────
// TAHSİS MOTORU TESTLERİ — FIFO, para korunumu, manuel önceliği, ünvan koşulu.
// Bu değişmezler bozulursa cari bakiye ve dolayısıyla mizan yanlış olur.
#[cfg(test)]
mod tahsis_testleri {
    use super::*;

    fn saf() -> Dataset { dataset(&HashMap::new()) }

    /// Para korunumu: bir tahsilatın faturalara dağıttığı toplam, kendi tutarını AŞAMAZ
    /// ve `dagitilan` alanı bu toplamla birebir tutmalı.
    #[test]
    fn tahsilat_tutari_asilmaz_ve_dagitilan_tutar() {
        let d = saf();
        let mut dagitim: HashMap<String, i64> = HashMap::new();
        for f in &d.faturalar {
            for t in &f.tahsisler {
                *dagitim.entry(t.tahsilat_id.clone()).or_insert(0) += t.tutar;
                assert!(t.tutar > 0, "sıfır/negatif tahsis üretilemez: {}", t.tahsilat_id);
            }
        }
        for t in &d.tahsilatlar {
            let toplam = dagitim.get(&t.id).copied().unwrap_or(0);
            assert!(toplam <= t.tutar, "{} tahsilatı tutarından fazla dağıtıldı: {toplam} > {}", t.id, t.tutar);
            assert_eq!(toplam, t.dagitilan, "{} için dagitilan alanı gerçek dağıtımla uyuşmuyor", t.id);
        }
    }

    /// Fatura muhasebesi: tahsil = Σ tahsis, kalan = toplam − tahsil, aşırı tahsilat olamaz.
    #[test]
    fn fatura_tahsil_ve_kalan_tutarli() {
        for f in saf().faturalar {
            let toplam_tahsis: i64 = f.tahsisler.iter().map(|t| t.tutar).sum();
            assert_eq!(f.tahsil, toplam_tahsis, "{} tahsil alanı tahsislerle uyuşmuyor", f.no);
            // kalan artık NET BORÇ üzerinden: toplam − tevkifat − iade − iskonto − tahsil
            assert_eq!(f.kalan, f.net_borc - f.tahsil, "{} kalan hesabı yanlış", f.no);
            assert!(f.kalan >= 0, "{} aşırı tahsil edilmiş (kalan negatif)", f.no);
            assert_eq!(f.odendi, f.gecerli && f.kalan == 0, "{} odendi bayrağı kalanla çelişiyor", f.no);
        }
    }

    /// FIFO (TBK 100): bir carinin faturaları tarih sırasında kapanır — açık bir faturadan
    /// SONRAKİ tarihli bir fatura tam kapanmış olamaz.
    #[test]
    fn fifo_sirasi_korunur() {
        let d = saf();
        let mut kova: HashMap<(String, String), Vec<&FaturaSim>> = HashMap::new();
        for f in &d.faturalar {
            // Hükümsüz belge ve tamamı iade edilmiş fatura (net borç 0) FIFO kuyruğunda değil —
            // "kapanmış" görünürler ama para almadıkları için sıra testine girmemeliler.
            if !f.gecerli || f.net_borc == 0 { continue; }
            kova.entry((f.karsi.clone(), f.yon.clone())).or_default().push(f);
        }
        for ((firma, yon), mut liste) in kova {
            liste.sort_by(|a, b| tarih_key(&a.tarih).cmp(&tarih_key(&b.tarih)).then(a.no.cmp(&b.no)));
            let Some(ilk_acik) = liste.iter().position(|f| f.kalan > 0) else { continue };
            for f in &liste[ilk_acik + 1..] {
                assert!(f.kalan > 0,
                    "FIFO ihlali — {firma}/{yon}: {} kapanmış ama daha eski {} hâlâ açık",
                    f.no, liste[ilk_acik].no);
            }
        }
    }

    /// Henüz doğmamış borç ödenemez: tahsilat, kendisinden SONRAKİ tarihli faturayı kapatamaz.
    #[test]
    fn gelecek_tarihli_fatura_kapatilamaz() {
        let d = saf();
        let tarihler: HashMap<&str, &str> =
            d.tahsilatlar.iter().map(|t| (t.id.as_str(), t.tarih.as_str())).collect();
        for f in &d.faturalar {
            for t in &f.tahsisler {
                let Some(th_tarih) = tarihler.get(t.tahsilat_id.as_str()) else { continue };
                assert!(tarih_key(&f.tarih) <= tarih_key(th_tarih),
                    "{} ({}) tarihli fatura, {} ({}) tarihli tahsilatla kapatılmış",
                    f.no, f.tarih, t.tahsilat_id, th_tarih);
            }
        }
    }

    /// Ünvan koşulu: ekstrede karşı taraf ünvanı yoksa OTOMATİK eşleşme yapılmaz —
    /// yanlış cariye tahsis yanlış bakiye demektir; bunlar maker kuyruğuna düşer.
    #[test]
    fn unvansiz_tahsilat_otomatik_dagitilmaz() {
        let d = saf();
        let unvansiz: Vec<&TahsilatSim> = d.tahsilatlar.iter().filter(|t| t.firma.is_empty()).collect();
        assert!(!unvansiz.is_empty(), "senaryoda ünvansız giriş bulunmalı");
        for t in &unvansiz {
            assert_eq!(t.dagitilan, 0, "{} ünvansız olduğu halde otomatik dağıtıldı", t.id);
            assert_eq!(t.durum, "BEKLIYOR", "{} maker kuyruğunda olmalı", t.id);
            assert!(!t.sebep.is_empty(), "{} için maker'a sebep yazılmalı", t.id);
        }
    }

    /// Tahsis yalnız AYNI cariye ve AYNI yöne ait faturaya yazılabilir (otomatik dağıtımda).
    #[test]
    fn tahsis_carisi_ve_yonu_uyusur() {
        let d = saf();
        let ths: HashMap<&str, &TahsilatSim> =
            d.tahsilatlar.iter().map(|t| (t.id.as_str(), t)).collect();
        for f in &d.faturalar {
            for t in &f.tahsisler {
                let Some(k) = ths.get(t.tahsilat_id.as_str()) else { continue };
                if k.firma.is_empty() { continue; } // ünvansız: cari maker beyanıyla gelir
                assert_eq!(k.firma, f.karsi, "{} yanlış cariye tahsis edilmiş", t.tahsilat_id);
                assert_eq!(k.yon, f.yon, "{} yanlış yöne tahsis edilmiş", t.tahsilat_id);
            }
        }
    }

    /// Manuel karar FIFO karinesini EZER: maker'ın seçtiği fatura tahsisi alır ve
    /// bu tahsilatın önceki otomatik eşleşmesi kendiliğinden kalkar.
    #[test]
    fn manuel_tahsis_fifoyu_ezer() {
        let onceki = saf();
        // Otomatik eşleşmiş, ünvanı olan bir tahsilat ve onun mevcut hedefi
        let (ths, eski_hedef) = onceki.faturalar.iter()
            .find_map(|f| f.tahsisler.first().map(|t| (t.tahsilat_id.clone(), f.clone())))
            .expect("otomatik eşleşme bulunmalı");
        let t = onceki.tahsilatlar.iter().find(|x| x.id == ths).expect("tahsilat bulunmalı");

        // Aynı cari + yön, ama BAŞKA bir fatura seç (en yenisi — FIFO'nun tersi)
        let yeni_hedef = onceki.faturalar.iter()
            .filter(|f| f.karsi == t.firma && f.yon == t.yon && f.no != eski_hedef.no)
            .max_by_key(|f| tarih_key(&f.tarih))
            .expect("aynı caride başka fatura bulunmalı")
            .no.clone();

        let manuel: HashMap<String, String> = [(ths.clone(), yeni_hedef.clone())].into_iter().collect();
        let sonra = dataset(&manuel);

        let hedef = sonra.faturalar.iter().find(|f| f.no == yeni_hedef).unwrap();
        let pay = hedef.tahsisler.iter().find(|x| x.tahsilat_id == ths)
            .unwrap_or_else(|| panic!("manuel tahsis {yeni_hedef} faturasına yazılmadı"));
        assert_eq!(pay.kaynak, "MANUEL", "manuel karar FIFO olarak işaretlenmiş");

        // Para korunumu manuel müdahaleden sonra da bozulmamalı.
        let dagitim: i64 = sonra.faturalar.iter()
            .flat_map(|f| &f.tahsisler).filter(|x| x.tahsilat_id == ths).map(|x| x.tutar).sum();
        assert!(dagitim <= t.tutar, "manuel tahsis sonrası tahsilat tutarı aşıldı");
    }

    /// Determinizm: motor sabit tohumla çalışır — aynı girdi her zaman aynı çıktıyı verir.
    /// (Bozulursa denetim izi tekrar üretilemez hale gelir.)
    #[test]
    fn motor_deterministik() {
        let a = saf();
        let b = saf();
        assert_eq!(a.faturalar.len(), b.faturalar.len());
        for (x, y) in a.faturalar.iter().zip(b.faturalar.iter()) {
            assert_eq!(x.no, y.no);
            assert_eq!(x.tahsil, y.tahsil);
            assert_eq!(x.kalan, y.kalan);
            assert_eq!(x.tahsisler.len(), y.tahsisler.len());
        }
    }

    /// Yevmiye dengesi: üretilen tüm fişlerde Σborç = Σalacak (çift taraflı kayıt).
    #[test]
    fn uretilen_fisler_dengeli() {
        let d = saf();
        let mut borc = 0i64;
        let mut alacak = 0i64;
        for f in &d.faturalar {
            for fis in [&f.tahakkuk, &f.iptal_fisi, &f.iade_fisi, &f.iskonto_fisi, &f.smm_fisi, &f.iade_stok_fisi, &f.odeme].into_iter().flatten() {
                let b: i64 = fis.satirlar.iter().map(|s| s.borc).sum();
                let a: i64 = fis.satirlar.iter().map(|s| s.alacak).sum();
                assert_eq!(b, a, "{} fişi dengesiz", fis.no);
                borc += b;
                alacak += a;
            }
        }
        assert_eq!(borc, alacak, "toplam yevmiye dengesi bozuk");
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// İPTAL / İADE / KDV VARYASYONLARI — docs/learnings/iptal-iade-muhasebesi.md kuralları.
#[cfg(test)]
mod iptal_iade_testleri {
    use super::*;

    fn veri() -> Dataset { dataset(&HashMap::new()) }

    /// SATILAN MALIN MALİYETİ (621) — hasılatla AYNI dönemde, aynı faturada doğmalı.
    /// Kaydedilmezse gelir tablosu %100 brüt marj gösterir ve vergi matrahı iki kata çıkar.
    #[test]
    fn satista_smm_dogar_ve_stoktan_duser() {
        let d = veri();
        let mut sayac = 0;
        for f in d.faturalar.iter().filter(|f| f.yon == "SATIS" && f.gecerli) {
            let fis = f.smm_fisi.as_ref()
                .unwrap_or_else(|| panic!("{} geçerli satış ama SMM fişi yok", f.no));
            let (b621, a621) = hesap_toplam(fis, "621");
            let (b153, a153) = hesap_toplam(fis, "153");
            assert_eq!(b621, f.maliyet, "{} · 621 borç maliyete eşit olmalı", f.no);
            assert_eq!(a153, f.maliyet, "{} · 153 alacak maliyete eşit olmalı", f.no);
            assert_eq!((a621, b153), (0, 0), "{} · SMM fişinde ters bacak olmamalı", f.no);
            // Maliyet hasılatı aşarsa zararına satış olur — simülasyonda kurgulanmadı.
            assert!(f.maliyet > 0 && f.maliyet < f.matrah,
                "{} · maliyet {} matrahın ({}) dışında", f.no, f.maliyet, f.matrah);
            sayac += 1;
        }
        assert!(sayac > 100, "yeterli satış faturası yok: {sayac}");
        // Alışta SMM doğmaz — alış stoğu ARTIRIR, azaltmaz.
        assert!(d.faturalar.iter().filter(|f| f.yon == "ALIS").all(|f| f.smm_fisi.is_none()),
            "alış faturası SMM üretti — stok çıkışı satışta doğar");
    }

    /// Satıştan iadede mal stoka GERİ DÖNER (153 borç / 621 alacak) — kısmi iadede kısmi maliyet.
    #[test]
    fn satistan_iade_stogu_geri_getirir() {
        let d = veri();
        let mut sayac = 0;
        for f in d.faturalar.iter().filter(|f| f.yon == "SATIS" && f.gecerli && f.iade_matrah > 0) {
            let fis = f.iade_stok_fisi.as_ref()
                .unwrap_or_else(|| panic!("{} iadeli ama stok dönüş fişi yok", f.no));
            let (b153, _) = hesap_toplam(fis, "153");
            let (_, a621) = hesap_toplam(fis, "621");
            assert_eq!(b153, f.iade_maliyet, "{} · iade maliyeti 153'e dönmedi", f.no);
            assert_eq!(a621, f.iade_maliyet, "{} · iade maliyeti 621'den düşmedi", f.no);
            // İade oranı ile maliyet oranı tutarlı olmalı: kısmi iade → kısmi maliyet.
            assert!(f.iade_maliyet <= f.maliyet, "{} · iade maliyeti satış maliyetini aşamaz", f.no);
            assert_eq!(f.iade_maliyet, f.maliyet * f.iade_matrah / f.matrah,
                "{} · iade maliyeti orantılı değil", f.no);
            sayac += 1;
        }
        assert!(sayac > 0, "iadeli satış faturası bulunamadı");
    }

    /// STOK NEGATİFE DÜŞEMEZ — satılamayan mal satılmış görünürse envanter uydurmadır.
    /// 153 = alışlar − alıştan iadeler − SMM + satıştan iadeler.
    #[test]
    fn stok_bakiyesi_negatife_dusmez() {
        let d = veri();
        let (mut borc, mut alacak) = (0i64, 0i64);
        for f in &d.faturalar {
            for fis in [&f.tahakkuk, &f.iptal_fisi, &f.iade_fisi, &f.iskonto_fisi,
                        &f.smm_fisi, &f.iade_stok_fisi].into_iter().flatten() {
                let (b, a) = hesap_toplam(fis, "153");
                borc += b; alacak += a;
            }
        }
        assert!(borc - alacak >= 0,
            "153 Ticari Mallar ALACAK bakiye verdi ({} kuruş) — stok negatife düştü, \
             satılan mal alınandan fazla", borc - alacak);
        // Brüt marj makul bantta olmalı; %100 marj (SMM=0) veya negatif marj kurgu hatasıdır.
        let smm: i64 = d.faturalar.iter().map(|f| f.maliyet - f.iade_maliyet).sum();
        let hasilat: i64 = d.faturalar.iter()
            .filter(|f| f.yon == "SATIS" && f.gecerli).map(|f| f.matrah - f.iade_matrah).sum();
        let marj = (hasilat - smm) * 100 / hasilat;
        assert!((25..=55).contains(&marj), "brüt marj %{marj} — gerçekçi bantta değil");
    }

    /// Hesap AİLESİ toplamı: 391 sorulduğunda 391.20 / 391.10 … hepsi sayılır.
    /// KDV artık muavin kırılımıyla yazılıyor (oran defterde taşınmalı, bkz. KDV1 beyannamesi).
    fn hesap_toplam(f: &Fis, kod: &str) -> (i64, i64) {
        let onek = format!("{kod}.");
        f.satirlar.iter()
            .filter(|s| s.hesap == kod || s.hesap.starts_with(&onek))
            .fold((0, 0), |(b, a), s| (b + s.borc, a + s.alacak))
    }

    /// TASLAK fatura HİÇ kayıt üretmez — henüz belge değildir (VUK 229).
    #[test]
    fn taslak_fatura_kayit_uretmez() {
        let d = veri();
        let taslaklar: Vec<&FaturaSim> = d.faturalar.iter().filter(|f| f.edurum == "TASLAK").collect();
        assert!(!taslaklar.is_empty(), "senaryoda taslak fatura bulunmalı");
        for f in taslaklar {
            assert!(f.tahakkuk.is_none(), "{} taslak olduğu halde tahakkuk üretmiş", f.no);
            assert!(!f.gecerli && f.net_borc == 0, "{} taslak cari bakiye doğurmamalı", f.no);
            assert!(f.tahsisler.is_empty(), "{} taslağa tahsilat tahsis edilmiş", f.no);
        }
    }

    /// IPTAL/RED faturada tahakkuk YAZILIR ama hemen ters kayıtla kapatılır (VUK 219: silme yok).
    /// Net etki sıfır olmalı ve fatura cari bakiye doğurmamalı.
    #[test]
    fn iptal_ters_kayitla_kapanir_net_sifir() {
        let d = veri();
        let iptaller: Vec<&FaturaSim> =
            d.faturalar.iter().filter(|f| f.edurum == "IPTAL" || f.edurum == "RED").collect();
        assert!(!iptaller.is_empty(), "senaryoda iptal/red fatura bulunmalı");
        for f in iptaller {
            let th = f.tahakkuk.as_ref().unwrap_or_else(|| panic!("{} tahakkuku yok", f.no));
            let ip = f.iptal_fisi.as_ref().unwrap_or_else(|| panic!("{} ters kaydı yok", f.no));

            // Ters kayıt: her hesapta borç↔alacak yer değiştirmiş olmalı, net etki sıfır.
            for kod in ["120", "600", "601", "391", "153", "191", "320"] {
                let (tb, ta) = hesap_toplam(th, kod);
                let (ib, ia) = hesap_toplam(ip, kod);
                assert_eq!(tb, ia, "{} — {kod} hesabında ters kayıt borcu asıl alacağı karşılamıyor", f.no);
                assert_eq!(ta, ib, "{} — {kod} hesabında ters kayıt alacağı asıl borcu karşılamıyor", f.no);
            }
            assert!(!f.gecerli, "{} hükümsüz belge geçerli işaretlenmiş", f.no);
            assert_eq!(f.net_borc, 0, "{} iptal edildiği halde cari borç doğurmuş", f.no);
            assert!(f.tahsisler.is_empty(), "{} iptal edilmiş faturaya tahsilat yazılmış", f.no);
        }
    }

    /// SATIŞTAN İADE 600'ü TERS ÇEVİRMEZ — 610'a yazılır (MSUGT gelir tablosu formatı).
    /// Aksi halde brüt satış bilgisi kaybolur ve iade oranı denetimde görünmez.
    #[test]
    fn iade_600u_ters_cevirmez_610_kullanir() {
        let d = veri();
        let iadeler: Vec<&FaturaSim> =
            d.faturalar.iter().filter(|f| f.yon == "SATIS" && f.iade_matrah > 0).collect();
        assert!(!iadeler.is_empty(), "senaryoda satıştan iade bulunmalı");
        for f in iadeler {
            let ia = f.iade_fisi.as_ref().unwrap_or_else(|| panic!("{} iade fişi yok", f.no));
            let (b610, _) = hesap_toplam(ia, "610");
            assert_eq!(b610, f.iade_matrah, "{} — iade matrahı 610'a yazılmamış", f.no);
            // 600 iade fişinde HİÇ yer almamalı
            let (b600, a600) = hesap_toplam(ia, "600");
            assert_eq!((b600, a600), (0, 0), "{} — iade kaydında 600 ters çevrilmiş (yanlış)", f.no);
            // KDVK 35: matrah düzeltmesi → 191 BORÇLANIR
            if f.iade_kdv > 0 {
                let (b191, a191) = hesap_toplam(ia, "191");
                assert_eq!(b191, f.iade_kdv, "{} — iade KDV'si 191'e borç yazılmamış", f.no);
                assert_eq!(a191, 0, "{} — satıştan iadede 191 alacaklanamaz", f.no);
            }
        }
    }

    /// ALIŞTAN İADE: 320 borç / 153 + 191 alacak — indirilecek KDV AZALIR.
    #[test]
    fn alistan_iade_191i_alacaklandirir() {
        let d = veri();
        let iadeler: Vec<&FaturaSim> =
            d.faturalar.iter().filter(|f| f.yon == "ALIS" && f.iade_matrah > 0).collect();
        assert!(!iadeler.is_empty(), "senaryoda alıştan iade bulunmalı");
        for f in iadeler {
            let ia = f.iade_fisi.as_ref().unwrap();
            let (b320, _) = hesap_toplam(ia, "320");
            let (_, a153) = hesap_toplam(ia, "153");
            assert_eq!(b320, f.iade_matrah + f.iade_kdv, "{} — 320 borcu yanlış", f.no);
            assert_eq!(a153, f.iade_matrah, "{} — stok 153'ten çıkmamış", f.no);
            if f.iade_kdv > 0 {
                let (b191, a191) = hesap_toplam(ia, "191");
                assert_eq!(a191, f.iade_kdv, "{} — alıştan iadede 191 alacaklanmalı", f.no);
                assert_eq!(b191, 0, "{} — alıştan iadede 191 borçlanamaz", f.no);
            }
        }
    }

    /// Cari açık bakiye = toplam − iade − iskonto − tahsilat. İade/iskonto borcu azaltır.
    #[test]
    fn net_borc_iade_ve_iskontoyu_dusur() {
        for f in veri().faturalar {
            if !f.gecerli { assert_eq!(f.net_borc, 0, "{} hükümsüz, net borç sıfır olmalı", f.no); continue; }
            let beklenen = ((f.toplam - f.tevkifat)
                - (f.iade_matrah + f.iade_kdv) - (f.iskonto_matrah + f.iskonto_kdv)).max(0);
            assert_eq!(f.net_borc, beklenen, "{} net borç hesabı yanlış", f.no);
            assert!(f.net_borc >= 0, "{} net borç negatif olamaz", f.no);
            assert!(f.net_borc <= f.toplam, "{} net borç fatura toplamını aşamaz", f.no);
            assert_eq!(f.kalan, f.net_borc - f.tahsil, "{} kalan hesabı net borçla uyuşmuyor", f.no);
        }
    }

    /// KDV oranı gerçekçi dağılmalı ve tutar oranla tutmalı; KDV=0 ise istisna kodu ZORUNLU.
    #[test]
    fn kdv_oranlari_ve_istisna_kodu() {
        let d = veri();
        let mut gorulen: Vec<i64> = d.faturalar.iter().map(|f| f.kdv_oran).collect();
        gorulen.sort_unstable();
        gorulen.dedup();
        for o in [1, 10, 20] {
            assert!(gorulen.contains(&o), "%{o} KDV oranı senaryoda yok — tek oran gerçekçi değil");
        }
        for f in &d.faturalar {
            assert_eq!(f.kdv, f.matrah * f.kdv_oran / 100, "{} KDV tutarı oranla uyuşmuyor", f.no);
            if f.kdv_oran == 0 {
                assert!(!f.istisna_kod.is_empty(),
                    "{} — KDV sıfır ama istisna sebep kodu yok (UBL-TR zorunlu alan)", f.no);
            }
        }
    }

    /// İhracat istisnasında hasılat 601'e yazılır ve 391 HİÇ doğmaz.
    #[test]
    fn ihracat_601e_yazilir_kdv_dogmaz() {
        let d = veri();
        let ihracat: Vec<&FaturaSim> = d.faturalar.iter()
            .filter(|f| f.yon == "SATIS" && f.kdv_oran == 0 && f.tahakkuk.is_some()).collect();
        assert!(!ihracat.is_empty(), "senaryoda ihracat faturası bulunmalı");
        for f in ihracat {
            let th = f.tahakkuk.as_ref().unwrap();
            let (_, a601) = hesap_toplam(th, "601");
            assert_eq!(a601, f.matrah, "{} — ihracat hasılatı 601'e yazılmamış", f.no);
            let (b391, a391) = hesap_toplam(th, "391");
            assert_eq!((b391, a391), (0, 0), "{} — istisnada hesaplanan KDV doğmamalı", f.no);
        }
    }

    /// TEVKİFAT (KDVK 9): satıcıda 391'e yalnız tevkifat DIŞI pay yazılır; kalanı alıcı beyan eder.
    #[test]
    fn tevkifatli_satista_391_kismi_yazilir() {
        let d = veri();
        let tevkifatli: Vec<&FaturaSim> =
            d.faturalar.iter().filter(|f| f.tevkifat > 0 && f.tahakkuk.is_some()).collect();
        assert!(!tevkifatli.is_empty(), "senaryoda tevkifatlı fatura bulunmalı");
        for f in tevkifatli {
            assert_eq!(f.yon, "SATIS", "{} tevkifat yalnız satışta kurgulanır", f.no);
            let th = f.tahakkuk.as_ref().unwrap();
            let (_, a391) = hesap_toplam(th, "391");
            assert_eq!(a391, f.kdv - f.tevkifat, "{} — 391'e tevkifat dışı pay yazılmalı", f.no);
            assert!(f.tevkifat < f.kdv, "{} tevkifat KDV'nin tamamı olamaz", f.no);
        }
    }

    /// Her fatura fişi tek tek dengeli olmalı — iade/iskonto/iptal dahil.
    #[test]
    fn tum_fisler_tek_tek_dengeli() {
        for f in veri().faturalar {
            for fis in [&f.tahakkuk, &f.iptal_fisi, &f.iade_fisi, &f.iskonto_fisi, &f.smm_fisi, &f.iade_stok_fisi, &f.odeme].into_iter().flatten() {
                let b: i64 = fis.satirlar.iter().map(|s| s.borc).sum();
                let a: i64 = fis.satirlar.iter().map(|s| s.alacak).sum();
                assert_eq!(b, a, "{} fişi dengesiz ({})", fis.no, f.no);
                assert!(b > 0, "{} boş fiş üretilmiş", fis.no);
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// KASA DEFTERİ + TEVSİK ZORUNLULUĞU
// Nakit ticaretin gerçeği: her tahsilat bankadan geçmez. Ama VUK mükerrer 257 / 459 sıra no.lu
// tebliğ, haddi aşan tahsilat ve ödemelerin banka/PTT üzerinden yapılmasını zorunlu kılar.
// Haddi aşan NAKİT hareket geçerli bir kayıttır ama ÖZEL USULSÜZLÜK riski taşır → denetim bulgusu.

/// Tevsik haddi (kuruş). DOĞRULANMADI: 2026 haddi tebliğden teyit edilmeli;
/// kalıcı çözümde data/vergi-parametreleri.json'dan sürümlü okunmalı, koda gömülü kalmamalı.
pub const TEVSIK_HADDI: i64 = 7_000_00;

#[derive(Serialize)]
pub struct KasaHareket {
    pub tahsilat_id: String, pub tarih: String, pub firma: String,
    pub yon: String, pub tutar: i64, pub bakiye_sonrasi: i64,
    /// Haddi aşan nakit → tevsik zorunluluğu ihlali (uyum bulgusu; kayıt geçerli).
    pub tevsik_ihlali: bool,
}

/// GET /api/simulasyon/kasa — kasa (100) defteri + tevsik zorunluluğu bulguları.
pub async fn kasa(axum::extract::State(state): axum::extract::State<crate::Shared>) -> Json<serde_json::Value> {
    let d = { let st = state.lock().unwrap(); crate::gorunum(&st) };
    Json(kasa_defteri(&d))
}

/// Kasa defterini hesaplar. SAF fonksiyon: kasa ucu ve deftere aktarım aynı sonucu görsün diye
/// ayrıldı — aksi halde kasa ekranındaki bakiye ile 100 hesabının bakiyesi tutmazdı.
pub fn kasa_defteri(d: &Dataset) -> serde_json::Value {
    let mut nakit: Vec<&TahsilatSim> = d.tahsilatlar.iter().filter(|t| t.enstruman == "NAKIT").collect();
    nakit.sort_by(|a, b| tarih_key(&a.tarih).cmp(&tarih_key(&b.tarih)).then(a.id.cmp(&b.id)));

    // BANKA → KASA VİRMANI: işletme kasayı bankadan besler (100 borç / 102 alacak).
    // Bu bir tahsilat değil, iç aktarımdır; kasanın negatife düşmemesi bununla sağlanır.
    // Ay başlarında, o ayın nakit çıkışını karşılayacak biçimde deterministik hesaplanır.
    let mut aylik_cikis: HashMap<String, i64> = HashMap::new();
    for t in &nakit {
        if t.yon != "SATIS" {
            *aylik_cikis.entry(t.tarih[3..].to_string()).or_insert(0) += t.tutar;
        }
    }

    let acilis = 250_000_00i64;
    let mut bakiye = acilis;
    let mut hareketler = Vec::new();
    let mut son_ay = String::new();
    let mut ihlal_adet = 0usize;
    let mut ihlal_tutar = 0i64;
    let mut virmanlar: Vec<(String, i64)> = Vec::new(); // (tarih, tutar) → deftere 100/102 fişi
    for t in nakit {
        // Yeni aya girerken kasayı besle (banka→kasa virmanı).
        let ay = t.tarih[3..].to_string();
        if ay != son_ay {
            son_ay = ay.clone();
            let ihtiyac = aylik_cikis.get(&ay).copied().unwrap_or(0);
            if ihtiyac > 0 {
                bakiye += ihtiyac;
                virmanlar.push((format!("01.{ay}"), ihtiyac));
                hareketler.push(KasaHareket {
                    tahsilat_id: format!("VIR-{}", ay.replace('.', "")),
                    tarih: format!("01.{ay}"), firma: "— Banka→Kasa virmanı (100/102)".into(),
                    yon: "GIRIS".into(), tutar: ihtiyac, bakiye_sonrasi: bakiye,
                    tevsik_ihlali: false, // iç aktarım, tevsik kapsamında değil
                });
            }
        }
        // SATIS = kasaya giriş (tahsilat) · ALIS = kasadan çıkış (ödeme)
        let giris = t.yon == "SATIS";
        if giris { bakiye += t.tutar } else { bakiye -= t.tutar }
        let ihlal = t.tutar > TEVSIK_HADDI;
        if ihlal { ihlal_adet += 1; ihlal_tutar += t.tutar; }
        hareketler.push(KasaHareket {
            tahsilat_id: t.id.clone(), tarih: t.tarih.clone(), firma: t.firma.clone(),
            yon: if giris { "GIRIS".into() } else { "CIKIS".into() },
            tutar: t.tutar, bakiye_sonrasi: bakiye, tevsik_ihlali: ihlal,
        });
    }
    // Kasa bakiyesi NEGATİFE düşemez — fiilen olmayan parayı ödemek mümkün değildir.
    // Negatif kasa, klasik bir denetim bulgusudur (kayıt dışı hasılat / eksik belge göstergesi).
    let negatif_gun = hareketler.iter().filter(|h| h.bakiye_sonrasi < 0).count();
    let virman_toplam: i64 = hareketler.iter().filter(|h| h.tahsilat_id.starts_with("VIR-")).map(|h| h.tutar).sum();

    serde_json::json!({
        "acilis_bakiye": acilis,
        "kapanis_bakiye": bakiye,
        "hareket_sayisi": hareketler.len(),
        "tevsik": {
            "had": TEVSIK_HADDI,
            "ihlal_adet": ihlal_adet,
            "ihlal_tutar": ihlal_tutar,
            "aciklama": "Haddi aşan nakit tahsilat/ödeme banka/PTT üzerinden yapılmalıydı (VUK mük. 257) — özel usulsüzlük riski",
            "dogrulanmadi": "2026 haddi tebliğden teyit edilmeli",
        },
        "negatif_bakiye_gun": negatif_gun,
        "banka_kasa_virmani": virman_toplam,
        "virmanlar": virmanlar,
        "hareketler": hareketler.into_iter().take(200).collect::<Vec<_>>(),
    })
}

#[cfg(test)]
mod kasa_testleri {
    use super::*;

    /// Nakit hareketler kasa defterine düşmeli ve tevsik haddini aşanlar bulgulanmalı.
    #[test]
    fn tevsik_haddi_asan_nakit_bulgulanir() {
        let d = dataset(&HashMap::new());
        let nakit: Vec<&TahsilatSim> = d.tahsilatlar.iter().filter(|t| t.enstruman == "NAKIT").collect();
        assert!(!nakit.is_empty(), "senaryoda nakit tahsilat bulunmalı (banka-dışı ticaret gerçeği)");
        let asan = nakit.iter().filter(|t| t.tutar > TEVSIK_HADDI).count();
        assert!(asan > 0, "haddi aşan nakit hareket senaryoda bulunmalı — bulgu üretilebilsin");
    }

    /// Enstrüman dağılımı gerçekçi olmalı: her şey bankadan geçmez.
    #[test]
    fn enstruman_dagilimi_banka_disini_da_icerir() {
        let d = dataset(&HashMap::new());
        for e in ["BANKA", "NAKIT", "CEK", "SENET"] {
            assert!(d.tahsilatlar.iter().any(|t| t.enstruman == e),
                "{e} enstrümanı senaryoda yok — banka-dışı ticaret temsil edilmiyor");
        }
    }
}

#[cfg(test)]
mod tarih_dagilim_testi {
    use super::*;
    /// Faturalar 12 aya YAYILMALI. Belirli aylarda hiç kayıt olmaması, beyanname
    /// dönemlerinin boş çıkmasına yol açar (KDV her ay beyan edilir).
    #[test]
    fn faturalar_tum_aylara_yayilir() {
        let d = dataset(&HashMap::new());
        let mut ay: HashMap<String, usize> = HashMap::new();
        for f in &d.faturalar { *ay.entry(f.tarih[3..5].to_string()).or_insert(0) += 1; }
        let mut liste: Vec<_> = ay.iter().collect();
        liste.sort();
        // Yalnız "her ayda kayıt var mı" yetmez — DAĞILIM da dengeli olmalı. Aylar arası
        // 3 kattan fazla fark, üreteçte sistematik bir çarpıklığa işarettir (LCG düşük bit tuzağı).
        let en_az = ay.values().copied().min().unwrap_or(0);
        let en_cok = ay.values().copied().max().unwrap_or(0);
        assert_eq!(ay.len(), 12, "12 ayın hepsinde fatura olmalı · dağılım: {liste:?}");
        assert!(en_cok <= en_az * 3,
            "aylar arası dağılım çarpık (en az {en_az}, en çok {en_cok}) · dağılım: {liste:?}");
    }
}
