//! YAŞAYAN OKUMA KATMANI TESTLERİ.
//!
//! Sınanan asıl şey: gerçek belge DÜZENLERİ. Uydurma "Etiket: değer" satırları değil,
//! e-Fatura çıktısında fiilen görülen tablo/sütun/alt-satır yerleşimleri.

use super::*;

/// pdftotext -layout çıktısına benzeyen GERÇEK e-Fatura düzeni.
/// Dört ayrı tuzak içerir ve dördü de gerçek belgelerde vardır.
pub const EFATURA: &str = "\
ABC TİCARET VE SANAYİ A.Ş.                              e-FATURA
Atatürk Bulvarı No:12 Çankaya/ANKARA
Vergi Dairesi: Çankaya          VKN: 1234567890

SAYIN
EGE TEKSTİL SANAYİ A.Ş.
Vergi Dairesi: Konak            VKN: 9876543210

Fatura No                                        ABC2026000000777
Fatura Tarihi                                    12.04.2026

 Sıra  Mal/Hizmet          Miktar   Birim Fiyat        Tutar
 1     Pamuk iplik          100     80,00           8.000,00

                                    Mal Hizmet Toplam Tutarı      8.000,00
                                    Hesaplanan KDV %20            1.600,00
                                    Vergiler Dahil Toplam Tutar   9.600,00
";

#[test]
fn oran_tutara_yapismaz() {
    // EN SİNSİ HATA: "Hesaplanan KDV %20   1.600,00" satırının tamamı sayıya çevrilince
    // %20 ile tutar birleşip 201.600,00 oluyordu. Yüzdeye bitişik sayı TUTAR DEĞİLDİR.
    let b = belirtecler("TUTAR", "Hesaplanan KDV %20            1.600,00");
    assert_eq!(b, vec!["1.600,00"], "oran tutar sanıldı: {b:?}");
}

#[test]
fn satirdaki_her_tutar_ayri_aday() {
    let b = belirtecler("TUTAR", " 1     Pamuk iplik          100     80,00           8.000,00");
    assert!(b.contains(&"80,00".to_string()) && b.contains(&"8.000,00".to_string()),
        "tablo satırındaki tutarlar ayrışmadı: {b:?}");
}

#[test]
fn tarih_ve_vkn_bicim_dogrular() {
    assert_eq!(belirtecler("TARIH", "Fatura Tarihi     12.04.2026"), vec!["12.04.2026"]);
    assert!(belirtecler("TARIH", "Tarih: 12.04.26").is_empty(), "2 haneli yıl tarih sayıldı");
    assert_eq!(belirtecler("VKN", "Vergi Dairesi: Çankaya   VKN: 1234567890"), vec!["1234567890"]);
    assert!(belirtecler("VKN", "No:12 Çankaya").is_empty(), "adresteki sayı VKN sanıldı");
}

#[test]
fn deger_alt_satirdayken_bulunur() {
    // "SAYIN" kendi satırında, ünvan ALTINDA. Tek satır bakan tarayıcı "SAYIN" alıyordu.
    let satirlar: Vec<&str> = EFATURA.lines().collect();
    let a = adaylar(&satirlar, &["sayin".to_string()], "METIN");
    assert!(a.iter().any(|x| x.deger.contains("EGE TEKSTİL")),
        "alt satırdaki ünvan bulunamadı: {:?}", a.iter().map(|x| &x.deger).collect::<Vec<_>>());
}

#[test]
fn saga_yasli_deger_bulunur() {
    // "Fatura Tarihi          12.04.2026" — iki nokta YOK, değer sağda.
    let satirlar: Vec<&str> = EFATURA.lines().collect();
    let a = adaylar(&satirlar, &["fatura tarihi".to_string()], "TARIH");
    assert!(a.iter().any(|x| x.deger == "12.04.2026"),
        "sağa yaslı tarih bulunamadı: {:?}", a.iter().map(|x| &x.deger).collect::<Vec<_>>());
}

#[test]
fn ayni_sablon_farkli_tutarlarda_ayni_izi_verir() {
    // Şablon kimliği TUTARLARA bağlı olmamalı; yoksa her fatura yeni şablon sayılır
    // ve öğrenme hiç birikmez.
    let ikinci = EFATURA.replace("8.000,00", "12.500,00").replace("1.600,00", "2.500,00")
        .replace("9.600,00", "15.000,00").replace("12.04.2026", "30.06.2026");
    assert_eq!(parmak_izi(EFATURA), parmak_izi(&ikinci),
        "aynı şablonun ikinci faturası farklı iz verdi — öğrenme birikmez");
    let baska = "MUTABAKAT\nŞirket unvani: X\nBorc bakiye: 1,00\n";
    assert_ne!(parmak_izi(EFATURA), parmak_izi(baska));
}

