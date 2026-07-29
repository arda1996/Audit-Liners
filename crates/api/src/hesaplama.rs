//! HESAP MAKİNESİ — muhasebecinin elle hesapladığı bağlı değişkenleri otomatik üretir.
//!
//! İlke: **hiçbir oran koda gömülü değildir.** Hepsi `data/vergi-parametreleri.json` içindeki
//! `hesaplama` bölümünden okunur; oran değişince yalnız o dosya güncellenir (proaktif değişiklik).
//!
//! Kapsam yalnız **%100 kesin** olan türev hesaplar:
//!   · KDV        — matrah↔KDV↔brüt (üç yönlü; brütten iç yüzde ile matrah ayrıştırma)
//!   · Tevkifat   — seçilen oranla KDV'nin bölünmesi (KDVK 9)
//!   · Stopaj     — brüt↔net (GVK 94; net üzerinden brüte tamamlama)
//!   · Damga      — nispi binde hesabı (488)
//!   · BSMV       — banka masrafı üzerinden (6802)
//!
//! Kapsam DIŞI (bilerek): hangi işlemde hangi tevkifat oranı uygulanacağı, ücret brüt↔net
//! (SGK + kümülatif GV tarifesi gerektirir), gecikme zammı (oran `dogrulandi:false`).
//! Bunlar "%100 doğru" sayılamayacağı için motor tahmin yürütmez.

use axum::{extract::Query, Json};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const PARAM_JSON: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/vergi-parametreleri.json"));

fn param() -> serde_json::Value {
    serde_json::from_str(PARAM_JSON).unwrap_or(serde_json::Value::Null)
}

/// gg.aa.yyyy → yyyyaagg (karşılaştırılabilir anahtar). Bozuk tarihte boş döner.
fn tkey(t: &str) -> String {
    let p: Vec<&str> = t.split('.').collect();
    if p.len() == 3 && p[0].len() == 2 && p[1].len() == 2 && p[2].len() == 4 {
        format!("{}{}{}", p[2], p[1], p[0])
    } else {
        String::new()
    }
}

#[derive(Serialize, Clone)]
pub struct OranBilgi {
    pub oran: i64,
    pub ad: String,
    pub muavin: String,
    pub gecerli: bool,
    pub tarihi: bool,
    pub not: Option<String>,
}

/// Parametre dosyasındaki KDV oranlarını, verilen tarihte geçerli olup olmadıklarıyla döndürür.
fn kdv_oranlari(tarih: Option<&str>) -> Vec<OranBilgi> {
    let p = param();
    let bos = vec![];
    let liste = p["hesaplama"]["kdv_oranlari"].as_array().unwrap_or(&bos).clone();
    let anahtar = tarih.map(tkey).unwrap_or_default();
    liste
        .iter()
        .map(|o| {
            let bas = o["gecerli_baslangic"].as_str().map(tkey).unwrap_or_default();
            let bit = o["gecerli_bitis"].as_str().map(tkey);
            // Tarih verilmediyse "bugün geçerli mi" sorusuna bitiş alanından cevap veriyoruz.
            let gecerli = if anahtar.is_empty() {
                bit.is_none()
            } else {
                anahtar >= bas && bit.as_ref().map(|b| &anahtar <= b).unwrap_or(true)
            };
            OranBilgi {
                oran: o["oran"].as_i64().unwrap_or(0),
                ad: o["ad"].as_str().unwrap_or("").to_string(),
                muavin: o["muavin"].as_str().unwrap_or("").to_string(),
                gecerli,
                tarihi: bit.is_some(),
                not: o["not"].as_str().map(String::from),
            }
        })
        .collect()
}

/// GET /api/hesapla/oranlar?tarih=gg.aa.yyyy — oran kataloğu (tarihe göre geçerlilik işaretli).
pub async fn oranlar(Query(q): Query<HashMap<String, String>>) -> Json<serde_json::Value> {
    let t = q.get("tarih").map(|s| s.as_str());
    let p = param();
    Json(serde_json::json!({
        "kdv": kdv_oranlari(t),
        "tevkifat": p["hesaplama"]["tevkifat_oranlari"],
        "tevkifat_notu": p["hesaplama"]["tevkifat_notu"],
        "stopaj": p["hesaplama"]["stopaj"],
        "damga": p["hesaplama"]["damga"],
        "bsmv": p["hesaplama"]["bsmv"],
        "kaynak": "data/vergi-parametreleri.json — oranlar koda gömülü değildir",
    }))
}

