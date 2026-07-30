//! İSTATİSTİKSEL ÖLÇÜM — düzelticinin gerçekten ne kadar isabetli olduğunu SAYIYLA söyler.
//!
//! ## Neden bu dosya var
//!
//! Düzelticiyi ilk yazdığımda "Matrih → Matrah" gibi elle uydurduğum birkaç örnekle test
//! ettim ve "çalışıyor" dedim. Bu bir ölçüm değildi:
//!   · Örnekleri ben seçtim → başarısız olacakları seçmedim (seçim yanlılığı).
//!   · Eşik (0.34) ve marj hisle konmuştu; hiçbir veriye dayanmıyordu.
//!   · **En tehlikeli metrik hiç ölçülmemişti:** terim OLMAYAN bir sözcüğün yanlışlıkla
//!     terime çekilme oranı. Muhasebede bu, sessiz veri bozulmasıdır — kullanıcı öneriyi
//!     görür, makul durur, onaylar ve yanlış hesaba kayıt açılır.
//!
//! Burada onu ölçüyoruz. Belge okuma isabetini ölçemeyiz (yer gerçeği yok, n çok küçük),
//! ama düzelticiyi ölçebiliriz: sözlük yer gerçeğinin ta kendisidir.
//!
//! ## Dört sonuç ve ağırlıkları
//!
//! | Sonuç | Anlamı | Maliyet |
//! |---|---|---|
//! | **KURTARMA** | Bozuk terim → doğru terim önerildi | kazanç |
//! | **ÇEKİMSER** | Öneri verilmedi | ucuz — kullanıcı elle yazar |
//! | **YANLIŞ DÜZELTME** | Bozuk terim → BAŞKA terim önerildi | **pahalı** |
//! | **UYDURMA** | Terim olmayan sözcük → terim önerildi | **en pahalı** |
//!
//! Çekimserlik hata değildir. Sessiz yanlış, gürültülü doğrudan beterdir.

use super::*;

// ─────────────────────────────────────────────────────────────────────────────
// Bozulma modeli — deterministik (sabit tohum), rastgelelik dışarıdan gelmez
// ─────────────────────────────────────────────────────────────────────────────

fn lcg(s: &mut u64) -> u64 {
    *s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    *s >> 33 // yüksek bitler — düşük bitlerin periyodu kısadır
}

/// Bir sözcüğü `n` kez bozar. Bozulma türleri gerçek okuma hatalarını taklit eder:
/// harf değiştirme (görsel karışma), düşme, ekleme, bitişik yer değiştirme.
fn boz(kelime: &str, n: u32, s: &mut u64) -> String {
    let karisan = "o0l1i5sb8z2g9nrmcaeuvtf";
    let mut h: Vec<char> = kelime.chars().collect();
    for _ in 0..n {
        if h.is_empty() { break; }
        let i = (lcg(s) as usize) % h.len();
        match lcg(s) % 4 {
            0 => { // görsel olarak karışan bir harfle değiştir
                let k: Vec<char> = karisan.chars().collect();
                h[i] = k[(lcg(s) as usize) % k.len()];
            }
            1 => { h.remove(i); }                     // harf düşmesi
            2 => { h.insert(i, 'l'); }                // fazladan çizgi → 'l'
            _ => { if i + 1 < h.len() { h.swap(i, i + 1); } } // bitişik yer değiştirme
        }
    }
    h.into_iter().collect()
}

/// TERİM OLMAYAN sözcükler — yanlış pozitif ölçümü için. Muhasebecinin belgede karşılaştığı
/// ama sözlükte olmayan gerçek sözcük türleri: özel isim, yer adı, gündelik sözcük, ürün adı.
const YABANCI: &[&str] = &[
    // Kişi/firma adları (okuduğum belgelerden ve tipik TR ünvanlarından)
    "Forbes", "Maxwell", "Reichmann", "Barretto", "Woodcock", "Osler", "Breville",
    "Anadolu", "Karabulut", "Yılmaz", "Demirtaş", "Öztürk", "Kayahan", "Sönmez",
    "Ege Tekstil", "Marmara Lojistik", "Toros Gıda", "Akdeniz İnşaat",
    // Yer adları
    "İstanbul", "Ankara", "İzmir", "Gaziantep", "Los Angeles", "Berlin", "Washington",
    // Gündelik sözcükler
    "merhaba", "teşekkür", "lütfen", "bugün", "yarın", "sayfa", "kalem", "masa",
    "hello", "please", "morning", "letter", "paper", "table", "window",
    // Ürün/hizmet adları (fatura satırlarında geçer, terim değildir)
    "Castor Oil", "Epsom Salts", "Senna", "Lotion", "vida", "civata", "kablo",
    "danışmanlık", "nakliye", "temizlik", "yazılım lisansı",
];

