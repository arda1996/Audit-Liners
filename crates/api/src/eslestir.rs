//! KAPALI KÜME EŞLEŞTİRME — bulmaca modeli.
//!
//! ## Sorun
//!
//! Taranmış belgede metin bozuk okunuyor: `ŞELALE GİYİM` → `SELALE GiYiM`, `ÇORAP` → `CORAP`.
//! Sayısal alanları denklem koruyor (`matrah + KDV = toplam`) ama **metnin aritmetiği yoktur.**
//! Bozuk ünvan muhasebe fişine ve oradan Ba/Bs mutabakatına gidiyor.
//!
//! ## Yaklaşım: bulmaca TAHMİN ETMEZ, ELER
//!
//! Bir kare bulmacada harfleri, uzunluğu ve kesişmeleri bilirsen genelde **tek** cevap kalır.
//! Bu bir tahmin değil **çıkarımdır**. İki cevap kalıyorsa çözen kişi birini seçmez — başka
//! ipucu bekler.
//!
//! Bu, denklem hakeminin metin karşılığıdır. Aynı doktrin, farklı veri türü:
//!
//! | | Eleyen | Karar |
//! |---|---|---|
//! | Sayı | Aritmetik denklem | Tek aday kalırsa kabul |
//! | Metin | Kapalı küme + karışım sınıfı | Tek aday kalırsa kabul |
//!
//! İkisinde de kural aynı: **birden fazla aday kalıyorsa doldurmayız, sorarız.**
//!
//! ## Neden bu bir dil modeli işi DEĞİL
//!
//! Bir alan muhasebe için ne kadar kritikse aday kümesi o kadar **kapalıdır**: etiketler
//! sözlükte, cari ünvanı defterde, VKN checksum'lı. `ŞELALE`'yi bulmak bir dil modeli
//! problemi değil, **elli kayıtlık bir listede arama** problemidir. Genel bir dil modeli
//! gazete Türkçesi bilir ve burada uydurma üretir; kapalı küme uyduramaz — ya listede
//! vardır ya yoktur.
//!
//! ## Üç katman, azalan kesinlik
//!
//! 1. **SADE** — aksan katlanmış tam eşleşme. `ŞELALE` ≡ `SELALE`, çünkü `sadelestir()`
//!    ikisini de `selale` yapar. Türkçe karakter kaybının TAMAMINI kapatır. Kesin.
//! 2. **MASKE** — her harf görsel karışım sınıfına genişletilir (`[oa0][l1i]…`), aday küme
//!    bu maskeye göre süzülür. Aksanın kapatamadığı `0/O`, `1/I`, `5/S` türü hataları
//!    yakalar. Kesin.
//! 3. **ONEK** — biri diğerinin sözcük sınırındaki öneki. Ticaret ünvanının hukuki eki
//!    (A.Ş. / LTD. ŞTİ.) belgede sık kesilir; düz uzaklık bunu ağır cezalandırır. **Olası**.
//! 4. **UZAKLIK** — karışma duyarlı düzenleme uzaklığı + marj. Harf eklenip düşmüşse
//!    (birleşen/bölünen sözcük) tek çare budur. **Olası** — kesin değil, onay ister.
//!
//! ## Denetim sınırı
//!
//! Denetimde **makul bir uydurma, bariz bir bozukluktan daha tehlikelidir.** `SELALE` gözle
//! yakalanır; uydurulmuş makul bir ünvan mutabakata sessizce girer. Bu yüzden bu modül
//! **karar vermez, gerekçeli aday döndürür** — deftere yazma kararı çağıranındır ve
//! `gerekce` alanı VUK 219 anlamında izi taşır: hangi katman, hangi kanıt.

use serde::Serialize;

/// Eşleşmenin ne kadar sağlam olduğu. Katmandan türer, ayrı bir tahmin değildir.
#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Guven {
    /// Kapalı kümede TEK aday kaldı — çıkarım. Yine de ilk kullanımda onay istenir.
    Kesin,
    /// Uzaklıkla bulundu; harf eklenmiş/düşmüş olabilir. Onaysız kullanılmaz.
    Olasi,
}

