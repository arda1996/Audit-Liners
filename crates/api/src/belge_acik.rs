//! Belgeler sayfası backend — GİB'den gelen/giden faturalar (SİMÜLASYON: data/ornek-edonusum.json)
//! + indirilebilir **UBL-TR** fatura XML'i + ilgili ödemenin **İŞARETLİ** olduğu ekstre (CSV).
//! İzole modül: AppState'e bağımsız (Banka gibi). Gerçek entegratör/banka gelince aynı uçlar dolar.

use axum::{extract::Path, http::header::{HeaderName, CONTENT_DISPOSITION, CONTENT_TYPE}, Json};
use serde_json::Value;

const EDONUSUM: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/ornek-edonusum.json"));

fn veri() -> Value {
    serde_json::from_str(EDONUSUM).unwrap_or(Value::Null)
}

/// kuruş → "240000.00"
fn tl(k: i64) -> String {
    format!("{}.{:02}", k / 100, (k % 100).abs())
}

/// GET /api/belge/oys/faturalar — GİB gelen/giden fatura listesi (tam detay).
pub async fn faturalar() -> Json<Value> {
    Json(veri().get("faturalar").cloned().unwrap_or(Value::Array(vec![])))
}

/// GET /api/belge/oys/fatura/:uuid/xml — faturayı UBL-TR XML olarak indir.
pub async fn fatura_xml(Path(uuid): Path<String>) -> ([(HeaderName, String); 2], String) {
    let v = veri();
    let bos = vec![];
    let faturalar = v["faturalar"].as_array().unwrap_or(&bos);
    let f = faturalar.iter().find(|f| f["uuid"].as_str() == Some(uuid.as_str()));
    let (xml, no) = match f {
        Some(f) => (ubltr_uret(f, &v["mukellef"]), f["no"].as_str().unwrap_or("fatura").to_string()),
        None => ("<Hata>Fatura bulunamadı</Hata>".to_string(), "bulunamadi".to_string()),
    };
    (
        [
            (CONTENT_TYPE, "application/xml; charset=utf-8".to_string()),
            (CONTENT_DISPOSITION, format!("attachment; filename=\"{no}.xml\"")),
        ],
        xml,
    )
}

/// GET /api/belge/oys/ekstre-isaretli/:ref — ilgili ödemenin İŞARETLİ olduğu ekstreyi CSV indir.
pub async fn ekstre_isaretli(Path(reference): Path<String>) -> ([(HeaderName, String); 2], String) {
    let v = veri();
    let bos = vec![];
    let hareketler = v["banka_hareketleri"].as_array().unwrap_or(&bos);
    let mut satir = String::from("\u{feff}İşaret;Referans;Tarih;Yön;Tutar (TL);Açıklama;Eşleşme\n");
    for h in hareketler {
        let ref_h = h["ref"].as_str().unwrap_or("");
        let isaret = if ref_h == reference { "► İŞARETLİ" } else { "" };
        satir.push_str(&format!(
            "{};{};{};{};{};{};{}\n",
            isaret,
            ref_h,
            h["tarih"].as_str().unwrap_or(""),
            h["yon"].as_str().unwrap_or(""),
            tl(h["tutar"].as_i64().unwrap_or(0)),
            h["aciklama"].as_str().unwrap_or("").replace(';', ","),
            h["eslesme"].as_str().unwrap_or("—"),
        ));
    }
    (
        [
            (CONTENT_TYPE, "text/csv; charset=utf-8".to_string()),
            (CONTENT_DISPOSITION, format!("attachment; filename=\"ekstre-{reference}-isaretli.csv\"")),
        ],
        satir,
    )
}

// ── Eşleştirme motoru — banka hareketi (gelir/gider) ↔ fatura, HESAPLANARAK ────
// JSON'daki gömülü eşleşmeyi kullanmaz; ham banka hareketlerinden karşı taraf + tutar + tarih + yön
// ile taksitleri eşler. Böylece sonuç deterministik + tutarlı + gerçek veriyle aynı motor.

