//! BELGE ALIM AKIŞI TESTLERİ — üç modun her biri, ve aralarındaki sözleşme.
//!
//! Asıl sınanan şey: **hangi yoldan girilirse girilsin aynı güvence**. Otomatik mod
//! kanıtsız doldurmuyor mu, seçimli mod onaysız ilerlemiyor mu, manuel mod çelişkiyi
//! sessizce yutmuyor mu.

use super::*;

fn coz(v: &serde_json::Value, yol: &str) -> serde_json::Value { v.pointer(yol).cloned().unwrap_or(serde_json::Value::Null) }

async fn basla(tur: &str, mod_: &str, metin: &str, id: &str) -> serde_json::Value {
    alim_basla(Json(BaslaGirdi {
        belge_turu: tur.into(), dil: "tr".into(), mod_: mod_.into(),
        metin: metin.into(), oturum_id: id.into(),
    })).await.0
}
async fn deger(id: &str, kod: &str, ham: &str, kaynak: &str) -> serde_json::Value {
    alim_deger(Json(DegerGirdi {
        oturum_id: id.into(), kod: kod.into(), ham: ham.into(), kaynak: kaynak.into(),
    })).await.0
}
async fn onayla(id: &str, kod: &str) -> serde_json::Value {
    alim_onayla(Json(OnayGirdi { oturum_id: id.into(), kod: kod.into() })).await.0
}
async fn tamamla(id: &str) -> serde_json::Value {
    alim_tamamla(Json(OnayGirdi { oturum_id: id.into(), kod: String::new() })).await.0
}

/// Kullanıcının örneğindeki mutabakat belgesi — metin katmanlı, denklemi kapanan.
const MUTABAKAT_DIJITAL: &str = "\
CARİ MUTABAKAT BİLDİRİMİ
Şirket ünvanı: ANADOLU DEMİR ÇELİK A.Ş.
VKN / TCKN: 1234567890
Borç bakiye: 45.000,00
Alacak bakiye: 12.500,00
Net bakiye: 32.500,00
Tarih: 31.12.2026";

// ═════════════════════════════════════════════════════════════════════════════
// PARAMETRE SÖZLEŞMESİ — arayüz bu sıraya göre soracak
// ═════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn parametreler_siraya_gore_gelir() {
    let y = basla("MUTABAKAT", "SECIMLI", "", "T-SIRA").await;
    let alanlar = coz(&y, "/oturum/alanlar");
    let liste = alanlar.as_array().expect("alan listesi yok");
    let kodlar: Vec<&str> = liste.iter().filter_map(|a| a["kod"].as_str()).collect();
    assert_eq!(kodlar, vec!["unvan", "vkn", "borc_bakiye", "alacak_bakiye", "net_bakiye", "tarih"],
        "parametreler kullanıcıya SIRAYLA sorulmalı — sıra sözleşmenin parçası");

    // Arayüz kullanıcıyı ilk alana yönlendirmeli.
    assert_eq!(coz(&y, "/aktif_alan/kod").as_str(), Some("unvan"));
    assert!(coz(&y, "/aktif_alan/ipucu").as_str().unwrap_or("").len() > 5,
        "her alan için kullanıcıya ipucu verilmeli — 'ne seçeceğim' sorusu kalmamalı");
}

