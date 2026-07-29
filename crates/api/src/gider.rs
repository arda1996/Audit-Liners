//! GİDER MOTORU — hangi gider hangi hesapta izlenir, indirilebilir mi, KKEG mi?
//!
//! Üç soruyu birlikte cevaplar; muhasebecinin en sık takıldığı yer burasıdır:
//!   1. **Hangi hesap?** Sektörün maliyet seçeneğine göre (7/A gider yeri · 7/B gider çeşidi · yok)
//!   2. **İndirilebilir mi?** Bazı giderler kaydedilir ama vergi matrahından düşülemez
//!   3. **KKEG mi?** Öyleyse ne kadarı — tamamı, oransal (%30) veya haddi aşan kısım
//!
//! KKEG'in kritik inceliği: **MSUGT'de ayrı bir "KKEG hesabı" YOKTUR.** Gider normal hesabına
//! yazılır, KKEG kısmı `.99` muavininde izlenir (ör. `770.99`). Beyannamede matraha GERİ EKLENİR;
//! **yevmiyede ters kayıt YAPILMAZ** — defter ticari kârı, beyanname mali kârı gösterir.

use axum::{extract::Query, Json};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const KATALOG: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/gider-katalogu.json"));
const SEKTORLER: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/sektorler.json"));

fn katalog() -> serde_json::Value { serde_json::from_str(KATALOG).unwrap_or(serde_json::Value::Null) }

/// Sektörün maliyet seçeneği: "7A" · "7B" · "yok". Bilinmeyen sektörde "yok".
pub fn maliyet_secenegi(sektor: &str) -> String {
    let s: serde_json::Value = serde_json::from_str(SEKTORLER).unwrap_or(serde_json::Value::Null);
    s["sektorler"].as_array().unwrap_or(&vec![]).iter()
        .find(|x| x["kod"].as_str() == Some(sektor))
        .and_then(|x| x["maliyet_secenegi"].as_str())
        .unwrap_or("yok").to_string()
}

/// Gider türünün, verilen maliyet seçeneğinde izleneceği hesabı verir.
///
/// · **7A** → gider yeri hesabı (710, 760, 770 …)
/// · **7B** → gider çeşidi hesabı (790, 793, 796 …)
/// · **yok** → 7'li hesap hiç kullanılmaz (ticaret / serbest meslek): gider DOĞRUDAN gelir
///   tablosu hesabına yazılır. Karşılığı, 7/A hesabının yansıtma HEDEFİDİR (760→631, 770→632…).
fn hesap_sec(g: &serde_json::Value, secenek: &str) -> String {
    let a = g["a"].as_str().unwrap_or("");
    let b = g["b"].as_str().unwrap_or("");
    match secenek {
        "7A" => if a.is_empty() || a == "-" { b.to_string() } else { a.to_string() },
        "7B" => if b.is_empty() || b == "-" { a.to_string() } else { b.to_string() },
        // Maliyet muhasebesi yok → yansıtma hedefi (6xx). Bulunamazsa 7/A kodu kalır.
        _ => {
            let k = katalog();
            let ilk = a.split(&['/', ' '][..]).next().unwrap_or(a);
            k["yansitma"].as_array().unwrap_or(&vec![]).iter()
                .find(|y| y["gider"].as_str() == Some(ilk))
                .and_then(|y| y["hedef"].as_str())
                .map(String::from)
                .unwrap_or_else(|| if a.is_empty() || a == "-" { b.to_string() } else { a.to_string() })
        }
    }
}

/// Katalogda bir gider birden çok hesaba yazılabiliyorsa ("792,793,794") ilki BİRİNCİL
/// öneridir, gerisi alternatiftir — muhasebeci gider yerine göre seçer.
fn birincil(kod: &str) -> String {
    kod.split(',').next().unwrap_or(kod).trim().to_string()
}
fn alternatifler(kod: &str) -> Vec<String> {
    kod.split(',').skip(1).map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect()
}

/// Üretim maliyeti hesapları (710/720/730) yalnız üretim yapan işletmede anlamlıdır.
fn uretim_hesabi(kod: &str) -> bool {
    matches!(kod, "710" | "720" | "730")
}

