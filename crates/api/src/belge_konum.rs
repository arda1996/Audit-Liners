//! KONUM MOTORU — PDF'i düz metin değil, KOORDİNATLI KELİMELER olarak okur.
//!
//! ## Neden gerekti
//!
//! `pdftotext -layout` sayfayı düz metne çeviriyor ve şunları ATIYOR: kelimelerin konumu,
//! kutu yüksekliği (≈ font boyutu), sütun hizası. Belgeler bu bilgi olmadan okunamıyor
//! çünkü her belgede her unsur farklı font ve boyutta:
//!
//! ```text
//! FATURA            ← 18pt başlık
//! Sayın                                    ← 9pt etiket
//!   EGE TEKSTİL A.Ş.                       ← 11pt değer, etiketin ALTINDA ve SAĞINDA
//! Matrah                        8.000,00   ← etiket solda, değer 300pt sağda
//! ```
//!
//! Düz metinde "Matrah" ile "8.000,00" arasındaki ilişki yalnız boşluk sayısıyla tahmin
//! edilir ve sütun genişliği değişince kopar. Koordinatla ise ilişki KESİNDİR: değer,
//! etiketle aynı yatay bantta ve sağındadır.
//!
//! ## Kaynak
//!
//! Yöntem, daha önce yazılan banka ekstresi ayrıştırıcısından geliyor
//! (`takipsistemiv2/pdfparseyöntemi.md`, "PDF Tablosu Okuma — bbox Bazlı"). Oradaki
//! çekirdek tespit hâlâ geçerli: **hazır tablo çıkarımı boşluk yutar, kelime kutularıyla
//! çalışmak gerekir.** O proje `pdfplumber` (Python) kullanıyordu; burada aynı bilgi
//! `pdftotext -bbox-layout` ile alınıyor — zaten kurulu, ek bağımlılık yok.
//!
//! Devralınmayan kısım: banka bazlı elle yazılmış ayrıştırıcılar. Onların yerine bu projede
//! **denklemle doğrulama** ve **şablon öğrenme** var; her belge türü için elle kural yazmak
//! ölçeklenmiyordu.

use std::process::Command;

#[derive(Debug, Clone)]
pub struct Kelime {
    pub metin: String,
    pub x0: f32, pub y0: f32, pub x1: f32, pub y1: f32,
    pub sayfa: usize,
}

impl Kelime {
    /// Kutu yüksekliği ≈ font boyutu. Başlık ile değeri ayırt etmeye yeter;
    /// gerçek punto bilgisi `pdftotext` çıktısında yok, ama oran korunuyor.
    pub fn yukseklik(&self) -> f32 { (self.y1 - self.y0).abs() }
    pub fn orta_y(&self) -> f32 { (self.y0 + self.y1) / 2.0 }
}

/// XHTML varlıklarını çöz. Kelime metni PDF'ten geldiği için `&amp;` vb. kaçışlı olabilir.
fn varlik_coz(s: &str) -> String {
    s.replace("&amp;", "&").replace("&lt;", "<").replace("&gt;", ">")
     .replace("&quot;", "\"").replace("&apos;", "'").replace("&#39;", "'")
}

fn oznitelik(etiket: &str, ad: &str) -> Option<f32> {
    let anahtar = format!("{ad}=\"");
    let p = etiket.find(&anahtar)? + anahtar.len();
    let son = etiket[p..].find('"')? + p;
    etiket[p..son].parse().ok()
}

/// PDF'ten koordinatlı kelimeleri çıkarır. Araç yoksa ya da PDF taranmışsa boş döner —
/// bu bir hata değil, "konum bilgisi yok" demektir ve çağıran metin yoluna düşer.
pub fn kelimeler(pdf_yolu: &std::path::Path) -> Vec<Kelime> {
    let Some(cikti) = Command::new("pdftotext").arg("-bbox-layout").arg(pdf_yolu).arg("-")
        .output().ok().filter(|o| o.status.success()) else { return Vec::new() };
    ayristir(&String::from_utf8_lossy(&cikti.stdout))
}

