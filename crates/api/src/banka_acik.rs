//! Açık bankacılık (ÖHVPS) **SİMÜLATÖRÜ** — İş Bankası'na bağlanınca 10 bankanın hesap hareketlerini
//! çekiyormuş gibi deterministik veri üretir. İzole modül: AppState'e/diğer sayfalara BAĞLI DEĞİL
//! (birindeki sorun diğerini etkilemesin ilkesi). Gerçek İş B. istemcisi bu DTO sözleşmesinin yerine
//! geçecek (aynı JSON şekli → frontend değişmez).
//!
//! Deterministik: sabit seed, `Date::now`/random YOK — tekrar üretilebilir (test + demo tutarlı).
//! Tutarlar tamsayı **kuruş**; tarih "gg.aa.yyyy". Bakiye zinciri tutarlı (zincir_dogrula test edilebilir).

use axum::{extract::Query, Json};
use serde::Serialize;
use std::collections::HashMap;

/// Simüle bankalar (kod = EFT/BIC benzeri; simülasyon amaçlı, resmî değil).
const BANKALAR: &[(&str, &str)] = &[
    ("0064", "Türkiye İş Bankası"),
    ("0062", "Garanti BBVA"),
    ("0046", "Akbank"),
    ("0067", "Yapı Kredi"),
    ("0015", "VakıfBank"),
    ("0010", "Ziraat Bankası"),
    ("0012", "Halkbank"),
    ("0111", "QNB"),
    ("0134", "Denizbank"),
    ("0205", "Kuveyt Türk"),
];

const KARSI_UNVANLAR: &[&str] = &[
    "ACME TEKSTİL SAN. TİC. A.Ş.",
    "BEYAZ LOJİSTİK LTD. ŞTİ.",
    "DEMİR İNŞAAT A.Ş.",
    "EGE GIDA PAZARLAMA LTD.",
    "FİLİZ AMBALAJ SAN.",
    "GÜNEŞ ENERJİ A.Ş.",
    "MERT OTOMOTİV LTD.",
];

/// (açıklama şablonu, yön: true=giriş/false=çıkış, taban tutar kuruş, karşı-taraf var mı)
const ISLEM_DESENLERI: &[(&str, bool, i64, bool)] = &[
    ("HAVALE GELEN - {u}", true, 1_250_00, true),
    ("FAST GELEN - {u}", true, 875_00, true),
    ("EFT GIDEN - {u}", false, 1_100_00, true),
    ("HAVALE GIDEN - {u}", false, 640_00, true),
    ("POS HAKEDIS", true, 3_400_00, false),
    ("MAAS ODEME", false, 2_800_00, false),
    ("KREDI TAKSIT ODEME", false, 1_950_00, false),
    ("EFT UCRETI", false, 10_50, false),
    ("HESAP ISLETIM UCRETI", false, 45_00, false),
    ("VERGI ODEME - GIB", false, 5_200_00, false),
    ("SGK ODEME", false, 3_100_00, false),
];

// ── DTO'lar (ÖHVPS islemler şekline yakın) ──────────────────────────────────
#[derive(Serialize)]
pub struct HesapOzet {
    pub banka_kodu: String,
    pub banka_ad: String,
    pub hesap_id: String,
    pub iban: String,
    pub para_birimi: String,
    pub bakiye: i64, // kuruş
}

#[derive(Serialize)]
pub struct Islem {
    pub islem_no: String,
    pub referans_no: String,
    pub tarih: String, // gg.aa.yyyy
    pub yon: &'static str, // "GIRIS" | "CIKIS"
    pub tutar: i64,    // kuruş, pozitif
    pub aciklama: String,
    pub karsi_unvan: Option<String>,
    pub karsi_iban: Option<String>, // maskeli
    pub kanal: &'static str,
    pub bakiye_sonrasi: i64, // kuruş
}

#[derive(Serialize)]
pub struct HareketlerYanit {
    pub hesap_id: String,
    pub banka_ad: String,
    pub iban: String,
    pub para_birimi: String,
    pub acilis_bakiye: i64,
    pub kapanis_bakiye: i64,
    pub islemler: Vec<Islem>,
}

// ── Deterministik üreteç (LCG — random değil) ────────────────────────────────
fn lcg(s: &mut u64) -> u64 {
    *s = s
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *s
}

fn hesap_id(kod: &str) -> String {
    format!("SIM-{kod}-01")
}

fn iban_uret(idx: usize, kod: &str) -> String {
    // Simüle IBAN — maskeli gösterim için son 4 anlamlı.
    format!("TR{:02} {} 0000 0000 {:04}", 30 + idx, kod, 1000 + idx * 137 % 9000)
}

fn tarih_ekle(mut yil: i32, mut ay: u32, mut gun: u32, ekle: u32) -> (i32, u32, u32) {
    gun += ekle;
    while gun > 28 {
        gun -= 28;
        ay += 1;
        if ay > 12 {
            ay = 1;
            yil += 1;
        }
    }
    (yil, ay, gun)
}