#[derive(Deserialize)]
pub struct KdvGirdi {
    /// KDV hariç tutar (kuruş). `brut` ile birlikte verilemez.
    pub matrah: Option<i64>,
    /// KDV dahil tutar (kuruş) — iç yüzde ile matrah ayrıştırılır.
    pub brut: Option<i64>,
    pub oran: i64,
    /// Fiş tarihi (gg.aa.yyyy) — oranın o tarihte geçerli olup olmadığı denetlenir.
    pub tarih: Option<String>,
    /// Tevkifat oranı payı (ör. 5 → 5/10). Verilmezse tevkifat yok.
    pub tevkifat_pay: Option<i64>,
    pub tevkifat_payda: Option<i64>,
}

/// POST /api/hesapla/kdv — matrah↔KDV↔brüt üçlüsünü tamamlar, muavin kodunu ve uyarıları verir.
///
/// Yuvarlama: kuruş tabanlı tam sayı aritmetiği; brütten ayrıştırmada matrah = brüt*100/(100+oran)
/// ve KDV = brüt − matrah — böylece matrah+KDV **her zaman** brüte eşit kalır (yuvarlama kaçağı olmaz).
pub async fn kdv(Json(g): Json<KdvGirdi>) -> Json<serde_json::Value> {
    let mut uyarilar: Vec<String> = Vec::new();

    // Oranın o tarihte geçerliliği — tarihî oran (%8/%18) bugünkü fişte kullanılamaz.
    let katalog = kdv_oranlari(g.tarih.as_deref());
    let bilgi = katalog.iter().find(|o| o.oran == g.oran);
    match bilgi {
        None => uyarilar.push(format!(
            "%{} oranı katalogda yok — geçerli oranlar: {}",
            g.oran,
            katalog.iter().map(|o| format!("%{}", o.oran)).collect::<Vec<_>>().join(", ")
        )),
        Some(o) if !o.gecerli => uyarilar.push(format!(
            "%{} bu tarihte YÜRÜRLÜKTE DEĞİL — {}{}",
            g.oran,
            o.ad,
            o.not.as_deref().map(|n| format!(" · {n}")).unwrap_or_default()
        )),
        _ => {}
    }

    let (matrah, kdv_tutar) = match (g.matrah, g.brut) {
        (Some(m), None) => (m, m * g.oran / 100),
        (None, Some(b)) => {
            // İç yüzde: brütün içinden matrahı ayrıştır. Fark KDV'ye yazılır ki toplam bozulmasın.
            let m = b * 100 / (100 + g.oran);
            (m, b - m)
        }
        (Some(_), Some(_)) => {
            uyarilar.push("matrah ve brüt birlikte verilemez — hangisinden hesaplanacağı belirsiz".into());
            (0, 0)
        }
        (None, None) => {
            uyarilar.push("matrah veya brüt verilmeli".into());
            (0, 0)
        }
    };
    let brut = matrah + kdv_tutar;

    // Tevkifat (KDVK 9): KDV'nin bir kısmını alıcı sorumlu sıfatıyla beyan eder.
    // Satıcıya ödenen tutar bu kadar azalır; 391'e yalnız tevkifat dışı pay yazılır.
    let (tevkifat, satici_kdv) = match (g.tevkifat_pay, g.tevkifat_payda) {
        (Some(pay), Some(payda)) if payda > 0 && pay > 0 && pay < payda => {
            let t = kdv_tutar * pay / payda;
            (t, kdv_tutar - t)
        }
        (Some(_), _) | (_, Some(_)) => {
            uyarilar.push("tevkifat oranı geçersiz — pay ve payda birlikte, 0<pay<payda olmalı".into());
            (0, kdv_tutar)
        }
        _ => (0, kdv_tutar),
    };

    if g.oran == 0 {
        uyarilar.push("KDV sıfır — e-belgede istisna sebep kodu ZORUNLU (UBL-TR TaxExemptionReason)".into());
    }

    let muavin = bilgi.map(|o| o.muavin.clone()).unwrap_or_default();
    Json(serde_json::json!({
        "matrah": matrah,
        "kdv": kdv_tutar,
        "brut": brut,
        "tevkifat": tevkifat,
        "satici_kdv": satici_kdv,           // 391'e yazılacak pay
        "odenecek_tutar": brut - tevkifat,  // cariye borç yazılacak tutar
        "muavin_391": if muavin.is_empty() { None } else { Some(format!("391.{muavin}")) },
        "muavin_191": if muavin.is_empty() { None } else { Some(format!("191.{muavin}")) },
        "uyarilar": uyarilar,
    }))
}