// ═════════════════════════════════════════════════════════════════════════════
// MOD 1 — OTOMATİK: kanıtsız doldurma YOK
// ═════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn otomatik_mod_denklem_tutuyorsa_doldurur() {
    let y = basla("MUTABAKAT", "OTOMATIK", MUTABAKAT_DIJITAL, "T-OTO").await;
    assert_eq!(coz(&y, "/otomatik/doldurma").as_str(), Some("YAPILDI"),
        "denklem tutan dijital belgede otomatik doldurma yapılmalıydı: {:?}", coz(&y, "/otomatik"));

    let alanlar = coz(&y, "/oturum/alanlar");
    let bul = |kod: &str| -> i64 {
        alanlar.as_array().unwrap().iter().find(|a| a["kod"] == kod)
            .and_then(|a| a["deger"].as_i64()).unwrap_or(-1)
    };
    assert_eq!(bul("borc_bakiye"), 4_500_000, "borç bakiye kuruş olarak yanlış");
    assert_eq!(bul("alacak_bakiye"), 1_250_000);
    assert_eq!(bul("net_bakiye"), 3_250_000);
    assert_eq!(bul("tarih"), 20261231, "tarih yyyyaagg olmalı");

    // DENKLEMLE KANITLANAN alanlar onaylı sayılır — kullanıcı tek tek onaylamak zorunda değil.
    // Ama HER otomatik alan değil: belgede birden çok aday bulunan alan (ör. iki VKN)
    // doldurulur ama onay kullanıcıya bırakılır. Denklem "borç−alacak=net" üçlüsünü
    // kanıtlar, başka alanları kanıtlamaz.
    for kod in ["borc_bakiye", "alacak_bakiye", "net_bakiye"] {
        let a = alanlar.as_array().unwrap().iter().find(|a| a["kod"] == kod).unwrap();
        assert_eq!(a["onaylandi"], true, "{kod} denklemin parçası — otomatik onaylanmalıydı");
    }

    // Denklem dışı alanlar (ünvan gibi) belirsizse onay kullanıcıdadır — akış bunu gerektirir.
    // Onaylanmadan tamamlanmaması ZORUNLU: eksik/onaysız belgeyle kayıt açılamaz (VUK 227).
    let once = tamamla("T-OTO").await;
    assert_eq!(coz(&once, "/tamamlandi").as_bool(), Some(false),
        "onaysız zorunlu alan varken tamamlandı — VUK 227 ihlali");

    for kod in ["unvan", "vkn", "tarih"] { onayla("T-OTO", kod).await; }
    let t = tamamla("T-OTO").await;
    assert_eq!(coz(&t, "/tamamlandi").as_bool(), Some(true), "bulgular: {:?}", coz(&t, "/bulgular"));
}

#[tokio::test]
async fn otomatik_mod_denklem_tutmuyorsa_HIC_DOLDURMAZ() {
    // ASIL KURAL: sıfır hata, "iyi tahmin" ile değil "kanıtsız yazmamak" ile sağlanır.
    // Net bakiye bilerek yanlış (32.500 yerine 30.000) — denklem tutmaz.
    let bozuk = MUTABAKAT_DIJITAL.replace("Net bakiye: 32.500,00", "Net bakiye: 30.000,00");
    let y = basla("MUTABAKAT", "OTOMATIK", &bozuk, "T-OTO-BOZUK").await;

    assert_eq!(coz(&y, "/otomatik/doldurma").as_str(), Some("YAPILMADI"),
        "denklem tutmadığı halde otomatik dolduruldu — sıfır hata güvencesi kırıldı");
    let alanlar = coz(&y, "/oturum/alanlar");
    assert!(alanlar.as_array().unwrap().iter().all(|a| a["deger"].is_null()),
        "kanıtlanmamış değerler forma yazıldı — yarım dolu form, boş formdan tehlikelidir");
    assert!(coz(&y, "/otomatik/oneri").as_str().unwrap_or("").contains("SECIMLI"),
        "kullanıcıya ne yapacağı söylenmeli");

    let t = tamamla("T-OTO-BOZUK").await;
    assert_eq!(coz(&t, "/tamamlandi").as_bool(), Some(false), "kayıt açılmamalıydı");
}

// ═════════════════════════════════════════════════════════════════════════════
// MOD 2 — SEÇİMLİ: seç → gör → onayla
// ═════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn secimli_mod_her_deger_ayri_onaylanir() {
    let id = "T-SEC";
    basla("MUTABAKAT", "SECIMLI", "", id).await;

    // Kullanıcı belge üzerinde seçim yapıyor — değer yazılır ama ONAY BEKLER.
    let y = deger(id, "unvan", "ANADOLU DEMİR ÇELİK A.Ş.", "SECIM").await;
    assert_eq!(coz(&y, "/onay_bekliyor").as_bool(), Some(true));
    assert_eq!(coz(&y, "/alan/onaylandi").as_bool(), Some(false),
        "seçim tek başına onay değildir — kullanıcı gördüğünü doğrulamalı");

    // Onaylanmadan tamamlanamaz.
    let t = tamamla(id).await;
    assert_eq!(coz(&t, "/tamamlandi").as_bool(), Some(false));

    // Sırayla doldur ve onayla.
    for (kod, ham) in [("unvan", "ANADOLU DEMİR ÇELİK A.Ş."), ("vkn", "1234567890"),
                       ("borc_bakiye", "45.000,00"), ("alacak_bakiye", "12.500,00"),
                       ("net_bakiye", "32.500,00"), ("tarih", "31.12.2026")] {
        deger(id, kod, ham, "SECIM").await;
        let o = onayla(id, kod).await;
        assert!(coz(&o, "/hata").is_null(), "{kod} onaylanamadı: {:?}", coz(&o, "/hata"));
    }

    let t = tamamla(id).await;
    assert_eq!(coz(&t, "/tamamlandi").as_bool(), Some(true), "bulgular: {:?}", coz(&t, "/bulgular"));
    assert_eq!(coz(&t, "/denklem_tutan").as_u64(), coz(&t, "/denklem_toplam").as_u64());
}

