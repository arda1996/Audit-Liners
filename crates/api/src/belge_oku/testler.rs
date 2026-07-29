//! BELGE OKUMA TESTLERİ.
//!
//! İki tür test var:
//!   1. **Birim** — sayı biçimi ve yazıyla tutar, beş dilde, dilin kendi tuzaklarıyla.
//!   2. **Gerçek belge** — internetten alınan iki gerçek belgenin okunmuş değerleri sabit
//!      veri olarak gömüldü. Bunlar uydurulmuş örnek değil: biri 1889 tarihli el yazısı
//!      fatura (Pikes Peak Toll Road), diğeri 1999 tarihli Rusça tahsilat makbuzu.
//!      Motorun görevi, o belgelerdeki tutarlılığı YAKALAYABİLMEK.

use super::*;

// ═════════════════════════════════════════════════════════════════════════════
// SAYI BİÇİMİ — dil bilinmeden sayı okunamaz
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn ayni_metin_dile_gore_farkli_okunur() {
    // EN ASIL TUZAK: "1.234" iki dilde iki farklı sayıdır. Motor dili bilmeden okursa
    // 1000 kat hata yapar ve bunu hiç fark etmez.
    assert_eq!(sayi_ayristir("1.234", "tr"), Some(123_400), "TR'de 1.234 = bin iki yüz otuz dört");
    assert_eq!(sayi_ayristir("1.234", "en"), Some(123),     "EN'de 1.234 = bir tam 234 binde");
}

#[test]
fn yerel_sayi_bicimleri() {
    assert_eq!(sayi_ayristir("1.234,56", "tr"), Some(123_456));
    assert_eq!(sayi_ayristir("1.234,56 TL", "tr"), Some(123_456), "para simgesi atılmalı");
    assert_eq!(sayi_ayristir("1,234.56", "en"), Some(123_456));
    assert_eq!(sayi_ayristir("$1,234.56", "en"), Some(123_456));
    assert_eq!(sayi_ayristir("1.234,56", "de"), Some(123_456));
    assert_eq!(sayi_ayristir("1 234,56", "fr"), Some(123_456), "Fransızca binlik ayracı boşluk");
    assert_eq!(sayi_ayristir("1\u{00A0}234,56 €", "fr"), Some(123_456), "kırılmaz boşluk da ayraçtır");
    assert_eq!(sayi_ayristir("1 234,56", "ru"), Some(123_456));
    // Elle doldurulan belgede yazan kişi yerel ayracı kullanmayabilir — yine de okunmalı.
    assert_eq!(sayi_ayristir("102.177,36", "ru"), Some(10_217_736), "binlik nokta yazılmış Rusça tutar");
    assert_eq!(sayi_ayristir("-1.500,00", "tr"), Some(-150_000), "eksi korunmalı");
    assert_eq!(sayi_ayristir("okunamadı", "tr"), None, "rakam yoksa None — sıfır DEĞİL");
}

// ═════════════════════════════════════════════════════════════════════════════
// YAZIYLA TUTAR — muhasebecinin elle yaptığı çapraz kontrolün kendisi
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn yaziyla_turkce() {
    assert_eq!(yaziyla_ayristir("yüz iki bin yüz yetmiş yedi", "tr"), Some(102_177));
    assert_eq!(yaziyla_ayristir("YALNIZ ON BEŞ BİN TÜRK LİRASI", "tr"), Some(15_000),
        "büyük harf, 'yalnız' ve para adı yok sayılmalı");
    assert_eq!(yaziyla_ayristir("bir milyon iki yüz elli bin", "tr"), Some(1_250_000));
    assert_eq!(yaziyla_ayristir("bin", "tr"), Some(1_000), "'bir bin' denmez, 'bin' tek başına 1000");
    assert_eq!(yaziyla_ayristir("yüz", "tr"), Some(100));
    assert_eq!(yaziyla_ayristir("dokuz yüz doksan dokuz", "tr"), Some(999));
}

#[test]
fn yaziyla_ingilizce() {
    assert_eq!(yaziyla_ayristir("one hundred two thousand one hundred seventy-seven", "en"), Some(102_177));
    assert_eq!(yaziyla_ayristir("Say: Fifteen Thousand Dollars Only", "en"), Some(15_000));
    assert_eq!(yaziyla_ayristir("two million three hundred forty-five thousand", "en"), Some(2_345_000));
    assert_eq!(yaziyla_ayristir("nineteen", "en"), Some(19), "on'lu sayılar tek sözcük");
}