/// Her (terim, şiddet) için kaç bozulma örneği üretilecek — örneklem büyüklüğü buradan gelir.
const TEKRAR: u32 = 3;

#[derive(Default, Debug, Clone)]
struct Skor {
    kurtarma: u32,      // bozuk terim → doğru terim
    cekimser: u32,      // bozuk terim → öneri yok
    yanlis: u32,        // bozuk terim → BAŞKA terim
    uydurma: u32,       // terim değil → terim önerildi
    yabanci_toplam: u32,
    /// Yanlış pozitif örnekleri — hangi sözcük hangi terime çekildi (teşhis için).
    uydurma_ornekleri: Vec<String>,
}

impl Skor {
    fn terim_toplam(&self) -> u32 { self.kurtarma + self.cekimser + self.yanlis }
    fn kurtarma_orani(&self) -> f32 { 100.0 * self.kurtarma as f32 / self.terim_toplam().max(1) as f32 }
    fn yanlis_orani(&self) -> f32 { 100.0 * self.yanlis as f32 / self.terim_toplam().max(1) as f32 }
    fn uydurma_orani(&self) -> f32 { 100.0 * self.uydurma as f32 / self.yabanci_toplam.max(1) as f32 }
    /// Tek sayıya indirgeme: kurtarma iyi, yanlış düzeltme 3 kat, uydurma 5 kat kötü.
    /// Ağırlıklar keyfi değil — muhasebede sessiz yanlış kaydın maliyeti bunu gerektiriyor.
    fn puan(&self) -> f32 {
        self.kurtarma_orani() - 3.0 * self.yanlis_orani() - 5.0 * self.uydurma_orani()
    }
}

/// Tüm sözlüğü bozup ölçer. `diller`: hangi dillerde ölçüleceği.
fn olc(esik: f32, marj: f32, siddetler: &[u32]) -> Skor {
    let s0 = sozluk();
    let mut sk = Skor::default();
    let mut tohum = 0x5EED_1453u64; // sabit — ölçüm tekrarlanabilir olmalı

    for dil in ["tr", "en", "de", "fr", "ru"] {
        let terimler: Vec<String> = s0["terimler"][dil].as_array().into_iter().flatten()
            .filter_map(|t| t["t"].as_str().map(String::from)).collect();

        for terim in &terimler {
            for &siddet in siddetler {
                // Her (terim, şiddet) için birden çok bozulma örneği — tek örnek şansa bağlıdır,
                // istatistiksel güç örneklem büyüklüğünden gelir.
                for _ in 0..TEKRAR {
                let bozuk = boz(&sade(terim), siddet, &mut tohum);
                if bozuk.trim().is_empty() { continue; }
                let d = duzelt_ayarli(&bozuk, dil, "", esik, marj);
                match (&d.tam_eslesme, &d.oneri) {
                    (Some(t), _) if sade(t) == sade(terim) => sk.kurtarma += 1,
                    (Some(_), _) => sk.yanlis += 1,     // başka terime tam oturdu
                    (None, Some(o)) if sade(&o.oneri) == sade(terim) => sk.kurtarma += 1,
                    (None, Some(_)) => sk.yanlis += 1,
                    (None, None) => sk.cekimser += 1,
                }
                }
            }
        }
    }

    // YANLIŞ POZİTİF — terim olmayan sözcükler hiç terime çekilmemeli.
    // Hem temiz hem hafifçe bozulmuş hallerini deniyoruz (gerçekte ikisi de gelir).
    for dil in ["tr", "en"] {
        for y in YABANCI {
            for siddet in [0u32, 1] {
                for _ in 0..TEKRAR {
                    let k = if siddet == 0 { sade(y) } else { boz(&sade(y), 1, &mut tohum) };
                    if k.trim().is_empty() { continue; }
                    let d = duzelt_ayarli(&k, dil, "", esik, marj);
                    sk.yabanci_toplam += 1;
                    let cekildi = d.oneri.as_ref().map(|o| o.oneri.clone())
                        .or_else(|| d.tam_eslesme.clone());
                    if let Some(hedef) = cekildi {
                        sk.uydurma += 1;
                        sk.uydurma_ornekleri.push(format!("[{dil}] '{k}' → '{hedef}'"));
                    }
                    if siddet == 0 { break; } // temiz hal tek kez denenir
                }
            }
        }
    }
    sk
}

