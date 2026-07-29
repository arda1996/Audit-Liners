//! ÖRNEKLEM ÖLÇÜM KOŞUSU — motorun gerçek belgelerde ne yaptığını SAYAR.
//!
//! ## Neden var
//!
//! Bugüne kadarki her doğruluk rakamı ya **belge-içi tutarlılık** (denklem tuttu mu) ya da
//! **sentetik bozulma** (ben bozdum, motor kurtardı mı) idi. İkisi de yer gerçeği değil.
//! Gerçek ölçüm için gerçek belge + elle doğrulanmış beklenen değer gerekir.
//!
//! ## Dizin düzeni
//!
//! ```text
//! data/ornek-belgeler/
//!   FATURA/
//!     K1-KOLAY/   fatura-01.pdf   fatura-01.json     ← her belgenin yanında beklenen değeri
//!     K2-ORTA/    ...
//!     K3-ZOR/     ...
//!     K4-ELYAZISI/...
//!   MAKBUZ/ ...
//! ```
//!
//! `<belge>.json` biçimi:
//! ```json
//! { "kaynak": "https://...", "lisans": "CC BY 4.0", "anonim": true,
//!   "beklenen": { "unvan": "...", "matrah": 320000, "kdv": 25600, "toplam": 345600 },
//!   "sutunlar": ["MAL_ADI","BIRIM","MIKTAR","BIRIM_FIYAT","KDV_ORANI","TUTAR"] }
//! ```
//! Tutarlar **KURUŞ**. `beklenen`i olmayan dosya test örneği DEĞİLDİR — ölçüme girmez.
//!
//! ## Ne ölçülür
//!
//! | Ölçü | Neden |
//! |---|---|
//! | **Kaynak** DIJITAL/OCR/YOK | Hangi yolun ne kadar kapsadığı |
//! | **Kelime · satır · koridor** | Sütun tespiti çalıştı mı |
//! | **Sütun adlandırma** | Kaç kolon tanındı, hangileri — kullanıcının asıl istediği |
//! | **Alan isabeti** | Beklenenle BİREBİR karşılaştırma |
//! | **Denklem** | Belge kendi kendini doğruladı mı |
//!
//! ## Dizin boşsa test ATLAR
//!
//! `data/ornek-belgeler/` git dışıdır (mükellef verisi içerebilir). Örnek yoksa koşu
//! "0 belge" der ve geçer — CI'ı kırmaz. Ama **sessizce geçmez**: kaç belge bulduğunu yazar.

use std::path::{Path, PathBuf};

#[derive(Debug, Default)]
pub struct Olcum {
    pub dosya: String,
    pub kategori: String,
    pub zorluk: String,
    /// DIJITAL · OCR · YOK
    pub kaynak: String,
    pub kelime: usize,
    pub satir: usize,
    pub sutun_taninan: usize,
    pub sutun_toplam: usize,
    pub alan_dogru: usize,
    pub alan_beklenen: usize,
    pub denklem_tuttu: bool,
    pub notlar: Vec<String>,
}

fn kok() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/ornek-belgeler"))
}

/// Belgeleri gezer: kök/KATEGORI/ZORLUK/*.pdf|png|jpg
fn belgeler() -> Vec<(String, String, PathBuf)> {
    let mut cikti = Vec::new();
    let Ok(kategoriler) = std::fs::read_dir(kok()) else { return cikti };
    for k in kategoriler.flatten().filter(|e| e.path().is_dir()) {
        let kat = k.file_name().to_string_lossy().to_string();
        let Ok(zorluklar) = std::fs::read_dir(k.path()) else { continue };
        for z in zorluklar.flatten().filter(|e| e.path().is_dir()) {
            let zor = z.file_name().to_string_lossy().to_string();
            let Ok(dosyalar) = std::fs::read_dir(z.path()) else { continue };
            for d in dosyalar.flatten() {
                let p = d.path();
                let uzanti = p.extension().and_then(|x| x.to_str()).unwrap_or("").to_lowercase();
                if matches!(uzanti.as_str(), "pdf" | "png" | "jpg" | "jpeg" | "tif" | "tiff") {
                    cikti.push((kat.clone(), zor.clone(), p));
                }
            }
        }
    }
    cikti.sort_by(|a, b| (&a.0, &a.1, &a.2).cmp(&(&b.0, &b.1, &b.2)));
    cikti
}