#[tokio::test]
async fn secimli_modda_yanlis_secim_denklemle_yakalanir() {
    let id = "T-SEC-YANLIS";
    basla("MUTABAKAT", "SECIMLI", "", id).await;
    // Kullanıcı alacak bakiye yerine yanlış bir sayı seçti (belgedeki başka bir tutar).
    for (kod, ham) in [("unvan", "X A.Ş."), ("borc_bakiye", "45.000,00"),
                       ("alacak_bakiye", "9.900,00"), ("net_bakiye", "32.500,00"),
                       ("tarih", "31.12.2026")] {
        deger(id, kod, ham, "SECIM").await;
        onayla(id, kod).await;
    }
    let t = tamamla(id).await;
    let bulgular = coz(&t, "/bulgular");
    let a21 = bulgular.as_array().unwrap().iter().find(|b| b["kod"] == "A21")
        .expect("yanlış seçim denklemle yakalanmadı");
    assert_eq!(a21["onem"], "ENGEL");
    assert!(a21["mesaj"].as_str().unwrap().contains("fark"), "fark tutarı bildirilmeli");
    assert_eq!(coz(&t, "/tamamlandi").as_bool(), Some(false), "denklem tutmadan kayıt açıldı");
}

#[tokio::test]
async fn zorunlu_alan_bos_onaylanamaz() {
    let id = "T-BOS";
    basla("MUTABAKAT", "SECIMLI", "", id).await;
    let o = onayla(id, "unvan").await;
    assert!(coz(&o, "/hata").as_str().unwrap_or("").contains("zorunlu"),
        "boş zorunlu alan onaylandı (VUK 227 ihlali): {o:?}");
}

// ═════════════════════════════════════════════════════════════════════════════
// MOD 3 — MANUEL: taramayla çelişki UYARI üretir, sessiz geçilmez
// ═════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn manuel_giris_taramayla_celisirse_uyarir() {
    let id = "T-MAN";
    // Belge var ve taranıyor; kullanıcı yine de elle yazıyor.
    basla("MUTABAKAT", "MANUEL", MUTABAKAT_DIJITAL, id).await;

    // Belgede 45.000,00 yazıyor; kullanıcı 54.000,00 yazdı (rakam yer değiştirmesi).
    let y = deger(id, "borc_bakiye", "54.000,00", "MANUEL").await;
    let bulgular = coz(&y, "/bulgular");
    let a11 = bulgular.as_array().unwrap().iter().find(|b| b["kod"] == "A11")
        .expect("manuel giriş taramayla çeliştiği halde uyarı YOK");
    assert_eq!(a11["onem"], "UYARI", "çelişki ENGEL değil UYARI olmalı — tarama da yanılabilir");
    let m = a11["mesaj"].as_str().unwrap();
    assert!(m.contains("54000.00") && m.contains("45000.00"),
        "uyarı İKİ değeri de göstermeli ki kullanıcı karşılaştırabilsin: {m}");

    // Kullanıcı ısrar ederse kendi değeri geçerli olur — ama görerek onaylamış olur.
    let o = onayla(id, "borc_bakiye").await;
    assert!(coz(&o, "/hata").is_null());
}

