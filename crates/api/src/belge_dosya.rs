//! BELGE DOSYASI — yüklenen PDF/görüntüden METİN KATMANI ve ÖNİZLEME çıkarır.
//!
//! ## Neden metin katmanı bu kadar önemli
//!
//! Bir PDF iki türlü olur:
//!   · **Metin katmanlı** (e-Fatura çıktısı, banka ekstresi, muhasebe programı çıktısı) —
//!     karakterler dosyanın içinde YAZILI durur. Çıkarım birebirdir, okuma hatası olmaz.
//!   · **Taranmış** (fotokopi, telefon fotoğrafı) — içinde yalnız piksel vardır.
//!
//! Bu ayrım, alım modunu belirler: metin katmanı varsa OTOMATİK mod anlamlıdır; yoksa
//! otomatik mod sahte bir güven verir ve akış SEÇİMLİ/MANUEL moda düşmelidir. Motor bunu
//! tahmin etmez, **ölçer**: metin çıkarılabiliyor mu, kaç karakter geldi.
//!
//! ## Neden hem metin hem görüntü döner
//!
//! Kullanıcı belgeyi GÖRMELİ. Değeri "belge üzerinden seçmek", belgeyi görmeden yapılamaz;
//! yalnız metin göstermek de yetmez çünkü sayının belgenin neresinde durduğu bağlamdır.

use axum::Json;
use serde::{Deserialize, Serialize};
use std::process::Command;

/// Yüklenebilecek en büyük dosya (ham bayt). Muhasebe belgeleri küçüktür; bu sınır
/// kazara yüklenen dev dosyanın belleği ve geçici diski doldurmasını engeller.
const AZAMI_BAYT: usize = 25 * 1024 * 1024;

/// Sayfa 1 önizlemesinin çözünürlüğü. Ekranda okunabilir olmalı ama yanıtı şişirmemeli.
const ONIZLEME_DPI: &str = "110";

// ─────────────────────────────────────────────────────────────────────────────
// base64 — bağımlılık eklemeden
// ─────────────────────────────────────────────────────────────────────────────
// Tek kullanım için crate eklemek yerine yazıldı: girdi kullanıcı dosyasıdır, bozuk
// veri gelebilir, bu yüzden çözücü hata durumunda None döner (panik yok).

const ABC: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn b64_coz(s: &str) -> Option<Vec<u8>> {
    let mut tersi = [255u8; 256];
    for (i, c) in ABC.iter().enumerate() { tersi[*c as usize] = i as u8; }
    let (mut tampon, mut bit, mut cikti) = (0u32, 0u32, Vec::new());
    for c in s.bytes() {
        if c == b'=' || c == b'\n' || c == b'\r' || c == b' ' { continue; }
        let v = tersi[c as usize];
        if v == 255 { return None; } // base64 dışı karakter → veri bozuk
        tampon = (tampon << 6) | v as u32;
        bit += 6;
        if bit >= 8 { bit -= 8; cikti.push((tampon >> bit) as u8); }
    }
    Some(cikti)
}

fn b64_yaz(v: &[u8]) -> String {
    let mut o = String::with_capacity(v.len().div_ceil(3) * 4);
    for p in v.chunks(3) {
        let n = ((p[0] as u32) << 16) | ((*p.get(1).unwrap_or(&0) as u32) << 8) | *p.get(2).unwrap_or(&0) as u32;
        o.push(ABC[(n >> 18 & 63) as usize] as char);
        o.push(ABC[(n >> 12 & 63) as usize] as char);
        o.push(if p.len() > 1 { ABC[(n >> 6 & 63) as usize] as char } else { '=' });
        o.push(if p.len() > 2 { ABC[(n & 63) as usize] as char } else { '=' });
    }
    o
}

// ─────────────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct DosyaGirdi {
    #[serde(default)] pub ad: String,
    /// Dosyanın base64 içeriği (data: öneki varsa ayıklanır).
    pub veri_b64: String,
}

#[derive(Serialize)]
pub struct DosyaSonuc {
    pub ad: String,
    pub tur: String,              // PDF · GORUNTU · BILINMIYOR
    pub bayt: usize,
    pub metin_katmani: bool,
    pub metin: String,
    /// Metin satırları — arayüz bunları TIKLANABİLİR gösterir; kullanıcı değeri seçer.
    pub satirlar: Vec<String>,
    pub sayfa_sayisi: usize,
    /// Sayfa 1 (veya görüntünün kendisi) — data URI olarak, ekranda göstermek için.
    pub onizleme: String,
    pub onerilen_mod: String,
    /// Koordinatlı okunan kelime sayısı. 0 ise konum bilgisi yok (taranmış PDF ya da
    /// araç eksik) ve düz metne düşülmüştür — arayüz bunu kullanıcıya söyleyebilmeli.
    pub konum_kelime: usize,
    pub uyarilar: Vec<String>,
}