fn beklenen(belge: &Path) -> Option<serde_json::Value> {
    let yan = belge.with_extension("json");
    serde_json::from_str(&std::fs::read_to_string(yan).ok()?).ok()
}

/// Tek belgeyi ölçer.
pub fn olc(kategori: &str, zorluk: &str, yol: &Path) -> Olcum {
    let mut o = Olcum {
        dosya: yol.file_name().unwrap_or_default().to_string_lossy().to_string(),
        kategori: kategori.into(), zorluk: zorluk.into(), ..Default::default()
    };

    // ── OKUMA — dijital metin katmanı önce, yoksa görüntü ────────────────────
    let uzanti = yol.extension().and_then(|x| x.to_str()).unwrap_or("").to_lowercase();
    let kelimeler = if uzanti == "pdf" {
        let k = crate::belge_konum::kelimeler(yol);
        if k.is_empty() { crate::belge_konum::goruntuden_kelimeler(yol) } else { k }
    } else {
        crate::belge_konum::goruntuden_kelimeler(yol)
    };
    o.kelime = kelimeler.len();
    if kelimeler.is_empty() {
        o.kaynak = "YOK".into();
        o.notlar.push("hiç kelime okunamadı".into());
        return o;
    }
    o.kaynak = if uzanti == "pdf" && !crate::belge_konum::kelimeler(yol).is_empty()
        { "DIJITAL".into() } else { "OCR".into() };

    let izgara = crate::belge_konum::izgara(&kelimeler);
    o.satir = izgara.len();

    // ── SÜTUN ADLANDIRMA ─────────────────────────────────────────────────────
    if let Some(b) = crate::belge_sutun::basliklari_bul(&izgara) {
        o.sutun_taninan = b.taninan;
        o.sutun_toplam = b.sutunlar.len();
        o.notlar.push(format!("başlık: {}", b.sutunlar.iter()
            .map(|s| if s.anlam.is_empty() { format!("{}=?", s.ham) } else { s.anlam.clone() })
            .collect::<Vec<_>>().join("|")));
    } else {
        o.notlar.push("sütun başlığı bulunamadı".into());
    }

    // ── ALAN İSABETİ — beklenen varsa ────────────────────────────────────────
    let metin = crate::belge_konum::izgaradan_metin(&kelimeler);
    let Some(bek) = beklenen(yol) else {
        o.notlar.push("beklenen değer dosyası YOK — isabet ölçülemedi".into());
        return o;
    };
    let dil = { let (d, _) = crate::belge_oku::dil_sez(&metin); if d.is_empty() { "tr".into() } else { d } };
    let matrah = bek["beklenen"]["matrah"].as_i64();
    let kalem = crate::belge_kalem::kalemleri_izgaradan_bul(&izgara, &dil, matrah);
    o.denklem_tuttu = kalem.matrahla_tutuyor.unwrap_or(false);

    if let Some(alanlar) = bek["beklenen"].as_object() {
        o.alan_beklenen = alanlar.len();
        for (kod, deger) in alanlar {
            let bulundu = match deger {
                // HÜCRE düzeyinde aranır, satır düzeyinde DEĞİL. "KDV %8   3.600,00"
                // satırında iki sayı vardır (oran ve tutar); satırı bir bütün olarak
                // ayrıştırmak ikisini karıştırır ve tutar hiç bulunamaz.
                serde_json::Value::Number(n) => {
                    let hedef = n.as_i64().unwrap_or(i64::MIN);
                    izgara.iter().flatten()
                        .any(|h| crate::belge_oku::sayi_ayristir(h, &dil) == Some(hedef))
                }
                // Metin alanı: önce ham eşleşme, tutmazsa KOD NORMALİZASYONU.
                // Ölçülen gerçek: OCR "GIB2009..." kodunu "GlB2009..." okuyor (I→l) ve
                // 22 taramanın 15'inde fatura numarası kayboluyordu.
                serde_json::Value::String(s) => {
                    let sade = crate::belge_oku::sadelestir(s);
                    let ham_metin = crate::belge_oku::sadelestir(&metin);
                    ham_metin.contains(&sade) || {
                        let hedef = crate::belge_oku::kod_normalize(s);
                        izgara.iter().flatten().flat_map(|h| h.split_whitespace())
                            .any(|w| crate::belge_oku::kod_normalize(w) == hedef)
                    }
                }
                _ => false,
            };
            if bulundu { o.alan_dogru += 1; } else { o.notlar.push(format!("bulunamadı: {kod}")); }
        }
    }
    o
}

