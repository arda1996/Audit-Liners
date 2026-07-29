//! Audit-Liners HTTP API — bellekte durum (Postgres adaptörü sonra).
//! Uçlar: GET /api/hesaplar · POST /api/fis · GET /api/mizan

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;

use domain::{
    bilanco, csv_yukle, fatura_fisi, fis_no, gelir_tablosu, iptal_fisi, kdv_mahsup, kesinlestir,
    muavin_olustur, Baglam, BelgeTipi, Dayanak, Defter, Donem, DonemDurum, Fatura, FaturaDurum,
    FaturaSatiri, FaturaYonu, Fis, FisTipi, Hesap, HesapDogasi, HesapId, HesapTipi, Kurus,
    SeriSayac, Tarih, YevmiyeSatiri,
};

mod banka_acik; // İzole açık bankacılık (ÖHVPS) simülatörü — AppState'e bağımsız.
mod belge_acik; // İzole Belgeler backend — GİB faturalar + UBL-TR indir + işaretli ekstre.
mod belge_alim;  // Belge alım akışı — otomatik / seçimli / manuel, tek denklem kapısı.
mod belge_dosya; // Yüklenen PDF/görüntüden metin katmanı + sayfa önizlemesi.
mod belge_ogren; // Yaşayan okuma katmanı — çok strateji + denklemle ayırt etme + şablon öğrenme.
mod belge_konum; // Konum motoru — koordinatlı kelime okuma (bbox), sütun ve font boyutu duyarlı.
mod belge_kalem; // Kalem satırları — mal cinsi, miktar, birim fiyat, tutar (aritmetikle bulunur).
mod belge_oku;  // Belge okuma — okunan metni sayıya çevirir, belgenin kendi aritmetiğiyle doğrular.
mod duzelt;     // Terim düzeltme — bozuk okunan finans terimini sözlükteki karşılığına bağlar.
mod eslestir;   // Kapalı küme eşleştirme — bulmaca modeli: eler, tahmin etmez (cari ünvanı).
mod gider;      // Gider hesapları — 7/A · 7/B seçimi, sektörel uygunluk, KKEG ayrımı.
mod hesaplama;
mod kontrol;    // Güvenlik katmanı — numaralı ön kontrol (K) + değişmez denetimi (D).  // Hesap makinesi — KDV/tevkifat/stopaj/damga; oranlar parametre dosyasından.
#[cfg(test)]
mod veri_setleri; // 10 senaryo veri seti — tahsis motorunu üreteçten bağımsız sınar.
mod simulasyon; // İzole kapsamlı simülasyon — 5000 kayıt, dengeli, banka/nakit/çek/senet enstrümanlı.

const TDHP: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/tdhp.csv"));
const ACIKLAMALAR_JSON: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/hesap-aciklamalari.json"));
const KURALLAR_JSON: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/hesap-kurallari.json"));
const KUMELER_JSON: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/aciklama-kumeleri.json"));
const SEKTORLER_JSON: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/sektorler.json"));
const SABLONLAR_JSON: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/sablonlar.json"));
const DEPARTMANLAR_JSON: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/departmanlar.json"));
const KADEMELER_JSON: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/kademeler.json"));

#[derive(Deserialize)]
struct KumeFile {
    kumeler: HashMap<String, Vec<String>>,
}

#[derive(Deserialize, Clone, Default)]
struct AciklamaRow {
    kod: String,
    #[serde(default)]
    ad: String,
    #[serde(default)]
    tanim: String,
    #[serde(default)]
    isleyis: String,
}
#[derive(Deserialize)]
struct AciklamaFile {
    hesaplar: Vec<AciklamaRow>,
}
#[derive(Deserialize, Clone, Default)]
struct KuralRow {
    kod: String,
    #[serde(default)]
    ad: String,
    #[serde(default)]
    doga: String,
    #[serde(default)]
    borc: String,
    #[serde(default)]
    alacak: String,
    #[serde(default)]
    karsi: Vec<String>,
    #[serde(default)]
    kapatma: Option<String>,
}
#[derive(Deserialize)]
struct KuralFile {
    hesaplar: Vec<KuralRow>,
}

pub struct AppState {
    hesaplar: HashMap<HesapId, Hesap>,
    hesap_sirali: Vec<Hesap>,
    donem: Donem,
    sayac: SeriSayac,
    fisler: Vec<Fis>,
    aciklamalar: HashMap<String, AciklamaRow>,
    kurallar: HashMap<String, KuralRow>,
    /// Standart açıklama kümeleri (koddan; salt-okunur) + kullanıcı eklemeleri (kod → metinler).
    kumeler: HashMap<String, Vec<String>>,
    kullanici_kumeler: HashMap<String, Vec<String>>,
    /// Muhasebecinin kendi fiş şablonları. Mükellefe DEĞİL kullanıcıya aittir (mükellef swap'ında
    /// taşınmaz) — SMMM aynı şablonu tüm müvekkillerinde kullanır; şablon müvekkil verisi taşımaz.
    kullanici_sablonlar: Vec<KullaniciSablon>,
    /// Şablon adı → kaç kez uygulandı. Sık kullanılanı listenin başına almak için (hazır şablonlar
    /// da sayılır). Şablonlar gibi kullanıcıya aittir, mükellef swap'ında taşınmaz.
    sablon_kullanim: HashMap<String, u32>,
    /// Muhasebecinin MANUEL tahsis kararları: tahsilat_id → fatura_no. FIFO karinesini ezer;
    /// kaydedilince o tahsilatın önceki otomatik eşleşmesi kalkar, kalan para yeniden FIFO akar.
    pub manuel_tahsis: HashMap<String, String>,
    /// Havada bakiye için maker işareti: para gelmiş/gitmiş ama FATURA HENÜZ KESİLMEMİŞ.
    /// tahsilat_id → (beklenen firma, işaret tarihi). Eşleştirme değil, TAKİP kaydıdır;
    /// belge gelene kadar hatırlatma üretir (VUK 231: fatura 7 gün içinde düzenlenir).
    pub fatura_bekleyen: HashMap<String, (String, String)>,
    /// Tahsis muhasebe defteri — her tahsis için ÜRETİLEN fiş kaydı. Türetilmiş değil KALICI:
    /// VUK 219 gereği kayıt silinmez; eşleşme değişince kayıt STORNO (ters kayıt) ile kapatılır,
    /// yenisi ayrı fiş olarak açılır. `anahtar` mükerrer kaydı önler (aynı tahsis iki fiş üretemez).
    pub tahsis_defteri: Vec<TahsisKayit>,
    pub fis_sayaci: u64,
    /// Gerçek deftere aktarılmış tahakkuk/tahsis anahtarları — mükerrer aktarımı engeller.
    pub aktarilan: std::collections::HashSet<String>,
    /// gorunum() ÖNBELLEĞİ. Dataset üretimi ~2200 fatura + FIFO dağıtımı demektir; her
    /// istekte yeniden hesaplanınca uçlar 10+ saniye sürüyordu. Dağıtımı etkileyen her
    /// mutasyonda `gorunum_surum` artar ve önbellek düşer.
    pub gorunum_surum: u64,
    pub gorunum_onbellek: Mutex<Option<(u64, Arc<simulasyon::Dataset>)>>,
    /// Banka ekstresi satırları (tutar kuruş; + giriş / − çıkış) → fiş eşleştirme.
    banka: Vec<BankaHareket>,
    /// Gelen belge kutusu (e-Fatura / e-Arşiv / diğer) → fiş eşleştirme. E-dönüşüm bu alanı dolduracak.
    gelen_belgeler: Vec<GelenBelge>,
    faturalar: Vec<Fatura>,
    /// belge_ref → orijinal belge içeriği (UBL XML). Kalıcılıkta dosya/DB arşivi olur.
    belgeler: HashMap<String, String>,
    /// fatura index → üretilen fiş no
    fatura_fisleri: HashMap<usize, String>,
    /// Denetim çalışma kağıdı notları (çalışma id → SMMM sonuç/not metni). Kalıcılıkta DB'ye taşınır.
    denetim_notlar: HashMap<String, String>,
    /// Kağıt üstü manuel düzenlemeler (çalışma id → {hucreler, satirlar}). Defter kayıtlarına ASLA dokunmaz;
    /// sistem değeri korunur, müdahale yanında saklanır (denetim izi).
    kagit_duzenlemeler: HashMap<String, serde_json::Value>,
    /// Vergi hesap kağıdı manuel satırları ("GV-3" → {kkeg:[{ad,tutar}], istisna:[{ad,tutar}]}). Bellekte.
    vergi_duzenlemeler: HashMap<String, serde_json::Value>,
    /// UFRS WorkSheet kayıtları (AJE/RJE/CF) — denetçi çalışmalarından doğar, WTB'ye akar.
    /// Deftere ASLA işlenmez; her kayıt dayanak + denetçi notu + hazırlayan/tarih izi taşır.
    ufrs_kayitlar: Vec<UfrsKayit>,
    /// Kesinleşmiş UFRS raporlama dönemleri (yıl). Kesin döneme kayıt atılamaz/vazgeçilemez;
    /// kesin dönem devir (CF) üretiminin kaynağıdır — firmalar kümüle ilerler.
    ufrs_kesin_donemler: Vec<String>,
    // ── Çoklu mükellef (görev #5) — "aktif çalışma seti + arşiv swap" deseni ──
    // Yukarıdaki mutable alanlar AKTİF mükellefin defteridir; diğerleri `arsiv`'de bekler.
    // 45 handler değişmeden çalışır: hep aktif alanları okur. Mükellef değişince swap yapılır.
    profiller: Vec<MukellefProfil>,
    aktif: String,
    arsiv: HashMap<String, MukellefKayit>,
    // ── Kullanıcı girişi + yetki (kayıt YOK; admin kullanıcı açar) ──
    kullanicilar: Vec<Kullanici>,
    oturumlar: HashMap<String, String>, // token → kullanıcı id
}

#[derive(Clone, Serialize)]
struct Kullanici {
    id: String,
    kullanici_adi: String,
    ad: String,
    rol: String, // "admin" | "kullanici"
    /// Erişebileceği mükellef id'leri. admin için boş = HEPSİ.
    mukellef_idleri: Vec<String>,
    /// Departman (görünürlük): MUHASEBE/VERGI/BORDRO/CARI/DENETIM/YONETIM — hangi modülleri görür.
    #[serde(default)]
    departman: String,
    /// Kademe (işlem yetkisi/hiyerarşi): ELEMAN/SORUMLU/MUDUR/YONETICI — ne yapabilir + onay eşiği.
    #[serde(default)]
    kademe: String,
    #[serde(skip)]
    sifre_ozet: String,
    #[serde(skip)]
    salt: String,
}

/// GEÇİCİ parola özeti — kalıcılık fazında argon2/bcrypt ile DEĞİŞTİRİLECEK.
/// std DefaultHasher kriptografik DEĞİLDİR; yalnız bellek-içi giriş kapısı için placeholder.
fn sifre_ozet(salt: &str, sifre: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut cur = format!("{salt}${sifre}");
    for _ in 0..10000 {
        let mut h = DefaultHasher::new();
        cur.hash(&mut h);
        cur = format!("{:016x}", h.finish());
    }
    cur
}

/// Authorization: Bearer <token> başlığından oturumdaki kullanıcıyı çözer.
fn oturumdaki<'a>(st: &'a AppState, h: &HeaderMap) -> Option<&'a Kullanici> {
    let tok = h.get("authorization")?.to_str().ok()?.strip_prefix("Bearer ")?;
    let uid = st.oturumlar.get(tok)?;
    st.kullanicilar.iter().find(|k| &k.id == uid)
}

/// Kademe (data/kademeler.json) → (izinli işlemler, onay tutar eşiği kuruş). Bilinmeyen kademe = boş yetki.
fn kademe_izinleri(kademe: &str) -> (Vec<String>, i64) {
    serde_json::from_str::<serde_json::Value>(KADEMELER_JSON).ok()
        .and_then(|v| {
            let k = v["kademeler"].as_array()?.iter().find(|k| k["kod"] == kademe)?;
            let islemler = k["islemler"].as_array()?.iter().filter_map(|x| x.as_str().map(String::from)).collect();
            Some((islemler, k["onay_esigi_kurus"].as_i64().unwrap_or(0)))
        })
        .unwrap_or((vec![], 0))
}

/// Oturumdaki kullanıcının `islem`i yapma yetkisi var mı? (admin rol veya kademe işlemi içerir / '*')
fn islem_yetkisi(st: &AppState, h: &HeaderMap, islem: &str) -> bool {
    match oturumdaki(st, h) {
        Some(k) if k.rol == "admin" => true,
        Some(k) => {
            let (islemler, _) = kademe_izinleri(&k.kademe);
            islemler.iter().any(|x| x == "*" || x == islem)
        }
        None => false,
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct MukellefProfil {
    id: String,
    unvan: String,
    #[serde(default)]
    vkn: String,
    #[serde(default)]
    sektor_kodlari: Vec<String>,
    #[serde(default)]
    maliyet_secenegi: String,
}

/// Aktif olmayan mükellefin defteri — aktif olan AppState alanlarında yaşar, bu arşivde bekler.
struct MukellefKayit {
    donem: Donem,
    sayac: SeriSayac,
    fisler: Vec<Fis>,
    kullanici_kumeler: HashMap<String, Vec<String>>,
    banka: Vec<BankaHareket>,
    gelen_belgeler: Vec<GelenBelge>,
    faturalar: Vec<Fatura>,
    belgeler: HashMap<String, String>,
    fatura_fisleri: HashMap<usize, String>,
    denetim_notlar: HashMap<String, String>,
    kagit_duzenlemeler: HashMap<String, serde_json::Value>,
    vergi_duzenlemeler: HashMap<String, serde_json::Value>,
    ufrs_kayitlar: Vec<UfrsKayit>,
    ufrs_kesin_donemler: Vec<String>,
}

fn bos_donem() -> Donem {
    Donem { durum: DonemDurum::Acik, baslangic: Tarih::yeni(2026, 1, 1), bitis: Tarih::yeni(2026, 12, 31) }
}

impl AppState {
    /// Aktif çalışma setini bir MukellefKayit'e taşır (alanları boşaltır).
    fn topla(&mut self) -> MukellefKayit {
        MukellefKayit {
            donem: std::mem::replace(&mut self.donem, bos_donem()),
            sayac: std::mem::replace(&mut self.sayac, SeriSayac::yeni()),
            fisler: std::mem::take(&mut self.fisler),
            kullanici_kumeler: std::mem::take(&mut self.kullanici_kumeler),
            banka: std::mem::take(&mut self.banka),
            gelen_belgeler: std::mem::take(&mut self.gelen_belgeler),
            faturalar: std::mem::take(&mut self.faturalar),
            belgeler: std::mem::take(&mut self.belgeler),
            fatura_fisleri: std::mem::take(&mut self.fatura_fisleri),
            denetim_notlar: std::mem::take(&mut self.denetim_notlar),
            kagit_duzenlemeler: std::mem::take(&mut self.kagit_duzenlemeler),
            vergi_duzenlemeler: std::mem::take(&mut self.vergi_duzenlemeler),
            ufrs_kayitlar: std::mem::take(&mut self.ufrs_kayitlar),
            ufrs_kesin_donemler: std::mem::take(&mut self.ufrs_kesin_donemler),
        }
    }
    /// Bir MukellefKayit'i aktif çalışma setine yükler.
    fn yerlestir(&mut self, k: MukellefKayit) {
        self.donem = k.donem;
        self.sayac = k.sayac;
        self.fisler = k.fisler;
        self.kullanici_kumeler = k.kullanici_kumeler;
        self.banka = k.banka;
        self.gelen_belgeler = k.gelen_belgeler;
        self.faturalar = k.faturalar;
        self.belgeler = k.belgeler;
        self.fatura_fisleri = k.fatura_fisleri;
        self.denetim_notlar = k.denetim_notlar;
        self.kagit_duzenlemeler = k.kagit_duzenlemeler;
        self.vergi_duzenlemeler = k.vergi_duzenlemeler;
        self.ufrs_kayitlar = k.ufrs_kayitlar;
        self.ufrs_kesin_donemler = k.ufrs_kesin_donemler;
    }
    fn bos_kayit() -> MukellefKayit {
        MukellefKayit {
            donem: bos_donem(), sayac: SeriSayac::yeni(), fisler: Vec::new(),
            kullanici_kumeler: HashMap::new(), banka: Vec::new(), gelen_belgeler: Vec::new(),
            faturalar: Vec::new(), belgeler: HashMap::new(), fatura_fisleri: HashMap::new(),
            denetim_notlar: HashMap::new(), kagit_duzenlemeler: HashMap::new(), vergi_duzenlemeler: HashMap::new(),
            ufrs_kayitlar: Vec::new(), ufrs_kesin_donemler: Vec::new(),
        }
    }
}

pub type Shared = Arc<Mutex<AppState>>;

// ---- DTO'lar (kanonik JSON formatı) ----

#[derive(Serialize)]
struct HesapDto {
    kod: String,
    ad: String,
    tip: String,
    dogasi: String,
    yaprak: bool,
    seviye: u8,
}

#[derive(Deserialize)]
struct MuavinGirdi {
    ana_kod: String,
    alt_kod: String,
    ad: String,
}

#[derive(Deserialize)]
struct SatirGirdi {
    hesap_kod: String,
    borc: i64,   // kuruş
    alacak: i64, // kuruş
    aciklama: Option<String>,
    masraf_yeri_id: Option<u64>,
}

#[derive(Deserialize)]
struct FisGirdi {
    tip: String,
    tarih: String, // "YYYY-MM-DD"
    belge_tipi: Option<String>,
    belge_no: Option<String>,
    satirlar: Vec<SatirGirdi>,
}

#[derive(Serialize)]
struct FisSonuc {
    fis_no: String,
}

#[derive(Serialize)]
struct HataDto {
    hata: String,
}

#[derive(Serialize)]
struct MizanSatir {
    kebir: String,
    kod: String,
    ad: String,
    aciklama: String,
    borc: i64,
    alacak: i64,
    borc_bakiye: i64,
    alacak_bakiye: i64,
}

#[derive(Serialize)]
struct FisOzet {
    id: usize,
    yevmiye_no: Option<u64>,
    fis_no: String,
    tip: String,
    tarih: String,
    aciklama: String,
    tutar: i64,
    dayanaksiz: bool,
    /// "Kesin" | "İptal edildi" | "İptal fişi"
    durum: String,
}

fn fis_durum_str(f: &Fis) -> String {
    if f.iptal { "İptal edildi".into() }
    else if f.iptal_kaynak_id.is_some() { "İptal fişi".into() }
    else { "Kesin".into() }
}

#[derive(Serialize)]
struct SatirDetay {
    kod: String,
    ad: String,
    aciklama: String,
    borc: i64,
    alacak: i64,
}

#[derive(Serialize)]
struct FisDetay {
    id: usize,
    fis_no: String,
    tip: String,
    tarih: String,
    aciklama: String,
    belge_tipi: Option<String>,
    belge_no: Option<String>,
    dayanaksiz: bool,
    satirlar: Vec<SatirDetay>,
}

// ---- yardımcılar ----

fn tip_str(t: &HesapTipi) -> &'static str {
    match t {
        HesapTipi::Aktif => "Aktif",
        HesapTipi::Pasif => "Pasif",
        HesapTipi::Gelir => "Gelir",
        HesapTipi::Gider => "Gider",
        HesapTipi::Maliyet => "Maliyet",
        HesapTipi::Nazim => "Nazım",
    }
}

fn doga_str(d: &HesapDogasi) -> &'static str {
    match d {
        HesapDogasi::Borc => "Borç",
        HesapDogasi::Alacak => "Alacak",
    }
}

fn parse_tip(s: &str) -> Option<FisTipi> {
    match s {
        "Tahsil" => Some(FisTipi::Tahsil),
        "Tediye" => Some(FisTipi::Tediye),
        "Mahsup" => Some(FisTipi::Mahsup),
        "Acilis" | "Açılış" => Some(FisTipi::Acilis),
        "Kapanis" | "Kapanış" => Some(FisTipi::Kapanis),
        _ => None,
    }
}

fn parse_belge(s: &str) -> BelgeTipi {
    match s {
        "Fatura" => BelgeTipi::Fatura,
        "e-Fatura" | "EFatura" => BelgeTipi::EFatura,
        "Makbuz" => BelgeTipi::Makbuz,
        "Dekont" => BelgeTipi::Dekont,
        "Sözleşme" | "Sozlesme" => BelgeTipi::Sozlesme,
        "Bordro" => BelgeTipi::Bordro,
        "Beyanname" => BelgeTipi::Beyanname,
        _ => BelgeTipi::Diger,
    }
}

/// Tarih girişi: hem gg.aa.yyyy (standart) hem yyyy-aa-gg (HTML date input) kabul edilir.
fn parse_tarih(s: &str) -> Option<Tarih> {
    let s = s.trim();
    if s.contains('.') {
        let mut p = s.split('.');
        let gun = p.next()?.parse().ok()?;
        let ay = p.next()?.parse().ok()?;
        let yil = p.next()?.parse().ok()?;
        return Some(Tarih::yeni(yil, ay, gun));
    }
    let mut p = s.split('-');
    let yil = p.next()?.parse().ok()?;
    let ay = p.next()?.parse().ok()?;
    let gun = p.next()?.parse().ok()?;
    Some(Tarih::yeni(yil, ay, gun))
}

/// STANDART gösterim: her koşulda gg.aa.yyyy.
fn tarih_str(t: &Tarih) -> String {
    format!("{:02}.{:02}.{:04}", t.gun, t.ay, t.yil)
}

fn fis_tip_str(t: &FisTipi) -> &'static str {
    match t {
        FisTipi::Tahsil => "Tahsil",
        FisTipi::Tediye => "Tediye",
        FisTipi::Mahsup => "Mahsup",
        FisTipi::Acilis => "Açılış",
        FisTipi::Kapanis => "Kapanış",
    }
}

fn belge_str(b: &BelgeTipi) -> &'static str {
    match b {
        BelgeTipi::Fatura => "Fatura",
        BelgeTipi::EFatura => "e-Fatura",
        BelgeTipi::Makbuz => "Makbuz",
        BelgeTipi::Dekont => "Dekont",
        BelgeTipi::PosFisi => "POS fişi",
        BelgeTipi::Sozlesme => "Sözleşme",
        BelgeTipi::Bordro => "Bordro",
        BelgeTipi::Beyanname => "Beyanname",
        BelgeTipi::Diger => "Diğer",
    }
}

/// Kırılım seviyesi: 1=sınıf, 2=grup, 3=ana hesap, 4+=muavin (her nokta bir seviye).
fn seviye(kod: &str) -> u8 {
    match kod.find('.') {
        Some(i) => 3 + kod[i..].matches('.').count() as u8,
        None => kod.len() as u8,
    }
}

fn hesap_dto(h: &Hesap) -> HesapDto {
    HesapDto {
        kod: h.kod.clone(),
        ad: h.ad.clone(),
        tip: tip_str(&h.tip).to_string(),
        dogasi: doga_str(&h.dogasi).to_string(),
        yaprak: h.yaprak,
        seviye: seviye(&h.kod),
    }
}

// ---- uçlar ----

/// Yalnız yaprak (postlanabilir) hesaplar — fiş satırı seçimi için.
async fn hesaplar(State(st): State<Shared>) -> Json<Vec<HesapDto>> {
    let st = st.lock().unwrap();
    let liste = st
        .hesap_sirali
        .iter()
        .filter(|h| h.yaprak)
        .map(hesap_dto)
        .collect();
    Json(liste)
}

/// Tüm hesap planı (sınıf→grup→ana→muavin) — ağaç görünümü için, seviye bilgisiyle.
async fn hesap_plani(State(st): State<Shared>) -> Json<Vec<HesapDto>> {
    let st = st.lock().unwrap();
    let liste = st.hesap_sirali.iter().map(hesap_dto).collect();
    Json(liste)
}

/// Ana hesap altına muavin (alt kırılım) ekler; ana hesabı yaprak olmaktan çıkarır.
async fn muavin_ekle(
    State(state): State<Shared>,
    Json(g): Json<MuavinGirdi>,
) -> Result<Json<HesapDto>, (StatusCode, Json<HataDto>)> {
    let hata = |m: String| (StatusCode::BAD_REQUEST, Json(HataDto { hata: m }));
    let mut guard = state.lock().unwrap();
    let st = &mut *guard;

    let ana = st
        .hesaplar
        .get(&g.ana_kod)
        .cloned()
        .ok_or_else(|| hata("ana hesap bulunamadı".into()))?;

    let muavin = muavin_olustur(&ana, g.alt_kod.trim(), g.ad.trim());
    if st.hesaplar.contains_key(&muavin.kod) {
        return Err(hata(format!("{} zaten var", muavin.kod)));
    }

    // Ana hesabı yaprak olmaktan çıkar (artık yalnız muavine kayıt yapılır).
    if let Some(h) = st.hesaplar.get_mut(&g.ana_kod) {
        h.yaprak = false;
    }
    for h in st.hesap_sirali.iter_mut() {
        if h.kod == g.ana_kod {
            h.yaprak = false;
        }
    }

    st.hesaplar.insert(muavin.kod.clone(), muavin.clone());
    st.hesap_sirali.push(muavin.clone());
    st.hesap_sirali.sort_by(|a, b| a.kod.cmp(&b.kod));

    Ok(Json(hesap_dto(&muavin)))
}

async fn fis_ekle(
    State(state): State<Shared>,
    headers: HeaderMap,
    Json(girdi): Json<FisGirdi>,
) -> Result<Json<FisSonuc>, (StatusCode, Json<HataDto>)> {
    let hata = |m: String| (StatusCode::BAD_REQUEST, Json(HataDto { hata: m }));
    let yhata = |m: String| (StatusCode::FORBIDDEN, Json(HataDto { hata: m }));

    // YETKİ: fişi kesinleştirmek kademe yetkisi ister (ELEMAN yalnız taslak — henüz taslak CRUD yok).
    {
        let st = state.lock().unwrap();
        if !islem_yetkisi(&st, &headers, "kesinlestir") {
            return Err(yhata("bu kademe fiş kesinleştiremez (yalnız taslak yetkisi)".into()));
        }
        // Onay tutar eşiği: kademenin kesinleştirebileceği azami fiş tutarı (0 = sınırsız).
        if let Some(k) = oturumdaki(&st, &headers) {
            if k.rol != "admin" {
                let (_, esik) = kademe_izinleri(&k.kademe);
                let toplam: i64 = girdi.satirlar.iter().map(|s| s.borc.max(0)).sum();
                if esik > 0 && toplam > esik {
                    return Err(yhata(format!("onay eşiği aşıldı: {} kademesi en çok {} TL kesinleştirebilir (fiş: {} TL)",
                        k.kademe, esik / 100, toplam / 100)));
                }
            }
        }
    }

    let tip = parse_tip(&girdi.tip).ok_or_else(|| hata("geçersiz fiş tipi".into()))?;
    let tarih = parse_tarih(&girdi.tarih).ok_or_else(|| hata("geçersiz tarih".into()))?;

    let satirlar: Vec<YevmiyeSatiri> = girdi
        .satirlar
        .iter()
        .map(|s| YevmiyeSatiri {
            hesap_id: s.hesap_kod.clone(),
            borc: Kurus::yeni(s.borc),
            alacak: Kurus::yeni(s.alacak),
            masraf_yeri_id: s.masraf_yeri_id,
            aciklama: s.aciklama.clone().unwrap_or_default(),
        })
        .collect();

    // Güvenlik: aynı belge tipi+no ile kesinleşmiş fiş varsa reddet (mükerrer kayıt).
    if let (Some(bt), Some(bn)) = (girdi.belge_tipi.as_deref().filter(|x| !x.is_empty()), girdi.belge_no.as_deref().filter(|x| !x.is_empty())) {
        let st = state.lock().unwrap();
        if let Some(f) = st.fisler.iter().find(|f| f.dayanak.as_ref().is_some_and(|d| belge_str(&d.belge_tipi) == bt && d.belge_no == bn)) {
            return Err(hata(format!("mükerrer belge: {} {} zaten {} fişinde kayıtlı", bt, bn, fis_no(f).unwrap_or_default())));
        }
    }

    // KRONOLOJİ: yevmiye tarih sırası korunur — son kesin fişten önceki tarihe kayıt reddedilir.
    {
        let st = state.lock().unwrap();
        if let Some(son) = st.fisler.last() {
            if tarih < son.tarih {
                return Err(hata(format!(
                    "kronoloji ihlali: son kayıt {} tarihli; daha eski tarihe ({}) kayıt girilemez",
                    tarih_str(&son.tarih), tarih_str(&tarih)
                )));
            }
        }
    }

    let mut fis = Fis::taslak(tip, tarih, satirlar);
    if let Some(bt) = girdi.belge_tipi.as_deref().filter(|x| !x.is_empty()) {
        fis.dayanak = Some(Dayanak {
            belge_tipi: parse_belge(bt),
            belge_no: girdi.belge_no.clone().unwrap_or_default(),
            belge_tarihi: tarih,
        });
    }

    let mut guard = state.lock().unwrap();
    let st = &mut *guard;
    let sonuc = {
        let ctx = Baglam {
            hesaplar: &st.hesaplar,
            donem: &st.donem,
        };
        kesinlestir(&mut fis, &ctx, &mut st.sayac)
    };
    match sonuc {
        Ok(()) => {
            let no = fis_no(&fis).unwrap_or_default();
            st.fisler.push(fis);
            Ok(Json(FisSonuc { fis_no: no }))
        }
        Err(e) => Err(hata(e.to_string())),
    }
}

/// Mizan — standart format: Kebir | Hesap Kodu | Hesap Adı | Açıklama | Borç | Alacak | Borç Bk. | Alacak Bk.
async fn mizan(State(state): State<Shared>) -> Json<Vec<MizanSatir>> {
    let st = state.lock().unwrap();
    let d = defter_kur(&st.fisler);
    let satirlar: Vec<MizanSatir> = d
        .mizan()
        .into_iter()
        .map(|m| {
            let kebir = m.hesap_id.split('.').next().unwrap_or(&m.hesap_id).to_string();
            let (ad, aciklama) = st
                .hesaplar
                .get(&m.hesap_id)
                .map(|h| (h.ad.clone(), format!("{} · {}", tip_str(&h.tip), doga_str(&h.dogasi))))
                .unwrap_or_default();
            MizanSatir {
                kebir,
                kod: m.hesap_id,
                ad,
                aciklama,
                borc: m.borc.deger(),
                alacak: m.alacak.deger(),
                borc_bakiye: m.borc_bakiye.deger(),
                alacak_bakiye: m.alacak_bakiye.deger(),
            }
        })
        .collect();
    Json(satirlar)
}

async fn fisler(State(state): State<Shared>, Query(q): Query<HashMap<String, String>>) -> Json<Vec<FisOzet>> {
    let st = state.lock().unwrap();
    let limit: usize = q.get("limit").and_then(|s| s.parse().ok()).unwrap_or(300);
    let n = st.fisler.len();
    let bas = if limit > 0 && n > limit { n - limit } else { 0 };
    let v = st
        .fisler
        .iter()
        .enumerate()
        .skip(bas)
        .map(|(i, f)| FisOzet {
            id: i,
            yevmiye_no: f.yevmiye_no,
            fis_no: fis_no(f).unwrap_or_default(),
            tip: fis_tip_str(&f.tip).to_string(),
            tarih: tarih_str(&f.tarih),
            aciklama: f.aciklama.clone(),
            tutar: f.toplam_borc().deger(),
            dayanaksiz: f.dayanaksiz,
            durum: fis_durum_str(f),
        })
        .collect();
    Json(v)
}

async fn fis_detay(
    State(state): State<Shared>,
    Path(id): Path<usize>,
) -> Result<Json<FisDetay>, StatusCode> {
    let st = state.lock().unwrap();
    let f = st.fisler.get(id).ok_or(StatusCode::NOT_FOUND)?;
    let satirlar = f
        .satirlar
        .iter()
        .map(|s| SatirDetay {
            kod: s.hesap_id.clone(),
            ad: st
                .hesaplar
                .get(&s.hesap_id)
                .map(|h| h.ad.clone())
                .unwrap_or_default(),
            aciklama: s.aciklama.clone(),
            borc: s.borc.deger(),
            alacak: s.alacak.deger(),
        })
        .collect();
    Ok(Json(FisDetay {
        id,
        fis_no: fis_no(f).unwrap_or_default(),
        tip: fis_tip_str(&f.tip).to_string(),
        tarih: tarih_str(&f.tarih),
        aciklama: f.aciklama.clone(),
        belge_tipi: f.dayanak.as_ref().map(|d| belge_str(&d.belge_tipi).to_string()),
        belge_no: f.dayanak.as_ref().map(|d| d.belge_no.clone()),
        dayanaksiz: f.dayanaksiz,
        satirlar,
    }))
}

#[derive(Serialize, Default)]
struct AciklamaDto {
    kod: String,
    ad: String,
    tanim: String,
    isleyis: String,
    karsi: Vec<String>,
    kapatma: Option<String>,
}

/// Hesabın resmi açıklaması (MSUGT tanım+işleyiş) + karşı bacaklar/kapatma.
/// Muavin (120.01) için ana hesabın (120) açıklamasına düşer.
async fn hesap_aciklama(
    State(state): State<Shared>,
    Path(kod): Path<String>,
) -> Result<Json<AciklamaDto>, StatusCode> {
    let st = state.lock().unwrap();
    let ana = kod.split('.').next().unwrap_or(&kod).to_string();
    let a = st.aciklamalar.get(&kod).or_else(|| st.aciklamalar.get(&ana));
    let k = st.kurallar.get(&kod).or_else(|| st.kurallar.get(&ana));
    if a.is_none() && k.is_none() {
        return Err(StatusCode::NOT_FOUND);
    }
    let ad = st
        .hesaplar
        .get(&kod)
        .map(|h| h.ad.clone())
        .or_else(|| a.map(|x| x.ad.clone()))
        .unwrap_or_default();
    Ok(Json(AciklamaDto {
        kod: kod.clone(),
        ad,
        tanim: a.map(|x| x.tanim.clone()).unwrap_or_default(),
        isleyis: a.map(|x| x.isleyis.clone()).unwrap_or_default(),
        karsi: k.map(|x| x.karsi.clone()).unwrap_or_default(),
        kapatma: k.and_then(|x| x.kapatma.clone()),
    }))
}

#[derive(Serialize)]
struct KdvDurum {
    indirilecek: i64,
    devreden: i64,
    hesaplanan: i64,
    fark: i64, // + ödenecek, − devreden yönlü
}

fn defter_kur(fisler: &[Fis]) -> Defter {
    let mut d = Defter::yeni();
    for f in fisler {
        d.ekle(f.clone());
    }
    d
}

async fn kdv_durum(State(state): State<Shared>) -> Json<KdvDurum> {
    let st = state.lock().unwrap();
    let d = defter_kur(&st.fisler);
    let net = |k: &str| { let (b, a) = d.hesap_bakiyesi(k); b.deger() - a.deger() };
    let ind = net("191").max(0);
    let dev = net("190").max(0);
    let hes = (-net("391")).max(0);
    Json(KdvDurum { indirilecek: ind, devreden: dev, hesaplanan: hes, fark: hes - ind - dev })
}

#[derive(Deserialize)]
struct KdvGirdi { tarih: String }

async fn kdv_mahsup_yap(
    State(state): State<Shared>,
    Json(g): Json<KdvGirdi>,
) -> Result<Json<FisSonuc>, (StatusCode, Json<HataDto>)> {
    let hata = |m: String| (StatusCode::BAD_REQUEST, Json(HataDto { hata: m }));
    let tarih = parse_tarih(&g.tarih).ok_or_else(|| hata("geçersiz tarih".into()))?;
    let mut guard = state.lock().unwrap();
    let st = &mut *guard;
    let d = defter_kur(&st.fisler);
    let mut fis = kdv_mahsup(&d, tarih).ok_or_else(|| hata("KDV hareketi yok".into()))?;
    let sonuc = {
        let ctx = Baglam { hesaplar: &st.hesaplar, donem: &st.donem };
        kesinlestir(&mut fis, &ctx, &mut st.sayac)
    };
    match sonuc {
        Ok(()) => { let no = fis_no(&fis).unwrap_or_default(); st.fisler.push(fis); Ok(Json(FisSonuc { fis_no: no })) }
        Err(e) => Err(hata(e.to_string())),
    }
}

#[derive(Deserialize)]
struct IptalGirdi {
    /// Hatanın fark edildiği tarih (VUK 217 — kaynak tarihe geri dönülmez).
    tarih: String,
    /// İptal gerekçesi (zorunlu — düzeltme kaydı açıklamasına yazılır).
    gerekce: String,
}

/// Kesin fişi İPTAL eder: VUK md.217'ye uygun ters kayıt (düzeltme) fişi üretir.
async fn fis_iptal(
    State(state): State<Shared>,
    headers: HeaderMap,
    Path(id): Path<usize>,
    Json(g): Json<IptalGirdi>,
) -> Result<Json<FisSonuc>, (StatusCode, Json<HataDto>)> {
    let hata = |m: String| (StatusCode::BAD_REQUEST, Json(HataDto { hata: m }));
    {
        let st = state.lock().unwrap();
        if !islem_yetkisi(&st, &headers, "iptal") {
            return Err((StatusCode::FORBIDDEN, Json(HataDto { hata: "iptal (VUK 217) yalnız müdür/yönetici kademesinde".into() })));
        }
    }
    let gerekce = g.gerekce.trim();
    if gerekce.is_empty() { return Err(hata("iptal gerekçesi zorunlu".into())); }
    let iptal_tarihi = parse_tarih(&g.tarih).ok_or_else(|| hata("geçersiz tarih".into()))?;

    let mut guard = state.lock().unwrap();
    let st = &mut *guard;
    let kaynak = st.fisler.get(id).ok_or_else(|| hata("fiş yok".into()))?;
    if kaynak.iptal { return Err(hata("fiş zaten iptal edilmiş".into())); }
    if kaynak.iptal_kaynak_id.is_some() { return Err(hata("iptal fişi iptal edilemez".into())); }
    // VUK 217: düzeltme fişi kaynak fişten ÖNCEKİ bir tarihe atılamaz.
    if iptal_tarihi < kaynak.tarih {
        return Err(hata("iptal tarihi kaynak fişin tarihinden önce olamaz (VUK 217)".into()));
    }
    let kaynak_no = fis_no(kaynak).unwrap_or_default();

    let mut ters = iptal_fisi(kaynak, id as u64, &kaynak_no, iptal_tarihi, gerekce);
    let sonuc = {
        let ctx = Baglam { hesaplar: &st.hesaplar, donem: &st.donem };
        kesinlestir(&mut ters, &ctx, &mut st.sayac)
    };
    match sonuc {
        Ok(()) => {
            let no = fis_no(&ters).unwrap_or_default();
            st.fisler.push(ters);
            st.fisler[id].iptal = true;
            Ok(Json(FisSonuc { fis_no: no }))
        }
        Err(e) => Err(hata(e.to_string())),
    }
}

// ---- Açıklama kümeleri (hesap koduna göre öneri) ----

#[derive(Serialize)]
struct KumeDto {
    standart: Vec<String>,
    kullanici: Vec<String>,
}

/// Hesabın açıklama kümesi. Muavin (153.01) ana koda (153) düşer; ikisi birleşir.
async fn aciklama_kume(State(state): State<Shared>, Path(kod): Path<String>) -> Json<KumeDto> {
    let st = state.lock().unwrap();
    let ana = kod.split('.').next().unwrap_or(&kod).to_string();
    let mut standart = Vec::new();
    for k in [&kod, &ana] {
        if let Some(v) = st.kumeler.get(k) {
            for m in v { if !standart.contains(m) { standart.push(m.clone()); } }
        }
    }
    let mut kullanici = Vec::new();
    for k in [&kod, &ana] {
        if let Some(v) = st.kullanici_kumeler.get(k) {
            for m in v { if !kullanici.contains(m) { kullanici.push(m.clone()); } }
        }
    }
    Json(KumeDto { standart, kullanici })
}

#[derive(Deserialize)]
struct KumeGirdi { kod: String, metin: String }

/// Kullanıcı ekleme: verilen koda (muavin verildiyse muavine) yeni açıklama kaydeder.
async fn aciklama_kume_ekle(State(state): State<Shared>, Json(g): Json<KumeGirdi>) -> Result<Json<KumeDto>, (StatusCode, Json<HataDto>)> {
    let metin = g.metin.trim().to_string();
    if metin.is_empty() || metin.len() > 120 {
        return Err((StatusCode::BAD_REQUEST, Json(HataDto { hata: "metin 1-120 karakter olmalı".into() })));
    }
    {
        let mut st = state.lock().unwrap();
        let ana = g.kod.split('.').next().unwrap_or(&g.kod).to_string();
        let std_var = st.kumeler.get(&ana).map(|v| v.contains(&metin)).unwrap_or(false);
        let e = st.kullanici_kumeler.entry(g.kod.clone()).or_default();
        if !std_var && !e.contains(&metin) { e.push(metin); }
    }
    Ok(aciklama_kume(State(state), Path(g.kod)).await)
}

// ---- Yasal defterler & mali tablolar ----

#[derive(Serialize)]
struct YevmiyeMadde {
    yevmiye_no: Option<u64>,
    fis_no: String,
    tarih: String,
    tip: String,
    aciklama: String,
    /// Belge bilgisi (yasal format zorunluluğu): "Fatura FT-1 · 2026-03-01"
    belge: Option<String>,
    toplam: i64,
    satirlar: Vec<SatirDetay>,
}

/// Yevmiye defteri: kesin fişler tarih + yevmiye no sırasıyla (müteselsil maddeler).
async fn yevmiye(State(state): State<Shared>, Query(q): Query<HashMap<String, String>>) -> Json<Vec<YevmiyeMadde>> {
    let st = state.lock().unwrap();
    let limit: usize = q.get("limit").and_then(|s| s.parse().ok()).unwrap_or(150);
    let mut v: Vec<&Fis> = st.fisler.iter().collect();
    v.sort_by(|a, b| a.tarih.cmp(&b.tarih).then(a.yevmiye_no.cmp(&b.yevmiye_no)));
    if limit > 0 && v.len() > limit { v.drain(0..v.len() - limit); }
    Json(v.into_iter().map(|f| YevmiyeMadde {
        yevmiye_no: f.yevmiye_no,
        fis_no: fis_no(f).unwrap_or_default(),
        tarih: tarih_str(&f.tarih),
        tip: fis_tip_str(&f.tip).to_string(),
        aciklama: f.aciklama.clone(),
        belge: f.dayanak.as_ref().map(|d| format!("{} {} · {}", belge_str(&d.belge_tipi), d.belge_no, tarih_str(&d.belge_tarihi))),
        toplam: f.toplam_borc().deger(),
        satirlar: f.satirlar.iter().map(|s| SatirDetay {
            kod: s.hesap_id.clone(),
            ad: st.hesaplar.get(&s.hesap_id).map(|h| h.ad.clone()).unwrap_or_default(),
            aciklama: s.aciklama.clone(),
            borc: s.borc.deger(),
            alacak: s.alacak.deger(),
        }).collect(),
    }).collect())
}

#[derive(Serialize)]
struct KebirDto {
    kod: String,
    ad: String,
    toplam_hareket: usize,
    hareketler: Vec<KebirHareketDto>,
}
#[derive(Serialize)]
struct KebirHareketDto { tarih: String, yevmiye_no: Option<u64>, fis_no: String, aciklama: String, karsi: String, borc: i64, alacak: i64, yuruyen_bakiye: i64, fis_id: Option<usize>, belge: String }

/// Defter-i kebir: hesabın hareketleri + yürüyen bakiye — SAYFALI (varsayılan son 300).
async fn kebir(State(state): State<Shared>, Path(kod): Path<String>, Query(q): Query<HashMap<String, String>>) -> Json<KebirDto> {
    let st = state.lock().unwrap();
    let d = defter_kur(&st.fisler);
    let tumu = d.kebir(&kod);
    let toplam = tumu.len();
    let limit: usize = q.get("limit").and_then(|s| s.parse().ok()).unwrap_or(300);
    let bas = if limit > 0 { toplam.saturating_sub(limit) } else { 0 };
    let hareketler = tumu[bas..].iter().map(|h| KebirHareketDto {
        tarih: tarih_str(&h.tarih), yevmiye_no: h.yevmiye_no, fis_no: h.fis_no.clone(),
        aciklama: h.aciklama.clone(), karsi: h.karsi.clone(),
        borc: h.borc.deger(), alacak: h.alacak.deger(), yuruyen_bakiye: h.yuruyen_bakiye,
        fis_id: None, belge: String::new(),
    }).collect();
    let ad = st.hesaplar.get(&kod).map(|h| h.ad.clone()).unwrap_or_default();
    Json(KebirDto { kod, ad, toplam_hareket: toplam, hareketler })
}

// ---- Muavin defteri: alt hesap düzeyinde, hesap hesap bloklar halinde döküm ----

/// Muavin ÖZETİ — büyük veri setinde donmayı önlemek için hareketler ayrı, sayfalı uçtan gelir.
#[derive(Serialize)]
struct MuavinOzet {
    kod: String,
    ad: String,
    hareket_sayisi: usize,
    toplam_borc: i64,
    toplam_alacak: i64,
    bakiye: i64, // borç(+)/alacak(−)
}

fn muavin_kodlari(st: &AppState, onek: Option<&str>) -> Vec<String> {
    let d = defter_kur(&st.fisler);
    let mut kodlar: Vec<String> = d.mizan().into_iter().map(|m| m.hesap_id).collect();
    if let Some(p) = onek {
        let alt = format!("{}.", p);
        kodlar.retain(|k| k == p || k.starts_with(&alt) || (!p.contains('.') && k.split('.').next().unwrap_or(k).starts_with(p)));
    }
    kodlar
}

async fn muavin(State(state): State<Shared>, Query(q): Query<HashMap<String, String>>) -> Json<Vec<MuavinOzet>> {
    let st = state.lock().unwrap();
    let kodlar = muavin_kodlari(&st, q.get("kod").map(|s| s.as_str()).filter(|s| !s.is_empty()));
    // Tek geçişte hesap başına sayaç + toplamlar (hareket listesi ÜRETİLMEZ — lite)
    let mut m: HashMap<String, (usize, i64, i64)> = HashMap::new();
    for f in &st.fisler {
        for s in &f.satirlar {
            let e = m.entry(s.hesap_id.clone()).or_insert((0, 0, 0));
            e.0 += 1;
            e.1 += s.borc.deger();
            e.2 += s.alacak.deger();
        }
    }
    Json(kodlar.into_iter().map(|kod| {
        let (n, tb, ta) = m.get(&kod).copied().unwrap_or((0, 0, 0));
        let ad = st.hesaplar.get(&kod).map(|h| h.ad.clone()).unwrap_or_default();
        MuavinOzet { kod, ad, hareket_sayisi: n, toplam_borc: tb, toplam_alacak: ta, bakiye: tb - ta }
    }).collect())
}

#[derive(Serialize)]
struct HareketSayfa {
    toplam: usize,
    offset: usize,
    hareketler: Vec<KebirHareketDto>,
}

/// Bir hesabın hareketleri — SAYFALI (offset/limit; varsayılan SON 200 hareket).
async fn muavin_hareket(
    State(state): State<Shared>,
    Path(kod): Path<String>,
    Query(q): Query<HashMap<String, String>>,
) -> Json<HareketSayfa> {
    let st = state.lock().unwrap();
    let d = defter_kur(&st.fisler);
    let tumu = d.kebir(&kod);
    let toplam = tumu.len();
    let limit: usize = q.get("limit").and_then(|s| s.parse().ok()).unwrap_or(200);
    // offset: sondan geriye (varsayılan 0 = en yeni sayfa)
    let offset: usize = q.get("offset").and_then(|s| s.parse().ok()).unwrap_or(0);
    let son = toplam.saturating_sub(offset);
    let bas = son.saturating_sub(limit);
    // fiş no → (id, dayanak) — kağıt/muavin drill-down'ı fişe ve belgesine bağlar
    let fis_ref: HashMap<String, (usize, String)> = st.fisler.iter().enumerate()
        .filter_map(|(i, f)| fis_no(f).map(|no| {
            let belge = f.dayanak.as_ref()
                .map(|dy| format!("{} {}", belge_str(&dy.belge_tipi), dy.belge_no))
                .unwrap_or_else(|| if f.dayanaksiz { "dayanaksız".into() } else { String::new() });
            (no, (i, belge))
        }))
        .collect();
    let hareketler = tumu[bas..son].iter().map(|h| KebirHareketDto {
        tarih: tarih_str(&h.tarih), yevmiye_no: h.yevmiye_no, fis_no: h.fis_no.clone(),
        aciklama: h.aciklama.clone(), karsi: h.karsi.clone(),
        borc: h.borc.deger(), alacak: h.alacak.deger(), yuruyen_bakiye: h.yuruyen_bakiye,
        fis_id: fis_ref.get(&h.fis_no).map(|(i, _)| *i),
        belge: fis_ref.get(&h.fis_no).map(|(_, b)| b.clone()).unwrap_or_default(),
    }).collect();
    Json(HareketSayfa { toplam, offset, hareketler })
}

fn tl_fmt(k: i64) -> String {
    let neg = k < 0;
    let mut n = (k.abs() / 100).to_string();
    // binlik ayraç (nokta)
    let mut s = String::new();
    while n.len() > 3 { let kalan = n.split_off(n.len() - 3); s = format!(".{}{}", kalan, s); }
    format!("{}{}{},{:02}", if neg { "-" } else { "" }, n, s, k.abs() % 100)
}

fn pad(s: &str, w: usize) -> String {
    let l = s.chars().count();
    if l >= w { s.chars().take(w).collect() } else { format!("{}{}", s, " ".repeat(w - l)) }
}

/// Muavin defterini düz metin (TXT) olarak dışa aktarır — klasik sabit genişlikli döküm.
async fn muavin_txt(State(state): State<Shared>, Query(q): Query<HashMap<String, String>>) -> ([(axum::http::HeaderName, String); 2], String) {
    let st = state.lock().unwrap();
    let d = defter_kur(&st.fisler);
    struct Blok { kod: String, ad: String, hareketler: Vec<KebirHareketDto>, toplam_borc: i64, toplam_alacak: i64, bakiye: i64 }
    let bloklar: Vec<Blok> = muavin_kodlari(&st, q.get("kod").map(|s| s.as_str()).filter(|s| !s.is_empty()))
        .into_iter()
        .map(|kod| {
            let hareketler: Vec<KebirHareketDto> = d.kebir(&kod).into_iter().map(|h| KebirHareketDto {
                tarih: tarih_str(&h.tarih), yevmiye_no: h.yevmiye_no, fis_no: h.fis_no,
                aciklama: h.aciklama, karsi: h.karsi,
                borc: h.borc.deger(), alacak: h.alacak.deger(), yuruyen_bakiye: h.yuruyen_bakiye,
                fis_id: None, belge: String::new(),
            }).collect();
            let tb: i64 = hareketler.iter().map(|h| h.borc).sum();
            let ta: i64 = hareketler.iter().map(|h| h.alacak).sum();
            let ad = st.hesaplar.get(&kod).map(|h| h.ad.clone()).unwrap_or_default();
            Blok { kod, ad, hareketler, toplam_borc: tb, toplam_alacak: ta, bakiye: tb - ta }
        })
        .collect();
    let mut out = String::new();
    out.push_str("MUAVİN DEFTERİ — Audit-Liners · Dönem 2026\n");
    out.push_str(&format!("{}\n\n", "=".repeat(132)));
    let mut gb = 0i64; let mut ga = 0i64;
    for b in &bloklar {
        out.push_str(&format!("HESAP: {} {}\n", b.kod, b.ad));
        out.push_str(&format!("{}\n", "-".repeat(132)));
        out.push_str(&format!("{}{}{}{}{}{:>14}{:>14}{:>16}\n",
            pad("Tarih", 12), pad("Yevmiye", 9), pad("Fiş No", 9), pad("Açıklama", 34), pad("Karşı Hesap", 22), "Borç", "Alacak", "Bakiye"));
        for h in &b.hareketler {
            let yon = if h.yuruyen_bakiye >= 0 { "B" } else { "A" };
            out.push_str(&format!("{}{}{}{}{}{:>14}{:>14}{:>14} {}\n",
                pad(&h.tarih, 12), pad(&h.yevmiye_no.map(|n| n.to_string()).unwrap_or_default(), 9),
                pad(&h.fis_no, 9), pad(&h.aciklama, 34), pad(&h.karsi, 22),
                if h.borc > 0 { tl_fmt(h.borc) } else { String::new() },
                if h.alacak > 0 { tl_fmt(h.alacak) } else { String::new() },
                tl_fmt(h.yuruyen_bakiye.abs()), yon));
        }
        let yon = if b.bakiye >= 0 { "B" } else { "A" };
        out.push_str(&format!("{}\n", "-".repeat(132)));
        out.push_str(&format!("{}{:>14}{:>14}{:>14} {}\n\n",
            pad("HESAP TOPLAMI", 86), tl_fmt(b.toplam_borc), tl_fmt(b.toplam_alacak), tl_fmt(b.bakiye.abs()), yon));
        gb += b.toplam_borc; ga += b.toplam_alacak;
    }
    out.push_str(&format!("{}\n", "=".repeat(132)));
    out.push_str(&format!("{}{:>14}{:>14}\n", pad(&format!("GENEL TOPLAM ({} hesap)", bloklar.len()), 86), tl_fmt(gb), tl_fmt(ga)));
    (
        [
            (axum::http::header::CONTENT_TYPE, "text/plain; charset=utf-8".to_string()),
            (axum::http::header::CONTENT_DISPOSITION, "attachment; filename=\"muavin-defteri.txt\"".to_string()),
        ],
        out,
    )
}

#[derive(Serialize)]
struct BilancoDto { donen: i64, duran: i64, aktif: i64, kvyk: i64, uvyk: i64, oz: i64, kar: i64, pasif: i64 }

async fn bilanco_uc(State(state): State<Shared>) -> Json<BilancoDto> {
    let st = state.lock().unwrap();
    let b = bilanco(&defter_kur(&st.fisler));
    Json(BilancoDto { donen: b.donen_varliklar, duran: b.duran_varliklar, aktif: b.aktif_toplam, kvyk: b.kisa_vadeli_yk, uvyk: b.uzun_vadeli_yk, oz: b.oz_kaynaklar, kar: b.donem_kari, pasif: b.pasif_toplam })
}

#[derive(Serialize)]
struct GtDto {
    brut_satislar: i64, satis_indirimleri: i64, net_satislar: i64, smm: i64, brut_kar: i64,
    faaliyet_giderleri: i64, faaliyet_kari: i64, diger_gelir: i64, diger_gider: i64,
    finansman: i64, olagan_kar: i64, od_gelir: i64, od_gider: i64,
    donem_kari: i64, vergi_karsiligi: i64, donem_net_kari: i64,
}

async fn gelir_tablosu_uc(State(state): State<Shared>) -> Json<GtDto> {
    let st = state.lock().unwrap();
    let g = gelir_tablosu(&defter_kur(&st.fisler));
    Json(GtDto {
        brut_satislar: g.brut_satislar, satis_indirimleri: g.satis_indirimleri, net_satislar: g.net_satislar,
        smm: g.satislarin_maliyeti, brut_kar: g.brut_satis_kari, faaliyet_giderleri: g.faaliyet_giderleri,
        faaliyet_kari: g.faaliyet_kari, diger_gelir: g.diger_olagan_gelir, diger_gider: g.diger_olagan_gider,
        finansman: g.finansman_giderleri, olagan_kar: g.olagan_kar, od_gelir: g.olagandisi_gelir,
        od_gider: g.olagandisi_gider, donem_kari: g.donem_kari, vergi_karsiligi: g.vergi_karsiligi,
        donem_net_kari: g.donem_net_kari,
    })
}

// ---- Finansal analiz: oranlar (doktrin) + dikey analiz + aylık karşılaştırma + yorum ----

#[derive(Serialize, Clone)]
struct Oran { grup: String, ad: String, deger: f64, birim: String, durum: String, yorum: String, banka: String }

/// Banka kredi tahsis perspektifi: oranların kredibilite/puanlama etkisi.
/// Kaynak: banka mali tahlil pratiği (kurumsal kredi derecelendirme kriterleri).
fn banka_yorumu(ad: &str, durum: &str) -> String {
    let (iyi, kotu): (&str, &str) = match ad {
        "Cari oran" => ("Bankalar ≥1,5'i pozitif puanlar; kredi limitini destekler.", "1'in altı kredi tahsisinde ciddi eksi puan — çoğu banka ek teminat ister."),
        "Asit-test (likidite) oranı" => ("≥1 banka skorlamasında güçlü likidite göstergesi.", "Düşük asit-test, stok finansmanı kredilerinde marjı daraltır."),
        "Nakit oranı" => ("Nakit tampon, nakit yönetimi ürünlerinde pazarlık gücü verir.", "Zayıf nakit oranı kısa vadeli kredi faizini yükseltebilir."),
        "Kaldıraç (yabancı kaynak/aktif)" => ("≤%50 kaldıraç bankalarca 'düşük riskli' sınıfa girer.", ">%70 kaldıraç çoğu bankada otomatik red/ek şart nedenidir."),
        "Borç / öz kaynak" => ("≤1 oran, borçlanma kapasitesinin sürdüğünü gösterir — limit artışı mümkün.", ">2 oran kredi komitesinde olumsuz; öz kaynak güçlendirme istenir."),
        "Öz kaynak / aktif" => ("Güçlü öz kaynak, teminatsız (açık) kredi şansını artırır.", "Zayıf öz kaynak → kefalet/ipotek şartı olasılığı yüksek."),
        "Alacak tahsil süresi" => ("Kısa tahsilat döngüsü, işletme kredisi skorunu yükseltir.", "Uzun tahsilat, alacak temliki/faktoring maliyetini artırır."),
        "Stok bekleme süresi" => ("Hızlı stok devri, stok rehinli kredilerde avantaj.", "Yavaş devir stok değerleme iskontosunu (haircut) artırır."),
        "Brüt kâr marjı" => ("Yüksek marj, faiz karşılama kapasitesini destekler.", "Dar marj, borç servis kapasitesi (DSCR) hesabında eksi."),
        "Faaliyet kâr marjı" => ("Ana faaliyet kârı bankaların baktığı ilk sürdürülebilirlik göstergesi.", "Faaliyet zararı kredi izlemede erken uyarı sinyalidir."),
        "Net kâr marjı" => ("Pozitif ve istikrarlı net marj kredi notunu yükseltir.", "Negatif/dalgalı net marj yakın izleme (watch list) nedeni."),
        "Aktif kârlılığı (ROA)" => ("Varlık verimliliği, uzun vadeli yatırım kredilerinde kilit skor.", "Düşük ROA, proje finansmanında özkaynak katkı şartını artırır."),
        "Öz kaynak kârlılığı (ROE)" => ("Güçlü ROE, ortakların işletmeyi büyütme kapasitesini gösterir.", "Zayıf ROE temettü/çekilme riskiyle birlikte değerlendirilir."),
        "SMM / net satışlar" => ("Kontrollü maliyet yapısı nakit akış projeksiyonunu güçlendirir.", "Yüksek maliyet oranı fiyat şoklarına kırılganlık demektir."),
        _ => ("", ""),
    };
    if durum == "iyi" { iyi.to_string() } else { kotu.to_string() }
}
#[derive(Serialize)]
struct DikeySatir { ad: String, tutar: i64, oran: f64 }
#[derive(Serialize)]
struct AylikSatir { ay: u8, net_satis: i64, net_kar: i64, cari_oran: f64, aktif: i64 }
#[derive(Serialize, Clone)]
struct KebirBilanco { kod: String, ad: String, taraf: String, bakiye: i64 }
#[derive(Serialize)]
struct KurAy { ay: u8, usd: f64, eur: f64, usd_eur: f64 }
#[derive(Serialize)]
struct KiyasDto { ay: u8, oranlar: Vec<Oran>, kebir_bilanco: Vec<KebirBilanco>, gt: GtDto, yorumlar: Vec<String> }
#[derive(Serialize)]
struct AnalizDto {
    ay: u8, oranlar: Vec<Oran>, kebir_bilanco: Vec<KebirBilanco>, gt: GtDto,
    dikey_bilanco: Vec<DikeySatir>, dikey_gt: Vec<DikeySatir>, aylik: Vec<AylikSatir>,
    kur: Vec<KurAy>, kiyas: Option<KiyasDto>,
}

/// 2026 örnek kur serisi (deterministik — fixture ile uyumlu). GERÇEK kaynak: TCMB EVDS (ileride bağlanacak).
fn kur_serisi() -> Vec<KurAy> {
    (1u8..=12).map(|ay| {
        let a = ay as f64;
        let usd = 41.2 + a * 0.62 + if ay % 3 == 0 { 0.35 } else { 0.0 };
        let eur = 44.9 + a * 0.78 + if ay % 4 == 0 { 0.4 } else { 0.0 };
        KurAy { ay, usd: (usd * 100.0).round() / 100.0, eur: (eur * 100.0).round() / 100.0, usd_eur: (usd / eur * 10000.0).round() / 10000.0 }
    }).collect()
}

fn oran_yap(grup: &str, ad: &str, deger: f64, birim: &str, esik: (f64, f64, bool), yorumlar: (&str, &str, &str)) -> Oran {
    // esik: (iyi_esigi, orta_esigi, buyuk_iyi: true→büyük değer iyi)
    let (iyi, orta, buyuk_iyi) = esik;
    let durum = if buyuk_iyi {
        if deger >= iyi { "iyi" } else if deger >= orta { "orta" } else { "riskli" }
    } else if deger <= iyi { "iyi" } else if deger <= orta { "orta" } else { "riskli" };
    let yorum = match durum { "iyi" => yorumlar.0, "orta" => yorumlar.1, _ => yorumlar.2 };
    let banka = banka_yorumu(ad, durum);
    Oran { grup: grup.into(), ad: ad.into(), deger, birim: birim.into(), durum: durum.into(), yorum: yorum.into(), banka }
}

/// Kebir bazlı bilanço: sınıf 1-5'in 3 haneli hesapları net bakiyeleriyle + dönem kârı satırı.
fn kebir_bilanco_hesapla(st: &AppState, d: &Defter) -> Vec<KebirBilanco> {
    let mut g: HashMap<String, i64> = HashMap::new();
    for m in d.mizan() {
        let sinif = m.hesap_id.as_bytes()[0] - b'0';
        if (1..=5).contains(&sinif) {
            let kebir = m.hesap_id.split('.').next().unwrap_or(&m.hesap_id).to_string();
            *g.entry(kebir).or_insert(0) += m.borc_bakiye.deger() - m.alacak_bakiye.deger();
        }
    }
    let mut v: Vec<KebirBilanco> = g.into_iter().filter(|(_, n)| *n != 0).map(|(kod, net)| {
        let sinif = kod.as_bytes()[0] - b'0';
        let aktif = sinif <= 2;
        KebirBilanco {
            ad: st.hesaplar.get(&kod).map(|h| h.ad.clone()).unwrap_or_default(),
            kod, taraf: if aktif { "Aktif".into() } else { "Pasif".into() },
            bakiye: if aktif { net } else { -net }, // pasif alacak bakiyesi pozitif gösterilir
        }
    }).collect();
    v.sort_by(|a, b| a.kod.cmp(&b.kod));
    let kar = donem_sonucu_pub(d);
    if kar != 0 {
        v.push(KebirBilanco { kod: "—".into(), ad: "Dönem net kârı/zararı (sonuç hesaplarından)".into(), taraf: "Pasif".into(), bakiye: kar });
    }
    v
}

fn donem_sonucu_pub(d: &Defter) -> i64 { domain::donem_sonucu(d) - gelir_tablosu(d).vergi_karsiligi }

fn gt_dto(d: &Defter) -> GtDto {
    let g = gelir_tablosu(d);
    GtDto {
        brut_satislar: g.brut_satislar, satis_indirimleri: g.satis_indirimleri, net_satislar: g.net_satislar,
        smm: g.satislarin_maliyeti, brut_kar: g.brut_satis_kari, faaliyet_giderleri: g.faaliyet_giderleri,
        faaliyet_kari: g.faaliyet_kari, diger_gelir: g.diger_olagan_gelir, diger_gider: g.diger_olagan_gider,
        finansman: g.finansman_giderleri, olagan_kar: g.olagan_kar, od_gelir: g.olagandisi_gelir,
        od_gider: g.olagandisi_gider, donem_kari: g.donem_kari, vergi_karsiligi: g.vergi_karsiligi,
        donem_net_kari: g.donem_net_kari,
    }
}

/// Karşılaştırma yorum motoru: oran değişimleri + kur bağlamı + süreklilik değerlendirmesi.
fn kiyas_yorumlari(simdi: &[Oran], once: &[Oran], ay: u8, kiyas_ay: u8, kur: &[KurAy]) -> Vec<String> {
    let bul = |list: &[Oran], ad: &str| list.iter().find(|o| o.ad == ad).map(|o| o.deger);
    let mut y = Vec::new();
    let usd_d = {
        let s = kur.iter().find(|k| k.ay == ay).map(|k| k.usd).unwrap_or(0.0);
        let o = kur.iter().find(|k| k.ay == kiyas_ay).map(|k| k.usd).unwrap_or(s);
        if o > 0.0 { 100.0 * (s - o) / o } else { 0.0 }
    };
    for (ad, buyuk_iyi, konu) in [
        ("Cari oran", true, "likidite"), ("Kaldıraç (yabancı kaynak/aktif)", false, "borçluluk"),
        ("Brüt kâr marjı", true, "satış kârlılığı"), ("Net kâr marjı", true, "net kârlılık"),
        ("Alacak tahsil süresi", false, "tahsilat hızı"),
    ] {
        if let (Some(s), Some(o)) = (bul(simdi, ad), bul(once, ad)) {
            if o.abs() < 0.0001 { continue; }
            // Yuvarlama içi (<%1) fark → yön yorumu yapma, "yatay seyir" de
            if ((s - o) / o).abs() < 0.01 {
                y.push(format!("{}: {:.2} → {:.2} (yatay seyir). {} istikrarlı.", ad, o, s, konu));
                continue;
            }
            let yon = s > o;
            let iyilesme = yon == buyuk_iyi;
            y.push(format!(
                "{}: {:.2} → {:.2} ({}). {}",
                ad, o, s,
                if yon { "yükseliş" } else { "düşüş" },
                if iyilesme { format!("{} güçleniyor — işletme sürekliliği lehine.", konu) }
                else { format!("{} zayıflıyor — trend sürerse izleme gerekir.", konu) }
            ));
        }
    }
    // Kur bağlamı
    if let (Some(bs), Some(bo)) = (bul(simdi, "Brüt kâr marjı"), bul(once, "Brüt kâr marjı")) {
        if usd_d > 3.0 {
            // %1 içi fark = korunmuş sayılır (yuvarlama gürültüsü yön yorumu üretmesin)
            if bs >= bo || (bo != 0.0 && ((bs - bo) / bo).abs() < 0.01) {
                y.push(format!("Kur bağlamı: USD/TL kıyas döneminden bu yana %{:.1} yükseldi; buna rağmen brüt marj korunmuş/iyileşmiş — fiyatlama gücü ve kur geçişkenliği yönetimi olumlu.", usd_d));
            } else {
                y.push(format!("Kur bağlamı: USD/TL %{:.1} yükselirken brüt marj daralmış — ithal maliyet baskısı olası; döviz cinsi alım/satış dengesi ve kur riski (forward/opsiyon) değerlendirilmeli.", usd_d));
            }
        }
    }
    if let Some(c) = bul(simdi, "Cari oran") {
        if c < 1.0 { y.push("SÜREKLİLİK UYARISI: Cari oran 1'in altında — kısa vadeli yükümlülükler dönen varlıkları aşıyor; nakit planı ve borç yapılandırması öncelikli.".into()); }
    }
    y.push("Sektör bağlamı: e-ticaret/perakendede stok devri ve POS tahsilat döngüsü sektör ortalamasına göre değerlendirilmelidir (sektör şablonları eklendiğinde kıyas otomatikleşecek).".into());
    y
}

/// Doktrin oranları (Brigham/Gitman; genel muhasebe analiz bölümü) — verilen defter durumundan.
fn oranlari_hesapla(d: &Defter) -> Vec<Oran> {
    let net = |onek: &str| { let (b, a) = d.bakiye_rollup(onek); b.deger() - a.deger() };
    let f = |x: i64| x as f64;
    let bil = bilanco(d);
    let gt = gelir_tablosu(d);
    let stok = net("15");
    let hazir = net("10") + net("11");
    let tic_alacak = net("12");
    let kvyk = bil.kisa_vadeli_yk.max(1);
    let aktif = bil.aktif_toplam.max(1);
    let oz = (bil.oz_kaynaklar + bil.donem_kari).max(1);
    let yab = bil.kisa_vadeli_yk + bil.uzun_vadeli_yk;
    let net_satis = gt.net_satislar.max(1);
    let mut o = vec![
        oran_yap("Likidite", "Cari oran", f(bil.donen_varliklar) / f(kvyk), "x", (2.0, 1.0, true),
            ("Kısa vadeli borç ödeme gücü güçlü (ideal ≥2).", "Yeterli ancak izlenmeli (1-2 arası).", "Dönen varlıklar kısa vadeli borçları karşılamıyor (<1) — likidite riski.")),
        oran_yap("Likidite", "Asit-test (likidite) oranı", f(bil.donen_varliklar - stok) / f(kvyk), "x", (1.0, 0.5, true),
            ("Stoklara bağımlı olmadan borç ödenebilir (≥1).", "Stok satışına kısmen bağımlı.", "Stoksuz likidite zayıf — tahsilat/nakit planı gerekli.")),
        oran_yap("Likidite", "Nakit oranı", f(hazir) / f(kvyk), "x", (0.2, 0.1, true),
            ("Ani yükümlülükler için nakit tamponu yeterli (≥0,2).", "Nakit tamponu sınırlı.", "Nakit tamponu çok zayıf.")),
        oran_yap("Finansal yapı", "Kaldıraç (yabancı kaynak/aktif)", f(yab) / f(aktif), "x", (0.5, 0.7, false),
            ("Varlıkların yarıdan azı borçla finanse — dengeli (≤0,5).", "Borç ağırlığı yükseliyor.", "Aşırı borçlanma (>0,7) — finansman riski.")),
        oran_yap("Finansal yapı", "Borç / öz kaynak", f(yab) / f(oz), "x", (1.0, 2.0, false),
            ("Öz kaynak borçları karşılıyor (≤1).", "Borçlar öz kaynağı aşıyor.", "Öz kaynağın 2 katından fazla borç — yapısal risk.")),
        oran_yap("Finansal yapı", "Öz kaynak / aktif", 100.0 * f(oz) / f(aktif), "%", (50.0, 30.0, true),
            ("Güçlü öz kaynak yapısı (≥%50).", "Orta düzey öz kaynak.", "Öz kaynak zayıf (<%30).")),
        oran_yap("Devir hızları", "Alacak tahsil süresi", if tic_alacak > 0 { 365.0 * f(tic_alacak) / f(net_satis) } else { 0.0 }, "gün", (45.0, 90.0, false),
            ("Alacaklar hızlı tahsil ediliyor (≤45 gün).", "Tahsilat süresi uzuyor.", "Tahsilat çok yavaş (>90 gün) — şüpheli alacak riski.")),
        oran_yap("Devir hızları", "Stok bekleme süresi", if stok > 0 && gt.satislarin_maliyeti > 0 { 365.0 * f(stok) / f(gt.satislarin_maliyeti) } else { 0.0 }, "gün", (60.0, 120.0, false),
            ("Stok hızlı dönüyor (≤60 gün).", "Stok devri yavaşlıyor.", "Stok >120 gün bekliyor — değer düşüklüğü riski.")),
        oran_yap("Kârlılık", "Brüt kâr marjı", 100.0 * f(gt.brut_satis_kari) / f(net_satis), "%", (30.0, 15.0, true),
            ("Satış kârlılığı güçlü.", "Marj orta — fiyat/maliyet baskısı izlenmeli.", "Marj zayıf — maliyet yapısı gözden geçirilmeli.")),
        oran_yap("Kârlılık", "Faaliyet kâr marjı", 100.0 * f(gt.faaliyet_kari) / f(net_satis), "%", (15.0, 5.0, true),
            ("Ana faaliyet kârlı.", "Faaliyet giderleri marjı daraltıyor.", "Faaliyet zararı/çok dar marj.")),
        oran_yap("Kârlılık", "Net kâr marjı", 100.0 * f(gt.donem_net_kari) / f(net_satis), "%", (10.0, 3.0, true),
            ("Net kârlılık güçlü.", "Net marj sınırlı.", "Net kârlılık zayıf/negatif.")),
        oran_yap("Kârlılık", "Aktif kârlılığı (ROA)", 100.0 * f(gt.donem_net_kari) / f(aktif), "%", (10.0, 4.0, true),
            ("Varlıklar verimli kullanılıyor.", "Varlık verimliliği orta.", "Varlıklar yeterli getiri üretmiyor.")),
        oran_yap("Kârlılık", "Öz kaynak kârlılığı (ROE)", 100.0 * f(gt.donem_net_kari) / f(oz), "%", (20.0, 8.0, true),
            ("Ortaklara güçlü getiri.", "Getiri orta.", "Öz kaynak getirisi zayıf.")),
    ];
    o.push(oran_yap("Kârlılık", "SMM / net satışlar", 100.0 * f(gt.satislarin_maliyeti) / f(net_satis), "%", (70.0, 85.0, false),
        ("Maliyet yapısı sağlıklı.", "Maliyet oranı yüksek.", "Maliyet satışları yutuyor.")));
    o
}

async fn analiz(State(state): State<Shared>, Query(q): Query<HashMap<String, String>>) -> Json<AnalizDto> {
    let st = state.lock().unwrap();
    let ay: u8 = q.get("ay").and_then(|s| s.parse().ok()).filter(|a| (1..=12).contains(a)).unwrap_or(12);
    let kiyas_ay: Option<u8> = q.get("kiyas").and_then(|s| s.parse().ok()).filter(|a| (1..=12).contains(a) && *a != ay);
    let donem_defteri = |son_ay: u8| { let mut dd = Defter::yeni(); for fi in st.fisler.iter().filter(|x| x.tarih.ay <= son_ay) { dd.ekle(fi.clone()); } dd };
    let d = donem_defteri(ay);
    let net = |onek: &str| { let (b, a) = d.bakiye_rollup(onek); b.deger() - a.deger() };
    let f = |x: i64| x as f64;

    let bil = bilanco(&d);
    let gt = gelir_tablosu(&d);
    let stok = net("15");
    let hazir = net("10") + net("11");
    let tic_alacak = net("12");
    let kvyk = bil.kisa_vadeli_yk.max(1);
    let aktif = bil.aktif_toplam.max(1);
    let oz = (bil.oz_kaynaklar + bil.donem_kari).max(1);
    let yab = bil.kisa_vadeli_yk + bil.uzun_vadeli_yk;
    let net_satis = gt.net_satislar.max(1);

    let o = oranlari_hesapla(&d);

    // Dikey analiz — bilanço (aktife oran) ve GT (net satışa oran)
    let db = vec![
        ("Dönen varlıklar", bil.donen_varliklar), ("· Hazır değerler (10-11)", hazir), ("· Ticari alacaklar (12)", tic_alacak),
        ("· Stoklar (15)", stok), ("Duran varlıklar", bil.duran_varliklar),
        ("Kısa vadeli yabancı kaynaklar", bil.kisa_vadeli_yk), ("Uzun vadeli yabancı kaynaklar", bil.uzun_vadeli_yk),
        ("Öz kaynaklar (dönem kârı dahil)", bil.oz_kaynaklar + bil.donem_kari),
    ].into_iter().map(|(ad, t)| DikeySatir { ad: ad.into(), tutar: t, oran: 100.0 * f(t) / f(aktif) }).collect();
    let dg = vec![
        ("Brüt satışlar", gt.brut_satislar), ("Satış indirimleri (-)", gt.satis_indirimleri), ("Net satışlar", gt.net_satislar),
        ("Satışların maliyeti (-)", gt.satislarin_maliyeti), ("Brüt satış kârı", gt.brut_satis_kari),
        ("Faaliyet giderleri (-)", gt.faaliyet_giderleri), ("Faaliyet kârı", gt.faaliyet_kari),
        ("Finansman giderleri (-)", gt.finansman_giderleri), ("Dönem kârı (vergi öncesi)", gt.donem_kari),
        ("Dönem net kârı", gt.donem_net_kari),
    ].into_iter().map(|(ad, t)| DikeySatir { ad: ad.into(), tutar: t, oran: 100.0 * f(t) / f(net_satis) }).collect();

    // Aylık karşılaştırma: ay sonuna kadar kümülatif defter → bilanço; GT aylık fark
    let mut aylik = Vec::new();
    let mut onceki_satis = 0i64;
    let mut onceki_kar = 0i64;
    for a in 1u8..=12 {
        let da = donem_defteri(a);
        if da.fisler.is_empty() { continue; }
        let b = bilanco(&da);
        let g = gelir_tablosu(&da);
        aylik.push(AylikSatir {
            ay: a,
            net_satis: g.net_satislar - onceki_satis,
            net_kar: g.donem_net_kari - onceki_kar,
            cari_oran: f(b.donen_varliklar) / f(b.kisa_vadeli_yk.max(1)),
            aktif: b.aktif_toplam,
        });
        onceki_satis = g.net_satislar;
        onceki_kar = g.donem_net_kari;
    }

    let kur = kur_serisi();
    // Kıyas dönemi: aynı hesaplamalar kıyas ayına kadar kümülatif defterle
    let kiyas = kiyas_ay.map(|ka| {
        let dk = donem_defteri(ka);
        let ok = oranlari_hesapla(&dk);
        KiyasDto {
            ay: ka,
            yorumlar: kiyas_yorumlari(&o, &ok, ay, ka, &kur),
            oranlar: ok,
            kebir_bilanco: kebir_bilanco_hesapla(&st, &dk),
            gt: gt_dto(&dk),
        }
    });

    Json(AnalizDto {
        ay,
        kebir_bilanco: kebir_bilanco_hesapla(&st, &d),
        gt: gt_dto(&d),
        oranlar: o, dikey_bilanco: db, dikey_gt: dg, aylik, kur, kiyas,
    })
}

// ---- Banka ekstresi + gelen belge kutusu (eşleştirme) ----

#[derive(Clone)]
struct BankaHareket { tarih: Tarih, aciklama: String, tutar: i64, fis_id: Option<usize> }
#[derive(Clone)]
struct GelenBelge { tip: String, no: String, tarih: Tarih, gonderen: String, tutar: i64, fis_id: Option<usize> }

fn gun_no(t: &Tarih) -> i64 { t.yil as i64 * 372 + t.ay as i64 * 31 + t.gun as i64 }

#[derive(Deserialize)]
struct EkstreSatirGirdi { tarih: String, aciklama: String, tutar: i64 }

#[derive(Serialize)]
struct BankaDto { id: usize, tarih: String, aciklama: String, tutar: i64, fis_no: Option<String> }

async fn banka_ekstre_yukle(State(state): State<Shared>, Json(satirlar): Json<Vec<EkstreSatirGirdi>>) -> Result<Json<serde_json::Value>, (StatusCode, Json<HataDto>)> {
    let hata = |m: String| (StatusCode::BAD_REQUEST, Json(HataDto { hata: m }));
    let mut st = state.lock().unwrap();
    let mut n = 0;
    for s in satirlar {
        let tarih = parse_tarih(&s.tarih).ok_or_else(|| hata(format!("geçersiz tarih: {}", s.tarih)))?;
        st.banka.push(BankaHareket { tarih, aciklama: s.aciklama, tutar: s.tutar, fis_id: None });
        n += 1;
    }
    Ok(Json(serde_json::json!({ "yuklenen": n })))
}

async fn banka_listesi(State(state): State<Shared>) -> Json<Vec<BankaDto>> {
    let st = state.lock().unwrap();
    Json(st.banka.iter().enumerate().map(|(i, h)| BankaDto {
        id: i, tarih: tarih_str(&h.tarih), aciklama: h.aciklama.clone(), tutar: h.tutar,
        fis_no: h.fis_id.and_then(|f| st.fisler.get(f)).and_then(fis_no),
    }).collect())
}

#[derive(Serialize)]
struct OneriDto { fis_id: usize, fis_no: String, tarih: String, aciklama: String, tutar: i64 }

/// Ekstre satırı için aday fişler: 102* hesabında AYNI tutar (yönü doğru) + ±3 gün.
async fn banka_oneri(State(state): State<Shared>, Path(id): Path<usize>) -> Result<Json<Vec<OneriDto>>, StatusCode> {
    let st = state.lock().unwrap();
    let h = st.banka.get(id).ok_or(StatusCode::NOT_FOUND)?;
    let eslenen: std::collections::HashSet<usize> = st.banka.iter().filter_map(|x| x.fis_id).collect();
    let mut v = Vec::new();
    for (i, f) in st.fisler.iter().enumerate() {
        if eslenen.contains(&i) { continue; }
        if (gun_no(&f.tarih) - gun_no(&h.tarih)).abs() > 3 { continue; }
        let uyan = f.satirlar.iter().any(|s| s.hesap_id.starts_with("102")
            && ((h.tutar > 0 && s.borc.deger() == h.tutar) || (h.tutar < 0 && s.alacak.deger() == -h.tutar)));
        if uyan {
            v.push(OneriDto { fis_id: i, fis_no: fis_no(f).unwrap_or_default(), tarih: tarih_str(&f.tarih), aciklama: f.aciklama.clone(), tutar: f.toplam_borc().deger() });
        }
        if v.len() >= 10 { break; }
    }
    Ok(Json(v))
}

#[derive(Deserialize)]
struct EsleGirdi { fis_id: usize }

async fn banka_esle(State(state): State<Shared>, Path(id): Path<usize>, Json(g): Json<EsleGirdi>) -> Result<Json<serde_json::Value>, (StatusCode, Json<HataDto>)> {
    let hata = |m: String| (StatusCode::BAD_REQUEST, Json(HataDto { hata: m }));
    let mut st = state.lock().unwrap();
    if g.fis_id >= st.fisler.len() { return Err(hata("fiş yok".into())); }
    let h = st.banka.get_mut(id).ok_or_else(|| hata("ekstre satırı yok".into()))?;
    h.fis_id = Some(g.fis_id);
    Ok(Json(serde_json::json!({ "ok": true })))
}

#[derive(Deserialize)]
struct BelgeGirdi { tip: String, no: String, tarih: String, gonderen: String, tutar: i64 }
#[derive(Serialize)]
struct BelgeDto { id: usize, tip: String, no: String, tarih: String, gonderen: String, tutar: i64, fis_no: Option<String> }

async fn belge_ekle(State(state): State<Shared>, Json(g): Json<BelgeGirdi>) -> Result<Json<serde_json::Value>, (StatusCode, Json<HataDto>)> {
    let hata = |m: String| (StatusCode::BAD_REQUEST, Json(HataDto { hata: m }));
    let tarih = parse_tarih(&g.tarih).ok_or_else(|| hata("geçersiz tarih".into()))?;
    let mut st = state.lock().unwrap();
    if st.gelen_belgeler.iter().any(|b| b.no == g.no) { return Err(hata(format!("mükerrer belge no: {}", g.no))); }
    st.gelen_belgeler.push(GelenBelge { tip: g.tip, no: g.no, tarih, gonderen: g.gonderen, tutar: g.tutar, fis_id: None });
    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn belge_listesi(State(state): State<Shared>) -> Json<Vec<BelgeDto>> {
    let st = state.lock().unwrap();
    Json(st.gelen_belgeler.iter().enumerate().map(|(i, b)| BelgeDto {
        id: i, tip: b.tip.clone(), no: b.no.clone(), tarih: tarih_str(&b.tarih), gonderen: b.gonderen.clone(), tutar: b.tutar,
        fis_no: b.fis_id.and_then(|f| st.fisler.get(f)).and_then(fis_no),
    }).collect())
}

/// Belge için aday fişler: fiş toplamı = belge tutarı, ±5 gün.
async fn belge_oneri(State(state): State<Shared>, Path(id): Path<usize>) -> Result<Json<Vec<OneriDto>>, StatusCode> {
    let st = state.lock().unwrap();
    let b = st.gelen_belgeler.get(id).ok_or(StatusCode::NOT_FOUND)?;
    let mut v = Vec::new();
    for (i, f) in st.fisler.iter().enumerate() {
        if (gun_no(&f.tarih) - gun_no(&b.tarih)).abs() > 5 { continue; }
        if f.toplam_borc().deger() == b.tutar {
            v.push(OneriDto { fis_id: i, fis_no: fis_no(f).unwrap_or_default(), tarih: tarih_str(&f.tarih), aciklama: f.aciklama.clone(), tutar: f.toplam_borc().deger() });
        }
        if v.len() >= 10 { break; }
    }
    Ok(Json(v))
}

async fn belge_esle(State(state): State<Shared>, Path(id): Path<usize>, Json(g): Json<EsleGirdi>) -> Result<Json<serde_json::Value>, (StatusCode, Json<HataDto>)> {
    let hata = |m: String| (StatusCode::BAD_REQUEST, Json(HataDto { hata: m }));
    let mut st = state.lock().unwrap();
    if g.fis_id >= st.fisler.len() { return Err(hata("fiş yok".into())); }
    let b = st.gelen_belgeler.get_mut(id).ok_or_else(|| hata("belge yok".into()))?;
    b.fis_id = Some(g.fis_id);
    Ok(Json(serde_json::json!({ "ok": true })))
}

// ---- Fatura / e-fatura yaşam döngüsü ----

#[derive(Deserialize)]
struct FaturaSatirGirdi { aciklama: String, hesap_kod: String, miktar: i64, birim_fiyat: i64, kdv_orani: u8, istisna_kodu: Option<String> }
#[derive(Deserialize)]
struct FaturaGirdi { yon: String, cari_kod: String, tarih: String, no: String, satirlar: Vec<FaturaSatirGirdi> }
#[derive(Serialize)]
struct FaturaOzet { id: usize, no: String, yon: String, cari_kod: String, tarih: String, matrah: i64, kdv: i64, toplam: i64, durum: String, efatura_uuid: Option<String>, fis_no: Option<String>, belge_var: bool }

fn durum_str(d: &FaturaDurum) -> &'static str {
    match d { FaturaDurum::Taslak => "Taslak", FaturaDurum::Gonderildi => "Gönderildi", FaturaDurum::Kesinlesti => "Kesinleşti", FaturaDurum::Reddedildi => "Reddedildi" }
}

fn fatura_ozet(i: usize, f: &Fatura, fis: Option<&String>) -> FaturaOzet {
    FaturaOzet {
        id: i, no: f.no.clone(),
        yon: if f.yon == FaturaYonu::Satis { "Satış".into() } else { "Alış".into() },
        cari_kod: f.cari_kod.clone(), tarih: tarih_str(&f.tarih),
        matrah: f.matrah(), kdv: f.toplam_kdv(), toplam: f.genel_toplam(),
        durum: durum_str(&f.durum).into(), efatura_uuid: f.efatura_uuid.clone(),
        fis_no: fis.cloned(), belge_var: f.belge_ref.is_some(),
    }
}

async fn fatura_olustur(State(state): State<Shared>, Json(g): Json<FaturaGirdi>) -> Result<Json<FaturaOzet>, (StatusCode, Json<HataDto>)> {
    let hata = |m: String| (StatusCode::BAD_REQUEST, Json(HataDto { hata: m }));
    let tarih = parse_tarih(&g.tarih).ok_or_else(|| hata("geçersiz tarih".into()))?;
    let yon = match g.yon.as_str() { "Satis" | "Satış" => FaturaYonu::Satis, "Alis" | "Alış" => FaturaYonu::Alis, _ => return Err(hata("yon: Satis|Alis".into())) };
    let mut st = state.lock().unwrap();
    if st.faturalar.iter().any(|f| f.no == g.no) {
        return Err(hata(format!("mükerrer fatura no: {}", g.no)));
    }
    let f = Fatura {
        no: g.no, yon, cari_kod: g.cari_kod, tarih,
        satirlar: g.satirlar.into_iter().map(|s| FaturaSatiri { aciklama: s.aciklama, hesap_kod: s.hesap_kod, miktar: s.miktar, birim_fiyat: Kurus::yeni(s.birim_fiyat), kdv_orani: s.kdv_orani, istisna_kodu: s.istisna_kodu }).collect(),
        durum: FaturaDurum::Taslak, efatura_uuid: None, belge_ref: None,
    };
    let i = st.faturalar.len();
    st.faturalar.push(f);
    Ok(Json(fatura_ozet(i, &st.faturalar[i], None)))
}

/// Entegratöre gönderim "call"u. MOCK: uuid üretir, Gönderildi yapar (gerçek entegratör portu buraya takılır).
async fn fatura_gonder(State(state): State<Shared>, Path(id): Path<usize>) -> Result<Json<FaturaOzet>, (StatusCode, Json<HataDto>)> {
    let hata = |m: String| (StatusCode::BAD_REQUEST, Json(HataDto { hata: m }));
    let mut st = state.lock().unwrap();
    let f = st.faturalar.get_mut(id).ok_or_else(|| hata("fatura yok".into()))?;
    if f.durum != FaturaDurum::Taslak { return Err(hata(format!("yalnız taslak gönderilir (durum: {})", durum_str(&f.durum)))); }
    f.efatura_uuid = Some(format!("EF-{}-{:06}", f.no.replace(' ', ""), id));
    f.durum = FaturaDurum::Gonderildi;
    Ok(Json(fatura_ozet(id, f, None)))
}

/// E-fatura sisteminde kesinleşme bildirimi (webhook/simülasyon):
/// belgeyi arşivler + OTOMATİK muhasebe fişi üretip kesinleştirir.
async fn fatura_kesinlesti(State(state): State<Shared>, Path(id): Path<usize>) -> Result<Json<FaturaOzet>, (StatusCode, Json<HataDto>)> {
    let hata = |m: String| (StatusCode::BAD_REQUEST, Json(HataDto { hata: m }));
    let mut guard = state.lock().unwrap();
    let st = &mut *guard;
    let f = st.faturalar.get_mut(id).ok_or_else(|| hata("fatura yok".into()))?;
    if f.durum != FaturaDurum::Gonderildi { return Err(hata(format!("yalnız gönderilmiş fatura kesinleşir (durum: {})", durum_str(&f.durum)))); }
    f.durum = FaturaDurum::Kesinlesti;

    // Orijinal belgeyi arşivle (UBL benzeri; gerçek sistemde entegratörden gelen XML/PDF saklanır)
    let belge_ref = format!("arsiv/{}.xml", f.no);
    let ubl = format!(
        "<Invoice><ID>{}</ID><UUID>{}</UUID><IssueDate>{}</IssueDate><LegalMonetaryTotal><PayableAmount>{}</PayableAmount></LegalMonetaryTotal></Invoice>",
        f.no, f.efatura_uuid.clone().unwrap_or_default(), tarih_str(&f.tarih), f.genel_toplam()
    );
    f.belge_ref = Some(belge_ref.clone());
    st.belgeler.insert(belge_ref, ubl);

    // Otomatik fiş: üret + kesinleştir
    let mut fis = fatura_fisi(&st.faturalar[id]).ok_or_else(|| hata("fiş üretilemedi".into()))?;
    let sonuc = {
        let ctx = Baglam { hesaplar: &st.hesaplar, donem: &st.donem };
        kesinlestir(&mut fis, &ctx, &mut st.sayac)
    };
    match sonuc {
        Ok(()) => {
            let no = fis_no(&fis).unwrap_or_default();
            st.fisler.push(fis);
            st.fatura_fisleri.insert(id, no.clone());
            Ok(Json(fatura_ozet(id, &st.faturalar[id], Some(&no))))
        }
        Err(e) => Err(hata(format!("fiş kesinleştirilemedi: {} (hesap planında alt hesaplar açık mı?)", e))),
    }
}

async fn faturalar(State(state): State<Shared>) -> Json<Vec<FaturaOzet>> {
    let st = state.lock().unwrap();
    Json(st.faturalar.iter().enumerate().map(|(i, f)| fatura_ozet(i, f, st.fatura_fisleri.get(&i))).collect())
}

/// Orijinal belge (arşivden) — "işlenen faturanın orijinalini istediğimizde görebilelim".
async fn fatura_belge(State(state): State<Shared>, Path(id): Path<usize>) -> Result<String, StatusCode> {
    let st = state.lock().unwrap();
    let f = st.faturalar.get(id).ok_or(StatusCode::NOT_FOUND)?;
    let r = f.belge_ref.as_ref().ok_or(StatusCode::NOT_FOUND)?;
    st.belgeler.get(r).cloned().ok_or(StatusCode::NOT_FOUND)
}

// ---- Örnek veri: 20.000 kayıtlık e-ticaret işletmesi (kronolojik, vergiye giden yol) ----

/// Deterministik LCG — testler tekrarlanabilir olsun (rastgelelik bağımlılığı yok).
struct Lcg(u64);
impl Lcg {
    fn yeni() -> Self { Lcg(0x2026_1453) }
    fn u(&mut self, n: u64) -> u64 { self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407); (self.0 >> 33) % n }
}

fn ay_gun(ay: u8) -> u8 { match ay { 1|3|5|7|8|10|12 => 31, 4|6|9|11 => 30, _ => 28 } }

/// E-ticaret 2026 senaryosu: günlük POS satışları (%20/%10 KDV karışık) + günlük SMM,
/// haftalık mal alışı + POS tahsilatı (komisyonlu) + tedarikçi ödemesi,
/// aylık kira/maaş/kargo/reklam + ay sonu KDV mahsubu. ~20.000 fiş, tamamı kronolojik.
async fn ornek_veri(State(state): State<Shared>) -> Result<Json<serde_json::Value>, (StatusCode, Json<HataDto>)> {
    let hata = |m: String| (StatusCode::BAD_REQUEST, Json(HataDto { hata: m }));
    {
        let st = state.lock().unwrap();
        if !st.fisler.is_empty() { return Err(hata("defter boş değil — örnek veri yalnız boş deftere yüklenir".into())); }
    }
    let mut rng = Lcg::yeni();
    // Muavinler (e-ticaret hesap planı)
    let muavinler: &[(&str, &str, &str)] = &[
        ("102", "01", "Banka TL"), ("108", "01", "Sanal POS bekleyen"),
        ("153", "01", "Elektronik"), ("153", "02", "Giyim"), ("153", "03", "Ev & Yaşam"),
        ("191", "10", "İnd. KDV %10"), ("191", "20", "İnd. KDV %20"),
        ("391", "10", "Hes. KDV %10"), ("391", "20", "Hes. KDV %20"),
        ("320", "01", "Toptancı A"), ("320", "02", "Toptancı B"), ("320", "03", "Kargo A.Ş."),
        ("600", "01", "Web satış"), ("600", "02", "Pazaryeri satış"),
        ("621", "01", "Satılan mal maliyeti"),
        ("760", "01", "Kargo gideri"), ("760", "02", "Pazaryeri komisyonu"), ("760", "03", "Reklam"),
        ("770", "01", "Kira"), ("770", "02", "Maaş"), ("770", "03", "Amortisman"),
        ("780", "01", "Kredi faizi"),
        // ── UFRS çalışmalarının alt plan kırılımı (denetçi alt hesap bazında çalışsın) ──
        ("120", "01", "Kurumsal bayi"), ("120", "02", "Zincir market"), ("120", "03", "İhracat müşterisi"),
        ("121", "01", "Müşteri senetleri"), ("121", "02", "Bayi senetleri"),
        ("128", "01", "Dava aşamasında"), ("128", "02", "İcra takibinde"),
        ("129", "01", "Dava karşılığı"), ("129", "02", "İcra karşılığı"),
        // 2. seviye: varlık grubu
        ("252", "01", "Depo binası"),
        ("252", "02", "İdari bina"),
        ("253", "01", "Üretim hattı"),
        ("253", "02", "Paketleme makineleri"),
        ("253", "03", "İstif ve taşıma ekipmanı"),
        ("254", "01", "Binek araçlar"),
        ("254", "02", "Ticari araçlar"),
        ("255", "01", "Ofis mobilyası"),
        ("255", "02", "Bilgisayar ve donanım"),
        // 3. seviye: TEK TEK VARLIKLAR (data/duran-varlik-envanteri.json) — üretim hattının 10 bileşeni,
        // 6 binek oto ve tamamlayıcıları ayrı hesapta; amortisman/GUD/değer düşüklüğü varlık bazında koşar.
        ("252.01", "001", "Depo binası — ana yapı"),
        ("252.01", "901", "Depo raf sistemi (tamamlayıcı)"),
        ("252.01", "902", "Yangın söndürme sistemi (tamamlayıcı)"),
        ("252.02", "001", "İdari bina"),
        ("252.02", "901", "İklimlendirme (VRF) sistemi (tamamlayıcı)"),
        ("253.01", "001", "CNC torna tezgahı"),
        ("253.01", "002", "CNC dik işleme merkezi"),
        ("253.01", "003", "Hidrolik pres"),
        ("253.01", "004", "Robot kaynak hücresi"),
        ("253.01", "005", "Konveyör bant hattı"),
        ("253.01", "006", "Vidalı hava kompresörü"),
        ("253.01", "007", "Kalite kontrol ölçüm istasyonu"),
        ("253.01", "008", "Otomasyon panosu ve PLC"),
        ("253.01", "901", "Kalıp ve aparat seti (tamamlayıcı)"),
        ("253.01", "902", "Kesici takım seti (tamamlayıcı)"),
        ("253.02", "001", "Otomatik dolum makinesi"),
        ("253.02", "002", "Etiketleme makinesi"),
        ("253.02", "003", "Shrink paketleme makinesi"),
        ("253.03", "001", "Forklift (dizel, 3 ton)"),
        ("253.03", "002", "Transpalet (akülü)"),
        ("253.03", "901", "Forklift ataşmanları (tamamlayıcı)"),
        ("254.01", "001", "Binek oto — Genel Müdür"),
        ("254.01", "002", "Binek oto — Mali İşler Müdürü"),
        ("254.01", "003", "Binek oto — Satış Müdürü"),
        ("254.01", "004", "Binek oto — Bölge sorumlusu (İzmir)"),
        ("254.01", "005", "Binek oto — Bölge sorumlusu (Ankara)"),
        ("254.01", "006", "Binek oto — Havuz aracı"),
        ("254.01", "901", "Kış lastik + jant setleri (tamamlayıcı)"),
        ("254.01", "902", "Araç takip ve donanım seti (tamamlayıcı)"),
        ("254.02", "001", "Ticari araç — dağıtım kamyoneti"),
        ("254.02", "002", "Ticari araç — panelvan"),
        ("254.02", "901", "Soğutma ünitesi (tamamlayıcı)"),
        ("255.01", "001", "Ofis mobilyası — çalışma istasyonları"),
        ("255.01", "002", "Toplantı odası donanımı"),
        ("255.02", "001", "Sunucu ve ağ altyapısı"),
        ("255.02", "002", "Dizüstü bilgisayarlar (28 adet)"),
        ("255.02", "901", "Kesintisiz güç kaynağı (tamamlayıcı)"),
        ("257", "01", "Bina birikmiş amort."), ("257", "02", "Makine birikmiş amort."),
        ("257", "03", "Taşıt birikmiş amort."), ("257", "04", "Demirbaş birikmiş amort."),
        ("300", "01", "İşletme kredisi"), ("300", "02", "Rotatif kredi"),
        ("400", "01", "Yatırım kredisi"), ("400", "02", "Uzun vadeli işletme kredisi"),
        ("320", "04", "Hizmet tedarikçisi"),
    ];
    {
        let mut guard = state.lock().unwrap();
        let st = &mut *guard;
        for (ana, alt, ad) in muavinler {
            if let Some(a) = st.hesaplar.get(*ana).cloned() {
                let m = muavin_olustur(&a, alt, ad);
                if !st.hesaplar.contains_key(&m.kod) {
                    if let Some(h) = st.hesaplar.get_mut(*ana) { h.yaprak = false; }
                    for h in st.hesap_sirali.iter_mut() { if h.kod == *ana { h.yaprak = false; } }
                    st.hesaplar.insert(m.kod.clone(), m.clone());
                    st.hesap_sirali.push(m);
                }
            }
        }
        st.hesap_sirali.sort_by(|a, b| a.kod.cmp(&b.kod));
    }

    let mut kes = |st: &mut AppState, mut f: Fis| -> Result<(), String> {
        let ctx = Baglam { hesaplar: &st.hesaplar, donem: &st.donem };
        kesinlestir(&mut f, &ctx, &mut st.sayac).map_err(|e| e.to_string())?;
        st.fisler.push(f);
        Ok(())
    };
    let b = |k: &str, t: i64| YevmiyeSatiri::borc(k.to_string(), Kurus::yeni(t));
    let a = |k: &str, t: i64| YevmiyeSatiri::alacak(k.to_string(), Kurus::yeni(t));
    // Açıklamalı satır: "kayıt en ince ayrıntısına kadar açıklanmalı" — satır bazlı detay
    // (hangi varlık, hangi marka/model, hangi seri/plaka; hangi cari; hangi belge).
    let bd = |k: &str, t: i64, d: &str| { let mut s = YevmiyeSatiri::borc(k.to_string(), Kurus::yeni(t)); s.aciklama = d.to_string(); s };
    let ad_ = |k: &str, t: i64, d: &str| { let mut s = YevmiyeSatiri::alacak(k.to_string(), Kurus::yeni(t)); s.aciklama = d.to_string(); s };

    let mut guard = state.lock().unwrap();
    let st = &mut *guard;
    let mut fatura_sira = 0u64;

    // AÇILIŞ FİŞİ (01.01) — tam bilanço: duran varlıklar, krediler, senetler, karşılıklar, özkaynak.
    // Amaç: mizanın TÜM sınıfları dolsun ki UFRS çalışmaları (amortisman, ECL, reeskont, GUD,
    // borçlanma maliyeti, reclass) gerçek girdi hesapları görsün.
    {
        let t = Tarih::yeni(2026, 1, 1);
        let mut f = Fis::taslak(FisTipi::Acilis, t, vec![
            b("100", 5_000_000),          // Kasa 50.000
            b("102.01", 200_000_000),     // Banka 2.000.000
            b("120.01", 25_000_000), b("120.02", 14_000_000), b("120.03", 6_000_000), // Alıcılar 450.000 (3 kırılım)
            b("121.01", 9_000_000), b("121.02", 6_000_000),   // Alacak senetleri 150.000
            b("128.01", 5_000_000), b("128.02", 3_000_000),   // Şüpheli alacaklar 80.000
            b("153.01", 30_000_000), b("153.02", 30_000_000), b("153.03", 30_000_000), // Stok 900.000
            b("159", 12_000_000),         // Verilen sipariş avansları 120.000
            b("180", 6_000_000),          // Gelecek aylara ait giderler 60.000
            b("250", 150_000_000),        // Arazi 1.500.000
            bd("252.01.001", 150000000, "Depo binası — ana yapı — Betonarme, 4.200 m² · Tapu: Ada 1204 Parsel 7 · edinim 15.03.2019"),
            bd("252.01.901", 20000000, "Depo raf sistemi (tamamlayıcı) — Dexion selektif raf, 1.800 palet göz · DX-2019-4471 · edinim 20.04.2019"),
            bd("252.01.902", 10000000, "Yangın söndürme sistemi (tamamlayıcı) — Sprinkler + hidrant hattı · YS-2019-88 · edinim 20.04.2019"),
            bd("252.02.001", 110000000, "İdari bina — Betonarme, 3 kat, 900 m² · Tapu: Ada 1204 Parsel 8 · edinim 15.03.2019"),
            bd("252.02.901", 10000000, "İklimlendirme (VRF) sistemi (tamamlayıcı) — Daikin VRV IV, 12 iç ünite · DK-VRV-2019-115 · edinim 01.06.2019"),
            bd("253.01.001", 18000000, "CNC torna tezgahı — Mazak QT-250MY · MZ-QT250-19A045 · edinim 10.05.2019"),
            bd("253.01.002", 15000000, "CNC dik işleme merkezi — Haas VF-4SS · HS-VF4-19B112 · edinim 10.05.2019"),
            bd("253.01.003", 9000000, "Hidrolik pres — Durma HP-160 · DR-HP160-2019 · edinim 10.05.2019"),
            bd("253.01.004", 12000000, "Robot kaynak hücresi — Fanuc ARC Mate 100iD · FN-AM100-20C77 · edinim 12.02.2020"),
            bd("253.01.005", 6000000, "Konveyör bant hattı — Interroll, 42 m modüler · IR-CV-2019-330 · edinim 10.05.2019"),
            bd("253.01.006", 4000000, "Vidalı hava kompresörü — Atlas Copco GA-45 · AC-GA45-19D201 · edinim 10.05.2019"),
            bd("253.01.007", 3000000, "Kalite kontrol ölçüm istasyonu — Zeiss CONTURA CMM · ZS-CT-20E018 · edinim 12.02.2020"),
            bd("253.01.008", 1500000, "Otomasyon panosu ve PLC — Siemens S7-1500 · SM-S71500-19F09 · edinim 10.05.2019"),
            bd("253.01.901", 1000000, "Kalıp ve aparat seti (tamamlayıcı) — Özel imalat, 24 parça · KLP-2019-24 · edinim 10.05.2019"),
            bd("253.01.902", 500000, "Kesici takım seti (tamamlayıcı) — Sandvik Coromant · SV-CT-2019 · edinim 10.05.2019"),
            bd("253.02.001", 20000000, "Otomatik dolum makinesi — Krones Volumetric 12-head · KR-VF12-20G455 · edinim 05.08.2020"),
            bd("253.02.002", 9000000, "Etiketleme makinesi — Herma 400 · HM-400-20H88 · edinim 05.08.2020"),
            bd("253.02.003", 6000000, "Shrink paketleme makinesi — Smipack BP-802 · SP-BP802-20J12 · edinim 05.08.2020"),
            bd("253.03.001", 9000000, "Forklift (dizel, 3 ton) — Toyota 8FD30 · TY-8FD30-19K334 · edinim 18.06.2019"),
            bd("253.03.002", 3000000, "Transpalet (akülü) — Jungheinrich EJE-120 · JH-EJE120-19L77 · edinim 18.06.2019"),
            bd("253.03.901", 3000000, "Forklift ataşmanları (tamamlayıcı) — Kaydırma tablası + varil tutucu · ATS-2019-2 · edinim 18.06.2019"),
            bd("254.01.001", 12000000, "Binek oto — Genel Müdür — Audi A6 2.0 TDI · 34 ABC 123 · edinim 10.01.2022"),
            bd("254.01.002", 9000000, "Binek oto — Mali İşler Müdürü — VW Passat 1.5 TSI · 34 DEF 456 · edinim 10.01.2022"),
            bd("254.01.003", 8500000, "Binek oto — Satış Müdürü — Skoda Superb 1.5 TSI · 34 GHI 789 · edinim 15.03.2022"),
            bd("254.01.004", 6500000, "Binek oto — Bölge sorumlusu (İzmir) — Renault Megane 1.3 TCe · 35 JKL 012 · edinim 20.06.2022"),
            bd("254.01.005", 7000000, "Binek oto — Bölge sorumlusu (Ankara) — Toyota Corolla 1.8 Hybrid · 06 MNO 345 · edinim 20.06.2022"),
            bd("254.01.006", 5000000, "Binek oto — Havuz aracı — Fiat Egea 1.4 · 34 PRS 678 · edinim 05.09.2023"),
            bd("254.01.901", 1500000, "Kış lastik + jant setleri (tamamlayıcı) — 6 araç için takım · LST-2022-6 · edinim 01.11.2022"),
            bd("254.01.902", 500000, "Araç takip ve donanım seti (tamamlayıcı) — Arvento GPS, 6 araç · AR-GPS-2022 · edinim 01.11.2022"),
            bd("254.02.001", 14000000, "Ticari araç — dağıtım kamyoneti — Ford Transit 350L · 34 TRN 901 · edinim 12.04.2021"),
            bd("254.02.002", 11000000, "Ticari araç — panelvan — Mercedes Vito 111 CDI · 34 VTO 234 · edinim 12.04.2021"),
            bd("254.02.901", 5000000, "Soğutma ünitesi (tamamlayıcı) — Thermo King V-300 · TK-V300-21 · edinim 12.04.2021"),
            bd("255.01.001", 9000000, "Ofis mobilyası — çalışma istasyonları — Bürotime, 24 istasyon · BT-2019-24 · edinim 01.06.2019"),
            bd("255.01.002", 6000000, "Toplantı odası donanımı — Masa + 12 koltuk + projeksiyon · TO-2019-1 · edinim 01.06.2019"),
            bd("255.02.001", 12000000, "Sunucu ve ağ altyapısı — Dell PowerEdge R740 ×2 + switch · DL-R740-19N01/02 · edinim 01.06.2019"),
            bd("255.02.901", 3000000, "Kesintisiz güç kaynağı (tamamlayıcı) — APC Smart-UPS 10kVA · APC-10K-19 · edinim 01.06.2019"),
            b("260", 20_000_000),         // Haklar (yazılım) 200.000
            a("129.01", 5_000_000), a("129.02", 3_000_000),     // Şüpheli alacak karşılığı (-) 80.000
            a("257.01", 45_000_000), a("257.02", 40_000_000), a("257.03", 18_000_000), a("257.04", 7_000_000), // Birikmiş amort. 1.100.000
            a("268", 5_000_000),          // Birikmiş amortisman MODV (-)
            a("300.01", 40_000_000), a("300.02", 20_000_000),   // Banka kredisi (kısa) 600.000
            a("320.01", 35_000_000),      // Satıcı açılış borcu
            a("321", 10_000_000),         // Borç senetleri 100.000
            a("331", 25_000_000),         // Ortaklara borçlar 250.000
            a("340", 9_000_000),          // Alınan sipariş avansları 90.000
            a("400.01", 100_000_000), a("400.02", 40_000_000), // Banka kredisi (uzun) 1.400.000
            a("500", 300_000_000),        // Sermaye 3.000.000
            a("540", 12_000_000),         // Yasal yedekler 120.000
            a("570", 367_000_000),        // Geçmiş yıllar kârları (denge — envanterle hizalı)
        ]);
        f.aciklama = "Açılış fişi — 01.01.2026 açılış bilançosu".into();
        f.dayanak = Some(Dayanak { belge_tipi: BelgeTipi::Diger, belge_no: "ACILIS-2026".into(), belge_tarihi: t });
        kes(st, f).map_err(|e| hata(e))?;
    }

    for ay in 1u8..=12 {
        for gun in 1u8..=ay_gun(ay) {
            let t = Tarih::yeni(2026, ay, gun);
            // ~48-56 satış/gün — her satış ayrı e-arşiv faturası (POS: 108.01)
            let satis_n = 48 + rng.u(9);
            let mut gun_smm = 0i64;
            for _ in 0..satis_n {
                fatura_sira += 1;
                let y20 = rng.u(10) < 7; // %70 genel oran
                let matrah = (15_000 + rng.u(250_000)) as i64; // 150₺–2.650₺
                let kdv = if y20 { matrah / 5 } else { matrah / 10 };
                let kanal = if rng.u(10) < 6 { "600.01" } else { "600.02" };
                let stok = match rng.u(3) { 0 => "153.01", 1 => "153.02", _ => "153.03" };
                let mut f = Fis::taslak(FisTipi::Mahsup, t, vec![
                    b("108.01", matrah + kdv),
                    a(kanal, matrah),
                    a(if y20 { "391.20" } else { "391.10" }, kdv),
                ]);
                f.aciklama = format!("E-arşiv satış #{}", fatura_sira);
                f.dayanak = Some(Dayanak { belge_tipi: BelgeTipi::EFatura, belge_no: format!("EA-2026-{:06}", fatura_sira), belge_tarihi: t });
                kes(st, f).map_err(|e| hata(e))?;
                gun_smm += matrah * 60 / 100; // ~%40 brüt marj
                let _ = stok;
            }
            // Günlük SMM (toplu): 621.01 B / 153.xx A
            let s1 = gun_smm / 3; let s2 = gun_smm / 3; let s3 = gun_smm - s1 - s2;
            let mut f = Fis::taslak(FisTipi::Mahsup, t, vec![b("621.01", gun_smm), a("153.01", s1), a("153.02", s2), a("153.03", s3)]);
            f.aciklama = "Günlük satılan mal maliyeti".into();
            kes(st, f).map_err(|e| hata(e))?;

            // Haftalık (pazartesi benzeri: gun % 7 == 1)
            if gun % 7 == 1 {
                // Mal alışı (%20 KDV): 153.xx + 191.20 / 320.01-02 — SMM ile orantılı (stok eksiye düşmesin)
                let alis = (28_000_000 + rng.u(8_000_000)) as i64;
                let kdv = alis / 5;
                let ted = if rng.u(2) == 0 { "320.01" } else { "320.02" };
                fatura_sira += 1;
                let a1 = alis / 3; let a2 = alis / 3; let a3 = alis - a1 - a2;
                let mut f = Fis::taslak(FisTipi::Mahsup, t, vec![b("153.01", a1), b("153.02", a2), b("153.03", a3), b("191.20", kdv), a(ted, alis + kdv)]);
                f.aciklama = "Haftalık mal alışı".into();
                f.dayanak = Some(Dayanak { belge_tipi: BelgeTipi::EFatura, belge_no: format!("AL-2026-{:06}", fatura_sira), belge_tarihi: t });
                kes(st, f).map_err(|e| hata(e))?;
                // POS tahsilatı: birikmiş 108.01 → 102.01, %2 komisyon (760.02 + %20 KDV'si)
                let d = defter_kur(&st.fisler);
                let (pb, pa) = d.hesap_bakiyesi("108.01");
                let pos = pb.deger() - pa.deger();
                if pos > 0 {
                    let kom = pos * 2 / 100;
                    let kom_kdv = kom / 5;
                    let mut f = Fis::taslak(FisTipi::Mahsup, t, vec![b("102.01", pos - kom - kom_kdv), b("760.02", kom), b("191.20", kom_kdv), a("108.01", pos)]);
                    f.aciklama = "Haftalık POS tahsilatı (komisyon kesintili)".into();
                    f.dayanak = Some(Dayanak { belge_tipi: BelgeTipi::Dekont, belge_no: format!("POS-{:02}{:02}", ay, gun), belge_tarihi: t });
                    kes(st, f).map_err(|e| hata(e))?;
                }
                // Tedarikçi ödemesi (bakiyenin ~%80'i)
                let (tb, ta) = d.hesap_bakiyesi(ted);
                let borc = ta.deger() - tb.deger();
                if borc > 0 {
                    let ode = borc * 8 / 10;
                    let mut f = Fis::taslak(FisTipi::Tediye, t, vec![b(ted, ode), a("102.01", ode)]);
                    f.aciklama = "Tedarikçi ödemesi".into();
                    f.dayanak = Some(Dayanak { belge_tipi: BelgeTipi::Dekont, belge_no: format!("EFT-{:02}{:02}", ay, gun), belge_tarihi: t });
                    kes(st, f).map_err(|e| hata(e))?;
                }
            }
            // Ay sonu
            if gun == ay_gun(ay) {
                // Kira 40.000+KDV, %20 stopaj (GVK 94); tam bordro (SGK işveren payı dahil); kargo/reklam
                fatura_sira += 1;
                // Kira: gider 40.000 (770.01) + KDV 8.000; stopaj 8.000 → 360; net ödeme 40.000 (KDV dâhil − stopaj)
                let mut f = Fis::taslak(FisTipi::Mahsup, t, vec![
                    bd("770.01", 4_000_000, "İşyeri kirası — brüt 40.000 TL (kira sözleşmesi 2026/A-1, aylık)"),
                    bd("191.20", 800_000, "İndirilecek KDV %20 — kira faturası"),
                    ad_("102.01", 4_000_000, "Banka TL — net kira ödemesi (brüt − %20 stopaj)"),
                    ad_("360", 800_000, "Ödenecek GV stopajı %20 (GVK 94/5-a) — muhtasar beyanına")]);
                f.aciklama = "Aylık kira (GVK 94 %20 stopajlı)".into();
                f.dayanak = Some(Dayanak { belge_tipi: BelgeTipi::EFatura, belge_no: format!("KR-2026-{:02}", ay), belge_tarihi: t });
                kes(st, f).map_err(|e| hata(e))?;
                // Tam bordro: brüt 180.000 + SGK işveren payı 36.000 = personel gideri 216.000; 361 = işçi 18.000 + işveren 36.000
                let mut f = Fis::taslak(FisTipi::Mahsup, t, vec![
                    bd("770.02", 18_000_000, "Personel brüt ücret tahakkuku — 180.000 TL (bordro dönemi)"),
                    bd("770.02", 3_600_000, "SGK işveren payı — 36.000 TL (işverene ait yük)"),
                    ad_("335", 12_600_000, "Personele borçlar — net ödenecek ücret 126.000 TL"),
                    ad_("360", 3_600_000, "Ödenecek GV stopajı + damga vergisi — 36.000 TL"),
                    ad_("361", 5_400_000, "Ödenecek SGK kesintileri — işçi payı 18.000 + işveren payı 36.000 = 54.000 TL")]);
                f.aciklama = "Aylık ücret tahakkuku (SGK işveren payı dahil)".into();
                f.dayanak = Some(Dayanak { belge_tipi: BelgeTipi::Bordro, belge_no: format!("BRD-2026-{:02}", ay), belge_tarihi: t });
                kes(st, f).map_err(|e| hata(e))?;
                fatura_sira += 1;
                let mut f = Fis::taslak(FisTipi::Mahsup, t, vec![b("760.01", 6_000_000), b("191.20", 1_200_000), a("320.03", 7_200_000)]);
                f.aciklama = "Aylık kargo faturası".into();
                f.dayanak = Some(Dayanak { belge_tipi: BelgeTipi::EFatura, belge_no: format!("KG-2026-{:02}", ay), belge_tarihi: t });
                kes(st, f).map_err(|e| hata(e))?;
                fatura_sira += 1;
                let mut f = Fis::taslak(FisTipi::Mahsup, t, vec![b("760.03", 3_000_000), b("191.20", 600_000), a("102.01", 3_600_000)]);
                f.aciklama = "Aylık reklam gideri".into();
                f.dayanak = Some(Dayanak { belge_tipi: BelgeTipi::EFatura, belge_no: format!("RK-2026-{:02}", ay), belge_tarihi: t });
                kes(st, f).map_err(|e| hata(e))?;
                // Aylık kredi faizi (780 → ay sonu 781/660 yansıtması) — WS-23 borçlanma maliyeti girdisi
                let mut f = Fis::taslak(FisTipi::Tediye, t, vec![
                    bd("780.01", 2_500_000, "Kredi faiz gideri — işletme kredisi (300.01), aylık tahakkuk"),
                    ad_("102.01", 2_500_000, "Banka TL — faiz ödemesi (dekont)")]);
                f.aciklama = "Aylık kredi faizi ödemesi".into();
                f.dayanak = Some(Dayanak { belge_tipi: BelgeTipi::Dekont, belge_no: format!("FAIZ-2026-{:02}", ay), belge_tarihi: t });
                kes(st, f).map_err(|e| hata(e))?;
                // Çeyrek sonu kredi anapara taksiti (300 kısa vadeli)
                if ay % 3 == 0 {
                    let mut f = Fis::taslak(FisTipi::Tediye, t, vec![
                        bd("300.01", 5_000_000, "İşletme kredisi anapara taksiti — çeyrek ödeme (itfa planı)"),
                        ad_("102.01", 5_000_000, "Banka TL — anapara ödemesi (dekont)")]);
                    f.aciklama = "Kredi anapara taksiti (çeyrek)".into();
                    f.dayanak = Some(Dayanak { belge_tipi: BelgeTipi::Dekont, belge_no: format!("ANP-2026-{:02}", ay), belge_tarihi: t });
                    kes(st, f).map_err(|e| hata(e))?;
                }
                // Mart: yıl içi demirbaş alımı (kıst amortisman konusu — WS-AMORT girdisi)
                if ay == 3 {
                    fatura_sira += 1;
                    let mut f = Fis::taslak(FisTipi::Mahsup, t, vec![
                        bd("255.02.002", 15_000_000, "Dizüstü bilgisayar 28 adet — Lenovo ThinkPad T14 · seri listesi faturaya ekli · edinim 10.03.2026"),
                        bd("191.20", 3_000_000, "İndirilecek KDV %20 — demirbaş alımı"),
                        ad_("102.01", 18_000_000, "Banka TL — Lenovo yetkili satıcı ödemesi")]);
                    f.aciklama = "Demirbaş alımı — 28 adet dizüstü (yıl içi, kıst amortisman)".into();
                    f.dayanak = Some(Dayanak { belge_tipi: BelgeTipi::EFatura, belge_no: format!("DB-2026-{:06}", fatura_sira), belge_tarihi: t });
                    kes(st, f).map_err(|e| hata(e))?;
                }
                // Aralık: yıllık amortisman (VUK) + dönem sonu kur değerlemeleri
                if ay == 12 {
                    let mut f = Fis::taslak(FisTipi::Mahsup, t, vec![
                        bd("770.03", 55_000_000, "Yıllık amortisman gideri — VUK oranları (varlık bazlı döküm ektedir)"),
                        ad_("257.01", 12_000_000, "Binalar birikmiş amortismanı — depo + idari bina (%2 VUK)"),
                        ad_("257.02", 24_000_000, "Makineler birikmiş amortismanı — üretim hattı 10 bileşen + paketleme (%10 VUK)"),
                        ad_("257.03", 10_000_000, "Taşıtlar birikmiş amortismanı — 6 binek + 2 ticari araç (%20 VUK)"),
                        ad_("257.04", 6_000_000, "Demirbaşlar birikmiş amortismanı — mobilya + BT donanımı"),
                        ad_("268", 3_000_000, "Haklar (yazılım) itfa payı")]);
                    f.aciklama = "Yıllık amortisman (VUK oranları)".into();
                    f.dayanak = Some(Dayanak { belge_tipi: BelgeTipi::Diger, belge_no: "AMR-2026".into(), belge_tarihi: t });
                    kes(st, f).map_err(|e| hata(e))?;
                    let mut f = Fis::taslak(FisTipi::Mahsup, t, vec![
                        bd("120.03", 2_500_000, "İhracat müşterisi dövizli alacak — dönem sonu kur değerlemesi (TCMB alış)"),
                        ad_("646", 2_500_000, "Kambiyo kârı — dövizli alacağın TL karşılığındaki artış")]);
                    f.aciklama = "Dövizli alacak kur değerlemesi (TCMB alış)".into();
                    kes(st, f).map_err(|e| hata(e))?;
                    let mut f = Fis::taslak(FisTipi::Mahsup, t, vec![
                        bd("656", 4_500_000, "Kambiyo zararı — dövizli satıcı borcunun TL karşılığındaki artış"),
                        ad_("320.02", 4_500_000, "Toptancı B dövizli borç — dönem sonu kur değerlemesi (TCMB alış)")]);
                    f.aciklama = "Dövizli borç kur değerlemesi (TCMB alış)".into();
                    kes(st, f).map_err(|e| hata(e))?;
                }
                // AY SONU YANSITMA (7 → 6): 760→761→631, 770→771→632.
                // 7'ler yansıtılmadan bilanço/kâr doğru çıkmaz (vergiye giden yolun 0. adımı).
                let d = defter_kur(&st.fisler);
                for (gider_on, yansitma, hedef) in [("760", "761", "631"), ("770", "771", "632"), ("780", "781", "660")] {
                    let altlar: Vec<(String, i64)> = d
                        .mizan()
                        .into_iter()
                        .filter(|m| m.hesap_id == gider_on || m.hesap_id.starts_with(&format!("{}.", gider_on)))
                        .map(|m| (m.hesap_id, m.borc_bakiye.deger() - m.alacak_bakiye.deger()))
                        .filter(|(_, n)| *n > 0)
                        .collect();
                    let toplam: i64 = altlar.iter().map(|(_, n)| n).sum();
                    if toplam > 0 {
                        // Yansıtma: 631/632 B / 7x1 A
                        let mut f = Fis::taslak(FisTipi::Mahsup, t, vec![b(hedef, toplam), a(yansitma, toplam)]);
                        f.aciklama = format!("Ay sonu yansıtma {} → {}", gider_on, hedef);
                        kes(st, f).map_err(|e| hata(e))?;
                        // Kapama: 7x1 B / 7xx.xx A (her alt hesap kendi bakiyesiyle)
                        let mut s = vec![b(yansitma, toplam)];
                        for (kod, n) in &altlar { s.push(a(kod, *n)); }
                        let mut f = Fis::taslak(FisTipi::Mahsup, t, s);
                        f.aciklama = format!("Yansıtma kapama {} ↔ {}", yansitma, gider_on);
                        kes(st, f).map_err(|e| hata(e))?;
                    }
                }
                // Ay sonu KDV mahsubu (alt hesapları tarar) — vergiye giden yol
                let d = defter_kur(&st.fisler);
                if let Some(mut f) = kdv_mahsup(&d, t) {
                    f.dayanak = Some(Dayanak { belge_tipi: BelgeTipi::Beyanname, belge_no: format!("KDV-2026-{:02}", ay), belge_tarihi: t });
                    kes(st, f).map_err(|e| hata(e))?;
                }
            }
        }
    }
    let toplam = st.fisler.len();
    Ok(Json(serde_json::json!({ "fis_sayisi": toplam, "aciklama": "E-ticaret 2026 örnek defteri yüklendi (kronolojik)" })))
}

// ---- Vergi yolu (G): KDV beyanname taslağı + geçici vergi hesap kağıdı ----

const VERGI_PARAM_JSON: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/vergi-parametreleri.json"));
const UFRS_PARAM_JSON: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/ufrs-parametreleri.json"));

/// KV genel oranı — data/vergi-parametreleri.json'dan (KVK-5520 "Genel oran": "%25").
/// Kod hardcode etmez; yıl tebliğiyle dosya güncellenir. Firma/yıl parametresi (E.5) gelince oradan.
fn kv_orani() -> i64 {
    serde_json::from_str::<serde_json::Value>(VERGI_PARAM_JSON)
        .ok()
        .and_then(|v| {
            v["kanunlar"].as_array()?.iter().find(|k| k["kod"] == "KVK-5520")?["alanlar"]
                .as_array()?
                .iter()
                .find(|a| a["ad"].as_str().map(|x| x.starts_with("Genel oran")).unwrap_or(false))?["deger"]
                .as_str()?
                .trim_start_matches('%')
                .parse()
                .ok()
        })
        .unwrap_or(25)
}

async fn vergi_parametreler() -> Json<serde_json::Value> {
    Json(serde_json::from_str(VERGI_PARAM_JSON).unwrap_or_else(|_| serde_json::json!({})))
}

fn ay_defteri(fisler: &[Fis], son_ay: u8) -> Defter {
    let mut dd = Defter::yeni();
    for f in fisler.iter().filter(|x| x.tarih.ay <= son_ay) {
        dd.ekle(f.clone());
    }
    dd
}

#[derive(Serialize)]
struct KdvMatrahSatir { kod: String, ad: String, oran: i64, matrah: i64, hesaplanan: i64, kaynak: String }
#[derive(Serialize, Clone)]
struct KdvManuel { ad: String, yon: String, matrah: i64, kdv: i64, kaynak: String }
#[derive(Serialize, Default)]
struct KdvIstisna { matrah: i64, belge_sayisi: usize, kaynak: String }
#[derive(Serialize, Default)]
struct KdvTevkifat { matrah: i64, toplam_kdv: i64, tevkif_edilen: i64, beyan_edilen: i64, belge_sayisi: usize, kaynak: String }

#[derive(Serialize)]
struct KdvBeyanname {
    ay: u8,
    donem: String,
    matrahlar: Vec<KdvMatrahSatir>,
    /// SMMM manuel müdahaleleri (doktrin: deftere işlemez, taslakta sistem verisiyle yan yana, izli)
    manuel: Vec<KdvManuel>,
    /// KDV1 formu "Bu döneme ait indirilecek KDV tutarının oranlara göre dağılımı" bölümü.
    /// Hesaplanan taraftaki kırılımın aynısı, 191 muavinlerinden.
    indirim_dagilimi: Vec<KdvMatrahSatir>,
    /// KDVK 11/12 istisna kapsamındaki teslimler — matrah BEYAN EDİLİR, KDV doğmaz.
    /// KDV1 formunda ayrı bölümdür; hesaplanan KDV'ye eklenmez.
    istisna: KdvIstisna,
    /// KDVK 9 tevkifatlı işlemler — KDV'nin alıcıda kalan payı burada izlenir.
    tevkifat: KdvTevkifat,
    hesaplanan_toplam: i64,
    onceki_devreden: i64,
    bu_donem_indirilecek: i64,
    indirimler_toplam: i64,
    odenecek: i64,
    devreden: i64,
    /// Doktrin: her beyan kalemi muhasebe kaydını referans gösterir
    kaynak_notlari: Vec<String>,
}

fn kdv_manuel_oku(v: Option<&serde_json::Value>) -> Vec<KdvManuel> {
    v.and_then(|d| d["satirlar"].as_array().cloned())
        .unwrap_or_default()
        .iter()
        .map(|r| KdvManuel {
            ad: r["ad"].as_str().unwrap_or("").to_string(),
            yon: r["yon"].as_str().unwrap_or("hesaplanan").to_string(),
            matrah: r["matrah"].as_i64().unwrap_or(0),
            kdv: r["kdv"].as_i64().unwrap_or(0),
            kaynak: r["kaynak"].as_str().unwrap_or("").to_string(),
        })
        .collect()
}

/// KDV1 taslağı: ay içi 391 muavin hareketlerinden oran kırılımlı matrah; 191 indirim; 190 devir zinciri.
async fn kdv_beyanname(State(state): State<Shared>, Query(q): Query<HashMap<String, String>>) -> Json<KdvBeyanname> {
    let st = state.lock().unwrap();
    let ay: u8 = q.get("ay").and_then(|s| s.parse().ok()).unwrap_or(12).clamp(1, 12);

    // Ay içi NET hareketler: 391 = alacak − borç (satış İADESİ/düzeltme fişleri hesaplananı DÜŞÜRÜR),
    // 191 = borç − alacak (alış iadesi indirilecekten düşer). Sistemin aylık mahsup fişi hariç tutulur —
    // o fiş 391/191'i topluca kapatır, beyan matrahının kaynağı değildir.
    let mut hes_muavin: std::collections::BTreeMap<String, i64> = std::collections::BTreeMap::new();
    let mut indirilecek = 0i64;
    let mut ind_muavin: std::collections::BTreeMap<String, i64> = std::collections::BTreeMap::new();
    for f in st.fisler.iter().filter(|x| {
        x.tarih.ay == ay && !x.aciklama.starts_with("Aylık KDV tahakkuku")
    }) {
        for sat in &f.satirlar {
            if onek_altinda(&sat.hesap_id, "391") {
                *hes_muavin.entry(sat.hesap_id.clone()).or_default() += sat.alacak.deger() - sat.borc.deger();
            }
            if onek_altinda(&sat.hesap_id, "191") {
                let net = sat.borc.deger() - sat.alacak.deger();
                indirilecek += net;
                *ind_muavin.entry(sat.hesap_id.clone()).or_default() += net;
            }
        }
    }
    let matrahlar: Vec<KdvMatrahSatir> = hes_muavin
        .into_iter()
        .filter(|(_, v)| *v > 0)
        .map(|(kod, hesaplanan)| {
            // Muavin kodu ARDIŞIK olduğu için oran koddan okunamaz — parametre katalogundan çözülür.
            let oran: i64 = hesaplama::muavin_orani(&kod).unwrap_or(20);
            let ad = st.hesaplar.get(&kod).map(|h| h.ad.clone()).unwrap_or_default();
            KdvMatrahSatir { matrah: hesaplanan * 100 / oran, kaynak: format!("{kod} muavini — {ay:02}/2026 alacak hareketi"), kod, ad, oran, hesaplanan }
        })
        .collect();
    // İSTİSNA ve TEVKİFAT — defter satırından okunamaz (391'de iz bırakmazlar): istisna
    // teslimde KDV hiç doğmaz, tevkifatta 391'e yalnız kalan pay yazılır. Bu yüzden belge
    // düzeyindeki fatura verisinden toplanır. KDV1 formunun ayrı bölümleridir.
    let gor = gorunum(&st);
    let mut istisna = KdvIstisna::default();
    let mut tevk = KdvTevkifat::default();
    for f in gor.faturalar.iter().filter(|f| f.gecerli && f.yon == "SATIS") {
        let f_ay: u8 = f.tarih[3..5].parse().unwrap_or(0);
        if f_ay != ay { continue; }
        if f.kdv_oran == 0 {
            istisna.matrah += f.matrah - f.iade_matrah - f.iskonto_matrah;
            istisna.belge_sayisi += 1;
        }
        if f.tevkifat > 0 {
            tevk.matrah += f.matrah;
            tevk.toplam_kdv += f.kdv;
            tevk.tevkif_edilen += f.tevkifat;
            tevk.beyan_edilen += f.kdv - f.tevkifat;
            tevk.belge_sayisi += 1;
        }
    }
    istisna.kaynak = "İhracat/istisna faturaları (601) — KDV doğmaz, matrah beyan edilir (KDVK 11/12)".into();
    tevk.kaynak = "Tevkifatlı satış faturaları — tevkif edilen kısım alıcının 2 No.lu KDV beyanına girer (KDVK 9)".into();

    // İndirilecek KDV'nin ORANLARA GÖRE DAĞILIMI — KDV1 formunun ayrı bölümü.
    // "Alınan mal ve hizmete ait bedel" = KDV / oran × 100 (muavin kodu oranı taşır).
    let indirim_dagilimi: Vec<KdvMatrahSatir> = ind_muavin
        .into_iter()
        .filter(|(_, v)| *v > 0)
        .map(|(kod, kdv)| {
            let oran: i64 = hesaplama::muavin_orani(&kod).unwrap_or(20);
            let ad = st.hesaplar.get(&kod).map(|h| h.ad.clone()).unwrap_or_default();
            KdvMatrahSatir { matrah: kdv * 100 / oran, hesaplanan: kdv, oran, ad,
                kaynak: format!("{kod} muavini — {ay:02}/2026 borç hareketi"), kod }
        })
        .collect();

    let manuel = kdv_manuel_oku(st.vergi_duzenlemeler.get(&format!("KDV-{ay}")));
    let man_hes: i64 = manuel.iter().filter(|m| m.yon == "hesaplanan").map(|m| m.kdv).sum();
    let man_ind: i64 = manuel.iter().filter(|m| m.yon == "indirim").map(|m| m.kdv).sum();
    let hesaplanan_toplam: i64 = matrahlar.iter().map(|m| m.hesaplanan).sum::<i64>() + man_hes;
    let onceki_devreden = if ay > 1 {
        let (b, a) = ay_defteri(&st.fisler, ay - 1).bakiye_rollup("190");
        (b.deger() - a.deger()).max(0)
    } else { 0 };
    let indirimler_toplam = indirilecek + onceki_devreden + man_ind;
    let fark = hesaplanan_toplam - indirimler_toplam;
    Json(KdvBeyanname {
        ay,
        donem: format!("01.{ay:02}.2026 – {}.{ay:02}.2026", [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31][(ay - 1) as usize]),
        matrahlar,
        manuel,
        indirim_dagilimi,
        istisna,
        tevkifat: tevk,
        hesaplanan_toplam,
        onceki_devreden,
        bu_donem_indirilecek: indirilecek,
        indirimler_toplam,
        odenecek: fark.max(0),
        devreden: (-fark).max(0),
        kaynak_notlari: vec![
            "Hesaplanan KDV: 391* muavinlerinin ay içi NET hareketi (alacak − borç; iade/düzeltme düşülür, sistem mahsup fişi hariç)".into(),
            "İndirilecek KDV: 191* muavinlerinin ay içi NET hareketi (borç − alacak; alış iadesi düşülür)".into(),
            format!("Devreden: önceki ay ({}) defterinde 190 borç bakiyesi", if ay > 1 { format!("{:02}/2026", ay - 1) } else { "—".into() }),
            "Manuel satırlar deftere işlemez; beyan taslağında izli tutulur (beyanname doktrini)".into(),
            "İstisna teslimler ayrı bölümdür — matrahı beyan edilir, hesaplanan KDV'ye EKLENMEZ".into(),
            "Tevkifatta 391'e yalnız beyan edilen pay yazılır; tevkif edilen kısmı alıcı 2 No.lu KDV ile beyan eder".into(),
        ],
    })
}

#[derive(Serialize, Clone)]
struct GvManuel { ad: String, tutar: i64 }
#[derive(Serialize)]
struct GvDto {
    kaynak_notlari: Vec<String>,
    ceyrek: u8,
    donem: String,
    ticari_kar: i64,
    kkeg_otomatik: i64,
    kkeg_manuel: Vec<GvManuel>,
    istisna_manuel: Vec<GvManuel>,
    matrah: i64,
    oran: i64,
    hesaplanan: i64,
    onceki_mahsup: i64,
    odenecek: i64,
}

fn manuel_oku(v: Option<&serde_json::Value>, alan: &str) -> Vec<GvManuel> {
    v.and_then(|d| d[alan].as_array().cloned())
        .unwrap_or_default()
        .iter()
        .map(|r| GvManuel { ad: r["ad"].as_str().unwrap_or("").to_string(), tutar: r["tutar"].as_i64().unwrap_or(0) })
        .collect()
}

/// Kümülatif geçici vergi (KVK: dönem başından çeyrek sonuna; önceki dönemde hesaplanan mahsup edilir).
fn gv_hesapla(st: &AppState, ceyrek: u8) -> GvDto {
    let dd = ay_defteri(&st.fisler, ceyrek);
    let g = gelir_tablosu(&dd);
    let ticari_kar = g.donem_kari;
    let (kb, ka) = dd.bakiye_rollup("689");
    let kkeg_otomatik = (kb.deger() - ka.deger()).max(0);
    let duzen = st.vergi_duzenlemeler.get(&format!("GV-{ceyrek}"));
    let kkeg_manuel = manuel_oku(duzen, "kkeg");
    let istisna_manuel = manuel_oku(duzen, "istisna");
    let km: i64 = kkeg_manuel.iter().map(|m| m.tutar).sum();
    let im: i64 = istisna_manuel.iter().map(|m| m.tutar).sum();
    let oran = kv_orani();
    let matrah = (ticari_kar + kkeg_otomatik + km - im).max(0);
    let hesaplanan = matrah * oran / 100;
    let onceki_mahsup = if ceyrek > 3 { gv_hesapla(st, ceyrek - 3).hesaplanan } else { 0 };
    GvDto {
        kaynak_notlari: vec![
            "Ticari kâr: gelir tablosu (dönem başından çeyrek sonuna kümülatif defter)".into(),
            "KKEG otomatik: 689* önek taraması (borç bakiyesi)".into(),
            format!("Oran: data/vergi-parametreleri.json KVK-5520 (%{})", oran),
            "Önceki mahsup: bir önceki çeyreğin hesaplanan geçici vergisi (KVK mük.120)".into(),
        ],
        ceyrek,
        donem: format!("01.01.2026 – {}", ["31.03.2026", "30.06.2026", "30.09.2026", "31.12.2026"][((ceyrek / 3) - 1) as usize]),
        ticari_kar,
        kkeg_otomatik,
        kkeg_manuel,
        istisna_manuel,
        matrah,
        oran,
        hesaplanan,
        onceki_mahsup,
        odenecek: (hesaplanan - onceki_mahsup).max(0),
    }
}

async fn gecici_vergi(State(state): State<Shared>, Query(q): Query<HashMap<String, String>>) -> Json<GvDto> {
    let st = state.lock().unwrap();
    let ceyrek: u8 = q.get("ceyrek").and_then(|s| s.parse().ok()).unwrap_or(12);
    let ceyrek = if [3u8, 6, 9, 12].contains(&ceyrek) { ceyrek } else { 12 };
    Json(gv_hesapla(&st, ceyrek))
}

async fn kdv_beyanname_duzenle(
    State(state): State<Shared>,
    Query(q): Query<HashMap<String, String>>,
    Json(g): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let mut st = state.lock().unwrap();
    let ay: u8 = q.get("ay").and_then(|s| s.parse().ok()).unwrap_or(12).clamp(1, 12);
    st.vergi_duzenlemeler.insert(format!("KDV-{ay}"), g);
    Json(serde_json::json!({ "ok": true }))
}

async fn gecici_vergi_duzenle(
    State(state): State<Shared>,
    Query(q): Query<HashMap<String, String>>,
    Json(g): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let mut st = state.lock().unwrap();
    let ceyrek: u8 = q.get("ceyrek").and_then(|s| s.parse().ok()).unwrap_or(12);
    st.vergi_duzenlemeler.insert(format!("GV-{ceyrek}"), g);
    Json(serde_json::json!({ "ok": true }))
}

// ---- Kullanıcı girişi + yetki (kayıt yok; admin kullanıcı açar) ----

#[derive(Deserialize)]
struct GirisGirdi { kullanici_adi: String, sifre: String }

async fn giris(State(state): State<Shared>, Json(g): Json<GirisGirdi>) -> Result<Json<serde_json::Value>, (StatusCode, Json<HataDto>)> {
    let mut st = state.lock().unwrap();
    let k = st.kullanicilar.iter().find(|k| k.kullanici_adi == g.kullanici_adi).cloned();
    let hata = || (StatusCode::UNAUTHORIZED, Json(HataDto { hata: "kullanıcı adı veya parola hatalı".into() }));
    let Some(k) = k else { return Err(hata()) };
    if sifre_ozet(&k.salt, &g.sifre) != k.sifre_ozet {
        return Err(hata());
    }
    let token = {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut h = DefaultHasher::new();
        (std::time::SystemTime::now(), &k.id, st.oturumlar.len()).hash(&mut h);
        format!("tok-{:016x}", h.finish())
    };
    st.oturumlar.insert(token.clone(), k.id.clone());
    Ok(Json(serde_json::json!({ "token": token, "kullanici": k })))
}

async fn oturum(State(state): State<Shared>, headers: HeaderMap) -> Result<Json<Kullanici>, StatusCode> {
    let st = state.lock().unwrap();
    oturumdaki(&st, &headers).cloned().map(Json).ok_or(StatusCode::UNAUTHORIZED)
}

async fn cikis(State(state): State<Shared>, headers: HeaderMap) -> Json<serde_json::Value> {
    let mut st = state.lock().unwrap();
    if let Some(tok) = headers.get("authorization").and_then(|v| v.to_str().ok()).and_then(|x| x.strip_prefix("Bearer ")) {
        st.oturumlar.remove(tok);
    }
    Json(serde_json::json!({ "ok": true }))
}

async fn kullanicilar(State(state): State<Shared>, headers: HeaderMap) -> Result<Json<Vec<Kullanici>>, StatusCode> {
    let st = state.lock().unwrap();
    let admin = oturumdaki(&st, &headers).map(|m| m.rol == "admin").unwrap_or(false);
    if !admin { return Err(StatusCode::FORBIDDEN); }
    Ok(Json(st.kullanicilar.clone()))
}

#[derive(Deserialize)]
struct YeniKullanici {
    kullanici_adi: String,
    ad: String,
    sifre: String,
    #[serde(default)]
    rol: String,
    #[serde(default)]
    mukellef_idleri: Vec<String>,
    #[serde(default)]
    departman: String,
    #[serde(default)]
    kademe: String,
}

async fn kullanici_ekle(State(state): State<Shared>, headers: HeaderMap, Json(g): Json<YeniKullanici>) -> Result<Json<serde_json::Value>, (StatusCode, Json<HataDto>)> {
    let mut st = state.lock().unwrap();
    let admin = oturumdaki(&st, &headers).map(|m| m.rol == "admin").unwrap_or(false);
    if !admin {
        return Err((StatusCode::FORBIDDEN, Json(HataDto { hata: "yalnız admin kullanıcı açabilir".into() })));
    }
    if g.kullanici_adi.trim().is_empty() || g.sifre.is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(HataDto { hata: "kullanıcı adı ve parola zorunlu".into() })));
    }
    if st.kullanicilar.iter().any(|k| k.kullanici_adi == g.kullanici_adi) {
        return Err((StatusCode::BAD_REQUEST, Json(HataDto { hata: "bu kullanıcı adı zaten var".into() })));
    }
    let salt = format!("s{}", st.kullanicilar.len());
    let id = format!("u{}", st.kullanicilar.len() + 1);
    st.kullanicilar.push(Kullanici {
        id: id.clone(), kullanici_adi: g.kullanici_adi, ad: g.ad,
        rol: if g.rol == "admin" { "admin".into() } else { "kullanici".into() },
        mukellef_idleri: g.mukellef_idleri,
        departman: if g.departman.is_empty() { "MUHASEBE".into() } else { g.departman },
        kademe: if g.kademe.is_empty() { "ELEMAN".into() } else { g.kademe },
        sifre_ozet: sifre_ozet(&salt, &g.sifre), salt,
    });
    Ok(Json(serde_json::json!({ "ok": true, "id": id })))
}

// ---- Çoklu mükellef + sektör (görev #5) ----

async fn sektorler() -> Json<serde_json::Value> {
    Json(serde_json::from_str(SEKTORLER_JSON).unwrap_or_else(|_| serde_json::json!({})))
}

/// Muhasebecinin kendi şablonu. `kullanici: true` ile hazır şablonlardan ayrılır (silinebilir olan bu).
#[derive(Serialize, Deserialize, Clone)]
struct KullaniciSablon {
    ad: String,
    tip: String,
    #[serde(default)]
    sektorler: Vec<String>,
    satirlar: Vec<KullaniciSablonSatir>,
}
#[derive(Serialize, Deserialize, Clone)]
struct KullaniciSablonSatir {
    kod: String,
    #[serde(default)]
    aciklama: String,
    /// "B" (borç) | "A" (alacak)
    taraf: String,
}

/// GET /api/sablonlar — hazır şablonlar + muhasebecinin eklediklerini birleştirir.
async fn sablonlar(State(state): State<Shared>) -> Json<serde_json::Value> {
    let mut v: serde_json::Value =
        serde_json::from_str(SABLONLAR_JSON).unwrap_or_else(|_| serde_json::json!({}));
    let st = state.lock().unwrap();
    if let Some(arr) = v.get_mut("sablonlar").and_then(|x| x.as_array_mut()) {
        for k in &st.kullanici_sablonlar {
            if let Ok(mut o) = serde_json::to_value(k) {
                o["kullanici"] = serde_json::json!(true);
                arr.push(o);
            }
        }
        // Kullanım sayacını her şablona iliştir (hazır + kullanıcı) → istemci sık kullanılanı öne alır.
        for o in arr.iter_mut() {
            let ad = o.get("ad").and_then(|s| s.as_str()).unwrap_or("").to_string();
            o["kullanim"] = serde_json::json!(st.sablon_kullanim.get(&ad).copied().unwrap_or(0));
        }
    }
    Json(v)
}

/// POST /api/sablon — muhasebeci sık attığı kaydı şablon olarak kaydeder.
/// Muhasebe doğrulaması: şablon en az bir BORÇ ve bir ALACAK bacağı içermeli — aksi halde
/// o şablondan üretilen fiş hiçbir tutarla Σborç=Σalacak sağlayamaz (VUK 219 / çift taraflı kayıt).
async fn sablon_ekle(
    State(state): State<Shared>,
    Json(g): Json<KullaniciSablon>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<HataDto>)> {
    let hata = |m: &str| (StatusCode::BAD_REQUEST, Json(HataDto { hata: m.into() }));
    let ad = g.ad.trim().to_string();
    if ad.is_empty() || ad.chars().count() > 60 {
        return Err(hata("şablon adı 1-60 karakter olmalı"));
    }
    if g.satirlar.len() < 2 {
        return Err(hata("şablon en az 2 satır içermeli"));
    }
    if !g.satirlar.iter().any(|s| s.taraf == "B") || !g.satirlar.iter().any(|s| s.taraf == "A") {
        return Err(hata("şablonda en az bir borç ve bir alacak satırı olmalı (çift taraflı kayıt)"));
    }
    {
        let mut st = state.lock().unwrap();
        // Hesap kodları plana ait mı? Olmayan kodla şablon kaydedilirse fiş üretilemez.
        for s in &g.satirlar {
            if !st.hesap_sirali.iter().any(|h| h.kod == s.kod) {
                return Err(hata(&format!("hesap kodu planda yok: {}", s.kod)));
            }
        }
        let hazir_var = serde_json::from_str::<serde_json::Value>(SABLONLAR_JSON)
            .ok()
            .and_then(|v| v.get("sablonlar").and_then(|a| a.as_array()).cloned())
            .map(|a| a.iter().any(|x| x.get("ad").and_then(|s| s.as_str()) == Some(ad.as_str())))
            .unwrap_or(false);
        if hazir_var {
            return Err(hata("bu ad hazır bir şablonda kullanılıyor — başka ad seç"));
        }
        let mut yeni = g.clone();
        yeni.ad = ad.clone();
        if yeni.sektorler.is_empty() {
            yeni.sektorler = vec!["*".into()];
        }
        // Aynı ad varsa üzerine yaz (güncelleme), yoksa ekle.
        match st.kullanici_sablonlar.iter().position(|x| x.ad == ad) {
            Some(i) => st.kullanici_sablonlar[i] = yeni,
            None => st.kullanici_sablonlar.push(yeni),
        }
    }
    Ok(sablonlar(State(state)).await)
}

/// Bir tahsisin muhasebe kaydı. Silinmez — iptalde `durum: IPTAL` olur ve karşısına
/// storno fişi açılır (VUK 219 / TTK 65: kayıt silinemez, karalanamaz; düzeltme ters kayıtla).
#[derive(Serialize, Deserialize, Clone)]
pub struct TahsisKayit {
    /// Mükerrer kaydı önleyen doğal anahtar: tahsilat + fatura + tutar.
    pub anahtar: String,
    pub fis_no: String,
    pub tahsilat_id: String,
    pub fatura_no: String,
    pub firma: String,
    pub tarih: String,
    pub tutar: i64,
    pub yon: String,        // SATIS (tahsilat) | ALIS (ödeme)
    pub kaynak: String,     // OTOMATIK | MANUEL
    pub durum: String,      // AKTIF | IPTAL
    pub satirlar: Vec<simulasyon::FisSat>,
    /// IPTAL ise bu kaydı kapatan storno fişi; storno kaydında ise iptal ettiği asıl fiş.
    pub karsi_fis: Option<String>,
    pub gerekce: Option<String>,
    pub storno: bool,
}

/// Bir tahsisin muhasebe satırları. Tahsilat: 102/100/101/121 B — 120 A · Ödeme: 320 B — 102/100/103/321 A
fn tahsis_satirlari(yon: &str, enstruman: &str, tutar: i64) -> Vec<simulasyon::FisSat> {
    let satis = yon == "SATIS";
    let h = match enstruman {
        "NAKIT" => "100",
        "CEK" => if satis { "101" } else { "103" },
        "SENET" => if satis { "121" } else { "321" },
        _ => "102",
    };
    let b = |k: &str, t: i64| simulasyon::FisSat { hesap: k.into(), borc: t, alacak: 0 };
    let a = |k: &str, t: i64| simulasyon::FisSat { hesap: k.into(), borc: 0, alacak: t };
    if satis { vec![b(h, tutar), a("120", tutar)] } else { vec![b("320", tutar), a(h, tutar)] }
}

/// Defteri güncel tahsis durumuyla SENKRONİZE eder — otomatik sistemin sürekliliği budur.
/// · Yeni tahsis → yeni fiş (yalnız anahtarı defterde yoksa; MÜKERRER KAYIT OLMAZ)
/// · Kalkmış tahsis → asıl kayıt IPTAL + STORNO fişi (silme yok, ters kayıt var)
/// Döner: (yeni fiş sayısı, storno sayısı)
fn donmus_tahsisler(st: &AppState, serbest: &std::collections::HashSet<String>) -> simulasyon::Donmus {
    let mut m = simulasyon::Donmus::new();
    for k in st.tahsis_defteri.iter().filter(|k| k.durum == "AKTIF" && !k.storno) {
        if serbest.contains(&k.tahsilat_id) { continue; } // yeniden dağıtılacak
        m.entry(k.tahsilat_id.clone()).or_default().push((k.fatura_no.clone(), k.tutar));
    }
    m
}

/// Okuma uçları için: defterdeki dağıtım neyse o (kayda geçmiş tahsis dondurulmuştur).
/// ÖNBELLEKLİ — aynı sürümde ikinci çağrı hesaplamaz. Sürüm, dağıtımı etkileyen her
/// mutasyonda `surum_arttir()` ile yükseltilir.
pub fn gorunum(st: &AppState) -> Arc<simulasyon::Dataset> {
    if let Some((s, d)) = st.gorunum_onbellek.lock().unwrap().as_ref() {
        if *s == st.gorunum_surum { return d.clone(); }
    }
    let d = Arc::new(simulasyon::dataset_ex(
        &st.manuel_tahsis, &donmus_tahsisler(st, &std::collections::HashSet::new())));
    *st.gorunum_onbellek.lock().unwrap() = Some((st.gorunum_surum, d.clone()));
    d
}

/// Dağıtımı etkileyen bir mutasyon oldu — önbelleği geçersiz kıl.
pub fn surum_arttir(st: &mut AppState) {
    st.gorunum_surum += 1;
    *st.gorunum_onbellek.lock().unwrap() = None;
}

fn defteri_senkronize(st: &mut AppState, serbest: &std::collections::HashSet<String>) -> (usize, usize) {
    let donmus = donmus_tahsisler(st, serbest);
    let d = simulasyon::dataset_ex(&st.manuel_tahsis, &donmus);
    // Güncel olması gereken tahsisler: anahtar → (kayıt alanları)
    let mut olmasi_gereken: HashMap<String, (String, String, String, String, i64, String, String)> = HashMap::new();
    for f in &d.faturalar {
        for t in &f.tahsisler {
            let anahtar = format!("{}|{}|{}", t.tahsilat_id, f.no, t.tutar);
            olmasi_gereken.insert(anahtar, (
                t.tahsilat_id.clone(), f.no.clone(), f.karsi.clone(), t.tarih.clone(),
                t.tutar, f.yon.clone(),
                if t.kaynak == "MANUEL" { "MANUEL".to_string() } else { "OTOMATIK".to_string() },
            ));
        }
    }

    // 1) Artık geçerli olmayan AKTİF kayıtlar → STORNO (ters kayıt)
    let mut storno_sayisi = 0usize;
    let iptal_edilecek: Vec<usize> = st.tahsis_defteri.iter().enumerate()
        .filter(|(_, k)| k.durum == "AKTIF" && !k.storno && !olmasi_gereken.contains_key(&k.anahtar))
        .map(|(i, _)| i).collect();
    for i in iptal_edilecek {
        st.fis_sayaci += 1;
        let storno_no = format!("STR-{:06}", st.fis_sayaci);
        let asil = st.tahsis_defteri[i].clone();
        // Ters kayıt: borç↔alacak yer değiştirir. Net etki sıfır, iki kayıt da defterde kalır.
        let ters: Vec<simulasyon::FisSat> = asil.satirlar.iter()
            .map(|s| simulasyon::FisSat { hesap: s.hesap.clone(), borc: s.alacak, alacak: s.borc })
            .collect();
        st.tahsis_defteri[i].durum = "IPTAL".into();
        st.tahsis_defteri[i].karsi_fis = Some(storno_no.clone());
        st.tahsis_defteri.push(TahsisKayit {
            anahtar: format!("STORNO|{}", asil.anahtar),
            fis_no: storno_no,
            tahsilat_id: asil.tahsilat_id.clone(),
            fatura_no: asil.fatura_no.clone(),
            firma: asil.firma.clone(),
            tarih: asil.tarih.clone(),
            tutar: asil.tutar,
            yon: asil.yon.clone(),
            kaynak: asil.kaynak.clone(),
            durum: "AKTIF".into(),
            satirlar: ters,
            karsi_fis: Some(asil.fis_no.clone()),
            gerekce: Some(format!("Eşleşme değişti — {} fişinin ters kaydı (VUK 219: kayıt silinmez)", asil.fis_no)),
            storno: true,
        });
        storno_sayisi += 1;
    }

    // 2) Defterde olmayan tahsisler → yeni fiş. Anahtar kontrolü mükerrer kaydı engeller.
    let mevcut: std::collections::HashSet<String> = st.tahsis_defteri.iter()
        .filter(|k| k.durum == "AKTIF" && !k.storno).map(|k| k.anahtar.clone()).collect();
    let mut yeni_sayisi = 0usize;
    let mut yeniler: Vec<(String, (String, String, String, String, i64, String, String))> =
        olmasi_gereken.into_iter().filter(|(a, _)| !mevcut.contains(a)).collect();
    yeniler.sort_by(|a, b| a.0.cmp(&b.0)); // deterministik fiş no sırası
    for (anahtar, (ths, fno, firma, tarih, tutar, yon, kaynak)) in yeniler {
        let enstruman = d.tahsilatlar.iter().find(|t| t.id == ths)
            .map(|t| t.enstruman.clone()).unwrap_or_else(|| "BANKA".into());
        st.fis_sayaci += 1;
        st.tahsis_defteri.push(TahsisKayit {
            anahtar,
            fis_no: format!("THS-F{:06}", st.fis_sayaci),
            tahsilat_id: ths, fatura_no: fno, firma, tarih, tutar,
            satirlar: tahsis_satirlari(&yon, &enstruman, tutar),
            yon, kaynak, durum: "AKTIF".into(),
            karsi_fis: None, gerekce: None, storno: false,
        });
        yeni_sayisi += 1;
    }
    if yeni_sayisi > 0 || storno_sayisi > 0 { surum_arttir(st); }
    (yeni_sayisi, storno_sayisi)
}

/// POST /api/muhasebe/aktar — eşleştirme sonuçlarını GERÇEK deftere (yevmiye/kebir/mizan) yazar.
/// Bu, sistemin eksik halkasıydı: tahsis defteri kendi başına bir kayıt değil, kayıt ADAYIDIR.
/// Aktarım iki katman üretir ve ikisi de kronolojik sırayla kesinleştirilir (VUK 219):
///   1. TAHAKKUK — fatura kaydı (120/600+391 · 153+191/320). Dayanak: fatura no.
///   2. TAHSİLAT — eşleşen para hareketi (102/120 · 320/102). Dayanak: tahsis fiş no.
/// İdempotent: aktarılan her kaydın anahtarı saklanır, ikinci çağrı mükerrer fiş üretmez.
async fn muhasebe_aktar(State(state): State<Shared>) -> Json<serde_json::Value> {
    let mut guard = state.lock().unwrap();
    let st = &mut *guard;
    let d = gorunum(st);

    // (sıra_anahtarı, tarih, tip, belge_no, defter_anahtarı, satırlar) — tarih sırasına dizilir.
    let mut kuyruk: Vec<(String, Tarih, FisTipi, String, String, Vec<YevmiyeSatiri>)> = Vec::new();
    let skey = |t: &str| -> String {
        let p: Vec<&str> = t.split('.').collect();
        if p.len() == 3 { format!("{}{}{}", p[2], p[1], p[0]) } else { t.to_string() }
    };
    // FİŞ TİPİ (VUK şekil şartı): kasa/banka'ya para GİRİŞİ Tahsil, ÇIKIŞI Tediye fişidir;
    // para hareketi doğurmayan kayıt (tahakkuk, iade, iskonto, storno, virman) Mahsup fişidir.
    // Hepsini Mahsup yazmak denetimde fiş tipi tutarsızlığı olarak sorulur.
    let fis_tipi = |satirlar: &Vec<YevmiyeSatiri>| -> FisTipi {
        let kasa_banka = |k: &str| k.starts_with("100") || k.starts_with("101") || k.starts_with("102");
        let giris = satirlar.iter().any(|s| kasa_banka(&s.hesap_id) && s.borc.deger() > 0);
        let cikis = satirlar.iter().any(|s| kasa_banka(&s.hesap_id) && s.alacak.deger() > 0);
        match (giris, cikis) {
            (true, false) => FisTipi::Tahsil,
            (false, true) => FisTipi::Tediye,
            _ => FisTipi::Mahsup, // ikisi birden (virman) veya hiçbiri (tahakkuk) → Mahsup
        }
    };

    let sat = |h: &str, b: i64, a: i64, ack: &str| YevmiyeSatiri {
        hesap_id: h.to_string(), borc: Kurus::yeni(b), alacak: Kurus::yeni(a),
        masraf_yeri_id: None, aciklama: ack.to_string(),
    };

    // AÇILIŞ FİŞİ — dönem başı bakiyeleri. Bu olmadan 102 Bankalar ALACAK bakiye verir:
    // ödemeler girişleri aşınca hesap eksiye düşer. Gerçek muhasebede banka eksiye düşemez;
    // düşüyorsa o kredili mevduattır (KMH) ve 300 Banka Kredileri'ne yazılır, 102'de durmaz.
    {
        let anahtar = "ACILIS|2026".to_string();
        if !st.aktarilan.contains(&anahtar) {
            if let Some(t) = parse_tarih("01.01.2026") {
                let banka = 60_000_000_00i64;
                let kasa = 250_000_00i64;
                kuyruk.push(("20260101".into(), t, FisTipi::Acilis, "ACILIS-2026".to_string(), anahtar,
                    vec![sat("102", banka, 0, "Dönem başı banka bakiyesi"),
                         sat("100", kasa, 0, "Dönem başı kasa bakiyesi"),
                         sat("500", 0, banka + kasa, "Ödenmiş sermaye")]));
            }
        }
    }

    // Faturanın ürettiği TÜM muhasebe fişleri simülasyon motorundan gelir — tahakkuk, iptal
    // (storno), iade (610) ve iskonto (611). Kayıt mantığı TEK yerde yaşasın diye burada
    // yeniden kurulmuyor; TASLAK faturada tahakkuk zaten yoktur (henüz belge değil).
    for f in &d.faturalar {
        let Some(t) = parse_tarih(&f.tarih) else { continue };
        for (etiket, fis) in [
            ("TAHAKKUK", &f.tahakkuk), ("IPTAL", &f.iptal_fisi),
            ("IADE", &f.iade_fisi), ("ISKONTO", &f.iskonto_fisi),
            // Satılan malın maliyeti hasılatla AYNI dönemde deftere girmeli (eşleştirme ilkesi).
            // Ayrı fiş: yevmiyede hasılat kaydı ile stok çıkışı ayrı okunabilsin.
            ("SMM", &f.smm_fisi), ("IADE_STOK", &f.iade_stok_fisi),
        ] {
            let Some(fis) = fis else { continue };
            let anahtar = format!("{etiket}|{}", f.no);
            if st.aktarilan.contains(&anahtar) { continue; }
            let satirlar: Vec<YevmiyeSatiri> = fis.satirlar.iter()
                .map(|x| sat(&x.hesap, x.borc, x.alacak, &f.karsi)).collect();
            let tip = fis_tipi(&satirlar);
            kuyruk.push((skey(&f.tarih), t, tip, f.no.clone(), anahtar, satirlar));
        }
    }

    // BANKA → KASA VİRMANI (100 borç / 102 alacak). Kasa ekranındaki bakiye ile 100 hesabının
    // bakiyesi ancak bu fişler deftere girerse tutar — aksi halde iki gerçeklik oluşurdu.
    let kasa = simulasyon::kasa_defteri(&d);
    if let Some(vs) = kasa.get("virmanlar").and_then(|x| x.as_array()) {
        for v in vs {
            let (Some(tarih_s), Some(tutar)) = (v.get(0).and_then(|x| x.as_str()), v.get(1).and_then(|x| x.as_i64())) else { continue };
            let anahtar = format!("VIRMAN|{tarih_s}");
            if st.aktarilan.contains(&anahtar) { continue; }
            let Some(t) = parse_tarih(tarih_s) else { continue };
            // Virman: hem 100 borç hem 102 alacak → kasa/banka iki yönde de var, Mahsup fişi.
            kuyruk.push((skey(tarih_s), t, FisTipi::Mahsup, format!("VIR-{tarih_s}"), anahtar,
                vec![sat("100", tutar, 0, "Bankadan kasaya virman"), sat("102", 0, tutar, "Kasaya aktarım")]));
        }
    }

    // Tahsis kayıtları: storno dahil (ters kayıt da deftere girer — iptal silme değil kayıttır).
    for k in &st.tahsis_defteri {
        let anahtar = format!("TAHSIS|{}", k.fis_no);
        if st.aktarilan.contains(&anahtar) { continue; }
        let _ = &anahtar;
        let Some(t) = parse_tarih(&k.tarih) else { continue };
        let satirlar: Vec<YevmiyeSatiri> = k.satirlar.iter()
            .map(|s| sat(&s.hesap, s.borc, s.alacak, &k.firma)).collect();
        let ack = if k.storno {
            format!("STORNO — {} fişinin ters kaydı ({})", k.karsi_fis.clone().unwrap_or_default(), k.fatura_no)
        } else {
            format!("{} — {} tahsis ({})", k.firma, k.fatura_no, k.kaynak)
        };
        let _ = ack;
        let tip = fis_tipi(&satirlar);
        kuyruk.push((skey(&k.tarih), t, tip, k.fis_no.clone(), anahtar, satirlar));
    }

    // KRONOLOJİ: yevmiye tarih sırası zorunlu — kuyruk tarihe göre dizilir, sonra kesinleşir.
    kuyruk.sort_by(|a, b| a.0.cmp(&b.0).then(a.3.cmp(&b.3)));
    // Defterde kayıt varsa, ondan eski tarihli aktarım yapılamaz (VUK kronoloji).
    let son_tarih = st.fisler.last().map(|f| f.tarih);
    let mut atlanan_kronoloji = 0usize;

    let mut basarili = 0usize;
    let mut hatali: Vec<String> = Vec::new();
    for (_, tarih, tip, belge_no, anahtar, satirlar) in kuyruk {
        // KRONOLOJİ (VUK 219): yevmiye geriye tarihlenemez. Sonradan doğan kayıt (storno,
        // düzeltilmiş eşleşme) OLAY tarihine değil KAYIT tarihine yazılır — belgenin kendi
        // tarihi dayanakta korunur, böylece iz kopmaz. Muhasebe pratiği de budur.
        let kayit_tarihi = match son_tarih {
            Some(s) if tarih < s => { atlanan_kronoloji += 1; s }
            _ => tarih,
        };
        let mut fis = Fis::taslak(tip, kayit_tarihi, satirlar);
        fis.dayanak = Some(Dayanak {
            belge_tipi: parse_belge("Fatura"), belge_no: belge_no.clone(), belge_tarihi: tarih,
        });
        let sonuc = {
            let ctx = Baglam { hesaplar: &st.hesaplar, donem: &st.donem };
            kesinlestir(&mut fis, &ctx, &mut st.sayac)
        };
        match sonuc {
            Ok(()) => {
                st.fisler.push(fis);
                st.aktarilan.insert(anahtar);
                basarili += 1;
            }
            Err(e) => { if hatali.len() < 5 { hatali.push(format!("{belge_no}: {e:?}")); } }
        }
    }

    let borc: i64 = st.fisler.iter().flat_map(|f| &f.satirlar).map(|s| s.borc.deger()).sum();
    let alacak: i64 = st.fisler.iter().flat_map(|f| &f.satirlar).map(|s| s.alacak.deger()).sum();
    Json(serde_json::json!({
        "aktarilan_fis": basarili,
        "kayit_tarihine_alinan": atlanan_kronoloji,
        "hatalar": hatali,
        "defterdeki_fis": st.fisler.len(),
        "yevmiye_borc": borc, "yevmiye_alacak": alacak, "denge_ok": borc == alacak,
    }))
}

/// POST /api/tahsis/senkronize — otomatik sistemi çalıştır: eşleşmeleri deftere işle.
/// Tekrar tekrar çağrılabilir (idempotent): mevcut kayıt varsa yeniden üretmez.
async fn tahsis_senkronize(State(state): State<Shared>) -> Json<serde_json::Value> {
    let mut st = state.lock().unwrap();
    let (yeni, storno) = defteri_senkronize(&mut st, &std::collections::HashSet::new());
    let aktif = st.tahsis_defteri.iter().filter(|k| k.durum == "AKTIF" && !k.storno).count();
    let borc: i64 = st.tahsis_defteri.iter().flat_map(|k| &k.satirlar).map(|s| s.borc).sum();
    let alacak: i64 = st.tahsis_defteri.iter().flat_map(|k| &k.satirlar).map(|s| s.alacak).sum();
    Json(serde_json::json!({
        "yeni_fis": yeni, "storno_fis": storno, "aktif_kayit": aktif,
        "defter_satir": st.tahsis_defteri.len(),
        "toplam_borc": borc, "toplam_alacak": alacak, "denge_ok": borc == alacak,
    }))
}

#[derive(Deserialize)]
struct BeklemeGirdi { tahsilat_id: String, firma: String, tarih: String }

/// POST /api/havada/fatura-bekleniyor — para hareketi var ama fatura henüz kesilmemiş.
/// Bu bir EŞLEŞTİRME DEĞİLDİR: ortada belge yok, dolayısıyla muhasebe kaydı da doğmaz.
/// Yalnız takip kaydı açar; belge gelince normal eşleştirme yapılır.
async fn havada_bekleme_isaretle(
    State(state): State<Shared>,
    Json(g): Json<BeklemeGirdi>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<HataDto>)> {
    if g.firma.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(HataDto { hata: "beklenen fatura firması seçilmeli".into() })));
    }
    let mut st = state.lock().unwrap();
    st.fatura_bekleyen.insert(g.tahsilat_id.clone(), (g.firma.clone(), g.tarih.clone()));
    Ok(Json(serde_json::json!({
        "tahsilat_id": g.tahsilat_id, "firma": g.firma, "durum": "FATURA_BEKLENIYOR",
        "not": "Belge gelmeden muhasebe kaydı doğmaz. VUK 231: fatura, teslim/ifadan itibaren 7 gün içinde düzenlenmelidir.",
    })))
}

/// DELETE /api/havada/fatura-bekleniyor/:id — takibi kaldır (fatura geldi ya da yanlış işaret).
async fn havada_bekleme_sil(State(state): State<Shared>, Path(id): Path<String>) -> Json<serde_json::Value> {
    let mut st = state.lock().unwrap();
    let vardi = st.fatura_bekleyen.remove(&id).is_some();
    Json(serde_json::json!({ "tahsilat_id": id, "kaldirildi": vardi }))
}

/// GET /api/havada — havada bakiyeler: hangi bankada, ne kadar, neden eşleşmedi, kaç gündür bekliyor.
async fn havada_liste(State(state): State<Shared>) -> Json<serde_json::Value> {
    let (bekleyen, d) = {
        let st = state.lock().unwrap();
        (st.fatura_bekleyen.clone(), gorunum(&st))
    };
    // Bugün (simülasyon dönemi sonu) — gerçek kurulumda sistem tarihi.
    let bugun = "31.12.2026";
    let gk = |t: &str| -> i64 {
        let p: Vec<&str> = t.split('.').collect();
        if p.len() == 3 { p[2].parse::<i64>().unwrap_or(0) * 360 + p[1].parse::<i64>().unwrap_or(0) * 30 + p[0].parse::<i64>().unwrap_or(0) } else { 0 }
    };

    let mut satirlar: Vec<serde_json::Value> = Vec::new();
    let mut banka_ozet: HashMap<String, (usize, i64)> = HashMap::new();
    for t in d.tahsilatlar.iter().filter(|t| t.dagitilan < t.tutar) {
        let acik = t.tutar - t.dagitilan;
        let bekleme = bekleyen.get(&t.id);
        let gun = (gk(bugun) - gk(&t.tarih)).max(0);
        let banka = if t.banka_ad.is_empty() { format!("{} (banka dışı)", t.enstruman) } else { t.banka_ad.clone() };
        let e = banka_ozet.entry(banka.clone()).or_insert((0, 0));
        e.0 += 1; e.1 += acik;
        satirlar.push(serde_json::json!({
            "tahsilat_id": t.id, "tarih": t.tarih, "yon": t.yon,
            "tutar": t.tutar, "acik": acik,
            "firma": t.firma, "enstruman": t.enstruman,
            "banka_ad": banka, "hesap_id": t.hesap_id, "iban": t.iban,
            "sebep": t.sebep,
            "fatura_bekleniyor": bekleme.is_some(),
            "bekleyen_firma": bekleme.map(|x| x.0.clone()).unwrap_or_default(),
            "bekleme_gun": gun,
            // VUK 231: fatura 7 gün içinde düzenlenir. Aşan bekleme HATIRLATMA üretir.
            "hatirlatma": if bekleme.is_some() && gun > 7 {
                format!("{gun} gündür fatura bekleniyor — VUK 231'e göre belge 7 gün içinde düzenlenmeliydi")
            } else { String::new() },
        }));
    }
    satirlar.sort_by(|a, b| b["acik"].as_i64().unwrap_or(0).cmp(&a["acik"].as_i64().unwrap_or(0)));

    let mut bankalar: Vec<serde_json::Value> = banka_ozet.into_iter()
        .map(|(ad, (n, t))| serde_json::json!({ "banka": ad, "adet": n, "tutar": t })).collect();
    bankalar.sort_by(|a, b| b["tutar"].as_i64().unwrap_or(0).cmp(&a["tutar"].as_i64().unwrap_or(0)));

    let toplam: i64 = satirlar.iter().map(|x| x["acik"].as_i64().unwrap_or(0)).sum();
    let gecikmis = satirlar.iter().filter(|x| !x["hatirlatma"].as_str().unwrap_or("").is_empty()).count();
    Json(serde_json::json!({
        "adet": satirlar.len(), "toplam": toplam,
        "bankalar": bankalar,
        "fatura_bekleyen": satirlar.iter().filter(|x| x["fatura_bekleniyor"] == true).count(),
        "gecikmis_hatirlatma": gecikmis,
        "satirlar": satirlar,
        "not": "Eşleşmemiş para: ya karşı taraf ünvanı yok ya da fatura henüz kesilmemiş. Eşleşene kadar tamlık ihlali / kayıt dışı hasılat riski taşır (VUK, BDS 500).",
    }))
}

/// GET /api/kontrol/saglik — SİSTEM SAĞLIĞI: tüm değişmezler + ters bakiye taraması tek uçta.
/// Her mutasyondan sonra ve dağıtım öncesi çağrılmalı. `saglikli: false` ise sistemde BUG var.
async fn kontrol_saglik(State(state): State<Shared>) -> Json<serde_json::Value> {
    let (d, borc, alacak, fis_sayisi) = {
        let st = state.lock().unwrap();
        let b: i64 = st.fisler.iter().flat_map(|f| &f.satirlar).map(|s| s.borc.deger()).sum();
        let a: i64 = st.fisler.iter().flat_map(|f| &f.satirlar).map(|s| s.alacak.deger()).sum();
        (gorunum(&st), b, a, st.fisler.len())
    };
    let degismez = kontrol::degismezler(&d, borc, alacak);
    let anomali = denetim_anomali(State(state)).await.0;
    let ters = anomali["bulgu_sayisi"].as_u64().unwrap_or(0);

    Json(serde_json::json!({
        "saglikli": degismez.is_empty() && ters == 0,
        "degismez_ihlali": degismez.len(),
        "degismezler": degismez,
        "ters_bakiye": ters,
        "ters_bakiyeler": anomali["bulgular"],
        "defter": { "fis": fis_sayisi, "borc": borc, "alacak": alacak, "denge_ok": borc == alacak },
        "not": "degismez_ihlali > 0 ise motor hatası vardır (para yaratılmış/kaybolmuş olabilir). ters_bakiye > 0 ise kayıt eksikliği vardır; her ters bakiyenin açıklanabilir sebebi olmalıdır.",
    }))
}

/// GET /api/denetim/anomali — TERS BAKİYE ve mantıksal tutarsızlık taraması.
///
/// Her TDHP hesabının bir DOĞASI vardır (aktif→borç, pasif→alacak). Bakiye doğasına ters
/// düşüyorsa ya kayıt hatalıdır ya da gerçek bir olay eksik kaydedilmiştir. Bu, vergi
/// incelemesinde ilk bakılan yerlerden biridir; her ters bakiyenin bir AÇIKLAMASI olmalıdır.
async fn denetim_anomali(State(state): State<Shared>) -> Json<serde_json::Value> {
    let st = state.lock().unwrap();
    // Defter nesnesi kurmak 11 bin fişi klonluyor (~0,7 sn). Yaprak hesapta rollup gereksiz —
    // alt hesabı yok, bakiyesi kendi satırlarının toplamı. Tek geçişte topla.
    let mut bakiyeler: HashMap<&str, (i64, i64)> = HashMap::new();
    for f in &st.fisler {
        for sr in &f.satirlar {
            let e = bakiyeler.entry(sr.hesap_id.as_str()).or_insert((0, 0));
            e.0 += sr.borc.deger();
            e.1 += sr.alacak.deger();
        }
    }
    let mut bulgular: Vec<serde_json::Value> = Vec::new();

    for h in st.hesap_sirali.iter().filter(|h| h.yaprak) {
        let (b, a) = bakiyeler.get(h.kod.as_str()).copied().unwrap_or((0, 0));
        let bakiye = b - a;
        if bakiye == 0 { continue; }
        let dogasi_borc = matches!(h.dogasi, HesapDogasi::Borc);
        let ters = (dogasi_borc && bakiye < 0) || (!dogasi_borc && bakiye > 0);
        if !ters { continue; }

        // Ters bakiyenin ANLAMI hesaba göre değişir — kör "hata" demek yanlış olur.
        let (onem, aciklama, oneri) = match h.kod.as_str() {
            k if k.starts_with("100") => ("YUKSEK",
                "Kasa alacak bakiye veremez — fiilen olmayan para ödenmiş görünüyor.",
                "Kayıt dışı hasılat veya eksik belge göstergesi; bankadan kasaya virman eksik olabilir."),
            k if k.starts_with("102") => ("YUKSEK",
                "Banka alacak bakiye veremez — hesapta olmayan para ödenmiş görünüyor.",
                "Gerçekte kredili mevduat (KMH) kullanıldıysa 300 Banka Kredileri'ne alınmalı; yoksa eksik tahsilat kaydı var."),
            k if k.starts_with("120") => ("ORTA",
                "Alıcılar alacak bakiye veriyor — müşteriden borcundan fazla tahsilat.",
                "Alınan avans ise 340 Alınan Avanslar'a alınmalı; değilse fazla/mükerrer tahsilat kaydı."),
            k if k.starts_with("320") => ("ORTA",
                "Satıcılar borç bakiye veriyor — satıcıya borcundan fazla ödeme.",
                "Verilen avans ise 159 Verilen Sipariş Avansları'na alınmalı; değilse mükerrer ödeme."),
            k if k.starts_with("191") || k.starts_with("391") => ("ORTA",
                "KDV hesabı ters bakiye veriyor.",
                "İade/düzeltme kayıtları ay sonu mahsubuyla uyumsuz olabilir; 191↔391 mahsubu kontrol edilmeli."),
            k if k.starts_with("153") => ("YUKSEK",
                "Stok alacak bakiye veriyor — olmayan mal satılmış görünüyor (negatif stok).",
                "Alış faturası eksik/geç kaydedilmiş olabilir; sayım ve belge kontrolü gerekir."),
            _ => ("DUSUK",
                "Bakiye hesabın doğasına ters.",
                "Kayıt yönü veya hesap seçimi kontrol edilmeli."),
        };
        bulgular.push(serde_json::json!({
            "kod": h.kod, "ad": h.ad,
            "dogasi": if dogasi_borc { "Borç" } else { "Alacak" },
            "bakiye": bakiye, "borc": b, "alacak": a,
            "onem": onem, "aciklama": aciklama, "oneri": oneri,
        }));
    }
    let yuksek = bulgular.iter().filter(|x| x["onem"] == "YUKSEK").count();
    Json(serde_json::json!({
        "bulgu_sayisi": bulgular.len(), "yuksek_onem": yuksek, "bulgular": bulgular,
        "not": "Ters bakiye her zaman hata değildir; ama her ters bakiyenin açıklanabilir bir sebebi olmalıdır (VUK 219 kayıt düzeni).",
    }))
}

/// GET /api/tahsis/defter — tahsis muhasebe defteri (iptal + storno dahil, denetim izi).
async fn tahsis_defter(
    State(state): State<Shared>,
    Query(q): Query<HashMap<String, String>>,
) -> Json<serde_json::Value> {
    let st = state.lock().unwrap();
    let mut liste: Vec<&TahsisKayit> = st.tahsis_defteri.iter().collect();
    if let Some(f) = q.get("fatura_no").filter(|s| !s.is_empty()) { liste.retain(|k| &k.fatura_no == f); }
    if let Some(t) = q.get("tahsilat_id").filter(|s| !s.is_empty()) { liste.retain(|k| &k.tahsilat_id == t); }
    if q.get("sadece_aktif").map(|s| s == "1").unwrap_or(false) { liste.retain(|k| k.durum == "AKTIF"); }
    let borc: i64 = liste.iter().flat_map(|k| &k.satirlar).map(|s| s.borc).sum();
    let alacak: i64 = liste.iter().flat_map(|k| &k.satirlar).map(|s| s.alacak).sum();
    let limit: usize = q.get("limit").and_then(|x| x.parse().ok()).unwrap_or(100).min(1000);
    let toplam = liste.len();
    liste.truncate(limit);
    Json(serde_json::json!({
        "toplam": toplam, "toplam_borc": borc, "toplam_alacak": alacak, "denge_ok": borc == alacak,
        "kayitlar": liste,
    }))
}

#[derive(Deserialize)]
struct TahsisGirdi { tahsilat_id: String, fatura_no: String }

/// POST /api/tahsis/manuel — muhasebeci bir tahsilatı elle bir faturaya bağlar.
/// FIFO karinesi ezilir: bu tahsilatın ÖNCEKİ otomatik eşleşmesi kalkar (dağıtım baştan
/// hesaplanır), kalan para yine FIFO akar. Yanıt: eşleşme öncesi/sonrası fark.
async fn tahsis_manuel(
    State(state): State<Shared>,
    Json(g): Json<TahsisGirdi>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<HataDto>)> {
    let hata = |m: String| (StatusCode::BAD_REQUEST, Json(HataDto { hata: m }));
    let onceki = { let st = state.lock().unwrap(); gorunum(&st) };


    // ── GÜVENLİK KATMANI: numaralı ön kontrol ──────────────────────────────
    // Kurallar tek yerde (kontrol.rs). ENGEL varsa işlem YAPILMAZ ve kullanıcıya
    // ne yapacağı söylenir — "hata var" demek yetmez.
    let manuel_mevcut = { state.lock().unwrap().manuel_tahsis.clone() };
    let _ = &hata;
    // ÖN KONTROL ÖNCE ÇALIŞIR. Eskiden burada `.expect("K01 geçti, tahsilat bulunmalı")`
    // vardı ve K01'DEN ÖNCE koşuyordu: olmayan bir tahsilat_id gönderildiğinde handler
    // panikliyor, istemci boş yanıt (bağlantı kopması) alıyordu. Kuralı çalıştırmadan
    // "kural geçti" varsaymak, güvenlik katmanını hiç kurmamakla aynı şey.
    let on = kontrol::tahsis_on_kontrol(&onceki, &manuel_mevcut, &g.tahsilat_id, &g.fatura_no);
    let engeller: Vec<&kontrol::Bulgu> = on.iter().filter(|x| x.onem == "ENGEL").collect();
    if !engeller.is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(HataDto {
            hata: format!("[{}] {} → {}", engeller[0].kod, engeller[0].mesaj, engeller[0].cozum),
        })));
    }
    let uyarilar: Vec<&kontrol::Bulgu> = on.iter().filter(|x| x.onem == "UYARI").collect();
    let uyari_json = serde_json::to_value(&uyarilar).unwrap_or(serde_json::Value::Null);

    // K01 buraya gelmeyi zaten engeller; yine de panik yerine hata döndürüyoruz —
    // bir kuralın çalıştığını varsaymak, çalıştığını doğrulamak değildir.
    let Some(t) = onceki.tahsilatlar.iter().find(|x| x.id == g.tahsilat_id).cloned() else {
        return Err((StatusCode::NOT_FOUND, Json(HataDto {
            hata: format!("Tahsilat bulunamadı: {}", g.tahsilat_id),
        })));
    };

    // Öncesi: tüm dağıtımın fotoğrafı (tahsilat_id, fatura_no) → tutar.
    // Sonrasıyla karşılaştırıp NE OLDUĞUNU ölçeceğiz — tahmin etmeyeceğiz.
    let foto = |d: &simulasyon::Dataset| -> HashMap<(String, String), i64> {
        let mut m = HashMap::new();
        for fa in &d.faturalar {
            for ts in &fa.tahsisler {
                *m.entry((ts.tahsilat_id.clone(), fa.no.clone())).or_insert(0) += ts.tutar;
            }
        }
        m
    };
    let once = foto(&onceki);

    { let mut st = state.lock().unwrap(); st.manuel_tahsis.insert(g.tahsilat_id.clone(), g.fatura_no.clone()); surum_arttir(&mut st); }

    // Muhasebe defterini senkronize et: kalkan eşleşmeler STORNO'lanır, yenisi fişlenir.
    // Yalnız BU tahsilat serbest bırakılır → geçmiş dağıtımlar donmuş kalır, zincirleme kayma olmaz.
    let (yeni_fis, storno_fis) = {
        let mut st = state.lock().unwrap();
        let serbest: std::collections::HashSet<String> = [g.tahsilat_id.clone()].into_iter().collect();
        defteri_senkronize(&mut st, &serbest)
    };
    let sonra = { let st = state.lock().unwrap(); gorunum(&st) };

    // Mutasyon sonrası DEĞİŞMEZ denetimi — boş olmalı. Dolu ise bu bir BUG'dır, sessiz geçilmez.
    let degismez_json = {
        let st = state.lock().unwrap();
        let b: i64 = st.fisler.iter().flat_map(|f| &f.satirlar).map(|s| s.borc.deger()).sum();
        let a: i64 = st.fisler.iter().flat_map(|f| &f.satirlar).map(|s| s.alacak.deger()).sum();
        serde_json::to_value(kontrol::degismezler(&sonra, b, a)).unwrap_or(serde_json::Value::Null)
    };

    // Sonrası ile karşılaştır — NE OLDU'yu tahminle değil, ölçerek söyle.
    let sonra_f = foto(&sonra);
    let mut dusurulen: Vec<serde_json::Value> = Vec::new();   // yer açmak için kaldırılanlar
    let mut tasinanlar: Vec<serde_json::Value> = Vec::new();  // yeniden dağıtılanlar
    for (k, eski_tutar) in &once {
        let yeni_tutar = sonra_f.get(k).copied().unwrap_or(0);
        if yeni_tutar < *eski_tutar {
            let fark = eski_tutar - yeni_tutar;
            let kayit = serde_json::json!({ "tahsilat_id": k.0, "fatura_no": k.1, "tutar": fark });
            if k.0 == g.tahsilat_id { tasinanlar.push(kayit) } else { dusurulen.push(kayit) }
        }
    }
    let hedefe_yazilan = sonra_f.get(&(g.tahsilat_id.clone(), g.fatura_no.clone())).copied().unwrap_or(0);
    let toplam_dagitim: i64 = sonra_f.iter().filter(|(k, _)| k.0 == g.tahsilat_id).map(|(_, v)| *v).sum();
    let artan = t.tutar - toplam_dagitim;
    let hedef_f = sonra.faturalar.iter().find(|x| x.no == g.fatura_no);

    // SONUÇ — maker ne olduğunu net görsün. Sessiz "hiçbir şey olmadı" durumu artık yok.
    let tl = |k: i64| format!("{},{:02}", (k / 100).to_string(), (k % 100).abs());
    let (sonuc, aciklama) = if hedefe_yazilan == 0 {
        ("YAZILAMADI", format!(
            "Tahsilat hedefe yazılamadı. Fatura net borcu {} kuruş ve tamamı manuel kararlarla dolu —              önce oradaki manuel eşleştirmeyi kaldır.",
            hedef_f.map(|f| f.net_borc).unwrap_or(0)))
    } else if hedefe_yazilan < t.tutar {
        ("KISMEN_YAZILDI", format!(
            "Tahsilatın {} TL'si hedefe yazıldı — faturanın borcu bu kadardı. Aşan {} TL aynı carinin diğer açık faturalarına FIFO ile dağıtıldı; {} TL hiç dağıtılamadı (avans, karşılığı fatura yok).",
            tl(hedefe_yazilan), tl(t.tutar - hedefe_yazilan), tl(artan.max(0))))
    } else if hedef_f.map(|f| f.kalan > 0).unwrap_or(false) {
        ("TAM_YAZILDI_FATURA_ACIK", format!(
            "Tahsilatın tamamı hedefe yazıldı ama fatura KAPANMADI — {} TL açık kaldı (kısmi ödeme, cari borç sürüyor).",
            tl(hedef_f.map(|f| f.kalan).unwrap_or(0))))
    } else {
        ("TAM_YAZILDI_FATURA_KAPANDI", "Tahsilatın tamamı hedefe yazıldı ve fatura kapandı.".to_string())
    };

    Ok(Json(serde_json::json!({
        "tahsilat_id": g.tahsilat_id,
        "fatura_no": g.fatura_no,
        "sonuc": sonuc,
        "aciklama": aciklama,
        "tahsilat_tutari": t.tutar,
        "hedefe_yazilan": hedefe_yazilan,
        "artan_avans": artan.max(0),
        // Yer açmak için BAŞKA tahsilatlardan düşürülenler — o para serbest kalıp yeniden dağıtıldı.
        "dusurulen_tahsisler": dusurulen,
        // Bu tahsilatın eski faturalarından çekilen kısımlar.
        "kaldirilan_otomatik_eslesmeler": tasinanlar,
        "muhasebe": { "yeni_fis": yeni_fis, "storno_fis": storno_fis },
        "on_kontrol_uyarilari": uyari_json,
        "degismez_ihlalleri": degismez_json,
        "fatura": sonra.faturalar.iter().find(|x| x.no == g.fatura_no).map(|f| serde_json::json!({
            "no": f.no, "toplam": f.toplam, "tahsil": f.tahsil, "kalan": f.kalan,
            "tahsisler": f.tahsisler,
        })),
    })))
}

/// DELETE /api/tahsis/manuel/:tahsilat_id — manuel kararı geri alır, tahsilat FIFO'ya döner.
async fn tahsis_manuel_sil(
    State(state): State<Shared>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<HataDto>)> {
    let mut st = state.lock().unwrap();
    let vardi = st.manuel_tahsis.remove(&id).is_some();
    if !vardi {
        return Err((StatusCode::NOT_FOUND, Json(HataDto { hata: "bu tahsilatta manuel eşleştirme yok".into() })));
    }
    surum_arttir(&mut st);

    // DEFTERİ DE GERİ AL. Eskiden burada yalnız `manuel_tahsis` haritasından siliniyordu:
    // tahsilat `serbest` kümesine girmediği için `donmus_tahsisler` onun DEFTERDEKİ aktif
    // kaydını hâlâ donmuş sayıyor ve motor adım 0'da aynı faturaya yeniden uyguluyordu.
    // Sonuç: manuel karar fiilen geri alınamıyor, üstelik aynı tahsis okuma modelinde
    // "FIFO" etiketleniyordu — insan kararı motora atfedilip denetim izi bozuluyordu.
    // `tahsis_manuel` ile aynı desen: yalnız BU tahsilat serbest bırakılır.
    let serbest: std::collections::HashSet<String> = [id.clone()].into_iter().collect();
    let (yeni_fis, storno_fis) = defteri_senkronize(&mut st, &serbest);

    Ok(Json(serde_json::json!({
        "tahsilat_id": id,
        "durum": "FIFO'ya döndü",
        "yeni_fis": yeni_fis,
        "storno_fis": storno_fis,
        "not": "Manuel eşleştirme kaldırıldı; defterdeki kayıt storno ile kapatıldı ve tahsilat FIFO'ya iade edildi (VUK 219 — kayıt silinmez).",
    })))
}

/// POST /api/sablon/:ad/kullan — şablon uygulandığında sayacı artırır (sık kullanılan öne gelsin).
async fn sablon_kullan(State(state): State<Shared>, Path(ad): Path<String>) -> Json<serde_json::Value> {
    let mut st = state.lock().unwrap();
    let e = st.sablon_kullanim.entry(ad).or_insert(0);
    *e += 1;
    Json(serde_json::json!({ "kullanim": *e }))
}

/// DELETE /api/sablon/:ad — yalnız kullanıcının kendi şablonunu siler (hazır şablonlar korunur).
async fn sablon_sil(
    State(state): State<Shared>,
    Path(ad): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<HataDto>)> {
    {
        let mut st = state.lock().unwrap();
        let onceki = st.kullanici_sablonlar.len();
        st.kullanici_sablonlar.retain(|x| x.ad != ad);
        if st.kullanici_sablonlar.len() == onceki {
            return Err((
                StatusCode::NOT_FOUND,
                Json(HataDto { hata: "kullanıcı şablonu bulunamadı (hazır şablonlar silinemez)".into() }),
            ));
        }
    }
    Ok(sablonlar(State(state)).await)
}

async fn departmanlar() -> Json<serde_json::Value> {
    Json(serde_json::from_str(DEPARTMANLAR_JSON).unwrap_or_else(|_| serde_json::json!({})))
}

async fn kademeler() -> Json<serde_json::Value> {
    Json(serde_json::from_str(KADEMELER_JSON).unwrap_or_else(|_| serde_json::json!({})))
}

async fn mukellefler(State(state): State<Shared>, headers: HeaderMap) -> Json<serde_json::Value> {
    let st = state.lock().unwrap();
    let (admin, izinli) = oturumdaki(&st, &headers)
        .map(|m| (m.rol == "admin", m.mukellef_idleri.clone()))
        .unwrap_or((false, vec![]));
    let liste: Vec<MukellefProfil> = st.profiller.iter()
        .filter(|p| admin || izinli.contains(&p.id))
        .cloned().collect();
    Json(serde_json::json!({ "aktif": st.aktif, "mukellefler": liste }))
}

#[derive(Deserialize)]
struct YeniMukellef {
    unvan: String,
    #[serde(default)]
    vkn: String,
    #[serde(default)]
    sektor_kodlari: Vec<String>,
    #[serde(default)]
    maliyet_secenegi: String,
}

async fn mukellef_ekle(State(state): State<Shared>, Json(g): Json<YeniMukellef>) -> Json<serde_json::Value> {
    let mut st = state.lock().unwrap();
    let id = format!("m{}", st.profiller.iter().filter_map(|p| p.id.trim_start_matches('m').parse::<u32>().ok()).max().unwrap_or(0) + 1);
    st.profiller.push(MukellefProfil {
        id: id.clone(), unvan: g.unvan, vkn: g.vkn, sektor_kodlari: g.sektor_kodlari,
        maliyet_secenegi: if g.maliyet_secenegi.is_empty() { "yok".into() } else { g.maliyet_secenegi },
    });
    st.arsiv.insert(id.clone(), AppState::bos_kayit());
    Json(serde_json::json!({ "ok": true, "id": id }))
}

/// Aktif mükellefi değiştirir: mevcut çalışma setini arşive yaz, hedefi yükle (swap).
async fn mukellef_aktif(
    State(state): State<Shared>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<HataDto>)> {
    let mut st = state.lock().unwrap();
    let yetkili = oturumdaki(&st, &headers)
        .map(|m| m.rol == "admin" || m.mukellef_idleri.contains(&id))
        .unwrap_or(false);
    if !yetkili {
        return Err((StatusCode::FORBIDDEN, Json(HataDto { hata: "bu mükellefe erişim yetkiniz yok".into() })));
    }
    if st.aktif == id {
        return Ok(Json(serde_json::json!({ "ok": true, "aktif": id })));
    }
    if !st.profiller.iter().any(|p| p.id == id) {
        return Err((StatusCode::BAD_REQUEST, Json(HataDto { hata: format!("mükellef yok: {id}") })));
    }
    let eski = std::mem::replace(&mut st.aktif, id.clone());
    let cikan = st.topla();
    st.arsiv.insert(eski, cikan);
    let giren = st.arsiv.remove(&id).unwrap_or_else(AppState::bos_kayit);
    st.yerlestir(giren);
    Ok(Json(serde_json::json!({ "ok": true, "aktif": id })))
}

// ---- Denetim çalışma kağıtları (H.4) — BDS 230 formatı, TMS/TFRS dayanaklı ----

const DENETIM_JSON: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/denetim-programlari.json"));

/// Önek eşleşmesi: "600" → 600 ve 600.xx; "60" → 60x grubu; "*" → tüm defter.
fn onek_altinda(kod: &str, onek: &str) -> bool {
    if onek == "*" || kod == onek {
        return true;
    }
    if onek.contains('.') {
        return kod.starts_with(&format!("{onek}."));
    }
    kod.starts_with(onek)
}

#[derive(Serialize)]
struct KagitHesap {
    kod: String,
    ad: String,
    borc: i64,
    alacak: i64,
    borc_bakiye: i64,
    alacak_bakiye: i64,
    hareket: usize,
    aylik: Vec<i64>,
}
#[derive(Serialize)]
struct KagitTest {
    motor: String,
    ad: String,
    durum: String,
    bulgular: Vec<String>,
}
#[derive(Serialize)]
struct KagitNitelik {
    onek: String,
    ad: String,
    tanim: String,
}
#[derive(Serialize)]
struct KagitDto {
    id: String,
    ad: String,
    sektor: String,
    standart: String,
    siddet: String,
    amac: String,
    ref_no: String,
    donem: String,
    tarih: String,
    hazirlayan: String,
    onekler: Vec<String>,
    nitelikler: Vec<KagitNitelik>,
    hesaplar: Vec<KagitHesap>,
    testler: Vec<KagitTest>,
    not_metni: String,
    /// Kağıt üstü manuel düzenlemeler (hucreler: kod:alan → değer; satirlar: manuel eklenen satırlar).
    duzenlemeler: serde_json::Value,
}

async fn denetim_programlar() -> Json<serde_json::Value> {
    Json(serde_json::from_str(DENETIM_JSON).unwrap_or_else(|_| serde_json::json!({})))
}

async fn denetim_kagit(
    State(state): State<Shared>,
    Path(id): Path<String>,
) -> Result<Json<KagitDto>, (StatusCode, Json<HataDto>)> {
    let hata = |m: String| (StatusCode::BAD_REQUEST, Json(HataDto { hata: m }));
    let prog: serde_json::Value = serde_json::from_str(DENETIM_JSON).unwrap_or_default();
    let mut secili: Option<(serde_json::Value, String)> = None;
    if let Some(sektorler) = prog["sektorler"].as_array() {
        for s in sektorler {
            if let Some(cs) = s["calismalar"].as_array() {
                for c in cs {
                    if c["id"].as_str() == Some(id.as_str()) {
                        secili = Some((c.clone(), s["ad"].as_str().unwrap_or("").to_string()));
                    }
                }
            }
        }
    }
    let (c, sektor) = secili.ok_or_else(|| hata(format!("çalışma bulunamadı: {id}")))?;
    let onekler: Vec<String> = c["hesaplar"]
        .as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    let st = state.lock().unwrap();
    let d = defter_kur(&st.fisler);

    // İlgili hesap tespiti: defterde hareket görmüş hesaplar, önek taramasıyla (muavin kuralı).
    let mut hesaplar: Vec<KagitHesap> = d
        .mizan()
        .into_iter()
        .filter(|m| onekler.iter().any(|o| o != "*" && onek_altinda(&m.hesap_id, o)))
        .map(|m| KagitHesap {
            ad: st.hesaplar.get(&m.hesap_id).map(|h| h.ad.clone()).unwrap_or_default(),
            kod: m.hesap_id,
            borc: m.borc.deger(),
            alacak: m.alacak.deger(),
            borc_bakiye: m.borc_bakiye.deger(),
            alacak_bakiye: m.alacak_bakiye.deger(),
            hareket: 0,
            aylik: vec![0; 12],
        })
        .collect();

    // Hareket sayısı + aylık net seri (hesap bazlı) + önek bazlı |net| seri (M1 girdisi) — tek geçiş.
    let kod_idx: HashMap<String, usize> =
        hesaplar.iter().enumerate().map(|(i, h)| (h.kod.clone(), i)).collect();
    let mut aylik_onek: HashMap<String, [i64; 12]> = HashMap::new();
    for f in &st.fisler {
        let ay = (f.tarih.ay as usize).clamp(1, 12) - 1;
        for s in &f.satirlar {
            if let Some(&i) = kod_idx.get(s.hesap_id.as_str()) {
                hesaplar[i].hareket += 1;
                let net = s.borc.deger() - s.alacak.deger();
                hesaplar[i].aylik[ay] += net;
                for o in &onekler {
                    if o != "*" && onek_altinda(&s.hesap_id, o) {
                        aylik_onek.entry(o.clone()).or_insert([0; 12])[ay] += net.abs();
                    }
                }
            }
        }
    }

    // Testler. M7 her kağıtta koşar (temel doğruluk taraması); çalışmanın kendi motoru varsa o da.
    let motor = c["motor"].as_str().unwrap_or("").to_string();
    let mut testler: Vec<KagitTest> = Vec::new();
    let mut m7 = Vec::new();
    for h in &hesaplar {
        if let Some(hs) = st.hesaplar.get(&h.kod) {
            let ters = match hs.dogasi {
                HesapDogasi::Borc => h.alacak_bakiye > 0,
                HesapDogasi::Alacak => h.borc_bakiye > 0,
            };
            if ters {
                m7.push(format!(
                    "{} {} doğasına ({}) ters bakiye veriyor",
                    h.kod,
                    h.ad,
                    doga_str(&hs.dogasi)
                ));
            }
        }
    }
    testler.push(KagitTest {
        motor: "M7".into(),
        ad: "Doğaya ters bakiye taraması".into(),
        durum: "çalıştı".into(),
        bulgular: m7,
    });
    if motor == "M1" {
        let esik = c["parametre"]["esik_sapma_pct"].as_f64().unwrap_or(15.0);
        let mut m1 = Vec::new();
        for o in &onekler {
            let Some(seri) = aylik_onek.get(o) else { continue };
            let dolu: Vec<i64> = seri.iter().copied().filter(|v| *v != 0).collect();
            if dolu.len() < 3 {
                continue;
            }
            let ort = dolu.iter().sum::<i64>() as f64 / dolu.len() as f64;
            for (ai, v) in seri.iter().enumerate() {
                if *v == 0 {
                    continue;
                }
                let sapma = 100.0 * (*v as f64 - ort) / ort.abs().max(1.0);
                if sapma.abs() > esik {
                    m1.push(format!(
                        "{}*: {}. ay hareket hacmi ortalamadan %{:.0} sapıyor — detay incelemeye alınmalı",
                        o,
                        ai + 1,
                        sapma
                    ));
                }
            }
        }
        testler.push(KagitTest {
            motor: "M1".into(),
            ad: "Aylık seri sapma analitiği (BDS 520)".into(),
            durum: "çalıştı".into(),
            bulgular: m1,
        });
    }
    match motor.as_str() {
        "M1" | "M7" => {}
        "M4" => testler.push(KagitTest {
            motor: motor.clone(),
            ad: "Mükerrer tarama".into(),
            durum: "kısmi — belge no mükerrerliği kayıt anında engelleniyor; tutar+karşı+tarih taraması planlı (H.2)".into(),
            bulgular: vec![],
        }),
        "M12" => testler.push(KagitTest {
            motor: motor.clone(),
            ad: "Kronoloji/süreklilik".into(),
            durum: "çalışıyor — kronoloji koruması ve müteselsil yevmiye no kayıt anında uygulanıyor".into(),
            bulgular: vec![],
        }),
        m => testler.push(KagitTest {
            motor: m.to_string(),
            ad: "Sektör çalışması motoru".into(),
            durum: "planlı (H.2) — motor eklenince bu kağıtta otomatik koşacak".into(),
            bulgular: vec![],
        }),
    }

    // Hesap nitelikleri (MSUGT resmi tanım) — kağıdın dayanak bölümü.
    let nitelikler: Vec<KagitNitelik> = onekler
        .iter()
        .filter(|o| o.as_str() != "*")
        .filter_map(|o| {
            st.aciklamalar.get(o).map(|a| KagitNitelik {
                onek: o.clone(),
                ad: a.ad.clone(),
                tanim: a.tanim.clone(),
            })
        })
        .collect();

    let not_metni = st.denetim_notlar.get(&id).cloned().unwrap_or_default();
    let duzenlemeler = st
        .kagit_duzenlemeler
        .get(&id)
        .cloned()
        .unwrap_or_else(|| serde_json::json!({ "hucreler": {}, "satirlar": [] }));
    Ok(Json(KagitDto {
        ref_no: format!("ÇK-{}/{}", id, st.donem.bitis.yil),
        id,
        ad: c["ad"].as_str().unwrap_or("").to_string(),
        sektor,
        standart: c["bds"].as_str().unwrap_or("").to_string(),
        siddet: c["siddet"].as_str().unwrap_or("").to_string(),
        amac: c["anlatim"].as_str().unwrap_or("").to_string(),
        donem: format!("{} – {}", tarih_str(&st.donem.baslangic), tarih_str(&st.donem.bitis)),
        tarih: tarih_str(&st.donem.bitis),
        hazirlayan: "SMMM Kullanıcı".into(),
        onekler,
        nitelikler,
        hesaplar,
        testler,
        not_metni,
        duzenlemeler,
    }))
}

/// Kağıt üstü düzenlemeleri kaydet. Serbest şema (proaktif değişiklik ilkesi):
/// {hucreler: {"102.01:borc": kuruş|metin}, satirlar: [{kod,ad,borc,alacak,not}]}
async fn denetim_duzenle(
    State(state): State<Shared>,
    Path(id): Path<String>,
    Json(g): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let mut st = state.lock().unwrap();
    st.kagit_duzenlemeler.insert(id, g);
    Json(serde_json::json!({ "ok": true }))
}

#[derive(Deserialize)]
struct NotGirdi {
    not_metni: String,
}
async fn denetim_not(
    State(state): State<Shared>,
    Path(id): Path<String>,
    Json(g): Json<NotGirdi>,
) -> Json<serde_json::Value> {
    let mut st = state.lock().unwrap();
    st.denetim_notlar.insert(id, g.not_metni);
    Json(serde_json::json!({ "ok": true }))
}

// ═══════ UFRS WorkSheet — WTB (dönüşüm mizanı) + denetçi çalışmaları (görev #26) ═══════
//
// Model: "dışarıdan atılan kayıtlar" (top-side entries). VUK defterine ASLA dokunulmaz;
// denetçinin TFRS/TMS çerçevesindeki çalışmalarından doğan AJE/RJE kayıtları WTB'ye akar:
//   VUK bakiye → [RJE sınıflama] → [AJE düzeltme] → TFRS bakiye
// Her kayıtta zorunlu: DAYANAK (ekspertiz raporu / piyasa uygunluğu / hesaplama / teyit)
// + DENETÇİ NOTU + hazırlayan/tarih izi (oturumdan — izsiz kayıt kabul edilmez).
// TMS 29 endekslemesi şu an uygulanmıyor (katalogda uyuyan); GUD ekseni esastır.

/// Duran varlık envanteri — her alt hesabın KİMLİĞİ (marka/model, seri/plaka, edinim, ömür).
/// "Kayıt en ince ayrıntısına kadar açıklanmalı": üretim hattı = 10 bileşen, binek = 6 araç + tamamlayıcı.
const VARLIK_JSON: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/duran-varlik-envanteri.json"));

/// Bir hesabın (veya grubun) varlık kartları + defterdeki bakiyeleri.
/// GET /api/varlik/:kod — "253.01" → 10 bileşen; "254.01.001" → tek araç.
async fn varlik_karti(
    State(state): State<Shared>,
    Path(kod): Path<String>,
) -> Json<serde_json::Value> {
    let st = state.lock().unwrap();
    let d = defter_kur(&st.fisler);
    let env: serde_json::Value = serde_json::from_str(VARLIK_JSON).unwrap_or_default();
    let bos = vec![];
    let liste = env["varliklar"].as_array().unwrap_or(&bos);
    let mut out: Vec<serde_json::Value> = Vec::new();
    for v in liste {
        let vk = v["kod"].as_str().unwrap_or_default();
        if !onek_altinda(vk, &kod) { continue; }
        let (b, a) = d.hesap_bakiyesi(vk);
        let mut o = v.clone();
        o["defter_bakiye"] = serde_json::json!(b.deger() - a.deger());
        out.push(o);
    }
    let toplam: i64 = out.iter().filter_map(|v| v["defter_bakiye"].as_i64()).sum();
    let maliyet: i64 = out.iter().filter_map(|v| v["maliyet_kurus"].as_i64()).sum();
    Json(serde_json::json!({ "kod": kod, "adet": out.len(), "toplam_bakiye": toplam, "toplam_maliyet": maliyet, "varliklar": out }))
}

const UFRS_JSON: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/ufrs-calismalar.json"));

/// Bir kayıt satırı: TDHP hesabı (mizanda var) VEYA serbest TFRS kalemi
/// (ör. "Kullanım Hakkı Varlığı" — TDHP'de karşılığı yok, WTB'de kendi satırı olur).
#[derive(Clone, Serialize, Deserialize)]
struct UfrsKayitSatir {
    hesap: String,
    borc: i64,
    alacak: i64,
    /// true = serbest TFRS kalemi (TDHP dışı); false = TDHP kodu — yapraklık doğrulanır,
    /// "ana.alt" deseninde alt yoksa otomatik muavin açılır (TFRS düzeltme kırılımı, ör. 129.90).
    #[serde(default)]
    serbest: bool,
}

#[derive(Clone, Serialize)]
struct UfrsKayit {
    no: String,        // "AJE-16-01" (standart no gömülü) / "RJE-01"
    tur: String,       // "AJE" (kâr etkiler) | "RJE" (tutar-nötr sınıflama)
    standart: String,  // "TMS 16", "TFRS 9", "TMS 1"…
    kaynak_ws: String, // doğduğu çalışma: "WS-GUD-MDV"
    aciklama: String,
    satirlar: Vec<UfrsKayitSatir>,
    dayanak_tur: String, // ekspertiz | piyasa | hesaplama | sozlesme | aktueryal | teyit
    dayanak_ref: String, // rapor no / kaynak tanımı — ZORUNLU
    denetci_notu: String, // ZORUNLU
    /// Ölçüm yöntemi (katalogdaki degerleme_yontemleri kodu) — ZORUNLU.
    /// "Kaydın hesabını verebilmek": hangi mantıkla değerlendi?
    #[serde(default)]
    degerleme_yontemi: String,
    /// Yöntemin NEYİ BAZ ALDIĞI (ör. "31.12.2026 ekspertiz raporu m² birim değeri";
    /// "son 3 yıl tahsilat verisinden kalibre temerrüt oranları") — ZORUNLU.
    #[serde(default)]
    degerleme_bazi: String,
    durum: String,        // "onerildi" | "vazgecildi"
    hazirlayan: String,   // oturumdan — sistem atar
    tarih: String,        // sistem atar (gg.aa.yyyy)
    /// Raporlama dönemi (yıl) — sistem atar; kesin döneme kayıt atılamaz.
    donem: String,
    /// Dönem devrinde davranış (katalogdan): TASI_BIRAK | TERS_CEVIR | TEKRARLA | TASIMA_YOK | YENIDEN_URET
    devir: String,
    /// Bu kayıt bir devir (CF) fişiyse kaynağı: "2026/AJE-16-01"
    devir_kaynak: Option<String>,
}

/// Sistem tarihi (UTC) → gg.aa.yyyy. Howard Hinnant civil_from_days algoritması (chrono'suz).
fn bugun_str() -> String {
    let sn = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let z = sn / 86_400 + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{:02}.{:02}.{:04}", d, m, y)
}

/// Aktif raporlama dönemi (yıl) — UFRS kayıtları bu döneme atılır.
fn ufrs_aktif_donem(st: &AppState) -> String {
    st.donem.baslangic.yil.to_string()
}

/// Katalogdan çalışmanın devir davranışını oku (yoksa TASI_BIRAK).
fn ufrs_ws_devir(ws: &str) -> String {
    serde_json::from_str::<serde_json::Value>(UFRS_JSON).ok()
        .and_then(|v| v["calismalar"].as_array()?.iter()
            .find(|c| c["id"] == ws)?["devir"].as_str().map(String::from))
        .unwrap_or_else(|| "TASI_BIRAK".into())
}

/// Çalışma kataloğu + kayıt sayıları + aktif mükellef sektörüne UYGUNLUK.
/// Sektör ilkesi: üretim 7/A akışı, inşaat YYİO gibi çalışmalar yalnız ilgili sektör
/// profillerinde uygun; WTB satırları zaten mizandan geldiği için sektöre kendiliğinden uyar.
async fn ufrs_calismalar(State(state): State<Shared>) -> Json<serde_json::Value> {
    let st = state.lock().unwrap();
    let aktif_sektorler: Vec<String> = st.profiller.iter()
        .find(|p| p.id == st.aktif)
        .map(|p| p.sektor_kodlari.clone())
        .unwrap_or_default();
    let mut v: serde_json::Value = serde_json::from_str(UFRS_JSON).unwrap_or_default();
    if let Some(calismalar) = v["calismalar"].as_array_mut() {
        for c in calismalar.iter_mut() {
            let ws = c["id"].as_str().unwrap_or_default().to_string();
            let sayi = st.ufrs_kayitlar.iter().filter(|k| k.kaynak_ws == ws && k.durum == "onerildi").count();
            c["kayit_sayisi"] = serde_json::json!(sayi);
            let uygun = c["sektorler"].as_array().map(|a| {
                a.iter().any(|s| s == "*" || aktif_sektorler.iter().any(|ak| s == ak.as_str()))
            }).unwrap_or(true);
            c["uygun"] = serde_json::json!(uygun);
        }
    }
    v["donem"] = serde_json::json!(ufrs_aktif_donem(&st));
    v["kesin_donemler"] = serde_json::json!(st.ufrs_kesin_donemler);
    Json(v)
}

/// Çalışma detayı: tanım + defterden girdi hesapları (önek eşleşme) + bu çalışmanın kayıtları.
async fn ufrs_calisma(
    State(state): State<Shared>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<HataDto>)> {
    let st = state.lock().unwrap();
    let katalog: serde_json::Value = serde_json::from_str(UFRS_JSON).unwrap_or_default();
    let tanim = katalog["calismalar"]
        .as_array()
        .and_then(|a| a.iter().find(|c| c["id"] == id.as_str()))
        .cloned()
        .ok_or((StatusCode::NOT_FOUND, Json(HataDto { hata: format!("çalışma yok: {id}") })))?;
    let onekler: Vec<String> = tanim["hesaplar"]
        .as_array()
        .map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect())
        .unwrap_or_default();
    let d = defter_kur(&st.fisler);
    let varlik_json: serde_json::Value = serde_json::from_str(VARLIK_JSON).unwrap_or_default();
    let bos_v: Vec<serde_json::Value> = Vec::new();
    let varlik_env = varlik_json["varliklar"].as_array().unwrap_or(&bos_v).clone();
    // Hareket sayısı tek geçişte — otomatik veri çekiminin parçası (denetim kağıdı deseni)
    let mut hareket: HashMap<String, usize> = HashMap::new();
    for f in &st.fisler {
        for sat in &f.satirlar {
            if onekler.iter().any(|o| onek_altinda(&sat.hesap_id, o)) {
                *hareket.entry(sat.hesap_id.clone()).or_insert(0) += 1;
            }
        }
    }
    let girdi: Vec<serde_json::Value> = d
        .mizan()
        .into_iter()
        .filter(|m| onekler.iter().any(|o| onek_altinda(&m.hesap_id, o)))
        .map(|m| {
            let ad = st.hesaplar.get(&m.hesap_id).map(|h| h.ad.clone()).unwrap_or_default();
            let kebir = m.hesap_id.split('.').next().unwrap_or(&m.hesap_id).to_string();
            // Varlık kimliği (envanterden) — kayıt en ince ayrıntısıyla açıklanır
            let vk = varlik_env.iter().find(|v| v["kod"] == m.hesap_id.as_str());
            serde_json::json!({
                "kod": m.hesap_id, "ad": ad, "kebir": kebir,
                "borc_bakiye": m.borc_bakiye.deger(), "alacak_bakiye": m.alacak_bakiye.deger(),
                "net": m.borc_bakiye.deger() - m.alacak_bakiye.deger(),
                "hareket": hareket.get(&m.hesap_id).copied().unwrap_or(0),
                "kimlik": vk.map(|v| format!("{} · {} · edinim {}",
                    v["marka_model"].as_str().unwrap_or(""), v["seri_plaka"].as_str().unwrap_or(""), v["edinim"].as_str().unwrap_or(""))),
                "vuk_omur": vk.and_then(|v| v["vuk_omur"].as_i64()),
                "tfrs_omur": vk.and_then(|v| v["tfrs_omur"].as_i64()),
                "tamamlayici_of": vk.and_then(|v| v["tamamlayici_of"].as_str()),
            })
        })
        .collect();
    let kayitlar: Vec<&UfrsKayit> = st.ufrs_kayitlar.iter().filter(|k| k.kaynak_ws == id).collect();
    Ok(Json(serde_json::json!({ "tanim": tanim, "girdi_hesaplar": girdi, "kayitlar": kayitlar })))
}

#[derive(Deserialize)]
struct UfrsKayitGirdi {
    kaynak_ws: String,
    tur: String,
    standart: String,
    aciklama: String,
    satirlar: Vec<UfrsKayitSatir>,
    dayanak_tur: String,
    dayanak_ref: String,
    denetci_notu: String,
    #[serde(default)]
    degerleme_yontemi: String,
    #[serde(default)]
    degerleme_bazi: String,
}

/// Yeni AJE/RJE — oturum ZORUNLU (hazırlayan izi), denge + dayanak + denetçi notu doğrulanır.
async fn ufrs_kayit_ekle(
    State(state): State<Shared>,
    headers: HeaderMap,
    Json(g): Json<UfrsKayitGirdi>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<HataDto>)> {
    let hata = |k: StatusCode, m: &str| (k, Json(HataDto { hata: m.into() }));
    let mut st = state.lock().unwrap();
    let hazirlayan = oturumdaki(&st, &headers)
        .map(|k| k.ad.clone())
        .ok_or_else(|| hata(StatusCode::UNAUTHORIZED, "oturum gerekli — kayıt hazırlayan izi olmadan atılamaz"))?;
    if g.tur != "AJE" && g.tur != "RJE" {
        return Err(hata(StatusCode::BAD_REQUEST, "tür AJE veya RJE olmalı"));
    }
    if g.satirlar.len() < 2 {
        return Err(hata(StatusCode::BAD_REQUEST, "kayıt en az 2 satır ister (borç + alacak)"));
    }
    for s in &g.satirlar {
        if s.hesap.trim().is_empty() {
            return Err(hata(StatusCode::BAD_REQUEST, "satırda hesap/kalem boş olamaz"));
        }
        if s.borc < 0 || s.alacak < 0 || (s.borc > 0) == (s.alacak > 0) {
            return Err(hata(StatusCode::BAD_REQUEST, "her satırda borç YA DA alacak (tek taraf, pozitif) olmalı"));
        }
    }
    // ALT KIRILIM doğrulaması (serbest olmayan satırlar) — fiş V4 paritesi:
    // yaprak şart; muavin açılmış anaya kayıt RED (alternatifler mesajda);
    // "ana.alt" deseninde alt yoksa OTOMATİK muavin açılır (TFRS düzeltme kırılımı).
    for sat in &g.satirlar {
        if sat.serbest { continue; }
        let kod = sat.hesap.trim().to_string();
        // Kayıt DETAYA atılır: sınıf/grup (1-2 hane) hesap değildir — kebir en az 3 hane.
        if kod.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) && kod.split('.').next().unwrap_or("").len() < 3 {
            return Err(hata(StatusCode::BAD_REQUEST, &format!(
                "{} bir sınıf/grup kodu — kayıt kebire (3 hane) ya da alt kırılıma atılır; üst toplamlar WTB'de kendiliğinden oluşur", kod)));
        }
        if st.hesaplar.contains_key(&kod) {
            // UFRS düzeltme katmanı: denetçi HEM alt kırılıma HEM üst hesaba kayıt atabilir (kullanıcı kararı).
            // VUK fişindeki yaprak zorunluluğu burada YOK — bu katman deftere işlemez, WTB zaten kebire toplar.
        } else if let Some((ana, alt)) = kod.split_once('.') {
            if !st.hesaplar.contains_key(ana) {
                return Err(hata(StatusCode::BAD_REQUEST, &format!(
                    "hesap yok: {} (ana {} tanımsız) — TFRS kalemi ise satırı 'serbest' işaretleyin", kod, ana)));
            }
            let ana_h = st.hesaplar.get(ana).cloned().unwrap();
            let m = muavin_olustur(&ana_h, alt, "TFRS Düzeltmeleri");
            if let Some(hh) = st.hesaplar.get_mut(ana) { hh.yaprak = false; }
            for hh in st.hesap_sirali.iter_mut() { if hh.kod == ana { hh.yaprak = false; } }
            st.hesaplar.insert(m.kod.clone(), m.clone());
            st.hesap_sirali.push(m);
            st.hesap_sirali.sort_by(|a, b| a.kod.cmp(&b.kod));
        } else {
            return Err(hata(StatusCode::BAD_REQUEST, &format!(
                "hesap yok: {} — TFRS kalemi ise satırı 'serbest' işaretleyin", kod)));
        }
    }
    let tb: i64 = g.satirlar.iter().map(|s| s.borc).sum();
    let ta: i64 = g.satirlar.iter().map(|s| s.alacak).sum();
    if tb != ta {
        return Err(hata(StatusCode::BAD_REQUEST, &format!("kayıt dengesiz: borç {} ≠ alacak {}", tb, ta)));
    }
    if g.dayanak_ref.trim().is_empty() {
        return Err(hata(StatusCode::BAD_REQUEST, "dayanak zorunlu — ekspertiz raporu / piyasa verisi / hesaplama referansı girin"));
    }
    if g.denetci_notu.trim().is_empty() {
        return Err(hata(StatusCode::BAD_REQUEST, "denetçi notu zorunlu (BDS 230 — yapılan iş/sonuç)"));
    }
    if g.degerleme_yontemi.trim().is_empty() {
        return Err(hata(StatusCode::BAD_REQUEST, "değerleme yöntemi zorunlu — kaydın hangi ölçüm mantığıyla atıldığı belirtilmeli"));
    }
    if g.degerleme_bazi.trim().is_empty() {
        return Err(hata(StatusCode::BAD_REQUEST, "değerlemenin neyi baz aldığı zorunlu (ör. 'ekspertiz raporu m² birim değeri', 'son 3 yıl tahsilat verisi')"));
    }
    // ---- FAZ 2 doğrulamaları (teşhis B1-B4, B7) ----
    let katalog: serde_json::Value = serde_json::from_str(UFRS_JSON).unwrap_or_default();
    // B3: kaynak_ws katalogda olmalı — "hayalet çalışma"ya kayıt engellenir.
    let calisma_v = katalog["calismalar"].as_array()
        .and_then(|a| a.iter().find(|c| c["id"] == g.kaynak_ws.as_str()).cloned());
    if !g.kaynak_ws.trim().is_empty() && calisma_v.is_none() {
        return Err(hata(StatusCode::BAD_REQUEST, &format!("çalışma tanımsız: {} — kayıt bir çalışma sayfasına bağlı olmalı", g.kaynak_ws)));
    }
    // B7: dayanak türü ve değerleme yöntemi katalogdan olmalı (serbest metin "asdf" geçemez).
    let dayanak_gecerli = katalog["dayanak_turleri"].as_array()
        .map(|a| a.iter().any(|d| d["kod"] == g.dayanak_tur.as_str())).unwrap_or(true);
    if !dayanak_gecerli {
        return Err(hata(StatusCode::BAD_REQUEST, &format!("dayanak türü tanımsız: {} — katalogdaki türlerden biri olmalı", g.dayanak_tur)));
    }
    if let Some(c) = &calisma_v {
        if let Some(yontemler) = c["degerleme_yontemleri"].as_array().filter(|a| !a.is_empty()) {
            if !yontemler.iter().any(|y| y == g.degerleme_yontemi.as_str()) {
                let liste: Vec<&str> = yontemler.iter().filter_map(|y| y.as_str()).collect();
                return Err(hata(StatusCode::BAD_REQUEST, &format!(
                    "değerleme yöntemi '{}' bu çalışmada tanımlı değil — geçerli: {}", g.degerleme_yontemi, liste.join(", "))));
            }
        }
    }
    // KAPALI DÜNYA: standart hangi hesapları ilgilendiriyorsa kayıt YALNIZ onlara atılabilir.
    // Küme = çalışmanın hesap önekleri + kayit_hesaplari + senaryo bacakları + EV hesapları.
    if let Some(c) = &calisma_v {
        let onekler: Vec<String> = c["hesaplar"].as_array()
            .map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
        let mut sabit_kebir: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut kebir_ekle = |kod: &str| {
            if kod.chars().next().map(|ch| ch.is_ascii_digit()).unwrap_or(false) {
                sabit_kebir.insert(kod.split('.').next().unwrap_or("").to_string());
            }
        };
        for yon in ["borc", "alacak"] {
            for h in c["kayit_hesaplari"][yon].as_array().unwrap_or(&vec![]) {
                if let Some(k) = h["kod"].as_str() { kebir_ekle(k); }
            }
        }
        for s in c["senaryolar"].as_array().unwrap_or(&vec![]) {
            for b in s["bacaklar"].as_array().unwrap_or(&vec![]) {
                if let Some(k) = b["hesap"].as_str() { if k != "@hedef" { kebir_ekle(k); } }
            }
        }
        if let Some(ev) = katalog["ev_hesaplari"].as_object() {
            for (_, v) in ev { if let Some(k) = v.as_str() { kebir_ekle(k); } }
        }
        if !onekler.is_empty() || !sabit_kebir.is_empty() {
            for s in &g.satirlar {
                if s.serbest { continue; }
                let kod = s.hesap.trim();
                let kebir = kod.split('.').next().unwrap_or("");
                let uygun = onekler.iter().any(|o| kod.starts_with(o.as_str())) || sabit_kebir.contains(kebir);
                if !uygun {
                    let mut kapsam: Vec<String> = onekler.iter().map(|o| format!("{}xx", o)).collect();
                    let mut sk: Vec<&String> = sabit_kebir.iter().collect();
                    sk.sort();
                    kapsam.extend(sk.into_iter().cloned());
                    return Err(hata(StatusCode::BAD_REQUEST, &format!(
                        "{} hesabı {} kapsamı dışında — bu standart yalnız şu hesapları ilgilendirir: {}. Farklı hesap başka çalışmanın konusudur.",
                        kod, c["standart"].as_str().unwrap_or("çalışma"), kapsam.join(", "))));
                }
            }
        }
    }
    // B2: RJE kâr-nötr olmalı (TMS 1 sunum sınıflaması) — 6xx/7xx bacağı kârı oynatır, AJE ister.
    if g.tur == "RJE" {
        for s in &g.satirlar {
            if !s.serbest && (s.hesap.starts_with('6') || s.hesap.starts_with('7')) {
                return Err(hata(StatusCode::BAD_REQUEST, &format!(
                    "RJE kâr/zararı etkileyemez: {} bir sonuç/maliyet hesabı — kâr etkili düzeltme AJE olarak atılmalı (WTB kâr köprüsü yalnız AJE sayar)", s.hesap)));
            }
        }
    }
    // B4: mükerrer kilidi — açık dönemde aynı çalışmaya aynı satır setiyle ikinci kayıt çift düzeltme yaratır.
    let mut yeni_imza: Vec<(String, i64, i64)> = g.satirlar.iter()
        .map(|s| (s.hesap.trim().to_string(), s.borc, s.alacak)).collect();
    yeni_imza.sort();
    let donem_simdi = ufrs_aktif_donem(&st);
    if let Some(eski) = st.ufrs_kayitlar.iter().find(|k| {
        if k.durum != "onerildi" || k.kaynak_ws != g.kaynak_ws || k.donem != donem_simdi { return false; }
        let mut i: Vec<(String, i64, i64)> = k.satirlar.iter()
            .map(|s| (s.hesap.trim().to_string(), s.borc, s.alacak)).collect();
        i.sort();
        i == yeni_imza
    }) {
        return Err(hata(StatusCode::CONFLICT, &format!(
            "mükerrer kayıt: {} aynı çalışmada aynı satır setiyle zaten açık — çift düzeltme/çift karşılık oluşur; önce onu vazgeçin ya da tutarı değiştirin", eski.no)));
    }
    // B1: hesap-doğa uyarısı (hesap-kurallari.json) — RED değil; azalış/iptal meşru olabilir, karar denetçinin.
    let mut dogrulama_uyarilari: Vec<String> = Vec::new();
    for s in &g.satirlar {
        if s.serbest { continue; }
        let kod = s.hesap.trim();
        let kebir: String = kod.chars().take(3).collect();
        if let Some(kural) = st.kurallar.get(kod).or_else(|| st.kurallar.get(&kebir)) {
            if kural.doga == "B" && s.alacak > 0 {
                dogrulama_uyarilari.push(format!("⚠ {} ({}) borç doğalı — alacak bacağı azalış/iptal demektir; kastınız buysa sorun yok.", kod, kural.ad));
            } else if kural.doga == "A" && s.borc > 0 {
                dogrulama_uyarilari.push(format!("⚠ {} ({}) alacak doğalı — borç bacağı azalış/iptal demektir; kastınız buysa sorun yok.", kod, kural.ad));
            }
        }
    }
    // Numara: {TUR}-{standart rakamları|SNF}-{sıra}
    let std_kisa: String = g.standart.chars().filter(|c| c.is_ascii_digit()).collect();
    let std_kisa = if std_kisa.is_empty() { "SNF".to_string() } else { std_kisa };
    let onek = format!("{}-{}-", g.tur, std_kisa);
    let sira = st.ufrs_kayitlar.iter().filter(|k| k.no.starts_with(&onek)).count() + 1;
    let donem = ufrs_aktif_donem(&st);
    if st.ufrs_kesin_donemler.contains(&donem) {
        return Err(hata(StatusCode::CONFLICT, &format!("{} dönemi KESİNLEŞMİŞ — kayıt atılamaz; sonraki dönem dosyasında çalışın", donem)));
    }
    let kayit = UfrsKayit {
        no: format!("{}{:02}", onek, sira),
        tur: g.tur,
        standart: g.standart,
        devir: ufrs_ws_devir(&g.kaynak_ws),
        kaynak_ws: g.kaynak_ws,
        aciklama: g.aciklama,
        satirlar: g.satirlar,
        dayanak_tur: g.dayanak_tur,
        dayanak_ref: g.dayanak_ref,
        denetci_notu: g.denetci_notu,
        degerleme_yontemi: g.degerleme_yontemi,
        degerleme_bazi: g.degerleme_bazi,
        durum: "onerildi".into(),
        hazirlayan,
        tarih: bugun_str(),
        donem,
        devir_kaynak: None,
    };
    let no = kayit.no.clone();
    st.ufrs_kayitlar.push(kayit);
    Ok(Json(serde_json::json!({ "ok": true, "no": no, "uyarilar": dogrulama_uyarilari })))
}

/// Kayıttan vazgeç (silme YOK — durum değişir, iz kalır; SUM/düzeltilmemiş yanlışlıklara aday).
async fn ufrs_kayit_vazgec(
    State(state): State<Shared>,
    headers: HeaderMap,
    Path(no): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<HataDto>)> {
    let hata = |k: StatusCode, m: &str| (k, Json(HataDto { hata: m.into() }));
    let mut st = state.lock().unwrap();
    if oturumdaki(&st, &headers).is_none() {
        return Err(hata(StatusCode::UNAUTHORIZED, "oturum gerekli"));
    }
    let kesin = st.ufrs_kesin_donemler.clone();
    let k = st.ufrs_kayitlar.iter_mut().find(|k| k.no == no)
        .ok_or_else(|| hata(StatusCode::NOT_FOUND, "kayıt yok"))?;
    if kesin.contains(&k.donem) {
        return Err(hata(StatusCode::CONFLICT, "kesinleşmiş dönemin kaydından vazgeçilemez"));
    }
    if k.tur == "CF" {
        return Err(hata(StatusCode::BAD_REQUEST, "devir (CF) kaydı elle değiştirilemez — kaynağı önceki dönem dosyasıdır"));
    }
    k.durum = "vazgecildi".into();
    Ok(Json(serde_json::json!({ "ok": true })))
}

/// Tüm kayıtlar (kayıt defteri / JE register).
async fn ufrs_kayitlar_listesi(State(state): State<Shared>) -> Json<Vec<UfrsKayit>> {
    let st = state.lock().unwrap();
    Json(st.ufrs_kayitlar.clone())
}

#[derive(Serialize)]
struct WtbSatir {
    kod: String,   // kebir (3 hane) veya TFRS kalem adı
    ad: String,
    tdhp: bool,    // false = yalnız TFRS'de var olan kalem (ör. Kullanım Hakkı Varlığı)
    sinif: String, // 1-9 TDHP sınıfı ("T" = TFRS-only kalem) — bölüm/ara toplam gruplaması
    vuk: i64,      // net bakiye (borç +, alacak −) — Per Books/Unadjusted
    aje: i64,      // AJE net (Big-4 değer zinciri: AJE önce)
    duzeltilmis: i64, // = vuk + aje (Adjusted)
    rje: i64,      // RJE net (sınıflama — tutar-nötr)
    cf: i64,       // devir (CF) net — önceki dönemden kümüle açılış katmanı
    tfrs: i64,     // = duzeltilmis + rje + cf (Final)
    aje_refs: Vec<String>, // bu satırı etkileyen kayıt no'ları (delinebilir tutar ilkesi)
    rje_refs: Vec<String>,
    cf_refs: Vec<String>,
}

/// WTB (Working Trial Balance / dönüşüm mizanı) — Big-4 değer zinciri:
/// VUK (Per Books) → AJE → Düzeltilmiş (Adjusted) → RJE → CF (devir) → TFRS (Final).
/// Kebir düzeyi + sınıf gruplaması + satır başına kayıt referansları (her tutar delinebilir)
/// + denge kontrolleri + kâr köprüsü + dönem/kesinlik durumu.
async fn ufrs_wtb(State(state): State<Shared>) -> Json<serde_json::Value> {
    let st = state.lock().unwrap();
    let donem = ufrs_aktif_donem(&st);
    let kesin = st.ufrs_kesin_donemler.contains(&donem);
    let d = defter_kur(&st.fisler);
    // 1) VUK tarafı: mizanı kebire (3 hane) topla
    let mut satirlar: std::collections::BTreeMap<String, WtbSatir> = std::collections::BTreeMap::new();
    for m in d.mizan() {
        let kebir = m.hesap_id.split('.').next().unwrap_or(&m.hesap_id).to_string();
        let sinif = kebir.chars().next().map(String::from).unwrap_or_default();
        let e = satirlar.entry(kebir.clone()).or_insert_with(|| WtbSatir {
            ad: st.hesaplar.get(&kebir).map(|h| h.ad.clone()).unwrap_or_default(),
            kod: kebir, tdhp: true, sinif, vuk: 0, aje: 0, duzeltilmis: 0, rje: 0, cf: 0, tfrs: 0,
            aje_refs: Vec::new(), rje_refs: Vec::new(), cf_refs: Vec::new(),
        });
        e.vuk += m.borc_bakiye.deger() - m.alacak_bakiye.deger();
    }
    // 2) Kayıt katmanı: yalnız AKTİF DÖNEMİN önerildi kayıtları WTB'ye akar
    for k in st.ufrs_kayitlar.iter().filter(|k| k.durum == "onerildi" && k.donem == donem) {
        for sat in &k.satirlar {
            let (anahtar, tdhp) = if !sat.serbest && (st.hesaplar.contains_key(&sat.hesap) || st.hesaplar.contains_key(sat.hesap.split('.').next().unwrap_or(""))) {
                (sat.hesap.split('.').next().unwrap_or(&sat.hesap).to_string(), true)
            } else {
                (sat.hesap.clone(), false)
            };
            let sinif = if tdhp { anahtar.chars().next().map(String::from).unwrap_or_default() } else { "T".to_string() };
            let e = satirlar.entry(anahtar.clone()).or_insert_with(|| WtbSatir {
                ad: if tdhp { st.hesaplar.get(&anahtar).map(|h| h.ad.clone()).unwrap_or_default() } else { anahtar.clone() },
                kod: if tdhp { anahtar.clone() } else { "—".into() },
                tdhp, sinif, vuk: 0, aje: 0, duzeltilmis: 0, rje: 0, cf: 0, tfrs: 0,
                aje_refs: Vec::new(), rje_refs: Vec::new(), cf_refs: Vec::new(),
            });
            let net = sat.borc - sat.alacak;
            match k.tur.as_str() {
                "RJE" => { e.rje += net; if !e.rje_refs.contains(&k.no) { e.rje_refs.push(k.no.clone()); } }
                "CF" => { e.cf += net; if !e.cf_refs.contains(&k.no) { e.cf_refs.push(k.no.clone()); } }
                _ => { e.aje += net; if !e.aje_refs.contains(&k.no) { e.aje_refs.push(k.no.clone()); } }
            }
        }
    }
    // 3) Değer zinciri + sıfır satır eleme + sıralama (sınıf → kod; TFRS-only en sonda kendi sınıfında)
    let mut rows: Vec<WtbSatir> = satirlar
        .into_values()
        .map(|mut r| {
            r.duzeltilmis = r.vuk + r.aje;
            r.tfrs = r.duzeltilmis + r.rje + r.cf;
            r
        })
        .filter(|r| r.vuk != 0 || r.aje != 0 || r.rje != 0 || r.cf != 0)
        .collect();
    rows.sort_by(|a, b| (a.sinif.clone(), !a.tdhp, a.kod.clone(), a.ad.clone()).cmp(&(b.sinif.clone(), !b.tdhp, b.kod.clone(), b.ad.clone())));
    // 4) Kontrol satırları + kâr köprüsü
    let t_vuk: i64 = rows.iter().map(|r| r.vuk).sum();
    let t_aje: i64 = rows.iter().map(|r| r.aje).sum();
    let t_rje: i64 = rows.iter().map(|r| r.rje).sum();
    let t_cf: i64 = rows.iter().map(|r| r.cf).sum();
    let t_tfrs: i64 = rows.iter().map(|r| r.tfrs).sum();
    let vuk_kar: i64 = rows.iter().filter(|r| r.tdhp && r.sinif == "6").map(|r| -r.vuk).sum();
    let aje_kar_etkisi: i64 = st.ufrs_kayitlar.iter()
        .filter(|k| k.durum == "onerildi" && k.donem == donem && k.tur == "AJE")
        .flat_map(|k| k.satirlar.iter())
        .filter(|sat| sat.hesap.starts_with('6') || sat.hesap.starts_with('7'))
        .map(|sat| sat.alacak - sat.borc)
        .sum();
    Json(serde_json::json!({
        "donem": donem,
        "kesin": kesin,
        "satirlar": rows,
        "kontrol": {
            "vuk_sifir": t_vuk == 0, "aje_sifir": t_aje == 0, "rje_sifir": t_rje == 0,
            "cf_sifir": t_cf == 0, "tfrs_sifir": t_tfrs == 0,
            "vuk": t_vuk, "aje": t_aje, "rje": t_rje, "cf": t_cf, "tfrs": t_tfrs,
        },
        "kar_koprusu": {
            "vuk_kar": vuk_kar,
            "aje_etkisi": aje_kar_etkisi,
            "tfrs_kar": vuk_kar + aje_kar_etkisi,
            "not": "VUK dönem sonucu (6'lı sınıf net) + AJE gelir tablosu etkisi = TFRS dönem sonucu"
        }
    }))
}

/// Dönemi KESİNLEŞTİR — Y3 kademe yetkisi ("kesinlestir") ister. Kesin döneme kayıt atılmaz;
/// kesin dönem, devir (CF) üretiminin kaynağı olur. BDS 230 dosya kilidi ruhu.
async fn ufrs_kesinlestir(
    State(state): State<Shared>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<HataDto>)> {
    let hata = |k: StatusCode, m: &str| (k, Json(HataDto { hata: m.into() }));
    let mut st = state.lock().unwrap();
    if !islem_yetkisi(&st, &headers, "kesinlestir") {
        return Err(hata(StatusCode::FORBIDDEN, "dönem kesinleştirme yalnız 'kesinlestir' yetkili kademede"));
    }
    let donem = ufrs_aktif_donem(&st);
    if st.ufrs_kesin_donemler.contains(&donem) {
        return Err(hata(StatusCode::CONFLICT, "dönem zaten kesin"));
    }
    // Kapanış kontrolü: dengesiz/vazgeçilmemiş sorun yok (kayıtlar girişte dengeli) — açık uç: eksik dayanaklı kayıt olamaz (girişte zorunlu).
    st.ufrs_kesin_donemler.push(donem.clone());
    let sayi = st.ufrs_kayitlar.iter().filter(|k| k.donem == donem && k.durum == "onerildi").count();
    Ok(Json(serde_json::json!({ "ok": true, "donem": donem, "kesinlesen_kayit": sayi })))
}

/// TAŞI-BIRAK dönüşümü (devir çekirdeği): önceki dönem AJE'sinin bilanço bacakları AYNEN,
/// gelir tablosu bacakları (6xx/7xx) 570 Geçmiş Yıllar Kârları'na (zarar netinde 580 — burada
/// kalem bazında 570 kullanılır; işaret zaten bacakta). Kayıt dengesi korunur.
fn tasi_birak(satirlar: &[UfrsKayitSatir]) -> Vec<UfrsKayitSatir> {
    satirlar.iter().map(|s| UfrsKayitSatir {
        hesap: if !s.serbest && (s.hesap.starts_with('6') || s.hesap.starts_with('7')) { "570".into() } else { s.hesap.clone() },
        borc: s.borc,
        alacak: s.alacak,
        serbest: s.serbest,
    }).collect()
}

/// DEVİR: kesinleşmiş kaynak dönemin kayıtlarını devir kurallarına göre aktif döneme taşı.
/// Kural seti (araştırma — CaseWare/CCH pratiği): TASI_BIRAK & TEKRARLA & TERS_CEVIR → CF fişi
/// (taşı-bırak dönüşümüyle; TERS_CEVIR ayrıca "yeniden ölçüm zorunlu" notu düşer);
/// TASIMA_YOK & YENIDEN_URET → taşınmaz (bilgi listesine). Kümüle ilke: firmalar kesintisiz
/// ilerler — CF katmanı önceki dönem TFRS pozisyonunu yeni dönemin açılışına kurar.
async fn ufrs_devir(
    State(state): State<Shared>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<HataDto>)> {
    let hata = |k: StatusCode, m: &str| (k, Json(HataDto { hata: m.into() }));
    let mut st = state.lock().unwrap();
    let hazirlayan = oturumdaki(&st, &headers)
        .map(|k| k.ad.clone())
        .ok_or_else(|| hata(StatusCode::UNAUTHORIZED, "oturum gerekli"))?;
    let hedef = ufrs_aktif_donem(&st);
    if st.ufrs_kesin_donemler.contains(&hedef) {
        return Err(hata(StatusCode::CONFLICT, "aktif dönem kesin — devir hedefi açık dönem olmalı"));
    }
    // Kaynak: hedeften küçük en büyük kesin dönem (yıl sırası)
    let kaynak = st.ufrs_kesin_donemler.iter()
        .filter(|d| d.as_str() < hedef.as_str())
        .max()
        .cloned()
        .ok_or_else(|| hata(StatusCode::BAD_REQUEST, "devredilecek KESİN önceki dönem yok — önce dönemi kesinleştirin"))?;
    // Çift devir kilidi: hedefte bu kaynaktan CF varsa reddet
    if st.ufrs_kayitlar.iter().any(|k| k.donem == hedef && k.tur == "CF"
        && k.devir_kaynak.as_deref().map(|x| x.starts_with(&format!("{}/", kaynak))).unwrap_or(false)) {
        return Err(hata(StatusCode::CONFLICT, &format!("{} → {} devri zaten yapılmış", kaynak, hedef)));
    }
    let kaynaklar: Vec<UfrsKayit> = st.ufrs_kayitlar.iter()
        .filter(|k| k.donem == kaynak && k.durum == "onerildi" && (k.tur == "AJE" || k.tur == "CF"))
        .cloned()
        .collect();
    let mut uretilen: Vec<String> = Vec::new();
    let mut atlanan: Vec<serde_json::Value> = Vec::new();
    let mut yeniden_olcum: Vec<String> = Vec::new();
    let mut sira = st.ufrs_kayitlar.iter().filter(|k| k.no.starts_with("CF-") && k.donem == hedef).count();
    for k in kaynaklar {
        match k.devir.as_str() {
            "TASIMA_YOK" => { atlanan.push(serde_json::json!({ "no": k.no, "neden": "kendini tüketen kayıt (cut-off) — taşınırsa çift sayım" })); }
            "YENIDEN_URET" => { atlanan.push(serde_json::json!({ "no": k.no, "neden": "sınıflama — yeni dönem bakiyesinden yeniden üretilir" })); }
            davranis => {
                sira += 1;
                let no = format!("CF-{:03}", sira);
                let yeniden = davranis == "TERS_CEVIR";
                if yeniden { yeniden_olcum.push(k.kaynak_ws.clone()); }
                let cf = UfrsKayit {
                    no: no.clone(),
                    tur: "CF".into(),
                    standart: k.standart.clone(),
                    kaynak_ws: k.kaynak_ws.clone(),
                    aciklama: format!("Devir (taşı-bırak): {} — {}", k.no, k.aciklama),
                    satirlar: tasi_birak(&k.satirlar),
                    dayanak_tur: "hesaplama".into(),
                    dayanak_ref: format!("Önceki dönem kesin dosyası {} / {}", kaynak, k.no),
                    denetci_notu: if yeniden {
                        format!("Otomatik devir. DİKKAT: {} her dönem yeniden ölçülür — cari dönem hesaplaması zorunlu (eski ölçüm bu CF ile açılışa kuruldu).", k.kaynak_ws)
                    } else {
                        "Otomatik devir kaydı — P/L bacakları 570'e, bilanço bacakları aynen (kümüle ilerleme).".into()
                    },
                    degerleme_yontemi: k.degerleme_yontemi.clone(),
                    degerleme_bazi: format!("Devir — kaynak kayıt {} ({})", k.no, k.degerleme_bazi),
                    durum: "onerildi".into(),
                    hazirlayan: hazirlayan.clone(),
                    tarih: bugun_str(),
                    donem: hedef.clone(),
                    devir: "AYNEN".into(),
                    devir_kaynak: Some(format!("{}/{}", kaynak, k.no)),
                };
                st.ufrs_kayitlar.push(cf);
                uretilen.push(no);
            }
        }
    }
    Ok(Json(serde_json::json!({
        "ok": true, "kaynak": kaynak, "hedef": hedef,
        "uretilen_cf": uretilen, "atlanan": atlanan,
        "yeniden_olcum_gerekli": yeniden_olcum,
    })))
}

// ═══════ UFRS Hesaplama Motoru — parametreli öneri üretimi (görev #26 detay dilimi) ═══════
//
// İlke: hesap BACKEND'de, kuruş i64 + ppm (milyonda bir) oran ölçeğiyle; ara değerler i128.
// Motor kayıt ATMAZ — ara tablo (denetim izi) + önerilen satırlar döner; denetçi formda düzenler,
// dayanak+not ekler, mevcut POST /api/ufrs/kayit ile atar (manuel müdahale katmanı ilkesi).
// TFRS düzeltme bacakları .90 alt kırılımına önerilir (VUK bakiyesi kirlenmez, devir eşleşir).

/// half-away-from-zero yuvarlamalı (pay/payda) — i128 ara değer.
fn yuv(pay: i128, payda: i128) -> i64 {
    if payda == 0 { return 0; }
    let yarim = payda / 2;
    (((pay + if pay >= 0 { yarim } else { -yarim }) / payda)) as i64
}

/// "39,75" / "%39,75" / "3.5" → ppm (39,75% = 397_500). En çok 4 ondalık hane.
fn yuzde_to_ppm(sv: &str) -> Option<i64> {
    let t = sv.trim().trim_start_matches('%').replace('.', ",");
    let mut par = t.splitn(2, ',');
    let tam: i64 = par.next()?.trim().parse().ok()?;
    let frac = par.next().unwrap_or("").trim();
    let frac4: String = format!("{:0<4}", frac).chars().take(4).collect();
    let f: i64 = if frac4.is_empty() { 0 } else { frac4.parse().ok()? };
    Some(tam * 10_000 + f)
}

/// "73.729,87" / "10000" (TL) → kuruş.
fn tl_to_kurus(sv: &str) -> Option<i64> {
    let t = sv.trim().replace('.', "").replace("₺", "");
    let mut par = t.splitn(2, ',');
    let tam: i64 = par.next()?.trim().parse().ok()?;
    let frac = par.next().unwrap_or("0").trim();
    let f2: String = format!("{:0<2}", frac).chars().take(2).collect();
    Some(tam * 100 + f2.parse::<i64>().ok()?)
}

/// Tek vadeli bugünkü değer (iç iskonto, gün bazlı): PV = T·365e6 / (365e6 + ppm·gün)
fn pv_gun(tutar: i64, oran_ppm: i64, gun: i64) -> i64 {
    let pay = tutar as i128 * 365_000_000i128;
    let payda = 365_000_000i128 + oran_ppm as i128 * gun as i128;
    yuv(pay, payda)
}

/// n yıllık bileşik iskonto çarpanı (ppm): c = c·1e6/(1e6+ppm), n kez.
fn bilesik_carpan_ppm(oran_ppm: i64, yil: i64) -> i64 {
    let mut c: i64 = 1_000_000;
    for _ in 0..yil.max(0) {
        c = yuv(c as i128 * 1_000_000, 1_000_000 + oran_ppm as i128);
    }
    c
}

#[derive(Serialize, Default)]
struct HesapSatiri {
    kalem: String,
    detay: String,
    tutar: i64,
    /// Bu değer NEREDEN geliyor — kullanıcı ilkesi: "müdahale edilen bakiye nereden geliyorsa öncesi/sonrası".
    /// "defter" = mizandan (değişmez; düzeltme başka çalışmada) · "parametre" = denetçi girdisi ·
    /// "formul" = hesaplanan (elle ezilirse izli override) · "sonuc" = ara/nihai toplam.
    #[serde(default)]
    kaynak: String,
    /// defter→hesap kodu, parametre→anahtar; izlenebilirlik referansı.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    kaynak_ref: String,
    /// Denetçi bu satırı elle değiştirebilir mi? (parametre/formül evet; defter hayır)
    #[serde(default)]
    mudahale: bool,
}
#[derive(Serialize)]
struct OneriSonuc {
    ara_tablo: Vec<HesapSatiri>,
    uyarilar: Vec<String>,
    onerilen: Option<serde_json::Value>, // {tur, satirlar[{hesap,borc,alacak,serbest}], aciklama, dayanak_tur, not_taslagi}
}

#[derive(Deserialize)]
struct HesaplaGirdi { parametreler: HashMap<String, String> }

/// POST /api/ufrs/calisma/:id/hesapla — defterden otomatik veri çek + parametrelerle hesapla → öneri.
async fn ufrs_hesapla(
    State(state): State<Shared>,
    Path(id): Path<String>,
    Json(g): Json<HesaplaGirdi>,
) -> Result<Json<OneriSonuc>, (StatusCode, Json<HataDto>)> {
    let hata = |m: String| (StatusCode::BAD_REQUEST, Json(HataDto { hata: m }));
    let st = state.lock().unwrap();
    let d = defter_kur(&st.fisler);
    let pget = |k: &str| g.parametreler.get(k).map(|x| x.as_str()).unwrap_or("");
    let ppm = |k: &str, vars: i64| yuzde_to_ppm(pget(k)).unwrap_or(vars);
    let kurus = |k: &str, vars: i64| tl_to_kurus(pget(k)).unwrap_or(vars);
    let tam = |k: &str, vars: i64| pget(k).trim().parse::<i64>().unwrap_or(vars);
    // yaprak net bakiyeleri (borç +)
    let netler: Vec<(String, String, i64)> = d.mizan().into_iter().map(|m| {
        let ad = st.hesaplar.get(&m.hesap_id).map(|h| h.ad.clone()).unwrap_or_default();
        (m.hesap_id, ad, m.borc_bakiye.deger() - m.alacak_bakiye.deger())
    }).collect();
    let onekli = |onek: &str| -> Vec<&(String, String, i64)> {
        netler.iter().filter(|(k, _, n)| *n != 0 && onek_altinda(k, onek)).collect()
    };
    let toplam = |onek: &str| -> i64 { onekli(onek).iter().map(|(_, _, n)| *n).sum() };
    let mut ara: Vec<HesapSatiri> = Vec::new();
    let mut uyarilar: Vec<String> = Vec::new();
    let sat = |hesap: &str, borc: i64, alacak: i64| serde_json::json!({ "hesap": hesap, "borc": borc, "alacak": alacak, "serbest": false });
    let oneri = |tur: &str, satirlar: Vec<serde_json::Value>, aciklama: String, dayanak: &str, not: String,
                 yontem: &str, baz: String| {
        Some(serde_json::json!({ "tur": tur, "satirlar": satirlar, "aciklama": aciklama, "dayanak_tur": dayanak,
                                 "not_taslagi": not, "degerleme_yontemi": yontem, "degerleme_bazi_taslagi": baz }))
    };

    // Köken helper'ları — her ara satır nereden geldiğini taşır (izlenebilirlik)
    let hs_defter = |kalem: &str, kod: &str, tutar: i64| HesapSatiri {
        kalem: kalem.into(), detay: format!("defter · {}", kod), tutar,
        kaynak: "defter".into(), kaynak_ref: kod.into(), mudahale: false,
    };
    let hs_param = |kalem: &str, anahtar: &str, tutar: i64| HesapSatiri {
        kalem: kalem.into(), detay: format!("parametre · {}", anahtar), tutar,
        kaynak: "parametre".into(), kaynak_ref: anahtar.into(), mudahale: true,
    };
    let hs_formul = |kalem: &str, detay: &str, tutar: i64| HesapSatiri {
        kalem: kalem.into(), detay: detay.into(), tutar,
        kaynak: "formul".into(), kaynak_ref: String::new(), mudahale: false,
    };
    let hs_sonuc = |kalem: &str, detay: &str, tutar: i64| HesapSatiri {
        kalem: kalem.into(), detay: detay.into(), tutar,
        kaynak: "sonuc".into(), kaynak_ref: String::new(), mudahale: false,
    };

    let onerilen = match id.as_str() {
        // ── TMS 1 Sınıflama (Reclass): UV kredinin KV kısmı — tutar üretmez, TAŞIR ──
        // Kullanıcı ilkesi somut örnek: her ara değer kökenli — 400 bakiyesi DEFTER'den (değişmez),
        // KV taksit PARAMETRE (denetçi itfa planından girer, müdahale edilebilir), fark FORMÜL.
        "WS-RECLASS" => {
            let uv_kredi = -toplam("400").min(0); // 400 pasif (alacak bakiye) → pozitif göster
            let kv_taksit = kurus("kv_taksit", 0);
            for (k, ad, n) in onekli("400") {
                ara.push(hs_defter(&format!("{} {}", k, ad), k, -*n));
            }
            ara.push(hs_formul("UV kredi toplamı (bilanço)", "400 net", uv_kredi));
            ara.push(hs_param("12 ay içinde ödenecek anapara", "kv_taksit", kv_taksit));
            let gecerli = kv_taksit.min(uv_kredi).max(0);
            if kv_taksit > uv_kredi {
                uyarilar.push(format!("KV taksit ({}) UV kredi bakiyesinden ({}) büyük — itfa planını kontrol edin.", tl(kv_taksit), tl(uv_kredi)));
            }
            ara.push(hs_sonuc("KV'ye sınıflanacak (kayıt tutarı)", "B 400 / A 303.90", gecerli));
            if gecerli == 0 { uyarilar.push("KV taksit girilmedi — sınıflama kaydı gerekmiyor.".into()); None }
            else {
                oneri("RJE", vec![sat("400", gecerli, 0), sat("303.90", 0, gecerli)],
                    "UV kredinin 12 ay içinde ödenecek kısmı (TMS 1 sunum sınıflaması)".into(), "sozlesme",
                    format!("Kredi itfa planına göre {} TL anapara 12 ay içinde ödenecek — uzun vadeli krediden (400) kısa vadeye (303) sınıflandı. Tutar üretmez, yalnız sunumu düzeltir; kâr/zarar etkisi yoktur.", tl(gecerli)),
                    "maliyet", format!("Kredi sözleşmesi itfa planı — 12 ay içinde vadesi gelen anapara {} TL.", tl(gecerli)))
            }
        }
        // ── TFRS 9 Reeskont: alt hesap bazlı PV; alacaklar 657.90/122.90, borçlar 322.90/647.90 ──
        "WS-REESKONT" => {
            let oran = ppm("reeskont_orani", 397_500);
            let gun = tam("vade_gun", 60);
            let esik = kurus("esik_tutar", 1_000_000);
            let mut re_alacak = 0i64;
            for (k, ad, n) in onekli("12") {
                if *n >= esik && !k.starts_with("122") && !k.starts_with("128") && !k.starts_with("129") {
                    let r = *n - pv_gun(*n, oran, gun);
                    re_alacak += r;
                    ara.push(HesapSatiri { kalem: format!("{} {}", k, ad), detay: format!("bakiye {} · vade {}g · reeskont", tl(*n), gun), tutar: r , ..Default::default() });
                }
            }
            let mut re_borc = 0i64;
            for (k, ad, n) in onekli("32") {
                if -*n >= esik && !k.starts_with("322") {
                    let b = -*n;
                    let r = b - pv_gun(b, oran, gun);
                    re_borc += r;
                    ara.push(HesapSatiri { kalem: format!("{} {}", k, ad), detay: format!("bakiye ({}) · vade {}g · reeskont", tl(b), gun), tutar: r , ..Default::default() });
                }
            }
            ara.push(HesapSatiri { kalem: "TOPLAM alacak reeskontu (gider)".into(), detay: "657.90 / 122.90".into(), tutar: re_alacak , ..Default::default() });
            ara.push(HesapSatiri { kalem: "TOPLAM borç reeskontu (gelir)".into(), detay: "322.90 / 647.90".into(), tutar: re_borc , ..Default::default() });
            if re_alacak == 0 && re_borc == 0 { uyarilar.push("Eşik üstü vadeli bakiye yok — kayıt gerekmiyor.".into()); None }
            else {
                let mut ss = Vec::new();
                if re_alacak > 0 { ss.push(sat("657.90", re_alacak, 0)); ss.push(sat("122.90", 0, re_alacak)); }
                if re_borc > 0 { ss.push(sat("322.90", re_borc, 0)); ss.push(sat("647.90", 0, re_borc)); }
                oneri("AJE", ss,
                    format!("Reeskont — etkin faiz %{} vade {}g (TFRS 9 itfa edilmiş maliyet)", pget("reeskont_orani"), gun),
                    "hesaplama",
                    format!("Ticari alacak/borçlar {} gün ortalama vade, %{} iskonto ile bugünkü değere indirildi. Alacak reeskontu {} TL gider, borç reeskontu {} TL gelir. Alt hesap dökümü hesap tablosunda.", gun, pget("reeskont_orani"), tl(re_alacak), tl(re_borc)),
                    "ic_iskonto",
                    format!("TCMB avans faiz oranı %{} (RG Tem 2026) ve alt hesap bazında {} gün ortalama kalan vade; iç iskonto formülü PV = T·365/(365 + oran·gün). Vade kaynağı: senetlerde senet vadesi, açık hesapta ortalama tahsilat/ödeme süresi.", pget("reeskont_orani"), gun))
            }
        }
        // ── TFRS 9 ECL: kova matrisi × temerrüt oranları + şahıs bazlı 128 ──
        "WS-ECL" => {
            let carpan = ppm("ileriye_donuk_carpan", 1_000_000).max(1);
            let kovalar: Vec<i64> = pget("kova_dagilimi").split('/').filter_map(|x| yuzde_to_ppm(x)).collect();
            let oranlar: Vec<i64> = pget("temerrut_oranlari").split('/').filter_map(|x| yuzde_to_ppm(x)).collect();
            if kovalar.len() != 5 || oranlar.len() != 5 { return Err(hata("kova dağılımı ve temerrüt oranları 5'er değer olmalı (a / b / c / d / e)".into())); }
            if kovalar.iter().sum::<i64>() != 1_000_000 { uyarilar.push(format!("Kova dağılımı %100 etmiyor (Σ={} ppm) — son kova farkı yutar.", kovalar.iter().sum::<i64>())); }
            let brut: i64 = toplam("120").max(0) + toplam("121").max(0);
            let k128 = toplam("128").max(0);
            let k129 = -toplam("129").min(0);
            let adlar = ["0-30", "31-90", "91-180", "181-360", "360+"];
            let mut dagitilan = 0i64; let mut ecl = 0i64;
            for i in 0..5 {
                let kt = if i < 4 { yuv(brut as i128 * kovalar[i] as i128, 1_000_000) } else { brut - dagitilan };
                dagitilan += if i < 4 { kt } else { 0 };
                let e = yuv(kt as i128 * oranlar[i] as i128 * carpan as i128, 1_000_000_000_000);
                ecl += e;
                ara.push(HesapSatiri { kalem: format!("Kova {} gün", adlar[i]), detay: format!("tutar {} × oran %{} ", tl(kt), (oranlar[i] as f64 / 10_000.0)), tutar: e , ..Default::default() });
            }
            let sahis = if pget("sahis_bazli_128") != "hayır" { (k128 - k129).max(0) } else { 0 };
            if sahis > 0 { ara.push(HesapSatiri { kalem: "Şahıs bazlı (128 − 129)".into(), detay: "şüpheli alacak %100".into(), tutar: sahis , ..Default::default() }); }
            let toplam_ecl = ecl + sahis;
            let mevcut90 = -toplam("129.90").min(0);
            let fark = toplam_ecl - k129 - mevcut90;
            ara.push(HesapSatiri { kalem: "Beklenen kredi zararı TOPLAM".into(), detay: format!("brüt {} üzerinden", tl(brut)), tutar: toplam_ecl , ..Default::default() });
            ara.push(HesapSatiri { kalem: "Mevcut karşılık (129 + 129.90)".into(), detay: "".into(), tutar: k129 + mevcut90 , ..Default::default() });
            ara.push(HesapSatiri { kalem: "FARK (kayıt tutarı)".into(), detay: "".into(), tutar: fark , ..Default::default() });
            if fark == 0 { uyarilar.push("Fark yok — bu dönem kayıt gerekmiyor.".into()); None }
            else if fark > 0 {
                oneri("AJE", vec![sat("654.90", fark, 0), sat("129.90", 0, fark)],
                    "Beklenen kredi zararı karşılığı (TFRS 9 basitleştirilmiş yaklaşım)".into(), "hesaplama",
                    format!("Karşılık matrisi: brüt {} TL, kova dağılımı {}, temerrüt oranları {}. ECL {} − mevcut karşılık {} = fark {} TL.", tl(brut), pget("kova_dagilimi"), pget("temerrut_oranlari"), tl(toplam_ecl), tl(k129 + mevcut90), tl(fark)),
                    "ecl_matris",
                    format!("Yaşlandırma kovaları {} (gün: 0-30/31-90/91-180/181-360/360+) × temerrüt oranları {} × ileriye dönük çarpan %{}. Oranların kalibrasyon kaynağı: geçmiş dönem tahsilat/yazma verisi — denetçi teyidi gerekir.", pget("kova_dagilimi"), pget("temerrut_oranlari"), pget("ileriye_donuk_carpan")))
            } else {
                oneri("AJE", vec![sat("129.90", -fark, 0), sat("644.90", 0, -fark)],
                    "Konusu kalmayan ECL karşılığı (TFRS 9)".into(), "hesaplama",
                    format!("ECL {} < mevcut karşılık {} — {} TL karşılık iptali.", tl(toplam_ecl), tl(k129 + mevcut90), tl(-fark)),
                    "ecl_matris",
                    format!("Karşılık matrisi yeniden ölçümü: kova dağılımı {} × temerrüt oranları {}; hesaplanan ECL mevcut karşılığın altında kaldı.", pget("kova_dagilimi"), pget("temerrut_oranlari")))
            }
        }
        // ── TMS 19 Kıdem: portföy-ortalaması basit modeli ──
        "WS-KIDEM" => {
            let kisi = tam("kisi_sayisi", 0);
            if kisi <= 0 { return Err(hata("çalışan sayısı girilmeli (SGK bildirgesinden)".into())); }
            let brut = kurus("ort_aylik_brut_ucret", 0);
            if brut <= 0 { return Err(hata("ortalama giydirilmiş brüt ücret girilmeli".into())); }
            let tavan = kurus("kidem_tavani", 7_372_987);
            let kidem_x10 = tam("ort_kidem_yil_x10", 50);
            let iskonto = ppm("reel_iskonto_orani", 35_000);
            let yil = tam("ort_kalan_sure_yil", 10);
            let hakedis = ppm("hak_edis_olasiligi", 950_000);
            let kisa_pay = ppm("kisa_vade_payi", 100_000);
            let birim = brut.min(tavan);
            let kazanilmis = yuv(kisi as i128 * birim as i128 * kidem_x10 as i128, 10);
            let c = bilesik_carpan_ppm(iskonto, yil);
            let karsilik = yuv(kazanilmis as i128 * hakedis as i128 * c as i128, 1_000_000_000_000);
            let vuk = -(toplam("372") + toplam("472")).min(0);
            let fark = karsilik - vuk;
            let kisa = yuv(fark.max(0) as i128 * kisa_pay as i128, 1_000_000);
            let uzun = fark.max(0) - kisa;
            ara.push(HesapSatiri { kalem: "Birim yük (min(brüt, tavan))".into(), detay: format!("tavan {}", tl(tavan)), tutar: birim , ..Default::default() });
            ara.push(HesapSatiri { kalem: "Kazanılmış yükümlülük".into(), detay: format!("{} kişi × {} × {} yıl", kisi, tl(birim), kidem_x10 as f64 / 10.0), tutar: kazanilmis , ..Default::default() });
            ara.push(HesapSatiri { kalem: "İskonto çarpanı".into(), detay: format!("%{} × {} yıl → {:.4}", pget("reel_iskonto_orani"), yil, c as f64 / 1e6), tutar: 0 , ..Default::default() });
            ara.push(HesapSatiri { kalem: "TFRS karşılığı (iskontolu × hak ediş)".into(), detay: format!("hak ediş %{}", pget("hak_edis_olasiligi")), tutar: karsilik , ..Default::default() });
            ara.push(HesapSatiri { kalem: "Mevcut VUK karşılığı (372+472)".into(), detay: "".into(), tutar: vuk , ..Default::default() });
            ara.push(HesapSatiri { kalem: "FARK (kayıt tutarı)".into(), detay: format!("kısa {} / uzun {}", tl(kisa), tl(uzun)), tutar: fark , ..Default::default() });
            uyarilar.push("Basit portföy-ortalaması modeli: tam TMS 19 kişi bazlı öngörülen birim kredi ister; aktüeryal fark OCI ayrımı yapılmaz (tamamı K/Z) — sınır denetçi notunda açıklanır.".into());
            if fark == 0 { None }
            else if fark > 0 {
                let mut ss = vec![sat("770.90", fark, 0)];
                if uzun > 0 { ss.push(sat("472.90", 0, uzun)); }
                if kisa > 0 { ss.push(sat("372.90", 0, kisa)); }
                oneri("AJE", ss, "Kıdem tazminatı karşılığı (TMS 19 — basit aktüeryal model)".into(), "aktueryal",
                    format!("{} kişi × min({}, tavan {}) × {} yıl kıdem = {} TL kazanılmış; reel iskonto %{} × {} yıl (çarpan {:.4}) × hak ediş %{} = {} TL karşılık; VUK {} → fark {} TL (kısa {} / uzun {}). Basit model sınırı: OCI ayrımı yok.",
                        kisi, tl(birim), tl(tavan), kidem_x10 as f64 / 10.0, tl(kazanilmis), pget("reel_iskonto_orani"), yil, c as f64 / 1e6, pget("hak_edis_olasiligi"), tl(karsilik), tl(vuk), tl(fark), tl(kisa), tl(uzun)),
                    "aktueryal_basit",
                    format!("Portföy-ortalaması modeli: SGK bildirgesinden {} çalışan, bordrodan ortalama giydirilmiş brüt {} TL (kıdem tavanı {} TL ile sınırlı), ortalama kazanılmış kıdem {} yıl; reel iskonto %{} (nominal − maaş artışı), emekliliğe ortalama {} yıl, hak ediş olasılığı %{}. Kişi bazlı tam aktüerya yerine portföy ortalaması kullanıldı.",
                        kisi, tl(brut), tl(tavan), kidem_x10 as f64 / 10.0, pget("reel_iskonto_orani"), yil, pget("hak_edis_olasiligi")))
            } else {
                oneri("AJE", vec![sat("472.90", -fark, 0), sat("644.90", 0, -fark)],
                    "Konusu kalmayan kıdem karşılığı".into(), "aktueryal",
                    format!("TFRS karşılığı {} < mevcut {} — {} TL iptal.", tl(karsilik), tl(vuk), tl(-fark)),
                    "aktueryal_basit",
                    format!("Portföy-ortalaması yeniden ölçümü ({} kişi, reel iskonto %{}); hesaplanan karşılık mevcut bakiyenin altında kaldı.", kisi, pget("reel_iskonto_orani")))
            }
        }
        // ── TMS 2 NRV: kalem kalem, grup içi netleştirme yok ──
        "WS-NRV" => {
            let nrv = ppm("nrv_orani", 1_000_000);
            let tamamlama = ppm("tamamlama_satis_maliyeti", 50_000);
            let mut kars = 0i64;
            for (k, ad, n) in onekli("15") {
                if *n <= 0 || k.starts_with("158") || k.starts_with("159") { continue; }
                let nrv_net = yuv(*n as i128 * nrv as i128 * (1_000_000 - tamamlama) as i128, 1_000_000_000_000);
                let ki = (*n - nrv_net).max(0);
                kars += ki;
                ara.push(HesapSatiri { kalem: format!("{} {}", k, ad), detay: format!("maliyet {} · NRV(net) {}", tl(*n), tl(nrv_net)), tutar: ki , ..Default::default() });
            }
            let mevcut = -(toplam("158")).min(0);
            let fark = kars - mevcut;
            ara.push(HesapSatiri { kalem: "Gereken karşılık TOPLAM".into(), detay: "kalem kalem (TMS 2.29 — netleştirme yok)".into(), tutar: kars , ..Default::default() });
            ara.push(HesapSatiri { kalem: "Mevcut karşılık (158)".into(), detay: "".into(), tutar: mevcut , ..Default::default() });
            ara.push(HesapSatiri { kalem: "FARK (kayıt tutarı)".into(), detay: "".into(), tutar: fark , ..Default::default() });
            if fark == 0 { uyarilar.push("Fark yok — NRV maliyetin üstünde, kayıt gerekmiyor.".into()); None }
            else if fark > 0 {
                oneri("AJE", vec![sat("654.91", fark, 0), sat("158.90", 0, fark)],
                    "Stok değer düşüklüğü karşılığı (TMS 2 NRV)".into(), "piyasa",
                    format!("NRV oranı %{} − tamamlama/satış %{}: gereken karşılık {} − mevcut {} = {} TL.", pget("nrv_orani"), pget("tamamlama_satis_maliyeti"), tl(kars), tl(mevcut), tl(fark)),
                    "nrv",
                    format!("Net gerçekleşebilir değer (TMS 2.6): tahmini satış fiyatı stok alt hesabı bazında maliyetin %{}'i olarak alındı, tamamlama+satış maliyeti %{} düşüldü. Fiyat kaynağı: güncel fiyat listesi / dönem sonrası gerçekleşen satışlar. Karşılık KALEM KALEM hesaplandı (TMS 2.29 — grup içi netleştirme yok).", pget("nrv_orani"), pget("tamamlama_satis_maliyeti")))
            } else {
                oneri("AJE", vec![sat("158.90", -fark, 0), sat("644.91", 0, -fark)],
                    "Stok değer düşüklüğü iptali (NRV toparlanması)".into(), "piyasa",
                    format!("Gereken {} < mevcut {} — {} TL iptal (en çok önceki karşılık kadar).", tl(kars), tl(mevcut), tl(-fark)),
                    "nrv",
                    format!("NRV yeniden ölçümü (satış fiyatı maliyetin %{}'i, tamamlama %{}): net gerçekleşebilir değer toparlandı, karşılık kısmen iptal edildi.", pget("nrv_orani"), pget("tamamlama_satis_maliyeti")))
            }
        }
        _ => return Err(hata(format!("{} için hesaplama motoru henüz yok — manuel kayıt kullanın (v2'de genişleyecek)", id))),
    };
    Ok(Json(OneriSonuc { ara_tablo: ara, uyarilar, onerilen }))
}

#[derive(Deserialize)]
struct SenaryoGirdi {
    senaryo_kod: String,
    #[serde(default)]
    hedef: String,           // @hedef yerine geçen hesap (üst ya da alt kırılım); sabit-bacaklı senaryoda boş
    tutar: String,           // TL
    #[serde(default)]
    tutar2: String,          // TL — karma (cift_tutar) senaryolarda ikinci tutar (önceki K/Z kısmı / 522 bakiyesi)
    #[serde(default)]
    kv_orani: String,        // "%25" — boşsa ufrs-parametreleri
}

/// POST /api/ufrs/calisma/:id/senaryo — SENARYO seçimiyle TAM kayıt kur (ertelenmiş vergi bacağı dahil).
/// Denetçi hesap aramaz: senaryo standardın çizdiği kayıt şeklini + ertelenmiş vergi kanalını taşır.
/// TMS 12.61A geri izleme: EV kanalı ana kaleme göre (K/Z→691, OCI→522/549); oran döneme göre.
async fn ufrs_senaryo(
    State(state): State<Shared>,
    Path(id): Path<String>,
    Json(g): Json<SenaryoGirdi>,
) -> Result<Json<OneriSonuc>, (StatusCode, Json<HataDto>)> {
    let hata = |m: String| (StatusCode::BAD_REQUEST, Json(HataDto { hata: m }));
    let katalog: serde_json::Value = serde_json::from_str(UFRS_JSON).unwrap_or_default();
    let calisma = katalog["calismalar"].as_array()
        .and_then(|a| a.iter().find(|c| c["id"] == id.as_str()))
        .ok_or_else(|| hata(format!("çalışma yok: {}", id)))?;
    let sen = calisma["senaryolar"].as_array()
        .and_then(|a| a.iter().find(|s| s["kod"] == g.senaryo_kod.as_str()))
        .ok_or_else(|| hata(format!("senaryo yok: {}", g.senaryo_kod)))?;
    let tutar = tl_to_kurus(&g.tutar).filter(|t| *t > 0)
        .ok_or_else(|| hata("geçerli bir tutar girin".into()))?;
    // B8: hedef doğrulaması — senaryo @hedef içeriyorsa hedef gerçek bir TDHP hesabı
    // (veya var olan ananın muavin deseni) olmalı; uydurma kod TDHP sanılamaz.
    let hedef_kullaniliyor = sen["bacaklar"].as_array().map(|a| a.iter().any(|b| b["hesap"] == "@hedef")).unwrap_or(false);
    if hedef_kullaniliyor {
        let hedef = g.hedef.trim();
        if hedef.is_empty() {
            return Err(hata("bu senaryo hedef hesap ister — girdi tablosundan ya da seçiciden hedef seçin".into()));
        }
        if hedef.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
            if hedef.split('.').next().unwrap_or("").len() < 3 {
                return Err(hata(format!("{} bir sınıf/grup kodu — hedef kebir (3 hane) ya da alt kırılım olmalı; üst toplamlar WTB'de kendiliğinden oluşur", hedef)));
            }
            let st = state.lock().unwrap();
            let taninan = st.hesaplar.contains_key(hedef)
                || hedef.split_once('.').map(|(ana, _)| st.hesaplar.contains_key(ana)).unwrap_or(false);
            if !taninan {
                return Err(hata(format!("hedef hesap tanımsız: {} — TDHP'de yok; serbest TFRS kalemi ise harfle başlayan ad kullanın", hedef)));
            }
        }
    }
    // v7: hedef_onekleri — kapsam dışıysa reddetme, UYAR (karar denetçinin).
    let mut kapsam_uyari: Option<String> = None;
    if hedef_kullaniliyor {
        if let Some(onekler) = sen["hedef_onekleri"].as_array().filter(|a| !a.is_empty()) {
            let hedef = g.hedef.trim();
            let uyan = onekler.iter().filter_map(|o| o.as_str()).any(|o| hedef.starts_with(o));
            if !uyan {
                let liste: Vec<&str> = onekler.iter().filter_map(|o| o.as_str()).collect();
                kapsam_uyari = Some(format!(
                    "⚠ hedef {} bu senaryonun tipik kapsamı ({}) dışında — standart gereği kontrol edin; karar denetçinindir.",
                    hedef, liste.join("/")));
            }
        }
    }
    // v7: karma (cift_tutar) senaryolar — TMS 16.39-40 / 36.60-61 simetrisi: ana=tam tutar,
    // ikinci=tutar2 (önceki K/Z kısmı ya da 522 bakiyesi), kalan=tutar−tutar2.
    let cift = sen["cift_tutar"].as_bool().unwrap_or(false);
    let tutar2 = if cift {
        let t2 = tl_to_kurus(&g.tutar2).filter(|t| *t > 0)
            .ok_or_else(|| hata(format!("bu senaryo ikinci tutarı ister: {}", sen["ikinci_tutar_ad"].as_str().unwrap_or("ikinci tutar"))))?;
        if t2 > tutar {
            return Err(hata(format!(
                "ikinci tutar ({}) toplam farkı ({}) aşamaz — TMS 16.39-40/36.117 simetri sınırı: {}",
                tl(t2), tl(tutar), sen["ikinci_tutar_ad"].as_str().unwrap_or(""))));
        }
        t2
    } else { 0 };
    // KV oranı: girdi ya da ufrs-parametreleri (kv_orani)
    let kv_ppm = yuzde_to_ppm(&g.kv_orani).unwrap_or_else(|| {
        serde_json::from_str::<serde_json::Value>(UFRS_PARAM_JSON).ok()
            .and_then(|v| v["parametreler"].as_array()?.iter()
                .find(|p| p["anahtar"] == "kv_orani")?["deger_ppm"].as_i64())
            .unwrap_or(250_000)
    });
    let evh = &katalog["ev_hesaplari"];
    let evv = evh["EVV"].as_str().unwrap_or("283");
    let evy = evh["EVY"].as_str().unwrap_or("483");

    let sat = |hesap: &str, borc: i64, alacak: i64|
        serde_json::json!({ "hesap": hesap, "borc": borc, "alacak": alacak, "serbest": !hesap.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) });
    let mut satirlar: Vec<serde_json::Value> = Vec::new();
    let mut ara: Vec<HesapSatiri> = Vec::new();

    // 1) Temel bacaklar (senaryo şekli; @hedef → seçili hesap).
    // v7: pay alanı — ana=tam tutar, ikinci=tutar2, kalan=tutar−tutar2 (0 ise bacak atlanır).
    // EV bölüşümü için bacak.kanal toplanır (karma senaryolarda EV kaynağın kanalını bacak bazında izler).
    let mut ev_parcalar: Vec<(String, i64)> = Vec::new(); // (kanal, matrah)
    for b in sen["bacaklar"].as_array().unwrap_or(&vec![]) {
        let raw = b["hesap"].as_str().unwrap_or_default();
        let hesap = if raw == "@hedef" { g.hedef.as_str() } else { raw };
        let borc_yon = b["yon"].as_str() == Some("B");
        let bt = match b["pay"].as_str() {
            Some("ikinci") => tutar2,
            Some("kalan") => tutar - tutar2,
            _ => tutar, // "ana" veya pay'sız (basit senaryo)
        };
        if bt == 0 { continue; } // tutar2 == tutar → kalan bacak düşer (örn. azalışın tamamı fondan)
        if let Some(kanal) = b["kanal"].as_str() {
            ev_parcalar.push((kanal.to_string(), bt));
        }
        satirlar.push(sat(hesap, if borc_yon { bt } else { 0 }, if borc_yon { 0 } else { bt }));
        ara.push(HesapSatiri {
            kalem: format!("{} {}", if borc_yon { "B" } else { "A" }, hesap),
            detay: match b["pay"].as_str() {
                Some("ikinci") => "ikinci tutar (simetri kısmı)".into(),
                Some("kalan") => "kalan kısım (tutar − ikinci)".into(),
                _ if raw == "@hedef" => "seçili hedef hesap".into(),
                _ => "standart kayıt bacağı".into(),
            },
            tutar: bt, kaynak: if raw == "@hedef" { "parametre".into() } else { "formul".into() },
            kaynak_ref: hesap.into(), mudahale: raw == "@hedef",
        });
    }

    // 2) Ertelenmiş vergi — TMS 12.61A geri izleme. v7: ev_kanal "yok" → kurulmaz (12.15
    // istisnaları / özkaynak-içi transferler); karma senaryoda kanal bacak bazında bölünür.
    let ev_kanal = sen["ev_kanal"].as_str().unwrap_or("");
    let ev_yon = sen["ev_yon"].as_str().unwrap_or("");
    let mut uyarilar: Vec<String> = Vec::new();
    if let Some(u) = kapsam_uyari { uyarilar.push(u); }
    if !ev_kanal.is_empty() && ev_kanal != "yok" && !ev_yon.is_empty() {
        if ev_parcalar.is_empty() { ev_parcalar.push((ev_kanal.to_string(), tutar)); }
        let kanal_ad = |k: &str| if k == "kz" { "K/Z (691)" } else if k == "oci_522" { "OCI — Yeniden Değerleme (522)" } else { "OCI — Aktüeryal Fon (549)" };
        for (kanal, matrah) in &ev_parcalar {
            let ev_tutar = ((*matrah as i128 * kv_ppm as i128 + 500_000) / 1_000_000) as i64;
            if ev_tutar == 0 { continue; }
            let karsi = match kanal.as_str() {
                "kz" => evh["kz"].as_str().unwrap_or("691"),
                "oci_522" => evh["oci_522"].as_str().unwrap_or("522.90"),
                "oci_549" => evh["oci_549"].as_str().unwrap_or("549.90"),
                _ => "691",
            };
            // EVV (varlık): 283 BORÇ / karşı ALACAK · EVY (yükümlülük): karşı BORÇ / 483 ALACAK
            let (ev_hesap, borc_hesap, alacak_hesap) = if ev_yon == "EVV" {
                (evv, evv, karsi)
            } else {
                (evy, karsi, evy)
            };
            satirlar.push(sat(borc_hesap, ev_tutar, 0));
            satirlar.push(sat(alacak_hesap, 0, ev_tutar));
            ara.push(HesapSatiri {
                kalem: format!("Ertelenmiş vergi ({}) — {}", ev_yon, ev_hesap),
                detay: format!("{} × %{} → kanal: {}", tl(*matrah), (kv_ppm as f64 / 10_000.0), kanal_ad(kanal)),
                tutar: ev_tutar, kaynak: "formul".into(), kaynak_ref: ev_hesap.into(), mudahale: false,
            });
            uyarilar.push(format!("Ertelenmiş vergi otomatik: {} TL, {} kanalında (TMS 12.61A geri izleme). Oran %{} — döneme göre değişebilir.", tl(ev_tutar), kanal_ad(kanal), (kv_ppm as f64 / 10_000.0)));
        }
    } else if ev_kanal == "yok" {
        uyarilar.push("Bu senaryoda ertelenmiş vergi bacağı kurulmaz (TMS 12.15 istisnası / özkaynak-içi transfer / EV'nin kendi kaydı).".into());
    }
    // v7: senaryoya bağlı doğrulama kuralları (V-katalog) bilgisi
    if let Some(kurallar) = sen["kurallar"].as_array().filter(|a| !a.is_empty()) {
        let liste: Vec<&str> = kurallar.iter().filter_map(|k| k.as_str()).collect();
        uyarilar.push(format!("Geçerli standart kuralları: {} (bkz. spiral-standart-haritasi-A1.md).", liste.join(", ")));
    }

    let aciklama = if g.hedef.trim().is_empty() {
        format!("{} — {}", sen["ad"].as_str().unwrap_or(""), calisma["kisa_ad"].as_str().unwrap_or(&id))
    } else {
        format!("{} — {}", sen["ad"].as_str().unwrap_or(""), g.hedef)
    };
    let onerilen = serde_json::json!({
        "tur": sen["tur"].as_str().unwrap_or("AJE"),
        "satirlar": satirlar,
        "aciklama": aciklama,
        "dayanak_tur": calisma["dayanak_onerisi"].as_str().unwrap_or("hesaplama"),
        "not_taslagi": sen["aciklama"].as_str().unwrap_or(""),
        "degerleme_yontemi": calisma["degerleme_yontemleri"].as_array().and_then(|a| a.first()).and_then(|x| x.as_str()).unwrap_or(""),
        "degerleme_bazi_taslagi": format!("{} senaryosu — {} hesabı, {} TL.", sen["ad"].as_str().unwrap_or(""), g.hedef, tl(tutar)),
    });
    Ok(Json(OneriSonuc { ara_tablo: ara, uyarilar, onerilen: Some(onerilen) }))
}

/// tl gösterimi (motor açıklamaları için) — kuruş → "1.234,56"
fn tl(k: i64) -> String {
    let neg = k < 0;
    let a = k.abs();
    let tam = a / 100;
    let kur = a % 100;
    let mut sayi = tam.to_string();
    let mut i = sayi.len() as i64 - 3;
    while i > 0 { sayi.insert(i as usize, '.'); i -= 3; }
    format!("{}{},{:02}", if neg { "-" } else { "" }, sayi, kur)
}

#[cfg(test)]
mod beyanname_testi {
    use super::*;

    fn dolu_state() -> Arc<Mutex<AppState>> {
        let mut st = super::defter_aktarim_testi::bos_state();
        defteri_senkronize(&mut st, &std::collections::HashSet::new());
        let a = Arc::new(Mutex::new(st));
        futures::executor::block_on(muhasebe_aktar(State(a.clone())));
        a
    }
    fn manuel_dene(state: &Arc<Mutex<AppState>>, ths: &str, fatura: &str) -> serde_json::Value {
        futures::executor::block_on(tahsis_manuel(
            State(state.clone()),
            Json(TahsisGirdi { tahsilat_id: ths.to_string(), fatura_no: fatura.to_string() }),
        )).unwrap_or_else(|_| panic!("manuel eşleştirme reddedildi: {ths} → {fatura}")).0
    }
    fn kdv1(state: &Arc<Mutex<AppState>>, ay: u8) -> KdvBeyanname {
        let mut q = HashMap::new();
        q.insert("ay".to_string(), ay.to_string());
        futures::executor::block_on(kdv_beyanname(State(state.clone()), Query(q))).0
    }

    /// KDV1 her ay için oran kırılımı vermeli ve satır KDV'leri toplamı hesaplananı tutmalı.
    /// (Faz 1'de muavin kırılımı eklenmeden önce tek satır %20 dönüyordu — regresyon koruması.)
    #[test]
    fn kdv1_oran_kirilimi_ve_toplam_tutarlilik() {
        let st = dolu_state();
        let mut dolu_ay = 0;
        for ay in 1..=12u8 {
            let b = kdv1(&st, ay);
            if b.hesaplanan_toplam == 0 { continue; }
            dolu_ay += 1;
            let satir_toplam: i64 = b.matrahlar.iter().map(|m| m.hesaplanan).sum();
            let man: i64 = b.manuel.iter().filter(|m| m.yon == "hesaplanan").map(|m| m.kdv).sum();
            assert_eq!(satir_toplam + man, b.hesaplanan_toplam, "{ay}. ay: satır toplamı hesaplananı tutmuyor");
            assert!(b.matrahlar.len() >= 2, "{ay}. ay tek orana çökmüş — muavin kırılımı bozuk");
            // Her satırda KDV = matrah × oran (yuvarlama toleransıyla)
            for m in &b.matrahlar {
                let beklenen = m.matrah * m.oran / 100;
                assert!((beklenen - m.hesaplanan).abs() <= m.matrah / 1000 + 100,
                    "{ay}. ay %{}: matrah×oran ({beklenen}) ile hesaplanan ({}) uyuşmuyor", m.oran, m.hesaplanan);
            }
        }
        assert_eq!(dolu_ay, 12, "12 ayın hepsi beyan üretmeli (LCG dağılım regresyonu)");
    }

    /// İNDİRİM DAĞILIMI: KDV1'in "indirilecek KDV'nin oranlara göre dağılımı" bölümü,
    /// satır toplamı indirilecek KDV'yi tutmalı ve her satırda KDV = bedel × oran olmalı.
    #[test]
    fn kdv1_indirim_dagilimi_toplami_tutar() {
        let st = dolu_state();
        let mut kontrol = 0;
        for ay in 1..=12u8 {
            let b = kdv1(&st, ay);
            if b.bu_donem_indirilecek <= 0 { continue; }
            kontrol += 1;
            let satir: i64 = b.indirim_dagilimi.iter().map(|m| m.hesaplanan).sum();
            assert_eq!(satir, b.bu_donem_indirilecek,
                "{ay}. ay: indirim dağılımı toplamı indirilecek KDV'yi tutmuyor");
            assert!(b.indirim_dagilimi.len() >= 2, "{ay}. ay indirim tek orana çökmüş");
            for m in &b.indirim_dagilimi {
                let beklenen = m.matrah * m.oran / 100;
                assert!((beklenen - m.hesaplanan).abs() <= m.matrah / 1000 + 100,
                    "{ay}. ay %{}: bedel×oran ile KDV uyuşmuyor", m.oran);
            }
        }
        assert_eq!(kontrol, 12, "12 ayın hepsinde indirilecek KDV olmalı");
    }

    /// Ödenecek/devreden ayrımı: ikisi aynı anda pozitif olamaz; fark doğru yöne düşmeli.
    #[test]
    fn kdv1_odenecek_devreden_ayrimi() {
        let st = dolu_state();
        for ay in 1..=12u8 {
            let b = kdv1(&st, ay);
            assert!(b.odenecek == 0 || b.devreden == 0, "{ay}. ay: ödenecek ve devreden aynı anda pozitif");
            let fark = b.hesaplanan_toplam - b.indirimler_toplam;
            assert_eq!(b.odenecek, fark.max(0), "{ay}. ay ödenecek yanlış");
            assert_eq!(b.devreden, (-fark).max(0), "{ay}. ay devreden yanlış");
        }
    }

    /// İSTİSNA teslim (ihracat) matrahı beyan edilir ama hesaplanan KDV'ye EKLENMEZ.
    #[test]
    fn kdv1_istisna_hesaplanana_eklenmez() {
        let st = dolu_state();
        let mut toplam_istisna = 0i64;
        for ay in 1..=12u8 {
            let b = kdv1(&st, ay);
            toplam_istisna += b.istisna.matrah;
            // İstisna matrahından KDV doğmadığı için oran satırlarında karşılığı olmamalı.
            assert!(!b.matrahlar.iter().any(|m| m.oran == 0), "{ay}. ay: %0 oran satırı hesaplanana girmiş");
        }
        assert!(toplam_istisna > 0, "senaryoda ihracat/istisna teslim bulunmalı");
    }

    /// TEVKİFAT: tevkif edilen + beyan edilen = toplam KDV; 391'e yalnız beyan edilen yazılır.
    #[test]
    fn kdv1_tevkifat_bolunmesi_tutarli() {
        let st = dolu_state();
        let mut var = false;
        for ay in 1..=12u8 {
            let t = kdv1(&st, ay).tevkifat;
            if t.belge_sayisi == 0 { continue; }
            var = true;
            assert_eq!(t.tevkif_edilen + t.beyan_edilen, t.toplam_kdv,
                "{ay}. ay: tevkifat bölünmesi toplam KDV'yi tutmuyor");
            assert!(t.tevkif_edilen > 0 && t.beyan_edilen > 0, "{ay}. ay: tevkifat tek tarafa çökmüş");
        }
        assert!(var, "senaryoda tevkifatlı fatura bulunmalı");
    }

    /// MANUEL EŞLEŞTİRME SINIR DURUMLARI — hepsi gerçek hatalardan doğdu.
    ///
    /// En kritiği: hedef fatura BAŞKA bir tahsilatla kapanmışsa, o tahsis DÜŞÜRÜLMELİ ve
    /// maker'ın kararı yazılmalıdır. Önceden sessizce hiçbir şey olmuyordu ama API
    /// "eski eşleşme kaldırıldı" diye YANLIŞ rapor veriyordu.
    #[test]
    fn manuel_esleştirme_sinir_durumlari() {
        let st = dolu_state();
        let d = { let g = st.lock().unwrap(); gorunum(&g) };

        let ths = d.tahsilatlar.iter()
            .find(|x| x.durum == "OTOMATIK" && !x.firma.is_empty() && x.yon == "SATIS")
            .expect("otomatik eşleşmiş tahsilat bulunmalı").clone();
        let ayni_cari: Vec<&simulasyon::FaturaSim> = d.faturalar.iter()
            .filter(|f| f.karsi == ths.firma && f.yon == ths.yon && f.gecerli && f.net_borc > 0).collect();

        // ── 1) KAPALI faturaya eşleştir → oradaki tahsis DÜŞÜRÜLÜR, maker kararı yazılır.
        let kapali = ayni_cari.iter()
            .find(|f| f.kalan == 0 && !f.tahsisler.is_empty()
                   && !f.tahsisler.iter().any(|t| t.tahsilat_id == ths.id))
            .expect("başka tahsilatın kapattığı fatura bulunmalı").no.clone();
        let r = manuel_dene(&st, &ths.id, &kapali);
        assert_ne!(r["sonuc"], "YAZILAMADI", "kapalı faturaya eşleştirme sessizce yok sayıldı");
        assert!(r["hedefe_yazilan"].as_i64().unwrap() > 0, "maker kararı hedefe yazılmadı");
        assert!(!r["dusurulen_tahsisler"].as_array().unwrap().is_empty(),
            "yer açmak için başka tahsis düşürülmeliydi");

        // Düşürülen para kaybolmamalı — her tahsilatın dağıtımı tutarını aşmamalı.
        let sonra = { let g = st.lock().unwrap(); gorunum(&g) };
        let mut dagitim: HashMap<String, i64> = HashMap::new();
        for f in &sonra.faturalar {
            for t in &f.tahsisler { *dagitim.entry(t.tahsilat_id.clone()).or_insert(0) += t.tutar; }
        }
        for t in &sonra.tahsilatlar {
            let d = dagitim.get(&t.id).copied().unwrap_or(0);
            assert!(d <= t.tutar, "{} tutarından fazla dağıtıldı ({d} > {})", t.id, t.tutar);
        }
    }

    /// Tahsilat faturanın borcundan BÜYÜKSE: hedefe borcu kadar yazılır, aşan kısım
    /// aynı carinin diğer açık faturalarına akar. Fatura aşırı tahsil edilmiş görünmemeli.
    #[test]
    fn tahsilat_fatura_borcunu_asamaz() {
        let st = dolu_state();
        let d = { let g = st.lock().unwrap(); gorunum(&g) };
        let ths = d.tahsilatlar.iter()
            .find(|x| x.durum == "OTOMATIK" && !x.firma.is_empty() && x.yon == "SATIS" && x.tutar > 100_00)
            .expect("tahsilat bulunmalı").clone();
        let kucuk = d.faturalar.iter()
            .find(|f| f.karsi == ths.firma && f.yon == ths.yon && f.gecerli
                   && f.net_borc > 0 && f.net_borc < ths.tutar)
            .map(|f| f.no.clone());
        let Some(hedef) = kucuk else { return }; // senaryoda yoksa test atlanır

        let r = manuel_dene(&st, &ths.id, &hedef);
        let yazilan = r["hedefe_yazilan"].as_i64().unwrap();
        let sonra = { let g = st.lock().unwrap(); gorunum(&g) };
        let f = sonra.faturalar.iter().find(|x| x.no == hedef).unwrap();
        assert!(yazilan <= f.net_borc, "faturaya borcundan fazla tahsis edildi");
        assert!(f.kalan >= 0, "fatura aşırı tahsil edilmiş (kalan negatif)");
        assert_eq!(r["sonuc"], "KISMEN_YAZILDI", "aşan tutar açıkça raporlanmalı");
    }

    /// Yanlış cariye eşleştirme REDDEDİLMELİ — yanlış cari yanlış bakiye demektir.
    #[test]
    fn yanlis_cariye_eslestirme_reddedilir() {
        let st = dolu_state();
        let d = { let g = st.lock().unwrap(); gorunum(&g) };
        let ths = d.tahsilatlar.iter()
            .find(|x| !x.firma.is_empty() && x.yon == "SATIS").expect("tahsilat").clone();
        let baska = d.faturalar.iter()
            .find(|f| f.karsi != ths.firma && f.yon == "SATIS" && f.gecerli)
            .expect("başka cariden fatura").no.clone();
        let mut q = HashMap::new(); q.insert("x".to_string(), String::new()); let _ = q;
        let sonuc = futures::executor::block_on(tahsis_manuel(
            State(st.clone()), Json(TahsisGirdi { tahsilat_id: ths.id.clone(), fatura_no: baska })));
        assert!(sonuc.is_err(), "yanlış cariye eşleştirme kabul edildi");
    }

    /// Ba/Bs: hükümsüz belge girmez, tutarlar KDV HARİÇ, haddi aşanlar işaretlenir, VKN dolu olur.
    #[test]
    fn babs_kdv_haric_ve_hukumsuz_belge_haric() {
        let st = dolu_state();
        let mut q = HashMap::new();
        q.insert("ay".to_string(), "6".to_string());
        let r = futures::executor::block_on(hesaplama::babs(State(st.clone()), Query(q))).0;

        assert_eq!(r["yururlukte"], false, "Ba/Bs 25.09.2024'te kaldırıldı — beyanname olarak sunulmamalı");
        let had = r["had"].as_i64().unwrap();
        for yon in ["ba", "bs"] {
            let satirlar = r[yon]["satirlar"].as_array().unwrap();
            assert!(!satirlar.is_empty(), "{yon} listesi boş");
            for x in satirlar {
                let tutar = x["tutar"].as_i64().unwrap();
                assert!(tutar > 0, "{yon}: sıfır/negatif tutarlı satır");
                assert_eq!(x["hadde_girdi"].as_bool().unwrap(), tutar >= had, "{yon}: had işareti yanlış");
                assert!(!x["vkn"].as_str().unwrap().is_empty(), "{yon}: VKN boş — bildirim düzenlenemez");
                assert!(x["belge_sayisi"].as_u64().unwrap() > 0);
            }
        }
        assert!(r["vkn_eksik"].as_array().unwrap().is_empty(), "haddi aşan carilerde VKN eksiği var");

        // Ba/Bs tutarı, o ayın GEÇERLİ faturalarının KDV hariç net matrahını aşamaz.
        let gor = { let g = st.lock().unwrap(); gorunum(&g) };
        let beklenen: i64 = gor.faturalar.iter()
            .filter(|f| f.gecerli && f.yon == "SATIS" && f.tarih[3..5].parse::<u32>().unwrap_or(0) == 6)
            .map(|f| f.matrah - f.iade_matrah - f.iskonto_matrah).filter(|x| *x > 0).sum();
        let bs_hepsi: i64 = r["bs"]["satirlar"].as_array().unwrap().iter()
            .map(|x| x["tutar"].as_i64().unwrap()).sum();
        assert_eq!(bs_hepsi, beklenen, "Bs toplamı KDV hariç net matrahla uyuşmuyor");
    }
}

#[cfg(test)]
mod defter_aktarim_testi {
    use super::*;

    /// Boş defterli bir AppState — aktarım ve storno mantığını izole test etmek için.
    pub fn bos_state() -> AppState {
        // Gerçek TDHP + KDV muavinleri — üretimdeki açılış seed'iyle aynı olmalı ki
        // aktarım testleri "hesap yok / yaprak değil" hatasına düşmesin.
        let mut hesaplar_v = csv_yukle(TDHP);
        for (ana, ad) in [("391", "Hesaplanan KDV"), ("191", "İndirilecek KDV")] {
            let Some(base) = hesaplar_v.iter().find(|h| h.kod == ana).cloned() else { continue };
            for (alt, oran) in hesaplama::kdv_muavin_kodlari() {
                hesaplar_v.push(muavin_olustur(&base, &alt, &format!("{ad} %{oran}")));
            }
            for h in hesaplar_v.iter_mut() { if h.kod == ana { h.yaprak = false; } }
        }
        let mut hm = HashMap::new();
        for h in &hesaplar_v { hm.insert(h.kod.clone(), h.clone()); }
        AppState {
            hesaplar: hm, hesap_sirali: hesaplar_v,
            donem: bos_donem(), sayac: SeriSayac::yeni(), fisler: Vec::new(),
            aciklamalar: HashMap::new(), kurallar: HashMap::new(),
            kumeler: HashMap::new(), kullanici_kumeler: HashMap::new(),
            kullanici_sablonlar: Vec::new(), sablon_kullanim: HashMap::new(),
            manuel_tahsis: HashMap::new(),
        fatura_bekleyen: HashMap::new(), tahsis_defteri: Vec::new(), fis_sayaci: 0,
            aktarilan: std::collections::HashSet::new(),
        gorunum_surum: 0,
        gorunum_onbellek: Mutex::new(None),
            banka: Vec::new(), gelen_belgeler: Vec::new(), faturalar: Vec::new(),
            belgeler: HashMap::new(), fatura_fisleri: HashMap::new(),
            denetim_notlar: HashMap::new(), kagit_duzenlemeler: HashMap::new(),
            vergi_duzenlemeler: HashMap::new(), ufrs_kayitlar: Vec::new(),
            ufrs_kesin_donemler: Vec::new(),
            profiller: vec![MukellefProfil { id: "m1".into(), unvan: "Test".into(), vkn: "1".into(),
                sektor_kodlari: vec!["TIC".into()], maliyet_secenegi: "yok".into() }],
            aktif: "m1".into(), arsiv: HashMap::new(), kullanicilar: Vec::new(), oturumlar: HashMap::new(),
        }
    }

    /// Senkronizasyon idempotent olmalı: ikinci çağrı MÜKERRER fiş üretmez (doğal anahtar korur).
    #[test]
    fn senkronizasyon_mukerrer_kayit_uretmez() {
        let mut st = bos_state();
        let bos = std::collections::HashSet::new();
        let (ilk, storno1) = defteri_senkronize(&mut st, &bos);
        assert!(ilk > 0, "ilk senkronizasyon fiş üretmeliydi");
        assert_eq!(storno1, 0, "ilk çalıştırmada storno olmamalı");

        let (ikinci, storno2) = defteri_senkronize(&mut st, &bos);
        assert_eq!(ikinci, 0, "ikinci çağrı {ikinci} mükerrer fiş üretti");
        assert_eq!(storno2, 0, "ikinci çağrı gereksiz storno üretti");

        // Doğal anahtar (tahsilat|fatura|tutar) tekil olmalı.
        let mut anahtarlar: Vec<&String> = st.tahsis_defteri.iter()
            .filter(|k| k.durum == "AKTIF" && !k.storno).map(|k| &k.anahtar).collect();
        let toplam = anahtarlar.len();
        anahtarlar.sort();
        anahtarlar.dedup();
        assert_eq!(anahtarlar.len(), toplam, "aktif kayıtlarda mükerrer anahtar var");
    }

    /// Tahsis defteri her zaman dengeli olmalı — storno dahil Σborç = Σalacak.
    #[test]
    fn tahsis_defteri_dengeli() {
        let mut st = bos_state();
        defteri_senkronize(&mut st, &std::collections::HashSet::new());
        let borc: i64 = st.tahsis_defteri.iter().flat_map(|k| &k.satirlar).map(|s| s.borc).sum();
        let alacak: i64 = st.tahsis_defteri.iter().flat_map(|k| &k.satirlar).map(|s| s.alacak).sum();
        assert_eq!(borc, alacak, "tahsis defteri dengesiz");
        for k in &st.tahsis_defteri {
            let b: i64 = k.satirlar.iter().map(|s| s.borc).sum();
            let a: i64 = k.satirlar.iter().map(|s| s.alacak).sum();
            assert_eq!(b, a, "{} kaydı tek başına dengesiz", k.fis_no);
        }
    }

    /// İPTAL: eşleşme değişince asıl kayıt SİLİNMEZ, IPTAL işaretlenir ve karşısına
    /// borç/alacağı yer değiştirmiş STORNO açılır (VUK 219 / TTK 65). Net etki sıfır.
    #[test]
    fn eslesme_degisince_storno_acilir_kayit_silinmez() {
        let mut st = bos_state();
        defteri_senkronize(&mut st, &std::collections::HashSet::new());

        // Otomatik eşleşmiş bir tahsisi seç, aynı carinin BAŞKA faturasına elle bağla.
        let k = st.tahsis_defteri.iter().find(|k| k.durum == "AKTIF" && !k.storno)
            .expect("aktif kayıt bulunmalı").clone();
        let d = gorunum(&st);
        let baska = d.faturalar.iter()
            .find(|f| f.karsi == k.firma && f.yon == k.yon && f.no != k.fatura_no && f.gecerli && f.net_borc > 0)
            .expect("aynı caride başka geçerli fatura bulunmalı").no.clone();

        let onceki_satir = st.tahsis_defteri.len();
        st.manuel_tahsis.insert(k.tahsilat_id.clone(), baska.clone());
        let serbest: std::collections::HashSet<String> = [k.tahsilat_id.clone()].into_iter().collect();
        let (yeni, storno) = defteri_senkronize(&mut st, &serbest);

        assert!(storno > 0, "eşleşme değişti ama storno üretilmedi");
        assert!(yeni > 0, "yeni eşleşme fişi üretilmedi");
        assert!(st.tahsis_defteri.len() > onceki_satir, "defter satır sayısı artmalı — kayıt silinmemeli");

        // Asıl kayıt hâlâ defterde, IPTAL işaretli ve storno'suna bağlı.
        let asil = st.tahsis_defteri.iter().find(|x| x.fis_no == k.fis_no)
            .expect("asıl kayıt SİLİNMİŞ — VUK 219 ihlali");
        assert_eq!(asil.durum, "IPTAL", "asıl kayıt iptal işaretlenmemiş");
        let storno_no = asil.karsi_fis.clone().expect("asıl kayıt storno fişine bağlanmamış");

        // Storno gerçekten ters kayıt mı, net etkisi sıfır mı?
        let s = st.tahsis_defteri.iter().find(|x| x.fis_no == storno_no).expect("storno fişi yok");
        assert!(s.storno && s.durum == "AKTIF");
        for (a, b) in asil.satirlar.iter().zip(s.satirlar.iter()) {
            assert_eq!(a.hesap, b.hesap, "storno farklı hesaba yazılmış");
            assert_eq!(a.borc, b.alacak, "storno borç/alacak yer değiştirmemiş");
            assert_eq!(a.alacak, b.borc, "storno borç/alacak yer değiştirmemiş");
        }
    }

    /// Gerçek deftere aktarım: idempotent olmalı ve yevmiye dengesini bozmamalı.
    #[test]
    fn deftere_aktarim_idempotent_ve_dengeli() {
        let mut st = bos_state();
        defteri_senkronize(&mut st, &std::collections::HashSet::new());

        let sayac_once = st.fisler.len();
        let d = gorunum(&st);
        let _ = d;
        // aktarımı iki kez çalıştır — ikincisi hiç fiş eklememeli
        let state = Arc::new(Mutex::new(st));
        let r1 = futures::executor::block_on(muhasebe_aktar(State(state.clone()))).0;
        let r2 = futures::executor::block_on(muhasebe_aktar(State(state.clone()))).0;

        assert!(r1["aktarilan_fis"].as_u64().unwrap() > 0, "ilk aktarım fiş yazmalıydı");
        assert_eq!(r2["aktarilan_fis"].as_u64().unwrap(), 0, "ikinci aktarım mükerrer fiş yazdı");
        assert_eq!(r1["denge_ok"], true, "yevmiye dengesi bozuldu");
        assert_eq!(r2["denge_ok"], true);

        let st = state.lock().unwrap();
        assert!(st.fisler.len() > sayac_once);
        // Her fiş tek başına dengeli mi?
        for f in &st.fisler {
            let b: i64 = f.satirlar.iter().map(|s| s.borc.deger()).sum();
            let a: i64 = f.satirlar.iter().map(|s| s.alacak.deger()).sum();
            assert_eq!(b, a, "{:?} fişi dengesiz", fis_no(f));
        }
    }

    /// FİŞ TİPİ (VUK şekil şartı): kasa/banka girişi Tahsil, çıkışı Tediye, gerisi Mahsup.
    #[test]
    fn fis_tipleri_dogru_secilir() {
        let mut st = bos_state();
        defteri_senkronize(&mut st, &std::collections::HashSet::new());
        let state = Arc::new(Mutex::new(st));
        futures::executor::block_on(muhasebe_aktar(State(state.clone())));
        let st = state.lock().unwrap();

        let kasa_banka = |k: &str| k.starts_with("100") || k.starts_with("101") || k.starts_with("102");
        let mut tahsil = 0; let mut tediye = 0;
        for f in &st.fisler {
            let giris = f.satirlar.iter().any(|s| kasa_banka(&s.hesap_id) && s.borc.deger() > 0);
            let cikis = f.satirlar.iter().any(|s| kasa_banka(&s.hesap_id) && s.alacak.deger() > 0);
            match f.tip {
                FisTipi::Tahsil => { tahsil += 1;
                    assert!(giris && !cikis, "Tahsil fişinde kasa/banka girişi olmalı: {:?}", fis_no(f)); }
                FisTipi::Tediye => { tediye += 1;
                    assert!(cikis && !giris, "Tediye fişinde kasa/banka çıkışı olmalı: {:?}", fis_no(f)); }
                // Açılış/Kapanış fişleri tek yönlü olabilir (dönem başı bakiye girişi) — kural dışı.
                FisTipi::Acilis | FisTipi::Kapanis => {}
                _ => assert!(!(giris ^ cikis), "Mahsup fişi tek yönlü para hareketi taşıyor: {:?}", fis_no(f)),
            }
        }
        assert!(tahsil > 0 && tediye > 0, "Tahsil ve Tediye fişi üretilmeli (tahsil {tahsil}, tediye {tediye})");
    }
}

#[cfg(test)]
mod ufrs_test {
    use super::*;

    fn sat(h: &str, b: i64, a: i64) -> UfrsKayitSatir {
        UfrsKayitSatir { hesap: h.into(), borc: b, alacak: a, serbest: false }
    }

    #[test]
    fn tasi_birak_pl_bacaklari_570e_bilanco_aynen() {
        // Kıdem AJE'si: 770 gider (B) / 472 karşılık (A) → devirde 570 (B) / 472 (A)
        let cf = tasi_birak(&[sat("770", 850_000, 0), sat("472", 0, 850_000)]);
        assert_eq!(cf[0].hesap, "570");
        assert_eq!(cf[0].borc, 850_000);
        assert_eq!(cf[1].hesap, "472");
        assert_eq!(cf[1].alacak, 850_000);
        // denge korunur
        let b: i64 = cf.iter().map(|s| s.borc).sum();
        let a: i64 = cf.iter().map(|s| s.alacak).sum();
        assert_eq!(b, a);
    }

    #[test]
    fn tasi_birak_6li_hesap_da_570e() {
        // GUD azalışı: 659 (B) / 25x (A) → 570 (B) / 25x (A); 646 geliri (A) → 570 (A)
        let cf = tasi_birak(&[sat("659", 10_000, 0), sat("253", 0, 10_000)]);
        assert_eq!(cf[0].hesap, "570");
        let cf2 = tasi_birak(&[sat("120", 5_000, 0), sat("646", 0, 5_000)]);
        assert_eq!(cf2[0].hesap, "120", "bilanço bacağı aynen kalmalı");
        assert_eq!(cf2[1].hesap, "570");
    }

    #[test]
    fn pv_ve_ppm_donusumleri() {
        assert_eq!(yuzde_to_ppm("39,75"), Some(397_500));
        assert_eq!(yuzde_to_ppm("3.5"), Some(35_000));
        assert_eq!(yuzde_to_ppm("%100"), Some(1_000_000));
        assert_eq!(tl_to_kurus("73.729,87"), Some(7_372_987));
        assert_eq!(tl_to_kurus("10.000"), Some(1_000_000));
        // PV: 100.000 TL, %39,75, 60 gün → ~93.867 TL (iç iskonto)
        let pv = pv_gun(10_000_000, 397_500, 60);
        assert!(pv > 9_380_000 && pv < 9_390_000, "pv={}", pv);
        // vade 0 → PV = tutar
        assert_eq!(pv_gun(10_000_000, 397_500, 0), 10_000_000);
    }

    #[test]
    fn bilesik_carpan_monoton_azalir() {
        let c1 = bilesik_carpan_ppm(35_000, 1);
        let c10 = bilesik_carpan_ppm(35_000, 10);
        assert!(c1 < 1_000_000 && c10 < c1, "c1={} c10={}", c1, c10);
        // %3,5 × 10 yıl ≈ 0,7089
        assert!(c10 > 700_000 && c10 < 720_000, "c10={}", c10);
    }

    #[test]
    fn tl_formati() {
        assert_eq!(tl(123_456_789), "1.234.567,89");
        assert_eq!(tl(-5000), "-50,00");
    }

    #[test]
    fn tasi_birak_tfrs_kalemi_aynen() {
        // TDHP-dışı TFRS kalemi (Kullanım Hakkı Varlığı) bilanço kalemidir — aynen taşınır
        let cf = tasi_birak(&[sat("Kullanım Hakkı Varlığı", 250_000, 0), sat("770", 0, 250_000)]);
        assert_eq!(cf[0].hesap, "Kullanım Hakkı Varlığı");
        assert_eq!(cf[1].hesap, "570");
    }
}


#[tokio::main]
async fn main() {
    let hesap_sirali = csv_yukle(TDHP);
    let hesap_map: HashMap<HesapId, Hesap> =
        hesap_sirali.iter().map(|h| (h.id.clone(), h.clone())).collect();

    let aciklamalar: HashMap<String, AciklamaRow> = serde_json::from_str::<AciklamaFile>(ACIKLAMALAR_JSON)
        .map(|f| f.hesaplar.into_iter().map(|r| (r.kod.clone(), r)).collect())
        .unwrap_or_default();
    let kurallar: HashMap<String, KuralRow> = serde_json::from_str::<KuralFile>(KURALLAR_JSON)
        .map(|f| f.hesaplar.into_iter().map(|r| (r.kod.clone(), r)).collect())
        .unwrap_or_default();

    let state = Arc::new(Mutex::new(AppState {
        hesaplar: hesap_map,
        hesap_sirali,
        aciklamalar,
        kurallar,
        kumeler: {
            // Elle derlenmiş kümeler + kural verisinden (borç/alacak koşulları) otomatik türetme.
            // Böylece HER hesabın (83 kurallı + kalanlar hesap adıyla UI'da) önerisi olur.
            let mut m = serde_json::from_str::<KumeFile>(KUMELER_JSON).map(|f| f.kumeler).unwrap_or_default();
            if let Ok(kf) = serde_json::from_str::<KuralFile>(KURALLAR_JSON) {
                for r in kf.hesaplar {
                    let e = m.entry(r.kod.clone()).or_default();
                    for t in [&r.borc, &r.alacak] {
                        // parantezli uzun koşulları kısalt: ilk parantez/virgüle kadar al
                        let kisa = t.split(['(', ',', ';']).next().unwrap_or("").trim().to_string();
                        if kisa.len() >= 4 && kisa.len() <= 60 && !e.iter().any(|x| x == &kisa) {
                            e.push(kisa);
                        }
                    }
                }
            }
            m
        },
        kullanici_kumeler: HashMap::new(),
        kullanici_sablonlar: Vec::new(),
        sablon_kullanim: HashMap::new(),
        manuel_tahsis: HashMap::new(),
        fatura_bekleyen: HashMap::new(),
        tahsis_defteri: Vec::new(),
        fis_sayaci: 0,
        aktarilan: std::collections::HashSet::new(),
        gorunum_surum: 0,
        gorunum_onbellek: Mutex::new(None),
        banka: Vec::new(),
        gelen_belgeler: Vec::new(),
        faturalar: Vec::new(),
        belgeler: HashMap::new(),
        fatura_fisleri: HashMap::new(),
        denetim_notlar: HashMap::new(),
        kagit_duzenlemeler: HashMap::new(),
        vergi_duzenlemeler: HashMap::new(),
        ufrs_kayitlar: Vec::new(),
        ufrs_kesin_donemler: Vec::new(),
        donem: bos_donem(),
        sayac: SeriSayac::yeni(),
        fisler: Vec::new(),
        profiller: vec![MukellefProfil {
            id: "m1".into(), unvan: "Örnek Mükellef A.Ş.".into(), vkn: "1234567890".into(),
            sektor_kodlari: vec!["ETIC".into()], maliyet_secenegi: "yok".into(),
        }],
        aktif: "m1".into(),
        arsiv: HashMap::new(),
        kullanicilar: vec![{
            let salt = "s0".to_string();
            Kullanici {
                id: "u1".into(), kullanici_adi: "admin".into(), ad: "Firma Yöneticisi".into(),
                rol: "admin".into(), mukellef_idleri: vec![],
                departman: "YONETIM".into(), kademe: "YONETICI".into(),
                sifre_ozet: sifre_ozet(&salt, "audit2026"), salt,
            }
        }],
        oturumlar: HashMap::new(),
    }));

    // KDV MUAVİNLERİ — 391.20, 191.10 … Oran kırılımı defterde taşınmazsa KDV1 beyannamesi
    // matrahı tek orandan geri hesaplar ve YANLIŞ beyan üretir (bkz. docs/analiz/09).
    // Kodlar parametre dosyasından okunur; yeni bir oran eklenince burası kendiliğinden açar.
    {
        let mut st = state.lock().unwrap();
        for (ana, ad) in [("391", "Hesaplanan KDV"), ("191", "İndirilecek KDV")] {
            let Some(base) = st.hesaplar.get(ana).cloned() else { continue };
            for o in hesaplama::kdv_muavin_kodlari() {
                let m = muavin_olustur(&base, &o.0, &format!("{ad} %{}", o.1));
                if st.hesaplar.contains_key(&m.kod) { continue; }
                st.hesaplar.insert(m.kod.clone(), m.clone());
                st.hesap_sirali.push(m);
            }
            // Ana hesap artık yaprak değil — kayıt yalnız muavine yapılabilir.
            if let Some(h) = st.hesaplar.get_mut(ana) { h.yaprak = false; }
            for h in st.hesap_sirali.iter_mut() { if h.kod == ana { h.yaprak = false; } }
        }
        st.hesap_sirali.sort_by(|a, b| a.kod.cmp(&b.kod));
    }

    let app = Router::new()
        .route("/api/giris", post(giris))
        .route("/api/cikis", post(cikis))
        .route("/api/oturum", get(oturum))
        .route("/api/kullanicilar", get(kullanicilar))
        .route("/api/kullanici", post(kullanici_ekle))
        .route("/api/sektorler", get(sektorler))
        .route("/api/sablonlar", get(sablonlar))
        .route("/api/sablon", post(sablon_ekle))
        .route("/api/sablon/:ad", delete(sablon_sil))
        .route("/api/sablon/:ad/kullan", post(sablon_kullan))
        .route("/api/departmanlar", get(departmanlar))
        .route("/api/kademeler", get(kademeler))
        .route("/api/mukellefler", get(mukellefler))
        .route("/api/mukellef", post(mukellef_ekle))
        .route("/api/mukellef/:id/aktif", post(mukellef_aktif))
        .route("/api/hesaplar", get(hesaplar))
        .route("/api/hesap-plani", get(hesap_plani))
        .route("/api/hesap/muavin", post(muavin_ekle))
        .route("/api/hesap/:kod/aciklama", get(hesap_aciklama))
        .route("/api/hesap/:kod/kume", get(aciklama_kume))
        .route("/api/kume", post(aciklama_kume_ekle))
        .route("/api/fis", post(fis_ekle))
        .route("/api/fisler", get(fisler))
        .route("/api/fis/:id", get(fis_detay))
        .route("/api/fis/:id/iptal", post(fis_iptal))
        .route("/api/mizan", get(mizan))
        .route("/api/yevmiye", get(yevmiye))
        .route("/api/kebir/:kod", get(kebir))
        .route("/api/muavin", get(muavin))
        .route("/api/muavin/hareket/:kod", get(muavin_hareket))
        .route("/api/muavin/txt", get(muavin_txt))
        .route("/api/bilanco", get(bilanco_uc))
        .route("/api/gelir-tablosu", get(gelir_tablosu_uc))
        .route("/api/analiz", get(analiz))
        .route("/api/fatura", post(fatura_olustur))
        .route("/api/faturalar", get(faturalar))
        .route("/api/fatura/:id/gonder", post(fatura_gonder))
        .route("/api/fatura/:id/kesinlestir", post(fatura_kesinlesti))
        .route("/api/fatura/:id/belge", get(fatura_belge))
        .route("/api/ornek-veri", post(ornek_veri))
        .route("/api/banka/ekstre", post(banka_ekstre_yukle))
        .route("/api/banka", get(banka_listesi))
        .route("/api/banka/:id/oneri", get(banka_oneri))
        .route("/api/banka/:id/esle", post(banka_esle))
        .route("/api/banka/oys/hesaplar", get(banka_acik::hesaplar))
        .route("/api/banka/oys/hareketler", get(banka_acik::hareketler))
        .route("/api/belge/oys/faturalar", get(belge_acik::faturalar))
        .route("/api/belge/oys/fatura/:uuid/xml", get(belge_acik::fatura_xml))
        .route("/api/belge/oys/ekstre-isaretli/:ref", get(belge_acik::ekstre_isaretli))
        .route("/api/belge/oys/eslestirme", get(belge_acik::eslestirme))
        .route("/api/belge/oys/iz/:anahtar", get(belge_acik::iz))
        .route("/api/belge/coz", post(belge_oku::belge_coz))
        .route("/api/belge/diller", get(belge_oku::belge_diller))
        .route("/api/belge/duzelt", get(duzelt::terim_duzelt))
        .route("/api/belge/terimler", get(duzelt::terim_listesi))
        .route("/api/belge/cari-oner", post(eslestir::cari_oner))
        .route("/api/belge/alim/sablonlar", get(belge_alim::alim_sablonlar))
        .route("/api/belge/alim/basla", post(belge_alim::alim_basla))
        .route("/api/belge/alim/deger", post(belge_alim::alim_deger))
        .route("/api/belge/alim/onayla", post(belge_alim::alim_onayla))
        .route("/api/belge/alim/tamamla", post(belge_alim::alim_tamamla))
        .route("/api/belge/alim/fis-taslagi", post(belge_alim::alim_fis_taslagi))
        .route("/api/belge/dosya", post(belge_dosya::belge_dosya))
        .route("/api/belge/sablonlar", get(belge_ogren::sablon_listesi))
        .route("/api/belge/sablon/:id", delete(belge_ogren::sablon_unut))
        .route("/api/belge/alim/:id", get(belge_alim::alim_durum))
        .route("/api/simulasyon/ozet", get(simulasyon::ozet))
        .route("/api/simulasyon/faturalar", get(simulasyon::faturalar))
        .route("/api/simulasyon/banka", get(simulasyon::banka))
        .route("/api/simulasyon/firmalar", get(simulasyon::firmalar))
        .route("/api/simulasyon/fatura-sekmeler", get(simulasyon::fatura_sekmeler))
        .route("/api/simulasyon/tahsilatlar", get(simulasyon::tahsilatlar))
        .route("/api/tahsis/manuel", post(tahsis_manuel))
        .route("/api/tahsis/senkronize", post(tahsis_senkronize))
        .route("/api/muhasebe/aktar", post(muhasebe_aktar))
        .route("/api/tahsis/defter", get(tahsis_defter))
        .route("/api/denetim/anomali", get(denetim_anomali))
        .route("/api/kontrol/saglik", get(kontrol_saglik))
        .route("/api/havada", get(havada_liste))
        .route("/api/havada/fatura-bekleniyor", post(havada_bekleme_isaretle))
        .route("/api/havada/fatura-bekleniyor/:id", delete(havada_bekleme_sil))
        .route("/api/tahsis/manuel/:tahsilat_id", delete(tahsis_manuel_sil))
        .route("/api/simulasyon/havada", get(simulasyon::havada))
        .route("/api/simulasyon/kasa", get(simulasyon::kasa))
        .route("/api/belge", post(belge_ekle))
        .route("/api/belgeler", get(belge_listesi))
        .route("/api/belge/:id/oneri", get(belge_oneri))
        .route("/api/belge/:id/esle", post(belge_esle))
        .route("/api/denetim/programlar", get(denetim_programlar))
        .route("/api/denetim/kagit/:id", get(denetim_kagit))
        .route("/api/denetim/kagit/:id/not", post(denetim_not))
        .route("/api/denetim/kagit/:id/duzenle", post(denetim_duzenle))
        .route("/api/varlik/:kod", get(varlik_karti))
        .route("/api/ufrs/calismalar", get(ufrs_calismalar))
        .route("/api/ufrs/calisma/:id", get(ufrs_calisma))
        .route("/api/ufrs/kayit", post(ufrs_kayit_ekle))
        .route("/api/ufrs/kayit/:no/vazgec", post(ufrs_kayit_vazgec))
        .route("/api/ufrs/kayitlar", get(ufrs_kayitlar_listesi))
        .route("/api/ufrs/wtb", get(ufrs_wtb))
        .route("/api/ufrs/calisma/:id/hesapla", post(ufrs_hesapla))
        .route("/api/ufrs/calisma/:id/senaryo", post(ufrs_senaryo))
        .route("/api/ufrs/kesinlestir", post(ufrs_kesinlestir))
        .route("/api/ufrs/devir", post(ufrs_devir))
        .route("/api/kdv", get(kdv_durum))
        .route("/api/kdv-beyanname", get(kdv_beyanname))
        .route("/api/kdv-beyanname/duzenle", post(kdv_beyanname_duzenle))
        .route("/api/gecici-vergi", get(gecici_vergi))
        .route("/api/gecici-vergi/duzenle", post(gecici_vergi_duzenle))
        .route("/api/beyanname/babs", get(hesaplama::babs))
        .route("/api/gider/katalog", get(gider::gider_katalog))
        .route("/api/gider/oner", post(gider::gider_oner))
        .route("/api/hesapla/oranlar", get(hesaplama::oranlar))
        .route("/api/hesapla/kdv", post(hesaplama::kdv))
        .route("/api/hesapla/stopaj", post(hesaplama::stopaj))
        .route("/api/hesapla/damga", post(hesaplama::damga))
        .route("/api/hesapla/otv", post(hesaplama::otv))
        .route("/api/vergi-parametreler", get(vergi_parametreler))
        .route("/api/kdv-mahsup", post(kdv_mahsup_yap))
        // GÖVDE SINIRI — axum'un varsayılanı 2 MB'dır ve belge yüklemede YETMEZ:
        // taranmış bir fatura PDF'i rahatça 5-20 MB olur, base64 ile bir de %33 şişer.
        // Sınır aşıldığında istemci "connection reset" görüyordu; hata mesajı bile yoktu.
        // 48 MB, belge_dosya'daki 25 MB ham dosya sınırının base64 karşılığını rahat kapsar.
        .layer(axum::extract::DefaultBodyLimit::max(48 * 1024 * 1024))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let adres = "127.0.0.1:8787";
    println!("Audit-Liners API → http://{adres}");
    let listener = tokio::net::TcpListener::bind(adres).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