#[tokio::test]
async fn manuel_giris_taramayla_uyusuyorsa_uyari_uretilmez() {
    let id = "T-MAN-OK";
    basla("MUTABAKAT", "MANUEL", MUTABAKAT_DIJITAL, id).await;
    let y = deger(id, "borc_bakiye", "45.000,00", "MANUEL").await;
    let bulgular = coz(&y, "/bulgular");
    assert!(bulgular.as_array().unwrap().iter().all(|b| b["kod"] != "A11"),
        "doğru değerde gereksiz uyarı üretildi — gürültü, gerçek uyarıyı görünmez kılar");
}

#[tokio::test]
async fn okunamayan_deger_kabul_edilmez() {
    let id = "T-COZULEMEZ";
    basla("MUTABAKAT", "SECIMLI", "", id).await;

    let y = deger(id, "borc_bakiye", "okunamadı", "SECIM").await;
    let bulgular = coz(&y, "/bulgular");
    assert!(bulgular.as_array().unwrap().iter().any(|b| b["kod"] == "A10" && b["onem"] == "ENGEL"),
        "sayıya çevrilemeyen değer kabul edildi");
    assert_eq!(coz(&y, "/onay_bekliyor").as_bool(), Some(false));

    // Eksik tarih de reddedilmeli — 2 haneli yıl belirsizdir.
    let y2 = deger(id, "tarih", "31.12.26", "SECIM").await;
    assert!(coz(&y2, "/bulgular").as_array().unwrap().iter().any(|b| b["kod"] == "A10"),
        "belirsiz tarih (2 haneli yıl) kabul edildi");
}

// ═════════════════════════════════════════════════════════════════════════════
// DİĞER BELGE TÜRLERİ — sözleşme genel, mutabakata özel değil
// ═════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn fatura_ve_gider_pusulasi_denklemleri_de_calisir() {
    // Fatura: matrah + KDV = toplam
    let id = "T-FAT";
    basla("FATURA", "SECIMLI", "", id).await;
    for (kod, ham) in [("yon", "SATIS"), ("unvan", "X A.Ş."), ("belge_no", "ABC2026000001"),
                       ("tarih", "01.02.2026"),
                       ("matrah", "10.000,00"), ("kdv", "2.000,00"), ("toplam", "12.000,00")] {
        deger(id, kod, ham, "SECIM").await;
        onayla(id, kod).await;
    }
    let t = tamamla(id).await;
    assert_eq!(coz(&t, "/tamamlandi").as_bool(), Some(true), "{:?}", coz(&t, "/bulgular"));

    // Gider pusulası: brüt − stopaj = net
    let id2 = "T-GP";
    basla("GIDER_PUSULASI", "SECIMLI", "", id2).await;
    for (kod, ham) in [("unvan", "Ali Veli"), ("vkn", "12345678901"), ("tarih", "05.03.2026"),
                       ("brut", "1.000,00"), ("stopaj", "200,00"), ("net", "700,00")] {
        deger(id2, kod, ham, "SECIM").await;
        onayla(id2, kod).await;
    }
    let t2 = tamamla(id2).await;
    assert_eq!(coz(&t2, "/tamamlandi").as_bool(), Some(false),
        "1.000 − 200 = 800, belgede 700 yazıyor — yakalanmalıydı");
    assert!(coz(&t2, "/bulgular").as_array().unwrap().iter().any(|b| b["kod"] == "A21"));
}

#[tokio::test]
async fn bilinmeyen_belge_turu_reddedilir() {
    let y = basla("UYDURMA_TUR", "SECIMLI", "", "T-YOK").await;
    assert!(coz(&y, "/hata").as_str().unwrap_or("").contains("bilinmeyen"));
    assert!(coz(&y, "/gecerli").as_array().map(|a| !a.is_empty()).unwrap_or(false),
        "geçerli türler listelenmeli");
}

// ═════════════════════════════════════════════════════════════════════════════
// FATURA → FİŞ — belgeden muhasebe kaydına
// ═════════════════════════════════════════════════════════════════════════════

async fn fatura_akisi(id: &str, alanlar: &[(&str, &str)]) -> serde_json::Value {
    basla("FATURA", "SECIMLI", "", id).await;
    for (kod, ham) in alanlar {
        deger(id, kod, ham, "SECIM").await;
        onayla(id, kod).await;
    }
    alim_fis_taslagi(Json(OnayGirdi { oturum_id: id.into(), kod: String::new() })).await.0
}