/// GÖRÜNTÜDEN koordinatlı kelimeler — taranmış belge / fotoğraf için.
///
/// Metin katmanı olmayan belgede tek yol budur. Çıktı, `kelimeler()` ile AYNI biçimdedir;
/// boru hattının geri kalanı belgenin taranmış mı dijital mi olduğunu BİLMEZ ve bilmesi
/// de gerekmez — satırlaştırma, sütun tespiti, denklem doğrulaması, şablon öğrenme
/// hepsi değişmeden çalışır.
///
/// Okuma modeli kurulu değilse boş döner (hata değil): çağıran kullanıcıya "görüntü
/// okunamadı, elle gir" der. Sessizce yanlış okumaktansa hiç okumamak doğrudur.
pub fn goruntuden_kelimeler(gorsel_yolu: &std::path::Path) -> Vec<Kelime> {
    let betik = std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../tools"))
        .join("goruntu-oku.py");
    let python = std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../tools"))
        .join("belge-venv/bin/python");
    if !betik.exists() { return Vec::new(); }
    let calistir = if python.exists() { python } else { std::path::PathBuf::from("python3") };

    let Some(cikti) = Command::new(calistir).arg(&betik).arg(gorsel_yolu).output().ok() else {
        return Vec::new();
    };
    if !cikti.status.success() { return Vec::new(); }

    String::from_utf8_lossy(&cikti.stdout).lines().filter_map(|satir| {
        let v: serde_json::Value = serde_json::from_str(satir).ok()?;
        Some(Kelime {
            metin: v["metin"].as_str()?.to_string(),
            x0: v["x0"].as_f64()? as f32, y0: v["y0"].as_f64()? as f32,
            x1: v["x1"].as_f64()? as f32, y1: v["y1"].as_f64()? as f32,
            sayfa: v["sayfa"].as_u64().unwrap_or(1) as usize,
        })
    }).collect()
}

/// `-bbox-layout` XHTML çıktısını ayrıştırır. Tam bir XML çözücüye gerek yok:
/// biçim sabit ve yalnız `<page` ile `<word ...>metin</word>` gerekiyor.
pub fn ayristir(xhtml: &str) -> Vec<Kelime> {
    let mut cikti = Vec::new();
    let mut sayfa = 0usize;
    let mut kalan = xhtml;
    loop {
        let Some(p) = kalan.find('<') else { break };
        kalan = &kalan[p..];
        if kalan.starts_with("<page") { sayfa += 1; kalan = &kalan[5..]; continue; }
        if !kalan.starts_with("<word") { kalan = &kalan[1..]; continue; }
        let Some(kapanis) = kalan.find('>') else { break };
        let etiket = &kalan[..kapanis];
        let govde = &kalan[kapanis + 1..];
        let Some(son) = govde.find("</word>") else { break };
        let metin = varlik_coz(&govde[..son]);
        if let (Some(x0), Some(y0), Some(x1), Some(y1)) = (
            oznitelik(etiket, "xMin"), oznitelik(etiket, "yMin"),
            oznitelik(etiket, "xMax"), oznitelik(etiket, "yMax"))
        {
            if !metin.trim().is_empty() {
                cikti.push(Kelime { metin, x0, y0, x1, y1, sayfa: sayfa.max(1) });
            }
        }
        kalan = &govde[son + 7..];
    }
    cikti
}

// ─────────────────────────────────────────────────────────────────────────────
// SATIRLAŞTIRMA — aynı yatay banttaki kelimeler
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Satir { pub kelimeler: Vec<Kelime>, pub y: f32, pub sayfa: usize }