#[test]
fn yaziyla_almanca_birlesik_yazimi_cozer() {
    // Almanca'nın tuzağı: sayı TEK SÖZCÜK yazılır ve birler onlardan ÖNCE gelir.
    // "siebenundsiebzig" = sieben(7) + und + siebzig(70) = 77 — okuma sırası ters.
    assert_eq!(yaziyla_ayristir("siebenundsiebzig", "de"), Some(77));
    assert_eq!(yaziyla_ayristir("einhundertzweitausendeinhundertsiebenundsiebzig", "de"), Some(102_177));
    assert_eq!(yaziyla_ayristir("fünfzehntausend", "de"), Some(15_000));
    assert_eq!(yaziyla_ayristir("dreiundzwanzig", "de"), Some(23));
}

#[test]
fn yaziyla_fransizca_yetmis_seksen_doksan_tuzagi() {
    // Fransızca 70/80/90 için ayrı sözcük YOKTUR: 60+10, 4×20, 4×20+10 diye kurulur.
    assert_eq!(yaziyla_ayristir("soixante-dix", "fr"), Some(70), "60 + 10");
    assert_eq!(yaziyla_ayristir("quatre-vingts", "fr"), Some(80), "4 × 20 — TOPLAMA değil ÇARPMA");
    assert_eq!(yaziyla_ayristir("quatre-vingt-dix-sept", "fr"), Some(97), "4×20 + 10 + 7");
    assert_eq!(yaziyla_ayristir("quinze mille euros", "fr"), Some(15_000));
    assert_eq!(yaziyla_ayristir("deux cent trente", "fr"), Some(230));
}

#[test]
fn yaziyla_rusca_cekimli_bicimleri_cozer() {
    // Rusça'da sayı sözcükleri çekimlenir: тысяча / тысячи / тысяч hepsi 1000'dir.
    assert_eq!(yaziyla_ayristir("сто две тысячи сто семьдесят семь", "ru"), Some(102_177));
    assert_eq!(yaziyla_ayristir("пятнадцать тысяч рублей", "ru"), Some(15_000));
    assert_eq!(yaziyla_ayristir("двести тридцать", "ru"), Some(230), "двести hazır yüzlük (200)");
    assert_eq!(yaziyla_ayristir("один миллион", "ru"), Some(1_000_000));
}

#[test]
fn cozulemeyen_yazi_none_doner_uydurulmaz() {
    assert_eq!(yaziyla_ayristir("filanca falanca", "tr"), None);
    assert_eq!(yaziyla_ayristir("", "tr"), None);
    assert_eq!(yaziyla_ayristir("yüz", "xx"), None, "bilinmeyen dilde okuma denenmez");
}

// ═════════════════════════════════════════════════════════════════════════════
// DİL VE TÜR SEZME
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn dil_ve_tur_etiketlerden_sezilir() {
    let tr = "FATURA  Fatura No: ABC123  Tarih: 01.02.2026  Sayın: X A.Ş.  Matrah  KDV  Genel Toplam";
    let (d, p) = dil_sez(tr);
    assert_eq!(d, "tr", "Türkçe sezilemedi (puan {p})");
    assert_eq!(belge_turu(tr, "tr"), "FATURA");

    let en = "INVOICE  Invoice No: 55  Date: 01/02/2026  Bill To: ACME  Subtotal  VAT  Total";
    assert_eq!(dil_sez(en).0, "en");
    assert_eq!(belge_turu(en, "en"), "FATURA");

    let ru = "КВИТАНЦИЯ к приходному кассовому ордеру  Принято от  Основание  Сумма прописью";
    assert_eq!(dil_sez(ru).0, "ru");
    assert_eq!(belge_turu(ru, "ru"), "MAKBUZ");

    let de = "RECHNUNG  Rechnungsnummer  Datum  Kunde  Nettobetrag  MwSt  Gesamtbetrag";
    assert_eq!(dil_sez(de).0, "de");

    let fr = "FACTURE  Numéro de facture  Date  Client  Montant HT  TVA  Total TTC";
    assert_eq!(dil_sez(fr).0, "fr");
}

