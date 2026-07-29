//! TERİM DÜZELTME TESTLERİ.
//!
//! Bir düzelticinin asıl sınavı ne düzelttiği değil, **neye dokunmadığıdır.**
//! Fazla hevesli bir düzeltici, muhasebede sessiz veri bozulması demektir.

use super::*;

// ═════════════════════════════════════════════════════════════════════════════
// DOKUNULMAYANLAR — düzelticinin en önemli işi
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn sayilara_asla_dokunulmaz() {
    // 1889 faturasında bir hane yanlış okunmuştu (41.50 → 47.50). Bir düzeltici bunu
    // "en yakın değere" çekmeye kalkarsa hatayı DÜZELTMEZ, UYDURUR.
    for sayi in ["41.50", "1.234,56", "102.177,36", "8.0", "-1.500,00", "2101.07"] {
        let d = duzelt(sayi, "tr", "FATURA");
        assert_eq!(d.durum, "SAYI", "{sayi} sayı olarak tanınmadı");
        assert!(d.oneri.is_none(), "{sayi} için düzeltme önerildi — sayı uydurma riski");
        assert!(d.aciklama.contains("aritmetik"), "sayının nasıl doğrulanacağı söylenmeli");
    }
}

#[test]
fn ozel_isimler_duzeltilmez() {
    // Okuduğum belgelerdeki gerçek isimler. Bunlar kapalı bir kümeden GELMEZ:
    // "en yakın terime" çekmek firma ünvanını değiştirmek, yani yanlış cariye kayıt demektir.
    for isim in ["Ann Forbes", "Charles May", "Reichmann", "Bréville", "Woodcock"] {
        let d = duzelt(isim, "en", "");
        assert!(d.oneri.is_none(),
            "özel isim '{isim}' için '{}' önerildi — ünvan tahrifi",
            d.oneri.map(|o| o.oneri).unwrap_or_default());
        assert_eq!(d.durum, "BILINMIYOR");
        assert!(d.aciklama.contains("ÖZEL İSİM"), "kullanıcıya sebebi söylenmeli");
    }
}

#[test]
fn uzak_sozcuk_zorla_eslestirilmez() {
    // Eşik olmasaydı her sözcük bir terime çekilirdi. "Kesa" → "Kasa" olabilir,
    // ama "Merhaba" hiçbir şey olmamalı.
    let d = duzelt("Merhaba", "tr", "FATURA");
    assert!(d.oneri.is_none(), "alakasız sözcük terime çekildi: {:?}", d.oneri.map(|o| o.oneri));
}

// ═════════════════════════════════════════════════════════════════════════════
// DÜZELTİLENLER — gerçek el yazısı hataları
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn bozuk_terim_dogru_terime_baglanir() {
    let vakalar = [
        ("Matrih",       "Matrah"),
        ("Matrha",       "Matrah"),        // bitişik harf yer değiştirmesi (Damerau)
        ("Tutur",        "Tutar"),         // tek sözcüklü etiket
        ("Tarlh",        "Tarih"),         // i → l karışması
        ("Tevklfat",     "Tevkifat"),      // i → l karışması
        ("Fatura N0",    "Fatura No"),     // O → 0 karışması
        ("Genel Toolam", "Genel Toplam"),
    ];
    for (bozuk, beklenen) in vakalar {
        let d = duzelt(bozuk, "tr", "FATURA");
        let o = d.oneri.as_ref()
            .unwrap_or_else(|| panic!("'{bozuk}' için öneri yok (durum {})", d.durum));
        assert_eq!(o.oneri, beklenen, "'{bozuk}' yanlış terime bağlandı");
        assert!(o.benzerlik >= 60, "'{bozuk}' benzerliği düşük: {}", o.benzerlik);
    }
}

#[test]
fn turkce_aksan_farki_ceza_almaz() {
    // Elle doldurulan belgede yazan da okuyan da aksanı atlar. "Odenecek" ile "Ödenecek"
    // AYNI sözcüktür — bu fark hiç mesafe üretmemeli.
    assert_eq!(uzaklik(&sade("Odenecek Tutar"), &sade("Ödenecek Tutar")), 0.0);
    assert_eq!(uzaklik(&sade("Ilgili"), &sade("İlgili")), 0.0);
    assert_eq!(uzaklik(&sade("Ihracat"), &sade("İhracat")), 0.0);

    let d = duzelt("Odenecek Tutar", "tr", "FATURA");
    assert_eq!(d.tam_eslesme.as_deref(), Some("Ödenecek Tutar"),
        "aksansız yazım tam eşleşme saymalı, düzeltme bile gerekmemeli");
}

#[test]
fn gorsel_karisma_duz_hatadan_ucuzdur() {
    // "0" ile "O" el yazısında ayırt edilemez; "0" ile "X" edilir. İkisi düz Levenshtein'da
    // aynı mesafede olurdu — bu ayrım olmadan doğru aday seçilemez.
    let karisan = uzaklik("n0", "no");
    let karismayan = uzaklik("nx", "no");
    assert!(karisan < karismayan, "görsel karışma ucuzlatılmamış: {karisan} vs {karismayan}");
    assert!(karisan <= 0.5, "karışan çift fazla pahalı: {karisan}");
}