impl Satir {
    /// Satırı metne çevirir. Sütun boşluğu KORUNUR: kelimeler arasında belirgin bir
    /// yatay boşluk varsa çok boşluk konur. Böylece "Matrah" ile "8.000,00" düz metinde de
    /// ayrı sütun olarak görünür — düz `-layout` çıktısında bu bilgi keyfîdir.
    pub fn metin(&self) -> String {
        let mut o = String::new();
        let mut onceki_x1: Option<f32> = None;
        for k in &self.kelimeler {
            if let Some(x1) = onceki_x1 {
                let bosluk = k.x0 - x1;
                // Ortalama karakter genişliğinin ~1.5 katından büyük boşluk = sütun sınırı.
                let esik = k.yukseklik() * 0.9;
                if bosluk > esik { o.push_str("   "); } else if bosluk > 0.5 { o.push(' '); }
            }
            o.push_str(&k.metin);
            onceki_x1 = Some(k.x1);
        }
        o
    }
}

/// Kelimeleri yatay bantlara böler. Bant toleransı kelime yüksekliğinden türer —
/// sabit bir piksel değeri, farklı punto karışımında (başlık + gövde) yanlış gruplar.
pub fn satirlar(kelimeler: &[Kelime]) -> Vec<Satir> {
    let mut sirali: Vec<Kelime> = kelimeler.to_vec();
    sirali.sort_by(|a, b| (a.sayfa, a.orta_y() as i32, a.x0 as i32)
        .partial_cmp(&(b.sayfa, b.orta_y() as i32, b.x0 as i32)).unwrap());

    let mut cikti: Vec<Satir> = Vec::new();
    for k in sirali {
        let tol = (k.yukseklik() * 0.6).max(2.0);
        match cikti.last_mut() {
            Some(s) if s.sayfa == k.sayfa && (s.y - k.orta_y()).abs() <= tol => {
                s.kelimeler.push(k);
                // Bandın y'si üyelerin ortalaması — tek bir kelimeye çakılı kalmasın.
                let n = s.kelimeler.len() as f32;
                s.y = s.kelimeler.iter().map(|w| w.orta_y()).sum::<f32>() / n;
            }
            _ => cikti.push(Satir { y: k.orta_y(), sayfa: k.sayfa, kelimeler: vec![k] }),
        }
    }
    for s in cikti.iter_mut() { s.kelimeler.sort_by(|a, b| a.x0.partial_cmp(&b.x0).unwrap()); }
    cikti
}

/// Konum bilgisinden, sütun boşlukları korunmuş düz metin üretir.
/// Mevcut metin tabanlı boru hattı bunu doğrudan tüketebilir.
pub fn konumdan_metin(kelimeler: &[Kelime]) -> String {
    satirlar(kelimeler).iter().map(|s| s.metin()).collect::<Vec<_>>().join("\n")
}

// ─────────────────────────────────────────────────────────────────────────────
// SÜTUN TESPİTİ — dikey boşluk koridorları
// ─────────────────────────────────────────────────────────────────────────────

/// Sayfadaki DİKEY BOŞLUK KORİDORLARINI bulur — sütun sınırları bunlardır.
///
/// ## Neden şablonsuz
///
/// Sütun sınırını şablona bağlamak kırılgandır: her belgede sütun sayısı, genişliği ve
/// sırası değişir. Oysa **hiçbir kelimenin uğramadığı dikey şerit** geometrik bir olgudur
/// ve her belgede vardır. Koridoru bulmak için belgeyi TANIMAK gerekmez.
///
/// Şablon, sütunun NEREDE olduğunu değil NE OLDUĞUNU (miktar mı, tutar mı) saklamalıdır —
/// o bilgi öğrenme katmanının işidir.
///
/// ## Yöntem
///
/// Tüm kelimelerin x aralıkları sayfaya izdüşürülür; hiç dolmayan aralıklardan yeterince
/// geniş olanlar koridordur. Eşik, kelime yüksekliğinin katı olarak alınır — sabit piksel
/// değeri farklı çözünürlükte (150 dpi tarama ile 72 dpi PDF) yanlış sonuç verir.
pub fn sutun_koridorlari(kelimeler: &[Kelime]) -> Vec<f32> {
    if kelimeler.len() < 4 { return Vec::new(); }
    let mut yuk: Vec<f32> = kelimeler.iter().map(|k| k.yukseklik()).collect();
    yuk.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let orta_yuk = yuk[yuk.len() / 2].max(1.0);
    let esik = orta_yuk * 1.2; // bundan geniş boşluk sütun sınırıdır

    // x aralıklarını birleştir (dolu şeritler).
    let mut aralik: Vec<(f32, f32)> = kelimeler.iter().map(|k| (k.x0, k.x1)).collect();
    aralik.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    let mut dolu: Vec<(f32, f32)> = Vec::new();
    for (a, b) in aralik {
        match dolu.last_mut() {
            Some(son) if a <= son.1 => { if b > son.1 { son.1 = b; } }
            _ => dolu.push((a, b)),
        }
    }
    // Dolu şeritler arasındaki geniş boşluklar koridordur; ORTASI sınır sayılır.
    dolu.windows(2)
        .filter(|w| w[1].0 - w[0].1 >= esik)
        .map(|w| (w[0].1 + w[1].0) / 2.0)
        .collect()
}