#[cfg(test)]
mod testler {
    use super::*;

    /// Örneklem ölçüm koşusu. Dizin boşsa ATLAR ama kaç belge bulduğunu YAZAR —
    /// sessizce geçmek, ölçüm yapıldığı yanılsaması yaratırdı.
    #[test]
    fn orneklem_olcum_kosusu() {
        let liste = belgeler();
        println!("\n╭─ ÖRNEKLEM ÖLÇÜMÜ ─ {} belge ─────────────────────────────", liste.len());
        if liste.is_empty() {
            println!("│ data/ornek-belgeler/ BOŞ — ölçüm yapılamadı.");
            println!("│ Düzen: <KATEGORI>/<K1-KOLAY|K2-ORTA|K3-ZOR|K4-ELYAZISI>/belge.pdf");
            println!("│ Her belgenin yanına aynı adla .json (beklenen değerler, KURUŞ).");
            println!("╰──────────────────────────────────────────────────────────\n");
            return;
        }

        let (mut okundu, mut sutunlu, mut tam_isabet) = (0usize, 0usize, 0usize);
        let mut alan_dogru = 0usize;
        let mut alan_toplam = 0usize;
        println!("│ {:<28} {:<10} {:<8} {:>6} {:>7} {:>9}", "DOSYA", "ZORLUK", "KAYNAK", "KELİME", "SÜTUN", "ALAN");
        for (kat, zor, yol) in &liste {
            let o = olc(kat, zor, yol);
            if o.kaynak != "YOK" { okundu += 1; }
            if o.sutun_taninan > 0 { sutunlu += 1; }
            alan_dogru += o.alan_dogru;
            alan_toplam += o.alan_beklenen;
            if o.alan_beklenen > 0 && o.alan_dogru == o.alan_beklenen { tam_isabet += 1; }
            println!("│ {:<28} {:<10} {:<8} {:>6} {:>3}/{:<3} {:>4}/{:<4}",
                o.dosya.chars().take(28).collect::<String>(), o.zorluk, o.kaynak,
                o.kelime, o.sutun_taninan, o.sutun_toplam, o.alan_dogru, o.alan_beklenen);
            for n in &o.notlar { println!("│     · {n}"); }
        }
        println!("├──────────────────────────────────────────────────────────");
        println!("│ OKUNDU        {okundu}/{}", liste.len());
        println!("│ SÜTUN TANINDI {sutunlu}/{}", liste.len());
        println!("│ TAM İSABET    {tam_isabet}/{}", liste.len());
        if alan_toplam > 0 {
            println!("│ ALAN İSABETİ  {alan_dogru}/{alan_toplam} (%{:.1})",
                alan_dogru as f64 * 100.0 / alan_toplam as f64);
        }
        println!("╰──────────────────────────────────────────────────────────\n");
    }
}