#[derive(Deserialize)]
pub struct StopajGirdi {
    /// Brütten hesapla (stopaj = brüt × oran).
    pub brut: Option<i64>,
    /// Netten brüte tamamla (brüt = net / (1 − oran)) — kira sözleşmeleri genelde net konuşulur.
    pub net: Option<i64>,
    /// "kira" | "serbest_meslek"
    pub tur: String,
}

/// POST /api/hesapla/stopaj — GVK 94 stopajı; brüt↔net iki yönlü.
pub async fn stopaj(Json(g): Json<StopajGirdi>) -> Json<serde_json::Value> {
    let p = param();
    let bos = vec![];
    let liste = p["hesaplama"]["stopaj"].as_array().unwrap_or(&bos).clone();
    let Some(t) = liste.iter().find(|x| x["kod"].as_str() == Some(g.tur.as_str())) else {
        return Json(serde_json::json!({
            "hata": format!("bilinmeyen stopaj türü: {}", g.tur),
            "gecerli": liste.iter().filter_map(|x| x["kod"].as_str()).collect::<Vec<_>>(),
        }));
    };
    let oran = t["oran"].as_i64().unwrap_or(0);

    let (brut, kesinti) = match (g.brut, g.net) {
        (Some(b), None) => (b, b * oran / 100),
        // Net üzerinden brüte tamamlama: brüt = net × 100 / (100 − oran)
        (None, Some(n)) if oran < 100 => {
            let b = n * 100 / (100 - oran);
            (b, b - n)
        }
        _ => return Json(serde_json::json!({ "hata": "brut veya net'ten yalnız biri verilmeli" })),
    };
    Json(serde_json::json!({
        "tur": g.tur, "oran": oran,
        "brut": brut, "stopaj": kesinti, "net": brut - kesinti,
        "hesap": t["hesap"],
        "not": "Stopaj 360 Ödenecek Vergi ve Fonlar'a alacak yazılır; muhtasar beyanla beyan edilir.",
    }))
}

#[derive(Deserialize)]
pub struct DamgaGirdi { pub tutar: i64, pub tur: String }

/// POST /api/hesapla/damga — nispi damga vergisi (488 sayılı Kanun).
pub async fn damga(Json(g): Json<DamgaGirdi>) -> Json<serde_json::Value> {
    let p = param();
    let bos = vec![];
    let liste = p["hesaplama"]["damga"].as_array().unwrap_or(&bos).clone();
    let Some(t) = liste.iter().find(|x| x["kod"].as_str() == Some(g.tur.as_str())) else {
        return Json(serde_json::json!({
            "hata": format!("bilinmeyen damga türü: {}", g.tur),
            "gecerli": liste.iter().filter_map(|x| x["kod"].as_str()).collect::<Vec<_>>(),
        }));
    };
    let binde = t["binde"].as_f64().unwrap_or(0.0);
    // binde → kuruş: tutar * binde / 1000, en yakın kuruşa yuvarla
    let damga = ((g.tutar as f64) * binde / 1000.0).round() as i64;
    Json(serde_json::json!({
        "tur": g.tur, "binde": binde, "matrah": g.tutar, "damga": damga,
        "not": "Nispi damga vergisi. Azami had ve maktu tutarlar bu motorda YOK — büyük tutarlı sözleşmelerde tebliğden teyit edilmeli.",
    }))
}

#[cfg(test)]
mod testler {
    use super::*;