#[test]
fn onaydan_kural_ogrenilir_ve_uygulanir() {
    let id = format!("TEST-A-{}", parmak_izi(EFATURA));
    // Bellek DİSKE yazılıyor (yaşayan katmanın gereği) — test önceki koşuşun kalıntısına
    // takılmasın diye temiz başlar. Testler birbirinden bağımsız olmalı.
    let _ = futures::executor::block_on(sablon_unut(axum::extract::Path(id.clone())));
    let onaylar = vec![
        ("belge_no".to_string(), "ABC2026000000777".to_string()),
        ("matrah".to_string(), "8.000,00".to_string()),
    ];
    let n = ogren(&id, "ABC TİCARET", "FATURA", EFATURA, &onaylar);
    assert!(n >= 1, "onaylardan kural çıkarılmadı ({n})");

    // Aynı şablonun İKİNCİ faturası — tutarlar farklı, düzen aynı.
    let ikinci = EFATURA.replace("ABC2026000000777", "ABC2026000000888");
    let satirlar: Vec<&str> = ikinci.lines().collect();
    let alanlar = vec![("belge_no".to_string(), "METIN".to_string())];
    let c = uygula(&id, &satirlar, &alanlar);
    assert_eq!(c.get("belge_no").map(|s| s.as_str()), Some("ABC2026000000888"),
        "öğrenilmiş kural ikinci belgeye uygulanmadı: {c:?}");
    let _ = futures::executor::block_on(sablon_unut(axum::extract::Path(id)));
}

#[test]
fn belgede_bulunmayan_deger_kural_uretmez() {
    // Kullanıcı elle bir şey yazdıysa (belgede geçmiyorsa) ondan kural çıkarılmamalı —
    // uydurma çapa, sonraki belgede yanlış okuma demektir.
    let id = "TEST-UYDURMA".to_string();
    let _ = futures::executor::block_on(sablon_unut(axum::extract::Path(id.clone())));
    let n = ogren(&id, "X", "FATURA", EFATURA,
        &[("unvan".to_string(), "BELGEDE OLMAYAN FİRMA".to_string())]);
    assert_eq!(n, 0, "belgede geçmeyen değerden kural çıkarıldı");
    let _ = futures::executor::block_on(sablon_unut(axum::extract::Path(id)));
}

/// ÖRNEK TEKSTİL faturasından (taranmış): alıcı adı ile İRSALİYE bilgisi YAN YANA
/// iki sütunda. Değer, etiketin bulunduğu HÜCREYLE sınırlı olmalı.
#[test]
fn yan_sutun_unvana_karismaz() {
    let satirlar = vec![
        "SAYIN: Ayşe YILMAZ   irsaliye Tarihi : 10.02.2006",
        "Örnek Caddesi Nu: 125   irsaliye Nu :45678",
    ];
    let a = adaylar(&satirlar, &["sayin".to_string()], "METIN");
    let ilk = &a.first().expect("aday yok").deger;
    assert_eq!(ilk, "Ayşe YILMAZ",
        "yan sütun ünvana karıştı — adaylar: {:?}",
        a.iter().map(|x| (&x.deger, &x.strateji, x.puan)).collect::<Vec<_>>());
}

/// ÖRNEK TEKSTİL faturasının TARAMA çıktısındaki toplam satırları — gerçek OCR metni.
#[test]
fn tarama_toplam_satirlari_okunur() {
    let satirlar = vec![
        "TOPLAM   3.200,00",
        "isK %",
        "NET TUTAR   3.200,00",
        "KDV % 8   256,00",
        "Yalniz: Ucbindortyuzellialti.liradir.   GENEL TOPLAM   3.456,00",
    ];
    let t = adaylar(&satirlar, &["genel toplam".into(), "toplam".into()], "TUTAR");
    assert!(t.iter().any(|x| x.deger == "3.456,00"),
        "genel toplam bulunamadı: {:?}", t.iter().map(|x| (&x.deger, &x.strateji)).collect::<Vec<_>>());
    let m = adaylar(&satirlar, &["net tutar".into(), "matrah".into()], "TUTAR");
    assert!(m.iter().any(|x| x.deger == "3.200,00"),
        "matrah bulunamadı: {:?}", m.iter().map(|x| (&x.deger, &x.strateji)).collect::<Vec<_>>());
    let k = adaylar(&satirlar, &["kdv".into(), "kdv tutari".into()], "TUTAR");
    assert!(k.iter().any(|x| x.deger == "256,00"),
        "kdv bulunamadı: {:?}", k.iter().map(|x| (&x.deger, &x.strateji)).collect::<Vec<_>>());
}