#[derive(Serialize, Debug, Clone)]
pub struct Eslesme {
    pub aday: String,
    /// SADE | MASKE | UZAKLIK — hangi katmanın bulduğu. İz kaydının çekirdeği.
    pub katman: &'static str,
    pub guven: Guven,
    /// İnsana gösterilecek gerekçe. "Neden bu ad deftere yazıldı?" sorusunun cevabı.
    pub gerekce: String,
}

#[derive(Serialize, Debug)]
pub struct Sonuc {
    /// TEK | BELIRSIZ | YOK
    pub karar: &'static str,
    pub eslesme: Option<Eslesme>,
    /// BELIRSIZ ise kullanıcının arasından seçeceği adaylar.
    pub adaylar: Vec<String>,
    pub not: String,
}

impl Sonuc {
    fn yok(n: impl Into<String>) -> Self {
        Sonuc { karar: "YOK", eslesme: None, adaylar: Vec::new(), not: n.into() }
    }
    fn tek(aday: &str, katman: &'static str, guven: Guven, gerekce: String) -> Self {
        Sonuc {
            karar: "TEK",
            not: format!("{} katmanında tek aday: {}", katman, aday),
            eslesme: Some(Eslesme { aday: aday.into(), katman, guven, gerekce }),
            adaylar: Vec::new(),
        }
    }
    fn belirsiz(adaylar: Vec<String>, katman: &'static str) -> Self {
        Sonuc {
            karar: "BELIRSIZ",
            not: format!(
                "{} katmanında {} aday eşleşti — hangisi olduğu belgeden ÇIKARILAMAZ, sorulmalı.",
                katman, adaylar.len()),
            eslesme: None,
            adaylar,
        }
    }
}

fn sade(s: &str) -> String { crate::belge_oku::sadelestir(s).trim().to_string() }

/// Bir karakterin karışım sınıfı — kendisi + görsel olarak karışabildiği harfler.
///
/// Aksan çiftleri (i/ı, s/ş, g/ğ …) burada GEREKMEZ: `sadelestir()` onları zaten katlıyor,
/// yani maskeye gelmeden önce eşitlenmiş oluyorlar. Bu katmanın işi aksanın kapatamadığı
/// **görsel** karışmalar (0/O, 1/I, 5/S, rn/m …).
fn sinif(c: char) -> Vec<char> {
    let mut v = crate::duzelt::karisim_ortaklari(c);
    if !v.contains(&c) { v.push(c); }
    v
}

/// Okunan sözcüğün karışım maskesi: her konumda hangi harfler kabul edilebilir.
pub fn maske(okunan_sade: &str) -> Vec<Vec<char>> {
    okunan_sade.chars().map(sinif).collect()
}

fn maskeye_uyar(m: &[Vec<char>], aday_sade: &str) -> bool {
    let a: Vec<char> = aday_sade.chars().collect();
    if a.len() != m.len() { return false; }
    a.iter().zip(m).all(|(c, s)| s.contains(c))
}

/// UZAKLIK katmanı eşikleri. Eşik uzunlukla ORANTILIDIR: uzun ünvanda 2 harflik sapma
/// olağandır, 4 harflik sözcükte aynı sapma başka bir sözcük demektir.
pub const UZAKLIK_ESIK: f32 = 0.25;
/// Kazanan, ikinciden en az bu kadar (oransal) iyi olmalı. Değilse BELİRSİZ.
pub const UZAKLIK_MARJ: f32 = 0.15;