const ALIS: &[(&str, &str)] = &[("yon", "ALIS"), ("unvan", "EGE TEKSTİL A.Ş."), ("vkn", "1234567890"),
    ("belge_no", "EGE2026000123"), ("tarih", "15.03.2026"),
    ("matrah", "10.000,00"), ("kdv", "2.000,00"), ("toplam", "12.000,00")];

#[tokio::test]
async fn alis_faturasi_dogru_hesaplara_yazilir() {
    // Alışta mal STOĞA girer, KDV İNDİRİLİR, borç satıcıya doğar.
    let d = fatura_akisi("FT-ALIS", ALIS).await;
    let sat = coz(&d, "/fis/satirlar");
    let bul = |h: &str| sat.as_array().unwrap().iter()
        .find(|s| s["hesap"] == h).cloned().unwrap_or(serde_json::Value::Null);

    assert_eq!(bul("153")["borc"].as_i64(), Some(1_000_000), "mal alışı 153'e borç yazılmalı");
    assert_eq!(bul("191.06")["borc"].as_i64(), Some(200_000), "indirilecek KDV %20 muavini 06");
    assert_eq!(bul("320")["alacak"].as_i64(), Some(1_200_000), "borç satıcıya (320) alacak yazılmalı");
    assert_eq!(coz(&d, "/denge/denk").as_bool(), Some(true));
    assert_eq!(coz(&d, "/ozet/kdv_orani").as_i64(), Some(20), "oran matrahtan türetilmeli");
    assert_eq!(coz(&d, "/fis/dayanak_no").as_str(), Some("EGE2026000123"),
        "fişin dayanağı fatura numarası olmalı (VUK 227)");
    assert_eq!(coz(&d, "/fis/tarih").as_str(), Some("15.03.2026"), "fiş belge tarihini taşımalı");
}

#[tokio::test]
async fn satis_faturasi_dogru_hesaplara_yazilir() {
    // Satışta alacak cariden doğar, hasılat 600, KDV hesaplanır (391).
    let d = fatura_akisi("FT-SATIS", &[("yon", "SATIS"), ("unvan", "TOROS GIDA LTD."),
        ("belge_no", "ABC2026000045"), ("tarih", "20.03.2026"),
        ("matrah", "5.000,00"), ("kdv", "500,00"), ("toplam", "5.500,00")]).await;
    let sat = coz(&d, "/fis/satirlar");
    let bul = |h: &str| sat.as_array().unwrap().iter()
        .find(|s| s["hesap"] == h).cloned().unwrap_or(serde_json::Value::Null);

    assert_eq!(bul("120")["borc"].as_i64(), Some(550_000));
    assert_eq!(bul("600")["alacak"].as_i64(), Some(500_000));
    // %10 oranın muavini 04 — kod ARDIŞIKTIR, orandan türetilmez (katalogdan çözülür).
    assert_eq!(bul("391.04")["alacak"].as_i64(), Some(50_000), "hesaplanan KDV %10 muavini 04");
    assert_eq!(coz(&d, "/ozet/kdv_orani").as_i64(), Some(10));
    assert_eq!(coz(&d, "/denge/denk").as_bool(), Some(true));
}

#[tokio::test]
async fn denklem_tutmayan_faturadan_fis_uretilmez() {
    // 1.000 + 200 ≠ 1.500 — belge kendi içinde tutarsız, kayıt açılamaz.
    let d = fatura_akisi("FT-BOZUK", &[("yon", "SATIS"), ("unvan", "X"), ("belge_no", "N1"),
        ("tarih", "01.01.2026"), ("matrah", "1.000,00"), ("kdv", "200,00"), ("toplam", "1.500,00")]).await;
    assert!(coz(&d, "/hata").as_str().unwrap_or("").contains("tamamlanmadı"),
        "tutarsız belgeden fiş üretildi: {d:?}");
    assert!(coz(&d, "/fis").is_null());
}