    fn kdv_hesapla(matrah: Option<i64>, brut: Option<i64>, oran: i64, tarih: &str) -> serde_json::Value {
        let g = KdvGirdi { matrah, brut, oran, tarih: Some(tarih.into()), tevkifat_pay: None, tevkifat_payda: None };
        futures::executor::block_on(kdv(Json(g))).0
    }

    /// Matrahtan KDV: 10.000 × %20 = 2.000 → brüt 12.000
    #[test]
    fn matrahtan_kdv() {
        let r = kdv_hesapla(Some(10_000_00), None, 20, "01.06.2026");
        assert_eq!(r["matrah"], 10_000_00);
        assert_eq!(r["kdv"], 2_000_00);
        assert_eq!(r["brut"], 12_000_00);
        // Muavin kodu ARDIŞIKTIR — oranın kendisi değil. Kod↔oran gidiş-dönüşü tutmalı.
        let kod = r["muavin_391"].as_str().unwrap();
        assert_eq!(muavin_orani(kod), Some(20), "{kod} → %20 çözülemedi");
        assert_eq!(kod, format!("391.{}", muavin_kodu(20)));
    }

    /// Brütten iç yüzde: matrah + KDV DAİMA brüte eşit olmalı (yuvarlama kaçağı olmamalı).
    #[test]
    fn brutten_ayristirma_kacak_vermez() {
        for brut in [12_000_00i64, 1_00, 33_33, 99_999_99, 7_777_77] {
            for oran in [1, 10, 20] {
                let r = kdv_hesapla(None, Some(brut), oran, "01.06.2026");
                let m = r["matrah"].as_i64().unwrap();
                let k = r["kdv"].as_i64().unwrap();
                assert_eq!(m + k, brut, "brüt {brut} · %{oran} — matrah+KDV brüte eşit değil");
            }
        }
    }

    /// TARİHÎ ORAN: %18 bugünkü fişte uyarı vermeli (10.07.2023'te %20 oldu).
    #[test]
    fn tarihi_oran_bugun_uyari_verir() {
        let r = kdv_hesapla(Some(10_000_00), None, 18, "01.06.2026");
        let u = r["uyarilar"].as_array().unwrap();
        assert!(u.iter().any(|x| x.as_str().unwrap_or("").contains("YÜRÜRLÜKTE DEĞİL")),
            "%18 için 2026 tarihinde uyarı bekleniyordu: {u:?}");
    }

    /// Aynı %18, ESKİ dönemde uyarı VERMEMELİ — düzeltme beyannamesi için gerekli.
    #[test]
    fn tarihi_oran_eski_donemde_gecerli() {
        let r = kdv_hesapla(Some(10_000_00), None, 18, "01.06.2023");
        let u = r["uyarilar"].as_array().unwrap();
        assert!(!u.iter().any(|x| x.as_str().unwrap_or("").contains("YÜRÜRLÜKTE DEĞİL")),
            "%18 09.07.2023 öncesinde geçerliydi, uyarı verilmemeli: {u:?}");
        assert_eq!(r["kdv"], 1_800_00);
    }

    /// %20 de 2023 öncesinde YOKTU — geriye dönük yanlış oran da yakalanmalı.
    #[test]
    fn yeni_oran_eski_donemde_uyari_verir() {
        let r = kdv_hesapla(Some(10_000_00), None, 20, "01.06.2023");
        let u = r["uyarilar"].as_array().unwrap();
        assert!(u.iter().any(|x| x.as_str().unwrap_or("").contains("YÜRÜRLÜKTE DEĞİL")),
            "%20 10.07.2023 öncesinde yoktu, uyarı bekleniyordu: {u:?}");
    }

    /// KDV=0 istisna sebep kodu uyarısı vermeli (UBL-TR zorunlu alan).
    #[test]
    fn istisnada_sebep_kodu_uyarisi() {
        let r = kdv_hesapla(Some(10_000_00), None, 0, "01.06.2026");
        assert_eq!(r["kdv"], 0);
        let u = r["uyarilar"].as_array().unwrap();
        assert!(u.iter().any(|x| x.as_str().unwrap_or("").contains("istisna sebep kodu")));
    }