#[derive(serde::Serialize)]
pub struct TaksitEsl {
    pub no: i64,
    pub tutar: i64,
    pub durum: &'static str, // TAHSIL | GECIKMIS | BEKLIYOR
    pub eslesen_ref: Option<String>,
    pub eslesen_tarih: Option<String>,
    pub eslesen_yon: Option<String>, // GIRIS(gelir) | CIKIS(gider)
}
#[derive(serde::Serialize)]
pub struct FaturaEsl {
    pub uuid: String,
    pub no: String,
    pub yon: String,
    pub karsi: String,
    pub toplam: i64,
    pub tahsil: i64,
    pub acik: i64,
    pub taksitler: Vec<TaksitEsl>,
}
#[derive(serde::Serialize)]
pub struct EslesmeyenH {
    pub reference: String,
    pub tarih: String,
    pub yon: String,
    pub tutar: i64,
    pub aciklama: String,
}
#[derive(serde::Serialize)]
pub struct EslestirmeYanit {
    pub faturalar: Vec<FaturaEsl>,
    pub eslesmeyenler: Vec<EslesmeyenH>,
}

/// İsim normalize: büyük harf, ünvan ekleri/nokta at, İ/I sadeleştir → ilk anlamlı kelime eşleşmesi.
fn norm(s: &str) -> String {
    let u = s.to_uppercase().replace('İ', "I").replace('.', " ");
    u.split_whitespace()
        .filter(|w| !["AŞ", "LTD", "ŞTI", "STI", "SAN", "TIC", "A.Ş", "A"].contains(w))
        .collect::<Vec<_>>()
        .join(" ")
}
fn ilk_kelime(s: &str) -> String {
    norm(s).split_whitespace().next().unwrap_or("").to_string()
}
fn anahtar(t: &str) -> String {
    let p: Vec<&str> = t.split('.').collect();
    if p.len() == 3 { format!("{}{}{}", p[2], p[1], p[0]) } else { t.to_string() }
}

/// GET /api/belge/oys/eslestirme — banka hareketleri ↔ faturalar (motorla hesaplanmış).
pub async fn eslestirme() -> Json<EslestirmeYanit> {
    let v = veri();
    let bos = vec![];
    let faturalar = v["faturalar"].as_array().unwrap_or(&bos).clone();
    let hareketler = v["banka_hareketleri"].as_array().unwrap_or(&bos).clone();

    let mut kullanildi: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut fatura_ciktilari = Vec::new();

    for f in &faturalar {
        let yon = f["yon"].as_str().unwrap_or("SATIS");
        let beklenen_hareket_yon = if yon == "SATIS" { "GIRIS" } else { "CIKIS" };
        let karsi = f["karsi"]["unvan"].as_str().unwrap_or("");
        let karsi_key = ilk_kelime(karsi);
        let fatura_tarih = anahtar(f["tarih"].as_str().unwrap_or(""));
        let mut tahsil = 0i64;
        let mut taksitler = Vec::new();

        for vp in f["vade_plani"].as_array().unwrap_or(&bos) {
            let no = vp["no"].as_i64().unwrap_or(0);
            let tutar = vp["tutar"].as_i64().unwrap_or(0);
            let vade = anahtar(vp["vade"].as_str().unwrap_or(""));
            // Uyan banka hareketini ara: yön + tutar + karşı taraf + tarih ≥ fatura tarihi.
            let esles = hareketler.iter().find(|h| {
                let r = h["ref"].as_str().unwrap_or("");
                !kullanildi.contains(r)
                    && h["yon"].as_str() == Some(beklenen_hareket_yon)
                    && h["tutar"].as_i64() == Some(tutar)
                    && norm(h["aciklama"].as_str().unwrap_or("")).contains(&karsi_key)
                    && anahtar(h["tarih"].as_str().unwrap_or("")) >= fatura_tarih
            });
            let (durum, eref, etarih, eyon) = match esles {
                Some(h) => {
                    let r = h["ref"].as_str().unwrap_or("").to_string();
                    kullanildi.insert(r.clone());
                    tahsil += tutar;
                    ("TAHSIL", Some(r), h["tarih"].as_str().map(String::from), h["yon"].as_str().map(String::from))
                }
                None => {
                    // Bugün (sim: 31.05.2026) itibarıyla vadesi geçti mi?
                    let d = if vade < "20260531".to_string() { "GECIKMIS" } else { "BEKLIYOR" };
                    (d, None, None, None)
                }
            };
            taksitler.push(TaksitEsl { no, tutar, durum, eslesen_ref: eref, eslesen_tarih: etarih, eslesen_yon: eyon });
        }
        let toplam = f["toplam"].as_i64().unwrap_or(0);
        fatura_ciktilari.push(FaturaEsl {
            uuid: f["uuid"].as_str().unwrap_or("").to_string(),
            no: f["no"].as_str().unwrap_or("").to_string(),
            yon: yon.to_string(),
            karsi: karsi.to_string(),
            toplam, tahsil, acik: toplam - tahsil, taksitler,
        });
    }

    // Hiçbir faturaya bağlanmayan banka hareketleri → manuel karar bekler.
    let eslesmeyenler = hareketler
        .iter()
        .filter(|h| !kullanildi.contains(h["ref"].as_str().unwrap_or("")))
        .map(|h| EslesmeyenH {
            reference: h["ref"].as_str().unwrap_or("").to_string(),
            tarih: h["tarih"].as_str().unwrap_or("").to_string(),
            yon: h["yon"].as_str().unwrap_or("").to_string(),
            tutar: h["tutar"].as_i64().unwrap_or(0),
            aciklama: h["aciklama"].as_str().unwrap_or("").to_string(),
        })
        .collect();

    Json(EslestirmeYanit { faturalar: fatura_ciktilari, eslesmeyenler })
}