/// **ÇAPA DEĞERİN KENDİSİNİ İÇERMEMELİ.**
///
/// Çapa satırın tamamından kurulunca değer de çapaya giriyordu:
/// `"SAYIN: EGE TEKSTİL A.Ş."` → çapa `"sayin ege tekstil a s"`. Müşteri adı çapanın
/// parçası olduğu için, aynı şablonun **farklı müşterili** bir sonraki faturasında kural
/// hiç eşleşmiyordu — her fatura yeni kural yazıyor, hiçbiri tekrar kullanılmıyordu.
/// Şablon öğrenmenin pratikte birikmemesinin başlıca sebebi buydu.
#[test]
fn capa_degeri_icermez_farkli_musteride_de_eslesir() {
    let id = "TEST-CAPA".to_string();
    let unut = |i: String| { let _ = futures::executor::block_on(sablon_unut(axum::extract::Path(i))); };
    unut(id.clone());

    let birinci = "\
ABC TİCARET A.Ş.                         e-FATURA
SAYIN: EGE TEKSTİL SANAYİ A.Ş.
Fatura No                                ABC2026000000777
Matrah                                   8.000,00";
    let n = ogren(&id, "EGE TEKSTİL SANAYİ A.Ş.", "FATURA", birinci,
        &[("unvan".to_string(), "EGE TEKSTİL SANAYİ A.Ş.".to_string())]);
    assert!(n >= 1, "kural öğrenilmedi ({n})");

    // AYNI şablon, BAŞKA müşteri. Çapa "sayin" olmalı ki bu belgede de tutsun.
    let ikinci = birinci.replace("EGE TEKSTİL SANAYİ A.Ş.", "MARMARA LOJİSTİK LTD.");
    let satirlar: Vec<&str> = ikinci.lines().collect();
    let alanlar = vec![("unvan".to_string(), "METIN".to_string())];
    let c = uygula(&id, &satirlar, &alanlar);

    let bulunan = c.get("unvan").map(|s| s.as_str()).unwrap_or("");
    assert!(bulunan.contains("MARMARA"),
        "öğrenilen kural farklı müşteride eşleşmedi — çapa değeri içeriyor. bulunan: {bulunan:?}, tüm: {c:?}");
    unut(id);
}

// ─────────────────────────────────────────────────────────────────────────────
// KİRLİ KURAL: silinmez, okunur
// ─────────────────────────────────────────────────────────────────────────────

/// **Wilson alt sınırı: az gözlem yüksek güven vermez.**
/// Nokta tahmini 1/1 ile 50/50'yi aynı gösterir; ilki hiçbir şey kanıtlamaz.
#[test]
fn guven_az_gozlemde_yuksek_cikmaz() {
    let k = |i, d| Kural { alan: "x".into(), strateji: "s".into(), capa: "c".into(),
        onek: String::new(), isabet: i, deneme: d, durum: "AKTIF".into(),
        karsi_ornek: String::new(), emekli_neden: String::new() };
    let g1 = guven(&k(1, 1));
    let g10 = guven(&k(10, 10));
    assert!(g1 < 0.35, "1/1 fazla güvenli çıktı: {g1:.2}");
    assert!(g10 > g1, "10/10 ({g10:.2}) 1/1'den ({g1:.2}) yüksek olmalı");
    assert!(guven(&k(1, 2)) < 0.2, "1 isabet 1 hata düşük olmalı");
    assert_eq!(guven(&k(0, 0)), 0.0);
}

/// **KİRLİ KURAL SİLİNMEZ.** Denklem reddedince kural emekli olur ama bellekte kalır,
/// ne ürettiği (`karsi_ornek`) ve neden emekli olduğu yazılır.
#[test]
fn reddedilen_kural_emekli_olur_ama_silinmez() {
    let id = "TEST-KIRLI".to_string();
    let unut = |i: String| { let _ = futures::executor::block_on(sablon_unut(axum::extract::Path(i))); };
    unut(id.clone());

    let metin = "FATURA\nSAYIN: EGE TEKSTİL A.Ş.\nMatrah   1.000,00";
    ogren(&id, "EGE", "FATURA", metin,
        &[("unvan".to_string(), "EGE TEKSTİL A.Ş.".to_string())]);

    // Hakem üç kez reddediyor → emekli. (Emekli olduktan sonraki çağrı AKTİF kural
    // bulamaz ve false döner — doğru davranış; bu yüzden dönüşler biriktirilir.)
    let mut emekli = false;
    for _ in 0..3 { emekli |= kural_geri_besle(&id, "unvan", false, "EGE TEKSTİL A.Ş. Vergi No"); }
    assert!(emekli, "3 redde rağmen emekli olmadı");

    // SİLİNMEDİ: bellekte duruyor ve karşı örneği taşıyor.
    let b = bellek().lock().unwrap();
    let k = b.sablonlar[&id].kurallar.iter().find(|k| k.alan == "unvan").expect("kural SİLİNDİ");
    assert_eq!(k.durum, "EMEKLI");
    assert!(k.karsi_ornek.contains("Vergi No"), "karşı örnek kaydedilmedi: {:?}", k.karsi_ornek);
    assert!(!k.emekli_neden.is_empty(), "emekli gerekçesi yok");
    drop(b);
    unut(id);
}