// ─────────────────────────────────────────────────────────────────────────────
// TÜRKÇE EK TOLERANSI — ölçümün kendisindeki boşluğu kapatan sınama
// ─────────────────────────────────────────────────────────────────────────────
// Yukarıdaki bozulma modeli terimi bozar ama EK EKLEMEZ. Oysa Türkçe belgede etiket
// çıplak yazılmaz: "Fatura Tarih" değil "Fatura Tarihi", "KDV Matrah" değil "KDV Matrahı".
// Yani ölçüm, gerçek dünyadaki en yaygın Türkçe varyasyonu hiç görmüyordu — ek desteği
// eklenince genel skor DÜŞTÜ (daha çok aday yakınlaşıp belirsizlik kuralı devreye girdi),
// çünkü kıyas ölçüm faydayı ölçemiyordu. Bu test o boşluğu kapatır.

/// Türkçe ad tamlamasında 3. tekil iyelik eki — belgede fiilen görülen biçim.
fn iyelik_ekle(terim: &str) -> Option<String> {
    let s = sade(terim);
    let son = s.chars().last()?;
    let sesli = "aeiou".contains(son);
    // Kaba ünlü uyumu: son ünlüye göre "ı/i/u/ü" (sadeleştirmede ı→i, ü→u).
    let kalin = s.chars().rev().find(|c| "aeiou".contains(*c)).map(|c| "aou".contains(c)).unwrap_or(true);
    let ek = match (sesli, kalin) {
        (true, true) => "si", (true, false) => "si",
        (false, true) => "i", (false, false) => "i",
    };
    Some(format!("{s}{ek}"))
}

#[test]
fn turkce_ekli_etiketler_taninir() {
    let s0 = sozluk();
    let terimler: Vec<String> = s0["terimler"]["tr"].as_array().into_iter().flatten()
        .filter_map(|t| t["t"].as_str().map(String::from)).collect();

    let (mut tanindi, mut kacti, mut toplam) = (0u32, 0u32, 0u32);
    let mut kacanlar: Vec<String> = Vec::new();
    for terim in &terimler {
        let Some(ekli) = iyelik_ekle(terim) else { continue };
        if ekli == sade(terim) { continue; }
        toplam += 1;
        let d = duzelt(&ekli, "tr", "");
        let dogru = d.tam_eslesme.as_deref().map(|x| sade(x) == sade(terim)).unwrap_or(false)
            || d.oneri.as_ref().map(|o| sade(&o.oneri) == sade(terim)).unwrap_or(false);
        if dogru { tanindi += 1 } else { kacti += 1; if kacanlar.len() < 10 { kacanlar.push(format!("'{ekli}' ⇏ '{terim}' ({})", d.durum)); } }
    }
    let oran = 100.0 * tanindi as f32 / toplam.max(1) as f32;
    println!("\n  TÜRKÇE EKLİ ETİKET TANIMA ({toplam} terim)");
    println!("    tanındı: {tanindi} (%{oran:.1}) · kaçtı: {kacti}");
    for k in &kacanlar { println!("      {k}"); }
    println!();

    // Ek desteği OLMADAN bu oran çok düşüktü: "Tutarı" ile "Tutar" arasında 1 harf fark var
    // ama kısa terimlerde bu eşiği aşıyor ve etiket hiç tanınmıyordu.
    assert!(oran >= 90.0,
        "Türkçe ekli etiketlerin %{oran:.1}'i tanınıyor — belgede etiketler EKLİ yazılır, \
         bu oran yüksek olmalı");
}

// ─────────────────────────────────────────────────────────────────────────────
// AYAR TARAMASI — eşiği ve marjı veriyle seç, hisle değil
// ─────────────────────────────────────────────────────────────────────────────