/// Okunanı kapalı aday kümesinde arar.
///
/// **Karar vermez** — gerekçeli sonuç döndürür. Deftere yazma kararı çağıranındır.
pub fn ara(okunan: &str, kume: &[String]) -> Sonuc {
    let o = sade(okunan);
    if o.chars().filter(|c| c.is_alphanumeric()).count() < 2 {
        return Sonuc::yok("okunan metin eşleştirilemeyecek kadar kısa");
    }
    if kume.is_empty() {
        return Sonuc::yok("aday kümesi boş — karşılaştırılacak kayıt yok");
    }

    // ── KATMAN 1: aksan katlanmış tam eşleşme ────────────────────────────────
    // Türkçe karakter kaybının TAMAMI burada kapanır: "SELALE GiYiM" ile
    // "ŞELALE GİYİM" sadeleştirilince aynı dizgidir.
    let t1: Vec<&String> = kume.iter().filter(|k| sade(k) == o).collect();
    if t1.len() == 1 {
        return Sonuc::tek(t1[0], "SADE", Guven::Kesin,
            format!("Aksan farkı dışında BİREBİR aynı: okunan \"{}\" ≡ kayıtlı \"{}\".",
                okunan.trim(), t1[0]));
    }
    if t1.len() > 1 {
        return Sonuc::belirsiz(t1.into_iter().cloned().collect(), "SADE");
    }

    // ── KATMAN 2: görsel karışım maskesi ─────────────────────────────────────
    // Aynı uzunlukta, her harf kendi karışım sınıfı içinde. "0RNEK" → "ÖRNEK".
    let m = maske(&o);
    let t2: Vec<&String> = kume.iter().filter(|k| maskeye_uyar(&m, &sade(k))).collect();
    if t2.len() == 1 {
        return Sonuc::tek(t2[0], "MASKE", Guven::Kesin,
            format!("Her harf görsel karışım sınıfı içinde ve kümede TEK aday uyuyor: \
                     okunan \"{}\" → \"{}\".", okunan.trim(), t2[0]));
    }
    if t2.len() > 1 {
        return Sonuc::belirsiz(t2.into_iter().cloned().collect(), "MASKE");
    }

    // ── KATMAN 3: önek / kapsama ─────────────────────────────────────────────
    // Ticaret ünvanı hukuki ek taşır: "A.Ş.", "LTD. ŞTİ.", "SAN. TİC.". Belgede bu ek
    // sık sık kesilir, kısaltılır ya da alt satıra taşar. Düz uzaklık bunu ağır
    // cezalandırır — "ATLAS TİCARET" ile "ATLAS TİCARET A.Ş." arası 4 harftir ve kayıt
    // bulunamaz. Oysa biri diğerinin öneki olması güçlü bir işarettir.
    //
    // Kesme SÖZCÜK SINIRINDA olmalı: "atlas tic" bir öneki olsa da sözcüğü ortadan
    // böler ve tesadüfi eşleşme üretir.
    let onek = |kisa: &str, uzun_s: &str| -> bool {
        if kisa.len() >= uzun_s.len() { return false; }
        let Some(kalan) = uzun_s.strip_prefix(kisa) else { return false };
        if !kalan.starts_with(' ') { return false; }
        // Kısa taraf, uzun tarafın en az yarısı olmalı — yoksa "EGE" her şeye oturur.
        kisa.chars().count() >= 5 && kisa.len() * 2 >= uzun_s.len()
    };
    let t3: Vec<&String> = kume.iter()
        .filter(|k| { let s = sade(k); onek(&o, &s) || onek(&s, &o) })
        .collect();
    if t3.len() == 1 {
        return Sonuc::tek(t3[0], "ONEK", Guven::Olasi,
            format!("Biri diğerinin sözcük sınırındaki öneki — ticaret ünvanının hukuki eki \
                     (A.Ş./LTD. ŞTİ.) belgede kesilmiş olabilir: okunan \"{}\" → \"{}\". \
                     ONAY gerekir.", okunan.trim(), t3[0]));
    }
    if t3.len() > 1 {
        return Sonuc::belirsiz(t3.into_iter().cloned().collect(), "ONEK");
    }

    // ── KATMAN 4: uzaklık + marj ─────────────────────────────────────────────
    // Harf eklenmiş/düşmüşse ilk iki katman kaçırır. Burada kesinlik yoktur — ONAY ister.
    let uzun = o.chars().count().max(1) as f32;
    let mut olcum: Vec<(f32, &String)> =
        kume.iter().map(|k| (crate::duzelt::uzaklik(&o, &sade(k)), k)).collect();
    olcum.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    let (en_iyi, aday) = olcum[0];
    if en_iyi > uzun * UZAKLIK_ESIK {
        return Sonuc::yok(format!(
            "Kümede yeterince yakın kayıt yok (en yakın \"{}\", uzaklık {:.1}) — \
             YENİ cari olabilir.", aday, en_iyi));
    }
    // İkinci en iyi yeterince uzakta değilse seçim yapılamaz. Bu, en önemli korumadır:
    // iki cari birbirine benziyorsa motor kura çekmez.
    if let Some((ikinci, _)) = olcum.get(1).copied() {
        if ikinci - en_iyi < uzun * UZAKLIK_MARJ {
            let adaylar = olcum.iter()
                .take_while(|(d, _)| *d - en_iyi < uzun * UZAKLIK_MARJ)
                .map(|(_, k)| (*k).clone()).collect();
            return Sonuc::belirsiz(adaylar, "UZAKLIK");
        }
    }
    Sonuc::tek(aday, "UZAKLIK", Guven::Olasi,
        format!("Kümede en yakın kayıt (uzaklık {:.1}/{:.0}) ve ikincisi belirgin şekilde \
                 daha uzak — ama harf eklenmiş/düşmüş olabilir, ONAY gerekir.", en_iyi, uzun))
}