fn sektore_uygun(g: &serde_json::Value, sektor: &str) -> bool {
    let s = g["sektorler"].as_array().cloned().unwrap_or_default();
    s.iter().any(|x| x.as_str() == Some("*") || x.as_str() == Some(sektor))
}

/// GET /api/gider/katalog?sektor=URT — sektöre uygun gider türleri + hesapları + KKEG durumu.
pub async fn gider_katalog(Query(q): Query<HashMap<String, String>>) -> Json<serde_json::Value> {
    let sektor = q.get("sektor").cloned().unwrap_or_else(|| "TIC".into());
    let secenek = maliyet_secenegi(&sektor);
    let k = katalog();

    let mut liste: Vec<serde_json::Value> = Vec::new();
    for g in k["gider_turleri"].as_array().unwrap_or(&vec![]) {
        if !sektore_uygun(g, &sektor) { continue; }
        let ham = hesap_sec(g, &secenek);
        let hesap = birincil(&ham);
        liste.push(serde_json::json!({
            "kod": g["kod"], "ad": g["ad"],
            "hesap": hesap,
            "alternatif_hesaplar": alternatifler(&ham),
            "hesap_7a": g["a"], "hesap_7b": g["b"],
            "indirilebilir": g["indirilebilir"],
            "kkeg": g["kkeg"],
            "kkeg_oran": g["kkeg_oran"],
            "dayanak": g["dayanak"],
            "not": g["not"],
            // KKEG kısmı ayrı muavinde izlenir — ana hesap değişmez.
            "kkeg_muavin": if g["kkeg"].is_null() { serde_json::Value::Null }
                else { serde_json::json!(format!("{hesap}.{}", k["kkeg_muavin_eki"].as_str().unwrap_or("99"))) },
        }));
    }

    Json(serde_json::json!({
        "sektor": sektor,
        "maliyet_secenegi": secenek,
        "secenek_bilgi": k["maliyet_secenegi"][&secenek],
        "gider_sayisi": liste.len(),
        "kkeg_etkili": liste.iter().filter(|x| !x["kkeg"].is_null()).count(),
        "giderler": liste,
        "yansitma": k["yansitma"],
        "kkeg_not": k["kkeg_not"],
    }))
}

#[derive(Deserialize)]
pub struct GiderGirdi {
    /// Katalog kodu (ör. "BINEK_YAKIT_BAKIM")
    pub kod: String,
    /// Gider tutarı (kuruş)
    pub tutar: i64,
    pub sektor: String,
    /// Haddi aşan KKEG türlerinde geçerli had (kuruş). Bilinmiyorsa verilmez → uyarı döner.
    #[serde(default)]
    pub had: Option<i64>,
    /// Faaliyeti binek oto kiralama/işletme mi? (GVK 40/1 istisnası)
    #[serde(default)]
    pub arac_isletmecisi: bool,
}