#[test]
fn taninmayan_dilde_varsayim_yapilmaz() {
    // Dil sezilemezse "tr" VARSAYILMAZ — yanlış dil, sayıyı da yanlış okur.
    let (d, p) = dil_sez("‡‡‡ 9999 ‡‡‡");
    assert!(d.is_empty() && p == 0, "tanınmayan belgede dil uyduruldu: {d}");
    let c = coz(&BelgeGirdi { metin: "‡‡‡".into(), toplam: "100".into(), ..Default::default() });
    assert!(c.bulgular.iter().any(|x| x.kod == "B01" && x.onem == "ENGEL"), "B01 verilmedi");
    assert!(!c.deftere_hazir);
}

#[test]
fn en_uzun_tur_eslesmesi_kazanir() {
    // "gider pusulası" içinde "fatura" geçmez ama benzer çakışmalarda uzun etiket kazanmalı.
    let m = "GİDER PUSULASI  Tarih  Mal/Hizmet Sunan  Brüt Tutar";
    assert_eq!(belge_turu(m, "tr"), "GIDER_PUSULASI");
}

// ═════════════════════════════════════════════════════════════════════════════
// ÇAPRAZ DOĞRULAMA — motorun asıl işi
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn rakam_yazi_uyusmazligi_engel_verir() {
    let g = BelgeGirdi {
        metin: "MAKBUZ Tarih Tahsilat makbuzu Yalnız".into(), dil: "tr".into(),
        tarih: "05.03.2026".into(), karsi_taraf: "ACME A.Ş.".into(),
        toplam: "15.000,00".into(),
        tutar_yaziyla: "on beş bin".into(),
        ..Default::default()
    };
    let c = coz(&g);
    assert!(c.bulgular.iter().any(|x| x.kod == "B00"), "tutan rakam-yazı doğrulanmadı");
    assert!(c.deftere_hazir, "doğrulanmış belge kayda hazır olmalıydı");

    // Şimdi yazıyı BOZ — motor yakalamalı ve kaydı ENGELLEMELİ.
    let bozuk = BelgeGirdi { tutar_yaziyla: "on beş bin beş yüz".into(), ..g };
    let c2 = coz(&bozuk);
    let b03 = c2.bulgular.iter().find(|x| x.kod == "B03").expect("B03 verilmedi — tahrifat kaçtı");
    assert_eq!(b03.onem, "ENGEL");
    assert!(!c2.deftere_hazir, "uyuşmayan belge kayda hazır sayıldı");
}

#[test]
fn zorunlu_alan_eksikse_kayit_dogmaz() {
    // VUK 227: belgesiz/eksik belgeyle kayıt yapılamaz.
    let c = coz(&BelgeGirdi {
        metin: "FATURA Fatura No Tarih Sayın Matrah KDV Genel Toplam".into(), dil: "tr".into(),
        toplam: "1.000,00".into(), // tarih ve karşı taraf YOK
        ..Default::default()
    });
    let b07 = c.bulgular.iter().find(|x| x.kod == "B07").expect("B07 verilmedi");
    assert_eq!(b07.onem, "ENGEL");
    assert!(b07.mesaj.contains("tarih"));
    assert!(!c.deftere_hazir);
}

#[test]
fn matrah_arti_vergi_toplami_tutmazsa_uyarir() {
    let c = coz(&BelgeGirdi {
        metin: "FATURA Tarih Sayın Matrah KDV Genel Toplam".into(), dil: "tr".into(),
        tarih: "01.02.2026".into(), karsi_taraf: "X".into(),
        matrah: "1.000,00".into(), vergi_tutari: "200,00".into(), vergi_orani: "%20".into(),
        toplam: "1.500,00".into(), // 1000 + 200 ≠ 1500
        ..Default::default()
    });
    assert!(c.bulgular.iter().any(|x| x.kod == "B05"), "toplam tutarsızlığı yakalanmadı");

    // Oran tutarsızlığı da ayrıca yakalanmalı.
    let c2 = coz(&BelgeGirdi {
        metin: "FATURA Tarih Sayın Matrah KDV Genel Toplam".into(), dil: "tr".into(),
        tarih: "01.02.2026".into(), karsi_taraf: "X".into(),
        matrah: "1.000,00".into(), vergi_tutari: "100,00".into(), vergi_orani: "%20".into(),
        toplam: "1.100,00".into(),
        ..Default::default()
    });
    assert!(c2.bulgular.iter().any(|x| x.kod == "B06"), "%20 oranla 200 çıkması gerekirken 100 kaçtı");
}