    /// Tevkifat 5/10: KDV ikiye bölünür, ödenecek tutar tevkifat kadar azalır.
    #[test]
    fn tevkifat_kdvyi_boler() {
        let g = KdvGirdi { matrah: Some(10_000_00), brut: None, oran: 20, tarih: Some("01.06.2026".into()),
                           tevkifat_pay: Some(5), tevkifat_payda: Some(10) };
        let r = futures::executor::block_on(kdv(Json(g))).0;
        assert_eq!(r["kdv"], 2_000_00);
        assert_eq!(r["tevkifat"], 1_000_00);
        assert_eq!(r["satici_kdv"], 1_000_00);        // 391'e yalnız bu yazılır
        assert_eq!(r["odenecek_tutar"], 11_000_00);   // cariye borç: 12.000 − 1.000
    }

    /// Stopaj netten brüte: net 8.000, %20 → brüt 10.000, stopaj 2.000.
    #[test]
    fn stopaj_netten_brute_tamamlar() {
        let g = StopajGirdi { brut: None, net: Some(8_000_00), tur: "kira".into() };
        let r = futures::executor::block_on(stopaj(Json(g))).0;
        assert_eq!(r["brut"], 10_000_00);
        assert_eq!(r["stopaj"], 2_000_00);
        assert_eq!(r["net"], 8_000_00);
    }

    /// Stopaj brütten: 10.000 × %20 = 2.000.
    #[test]
    fn stopaj_brutten_keser() {
        let g = StopajGirdi { brut: Some(10_000_00), net: None, tur: "serbest_meslek".into() };
        let r = futures::executor::block_on(stopaj(Json(g))).0;
        assert_eq!(r["stopaj"], 2_000_00);
        assert_eq!(r["net"], 8_000_00);
    }

    /// Damga: 100.000 TL sözleşme × binde 9,48 = 948 TL.
    #[test]
    fn damga_nispi_hesap() {
        let g = DamgaGirdi { tutar: 100_000_00, tur: "sozlesme".into() };
        let r = futures::executor::block_on(damga(Json(g))).0;
        assert_eq!(r["damga"], 948_00);
    }

    /// Muavin kodları ARDIŞIK olmalı (01,02,03…) ve her kod tek bir orana çözülmeli.
    #[test]
    fn muavin_kodlari_ardisik_ve_tekil() {
        let k = kdv_oranlari(None);
        let kodlar: Vec<String> = k.iter().map(|o| o.muavin.clone()).collect();
        let beklenen: Vec<String> = (1..=k.len()).map(|i| format!("{i:02}")).collect();
        assert_eq!(kodlar, beklenen, "muavin kodları ardışık değil (TDHP alt kırılım kuralı)");
        for o in &k {
            assert_eq!(muavin_orani(&o.muavin), Some(o.oran), "{} kodu {} oranına çözülmüyor", o.muavin, o.oran);
            assert_eq!(muavin_kodu(o.oran), o.muavin, "%{} oranı {} koduna çözülmüyor", o.oran, o.muavin);
        }
    }

    /// Oran kataloğu parametre dosyasından gelmeli ve bugün için doğru geçerlilik vermeli.
    #[test]
    fn katalog_gecerlilik_tarihe_gore() {
        let bugun = kdv_oranlari(Some("01.06.2026"));
        let g: Vec<i64> = bugun.iter().filter(|o| o.gecerli).map(|o| o.oran).collect();
        assert!(g.contains(&20) && g.contains(&10) && g.contains(&1), "2026'da %1/%10/%20 geçerli olmalı: {g:?}");
        assert!(!g.contains(&18) && !g.contains(&8), "2026'da %8/%18 geçerli olmamalı: {g:?}");

        let eski = kdv_oranlari(Some("01.06.2023"));
        let ge: Vec<i64> = eski.iter().filter(|o| o.gecerli).map(|o| o.oran).collect();
        assert!(ge.contains(&18) && ge.contains(&8), "2023 Haziran'da %8/%18 geçerliydi: {ge:?}");
    }
}