// ── İZ-SÜRME (provenance) — tutardan tüm zincire: bileşenler (yukarı) + beslediği (aşağı) ──
#[derive(serde::Serialize)]
pub struct IzHedef { pub rota: String, pub id: Option<String>, pub url: Option<String> }
#[derive(serde::Serialize)]
pub struct IzDugum {
    pub id: String, pub tip: String, pub etiket: String, pub tutar: Option<i64>, pub durum: String,
    pub kaynak: String, // DIS(banka) | GIB(fatura) | IC(fiş) | BELGE
    pub alanlar: serde_json::Value,
    pub satirlar: Option<serde_json::Value>, // tahakkuk fişi satırları
    pub kalemler: Option<serde_json::Value>, // fatura kalemleri (bileşenler)
    pub hedef: Option<IzHedef>,
}
#[derive(serde::Serialize)]
pub struct IzKenar { pub kaynak: String, pub hedef: String, pub etiket: String }
#[derive(serde::Serialize)]
pub struct IzUc { pub etiket: String, pub deger: String, pub tutar: Option<i64> }
#[derive(serde::Serialize)]
pub struct IzYanit {
    pub iz_id: String,
    pub guven: serde_json::Value, // { skor, durum, gerekce }
    pub ozet: serde_json::Value,  // { sol, sag, yon }
    pub dugumler: Vec<IzDugum>,
    pub kenarlar: Vec<IzKenar>,
}