#[tokio::test]
async fn gecersiz_yon_reddedilir() {
    let id = "FT-YON";
    basla("FATURA", "SECIMLI", "", id).await;
    let y = deger(id, "yon", "satis faturasi", "SECIM").await;
    let b = coz(&y, "/bulgular");
    let a13 = b.as_array().unwrap().iter().find(|x| x["kod"] == "A13")
        .expect("serbest metin yön olarak kabul edildi — fiş ters yöne kurulurdu");
    assert_eq!(a13["onem"], "ENGEL");
    assert!(a13["cozum"].as_str().unwrap().contains("SATIS"), "geçerli seçenekler söylenmeli");

    // Geçerli seçenek kabul edilmeli — SECENEK alanının sayısal karşılığı olmaması hata değildir.
    let y2 = deger(id, "yon", "ALIS", "SECIM").await;
    assert!(coz(&y2, "/bulgular").as_array().unwrap().iter().all(|x| x["onem"] != "ENGEL"),
        "geçerli seçenek reddedildi: {:?}", coz(&y2, "/bulgular"));
    assert_eq!(coz(&y2, "/onay_bekliyor").as_bool(), Some(true));
}

#[tokio::test]
async fn fatura_disi_belgeden_fis_istenirse_soylenir() {
    basla("MUTABAKAT", "SECIMLI", "", "FT-MUT").await;
    let d = alim_fis_taslagi(Json(OnayGirdi { oturum_id: "FT-MUT".into(), kod: String::new() })).await.0;
    assert!(coz(&d, "/hata").as_str().unwrap_or("").contains("FATURA"),
        "fatura dışı türde net mesaj verilmeli");
}

// ═════════════════════════════════════════════════════════════════════════════
// TARAMA SAĞLAMLIĞI — bozuk glif, eşanlamlı etiket, alan çakışması
// ═════════════════════════════════════════════════════════════════════════════

/// Gerçek e-Fatura PDF'lerinde gömülü font eşlemesi bozulabiliyor: "KDV tutarı" metin
/// katmanında "KDV tutar·" çıkabiliyor. Tam eşleşme arayan tarama bunu kaçırıyordu.
#[tokio::test]
async fn bozuk_glifli_etiket_yine_de_bulunur() {
    let metin = "FATURA\nFatura no: ABC1\nFatura tarihi: 12.04.2026\nSay·n: EGE TEKST·L A.·.\n\
                 Matrah: 8.000,00\nKDV tutar·: 1.600,00\nGenel toplam: 9.600,00";
    let y = basla("FATURA", "OTOMATIK", metin, "T-GLIF").await;
    assert_eq!(coz(&y, "/otomatik/doldurma").as_str(), Some("YAPILDI"),
        "bozuk glif yüzünden okuma düştü: {:?}", coz(&y, "/otomatik"));
    let alanlar = coz(&y, "/oturum/alanlar");
    let ham = |kod: &str| alanlar.as_array().unwrap().iter()
        .find(|a| a["kod"] == kod).and_then(|a| a["ham"].as_str()).unwrap_or("").to_string();
    assert_eq!(ham("kdv"), "1.600,00", "bozuk etiketli KDV satırı okunmadı");
    assert_eq!(ham("matrah"), "8.000,00");
}

/// Belge "Karşı taraf ünvanı" yazmaz; "Sayın" ya da "Alıcı" yazar. Eşanlamlılar
/// `belge-sozlugu.json`'dan gelir — yeni etiket eklemek kod değişikliği istemez.
#[tokio::test]
async fn etiket_esanlamlilari_kullanilir() {
    let metin = "FATURA\nSayın: TOROS GIDA LTD.\nFatura tarihi: 01.05.2026\n\
                 Matrah: 1.000,00\nKDV tutarı: 200,00\nGenel toplam: 1.200,00";
    let y = basla("FATURA", "OTOMATIK", metin, "T-ESANLAM").await;
    let alanlar = coz(&y, "/oturum/alanlar");
    let unvan = alanlar.as_array().unwrap().iter().find(|a| a["kod"] == "unvan").unwrap();
    assert_eq!(unvan["ham"].as_str(), Some("TOROS GIDA LTD."),
        "'Sayın' etiketi 'Karşı taraf ünvanı' alanına bağlanmadı");
}