/// Satırı, sayfa genelindeki koridorlara göre HÜCRELERE böler.
///
/// Bir koridor, o satırda gerçekten boşsa böler; kelime üstünden geçiyorsa bölmez.
/// Bu yüzden aynı sayfada hem 6 sütunlu kalem tablosu hem 2 sütunlu başlık bloğu
/// doğru bölünür — belgenin tek tip bir ızgara olduğunu VARSAYMAYIZ.
pub fn hucreler(satir: &Satir, koridorlar: &[f32]) -> Vec<String> {
    if satir.kelimeler.is_empty() { return Vec::new(); }
    let mut cikti: Vec<String> = Vec::new();
    let mut gecerli: Vec<&str> = Vec::new();
    for (i, k) in satir.kelimeler.iter().enumerate() {
        if i > 0 {
            let onceki_x1 = satir.kelimeler[i - 1].x1;
            // Bu iki kelime arasından geçen bir koridor var mı?
            if koridorlar.iter().any(|c| *c > onceki_x1 && *c < k.x0) {
                if !gecerli.is_empty() { cikti.push(gecerli.join(" ")); gecerli.clear(); }
            }
        }
        gecerli.push(&k.metin);
    }
    if !gecerli.is_empty() { cikti.push(gecerli.join(" ")); }
    cikti
}

/// Sayfayı HÜCRE IZGARASINA çevirir: her satır, hücre listesi.
///
/// **Koridorlar SAYFA BAŞINA hesaplanır.** Eskiden tüm sayfaların kelimeleri birlikte
/// izdüşürülüyordu; `satirlar()` sayfayı ayırıyor ama `sutun_koridorlari()` ayırmıyordu.
/// Sonuç: 1. sayfası iki sütunlu başlık bloğu, 2. sayfası tam genişlik kalem tablosu olan
/// bir belgede, 2. sayfanın geniş satırları 1. sayfanın boşluk koridorunu DOLDURUYOR ve
/// koridor listesi boşalıyordu. Izgara tek hücreye düşüyor, yan sütun değere karışıyordu —
/// yani `yan_sutun_degere_karismaz` testinin koruduğu davranış çok sayfalı belgede
/// sessizce kayboluyordu.
pub fn izgara(kelimeler: &[Kelime]) -> Vec<Vec<String>> {
    let mut sayfalar: Vec<usize> = kelimeler.iter().map(|k| k.sayfa).collect();
    sayfalar.sort_unstable();
    sayfalar.dedup();

    let mut cikti = Vec::new();
    for s in sayfalar {
        let sayfa_kelimeleri: Vec<Kelime> =
            kelimeler.iter().filter(|k| k.sayfa == s).cloned().collect();
        let koridorlar = sutun_koridorlari(&sayfa_kelimeleri);
        cikti.extend(satirlar(&sayfa_kelimeleri).iter().map(|r| hucreler(r, &koridorlar)));
    }
    cikti
}