/// **Emekli kural değer ÜRETMEZ** ama **aynı kural yeniden ÖĞRENİLMEZ** de —
/// yoksa her belge aynı yanlışı yazar, motor hiç ilerlemez.
#[test]
fn emekli_kural_ne_uygulanir_ne_yeniden_ogrenilir() {
    let id = "TEST-KIRLI2".to_string();
    let unut = |i: String| { let _ = futures::executor::block_on(sablon_unut(axum::extract::Path(i))); };
    unut(id.clone());

    let metin = "FATURA\nSAYIN: EGE TEKSTİL A.Ş.\nMatrah   1.000,00";
    ogren(&id, "EGE", "FATURA", metin, &[("unvan".to_string(), "EGE TEKSTİL A.Ş.".to_string())]);
    for _ in 0..3 { kural_geri_besle(&id, "unvan", false, "kirli değer"); }

    // 1) Uygulanmıyor
    let satirlar: Vec<&str> = metin.lines().collect();
    let c = uygula(&id, &satirlar, &[("unvan".to_string(), "METIN".to_string())]);
    assert!(c.get("unvan").is_none(), "emekli kural yine de değer üretti: {c:?}");

    // 2) Yeniden öğrenilmiyor — aynı belge tekrar onaylansa bile
    let onceki = { bellek().lock().unwrap().sablonlar[&id].kurallar.len() };
    ogren(&id, "EGE", "FATURA", metin, &[("unvan".to_string(), "EGE TEKSTİL A.Ş.".to_string())]);
    let sonraki = { bellek().lock().unwrap().sablonlar[&id].kurallar.len() };
    assert_eq!(onceki, sonraki, "emekli kural yeniden öğrenildi — sonsuz döngü");
    let hala_emekli = { bellek().lock().unwrap().sablonlar[&id]
        .kurallar.iter().any(|k| k.alan == "unvan" && k.durum == "EMEKLI") };
    assert!(hala_emekli, "emekli kural dirildi");
    unut(id);
}

/// **Yeni kural eskisini EZMEZ** — yan yana dururlar, en güvenilir olan kazanır.
/// Eskiden tek yanlış onay, kanıtlanmış bir kuralı yok ediyor ve itibarı sıfırlıyordu.
#[test]
fn yeni_kural_kanitlanmis_kurali_ezmez() {
    let id = "TEST-EZME".to_string();
    let unut = |i: String| { let _ = futures::executor::block_on(sablon_unut(axum::extract::Path(i))); };
    unut(id.clone());

    let m1 = "FATURA\nSAYIN: EGE TEKSTİL A.Ş.\nMatrah   1.000,00";
    for _ in 0..4 { ogren(&id, "EGE", "FATURA", m1, &[("unvan".to_string(), "EGE TEKSTİL A.Ş.".to_string())]); }

    // Başka çapalı bir belge aynı alanı öğretiyor.
    let m2 = "FATURA\nAlıcı Ünvanı: MARMARA LOJİSTİK\nMatrah   2.000,00";
    ogren(&id, "MARMARA", "FATURA", m2, &[("unvan".to_string(), "MARMARA LOJİSTİK".to_string())]);

    let b = bellek().lock().unwrap();
    let unvan_kurallari: Vec<_> = b.sablonlar[&id].kurallar.iter().filter(|k| k.alan == "unvan").collect();
    assert!(unvan_kurallari.len() >= 2,
        "yeni kural eskisini ezdi — {} kural kaldı", unvan_kurallari.len());
    let en_iyi = unvan_kurallari.iter().max_by(|a, b|
        guven(a).partial_cmp(&guven(b)).unwrap()).unwrap();
    assert!(en_iyi.deneme >= 4, "kanıtlanmış kural (4 deneme) kaybedildi: {en_iyi:?}");
    drop(b);
    unut(id);
}