/// **AYAR ARACI — normal koşuda ATLANIR.**
///
/// Bu bir regresyon koruması değil, bir *parametre taraması*: 20 ızgara noktasının her
/// birinde tüm terim külliyatını bozup ölçüyor. Koruma görevini
/// `olculen_isabet_taban_degerlerin_altina_dusmez` yapıyor; bu test yalnız `ESIK_CARPANI`
/// ve `MARJ` seçilirken gerekiyor.
///
/// Sözlük büyüdükçe maliyeti doğrudan artıyor: terim sayısı 202 → 468 olunca tam takım
/// süresi **140 s → 631 s** çıktı ve bunun neredeyse tamamı bu taramaydı. Ayar aracının
/// her derlemede koşması, sözlüğü büyütmenin bedelini gereksiz yere yükseltir.
///
/// Koşmak için:  `cargo test -p api ayar_taramasi -- --ignored --nocapture`
#[test]
#[ignore = "ayar taraması — 20 ızgara noktası × tüm külliyat; --ignored ile koşulur"]
fn ayar_taramasi_ve_secilen_nokta_en_iyisi() {
    let siddetler = [1u32, 2];
    let mut satirlar: Vec<(f32, f32, Skor)> = Vec::new();
    for e in [0.20f32, 0.28, 0.34, 0.42, 0.50] {
        for m in [0.0f32, 0.15, 0.30, 0.50] {
            satirlar.push((e, m, olc(e, m, &siddetler)));
        }
    }

    println!("\n  eşik  marj │ kurtarma  yanlış  uydurma │ puan");
    println!("  ───────────┼──────────────────────────┼──────");
    for (e, m, s) in &satirlar {
        let secili = (*e - ESIK_CARPANI).abs() < 0.001 && (*m - MARJ).abs() < 0.001;
        println!("  {:.2}  {:.2} │  {:>6.1}%  {:>5.1}%  {:>6.1}% │ {:>6.1} {}",
            e, m, s.kurtarma_orani(), s.yanlis_orani(), s.uydurma_orani(), s.puan(),
            if secili { "← SEÇİLİ" } else { "" });
    }

    let en_iyi = satirlar.iter().max_by(|a, b| a.2.puan().partial_cmp(&b.2.puan()).unwrap()).unwrap();
    let secili = satirlar.iter()
        .find(|(e, m, _)| (*e - ESIK_CARPANI).abs() < 0.001 && (*m - MARJ).abs() < 0.001)
        .expect("seçili ayar ızgarada yok — ESIK_CARPANI/MARJ ile tarama uyuşmuyor");

    println!("\n  en iyi ızgara noktası: eşik {:.2} marj {:.2} → puan {:.1}", en_iyi.0, en_iyi.1, en_iyi.2.puan());
    println!("  seçili ayar          : eşik {:.2} marj {:.2} → puan {:.1}\n", secili.0, secili.1, secili.2.puan());

    // Seçili ayar, ızgaranın en iyisine YAKIN olmalı. Uzaksa sabitler güncellenmelidir.
    assert!(secili.2.puan() >= en_iyi.2.puan() - 3.0,
        "Seçili ayar (eşik {:.2}, marj {:.2}) artık en iyi değil. \
         Izgarada eşik {:.2}, marj {:.2} daha iyi ({:.1} > {:.1}) — ESIK_CARPANI/MARJ güncellenmeli.",
        secili.0, secili.1, en_iyi.0, en_iyi.1, en_iyi.2.puan(), secili.2.puan());
}

// ─────────────────────────────────────────────────────────────────────────────
// KALİTE TABANI — regresyon bariyeri
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn olculen_isabet_taban_degerlerin_altina_dusmez() {
    let s = olc(ESIK_CARPANI, MARJ, &[1, 2]);
    let n = s.terim_toplam();

    println!("\n  ÖLÇÜM (5 dil · {n} bozulmuş terim · {} yabancı sözcük)", s.yabanci_toplam);
    println!("    kurtarma       : {:>5.1}%  ({} adet)", s.kurtarma_orani(), s.kurtarma);
    println!("    çekimser       : {:>5.1}%  ({} adet) — hata değil, elle yazılır",
        100.0 * s.cekimser as f32 / n as f32, s.cekimser);
    println!("    YANLIŞ DÜZELTME: {:>5.1}%  ({} adet)", s.yanlis_orani(), s.yanlis);
    println!("    UYDURMA        : {:>5.1}%  ({} adet)\n", s.uydurma_orani(), s.uydurma);

    assert!(n > 1000, "örneklem çok küçük ({n}) — ölçüm anlamlı olmaz");
    if !s.uydurma_ornekleri.is_empty() {
        println!("    UYDURMA ÖRNEKLERİ (terim olmayan sözcük terime çekildi):");
        for o in s.uydurma_ornekleri.iter().take(12) { println!("      {o}"); }
        println!();
    }
    // Tabanlar ölçülen değerlerden GEVŞEK seçildi: amaç mevcut seviyeyi dondurmak değil,
    // ciddi gerilemeyi yakalamak. Sıkı taban, sözlük büyüdükçe yanlış alarm üretir.
    assert!(s.kurtarma_orani() >= 55.0, "kurtarma oranı düştü: %{:.1}", s.kurtarma_orani());
    assert!(s.yanlis_orani() <= 20.0, "YANLIŞ DÜZELTME arttı: %{:.1}", s.yanlis_orani());
    assert!(s.uydurma_orani() <= 12.0, "UYDURMA arttı: %{:.1} — terim olmayan sözcükler terime çekiliyor",
        s.uydurma_orani());
}