/// Hücre sınırları KORUNARAK metin. Hücreler arasına ayraç konur ki aşağı akıştaki
/// çıkarım "sütunun sağındaki" ile "satırın geri kalanı"nı karıştırmasın.
///
/// Eski `konumdan_metin` yalnız boşluk genişliğine bakıyordu ve sütun sınırı satırdan
/// satıra kayıyordu; "Ayşe YILMAZ" ile yanındaki "İrsaliye Tarihi" sütunu tek değere
/// yapışıyordu. Koridorlar sayfa genelinde SABİT olduğu için bu kayma ortadan kalkar.
pub fn izgaradan_metin(kelimeler: &[Kelime]) -> String {
    izgara(kelimeler).iter()
        .map(|h| h.join("   "))
        .collect::<Vec<_>>().join("\n")
}

// ─────────────────────────────────────────────────────────────────────────────
// GEOMETRİK ETİKET → DEĞER
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct KonumAday {
    pub etiket: String,
    pub deger: String,
    /// SAGINDA · ALTINDA — hangi geometrik ilişkiyle bulundu.
    pub iliski: String,
    /// Etikete uzaklık (punto). Küçük olan tercih edilir.
    pub uzaklik: f32,
}

/// Etiketin sağındaki ve altındaki metni geometriyle bulur.
///
/// Metin tabanlı aramanın çözemediği şeyi çözer: değer etiketten 300 punto uzakta olabilir,
/// arada başka sütun bulunabilir, font boyutu farklı olabilir — ilişki yine de kesindir.
///
/// **Font boyutu süzgeci:** değerin kutu yüksekliği, etiketinkinin 2 katından büyükse
/// bu bir başlıktır, değer değildir. ("FATURA" başlığı, "Sayın" etiketinin değeri sanılmasın.)
pub fn etiket_degerleri(kelimeler: &[Kelime], etiket_sade: &str) -> Vec<KonumAday> {
    let satirlar = satirlar(kelimeler);
    let mut cikti = Vec::new();

    for (i, s) in satirlar.iter().enumerate() {
        // Etiket bu satırda hangi kelimelerde başlıyor?
        let sade = crate::belge_oku::sadelestir(&s.metin());
        if !sade.contains(etiket_sade) { continue; }

        // Etiketin son kelimesini bul — değer ondan SONRA gelir.
        let etiket_kelimeleri: Vec<&Kelime> = s.kelimeler.iter()
            .filter(|k| etiket_sade.contains(&crate::belge_oku::sadelestir(&k.metin)))
            .collect();
        let Some(son) = etiket_kelimeleri.last() else { continue };
        let etiket_yuk = son.yukseklik();

        // 1) SAĞINDA — aynı bantta, etiketin sağındaki kelimeler.
        let sag: Vec<&Kelime> = s.kelimeler.iter()
            .filter(|k| k.x0 > son.x1 + 1.0 && k.yukseklik() <= etiket_yuk * 2.0)
            .collect();
        if !sag.is_empty() {
            let deger = sag.iter().map(|k| k.metin.as_str()).collect::<Vec<_>>().join(" ");
            cikti.push(KonumAday { etiket: etiket_sade.into(), deger,
                iliski: "SAGINDA".into(), uzaklik: sag[0].x0 - son.x1 });
        }

        // 2) ALTINDA — bir sonraki bantta, etiketle YATAY OLARAK ÖRTÜŞEN kelimeler.
        //    Örtüşme şartı önemli: aynı satırdaki başka sütunun değeri alınmasın.
        if let Some(alt) = satirlar.get(i + 1) {
            let sol = son.x0 - etiket_yuk * 2.0;
            let sag_sinir = son.x1 + etiket_yuk * 12.0;
            let ortusen: Vec<&Kelime> = alt.kelimeler.iter()
                .filter(|k| k.x1 > sol && k.x0 < sag_sinir && k.yukseklik() <= etiket_yuk * 2.0)
                .collect();
            if !ortusen.is_empty() {
                let deger = ortusen.iter().map(|k| k.metin.as_str()).collect::<Vec<_>>().join(" ");
                cikti.push(KonumAday { etiket: etiket_sade.into(), deger,
                    iliski: "ALTINDA".into(), uzaklik: (alt.y - s.y).abs() });
            }
        }
    }
    cikti.sort_by(|a, b| a.uzaklik.partial_cmp(&b.uzaklik).unwrap_or(std::cmp::Ordering::Equal));
    cikti
}

