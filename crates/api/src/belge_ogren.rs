//! YAŞAYAN OKUMA KATMANI — çok stratejili çıkarım + kullanımdan öğrenen şablon belleği.
//!
//! ## Neden tek kural yetmiyor
//!
//! İlk tarayıcı tek bir düzen varsayıyordu: `Etiket: değer`. Gerçek belgeler öyle değil.
//! Bir e-Fatura çıktısında aynı anda şunlar bulunur:
//!
//! ```text
//! SAYIN                                     ← etiket kendi satırında
//! EGE TEKSTİL SANAYİ A.Ş.                   ← değer ALT satırda
//!
//! Fatura Tarihi                12.04.2026   ← değer aynı satırda ama SAĞA YASLI, iki nokta yok
//! Hesaplanan KDV %20            1.600,00    ← satırda İKİ sayı var; yanlışı alırsan 201.600,00 olur
//! Atatürk Bulvarı No:12                     ← "No:" adreste de geçer; ilk eşleşme yanlış alan
//! ```
//!
//! Bu yüzden katman üç şey yapar:
//!   1. **Çok strateji** — aynı satır sonrası · satırın sağ ucu · alt satır · sütun bandı.
//!   2. **Tipli ayıklama** — TUTAR alanında satırın tamamı değil, PARA BİÇİMLİ belirteç alınır.
//!      "%20" ile "1.600,00" böylece birbirine yapışmaz.
//!   3. **Öğrenme** — kullanıcı bir değeri onayladığında, o değerin belgede NEREDE bulunduğu
//!      ve hangi stratejiyle çıkarıldığı, belgenin **parmak izine** bağlı olarak saklanır.
//!      Aynı düzendeki bir sonraki belge doğrudan oradan okunur.
//!
//! ## "Yaşayan" ne demek
//!
//! Her mükellefin tedarikçileri bellidir ve her tedarikçinin fatura şablonu sabittir.
//! Muhasebeci bir kez düzelttiğinde sistem o şablonu öğrenir; ikinci faturada elle iş kalmaz.
//! Kural sayısı zamanla artar — ama **güvence artmaz, aynı kalır**: öğrenilmiş kural da
//! denklemden geçmek zorundadır. Denklemi bozan kural uygulanmaz ve itibarını kaybeder.
//! Öğrenme, doğruluk ölçütünü gevşetmez; yalnız DOĞRU CEVABA daha çabuk ulaştırır.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// Öğrenilen şablonların saklandığı dosya. Bu katmanın kalıcı olması ŞART:
/// "yaşayan" olması, öğrendiğini sunucu yeniden başlayınca unutmamasına bağlı.
fn bellek_yolu() -> std::path::PathBuf {
    std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../data"))
        .join("ogrenilen-sablonlar.json")
}

// ─────────────────────────────────────────────────────────────────────────────
// TİPLİ DEĞER AYIKLAMA — satırın tamamını değil, doğru BİÇİMDEKİ belirteci al
// ─────────────────────────────────────────────────────────────────────────────

/// Metinden alan tipine uygun belirteçleri çıkarır (soldan sağa sırayla).
///
/// En kritik düzeltme burada: eskiden satırın tamamı sayıya çevriliyordu ve
/// `Hesaplanan KDV %20   1.600,00` satırı `201.600,00` oluyordu — oran tutara yapışıyordu.
/// Artık satırdaki her para biçimli belirteç ayrı bir adaydır.
pub fn belirtecler(tip: &str, metin: &str) -> Vec<String> {
    let mut cikti = Vec::new();
    match tip {
        "TUTAR" => {
            // Para biçimi: rakamla başlar, içinde binlik/ondalık ayracı olabilir.
            // Yüzde işaretine bitişik sayı (oran) DIŞLANIR — o bir tutar değildir.
            let h: Vec<char> = metin.chars().collect();
            let mut i = 0;
            while i < h.len() {
                if !h[i].is_ascii_digit() { i += 1; continue; }
                let bas = i;
                while i < h.len() && (h[i].is_ascii_digit() || h[i] == '.' || h[i] == ',') { i += 1; }
                let mut son = i;
                // Sondaki ayraçlar belirtecin parçası değildir ("1.600," → "1.600").
                while son > bas && (h[son - 1] == '.' || h[son - 1] == ',') { son -= 1; }
                // Yüzde işareti sayıya bitişik olmayabilir: "KDV % 8" da bir orandır.
                // Sayıdan geriye doğru yalnız boşluk atlanarak bakılır.
                let onceki_yuzde = {
                    let mut p = bas;
                    while p > 0 && h[p - 1] == ' ' { p -= 1; }
                    p > 0 && h[p - 1] == '%'
                };
                let sonraki_yuzde = son < h.len() && h[son] == '%';
                let s: String = h[bas..son].iter().collect();
                if !onceki_yuzde && !sonraki_yuzde && !s.is_empty() { cikti.push(s); }
            }
        }
        "TARIH" => {
            // gg.aa.yyyy · gg/aa/yyyy · gg-aa-yyyy
            let h: Vec<char> = metin.chars().collect();
            let mut i = 0;
            while i < h.len() {
                if !h[i].is_ascii_digit() { i += 1; continue; }
                let bas = i;
                while i < h.len() && (h[i].is_ascii_digit() || h[i] == '.' || h[i] == '/' || h[i] == '-') { i += 1; }
                let s: String = h[bas..i].iter().collect();
                if s.chars().filter(|c| c.is_ascii_digit()).count() == 8 { cikti.push(s); }
            }
        }
        "VKN" => {
            for p in metin.split(|c: char| !c.is_ascii_digit()) {
                if p.len() == 10 || p.len() == 11 { cikti.push(p.to_string()); }
            }
        }
        _ => {
            let t = metin.trim();
            if !t.is_empty() { cikti.push(t.to_string()); }
        }
    }
    cikti
}