/// GET /api/belge/oys/iz/:anahtar — anahtar = fatura no (ACM../TED..) veya banka ref (REF..).
/// Tutarın tüm zincirini üretir: FATURA(kalemler=bileşen) → TAHAKKUK FİŞİ(besleme) → TAHSİS → BANKA → DAYANAK.
pub async fn iz(
    axum::extract::State(state): axum::extract::State<crate::Shared>,
    Path(anahtar): Path<String>,
) -> Json<IzYanit> {
    // İz-sür, listelerle AYNI kaynağı okumalı: manuel eşleştirme + deftere geçmiş (donmuş)
    // dağıtım dahil görünüm. Yoksa maker'ın kararı izde görünmez → denetim izi yanıltıcı olur.
    let gorunum = { let st = state.lock().unwrap(); crate::gorunum(&st) };
    // HAVADA BAKİYE: bankaya para girmiş ama karşılığında belge/kayıt yok → tamlık bulgusu.
    if anahtar.starts_with("HAVADA-") {
        return Json(IzYanit {
            iz_id: anahtar.clone(),
            guven: serde_json::json!({ "skor": 0, "durum": "ESLESMEYEN", "gerekce": "HAVADA BAKİYE — bankaya para girmiş, karşılığında belge/kayıt YOK. Tamlık ihlali / kayıt dışı hasılat riski (VUK). Denetçi/muhasebeci sorgulamalı." }),
            ozet: serde_json::json!({ "sol": { "etiket": "Belgesiz banka girişi", "deger": anahtar, "tutar": serde_json::Value::Null }, "sag": serde_json::Value::Null, "yon": "?" }),
            dugumler: vec![IzDugum {
                id: "n_banka".into(), tip: "BANKA_HAREKETI".into(), etiket: "Belgesiz para girişi".into(),
                tutar: None, durum: "ESLESMEYEN".into(), kaynak: "DIS".into(),
                alanlar: serde_json::json!({ "Referans": anahtar, "Durum": "Dayanaksız — havada bakiye", "Uyarı": "Karşılığında fatura/kayıt yok — kayıt dışı riski" }),
                satirlar: None, kalemler: None,
                hedef: Some(IzHedef { rota: "banka".into(), id: Some(anahtar.clone()), url: None }),
            }],
            kenarlar: vec![],
        });
    }
    // Simülasyon banka referansı (SIMREF-{fatura no}) → doğrudan faturaya çöz.
    let anahtar = anahtar.strip_prefix("SIMREF-").map(|s| s.to_string()).unwrap_or(anahtar);
    // TAHSİLAT referansı (THS-n) → tahsis motorundan hangi faturayı/faturaları kapattığını bul.
    // Bir tahsilat birden çok faturayı kapatabilir (FIFO), o yüzden ilk tahsisin faturasına köprüle.
    let anahtar = if anahtar.starts_with("THS-") {
        gorunum.faturalar.iter()
            .find(|f| f.tahsisler.iter().any(|t| t.tahsilat_id == anahtar))
            .map(|f| f.no.clone())
            .unwrap_or(anahtar)
    } else { anahtar };
    let v = veri();
    let bos = vec![];
    let faturalar = v["faturalar"].as_array().unwrap_or(&bos);
    let hareketler = v["banka_hareketleri"].as_array().unwrap_or(&bos);

    // Anahtar banka ref ise → eşleştiği faturayı bul.
    let fatura = if anahtar.starts_with("REF") {
        let esl = hareketler.iter().find(|h| h["ref"].as_str() == Some(anahtar.as_str()));
        let esl_str = esl.and_then(|h| h["eslesme"].as_str()).unwrap_or("");
        let fno = esl_str.split(" / ").next().unwrap_or("");
        faturalar.iter().find(|f| f["no"].as_str() == Some(fno))
    } else {
        faturalar.iter().find(|f| f["no"].as_str() == Some(anahtar.as_str()))
    };

    // Simülasyon faturası (Belgeler sayfası buradan besleniyor) — ornek-edonusum'da yoksa köprüle.
    if fatura.is_none() && !anahtar.starts_with("REF") {
        if let Some(f) = gorunum.faturalar.clone().into_iter().find(|x| x.no == anahtar) {
            let satis = f.yon == "SATIS";
            let mut dl = Vec::new();
            let mut kl = Vec::new();
            dl.push(IzDugum {
                id: "n_fatura".into(), tip: "FATURA".into(),
                etiket: format!("{} · {}", if satis { "SATIŞ" } else { "ALIŞ" }, f.karsi),
                tutar: Some(f.toplam), durum: if f.kalan == 0 { "TAM".into() } else if f.tahsil > 0 { "KISMI".into() } else { "ESLESMEYEN".into() }, kaynak: "GIB".into(),
                alanlar: serde_json::json!({ "Belge no": f.no, "Karşı taraf": f.karsi, "Matrah": f.matrah, "KDV": f.kdv, "Toplam": f.toplam, "Tahsil": f.tahsil, "Açık (kalan)": f.kalan, "Enstrüman": f.enstruman }),
                satirlar: None, kalemler: None,
                hedef: Some(IzHedef { rota: "belgeler".into(), id: Some(f.no.clone()), url: None }),
            });
            // TASLAK faturada tahakkuk yoktur (henüz belge değil) → zincirde de görünmez.
            if let Some(th) = &f.tahakkuk {
                dl.push(IzDugum {
                    id: "n_fis".into(), tip: "TAHAKKUK_FIS".into(), etiket: format!("Tahakkuk · {}", th.no),
                    tutar: Some(f.toplam), durum: if f.gecerli { "TAM".into() } else { "ESLESMEYEN".into() },
                    kaynak: "IC".into(),
                    alanlar: serde_json::json!({
                        "Fiş no": th.no, "Tarih": th.tarih,
                        "Durum": if f.gecerli { "geçerli" } else { "hükümsüz — ters kayıtla kapatıldı" },
                    }),
                    satirlar: Some(serde_json::to_value(&th.satirlar).unwrap_or(serde_json::Value::Null)),
                    kalemler: None, hedef: None,
                });
                kl.push(IzKenar { kaynak: "n_fatura".into(), hedef: "n_fis".into(), etiket: "tahakkuk".into() });
            }
            // İPTAL/RED → ters kayıt zincirde ayrı düğüm; denetçi iptali görebilmeli.
            if let Some(ip) = &f.iptal_fisi {
                dl.push(IzDugum {
                    id: "n_iptal".into(), tip: "TAHAKKUK_FIS".into(), etiket: format!("İPTAL · {}", ip.no),
                    tutar: Some(f.toplam), durum: "ESLESMEYEN".into(), kaynak: "IC".into(),
                    alanlar: serde_json::json!({ "Fiş no": ip.no, "Tarih": ip.tarih, "Gerekçe": ip.aciklama }),
                    satirlar: Some(serde_json::to_value(&ip.satirlar).unwrap_or(serde_json::Value::Null)),
                    kalemler: None, hedef: None,
                });
                kl.push(IzKenar { kaynak: "n_fis".into(), hedef: "n_iptal".into(), etiket: "ters kayıt".into() });
            }
            // İADE → 610 kaydı (600 ters çevrilmez).
            if let Some(ia) = &f.iade_fisi {
                dl.push(IzDugum {
                    id: "n_iade".into(), tip: "TAHAKKUK_FIS".into(), etiket: format!("İADE · {}", ia.no),
                    tutar: Some(f.iade_matrah + f.iade_kdv), durum: "KISMI".into(), kaynak: "IC".into(),
                    alanlar: serde_json::json!({ "Fiş no": ia.no, "Tarih": ia.tarih, "Açıklama": ia.aciklama }),
                    satirlar: Some(serde_json::to_value(&ia.satirlar).unwrap_or(serde_json::Value::Null)),
                    kalemler: None, hedef: None,
                });
                kl.push(IzKenar { kaynak: "n_fis".into(), hedef: "n_iade".into(), etiket: "iade".into() });
            }
            if let Some(od) = &f.odeme {
                dl.push(IzDugum {
                    id: "n_odeme".into(),
                    tip: if f.enstruman == "BANKA" { "BANKA_HAREKETI".into() } else { "TAHSIS".into() },
                    etiket: format!("Ödeme · {}", f.enstruman), tutar: Some(f.toplam), durum: "TAM".into(),
                    kaynak: if f.enstruman == "BANKA" { "DIS".into() } else { "IC".into() },
                    alanlar: serde_json::json!({ "Fiş no": od.no, "Enstrüman": f.enstruman, "Tahsil": f.tahsil, "Açıklama": od.aciklama }),
                    satirlar: Some(serde_json::to_value(&od.satirlar).unwrap_or(serde_json::Value::Null)),
                    kalemler: None,
                    // BANKA ödemesi → ekstredeki ilgili satıra geçiş (SIMREF-{fatura no}).
                    hedef: if f.enstruman == "BANKA" {
                        f.tahsisler.first().map(|t| IzHedef { rota: "banka".into(), id: Some(t.tahsilat_id.clone()), url: None })
                    } else { None },
                });
                kl.push(IzKenar { kaynak: "n_odeme".into(), hedef: "n_fatura".into(), etiket: format!("{} ödeme", f.enstruman) });
            }
            let (durum, skor) = if f.kalan == 0 { ("TAM", 100u8) } else if f.tahsil > 0 { ("KISMI", 60) } else { ("ESLESMEYEN", 30) };
            let gerekce = if f.kalan == 0 { format!("{} ile tam ödendi (kalan 0).", f.enstruman) }
                else if f.tahsil > 0 { format!("{} ile KISMİ: tahsil {} ₺ · açık {} ₺.", f.enstruman, f.tahsil / 100, f.kalan / 100) }
                else { "Açık — henüz ödeme yok (şüpheli alacak adayı).".to_string() };
            return Json(IzYanit {
                iz_id: f.no.clone(),
                guven: serde_json::json!({ "skor": skor, "durum": durum, "gerekce": gerekce }),
                ozet: serde_json::json!({ "sol": { "etiket": if satis { "Fatura (satış)" } else { "Fatura (alış)" }, "deger": f.no, "tutar": f.toplam }, "sag": { "etiket": "Karşı taraf", "deger": f.karsi }, "yon": if satis { "TAHSILAT" } else { "ODEME" } }),
                dugumler: dl, kenarlar: kl,
            });
        }
    }

    let mut dugumler = Vec::new();
    let mut kenarlar = Vec::new();

    let Some(f) = fatura else {
        // Eşleşmeyen banka hareketi → tek düğüm.
        if let Some(h) = hareketler.iter().find(|h| h["ref"].as_str() == Some(anahtar.as_str())) {
            dugumler.push(IzDugum {
                id: "n_banka".into(), tip: "BANKA_HAREKETI".into(),
                etiket: format!("İş Bankası · {}", h["aciklama"].as_str().unwrap_or("")),
                tutar: h["tutar"].as_i64(), durum: "ESLESMEYEN".into(), kaynak: "DIS".into(),
                alanlar: serde_json::json!({ "Referans": h["ref"], "Tarih": h["tarih"], "Yön": h["yon"], "Açıklama": h["aciklama"] }),
                satirlar: None, kalemler: None,
                hedef: Some(IzHedef { rota: "banka".into(), id: h["ref"].as_str().map(String::from), url: None }),
            });
        }
        return Json(IzYanit {
            iz_id: anahtar.clone(),
            guven: serde_json::json!({ "skor": 0, "durum": "ESLESMEYEN", "gerekce": "Bu hareket hiçbir faturaya bağlanamadı — dayanaksız (Tamlık bulgusu)." }),
            ozet: serde_json::json!({ "sol": { "etiket": "Banka hareketi", "deger": anahtar, "tutar": serde_json::Value::Null }, "sag": serde_json::Value::Null, "yon": "?" }),
            dugumler, kenarlar,
        });
    };

    let yon = f["yon"].as_str().unwrap_or("SATIS");
    let satis = yon == "SATIS";
    let no = f["no"].as_str().unwrap_or("").to_string();
    let toplam = f["toplam"].as_i64().unwrap_or(0);
    let karsi = f["karsi"]["unvan"].as_str().unwrap_or("");
    let karsi_key = ilk_kelime(karsi);
    let fatura_key = anahtar_t(f["tarih"].as_str().unwrap_or(""));

    // FATURA düğümü — kalemler = tutarın BİLEŞENLERİ (yukarı)
    dugumler.push(IzDugum {
        id: "n_fatura".into(), tip: "FATURA".into(),
        etiket: format!("{} · {}", if satis { "SATIŞ" } else { "ALIŞ" }, karsi),
        tutar: Some(toplam), durum: "TAM".into(), kaynak: "GIB".into(),
        alanlar: serde_json::json!({ "Belge no": no, "VKN": f["karsi"]["vkn"], "Matrah": f["matrah"], "KDV": f["kdv"], "Toplam": toplam }),
        satirlar: None, kalemler: Some(f["satirlar"].clone()),
        hedef: Some(IzHedef { rota: "belgeler".into(), id: Some(no.clone()), url: None }),
    });
    // TAHAKKUK FİŞİ — tutarın BESLEDİĞİ muhasebe kaydı (aşağı)
    dugumler.push(IzDugum {
        id: "n_fis".into(), tip: "TAHAKKUK_FIS".into(), etiket: "Tahakkuk fişi · Mahsup".into(),
        tutar: Some(toplam), durum: "TAM".into(), kaynak: "IC".into(),
        alanlar: serde_json::json!({ "Tip": f["tahakkuk_fis"]["tip"], "Tarih": f["tahakkuk_fis"]["tarih"] }),
        satirlar: Some(f["tahakkuk_fis"]["satirlar"].clone()), kalemler: None, hedef: None,
    });
    kenarlar.push(IzKenar { kaynak: "n_fatura".into(), hedef: "n_fis".into(), etiket: "tahakkuk".into() });

    // TAHSİS + BANKA — ödemeler (hesaplanmış eşleştirme)
    let mut kullanildi = std::collections::HashSet::new();
    let mut tahsil = 0i64;
    let bh = if satis { "GIRIS" } else { "CIKIS" };
    for vp in f["vade_plani"].as_array().unwrap_or(&bos) {
        let tutar = vp["tutar"].as_i64().unwrap_or(0);
        let no_t = vp["no"].as_i64().unwrap_or(0);
        let esles = hareketler.iter().find(|h| {
            let r = h["ref"].as_str().unwrap_or("");
            !kullanildi.contains(r) && h["yon"].as_str() == Some(bh) && h["tutar"].as_i64() == Some(tutar)
                && norm(h["aciklama"].as_str().unwrap_or("")).contains(&karsi_key)
                && anahtar_t(h["tarih"].as_str().unwrap_or("")) >= fatura_key
        });
        if let Some(h) = esles {
            let r = h["ref"].as_str().unwrap_or("").to_string();
            kullanildi.insert(r.clone());
            tahsil += tutar;
            let tid = format!("n_tahsis{no_t}");
            let bid = format!("n_banka{no_t}");
            dugumler.push(IzDugum {
                id: tid.clone(), tip: "TAHSIS".into(), etiket: format!("Taksit {no_t} tahsisi"),
                tutar: Some(tutar), durum: "TAM".into(), kaynak: "IC".into(),
                alanlar: serde_json::json!({ "Taksit no": no_t, "Vade": vp["vade"], "Tutar": tutar }),
                satirlar: None, kalemler: None, hedef: None,
            });
            dugumler.push(IzDugum {
                id: bid.clone(), tip: "BANKA_HAREKETI".into(),
                etiket: format!("İş Bankası · {}", h["aciklama"].as_str().unwrap_or("")),
                tutar: Some(tutar), durum: "TAM".into(), kaynak: "DIS".into(),
                alanlar: serde_json::json!({ "Referans": r, "Tarih": h["tarih"], "Yön": h["yon"], "Açıklama": h["aciklama"] }),
                satirlar: None, kalemler: None,
                hedef: Some(IzHedef { rota: "banka".into(), id: Some(r), url: None }),
            });
            kenarlar.push(IzKenar { kaynak: bid, hedef: tid.clone(), etiket: format!("{} ₺", tutar / 100) });
            kenarlar.push(IzKenar { kaynak: tid, hedef: "n_fatura".into(), etiket: format!("taksit {no_t}") });
        }
    }
    let acik = toplam - tahsil;

    // DAYANAK — UBL-TR belge (indirilebilir)
    if let Some(uuid) = f["uuid"].as_str() {
        dugumler.push(IzDugum {
            id: "n_dayanak".into(), tip: "DAYANAK".into(), etiket: "UBL-TR e-Fatura XML".into(),
            tutar: None, durum: "TAM".into(), kaynak: "BELGE".into(),
            alanlar: serde_json::json!({ "Tür": "UBL-TR 2.1", "UUID": uuid, "Senaryo": f["senaryo"] }),
            satirlar: None, kalemler: None,
            hedef: Some(IzHedef { rota: "indir".into(), id: None, url: Some(format!("/api/belge/oys/fatura/{uuid}/xml")) }),
        });
        kenarlar.push(IzKenar { kaynak: "n_fis".into(), hedef: "n_dayanak".into(), etiket: "dayanak".into() });
    }

    let (durum, skor) = if acik == 0 { ("TAM", 100u8) } else if tahsil > 0 { ("KISMI", 60) } else { ("ESLESMEYEN", 30) };
    Json(IzYanit {
        iz_id: no.clone(),
        guven: serde_json::json!({ "skor": skor, "durum": durum,
            "gerekce": format!("Toplam {} ₺ · tahsil {} ₺ · açık {} ₺", toplam/100, tahsil/100, acik/100) }),
        ozet: serde_json::json!({
            "sol": { "etiket": if satis {"Fatura (satış)"} else {"Fatura (alış)"}, "deger": no, "tutar": toplam },
            "sag": { "etiket": "Karşı taraf", "deger": karsi, "tutar": serde_json::Value::Null },
            "yon": if satis { "TAHSILAT" } else { "ODEME" }
        }),
        dugumler, kenarlar,
    })
}