/// KDV muavin kodları — (alt_kod, oran). Defter ve simülasyon buradan okur; koda gömülmez.
pub fn kdv_muavin_kodlari() -> Vec<(String, i64)> {
    kdv_oranlari(None).into_iter().map(|o| (o.muavin, o.oran)).collect()
}

// ─────────────────────────────────────────────────────────────────────────────
// CARİ BAZLI ALIM/SATIM MUTABAKATI (tarihî adıyla Form Ba / Form Bs)
//
// ⚠ Ba/Bs BİLDİRİMİ YÜRÜRLÜKTE DEĞİLDİR: 565 Sıra No.lu VUK Genel Tebliği ile **25.09.2024**
// tarihinden itibaren kaldırıldı. GİB alım/satım verisine e-Fatura/e-Arşiv üzerinden doğrudan
// eriştiği için ayrı bildirime gerek kalmadı. Bu uç bir BEYANNAME ÜRETMEZ.
//
// Yine de motor korunuyor, çünkü ürettiği veri hâlâ üç işe yarar:
//   1) İÇ MUTABAKAT — cari bazlı aylık alım/satım toplamı, cari ekstresiyle karşılaştırılır.
//   2) ÇAPRAZ KONTROL — bizim satışımız karşı tarafın alışıyla tutmalı; tutmuyorsa sahte veya
//      muhteviyatı itibariyle yanıltıcı belge riski (vergi denetim modülünün temeli).
//   3) DÜZELTME — 25.09.2024 öncesi dönemler için düzeltme bildirimi hâlâ gerekebilir.
//
// Bildirim KDV HARİÇ tutar üzerinden hesaplanır ve karşı tarafın VKN'siyle eşleşir.

use axum::extract::State;

#[derive(Serialize)]
pub struct BabsSatir {
    pub vkn: String,
    pub unvan: String,
    pub belge_sayisi: usize,
    /// KDV HARİÇ toplam (kuruş)
    pub tutar: i64,
    pub hadde_girdi: bool,
}