// ─────────────────────────────────────────────────────────────────────────────
// ÇIKARIM STRATEJİLERİ
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Strateji {
    /// `Matrah: 1.000,00` — iki nokta sonrası.
    IkiNoktaSonrasi,
    /// `Fatura Tarihi          12.04.2026` — etiketten SONRAKİ kısımda, tabloların olağan hali.
    SatirSagi,
    /// `SAYIN` / `EGE TEKSTİL A.Ş.` — etiket kendi satırında, değer ALT satırda.
    AltSatir,
    /// Etiket satırının 1-3 satır altında, aynı sütun bandında.
    SutunBandi,
}

impl Strateji {
    fn ad(&self) -> &'static str {
        match self {
            Strateji::IkiNoktaSonrasi => "iki nokta sonrası",
            Strateji::SatirSagi => "satırın sağı",
            Strateji::AltSatir => "alt satır",
            Strateji::SutunBandi => "sütun bandı",
        }
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct Aday {
    pub deger: String,
    pub strateji: String,
    /// Etiketin bulunduğu satır numarası — öğrenme bunu saklar.
    pub satir: usize,
    /// Yüksek olan tercih edilir. Denklem doğrulaması bunu EZEBİLİR.
    pub puan: i32,
    pub gerekce: String,
}

/// Metnin İLK HÜCRESİ. Konum motoru hücreleri üç boşlukla ayırıyor; etiketin bulunduğu
/// hücreden sonraki ilk hücre değerdir. Satırın tamamını almak yan sütunu da yutuyordu
/// ("Ayşe YILMAZ   İrsaliye Tarihi : 10.02.2006" tek değer sanılıyordu).
fn ilk_hucre(s: &str) -> String {
    s.split("   ").find(|p| !p.trim().is_empty()).unwrap_or(s).trim().to_string()
}

/// Bir alan için belgedeki TÜM makul adayları çıkarır.
///
/// Tek bir "doğru" seçmez — seçimi çağırana bırakır, çünkü asıl hakem **denklemdir**:
/// hangi aday kombinasyonu belgenin kendi aritmetiğini sağlıyorsa doğru olan odur.
/// Tek aday seçip denkleme sokmak, ilk eşleşmenin yanlış olduğu durumda (adresteki "No:12")
/// tüm okumayı çöpe atıyordu.
pub fn adaylar(satirlar: &[&str], etiketler: &[String], tip: &str) -> Vec<Aday> {
    let mut cikti: Vec<Aday> = Vec::new();
    // Her satır HÜCRELERE bölünür. Konum motoru sütun sınırlarını üç boşlukla işaretliyor.
    let izgara: Vec<Vec<&str>> = satirlar.iter()
        .map(|s| s.split("   ").map(|h| h.trim()).filter(|h| !h.is_empty()).collect())
        .collect();

    for (i, hucreler) in izgara.iter().enumerate() {
        for (j, hucre) in hucreler.iter().enumerate() {
            let sade = crate::belge_oku::sadelestir(hucre);
            // Etiket BU HÜCREDE mi? En uzun eşleşen etiket kazanır ("fatura tarihi" > "tarih").
            let Some(etiket) = etiketler.iter()
                .filter(|e| !e.is_empty() && sade.contains(*e))
                .max_by_key(|e| e.chars().count()) else { continue };
            let belirginlik = etiket.chars().count() as i32;

            // Etiketten SONRA hücre içinde kalan kısım. Konum ham metin üzerinde bulunur;
            // oranla tahmin YAPILMAZ — sadeleştirme yalnız EŞLEŞME için, kesme için değil.
            let kalan = hucre_kuyrugu(hucre, etiket);

            // 1) DEĞER AYNI HÜCREDE — "Matrah: 1.000,00" ya da "Sıra 32450".
            if let Some(k) = &kalan {
                for (n, d) in belirtecler(tip, k).into_iter().enumerate() {
                    cikti.push(Aday { deger: d, strateji: Strateji::IkiNoktaSonrasi.ad().into(),
                        satir: i, puan: 100 + belirginlik - n as i32 * 5,
                        gerekce: format!("'{etiket}' etiketiyle AYNI hücrede, ardından") });
                }
            }

            // 2) DEĞER SONRAKİ HÜCREDE — tablo düzeninin olağan hali.
            //
            // Bu kural, birinci kural bir şey bulsa DA denenir. Eskiden "aynı hücrede değer
            // varsa sağa bakma" deniyordu ve "KDV % 8   256,00" satırında oran (%8) gerçek
            // tutarı (256,00) bastırıyordu. Doktrin gereği TÜM adaylar üretilir; hangisinin
            // doğru olduğuna denklem karar verir — erken eleme, denklemin işini elinden alır.
            {
                if let Some(sonraki) = hucreler.get(j + 1) {
                    for (n, d) in belirtecler(tip, sonraki).into_iter().enumerate() {
                        cikti.push(Aday { deger: d, strateji: Strateji::SatirSagi.ad().into(),
                            satir: i, puan: 95 + belirginlik - n as i32 * 5,
                            gerekce: format!("'{etiket}' etiketinin SAĞINDAKİ hücrede") });
                    }
                }
                // 3) DEĞER ALT SATIRDA — etiket kendi başına duruyorsa.
                // Aynı sütun konumundaki (j) hücre önce denenir; yoksa ilk hücre.
                for adim in 1..=2usize {
                    let Some(alt) = izgara.get(i + adim) else { break };
                    if alt.is_empty() { continue; }
                    let hedef = alt.get(j).or_else(|| alt.first());
                    if let Some(h) = hedef {
                        let strat = if adim == 1 { Strateji::AltSatir } else { Strateji::SutunBandi };
                        for d in belirtecler(tip, h) {
                            cikti.push(Aday { deger: d, strateji: strat.ad().into(),
                                satir: i, puan: 85 + belirginlik - (adim as i32 - 1) * 10,
                                gerekce: format!("'{etiket}' etiketinin {adim} satır altında, aynı sütunda") });
                        }
                    }
                    break;
                }
            }
        }
    }

    cikti.sort_by(|a, b| b.puan.cmp(&a.puan));
    cikti.dedup_by(|a, b| a.deger == b.deger && a.satir == b.satir);
    cikti
}

/// Hücre içinde etiketten SONRA kalan metin. Bulunamazsa None (etiket hücrenin tamamı).
///
/// Sadeleştirme yalnız EŞLEŞME içindir; kesme HAM metin üzerinde yapılır. Eskiden iki
/// dizgi arasında oranla konum tahmin ediliyordu ve kesme kayıyordu ("Sıra 32450" →
/// "a 32450"). Etiketin son sözcüğünü ham metinde aramak kesin sonuç verir.
fn hucre_kuyrugu(hucre: &str, etiket_sade: &str) -> Option<String> {
    let son_sozcuk = etiket_sade.split(' ').next_back()?;
    // Ham hücredeki sözcükleri dolaşıp sadeleştirilmiş hali eşleşeni bul.
    let mut yer = 0usize;
    for sozcuk in hucre.split_inclusive(|c: char| c == ' ' || c == ':') {
        let temiz = crate::belge_oku::sadelestir(sozcuk);
        yer += sozcuk.len();
        if !temiz.is_empty() && (temiz == son_sozcuk || temiz.ends_with(son_sozcuk)) {
            let kuyruk = hucre[yer.min(hucre.len())..]
                .trim_start_matches([':', ' ', '.', '-']).trim();
            return if kuyruk.is_empty() { None } else { Some(kuyruk.to_string()) };
        }
    }
    None
}

// ─────────────────────────────────────────────────────────────────────────────
// ŞABLON BELLEĞİ — öğrenilen kurallar
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Kural {
    pub alan: String,
    pub strateji: String,
    /// Etiketin bulunduğu satırın sadeleştirilmiş hali — konum yerine İÇERİK saklanır.
    /// Satır numarası saklamak kırılgandır: belgeye tek satır eklenince tüm kurallar kayar.
    pub capa: String,
    /// Değerden ÖNCE gelen metin (etiket). Uygulama sırasında kesilir.
    /// Bu olmadan METİN alanında satırın tamamı değer sanılıyordu:
    /// "Fatura No        ABC2026000888" → değer olarak satırın hepsi dönüyordu.
    #[serde(default)]
    pub onek: String,
    /// Kaç kez doğru çıktı / kaç kez denendi. Güvenilirliği buradan ölçülür.
    pub isabet: u32,
    pub deneme: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Sablon {
    pub id: String,
    pub ad: String,
    pub belge_turu: String,
    pub kurallar: Vec<Kural>,
    pub kullanim: u32,
}

#[derive(Serialize, Deserialize, Default)]
struct Bellek { sablonlar: HashMap<String, Sablon> }

static BELLEK: OnceLock<Mutex<Bellek>> = OnceLock::new();
fn bellek() -> &'static Mutex<Bellek> {
    BELLEK.get_or_init(|| {
        let b = std::fs::read_to_string(bellek_yolu()).ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        Mutex::new(b)
    })
}

fn kaydet(b: &Bellek) {
    if let Ok(s) = serde_json::to_string_pretty(b) {
        let _ = std::fs::write(bellek_yolu(), s);
    }
}

/// Belgenin PARMAK İZİ — aynı şablondan gelen belgeleri eşleştirmek için.
///
/// Tutarlar ve tarihler DIŞLANIR: onlar her faturada değişir. Kalan iskelet (etiket
/// sözcükleri, sıraları) şablona özgüdür. Aynı tedarikçinin ikinci faturası aynı izi verir.
pub fn parmak_izi(metin: &str) -> String {
    let mut iskelet: Vec<String> = Vec::new();
    for satir in metin.lines().take(60) {
        let s = crate::belge_oku::sadelestir(satir);
        // Rakamları at — değişken olan onlar. Kalan harf dizisi düzenin kimliğidir.
        let harfler: String = s.chars().filter(|c| c.is_alphabetic() || *c == ' ').collect();
        let t = harfler.split_whitespace().collect::<Vec<_>>().join(" ");
        if t.chars().count() >= 4 { iskelet.push(t); }
    }
    let birlesik = iskelet.join("|");
    // Kısa ve kararlı bir özet — kriptografik olması gerekmiyor, ayırt etmesi yeterli.
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in birlesik.bytes() { h ^= b as u64; h = h.wrapping_mul(0x100_0000_01b3); }
    format!("SB-{h:016x}")
}

/// Öğrenilmiş kuralları uygular. Dönen: alan kodu → değer.
///
/// Kural bulunması değeri KABUL ETTİRMEZ; yalnız aday üretir. Kabul kararı yine denklemindir.
pub fn uygula(sablon_id: &str, satirlar: &[&str], alanlar: &[(String, String)]) -> HashMap<String, String> {
    let b = bellek().lock().unwrap();
    let Some(s) = b.sablonlar.get(sablon_id) else { return HashMap::new() };
    let mut cikti = HashMap::new();
    for (kod, tip) in alanlar {
        let Some(k) = s.kurallar.iter().find(|k| &k.alan == kod) else { continue };
        // Çapa satırını içerikten bul — satır numarasına güvenmiyoruz.
        for (i, ham) in satirlar.iter().enumerate() {
            if !crate::belge_oku::sadelestir(ham).contains(&k.capa) { continue; }
            let aday = match k.strateji.as_str() {
                x if x == Strateji::AltSatir.ad() =>
                    satirlar.get(i + 1).map(|s| s.to_string()).unwrap_or_default(),
                x if x == Strateji::IkiNoktaSonrasi.ad() =>
                    ham.split_once(':').map(|(_, s)| s.to_string()).unwrap_or_default(),
                // Satırın sağı: öğrenilen ÖNEK kesilir, kalanı değerdir.
                _ => match (!k.onek.is_empty(), ham.find(k.onek.as_str())) {
                    (true, Some(p)) => ham[p + k.onek.len()..].to_string(),
                    _ => ham.to_string(),
                },
            };
            // HÜCRE SINIRINA SAYGI — öğrenilmiş kural da yan sütunu yutamaz.
            // Bu eksikti: eski (kirli çıkarım döneminde) öğrenilmiş bir kural, düzeltmeden
            // SONRA bile kirli değer üretiyordu; puanı en yüksek olduğu için de kazanıyordu.
            let aday = ilk_hucre(&aday);
            if let Some(d) = belirtecler(tip, &aday).into_iter().next_back() {
                cikti.insert(kod.clone(), d);
                break;
            }
        }
    }
    cikti
}

/// Kullanıcı alanları onayladı — bu belgeden ÖĞREN.
///
/// Onaylanan her değer için belgede nerede geçtiği bulunur ve o konumun ÇAPASI (etiket satırı)
/// ile stratejisi saklanır. Değer belgede hiç bulunamıyorsa (kullanıcı elle yazmışsa) kural
/// çıkarılmaz — uydurma çapa, sonraki belgede yanlış okuma demektir.
pub fn ogren(sablon_id: &str, ad: &str, belge_turu: &str, metin: &str, onaylar: &[(String, String)]) -> usize {
    if metin.trim().is_empty() { return 0; }
    let satirlar: Vec<&str> = metin.lines().collect();
    let mut b = bellek().lock().unwrap();
    let s = b.sablonlar.entry(sablon_id.to_string()).or_insert_with(|| Sablon {
        id: sablon_id.into(), ad: ad.into(), belge_turu: belge_turu.into(), ..Default::default()
    });
    s.kullanim += 1;
    let mut ogrenilen = 0;

    for (kod, deger) in onaylar {
        let d = deger.trim();
        if d.is_empty() { continue; }
        // Değerin geçtiği satırı bul.
        let Some(i) = satirlar.iter().position(|l| l.contains(d)) else { continue };
        let strateji = if satirlar[i].split_once(':').map(|(_, t)| t.contains(d)).unwrap_or(false) {
            Strateji::IkiNoktaSonrasi
        } else { Strateji::SatirSagi };
        // Değerin ÖNÜNDEKİ metni sakla — uygulama sırasında kesilecek.
        let onek = satirlar[i].find(d).map(|p| satirlar[i][..p].trim_end().to_string())
            .unwrap_or_default();
        // Çapa: değerin bulunduğu satırın harf kısmı; boşsa BİR ÜST satır (etiket üstte olabilir).
        let capa_kaynak = {
            let harf: String = crate::belge_oku::sadelestir(satirlar[i])
                .chars().filter(|c| c.is_alphabetic() || *c == ' ').collect();
            if harf.trim().chars().count() >= 4 { (satirlar[i], strateji.clone()) }
            else if i > 0 { (satirlar[i - 1], Strateji::AltSatir) }
            else { continue }
        };
        let capa: String = crate::belge_oku::sadelestir(capa_kaynak.0)
            .chars().filter(|c| c.is_alphabetic() || *c == ' ').collect();
        let capa = capa.split_whitespace().collect::<Vec<_>>().join(" ");
        if capa.chars().count() < 4 { continue; }

        match s.kurallar.iter_mut().find(|k| &k.alan == kod) {
            Some(k) => {
                k.deneme += 1;
                if k.capa == capa && k.strateji == capa_kaynak.1.ad() { k.isabet += 1; }
                else { *k = Kural { alan: kod.clone(), strateji: capa_kaynak.1.ad().into(), capa, onek: onek.clone(), isabet: 1, deneme: k.deneme }; }
            }
            None => {
                s.kurallar.push(Kural { alan: kod.clone(), strateji: capa_kaynak.1.ad().into(), capa, onek: onek.clone(), isabet: 1, deneme: 1 });
                ogrenilen += 1;
            }
        }
    }
    kaydet(&b);
    ogrenilen
}

/// GET /api/belge/sablonlar — öğrenilmiş şablonlar (arayüzde görünür olmalı:
/// sistem ne öğrendiyse kullanıcı görebilmeli, kara kutu olmamalı).
pub async fn sablon_listesi() -> axum::Json<serde_json::Value> {
    let b = bellek().lock().unwrap();
    let mut liste: Vec<&Sablon> = b.sablonlar.values().collect();
    liste.sort_by_key(|s| std::cmp::Reverse(s.kullanim));
    axum::Json(serde_json::json!({
        "adet": liste.len(),
        "sablonlar": liste,
        "not": "Öğrenilen kurallar okumayı HIZLANDIRIR, doğruluk ölçütünü gevşetmez — \
                her değer yine belgenin denkleminden geçer.",
    }))
}

/// DELETE /api/belge/sablon/:id — yanlış öğrenilmiş şablonu unut.
pub async fn sablon_unut(axum::extract::Path(id): axum::extract::Path<String>) -> axum::Json<serde_json::Value> {
    let mut b = bellek().lock().unwrap();
    let vardi = b.sablonlar.remove(&id).is_some();
    kaydet(&b);
    axum::Json(serde_json::json!({ "unutuldu": vardi, "id": id }))
}

#[cfg(test)]
mod testler;
