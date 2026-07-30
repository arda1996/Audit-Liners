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