/// GET /api/beyanname/babs?ay=6 — cari bazlı alım/satım mutabakatı (tarihî Ba/Bs biçiminde).
/// Bir BEYANNAME DEĞİLDİR — Ba/Bs bildirimi 25.09.2024'te kaldırıldı (565 SN VUK GT).
///
/// Kapsam dışı bırakılanlar (bilerek): hükümsüz belge (TASLAK/IPTAL/RED) bildirime girmez —
/// hüküm doğurmayan fatura alım/satım sayılmaz. İade ve iskonto tutarı düşülür.
pub async fn babs(
    State(state): State<crate::Shared>,
    Query(q): Query<HashMap<String, String>>,
) -> Json<serde_json::Value> {
    let ay: u32 = q.get("ay").and_then(|s| s.parse().ok()).unwrap_or(12).clamp(1, 12);
    let p = param();
    let had = p["hesaplama"]["babs"]["had"].as_i64().unwrap_or(500000);

    let d = { let st = state.lock().unwrap(); crate::gorunum(&st) };
    // (yon, unvan) → (belge sayısı, KDV hariç net tutar)
    let mut kova: HashMap<(String, String), (usize, i64)> = HashMap::new();
    for f in d.faturalar.iter().filter(|f| f.gecerli) {
        let f_ay: u32 = f.tarih[3..5].parse().unwrap_or(0);
        if f_ay != ay { continue; }
        // Bildirim KDV HARİÇ tutar üzerinden; iade ve iskonto matrahı düşürür.
        let net_matrah = f.matrah - f.iade_matrah - f.iskonto_matrah;
        if net_matrah <= 0 { continue; }
        let e = kova.entry((f.yon.clone(), f.karsi.clone())).or_insert((0, 0));
        e.0 += 1;
        e.1 += net_matrah;
    }

    let mut ba: Vec<BabsSatir> = Vec::new(); // ALIŞ
    let mut bs: Vec<BabsSatir> = Vec::new(); // SATIŞ
    for ((yon, unvan), (adet, tutar)) in kova {
        let satir = BabsSatir {
            vkn: crate::simulasyon::vkn(&unvan),
            unvan, belge_sayisi: adet, tutar, hadde_girdi: tutar >= had,
        };
        if yon == "ALIS" { ba.push(satir) } else { bs.push(satir) }
    }
    let duzenle = |v: &mut Vec<BabsSatir>| v.sort_by(|a, b| b.tutar.cmp(&a.tutar));
    duzenle(&mut ba); duzenle(&mut bs);

    // VKN'si olmayan cari bildirime GİREMEZ — bu bir veri eksikliği bulgusudur.
    let vknsiz: Vec<&str> = ba.iter().chain(bs.iter())
        .filter(|x| x.hadde_girdi && x.vkn.is_empty()).map(|x| x.unvan.as_str()).collect();

    let toplam = |v: &Vec<BabsSatir>| -> (usize, i64) {
        v.iter().filter(|x| x.hadde_girdi).fold((0, 0), |(n, t), x| (n + 1, t + x.tutar))
    };
    let (ba_n, ba_t) = toplam(&ba);
    let (bs_n, bs_t) = toplam(&bs);

    Json(serde_json::json!({
        "ay": ay,
        "donem": format!("{ay:02}/2026"),
        "had": had,
        "yururlukte": p["hesaplama"]["babs"]["yururlukte"],
        "kaldirilma_tarihi": p["hesaplama"]["babs"]["kaldirilma_tarihi"],
        "dayanak": p["hesaplama"]["babs"]["dayanak"],
        "had_notu": p["hesaplama"]["babs"]["not"],
        "ba": { "aciklama": "Mal ve hizmet ALIMLARI", "bildirilecek_cari": ba_n, "toplam": ba_t, "satirlar": ba },
        "bs": { "aciklama": "Mal ve hizmet SATIŞLARI", "bildirilecek_cari": bs_n, "toplam": bs_t, "satirlar": bs },
        "vkn_eksik": vknsiz,
        "notlar": [
            "Tutarlar KDV HARİÇ; iade ve iskonto düşülmüştür.",
            "Hükümsüz belge (taslak/iptal/red) bildirime girmez — alım/satım gerçekleşmemiştir.",
            "⚠ Ba/Bs BİLDİRİMİ KALDIRILDI (565 SN VUK GT, 25.09.2024) — bu bir beyanname değil, iç mutabakat raporudur.",
            "Satış toplamı karşı tarafın alış kaydıyla tutmalıdır; tutmuyorsa belge riski incelenir (çapraz kontrol).",
        ],
    }))
}

/// Orana karşılık gelen ARDIŞIK muavin kodunu verir (20 → "06"). Bulunamazsa boş.
pub fn muavin_kodu(oran: i64) -> String {
    kdv_oranlari(None).into_iter().find(|o| o.oran == oran).map(|o| o.muavin).unwrap_or_default()
}

/// Muavin kodundan oranı ÇÖZER ("06" → 20). Kod ardışık olduğu için oran koddan okunamaz;
/// katalogdan çözülür. Tam hesap kodu da verilebilir ("391.06").
pub fn muavin_orani(kod: &str) -> Option<i64> {
    let alt = kod.rsplit('.').next().unwrap_or(kod);
    kdv_oranlari(None).into_iter().find(|o| o.muavin == alt).map(|o| o.oran)
}

#[derive(Deserialize)]
pub struct OtvGirdi {
    /// ÖTV hariç satış bedeli (kuruş) — ÖTV matrahı.
    pub bedel: i64,
    /// ÖTV oranı (%). Nispi ÖTV için. Maktu ÖTV'de 0 verilip `maktu` kullanılır.
    #[serde(default)]
    pub otv_oran: i64,
    /// Maktu ÖTV (kuruş) — I. liste (litre/kg başına) gibi birim vergilerde.
    #[serde(default)]
    pub otv_maktu: i64,
    pub kdv_oran: i64,
}