#[cfg(test)]
mod testler {
    use super::*;

    /// `pdftotext -bbox-layout` çıktısının biçimi — gerçek çıktıdan alınmıştır.
    const XHTML: &str = r#"<doc>
  <page width="595" height="842">
    <flow><block><line>
      <word xMin="60.0" yMin="70.0" xMax="140.0" yMax="88.0">FATURA</word>
    </line></block>
    <flow><block><line>
      <word xMin="60.0" yMin="110.0" xMax="92.0" yMax="120.0">Sayın</word>
    </line></block>
    <flow><block><line>
      <word xMin="60.0" yMin="126.0" xMax="130.0" yMax="136.0">EGE</word>
      <word xMin="133.0" yMin="126.0" xMax="200.0" yMax="136.0">TEKSTİL</word>
    </line></block>
    <flow><block><line>
      <word xMin="60.0" yMin="160.0" xMax="100.0" yMax="170.0">Matrah</word>
      <word xMin="420.0" yMin="160.0" xMax="480.0" yMax="170.0">8.000,00</word>
    </line></block>
  </page></doc>"#;

    #[test]
    fn kelimeler_koordinatlariyla_ayristirilir() {
        let k = ayristir(XHTML);
        assert_eq!(k.len(), 6, "kelime sayısı yanlış: {:?}", k.iter().map(|x| &x.metin).collect::<Vec<_>>());
        assert_eq!(k[0].metin, "FATURA");
        assert_eq!(k[0].sayfa, 1);
        // Başlık gövdeden BÜYÜK — font boyutu ayrımı kutu yüksekliğinden geliyor.
        assert!(k[0].yukseklik() > k[1].yukseklik(), "başlık yüksekliği ayırt edilemedi");
    }

    #[test]
    fn turkce_karakter_bozulmadan_gelir() {
        // Türkçe upper() tuzağı (önceki projede yakalanmış): ham yazım KORUNMALI.
        let k = ayristir(XHTML);
        assert!(k.iter().any(|x| x.metin == "TEKSTİL"), "İ harfi bozuldu");
        assert!(k.iter().any(|x| x.metin == "Sayın"), "ı harfi bozuldu");
    }

    #[test]
    fn sutun_bosluğu_metinde_korunur() {
        // "Matrah" ile "8.000,00" arası 320 punto — düz metinde de sütun olarak görünmeli.
        let s = konumdan_metin(&ayristir(XHTML));
        let satir = s.lines().find(|l| l.contains("Matrah")).expect("matrah satırı yok");
        assert!(satir.contains("   "), "sütun boşluğu kayboldu: {satir:?}");
        assert!(satir.contains("8.000,00"));
    }

    #[test]
    fn deger_saginda_bulunur() {
        let a = etiket_degerleri(&ayristir(XHTML), "matrah");
        let sag = a.iter().find(|x| x.iliski == "SAGINDA").expect("sağdaki değer bulunamadı");
        assert_eq!(sag.deger, "8.000,00");
    }

    #[test]
    fn deger_altinda_bulunur() {
        // "Sayın" kendi satırında; ünvan ALTINDA ve yatayda örtüşüyor.
        let a = etiket_degerleri(&ayristir(XHTML), "sayin");
        let alt = a.iter().find(|x| x.iliski == "ALTINDA").expect("alttaki değer bulunamadı");
        assert!(alt.deger.contains("EGE") && alt.deger.contains("TEKSTİL"),
            "alt satırdaki ünvan eksik: {}", alt.deger);
    }