// ═════════════════════════════════════════════════════════════════════════════
// GERÇEK BELGE 1 — el yazısı fatura, Pikes Peak Toll Road, 19 Ocak 1889
// ═════════════════════════════════════════════════════════════════════════════
// Kaynak: Wikimedia Commons, kamuya açık. Aşağıdaki tutarlar görüntüden OKUNDU;
// hiçbiri toplamdan geri hesaplanmadı. Belgenin kendi "Forward" satırı 2101.07 yazıyor.
// Motorun sınavı: okumanın doğruluğunu ARİTMETİKLE kanıtlayabilmek.

#[test]
fn gercek_belge_1889_elyazisi_fatura_toplami_tutar() {
    let kalemler: Vec<String> = [
        "706.00", // Right of way (600.00 + 100.00 + 6.00)
        "6.00",   // Painting signs
        "8.00",   // Ticket stamps
        "12.00",  // Ticket Case
        "41.50",  // Printing Tickets
        "24.70",  // County Surveyor examining road
        "285.05", // Banquet to the "Press" (76.00 + 204.05 + 5.00)
        "13.00",  // Ticket Punches (1.00 + 12.00)
        "2.10",   // Notice in "Gazette"
        "10.75",  // County Surveyors Expenses
        "44.70",  // Medical Services E.L. Woods
        "4.25",   // Printing Pass Checks
        "41.80",  // Views of Road
        "901.22", // Engineering (11 alt kalem)
    ].iter().map(|s| s.to_string()).collect();

    let c = coz(&BelgeGirdi {
        // Belgede "Invoice" etiketi YOK (1889 tarihli antetli kağıt) — dil elle verilir.
        // Bu, sistemin gerçek bir sınırı: etiketsiz belgede dil sezilemez, sorulur.
        dil: "en".into(),
        tarih: "Jan 19 1889".into(),
        karsi_taraf: "Pikes Peak Toll Road".into(),
        toplam: "2101.07".into(),
        kalemler,
        ..Default::default()
    });

    assert_eq!(c.kalem_toplami, Some(210_107), "14 kalemin toplamı yanlış çözüldü");
    assert_eq!(c.toplam, Some(210_107));
    assert!(c.bulgular.iter().any(|x| x.kod == "B00" && x.mesaj.contains("oturdu")),
        "kalem–toplam mutabakatı doğrulanmadı: {:?}",
        c.bulgular.iter().map(|x| format!("{} {}", x.kod, x.mesaj)).collect::<Vec<_>>());
    assert!(c.bulgular.iter().all(|x| x.kod != "B04"), "yanlış tutarsızlık bulgusu üretildi");

    // Alt toplamların kendisi de okundu — onlar da tutmalı.
    let engineering = ["682.33","89.52","1.39","16.55","1.00","4.50","4.50","26.37","16.45","30.11","28.50"];
    let t: i64 = engineering.iter().filter_map(|x| sayi_ayristir(x, "en")).sum();
    assert_eq!(t, 90_122, "Engineering alt kalemleri 901.22 vermedi");
}

#[test]
fn gercek_belge_1889_bir_kalem_yanlis_okunursa_yakalanir() {
    // Asıl değer testi: bir hane yanlış okunduğunda motor bunu GÖRÜR mü?
    // 41.50 yerine 47.50 okunmuş olsun (1↔7 karışması — en sık el yazısı hatası).
    let mut kalemler: Vec<String> = ["706.00","6.00","8.00","12.00","47.50","24.70","285.05",
        "13.00","2.10","10.75","44.70","4.25","41.80","901.22"].iter().map(|s| s.to_string()).collect();
    let c = coz(&BelgeGirdi {
        dil: "en".into(), tarih: "Jan 19 1889".into(),
        karsi_taraf: "Pikes Peak Toll Road".into(),
        toplam: "2101.07".into(), kalemler: kalemler.clone(), ..Default::default()
    });
    let b04 = c.bulgular.iter().find(|x| x.kod == "B04").expect("yanlış okunan kalem KAÇTI");
    assert!(b04.mesaj.contains("6.00"), "fark tutarı bildirilmeli: {}", b04.mesaj);

    // Düzeltilince bulgu kalkmalı — motor gürültü üretmemeli.
    kalemler[4] = "41.50".into();
    let c2 = coz(&BelgeGirdi {
        dil: "en".into(), tarih: "Jan 19 1889".into(),
        karsi_taraf: "Pikes Peak Toll Road".into(),
        toplam: "2101.07".into(), kalemler, ..Default::default()
    });
    assert!(c2.bulgular.iter().all(|x| x.kod != "B04"));
}