/// Bozulma şiddeti arttıkça motor kurtarmayı bırakıp ÇEKİMSER kalmalı — yanlış düzeltmeye
/// KAÇMAMALI. Bir düzelticinin bozulma altındaki davranışı, temiz verideki başarısından önemlidir.
#[test]
fn siddet_artinca_yanlisa_degil_cekimserlige_kacar() {
    let hafif = olc(ESIK_CARPANI, MARJ, &[1]);
    let agir = olc(ESIK_CARPANI, MARJ, &[3]);
    println!("\n  hafif bozulma (1): kurtarma %{:.1} · yanlış %{:.1}", hafif.kurtarma_orani(), hafif.yanlis_orani());
    println!("  ağır bozulma  (3): kurtarma %{:.1} · yanlış %{:.1}\n", agir.kurtarma_orani(), agir.yanlis_orani());

    assert!(agir.kurtarma_orani() < hafif.kurtarma_orani(),
        "ağır bozulmada kurtarma düşmedi — ölçüm kurgusu şüpheli");
    let cekimser_agir = 100.0 * agir.cekimser as f32 / agir.terim_toplam().max(1) as f32;
    let cekimser_hafif = 100.0 * hafif.cekimser as f32 / hafif.terim_toplam().max(1) as f32;
    assert!(cekimser_agir > cekimser_hafif,
        "ağır bozulmada çekimserlik artmadı — motor tahmine kaçıyor olabilir");
}

/// Türkçe ve İngilizce ÇEKİRDEK dillerdir; ikinci halkadan belirgin geride olmamalılar.
#[test]
fn cekirdek_diller_ikinci_halkadan_geride_degil() {
    let tek = |dil: &str| -> Skor {
        let s0 = sozluk();
        let mut sk = Skor::default();
        let mut t = 0x5EED_1453u64;
        for terim in s0["terimler"][dil].as_array().into_iter().flatten()
            .filter_map(|x| x["t"].as_str()) {
            for siddet in [1u32, 2] {
                let bozuk = boz(&sade(terim), siddet, &mut t);
                if bozuk.trim().is_empty() { continue; }
                let d = duzelt_ayarli(&bozuk, dil, "", ESIK_CARPANI, MARJ);
                let dogru = d.tam_eslesme.as_deref().map(|x| sade(x) == sade(terim)).unwrap_or(false)
                    || d.oneri.as_ref().map(|o| sade(&o.oneri) == sade(terim)).unwrap_or(false);
                if dogru { sk.kurtarma += 1 }
                else if d.oneri.is_some() || d.tam_eslesme.is_some() { sk.yanlis += 1 }
                else { sk.cekimser += 1 }
            }
        }
        sk
    };
    println!();
    let mut oranlar = Vec::new();
    for dil in ["tr", "en", "de", "fr", "ru"] {
        let s = tek(dil);
        println!("    {dil}: kurtarma %{:>5.1} · yanlış %{:>5.1} · n={}",
            s.kurtarma_orani(), s.yanlis_orani(), s.terim_toplam());
        oranlar.push((dil, s.kurtarma_orani()));
    }
    println!();
    let cekirdek = oranlar.iter().filter(|(d, _)| *d == "tr" || *d == "en")
        .map(|(_, o)| *o).fold(f32::MAX, f32::min);
    assert!(cekirdek >= 50.0, "çekirdek dilde kurtarma çok düşük: %{cekirdek:.1}");
}