fn anahtar_t(t: &str) -> String {
    let p: Vec<&str> = t.split('.').collect();
    if p.len() == 3 { format!("{}{}{}", p[2], p[1], p[0]) } else { t.to_string() }
}

/// Fatura JSON'undan basit (temsili) UBL-TR XML üretir. yon=SATIS ise satıcı=mükellef; ALIS ise alıcı=mükellef.
fn ubltr_uret(f: &Value, mukellef: &Value) -> String {
    let yon = f["yon"].as_str().unwrap_or("SATIS");
    let mk_vkn = mukellef["vkn"].as_str().unwrap_or("");
    let mk_ad = mukellef["unvan"].as_str().unwrap_or("");
    let kr_vkn = f["karsi"]["vkn"].as_str().unwrap_or("");
    let kr_ad = f["karsi"]["unvan"].as_str().unwrap_or("");
    let (sat_vkn, sat_ad, alc_vkn, alc_ad) = if yon == "SATIS" {
        (mk_vkn, mk_ad, kr_vkn, kr_ad)
    } else {
        (kr_vkn, kr_ad, mk_vkn, mk_ad)
    };

    let bos = vec![];
    let mut satirlar_xml = String::new();
    for (i, s) in f["satirlar"].as_array().unwrap_or(&bos).iter().enumerate() {
        satirlar_xml.push_str(&format!(
            "  <cac:InvoiceLine>\n    <cbc:ID>{}</cbc:ID>\n    <cbc:InvoicedQuantity>{}</cbc:InvoicedQuantity>\n    <cbc:LineExtensionAmount currencyID=\"TRY\">{}</cbc:LineExtensionAmount>\n    <cac:Item><cbc:Name>{}</cbc:Name></cac:Item>\n    <cac:Price><cbc:PriceAmount currencyID=\"TRY\">{}</cbc:PriceAmount></cac:Price>\n  </cac:InvoiceLine>\n",
            i + 1,
            s["miktar"].as_i64().unwrap_or(0),
            tl(s["tutar"].as_i64().unwrap_or(0)),
            s["ad"].as_str().unwrap_or(""),
            tl(s["birim_fiyat"].as_i64().unwrap_or(0)),
        ));
    }

    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<Invoice xmlns=\"urn:oasis:names:specification:ubl:schema:xsd:Invoice-2\" xmlns:cbc=\"urn:oasis:names:specification:ubl:schema:xsd:CommonBasicComponents-2\" xmlns:cac=\"urn:oasis:names:specification:ubl:schema:xsd:CommonAggregateComponents-2\">\n  <cbc:UBLVersionID>2.1</cbc:UBLVersionID>\n  <cbc:CustomizationID>TR1.2.1</cbc:CustomizationID>\n  <cbc:ProfileID>{senaryo}</cbc:ProfileID>\n  <cbc:ID>{no}</cbc:ID>\n  <cbc:UUID>{uuid}</cbc:UUID>\n  <cbc:IssueDate>{tarih}</cbc:IssueDate>\n  <cbc:InvoiceTypeCode>{yon}</cbc:InvoiceTypeCode>\n  <cac:AccountingSupplierParty><cac:Party>\n    <cac:PartyIdentification><cbc:ID schemeID=\"VKN\">{sat_vkn}</cbc:ID></cac:PartyIdentification>\n    <cac:PartyName><cbc:Name>{sat_ad}</cbc:Name></cac:PartyName>\n  </cac:Party></cac:AccountingSupplierParty>\n  <cac:AccountingCustomerParty><cac:Party>\n    <cac:PartyIdentification><cbc:ID schemeID=\"VKN\">{alc_vkn}</cbc:ID></cac:PartyIdentification>\n    <cac:PartyName><cbc:Name>{alc_ad}</cbc:Name></cac:PartyName>\n  </cac:Party></cac:AccountingCustomerParty>\n  <cac:TaxTotal>\n    <cbc:TaxAmount currencyID=\"TRY\">{kdv}</cbc:TaxAmount>\n    <cac:TaxSubtotal><cbc:TaxableAmount currencyID=\"TRY\">{matrah}</cbc:TaxableAmount><cbc:Percent>20</cbc:Percent></cac:TaxSubtotal>\n  </cac:TaxTotal>\n  <cac:LegalMonetaryTotal>\n    <cbc:LineExtensionAmount currencyID=\"TRY\">{matrah}</cbc:LineExtensionAmount>\n    <cbc:TaxExclusiveAmount currencyID=\"TRY\">{matrah}</cbc:TaxExclusiveAmount>\n    <cbc:TaxInclusiveAmount currencyID=\"TRY\">{toplam}</cbc:TaxInclusiveAmount>\n    <cbc:PayableAmount currencyID=\"TRY\">{toplam}</cbc:PayableAmount>\n  </cac:LegalMonetaryTotal>\n{satirlar}</Invoice>\n",
        senaryo = f["senaryo"].as_str().unwrap_or("TEMELFATURA"),
        no = f["no"].as_str().unwrap_or(""),
        uuid = f["uuid"].as_str().unwrap_or(""),
        tarih = f["tarih"].as_str().unwrap_or(""),
        yon = yon,
        sat_vkn = sat_vkn, sat_ad = sat_ad, alc_vkn = alc_vkn, alc_ad = alc_ad,
        kdv = tl(f["kdv"].as_i64().unwrap_or(0)),
        matrah = tl(f["matrah"].as_i64().unwrap_or(0)),
        toplam = tl(f["toplam"].as_i64().unwrap_or(0)),
        satirlar = satirlar_xml,
    )
}