// ═════════════════════════════════════════════════════════════════════════════
// GERÇEK BELGE 2 — Rusça tahsilat makbuzu, 03.12.1999
// ═════════════════════════════════════════════════════════════════════════════
// Kaynak: Wikimedia Commons, kamuya açık. Matbu form + el yazısı doldurma —
// gider pusulası / tahsilat makbuzu ile YAPISAL OLARAK AYNI durum.
// Belgede tutar hem rakamla (102.177,36) hem yazıyla yazılmış.

#[test]
fn gercek_belge_1999_rusca_makbuz_rakam_yazi_dogrulanir() {
    let metin = "КВИТАНЦИЯ к приходному кассовому ордеру № Принято от Основание Сумма прописью руб коп";
    assert_eq!(dil_sez(metin).0, "ru", "Rusça makbuz dili sezilemedi");
    assert_eq!(belge_turu(metin, "ru"), "MAKBUZ");

    let c = coz(&BelgeGirdi {
        metin: metin.into(),
        tarih: "03.12.1999".into(),
        // "Принято от" alanı yükleyen tarafından karartılmıştı — OKUNAMADI, uydurulmadı.
        karsi_taraf: String::new(),
        toplam: "102.177,36".into(),
        tutar_yaziyla: "сто две тысячи сто семьдесят семь".into(),
        para_birimi: "RUB".into(),
        ..Default::default()
    });

    assert_eq!(c.toplam, Some(10_217_736), "rakamla tutar yanlış çözüldü");
    assert_eq!(c.yaziyla_tutar, Some(10_217_700), "yazıyla tutar yanlış çözüldü");
    // Yazı yalnız tam kısmı söyler (36 kopek ayrı alanda) — motor bunu doğru saymalı.
    assert!(c.bulgular.iter().any(|x| x.kod == "B00" && x.mesaj.contains("doğruladı")),
        "rakam–yazı çapraz doğrulaması yapılmadı: {:?}",
        c.bulgular.iter().map(|x| format!("{} {}", x.kod, x.mesaj)).collect::<Vec<_>>());
    assert!(c.bulgular.iter().all(|x| x.kod != "B03"), "yanlış uyuşmazlık bulgusu");

    // İkinci halka dil olduğu bildirilmeli — kullanıcı kapsamı bilsin.
    assert!(c.bulgular.iter().any(|x| x.kod == "B09"), "ikinci halka dil uyarısı verilmedi");
    // Makbuzda karşı taraf zorunlu alan değil; tarih ve tutar var → kayda hazır.
    assert!(c.deftere_hazir, "doğrulanmış makbuz kayda hazır değil: guven={}", c.guven);
}

// ═════════════════════════════════════════════════════════════════════════════
// GÜVEN SKORU — doğrulanmamış okuma güven vermemeli
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn dogrulanmamis_okuma_yuksek_guven_almaz() {
    let temel = BelgeGirdi {
        metin: "FATURA Fatura No Tarih Sayın Matrah KDV Genel Toplam".into(), dil: "tr".into(),
        tarih: "01.02.2026".into(), karsi_taraf: "ACME A.Ş.".into(),
        toplam: "1.200,00".into(), ..Default::default()
    };
    let yalin = coz(&temel);

    // Aynı belge, artı çapraz doğrulanabilir alanlar → güven belirgin ARTMALI.
    let dogrulanmis = coz(&BelgeGirdi {
        matrah: "1.000,00".into(), vergi_tutari: "200,00".into(), vergi_orani: "%20".into(),
        tutar_yaziyla: "bin iki yüz".into(),
        kalemler: vec!["600,00".into(), "400,00".into()],
        ..temel
    });

    assert!(dogrulanmis.guven > yalin.guven + 25,
        "çapraz doğrulama güveni yeterince artırmadı: {} → {}", yalin.guven, dogrulanmis.guven);
    assert!(dogrulanmis.deftere_hazir);
    assert!(!yalin.deftere_hazir, "doğrulanmamış okuma kayda hazır sayıldı (guven {})", yalin.guven);
}