/// POST /api/gider/oner — bir gider için hesap + KKEG ayrımını hesaplar.
///
/// Çıktı, doğrudan fiş satırına dönüşecek biçimdedir: indirilebilir kısım ana hesaba,
/// KKEG kısmı `.99` muavinine. İkisinin toplamı daima gider tutarına eşittir.
pub async fn gider_oner(Json(g): Json<GiderGirdi>) -> Json<serde_json::Value> {
    let k = katalog();
    let Some(t) = k["gider_turleri"].as_array().unwrap_or(&vec![]).iter()
        .find(|x| x["kod"].as_str() == Some(g.kod.as_str())).cloned() else {
        return Json(serde_json::json!({
            "hata": format!("bilinmeyen gider türü: {}", g.kod),
            "gecerli": k["gider_turleri"].as_array().unwrap_or(&vec![]).iter()
                .filter_map(|x| x["kod"].as_str()).collect::<Vec<_>>(),
        }));
    };

    let secenek = maliyet_secenegi(&g.sektor);
    let ham_hesap = hesap_sec(&t, &secenek);
    let hesap = birincil(&ham_hesap);
    let alt = alternatifler(&ham_hesap);
    let mut uyarilar: Vec<String> = Vec::new();
    if !alt.is_empty() {
        uyarilar.push(format!(
            "Bu gider birden çok hesapta izlenebilir; gider yerine göre seç. Birincil: {hesap} · alternatif: {}",
            alt.join(", ")));
    }
    // Maliyet muhasebesi olmayan işletmede üretim hesabı önerilemez.
    if secenek == "yok" && uretim_hesabi(t["a"].as_str().unwrap_or("")) {
        uyarilar.push(format!(
            "{} sektöründe maliyet muhasebesi yok — üretim maliyeti hesabı ({}) kullanılmaz. \
             Ticarette mal alışı 153 Ticari Mallar'a, satılınca 621'e gider.",
            g.sektor, t["a"].as_str().unwrap_or("")));
    }
    if !sektore_uygun(&t, &g.sektor) {
        uyarilar.push(format!(
            "Bu gider türü {} sektöründe olağan değil — katalogda yalnız {:?} için tanımlı.",
            g.sektor, t["sektorler"]));
    }

    // KKEG hesabı — türe göre üç davranış.
    let kkeg_tur = t["kkeg"].as_str().unwrap_or("");
    let (kkeg, indirilebilir) = match kkeg_tur {
        "TAMAMI" => (g.tutar, 0),
        "ORANSAL" => {
            let oran = t["kkeg_oran"].as_i64().unwrap_or(0);
            // GVK 40/1 istisnası: faaliyeti araç kiralama/işletme olanda binek kısıtı UYGULANMAZ.
            if g.arac_isletmecisi && g.kod.starts_with("BINEK") {
                uyarilar.push("Faaliyeti binek oto kiralama/işletme olduğu için kısıt UYGULANMADI (GVK 40/1 parantez içi hüküm).".into());
                (0, g.tutar)
            } else {
                let kk = g.tutar * oran / 100;
                (kk, g.tutar - kk)
            }
        }
        "HADDI_ASAN" => {
            if g.arac_isletmecisi && g.kod.starts_with("BINEK") {
                uyarilar.push("Faaliyeti binek oto kiralama/işletme olduğu için had kısıtı UYGULANMADI (GVK 40/1).".into());
                (0, g.tutar)
            } else if let Some(had) = g.had {
                let kk = (g.tutar - had).max(0);
                (kk, g.tutar - kk)
            } else {
                uyarilar.push("Had verilmedi — KKEG hesaplanamadı. Yıllık tebliğdeki haddi girip yeniden hesapla.".into());
                (0, g.tutar)
            }
        }
        "KISMEN" => {
            uyarilar.push(format!(
                "Bu gider KISMEN KKEG olabilir; oran/tutar olaya göre değişir, motor tahmin YÜRÜTMEZ. Dayanak: {}",
                t["dayanak"].as_str().unwrap_or("")));
            (0, g.tutar)
        }
        _ => (0, g.tutar),
    };

    if t["indirilebilir"].as_bool() == Some(false) && kkeg == 0 && kkeg_tur.is_empty() {
        uyarilar.push("Bu gider vergi matrahından indirilemez.".into());
    }
    if let Some(n) = t["not"].as_str() { if !n.is_empty() { uyarilar.push(n.to_string()); } }

    let kkeg_hesap = format!("{hesap}.{}", k["kkeg_muavin_eki"].as_str().unwrap_or("99"));
    let mut satirlar: Vec<serde_json::Value> = Vec::new();
    if indirilebilir > 0 {
        satirlar.push(serde_json::json!({ "hesap": hesap, "borc": indirilebilir, "aciklama": t["ad"] }));
    }
    if kkeg > 0 {
        satirlar.push(serde_json::json!({ "hesap": kkeg_hesap, "borc": kkeg, "aciklama": format!("{} — KKEG", t["ad"].as_str().unwrap_or("")) }));
    }

    Json(serde_json::json!({
        "kod": g.kod, "ad": t["ad"], "sektor": g.sektor,
        "maliyet_secenegi": secenek,
        "hesap": hesap,
        "alternatif_hesaplar": alt,
        "tutar": g.tutar,
        "indirilebilir": indirilebilir,
        "kkeg": kkeg,
        "kkeg_turu": if kkeg_tur.is_empty() { serde_json::Value::Null } else { serde_json::json!(kkeg_tur) },
        "kkeg_hesap": if kkeg > 0 { serde_json::json!(kkeg_hesap) } else { serde_json::Value::Null },
        "dayanak": t["dayanak"],
        "fis_satirlari": satirlar,
        "uyarilar": uyarilar,
        "not": "KKEG kısmı deftere GİDER olarak yazılır ama beyannamede matraha GERİ EKLENİR — yevmiyede ters kayıt yapılmaz.",
    }))
}