/// İki alan aynı satırı KAPAMAMALI. "Tevkifata tabi KDV" genel KDV etiketlerini
/// paylaşırsa "KDV tutarı" satırını kapar; fiş sessizce yanlış kurulur
/// (cariye eksik borç, satıcıda sıfır KDV).
#[tokio::test]
async fn ayni_satir_iki_alana_yazilmaz() {
    let metin = "FATURA\nSayın: X A.Ş.\nFatura tarihi: 01.05.2026\n\
                 Matrah: 1.000,00\nKDV tutarı: 200,00\nGenel toplam: 1.200,00";
    let y = basla("FATURA", "OTOMATIK", metin, "T-CAKISMA").await;
    let alanlar = coz(&y, "/oturum/alanlar");
    let bul = |kod: &str| alanlar.as_array().unwrap().iter()
        .find(|a| a["kod"] == kod).and_then(|a| a["ham"].as_str()).unwrap_or("").to_string();
    assert_eq!(bul("kdv"), "200,00");
    assert_eq!(bul("tevkifat"), "",
        "tevkifat, KDV satırını kapmış — belgede tevkifat YOK, alan boş kalmalı");
}

// ═════════════════════════════════════════════════════════════════════════════
// TEVKİFAT (KDVK 9) — alıcı KDV'nin bir kısmını satıcıya ÖDEMEZ
// ═════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn tevkifatli_alista_sorumlu_sifatiyla_kdv_borcu_dogar() {
    let d = fatura_akisi("FT-TEV", &[("yon", "ALIS"), ("unvan", "ZETA İNŞAAT LTD."),
        ("belge_no", "ZET1"), ("tarih", "10.04.2026"), ("matrah", "10.000,00"),
        ("kdv", "2.000,00"), ("toplam", "12.000,00"), ("tevkifat", "1.000,00")]).await;
    let sat = coz(&d, "/fis/satirlar");
    let liste = sat.as_array().unwrap();

    // Cariye yalnız ödenecek tutar borçlanır: 12.000 − 1.000 = 11.000
    let s320 = liste.iter().find(|s| s["hesap"] == "320").unwrap();
    assert_eq!(s320["alacak"].as_i64(), Some(1_100_000),
        "tevkif edilen KDV satıcıya ödenmez — cari borcu düşük olmalı");
    // Tevkifat, 2 No.lu beyanname ile ödenecek borçtur.
    let s360 = liste.iter().find(|s| s["hesap"] == "360")
        .expect("sorumlu sıfatıyla ödenecek KDV (360) satırı yok");
    assert_eq!(s360["alacak"].as_i64(), Some(100_000));
    assert_eq!(coz(&d, "/denge/denk").as_bool(), Some(true), "tevkifatlı fiş dengesiz");
    assert_eq!(coz(&d, "/ozet/cari_tutar").as_i64(), Some(1_100_000));
}

#[tokio::test]
async fn tevkifatli_satista_391e_yalniz_tevkifat_disi_pay_yazilir() {
    let d = fatura_akisi("FT-TEV-S", &[("yon", "SATIS"), ("unvan", "KAMU İDARESİ"),
        ("belge_no", "S1"), ("tarih", "10.04.2026"), ("matrah", "10.000,00"),
        ("kdv", "2.000,00"), ("toplam", "12.000,00"), ("tevkifat", "1.000,00")]).await;
    let liste = coz(&d, "/fis/satirlar");
    let liste = liste.as_array().unwrap();
    assert_eq!(liste.iter().find(|s| s["hesap"] == "120").unwrap()["borc"].as_i64(), Some(1_100_000),
        "alıcı tevkif ettiği KDV'yi satıcıya ödemez (KDVK 9)");
    assert_eq!(liste.iter().find(|s| s["hesap"] == "391.06").unwrap()["alacak"].as_i64(), Some(100_000),
        "391'e yalnız tevkifat DIŞI pay yazılır");
    assert_eq!(coz(&d, "/denge/denk").as_bool(), Some(true));
}

#[tokio::test]
async fn tevkifat_kdvden_buyuk_olamaz() {
    let d = fatura_akisi("FT-TEV-COK", &[("yon", "ALIS"), ("unvan", "X"), ("belge_no", "N"),
        ("tarih", "01.01.2026"), ("matrah", "1.000,00"), ("kdv", "200,00"),
        ("toplam", "1.200,00"), ("tevkifat", "500,00")]).await;
    let b = coz(&d, "/bulgular");
    assert!(b.as_array().unwrap().iter().any(|x| x["kod"] == "A32" && x["onem"] == "ENGEL"),
        "tevkifat KDV'yi aşıyor, yakalanmadı: {b:?}");
}