// ─────────────────────────────────────────────────────────────────────────────
// UÇ — defterdeki cari listesini aday kümesi olarak kullanır
// ─────────────────────────────────────────────────────────────────────────────

/// Defterde geçen cari ünvanları. Aday kümesi BURADAN gelir — motorun uydurabileceği
/// bir isim yoktur, yalnız daha önce görülmüş bir cariyi tanıyabilir.
fn cari_kumesi(state: &crate::Shared) -> Vec<String> {
    let d = { let st = state.lock().unwrap(); crate::gorunum(&st) };
    let set: std::collections::BTreeSet<String> = d.faturalar.iter()
        .map(|f| f.karsi.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    set.into_iter().collect()
}

#[derive(serde::Deserialize)]
pub struct CariGirdi { pub okunan: String }

/// POST /api/belge/cari-oner — belgeden okunan ünvanı defterdeki cari listesinde arar.
///
/// Motorun **karar vermediği** yer burası: sonuç her zaman gerekçelidir ve `karar` alanı
/// TEK/BELIRSIZ/YOK olarak kullanıcıya ne yapılacağını söyler. Deftere yazma kararı
/// arayüzde, kullanıcının onayıyla verilir.
pub async fn cari_oner(
    axum::extract::State(state): axum::extract::State<crate::Shared>,
    axum::Json(g): axum::Json<CariGirdi>,
) -> axum::Json<serde_json::Value> {
    let kume = cari_kumesi(&state);
    let s = ara(&g.okunan, &kume);
    axum::Json(serde_json::json!({
        "okunan": g.okunan,
        "karar": s.karar,
        "eslesme": s.eslesme,
        "adaylar": s.adaylar,
        "not": s.not,
        "kume_buyuklugu": kume.len(),
        "aciklama": "Aday kümesi defterdeki cari ünvanlarıdır. Kümede olmayan bir ad \
                     ÜRETİLMEZ — 'YOK' sonucu yeni cari demektir, hata değil.",
    }))
}

#[cfg(test)]
mod testler {
    use super::*;

    fn kume() -> Vec<String> {
        ["ÖRNEK TEKSTİL SAN. TİC. LTD. ŞTİ.", "AYŞE YILMAZ", "EGE GIDA A.Ş.",
         "MARMARA LOJİSTİK", "ANADOLU ÇELİK", "BOSPHORUS TRADING"]
            .iter().map(|s| s.to_string()).collect()
    }

    /// ASIL VAKA: OCR Türkçe karakterleri düşürdü. Aksan katlaması bunu TEK BAŞINA çözer.
    #[test]
    fn aksan_kaybi_sade_katmaninda_cozulur() {
        let s = ara("ORNEK TEKSTIL SAN. TIC. LTD. STI.", &kume());
        assert_eq!(s.karar, "TEK", "{}", s.not);
        let e = s.eslesme.unwrap();
        assert_eq!(e.katman, "SADE");
        assert_eq!(e.guven, Guven::Kesin);
        assert!(e.aday.starts_with("ÖRNEK TEKSTİL"));
    }

    #[test]
    fn kisi_adi_aksansiz_okunsa_da_bulunur() {
        let s = ara("AYSE YILMAZ", &kume());
        assert_eq!(s.karar, "TEK", "{}", s.not);
        assert_eq!(s.eslesme.unwrap().aday, "AYŞE YILMAZ");
    }

    /// Görsel karışma: O→0, I→1. Aksan bunu kapatmaz, maske kapatır.
    #[test]
    fn gorsel_karisma_maske_katmaninda_cozulur() {
        let s = ara("MARMARA L0J1STIK", &kume());
        assert_eq!(s.karar, "TEK", "{}", s.not);
        let e = s.eslesme.unwrap();
        assert_eq!(e.katman, "MASKE", "beklenen MASKE, gerekçe: {}", e.gerekce);
        assert_eq!(e.aday, "MARMARA LOJİSTİK");
    }

    /// EN ÖNEMLİ KORUMA: kümede olmayan bir cari UYDURULMAZ.
    #[test]
    fn kumede_olmayan_cari_uydurulmaz() {
        let s = ara("KAYSERİ MOBİLYA PAZARLAMA", &kume());
        assert_eq!(s.karar, "YOK", "olmayan cari için eşleşme üretildi: {:?}", s.eslesme);
        assert!(s.not.contains("YENİ cari"), "not: {}", s.not);
    }

    /// İki cari birbirine benziyorsa motor KURA ÇEKMEZ — sorar.
    #[test]
    fn benzer_iki_cari_belirsiz_kalir() {
        let k: Vec<String> = ["ATLAS TİCARET A.Ş.", "ATLAS TİCARET LTD."]
            .iter().map(|s| s.to_string()).collect();
        let s = ara("ATLAS TICARET", &k);
        assert_eq!(s.karar, "BELIRSIZ", "{} — {:?}", s.not, s.eslesme);
        assert_eq!(s.adaylar.len(), 2);
    }

    /// Aynı sadeleştirilmiş biçime düşen iki kayıt varsa da sorulmalı.
    #[test]
    fn ayni_sade_bicim_iki_kayit_belirsiz() {
        let k: Vec<String> = ["ÇELİK A.Ş.", "CELIK A.Ş."].iter().map(|s| s.to_string()).collect();
        let s = ara("CELIK A.S.", &k);
        assert_eq!(s.karar, "BELIRSIZ", "{}", s.not);
    }

    #[test]
    fn bos_kume_ve_kisa_metin_guvenli_doner() {
        assert_eq!(ara("EGE GIDA", &[]).karar, "YOK");
        assert_eq!(ara("A", &kume()).karar, "YOK");
        assert_eq!(ara("", &kume()).karar, "YOK");
    }

    /// Harf düşmesi UZAKLIK katmanına düşer ve KESİN değil OLASI döner — onay ister.
    #[test]
    fn harf_dusmesi_olasi_isaretlenir() {
        let s = ara("BOSPHORS TRADING", &kume());   // 'U' düştü
        assert_eq!(s.karar, "TEK", "{}", s.not);
        let e = s.eslesme.unwrap();
        assert_eq!(e.katman, "UZAKLIK");
        assert_eq!(e.guven, Guven::Olasi, "harf düşmesinde kesinlik iddia edilemez");
        assert!(e.gerekce.contains("ONAY"), "gerekçe onay istemiyor: {}", e.gerekce);
    }

    /// Her eşleşme İZ taşımalı — deftere yazılan adın nereden geldiği açıklanabilmeli (VUK 219).
    #[test]
    fn her_eslesme_gerekce_tasir() {
        for okunan in ["ORNEK TEKSTIL SAN. TIC. LTD. STI.", "MARMARA L0J1STIK", "BOSPHORS TRADING"] {
            let s = ara(okunan, &kume());
            let e = s.eslesme.expect(okunan);
            assert!(e.gerekce.len() > 30, "gerekçe yetersiz: {}", e.gerekce);
            assert!(!e.katman.is_empty());
        }
    }
}