#[cfg(test)]
mod testler {
    use super::*;

    fn oner(kod: &str, tutar: i64, sektor: &str, had: Option<i64>, arac: bool) -> serde_json::Value {
        futures::executor::block_on(gider_oner(Json(GiderGirdi {
            kod: kod.into(), tutar, sektor: sektor.into(), had, arac_isletmecisi: arac,
        }))).0
    }

    /// İndirilebilir + KKEG toplamı DAİMA gider tutarına eşit olmalı — para kaybolmaz.
    #[test]
    fn indirilebilir_arti_kkeg_tutari_verir() {
        for (kod, had) in [("BINEK_YAKIT_BAKIM", None), ("CEZA_GECIKME", None),
                           ("BINEK_KIRA", Some(30_000_00)), ("KIRA", None), ("HAMMADDE", None)] {
            let r = oner(kod, 100_000_00, "TIC", had, false);
            let i = r["indirilebilir"].as_i64().unwrap();
            let k = r["kkeg"].as_i64().unwrap();
            assert_eq!(i + k, 100_000_00, "{kod}: indirilebilir+KKEG tutarı vermiyor");
            // Fiş satırlarının toplamı da tutmalı
            let s: i64 = r["fis_satirlari"].as_array().unwrap().iter()
                .map(|x| x["borc"].as_i64().unwrap()).sum();
            assert_eq!(s, 100_000_00, "{kod}: fiş satırları tutarı vermiyor");
        }
    }

    /// Binek oto gider kısıtı: %70 indirilebilir, %30 KKEG (GVK 40/5).
    #[test]
    fn binek_oto_kisiti_yuzde_otuz_kkeg() {
        let r = oner("BINEK_YAKIT_BAKIM", 10_000_00, "TIC", None, false);
        assert_eq!(r["indirilebilir"], 7_000_00);
        assert_eq!(r["kkeg"], 3_000_00);
        assert_eq!(r["kkeg_turu"], "ORANSAL");
        assert_eq!(r["kkeg_hesap"].as_str().unwrap().split('.').next_back(), Some("99"));
    }

    /// GVK 40/1 İSTİSNASI: faaliyeti araç kiralama/işletme olanda binek kısıtı UYGULANMAZ.
    #[test]
    fn arac_isletmecisinde_binek_kisiti_uygulanmaz() {
        let r = oner("BINEK_YAKIT_BAKIM", 10_000_00, "HIZ", None, true);
        assert_eq!(r["kkeg"], 0, "araç işletmecisinde kısıt uygulanmamalı");
        assert_eq!(r["indirilebilir"], 10_000_00);
        assert!(r["uyarilar"].as_array().unwrap().iter()
            .any(|x| x.as_str().unwrap_or("").contains("GVK 40/1")));
    }

    /// Vergi cezası / gecikme zammı TAMAMEN KKEG'dir (KVK 11/1-d).
    #[test]
    fn ceza_ve_gecikme_tamami_kkeg() {
        let r = oner("CEZA_GECIKME", 5_000_00, "TIC", None, false);
        assert_eq!(r["kkeg"], 5_000_00);
        assert_eq!(r["indirilebilir"], 0);
        assert_eq!(r["kkeg_turu"], "TAMAMI");
    }