/// Bir hesap için deterministik hareket listesi (2026 başından itibaren).
fn islemler_uret(idx: usize, kod: &str) -> (Vec<Islem>, i64, i64) {
    let mut s = 0x2026_1453u64 ^ ((idx as u64 + 1) * 0x9E37_79B9);
    let adet = 18 + (lcg(&mut s) % 15) as usize; // 18-32 hareket
    let acilis = 20_000_00 + (lcg(&mut s) % 80_000_00) as i64; // 20k-100k TL
    let mut bakiye = acilis;
    let (mut yil, mut ay, mut gun) = (2026i32, 1u32, 1u32 + (idx as u32 % 5));
    let mut islemler = Vec::with_capacity(adet);
    for i in 0..adet {
        let (sablon, giris, taban, karsili) = ISLEM_DESENLERI[(lcg(&mut s) % ISLEM_DESENLERI.len() as u64) as usize];
        let carpan = 50 + (lcg(&mut s) % 250); // %50–%300
        let tutar = taban * carpan as i64 / 100;
        let (y, a, g) = tarih_ekle(yil, ay, gun, 1 + (lcg(&mut s) % 6) as u32);
        yil = y; ay = a; gun = g;
        let unvan = KARSI_UNVANLAR[(lcg(&mut s) % KARSI_UNVANLAR.len() as u64) as usize];
        let aciklama = sablon.replace("{u}", unvan);
        if giris { bakiye += tutar; } else { bakiye -= tutar; }
        islemler.push(Islem {
            islem_no: format!("{kod}-{:05}", i + 1),
            referans_no: format!("REF{:010}", lcg(&mut s) % 10_000_000_000),
            tarih: format!("{:02}.{:02}.{:04}", g, a, y),
            yon: if giris { "GIRIS" } else { "CIKIS" },
            tutar,
            aciklama,
            karsi_unvan: if karsili { Some(unvan.to_string()) } else { None },
            karsi_iban: if karsili {
                Some(format!("TR** **** **** **** {:04}", lcg(&mut s) % 10000))
            } else {
                None
            },
            kanal: if giris { "SUBE/İNTERNET" } else { "İNTERNET" },
            bakiye_sonrasi: bakiye,
        });
    }
    (islemler, acilis, bakiye)
}

// ── Uçlar (izole; State yok) ─────────────────────────────────────────────────

/// GET /api/banka/oys/hesaplar — bağlı 10 bankanın hesap özetleri.
pub async fn hesaplar() -> Json<Vec<HesapOzet>> {
    let liste = BANKALAR
        .iter()
        .enumerate()
        .map(|(idx, (kod, ad))| {
            let (_, _, kapanis) = islemler_uret(idx, kod);
            HesapOzet {
                banka_kodu: kod.to_string(),
                banka_ad: ad.to_string(),
                hesap_id: hesap_id(kod),
                iban: iban_uret(idx, kod),
                para_birimi: "TRY".to_string(),
                bakiye: kapanis,
            }
        })
        .collect();
    Json(liste)
}

/// GET /api/banka/oys/hareketler?hesap=SIM-0064-01[&baslangic=gg.aa.yyyy&bitis=...]
pub async fn hareketler(Query(q): Query<HashMap<String, String>>) -> Json<HareketlerYanit> {
    let hesap = q.get("hesap").cloned().unwrap_or_default();
    let (idx, kod, ad) = BANKALAR
        .iter()
        .enumerate()
        .find(|(_, (kod, _))| hesap_id(kod) == hesap)
        .map(|(idx, (kod, ad))| (idx, *kod, *ad))
        .unwrap_or((0, BANKALAR[0].0, BANKALAR[0].1));

    let (mut islemler, acilis, kapanis) = islemler_uret(idx, kod);
    // Tarih filtresi (gg.aa.yyyy karşılaştırması yyyy-aa-gg'ye çevirerek).
    let anahtar = |t: &str| -> String {
        let p: Vec<&str> = t.split('.').collect();
        if p.len() == 3 { format!("{}{}{}", p[2], p[1], p[0]) } else { t.to_string() }
    };
    if let Some(b) = q.get("baslangic") {
        let bk = anahtar(b);
        islemler.retain(|i| anahtar(&i.tarih) >= bk);
    }
    if let Some(b) = q.get("bitis") {
        let bk = anahtar(b);
        islemler.retain(|i| anahtar(&i.tarih) <= bk);
    }
    Json(HareketlerYanit {
        hesap_id: hesap_id(kod),
        banka_ad: ad.to_string(),
        iban: iban_uret(idx, kod),
        para_birimi: "TRY".to_string(),
        acilis_bakiye: acilis,
        kapanis_bakiye: kapanis,
        islemler,
    })
}