/// POST /api/hesapla/otv — ÖTV → KDV zinciri.
///
/// **KRİTİK KURAL: ÖTV, KDV MATRAHINA DAHİLDİR** (ÖTVK 11, KDVK 24/b).
/// Yani KDV, ÖTV'nin de üzerinden hesaplanır — vergi üstünden vergi. Sıra bozulursa
/// KDV eksik hesaplanır ve beyan yanlış olur.
///
/// bedel → ÖTV → **KDV matrahı = bedel + ÖTV** → KDV → toplam
pub async fn otv(Json(g): Json<OtvGirdi>) -> Json<serde_json::Value> {
    let mut uyarilar: Vec<String> = Vec::new();
    let otv = if g.otv_maktu > 0 { g.otv_maktu } else { g.bedel * g.otv_oran / 100 };
    // ÖTV KDV matrahına girer — zincirin can alıcı noktası.
    let kdv_matrah = g.bedel + otv;
    let kdv = kdv_matrah * g.kdv_oran / 100;
    let toplam = kdv_matrah + kdv;

    if g.otv_maktu > 0 && g.otv_oran > 0 {
        uyarilar.push("Hem maktu hem nispi ÖTV verildi — maktu esas alındı. Karma vergilemede (III. liste) ikisi ayrı hesaplanır.".into());
    }
    if otv > 0 {
        uyarilar.push("ÖTV, KDV matrahına DAHİL edildi (ÖTVK 11 · KDVK 24/b) — KDV, bedel+ÖTV üzerinden hesaplandı.".into());
    }
    let mk = muavin_kodu(g.kdv_oran);
    let muavin_191 = if mk.is_empty() { None } else { Some(format!("191.{mk}")) };
    Json(serde_json::json!({
        "bedel": g.bedel,
        "otv": otv,
        "kdv_matrah": kdv_matrah,
        "kdv": kdv,
        "toplam": toplam,
        "muavin_191": muavin_191,
        "uyarilar": uyarilar,
        "not": "ÖTV alışta MALİYETE girer (VUK 262) — ayrı indirilecek vergi değildir. İndirilemeyen KDV varsa (KDVK 30) gider/maliyete yazılır ve KKEG olabilir.",
    }))
}

#[cfg(test)]
mod otv_testleri {
    use super::*;
    fn hesapla(bedel: i64, otv_oran: i64, kdv_oran: i64) -> serde_json::Value {
        futures::executor::block_on(otv(Json(OtvGirdi { bedel, otv_oran, otv_maktu: 0, kdv_oran }))).0
    }

    /// ÖTV, KDV MATRAHINA DAHİLDİR (ÖTVK 11 · KDVK 24/b) — KDV bedel+ÖTV üzerinden hesaplanır.
    /// Yaygın hata: KDV'yi yalnız bedel üzerinden hesaplamak. Bu test onu engeller.
    #[test]
    fn otv_kdv_matrahina_dahildir() {
        // 100.000 bedel · %50 ÖTV · %20 KDV
        let r = hesapla(100_000_00, 50, 20);
        assert_eq!(r["otv"], 50_000_00);
        assert_eq!(r["kdv_matrah"], 150_000_00, "KDV matrahı bedel+ÖTV olmalı");
        assert_eq!(r["kdv"], 30_000_00, "KDV, ÖTV dahil matrah üzerinden hesaplanmalı (yalnız bedelden değil)");
        assert_eq!(r["toplam"], 180_000_00);
        // Yalnız bedelden hesaplansaydı KDV 20.000 olurdu — 10.000 eksik beyan.
        assert_ne!(r["kdv"], 20_000_00);
    }

    /// Maktu ÖTV (I. liste — litre/kg başına) de aynı zincire girer.
    #[test]
    fn maktu_otv_de_kdv_matrahina_girer() {
        let r = futures::executor::block_on(otv(Json(OtvGirdi {
            bedel: 10_000_00, otv_oran: 0, otv_maktu: 7_500_00, kdv_oran: 20 }))).0;
        assert_eq!(r["otv"], 7_500_00);
        assert_eq!(r["kdv_matrah"], 17_500_00);
        assert_eq!(r["kdv"], 3_500_00);
    }

    /// ÖTV yoksa zincir bozulmamalı — normal KDV hesabı gibi davranmalı.
    #[test]
    fn otvsiz_islemde_zincir_bozulmaz() {
        let r = hesapla(10_000_00, 0, 20);
        assert_eq!(r["otv"], 0);
        assert_eq!(r["kdv_matrah"], 10_000_00);
        assert_eq!(r["kdv"], 2_000_00);
    }
}