    #[test]
    fn baslik_deger_sanilmaz() {
        // "FATURA" 18pt başlık; 10pt bir etiketin değeri olarak DÖNMEMELİ.
        // (Etiket 'sayin' 10pt; başlık onun 1.8 katı ama ÜSTÜNDE, o yüzden zaten aday değil.
        //  Buradaki sınama: font süzgeci çalışıyor mu.)
        let k = ayristir(XHTML);
        let baslik = k.iter().find(|x| x.metin == "FATURA").unwrap();
        let etiket = k.iter().find(|x| x.metin == "Sayın").unwrap();
        assert!(baslik.yukseklik() > etiket.yukseklik() * 1.5,
            "başlık/gövde oranı ayırt edilemiyor — font süzgeci anlamsız kalır");
    }

    #[test]
    fn satirlar_farkli_puntoyu_karistirmaz() {
        // Başlık (18pt) ile onun 40 punto altındaki etiket (10pt) AYNI satır sayılmamalı.
        let s = satirlar(&ayristir(XHTML));
        assert!(s.len() >= 4, "farklı bantlar birleşti: {} satır", s.len());
        assert!(s[0].kelimeler.iter().all(|k| k.metin == "FATURA"));
    }
}

#[cfg(test)]
mod izgara_testleri {
    use super::*;

    fn k(m: &str, x0: f32, y: f32, w: f32) -> Kelime {
        Kelime { metin: m.into(), x0, y0: y, x1: x0 + w, y1: y + 10.0, sayfa: 1 }
    }

    /// ÖRNEK TEKSTİL faturasının başlık bloğu: solda alıcı, sağda irsaliye bilgisi.
    /// Aralarında geniş bir boşluk koridoru var; düz metinde bu bilgi kayboluyordu.
    fn belge() -> Vec<Kelime> {
        vec![
            // sol sütun                                    sağ sütun (x=330'dan sonra)
            k("SAYIN:", 48.0, 130.0, 34.0), k("Ayşe", 86.0, 130.0, 32.0), k("YILMAZ", 122.0, 130.0, 30.0),
            k("İrsaliye", 330.0, 130.0, 38.0), k("Tarihi", 372.0, 130.0, 30.0), k("10.02.2006", 410.0, 130.0, 50.0),
            k("Örnek", 48.0, 145.0, 42.0), k("Caddesi", 94.0, 145.0, 38.0),
            k("İrsaliye", 330.0, 145.0, 38.0), k("Nu", 372.0, 145.0, 14.0), k("45678", 390.0, 145.0, 28.0),
            // kalem tablosu — daha çok sütun
            k("ÇORAP", 45.0, 220.0, 34.0), k("ADET", 165.0, 220.0, 26.0), k("200", 240.0, 220.0, 18.0),
            k("2,50", 305.0, 220.0, 22.0), k("8", 385.0, 220.0, 6.0), k("500,00", 470.0, 220.0, 34.0),
        ]
    }

    #[test]
    fn sutun_koridorlari_bulunur() {
        let c = sutun_koridorlari(&belge());
        assert!(!c.is_empty(), "hiç koridor bulunamadı");
        // Sol blok x≈152'de bitiyor, sağ blok x=330'da başlıyor → arada koridor olmalı.
        assert!(c.iter().any(|x| *x > 152.0 && *x < 330.0),
            "sol/sağ blok arasındaki koridor bulunamadı: {c:?}");
    }

    #[test]
    fn yan_sutun_degere_karismaz() {
        // ASIL HATA BUYDU: "Ayşe YILMAZ" değeri, yanındaki "İrsaliye Tarihi" sütununu
        // da yutup fişe "Ayşe YILMAZ   irsaliye Ta" olarak yazılıyordu.
        let kel = belge();
        let c = sutun_koridorlari(&kel);
        let s = satirlar(&kel);
        let h = hucreler(&s[0], &c);
        assert!(h.len() >= 2, "satır hücrelere bölünmedi: {h:?}");
        assert!(h[0].contains("Ayşe YILMAZ"), "sol hücre yanlış: {:?}", h[0]);
        assert!(!h[0].contains("İrsaliye"), "yan sütun sol hücreye karıştı: {:?}", h[0]);
    }