fn gecici_dizin() -> std::path::PathBuf {
    std::env::temp_dir().join("audit-liners-belge")
}

/// POST /api/belge/dosya — yüklenen belgeden metin katmanı + önizleme çıkarır.
///
/// Metin katmanı YOKSA bunu açıkça söyler ve otomatik mod önermez. Sessizce boş metin
/// dönmek, kullanıcının "otomatik okudu ama bir şey bulamadı" sanmasına yol açardı.
pub async fn belge_dosya(Json(g): Json<DosyaGirdi>) -> Json<serde_json::Value> {
    let ham = g.veri_b64.split_once(";base64,").map(|(_, b)| b).unwrap_or(&g.veri_b64);
    let Some(veri) = b64_coz(ham) else {
        return Json(serde_json::json!({ "hata": "dosya içeriği çözülemedi (base64 bozuk)" }));
    };
    if veri.is_empty() { return Json(serde_json::json!({ "hata": "dosya boş" })); }
    if veri.len() > AZAMI_BAYT {
        return Json(serde_json::json!({
            "hata": format!("dosya çok büyük: {} MB (sınır {} MB)", veri.len() / 1048576, AZAMI_BAYT / 1048576) }));
    }

    let pdf = veri.starts_with(b"%PDF");
    let goruntu = veri.starts_with(&[0xFF, 0xD8])            // JPEG
        || veri.starts_with(b"\x89PNG")
        || veri.starts_with(b"GIF8")
        || (veri.len() > 12 && &veri[8..12] == b"WEBP");
    let mut uyarilar: Vec<String> = Vec::new();

    if !pdf && !goruntu {
        return Json(serde_json::json!({
            "hata": "desteklenmeyen dosya türü — PDF veya görüntü (JPEG/PNG) bekleniyor",
            "ipucu": "Word/Excel belgesini önce PDF'e çevir." }));
    }

    // GÖRÜNTÜ: metin katmanı yoktur, önizleme dosyanın kendisidir.
    if goruntu {
        let mime = if veri.starts_with(&[0xFF, 0xD8]) { "image/jpeg" }
                   else if veri.starts_with(b"\x89PNG") { "image/png" }
                   else if veri.starts_with(b"GIF8") { "image/gif" } else { "image/webp" };
        // GÖRÜNTÜ OKUMA — taranmış belge burada metne çevrilir.
        // Çıktı, metin katmanlı PDF ile aynı biçimdedir; boru hattı ikisini ayırt etmez.
        let dizin = gecici_dizin();
        let _ = std::fs::create_dir_all(&dizin);
        let anahtar = veri.len() as u64 ^ veri.iter().take(512)
            .fold(0u64, |a, b| a.wrapping_mul(31).wrapping_add(*b as u64));
        let uzanti = if mime == "image/png" { "png" } else { "jpg" };
        let gyol = dizin.join(format!("g-{anahtar:x}.{uzanti}"));
        let _ = std::fs::write(&gyol, &veri);
        let kelimeler = crate::belge_konum::goruntuden_kelimeler(&gyol);
        let _ = std::fs::remove_file(&gyol);

        let metin = crate::belge_konum::izgaradan_metin(&kelimeler);
        let satirlar: Vec<String> = metin.lines().map(|s| s.trim_end().to_string())
            .filter(|s| !s.trim().is_empty()).collect();
        let okundu = kelimeler.len();
        let mut uy: Vec<String> = Vec::new();
        if okundu == 0 {
            uy.push("Görüntüden metin çıkarılamadı — okuma modeli kurulu olmayabilir ya da \
                     görüntü çok bozuk olabilir. Değerleri elle gir.".into());
        } else {
            uy.push(format!("Görüntüden {okundu} kelime okundu. Taranmış belgede okuma \
                             HATA PAYI TAŞIR — motor yalnız denklemi tutan alanları otomatik \
                             doldurur, gerisini onayına bırakır.").replace("  ", " "));
        }
        return Json(serde_json::to_value(DosyaSonuc {
            ad: g.ad.clone(), tur: "GORUNTU".into(), bayt: veri.len(),
            metin_katmani: okundu > 0, metin, satirlar, sayfa_sayisi: 1,
            konum_kelime: okundu,
            onizleme: format!("data:{mime};base64,{}", b64_yaz(&veri)),
            onerilen_mod: if okundu > 0 { "OTOMATIK".into() } else { "MANUEL".into() },
            uyarilar: uy,
        }).unwrap_or(serde_json::Value::Null));
    }

    // PDF: geçici dosyaya yaz, harici araçlarla metni ve sayfa görüntüsünü çıkar.
    let dizin = gecici_dizin();
    if std::fs::create_dir_all(&dizin).is_err() {
        return Json(serde_json::json!({ "hata": "geçici dizin açılamadı" }));
    }
    // Dosya adı içerikten türetilir; kullanıcının verdiği ad KULLANILMAZ (yol gezinme riski).
    let anahtar = veri.len() as u64 ^ veri.iter().take(512).fold(0u64, |a, b| a.wrapping_mul(31).wrapping_add(*b as u64));
    let yol = dizin.join(format!("b-{anahtar:x}.pdf"));
    if std::fs::write(&yol, &veri).is_err() {
        return Json(serde_json::json!({ "hata": "geçici dosya yazılamadı" }));
    }

    // ÖNCE KONUM MOTORU: kelimeleri koordinatlarıyla oku ve metni ORADAN üret.
    // Düz `-layout` çıktısı sütun bilgisini boşluk sayısına indirger ve farklı punto
    // karışımında bantları birleştirir; koordinatla satırlaştırma bunu doğru yapar.
    let konumlu = crate::belge_konum::kelimeler(&yol);
    let mut metin = crate::belge_konum::izgaradan_metin(&konumlu);
    let mut konum_kelime = konumlu.len();
    if metin.trim().is_empty() {
        // Konum bilgisi alınamadıysa düz metne düş — bu bir gerileme değil, yedek yol.
        metin = Command::new("pdftotext").arg("-layout").arg(&yol).arg("-").output()
            .ok().filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
            .unwrap_or_default();
        if !metin.trim().is_empty() {
            uyarilar.push("Konum bilgisi okunamadı, düz metne düşüldü — sütun ayrımı zayıf olabilir.".into());
        }
    }
    // TARANMIŞ PDF — metin katmanı yoksa sayfa görüntüye çevrilip OCR'a verilir.
    // Kullanıcının elindeki kağıt fatura taraması tam olarak bu yoldan geçer.
    let mut goruntuden = 0usize;
    if metin.chars().filter(|c| c.is_alphanumeric()).count() < 40 {
        let png = dizin.join(format!("s-{anahtar:x}"));
        let ok = Command::new("pdftoppm").arg("-png").arg("-r").arg("200")
            .arg("-f").arg("1").arg("-l").arg("1").arg(&yol).arg(&png)
            .output().map(|o| o.status.success()).unwrap_or(false);
        if ok {
            for ek in ["-1.png", "-01.png"] {
                let p = dizin.join(format!("s-{anahtar:x}{ek}"));
                if p.exists() {
                    let k = crate::belge_konum::goruntuden_kelimeler(&p);
                    let _ = std::fs::remove_file(&p);
                    if !k.is_empty() {
                        goruntuden = k.len();
                        metin = crate::belge_konum::izgaradan_metin(&k);
                        uyarilar.push(format!(
                            "PDF'te metin katmanı yok — sayfa görüntüsünden {goruntuden} kelime okundu. \
                             Taranmış okuma HATA PAYI TAŞIR; denklemi tutmayan alan doldurulmaz."));
                    }
                    break;
                }
            }
        }
    }
    if metin.is_empty() {
        uyarilar.push("Ne metin katmanı ne de görüntüden okuma sonuç verdi — değerleri elle gir.".into());
    }

    // Önizleme: yalnız 1. sayfa. Çok sayfalı belgede kullanıcı zaten dosyayı açabilir;
    // tüm sayfaları göndermek yanıtı megabaytlara çıkarırdı.
    let onizleme = Command::new("pdftoppm").arg("-png").arg("-r").arg(ONIZLEME_DPI)
        .arg("-f").arg("1").arg("-l").arg("1").arg(&yol).arg(dizin.join(format!("o-{anahtar:x}")))
        .output().ok()
        .and_then(|o| if o.status.success() {
            let p = dizin.join(format!("o-{anahtar:x}-1.png"));
            let p2 = dizin.join(format!("o-{anahtar:x}-01.png")); // pdftoppm sürüme göre dolgu değiştirir
            std::fs::read(&p).or_else(|_| std::fs::read(&p2)).ok()
        } else { None })
        .map(|b| format!("data:image/png;base64,{}", b64_yaz(&b)))
        .unwrap_or_default();
    if onizleme.is_empty() { uyarilar.push("Sayfa önizlemesi üretilemedi (pdftoppm).".into()); }

    // İşimiz bitti — geçici dosyalar silinir. Muhasebe belgesi geçici dizinde kalmamalı.
    let _ = std::fs::remove_file(&yol);
    for ek in ["-1.png", "-01.png"] { let _ = std::fs::remove_file(dizin.join(format!("o-{anahtar:x}{ek}"))); }

    if goruntuden > 0 { konum_kelime = goruntuden; }
    let satirlar: Vec<String> = metin.lines()
        .map(|s| s.trim_end().to_string())
        .filter(|s| !s.trim().is_empty())
        .collect();
    let sayfa_sayisi = metin.matches('\u{c}').count().max(1);
    // "Metin katmanı var" demek için birkaç karakter yetmez: taranmış PDF'lerde de
    // damga/filigran metni bulunabilir. Anlamlı bir eşik konur.
    let metin_katmani = metin.chars().filter(|c| c.is_alphanumeric()).count() >= 40;
    if !metin_katmani && !metin.trim().is_empty() {
        uyarilar.push("Çok az metin çıktı — belge büyük olasılıkla taranmış.".into());
    }

    Json(serde_json::to_value(DosyaSonuc {
        ad: g.ad.clone(), tur: "PDF".into(), bayt: veri.len(),
        metin_katmani, metin, satirlar, sayfa_sayisi, onizleme, konum_kelime,
        onerilen_mod: if metin_katmani { "OTOMATIK".into() } else { "SECIMLI".into() },
        uyarilar,
    }).unwrap_or(serde_json::Value::Null))
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn base64_gidis_donus() {
        for v in [&b""[..], b"a", b"ab", b"abc", b"%PDF-1.7\n\x00\xff\xfe"] {
            let s = b64_yaz(v);
            assert_eq!(b64_coz(&s).as_deref(), Some(v), "gidiş-dönüş bozuldu: {s}");
        }
    }

    #[test]
    fn bozuk_base64_panik_atmaz_none_doner() {
        // Girdi kullanıcı dosyasıdır; bozuk veri gelmesi olağandır, panik seçenek değildir.
        assert_eq!(b64_coz("!!!!"), None);
        assert_eq!(b64_coz("ab€cd"), None);
    }

    #[test]
    fn data_uri_oneki_ayiklanir() {
        let s = format!("data:application/pdf;base64,{}", b64_yaz(b"%PDF-x"));
        let ham = s.split_once(";base64,").map(|(_, b)| b).unwrap();
        assert_eq!(b64_coz(ham).as_deref(), Some(&b"%PDF-x"[..]));
    }

    #[tokio::test]
    async fn desteklenmeyen_tur_reddedilir() {
        let y = belge_dosya(Json(DosyaGirdi { ad: "x.docx".into(), veri_b64: b64_yaz(b"PK\x03\x04zip") })).await.0;
        assert!(y["hata"].as_str().unwrap_or("").contains("desteklenmeyen"), "{y:?}");
        assert!(y["ipucu"].as_str().unwrap_or("").contains("PDF"), "kullanıcıya çözüm söylenmeli");
    }

    #[tokio::test]
    async fn okunamayan_goruntude_otomatik_mod_onerilmez() {
        // Görüntü artık OCR'dan geçiyor. Ama okunamıyorsa otomatik mod ÖNERİLMEZ:
        // boş okumayla "otomatik doldurdum" demek sahte güven verir.
        // (Bu test eskiden "görüntüde otomatik mod hiç olmaz" diyordu; okuma zinciri
        //  bağlandıktan sonra kural değişti — okunabiliyorsa otomatik mod meşrudur.)
        let mut jpeg = vec![0xFF, 0xD8, 0xFF, 0xE0];
        jpeg.extend_from_slice(&[0u8; 64]); // içi boş — OCR bir şey bulamaz
        let y = belge_dosya(Json(DosyaGirdi { ad: "fis.jpg".into(), veri_b64: b64_yaz(&jpeg) })).await.0;
        assert_eq!(y["tur"], "GORUNTU");
        assert_eq!(y["metin_katmani"], false, "boş görüntüde metin katmanı iddia edilemez");
        assert_eq!(y["onerilen_mod"], "MANUEL", "okunamayan görüntüde elle giriş önerilmeli");
        assert!(y["onizleme"].as_str().unwrap_or("").starts_with("data:image/jpeg"),
            "görüntü kendi önizlemesidir");
        assert!(y["uyarilar"].as_array().unwrap().iter()
            .any(|u| u.as_str().unwrap_or("").contains("elle")),
            "kullanıcıya ne yapacağı söylenmeli");
    }

    #[tokio::test]
    async fn bos_dosya_reddedilir() {
        let y = belge_dosya(Json(DosyaGirdi { ad: "".into(), veri_b64: String::new() })).await.0;
        assert!(y["hata"].as_str().unwrap_or("").contains("boş"));
    }
}