    /// Haddi aşan KKEG: had verilmezse motor TAHMİN ETMEZ, uyarır.
    #[test]
    fn had_verilmezse_tahmin_yurutulmez() {
        let r = oner("BINEK_KIRA", 50_000_00, "TIC", None, false);
        assert_eq!(r["kkeg"], 0, "had yokken KKEG uydurulmamalı");
        assert!(r["uyarilar"].as_array().unwrap().iter()
            .any(|x| x.as_str().unwrap_or("").contains("Had verilmedi")));

        // Had verilince doğru hesaplasın: 50.000 kira, 30.000 had → 20.000 KKEG
        let r2 = oner("BINEK_KIRA", 50_000_00, "TIC", Some(30_000_00), false);
        assert_eq!(r2["kkeg"], 20_000_00);
        assert_eq!(r2["indirilebilir"], 30_000_00);
    }

    /// Sektörün maliyet seçeneği hesabı belirler: üretimde 7/A (710), hizmette 7/B (790).
    #[test]
    fn sektor_maliyet_secenegine_gore_hesap_secilir() {
        assert_eq!(maliyet_secenegi("URT"), "7A");
        assert_eq!(maliyet_secenegi("HIZ"), "7B");
        assert_eq!(maliyet_secenegi("TIC"), "yok");

        let uretim = oner("HAMMADDE", 1_000_00, "URT", None, false);
        assert_eq!(uretim["hesap"], "710", "üretimde 7/A gider yeri hesabı");
        let hizmet = oner("HAMMADDE", 1_000_00, "GID", None, false);
        assert_eq!(hizmet["hesap"], "790", "7/B'de gider çeşidi hesabı");

        // Maliyet muhasebesi YOK olan ticarette gider doğrudan 6xx'e gider — 7'li kullanılmaz.
        let ticaret = oner("GENEL_YONETIM", 1_000_00, "TIC", None, false);
        assert_eq!(ticaret["hesap"], "632", "ticarette genel yönetim doğrudan 632'ye");
        let pazarlama = oner("PAZARLAMA", 1_000_00, "TIC", None, false);
        assert_eq!(pazarlama["hesap"], "631", "ticarette pazarlama doğrudan 631'e");
    }

    /// Maliyet muhasebesi olmayan sektörde ÜRETİM hesabı önerilmemeli, uyarı verilmeli.
    #[test]
    fn maliyetsiz_sektorde_uretim_hesabi_uyarir() {
        let r = oner("HAMMADDE", 1_000_00, "TIC", None, false);
        assert!(r["uyarilar"].as_array().unwrap().iter()
            .any(|x| x.as_str().unwrap_or("").contains("maliyet muhasebesi yok")),
            "üretim hesabı uyarısı yok: {:?}", r["uyarilar"]);
    }

    /// Sektöre uygun olmayan gider türü UYARI vermeli (ör. e-ticarette inşaat maliyeti).
    #[test]
    fn sektor_disi_gider_uyari_verir() {
        let r = oner("INSAAT_MALIYET", 100_00, "ETIC", None, false);
        assert!(r["uyarilar"].as_array().unwrap().iter()
            .any(|x| x.as_str().unwrap_or("").contains("olağan değil")),
            "sektör dışı gider uyarısı yok: {:?}", r["uyarilar"]);
    }

    /// Katalog sektöre göre süzülmeli ve yansıtma kuralları eksiksiz gelmeli.
    #[test]
    fn katalog_sektore_gore_suzulur() {
        let mut q = HashMap::new(); q.insert("sektor".to_string(), "ETIC".to_string());
        let r = futures::executor::block_on(gider_katalog(Query(q))).0;
        assert_eq!(r["maliyet_secenegi"], "yok");
        let g = r["giderler"].as_array().unwrap();
        assert!(!g.is_empty());
        // E-ticarete özgü kalem gelmeli, inşaat maliyeti GELMEMELİ
        assert!(g.iter().any(|x| x["kod"] == "PLATFORM_KOMISYON"), "e-ticarette platform komisyonu olmalı");
        assert!(!g.iter().any(|x| x["kod"] == "INSAAT_MALIYET"), "e-ticarette inşaat maliyeti olmamalı");
        assert!(r["yansitma"].as_array().unwrap().len() >= 8, "yansıtma kuralları eksik");
    }
}