    #[test]
    fn ayni_sayfada_farkli_sutun_sayisi_calisir() {
        // Başlık bloğu 2 sütun, kalem tablosu 6 sütun — aynı sayfada. Belgenin tek tip
        // ızgara olduğunu varsayan bir çözüm burada çöker.
        let g = izgara(&belge());
        let baslik = g.iter().find(|r| r.iter().any(|c| c.contains("Ayşe"))).unwrap();
        let kalem = g.iter().find(|r| r.iter().any(|c| c.contains("ÇORAP"))).unwrap();
        assert!(kalem.len() > baslik.len(),
            "kalem satırı başlıktan daha çok hücreye bölünmeliydi: kalem={kalem:?} başlık={baslik:?}");
        assert!(kalem.len() >= 4, "kalem tablosu yeterince bölünmedi: {kalem:?}");
    }

    /// **B-06 — koridorlar SAYFA BAŞINA hesaplanır.**
    /// 1. sayfa iki sütunlu başlık, 2. sayfa tam genişlik metin. Koridorlar tüm sayfalardan
    /// birlikte hesaplanınca 2. sayfanın geniş satırı 1. sayfanın boşluğunu dolduruyor ve
    /// koridor kayboluyordu — ızgara tek hücreye düşüp yan sütun değere karışıyordu.
    #[test]
    fn koridorlar_sayfa_basina_hesaplanir() {
        let w = |m: &str, x0: f32, y: f32, g: f32, sf: usize| Kelime {
            metin: m.into(), x0, y0: y, x1: x0 + g, y1: y + 10.0, sayfa: sf,
        };
        let kelimeler = vec![
            // 1. sayfa: sol sütun x<200, sağ sütun x>330 — arada geniş koridor
            w("SAYIN:", 48.0, 130.0, 34.0, 1), w("Ayse", 86.0, 130.0, 30.0, 1),
            w("Irsaliye", 330.0, 130.0, 38.0, 1), w("10.02.2006", 372.0, 130.0, 50.0, 1),
            // 2. sayfa: TAM GENİŞLİK tek satır — 1. sayfanın koridorunu kaplıyor
            w("ACIKLAMA", 45.0, 90.0, 60.0, 2), w("ORTA", 200.0, 90.0, 40.0, 2),
            w("METIN", 260.0, 90.0, 40.0, 2), w("SAGDA", 350.0, 90.0, 40.0, 2),
        ];
        let g = izgara(&kelimeler);
        let baslik = g.iter().find(|r| r.iter().any(|c| c.contains("Ayse")))
            .expect("1. sayfa başlık satırı yok");
        assert!(baslik.len() >= 2,
            "1. sayfanın sütun koridoru 2. sayfa yüzünden kayboldu — satır tek hücreye düştü: {baslik:?}");
        assert!(baslik[0].contains("Ayse") && !baslik[0].contains("Irsaliye"),
            "yan sütun sol hücreye karıştı: {:?}", baslik[0]);
    }

    #[test]
    fn koridor_kelime_ustunden_gecerse_bolmez() {
        // Bir koridor, bazı satırlarda dolu olabilir. O satır bölünmemeli.
        let mut kel = belge();
        kel.push(k("UZUNBIRBASLIKSATIRI", 45.0, 180.0, 470.0)); // tüm genişliği kaplıyor
        let c = sutun_koridorlari(&kel);
        let s = satirlar(&kel);
        let uzun = s.iter().find(|x| x.kelimeler.iter().any(|w| w.metin.starts_with("UZUN"))).unwrap();
        assert_eq!(hucreler(uzun, &c).len(), 1, "dolu satır yine de bölündü");
    }
}