// ═════════════════════════════════════════════════════════════════════════════
// BELİRSİZLİK — tahmin yürütmeme
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn esit_yakinlikta_secim_yapilmaz() {
    // "Borç" ve "Boro" gibi eşit uzaklıkta adaylar varsa motor SUSMALI.
    // Dekont bağlamında "Borç" ve "Alacak" var; "Krgdit" gibi bir okuma için
    // tek net aday olmalı — ama eşitlik doğduğunda seçim yapılmamalı.
    let d = duzelt("Bebit", "en", "DEKONT"); // Debit ve Credit'e yakın olabilir
    if d.durum == "BELIRSIZ" {
        assert!(d.belirsiz.len() > 1);
        assert!(d.oneri.is_none(), "belirsizken öneri verildi");
        assert!(d.aciklama.contains("beterdir"));
    } else {
        // Tek aday çıktıysa da kabul — ama sessizce yanlış seçim OLMAMALI.
        assert!(d.oneri.is_none() || d.oneri.as_ref().unwrap().benzerlik >= 60);
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// BAĞLAM — dekontta fatura terimi aranmaz
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn baglam_alakasiz_terimleri_eler() {
    // "Tevkifat" bir FATURA terimidir; banka dekontunda aranmaz.
    let fatura = duzelt("Tevklfat", "tr", "FATURA");
    assert_eq!(fatura.oneri.as_ref().map(|o| o.oneri.as_str()), Some("Tevkifat"));

    let dekont = duzelt("Tevklfat", "tr", "DEKONT");
    assert!(dekont.oneri.is_none() || dekont.oneri.as_ref().unwrap().oneri != "Tevkifat",
        "dekont bağlamında fatura terimi önerildi");
}

#[test]
fn terim_hesap_kodunu_da_getirir() {
    // Düzeltmenin asıl değeri: terim tanınınca HANGİ HESABA gideceği de belli olur.
    let d = duzelt("Hesaplanan KDV", "tr", "FATURA");
    assert_eq!(d.tam_eslesme.as_deref(), Some("Hesaplanan KDV"));

    let bozuk = duzelt("Hesaplanan KOV", "tr", "FATURA");
    let o = bozuk.oneri.expect("öneri yok");
    assert_eq!(o.oneri, "Hesaplanan KDV");
    assert_eq!(o.hesap.as_deref(), Some("391"), "terim hesabıyla birlikte dönmeli");
    assert!(o.gerekce.contains("391"));
}

// ═════════════════════════════════════════════════════════════════════════════
// ÇOK DİLLİ — TR/EN çekirdek, DE/FR/RU ikinci halka
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn bes_dilde_de_calisir() {
    let vakalar = [
        ("tr", "Tevklfat",   "Tevkifat"),
        ("en", "Subtotai",   "Subtotal"),
        ("en", "Batance",    "Balance"),
        ("de", "Gesamtbetrqg", "Gesamtbetrag"),
        ("fr", "Facturc",    "Facture"),
        ("ru", "Квитанциа",  "Квитанция"),
    ];
    for (dil, bozuk, beklenen) in vakalar {
        let d = duzelt(bozuk, dil, "");
        let o = d.oneri.as_ref()
            .unwrap_or_else(|| panic!("[{dil}] '{bozuk}' için öneri yok (durum {})", d.durum));
        assert_eq!(o.oneri, beklenen, "[{dil}] '{bozuk}' yanlış bağlandı");
    }
}

#[test]
fn gercek_belgelerden_okunan_terimler_sozlukte_var() {
    // Okuduğum belgelerde geçen gerçek terimler tam eşleşmeli — sözlük gerçek belgeyi
    // karşılamıyorsa düzeltici de işe yaramaz.
    let vakalar = [
        ("en", "Forward"),          // 1889 faturası, devir satırı
        ("en", "Sundry Expenses"),  // 1889 faturası
        ("en", "Labor"),            // 1889 faturası
        ("en", "Settled"),          // 1828 eczane hesabı
        ("en", "Bought of"),        // 1828 eczane hesabı antedi
        ("ru", "Принято от"),       // 1999 makbuzu
        ("ru", "Основание"),        // 1999 makbuzu
        ("ru", "Сумма прописью"),   // 1999 makbuzu
        ("ru", "Главный бухгалтер"),// 1999 makbuzu
    ];
    for (dil, terim) in vakalar {
        let d = duzelt(terim, dil, "");
        assert_eq!(d.tam_eslesme.as_deref(), Some(terim),
            "[{dil}] gerçek belgede geçen '{terim}' sözlükte yok (durum {})", d.durum);
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// UZAKLIK ÖLÇÜSÜ
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn uzaklik_temel_ozellikler() {
    assert_eq!(uzaklik("matrah", "matrah"), 0.0, "aynı sözcük 0 uzaklıkta");
    assert_eq!(uzaklik("", "abc"), 3.0);
    assert_eq!(uzaklik("abc", ""), 3.0);
    // Yer değiştirme tek hata sayılır (iki değil).
    assert_eq!(uzaklik("matrha", "matrah"), 1.0);
    // Simetri
    assert_eq!(uzaklik("kasa", "kesa"), uzaklik("kesa", "kasa"));
}

/// Gerçek faturadan: "Yalnız: Üçbindörtyüzellialtı.liradır." — Türkçede yazıyla tutar
/// BİTİŞİK yazılır. Boşluğa göre bölen bir çözümleyici bunu hiç okuyamaz.
#[test]
fn turkce_bitisik_yaziyla_tutar_cozulur() {
    use crate::belge_oku::yaziyla_ayristir;
    assert_eq!(yaziyla_ayristir("Üçbindörtyüzellialtı", "tr"), Some(3456));
    assert_eq!(yaziyla_ayristir("Yalnız: Üçbindörtyüzellialtı.liradır.", "tr"), Some(3456));
    assert_eq!(yaziyla_ayristir("onbeşbin", "tr"), Some(15_000));
    assert_eq!(yaziyla_ayristir("ikiyüzelli", "tr"), Some(250));
}
