//! ON VERİ SETİ — tahsis (fatura↔banka eşleştirme) motorunun senaryo bazlı sınaması.
//!
//! NEDEN AYRI VERİ SETLERİ: Motor bugüne kadar yalnız tek bir 2228 kayıtlık simülasyon
//! kümesiyle sınanıyordu. O kümede aykırı durumlar (ünvansız hareket, tamamı iade edilmiş
//! fatura, aşan tahsilat, donmuş tahsisin manuelle ezilmesi) ya hiç yok ya da tek örnekle
//! temsil ediliyor — dolayısıyla "testler geçti" ile "motor doğru" aynı şey değildi.
//! Burada her senaryo, YALNIZ o senaryoyu içeren küçük ve elle okunabilir bir veri setiyle
//! sınanır; beklenen sonuç kuruşu kuruşuna yazılır.
//!
//! HER VERİ SETİ İKİ KAPIDAN GEÇER:
//!   1. `kos()` — D01–D05 değişmezleri + bağımsız para korunumu sayımı (senaryodan bağımsız).
//!   2. Senaryonun kendi iddiası — hangi fatura ne kadar kapandı, hangi para nerede kaldı.
//!
//! TARİH TUZAĞI: Tarihler "gg.aa.yyyy". Ham metin sıralaması yanlıştır (05.02.2026 metin
//! olarak 20.01.2026'dan ÖNCE gelir). VS-01 bu tuzağı bilerek kurar — motor tarihi
//! anahtarlayarak sıralamazsa test kırılır.

#![cfg(test)]

use crate::kontrol::degismezler;
use crate::simulasyon::{tahsis_uygula, Dataset, Donmus, FaturaSim, TahsilatSim};
use std::collections::HashMap;

// ─────────────────────────────────────────────────────────────────────────────
// Kurucular — veri setleri elle okunabilir kalsın diye
// ─────────────────────────────────────────────────────────────────────────────

/// TL → kuruş. Veri setleri TL yazar, motor kuruş görür.
fn l(tl: i64) -> i64 { tl * 100 }

fn temel_fatura() -> FaturaSim {
    FaturaSim {
        no: String::new(), tarih: String::new(), yon: "SATIS".into(), karsi: String::new(),
        matrah: 0, kdv: 0, toplam: 0, enstruman: "ACIK".into(), odendi: false,
        tahsil: 0, kalan: 0,
        kdv_oran: 20, istisna_kod: String::new(), tevkifat: 0,
        gecerli: true, iade_matrah: 0, iade_kdv: 0, iskonto_matrah: 0, iskonto_kdv: 0,
        net_borc: 0, tahsisler: Vec::new(),
        ebelge_tip: "E_FATURA".into(), edurum: "KABUL".into(),
        tahakkuk: None, iptal_fisi: None, iade_fisi: None, iskonto_fisi: None, odeme: None,
        // Tahsis motoru maliyetle ilgilenmez (para dağıtır, stok hareketi doğurmaz).
        maliyet: 0, iade_maliyet: 0, smm_fisi: None, iade_stok_fisi: None,
    }
}

/// Türetilen alanları motorun kuralına göre YENİDEN hesaplar.
///
/// Veri setinin elle tutarsız kurulmasını engeller: `net_borc`'u testin kendisi uydurursa
/// motor hatası ile veri seti hatası birbirine karışır. Formül `ham_uret` ile aynıdır —
/// tahsil edilebilir tutar `toplam − tevkifat`, iade ve iskonto onu düşürür, altı sıfırdır.
fn hazirla(mut f: FaturaSim) -> FaturaSim {
    f.kdv = f.matrah * f.kdv_oran / 100;
    f.toplam = f.matrah + f.kdv;
    let tahsil_edilebilir = f.toplam - f.tevkifat;
    f.net_borc = if f.gecerli {
        (tahsil_edilebilir - (f.iade_matrah + f.iade_kdv) - (f.iskonto_matrah + f.iskonto_kdv)).max(0)
    } else { 0 };
    f.kalan = f.net_borc;
    f
}

/// Satış faturası — matrah TL cinsinden verilir, KDV %20 varsayılır.
fn satis(no: &str, tarih: &str, karsi: &str, matrah_tl: i64) -> FaturaSim {
    hazirla(FaturaSim {
        no: no.into(), tarih: tarih.into(), karsi: karsi.into(),
        yon: "SATIS".into(), matrah: l(matrah_tl), ..temel_fatura()
    })
}

/// Alış faturası — yön ayrımının sınanabilmesi için.
fn alis(no: &str, tarih: &str, karsi: &str, matrah_tl: i64) -> FaturaSim {
    hazirla(FaturaSim {
        no: no.into(), tarih: tarih.into(), karsi: karsi.into(),
        yon: "ALIS".into(), matrah: l(matrah_tl), ..temel_fatura()
    })
}

fn temel_tahsilat() -> TahsilatSim {
    TahsilatSim {
        id: String::new(), firma: String::new(), yon: "SATIS".into(), tarih: String::new(),
        tutar: 0, enstruman: "BANKA".into(), dagitilan: 0,
        durum: String::new(), sebep: String::new(),
        banka_ad: "Türkiye İş Bankası".into(),
        hesap_id: "SIM-0064-01".into(),
        iban: "TR30 0064 0000 0000 1000".into(),
        fatura_bekleniyor: false, bekleyen_firma: String::new(),
    }
}

/// Gelen para (satış tahsilatı). `firma` boş verilirse ekstrede ünvan yok demektir.
fn tahsilat(id: &str, tarih: &str, firma: &str, tutar_tl: i64) -> TahsilatSim {
    TahsilatSim {
        id: id.into(), tarih: tarih.into(), firma: firma.into(),
        yon: "SATIS".into(), tutar: l(tutar_tl), ..temel_tahsilat()
    }
}

/// Giden para (alış ödemesi).
fn odeme(id: &str, tarih: &str, firma: &str, tutar_tl: i64) -> TahsilatSim {
    TahsilatSim {
        id: id.into(), tarih: tarih.into(), firma: firma.into(),
        yon: "ALIS".into(), tutar: l(tutar_tl), ..temel_tahsilat()
    }
}

fn manuel(ciftler: &[(&str, &str)]) -> HashMap<String, String> {
    ciftler.iter().map(|(t, f)| (t.to_string(), f.to_string())).collect()
}

// ─────────────────────────────────────────────────────────────────────────────
// Ortak kapı — her veri seti önce buradan geçer
// ─────────────────────────────────────────────────────────────────────────────

/// Veri setini motordan geçirir ve SENARYODAN BAĞIMSIZ doğruları sınar.
///
/// Defter dengesi (D06) burada sınanmaz — bu katman fiş üretmez, tahsis dağıtır;
/// fiş dengesi ayrı testlerin işidir. 0/0 vererek D06 devre dışı bırakılır.
fn kos(ad: &str, f: Vec<FaturaSim>, t: Vec<TahsilatSim>,
       manuel: &HashMap<String, String>, donmus: &Donmus) -> Dataset {
    let girilen: i64 = t.iter().map(|x| x.tutar).sum();
    let d = tahsis_uygula(f, t, manuel, donmus);

    let ihlal = degismezler(&d, 0, 0);
    assert!(ihlal.is_empty(), "[{ad}] DEĞİŞMEZ İHLALİ: {:?}",
        ihlal.iter().map(|x| format!("{} · {}", x.kod, x.mesaj)).collect::<Vec<_>>());

    // PARA KORUNUMU — motorun kendi `dagitilan` alanına GÜVENMEDEN ikinci bir sayım.
    // D02 zaten bunu kontrol ediyor; burada bağımsız olarak tekrar sayıyoruz ki
    // değişmez katmanının kendisi bozulursa da yakalayalım.
    let dagitilan: i64 = d.faturalar.iter().flat_map(|f| &f.tahsisler).map(|x| x.tutar).sum();
    let beyan: i64 = d.tahsilatlar.iter().map(|x| x.dagitilan).sum();
    assert_eq!(dagitilan, beyan, "[{ad}] dağıtılan para beyan edilenle uyuşmuyor");
    assert!(dagitilan <= girilen,
        "[{ad}] PARA YARATILDI: dağıtılan {dagitilan} > giren {girilen}");

    // kalan = net_borc − tahsil, her faturada.
    for f in &d.faturalar {
        assert_eq!(f.kalan, f.net_borc - f.tahsil, "[{ad}] {} kalan hesabı bozuk", f.no);
        assert!(f.kalan >= 0, "[{ad}] {} kalanı negatif", f.no);
    }
    d
}

fn fat<'a>(d: &'a Dataset, no: &str) -> &'a FaturaSim {
    d.faturalar.iter().find(|x| x.no == no).unwrap_or_else(|| panic!("fatura yok: {no}"))
}
fn ths<'a>(d: &'a Dataset, id: &str) -> &'a TahsilatSim {
    d.tahsilatlar.iter().find(|x| x.id == id).unwrap_or_else(|| panic!("tahsilat yok: {id}"))
}
/// Faturanın havada kalan (kapanmamış) borcu — okunurluk için.
fn acik(d: &Dataset, no: &str) -> i64 { fat(d, no).kalan }

// ═════════════════════════════════════════════════════════════════════════════
// VS-01 · DÜZ FIFO — en eski fatura önce kapanır (TBK 100)
// ═════════════════════════════════════════════════════════════════════════════
// TARİH TUZAĞI KURULU: 20.01.2026 gerçekte 05.02.2026'dan ESKİ, ama METİN olarak SONRA gelir.
// Motor tarihi anahtarlamadan sıralarsa yanlış faturayı kapatır ve bu test kırılır.

#[test]
fn vs01_fifo_en_eski_faturadan_kapatir() {
    let f = vec![
        satis("FT-1", "20.01.2026", "ALFA A.Ş.", 1_000), // en ESKİ (metin sırasında sonda)
        satis("FT-2", "05.02.2026", "ALFA A.Ş.", 1_000),
        satis("FT-3", "11.03.2026", "ALFA A.Ş.", 1_000),
    ];
    // 2.400 TL = tam iki faturanın toplamı (1.200 + 1.200)
    let t = vec![tahsilat("T-1", "31.03.2026", "ALFA A.Ş.", 2_400)];
    let d = kos("VS-01", f, t, &HashMap::new(), &Donmus::new());

    assert_eq!(acik(&d, "FT-1"), 0, "en eski fatura kapanmadı — FIFO tarih sırası bozuk");
    assert_eq!(acik(&d, "FT-2"), 0, "ikinci fatura kapanmadı");
    assert_eq!(acik(&d, "FT-3"), l(1_200), "en yeni fatura AÇIK kalmalıydı");
    assert_eq!(ths(&d, "T-1").dagitilan, l(2_400), "para tamamen dağıtılmalıydı");
    assert_eq!(ths(&d, "T-1").durum, "OTOMATIK");
}

// ═════════════════════════════════════════════════════════════════════════════
// VS-02 · KISMİ TAHSİLAT — fatura birden çok ödemeyle kapanır
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn vs02_kismi_tahsilat_faturayi_asamali_kapatir() {
    let fatura = || vec![satis("FT-1", "10.01.2026", "BETA LTD.", 1_000)]; // 1.200 TL borç

    // Adım 1 — yalnız 500 TL geldi: fatura AÇIK kalmalı, kalan 700.
    let d1 = kos("VS-02/adım1", fatura(),
        vec![tahsilat("T-1", "15.01.2026", "BETA LTD.", 500)],
        &HashMap::new(), &Donmus::new());
    assert_eq!(acik(&d1, "FT-1"), l(700), "kısmi tahsilat sonrası kalan yanlış");
    assert_eq!(fat(&d1, "FT-1").tahsil, l(500));
    assert!(!fat(&d1, "FT-1").odendi, "kısmi ödenen fatura 'ödendi' sayılamaz");

    // Adım 2 — kalan 700 de geldi: fatura KAPANMALI, iki ayrı tahsis izlenmeli.
    let d2 = kos("VS-02/adım2", fatura(),
        vec![
            tahsilat("T-1", "15.01.2026", "BETA LTD.", 500),
            tahsilat("T-2", "20.02.2026", "BETA LTD.", 700),
        ],
        &HashMap::new(), &Donmus::new());
    assert_eq!(acik(&d2, "FT-1"), 0, "iki tahsilat faturayı kapatmadı");
    assert!(fat(&d2, "FT-1").odendi);
    assert_eq!(fat(&d2, "FT-1").tahsisler.len(), 2, "her tahsilat AYRI tahsis olarak izlenmeli");
    // Kronoloji: önce gelen para önce yazılmalı (denetim izi).
    assert_eq!(fat(&d2, "FT-1").tahsisler[0].tahsilat_id, "T-1");
    assert_eq!(fat(&d2, "FT-1").tahsisler[0].tutar, l(500));
    assert_eq!(fat(&d2, "FT-1").tahsisler[1].tutar, l(700));
}

// ═════════════════════════════════════════════════════════════════════════════
// VS-03 · AŞAN TAHSİLAT — fazla para AVANS olarak havada kalır, kaybolmaz
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn vs03_asan_tahsilat_avansta_kalir() {
    let f = vec![satis("FT-1", "10.01.2026", "GAMA A.Ş.", 1_000)]; // 1.200 borç
    let t = vec![tahsilat("T-1", "15.01.2026", "GAMA A.Ş.", 2_000)]; // 800 fazla
    let d = kos("VS-03", f, t, &HashMap::new(), &Donmus::new());

    assert_eq!(acik(&d, "FT-1"), 0, "fatura kapanmalıydı");
    let t1 = ths(&d, "T-1");
    assert_eq!(t1.dagitilan, l(1_200), "faturanın borcundan fazlası dağıtılamaz");
    assert_eq!(t1.tutar - t1.dagitilan, l(800), "aşan 800 TL avans olarak durmalı");
    assert!(t1.sebep.contains("avans"), "aşan tutar kullanıcıya AVANS diye bildirilmeli: {}", t1.sebep);
    // Fatura borcundan fazla tahsis almamalı (D03 zaten bakar; burada kuruşu doğruluyoruz).
    assert_eq!(fat(&d, "FT-1").tahsisler.iter().map(|x| x.tutar).sum::<i64>(), l(1_200));
}

// ═════════════════════════════════════════════════════════════════════════════
// VS-04 · HÜKÜMSÜZ BELGE — TASLAK / İPTAL / RED tahsis alamaz (VUK 227)
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn vs04_hukumsuz_belge_tahsis_almaz() {
    let gecersiz = |no: &str, tarih: &str, durum: &str| {
        hazirla(FaturaSim {
            no: no.into(), tarih: tarih.into(), karsi: "DELTA A.Ş.".into(),
            matrah: l(1_000), gecerli: false, edurum: durum.into(), ..temel_fatura()
        })
    };
    let f = vec![
        gecersiz("FT-TASLAK", "05.01.2026", "TASLAK"),
        gecersiz("FT-IPTAL", "06.01.2026", "IPTAL"),
        gecersiz("FT-RED", "07.01.2026", "RED"),
        satis("FT-GECERLI", "08.01.2026", "DELTA A.Ş.", 1_000),
    ];
    // Hepsini kapatmaya yetecek para gönder — motor yalnız geçerli olanı kapatmalı.
    let t = vec![tahsilat("T-1", "20.01.2026", "DELTA A.Ş.", 5_000)];
    let d = kos("VS-04", f, t, &HashMap::new(), &Donmus::new());

    for no in ["FT-TASLAK", "FT-IPTAL", "FT-RED"] {
        assert_eq!(fat(&d, no).net_borc, 0, "{no} hükümsüz — cari borç doğuramaz");
        assert!(fat(&d, no).tahsisler.is_empty(), "{no} hükümsüz olduğu halde tahsis aldı");
    }
    assert_eq!(acik(&d, "FT-GECERLI"), 0, "geçerli fatura kapanmalıydı");
    // Kalan 3.800 TL karşılıksız — avansta durmalı, hükümsüz belgelere dağıtılmamalı.
    assert_eq!(ths(&d, "T-1").dagitilan, l(1_200));
}

// ═════════════════════════════════════════════════════════════════════════════
// VS-05 · İADE ve İSKONTO — net borç düşer; tamamı iade edilen fatura tahsis almaz
// ═════════════════════════════════════════════════════════════════════════════
// Muhasebe kuralı: iade 610'a, iskonto 611'e yazılır; 600 TERS ÇEVRİLMEZ.
// Tahsis motorunu ilgilendiren kısmı: kapatılacak borç `toplam − iade − iskonto`.

#[test]
fn vs05_iade_ve_iskonto_net_borcu_dusurur() {
    // A: tamamı iade → net borç 0 → tahsis ALMAMALI (K06 ile de engellenir).
    let tam_iade = hazirla(FaturaSim {
        no: "FT-A".into(), tarih: "05.01.2026".into(), karsi: "EPSILON A.Ş.".into(),
        matrah: l(1_000), iade_matrah: l(1_000), iade_kdv: l(200), ..temel_fatura()
    });
    // B: kısmi iade 400+80 → 1.200 − 480 = 720 borç.
    let kismi_iade = hazirla(FaturaSim {
        no: "FT-B".into(), tarih: "10.01.2026".into(), karsi: "EPSILON A.Ş.".into(),
        matrah: l(1_000), iade_matrah: l(400), iade_kdv: l(80), ..temel_fatura()
    });
    // C: fatura sonrası iskonto 200+40 → 1.200 − 240 = 960 borç.
    let iskontolu = hazirla(FaturaSim {
        no: "FT-C".into(), tarih: "15.01.2026".into(), karsi: "EPSILON A.Ş.".into(),
        matrah: l(1_000), iskonto_matrah: l(200), iskonto_kdv: l(40), ..temel_fatura()
    });
    assert_eq!(tam_iade.net_borc, 0, "veri seti kurulumu hatalı");
    assert_eq!(kismi_iade.net_borc, l(720), "veri seti kurulumu hatalı");
    assert_eq!(iskontolu.net_borc, l(960), "veri seti kurulumu hatalı");

    let t = vec![tahsilat("T-1", "31.01.2026", "EPSILON A.Ş.", 5_000)];
    let d = kos("VS-05", vec![tam_iade, kismi_iade, iskontolu], t, &HashMap::new(), &Donmus::new());

    assert!(fat(&d, "FT-A").tahsisler.is_empty(), "tamamı iade edilmiş faturaya para yazıldı");
    assert_eq!(fat(&d, "FT-B").tahsil, l(720), "kısmi iadeli faturanın net borcu yanlış kapandı");
    assert_eq!(fat(&d, "FT-C").tahsil, l(960), "iskontolu faturanın net borcu yanlış kapandı");
    // Toplam dağıtım = 720 + 960; gerisi avans.
    assert_eq!(ths(&d, "T-1").dagitilan, l(1_680));
}

// ═════════════════════════════════════════════════════════════════════════════
// VS-06 · TEVKİFAT — alıcı KDV'nin bir kısmını satıcıya ÖDEMEZ (KDVK 9)
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn vs06_tevkifatli_faturada_tahsil_edilebilir_tutar_dusuk() {
    // 1.000 matrah · %20 KDV = 200 · tevkifat 5/10 = 100 → cariden istenecek: 1.100
    let f = hazirla(FaturaSim {
        no: "FT-TEV".into(), tarih: "10.01.2026".into(), karsi: "ZETA İNŞAAT".into(),
        matrah: l(1_000), tevkifat: l(100), ..temel_fatura()
    });
    assert_eq!(f.toplam, l(1_200), "veri seti kurulumu hatalı");
    assert_eq!(f.net_borc, l(1_100), "tevkifat cari borcundan düşmeli — alıcı o KDV'yi satıcıya ödemez");

    // Doğru tutar gelirse fatura kapanır.
    let d = kos("VS-06/doğru", vec![f.clone()],
        vec![tahsilat("T-1", "20.01.2026", "ZETA İNŞAAT", 1_100)],
        &HashMap::new(), &Donmus::new());
    assert_eq!(acik(&d, "FT-TEV"), 0);

    // Cari yanlışlıkla BRÜT öderse (1.200), fazla 100 TL faturaya yazılmamalı — avans olmalı.
    let d2 = kos("VS-06/brüt", vec![f],
        vec![tahsilat("T-2", "20.01.2026", "ZETA İNŞAAT", 1_200)],
        &HashMap::new(), &Donmus::new());
    assert_eq!(fat(&d2, "FT-TEV").tahsil, l(1_100), "tevkifatlı faturaya brüt tutar yazılamaz");
    assert_eq!(ths(&d2, "T-2").tutar - ths(&d2, "T-2").dagitilan, l(100), "fazla 100 TL avansta kalmalı");
}

// ═════════════════════════════════════════════════════════════════════════════
// VS-07 · ÜNVANSIZ HAREKET — otomatik eşleşme YASAK, maker kuyruğuna düşer
// ═════════════════════════════════════════════════════════════════════════════
// Ekstrede karşı taraf ünvanı yoksa paranın hangi cariye ait olduğu BİLİNEMEZ.
// Tahmin yürütmek yanlış carinin bakiyesini bozar — bu yüzden motor durur, maker karar verir.

#[test]
fn vs07_unvansiz_hareket_otomatik_eslesmez_manuelle_cozulur() {
    let fatura = || vec![
        satis("FT-1", "10.01.2026", "THETA A.Ş.", 1_000),
        satis("FT-2", "20.01.2026", "THETA A.Ş.", 1_000),
    ];
    let unvansiz = || vec![tahsilat("T-BOS", "25.01.2026", "", 1_200)]; // firma BOŞ

    // Adım 1 — otomatik: hiçbir faturaya dokunmamalı.
    let d1 = kos("VS-07/otomatik", fatura(), unvansiz(), &HashMap::new(), &Donmus::new());
    assert_eq!(ths(&d1, "T-BOS").dagitilan, 0, "ünvansız hareket otomatik eşleşti — yanlış cari riski");
    assert_eq!(ths(&d1, "T-BOS").durum, "BEKLIYOR");
    assert!(ths(&d1, "T-BOS").sebep.contains("ünvan"),
        "maker'a sebep bildirilmeli: {}", ths(&d1, "T-BOS").sebep);
    assert_eq!(acik(&d1, "FT-1"), l(1_200), "fatura açık kalmalıydı");
    assert_eq!(acik(&d1, "FT-2"), l(1_200), "fatura açık kalmalıydı");

    // Adım 2 — maker müdahale eder: parayı FT-2'ye bağlar (ünvan yoksa cariyi maker beyan eder).
    let d2 = kos("VS-07/manuel", fatura(), unvansiz(),
        &manuel(&[("T-BOS", "FT-2")]), &Donmus::new());
    assert_eq!(acik(&d2, "FT-2"), 0, "maker kararı uygulanmadı");
    assert_eq!(acik(&d2, "FT-1"), l(1_200), "maker FT-2 dedi, FIFO FT-1'i kapatmamalı");
    assert_eq!(ths(&d2, "T-BOS").durum, "MANUEL");
}

// ═════════════════════════════════════════════════════════════════════════════
// VS-08 · MANUEL MÜDAHALE — maker kararı FIFO'yu ezer, eski eşleşme BOŞA ÇIKAR
// ═════════════════════════════════════════════════════════════════════════════
// Kullanıcının doğrudan tarif ettiği davranış: "manuel eşleştirme yapılınca önceki
// otomatik fatura eşleşmesi kaldırılmalı, o fatura tekrar açığa düşmeli."

#[test]
fn vs08_manuel_karar_fifo_eslesmesini_dusurur() {
    let fatura = || vec![
        satis("FT-1", "10.01.2026", "IOTA A.Ş.", 1_000), // FIFO bunu kapatır
        satis("FT-2", "20.02.2026", "IOTA A.Ş.", 1_000), // maker bunu ister
    ];
    let para = || vec![tahsilat("T-1", "25.02.2026", "IOTA A.Ş.", 1_200)];

    // Önce: saf FIFO en eskiyi kapatır.
    let once = kos("VS-08/önce", fatura(), para(), &HashMap::new(), &Donmus::new());
    assert_eq!(acik(&once, "FT-1"), 0);
    assert_eq!(acik(&once, "FT-2"), l(1_200));
    assert_eq!(once.faturalar.iter().find(|x| x.no == "FT-1").unwrap().tahsisler[0].kaynak, "FIFO");

    // Sonra: maker "bu para FT-2'ye ait" der.
    let sonra = kos("VS-08/sonra", fatura(), para(), &manuel(&[("T-1", "FT-2")]), &Donmus::new());
    assert_eq!(acik(&sonra, "FT-2"), 0, "maker kararı uygulanmadı");
    assert_eq!(acik(&sonra, "FT-1"), l(1_200),
        "önceki OTOMATİK eşleşme kaldırılmadı — fatura boşa çıkmalıydı");
    assert_eq!(fat(&sonra, "FT-2").tahsisler[0].kaynak, "MANUEL", "tahsis kaynağı MANUEL işaretlenmeli");
    assert!(fat(&sonra, "FT-1").tahsisler.is_empty(), "düşürülen faturada tahsis izi kalmamalı");
}

// ═════════════════════════════════════════════════════════════════════════════
// VS-09 · DONMUŞ TAHSİS — defterlenmiş dağıtım korunur, ama maker onu ezebilir
// ═════════════════════════════════════════════════════════════════════════════
// Deftere fiş atılmış bir dağıtım kendiliğinden değişemez (zincirleme storno olurdu).
// Maker açıkça müdahale ederse: `tahsis_manuel` YALNIZ o tahsilatı donmuş kümeden
// serbest bırakır (main.rs: serbest = {tahsilat_id}) — bu test o davranışı taklit eder.

#[test]
fn vs09_donmus_tahsis_korunur_maker_serbest_birakinca_tasinir() {
    let fatura = || vec![
        satis("FT-1", "10.01.2026", "KAPPA LTD.", 1_000),
        satis("FT-2", "20.02.2026", "KAPPA LTD.", 1_000),
    ];
    let para = || vec![tahsilat("T-1", "25.02.2026", "KAPPA LTD.", 1_200)];
    // Defterde: T-1 → FT-1 (1.200 TL) fişi atılmış durumda.
    let donmus: Donmus = [("T-1".to_string(), vec![("FT-1".to_string(), l(1_200))])]
        .into_iter().collect();

    // 1) Donmuş dağıtım KORUNUR — motor yeniden hesaplasa bile FT-1 kapalı kalır.
    let d1 = kos("VS-09/donmuş", fatura(), para(), &HashMap::new(), &donmus);
    assert_eq!(acik(&d1, "FT-1"), 0, "defterlenmiş dağıtım korunmadı");
    assert_eq!(acik(&d1, "FT-2"), l(1_200));

    // 2) Maker T-1'i FT-2'ye taşır. main.rs bu tahsilatı serbest bırakır → donmuş kümeden çıkar.
    let serbest_donmus = Donmus::new(); // T-1 serbest bırakıldığı için donmuş pay kalmaz
    let d2 = kos("VS-09/taşındı", fatura(), para(),
        &manuel(&[("T-1", "FT-2")]), &serbest_donmus);
    assert_eq!(acik(&d2, "FT-2"), 0, "maker kararı uygulanmadı");
    assert_eq!(acik(&d2, "FT-1"), l(1_200), "donmuş eşleşme kaldırılmadı — fatura açığa düşmeliydi");

    // 3) SERBEST BIRAKILMAZSA: donmuş pay FT-1'de durur ve maker kararı BOŞA DÜŞER.
    //    Bu, motorun bilinen sınırıdır; API katmanı serbest bırakarak çözer. Sınırı kayıt altına
    //    alıyoruz ki ileride biri `serbest` mekanizmasını kaldırırsa test bunu haber versin.
    let d3 = kos("VS-09/serbest-değil", fatura(), para(),
        &manuel(&[("T-1", "FT-2")]), &donmus);
    assert_eq!(acik(&d3, "FT-1"), 0,
        "serbest bırakılmayan donmuş tahsis düşmemeli");
    assert_eq!(acik(&d3, "FT-2"), l(1_200),
        "donmuş para zaten kullanılmışken maker kararı ikinci kez para yaratmamalı");
}

// ═════════════════════════════════════════════════════════════════════════════
// VS-10 · KARIŞIK / AYKIRI — çoklu cari, iki yön, aynı gün, gelecek tarihli fatura
// ═════════════════════════════════════════════════════════════════════════════
// Gerçek defterde hepsi aynı anda bulunur. Burada aranan: SIZINTI YOK.
//   · Bir carinin parası başka cariyi kapatmamalı
//   · Tahsilat (gelen) alış faturasını kapatmamalı, ödeme (giden) satışı kapatmamalı
//   · Tahsilattan SONRA kesilen fatura o parayla kapanmamalı (doğmamış borç ödenemez)

#[test]
fn vs10_karisik_defterde_cari_ve_yon_sizintisi_olmaz() {
    let f = vec![
        satis("S-ALFA-1", "10.01.2026", "ALFA A.Ş.", 1_000),
        satis("S-ALFA-2", "10.01.2026", "ALFA A.Ş.", 500),   // AYNI GÜN — kararlı sıra belge no ile
        satis("S-BETA-1", "12.01.2026", "BETA LTD.", 1_000),
        alis("A-ALFA-1", "14.01.2026", "ALFA A.Ş.", 800),    // aynı cari, TERS yön
        satis("S-ALFA-3", "20.06.2026", "ALFA A.Ş.", 1_000), // GELECEK tarihli (tahsilattan sonra)
    ];
    let t = vec![
        tahsilat("T-ALFA", "31.01.2026", "ALFA A.Ş.", 1_800), // ALFA satışlarına
        odeme("O-ALFA", "31.01.2026", "ALFA A.Ş.", 960),      // ALFA alışına (800+160 KDV)
    ];
    let d = kos("VS-10", f, t, &HashMap::new(), &Donmus::new());

    // ALFA satışları FIFO ile: S-ALFA-1 (1.200) tam, kalan 600 → S-ALFA-2'ye (600) tam.
    assert_eq!(acik(&d, "S-ALFA-1"), 0);
    assert_eq!(acik(&d, "S-ALFA-2"), 0, "aynı gün ikinci fatura da kapanmalıydı");
    assert_eq!(ths(&d, "T-ALFA").dagitilan, l(1_800));

    // BETA'ya SIZINTI OLMAMALI — ALFA'nın parası BETA'nın faturasını kapatamaz.
    assert_eq!(acik(&d, "S-BETA-1"), l(1_200), "cari sızıntısı: ALFA parası BETA faturasını kapattı");

    // YÖN AYRIMI — giden para yalnız alış faturasını kapatır.
    assert_eq!(acik(&d, "A-ALFA-1"), 0, "alış faturası ödemeyle kapanmalıydı");
    assert_eq!(ths(&d, "O-ALFA").dagitilan, l(960));

    // GELECEK TARİHLİ FATURA — 31.01'de gelen para 20.06'da kesilecek faturayı kapatamaz.
    assert_eq!(acik(&d, "S-ALFA-3"), l(1_200),
        "henüz doğmamış borç kapatıldı — tahsilat tarihinden sonraki fatura FIFO'ya girmemeli");
}

// ═════════════════════════════════════════════════════════════════════════════
// ORTAK · DETERMİNİZM — aynı girdi HER ZAMAN aynı sonucu vermeli
// ═════════════════════════════════════════════════════════════════════════════
// Motor içeride HashMap kullanıyor. HashMap iterasyon sırası Rust'ta çalıştırmadan
// çalıştırmaya DEĞİŞİR. Sonuç bu sıraya bağımlı hale gelirse defter, hiçbir şey
// değişmediği halde her yeniden başlatmada farklı çıkar — denetimde savunulamaz.

#[test]
fn tum_veri_setleri_deterministik() {
    let kur = || (
        vec![
            satis("FT-1", "20.01.2026", "ALFA A.Ş.", 1_000),
            satis("FT-2", "05.02.2026", "ALFA A.Ş.", 1_000),
            satis("FT-3", "05.02.2026", "BETA LTD.", 1_000), // aynı tarih, farklı cari
            alis("FT-4", "05.02.2026", "ALFA A.Ş.", 700),
        ],
        vec![
            tahsilat("T-1", "10.02.2026", "ALFA A.Ş.", 1_500),
            tahsilat("T-2", "10.02.2026", "BETA LTD.", 900),
            odeme("O-1", "10.02.2026", "ALFA A.Ş.", 400),
            tahsilat("T-3", "10.02.2026", "", 300),
        ],
    );
    let imza = |d: &Dataset| -> String {
        // Sonucun TAM parmak izi — hangi fatura, hangi tahsilattan, ne kadar, hangi kaynakla.
        d.faturalar.iter().map(|f| format!("{}:{}:{}|{}", f.no, f.tahsil, f.kalan,
            f.tahsisler.iter().map(|x| format!("{}={}#{}", x.tahsilat_id, x.tutar, x.kaynak))
                .collect::<Vec<_>>().join(","))).collect::<Vec<_>>().join(";")
    };

    let (f0, t0) = kur();
    let beklenen = imza(&tahsis_uygula(f0, t0, &HashMap::new(), &Donmus::new()));
    for tur in 1..=25 {
        let (f, t) = kur();
        let simdi = imza(&tahsis_uygula(f, t, &HashMap::new(), &Donmus::new()));
        assert_eq!(simdi, beklenen, "tur {tur}: aynı girdi farklı sonuç verdi — HashMap sırası sızıyor");
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// ORTAK · YÖN BÜTÜNLÜĞÜ — ünvansız hareket bile ters yöne bağlanamaz (K05 · D07)
// ═════════════════════════════════════════════════════════════════════════════
// Bulunan hata: K05 ve motorun yön denetimi `!t.firma.is_empty()` koşulunun içindeydi.
// Havadaki hareketlerin TAMAMINDA ünvan boş olduğu için yön kontrolü, manuel eşleştirmenin
// fiilen kullanıldığı HER durumda kapalıydı. Maker bir banka GİRİŞİNİ alış faturasına
// bağlayabiliyor, fiş `320 borç / 102 ALACAK` yazılıyordu: giren para çıkmış gibi kaydediliyor,
// mizan yine denk olduğu için hiçbir kontrol öterme yapmıyordu.

#[test]
fn unvansiz_hareket_ters_yone_baglanamaz() {
    let f = vec![
        alis("A-1", "10.01.2026", "NU LTD.", 1_000),   // ALIŞ faturası
        satis("S-1", "10.01.2026", "NU LTD.", 1_000),  // SATIŞ faturası
    ];
    // Ünvansız GELEN para (yon = SATIS). Maker yanlışlıkla ALIŞ faturasını seçiyor.
    let t = vec![tahsilat("T-BOS", "20.01.2026", "", 1_200)];

    let d = kos("YÖN", f, t, &manuel(&[("T-BOS", "A-1")]), &Donmus::new());
    assert!(fat(&d, "A-1").tahsisler.is_empty(),
        "gelen para ALIŞ faturasına bağlandı — 102 ALACAK'a yazılırdı");
    assert_eq!(acik(&d, "A-1"), l(1_200), "alış faturası açık kalmalıydı");

    // Doğru yön kabul edilmeli — kural yönü kapatmak değil, doğru yöne zorlamak.
    let f2 = vec![
        alis("A-1", "10.01.2026", "NU LTD.", 1_000),
        satis("S-1", "10.01.2026", "NU LTD.", 1_000),
    ];
    let d2 = kos("YÖN/doğru", f2, vec![tahsilat("T-BOS", "20.01.2026", "", 1_200)],
        &manuel(&[("T-BOS", "S-1")]), &Donmus::new());
    assert_eq!(acik(&d2, "S-1"), 0, "doğru yöndeki manuel karar uygulanmalıydı");
}

/// D07 kuralın kendisini de sınar: ters yön kurulursa değişmez katmanı YAKALAMALI.
#[test]
fn d07_ters_yon_tahsisi_yakalar() {
    use crate::simulasyon::TahsisSim;
    let mut d = tahsis_uygula(
        vec![alis("A-1", "10.01.2026", "KSI LTD.", 1_000)],
        vec![tahsilat("T-1", "20.01.2026", "KSI LTD.", 1_200)], // yon = SATIS
        &HashMap::new(), &Donmus::new());
    // Motoru atlayıp elle bozuyoruz — değişmez katmanı bunu görmeli.
    d.faturalar[0].tahsisler.push(TahsisSim {
        tahsilat_id: "T-1".into(), tarih: "20.01.2026".into(), tutar: l(1_200),
        kaynak: "MANUEL".into(), enstruman: "BANKA".into(),
    });
    let bulgular = degismezler(&d, 0, 0);
    assert!(bulgular.iter().any(|x| x.kod == "D07"),
        "ters yön tahsisi D07 ile yakalanmadı: {:?}",
        bulgular.iter().map(|x| x.kod.clone()).collect::<Vec<_>>());
}

// ═════════════════════════════════════════════════════════════════════════════
// ORTAK · KARMA ENSTRÜMAN — parayı hangi kasa/hesap aldıysa oraya yazılmalı
// ═════════════════════════════════════════════════════════════════════════════
// Bir fatura kısmen nakit, kısmen havale ile kapanabilir. Ödeme fişi tek bir enstrüman
// varsayarsa, bankaya giren para kasada (veya çek portföyünde) görünür — mizan denk kalır,
// ama HANGİ VARLIK arttığı yanlış olur. Denetimde banka mutabakatı tutmaz.

#[test]
fn karma_enstrumanli_odeme_her_hesaba_kendi_tutarini_yazar() {
    let f = vec![satis("FT-1", "10.01.2026", "MU A.Ş.", 1_000)]; // 1.200 TL borç
    let t = vec![
        TahsilatSim { enstruman: "NAKIT".into(), ..tahsilat("T-N", "15.01.2026", "MU A.Ş.", 300) },
        TahsilatSim { enstruman: "BANKA".into(), ..tahsilat("T-B", "20.01.2026", "MU A.Ş.", 900) },
    ];
    let d = kos("KARMA", f, t, &HashMap::new(), &Donmus::new());
    let fatura = fat(&d, "FT-1");

    assert_eq!(acik(&d, "FT-1"), 0, "fatura kapanmalıydı");
    assert_eq!(fatura.enstruman, "KARMA", "tek enstrüman varsayımı — fatura karma kapandı");

    let fis = fatura.odeme.as_ref().expect("ödeme fişi üretilmedi");
    let bul = |h: &str| fis.satirlar.iter().filter(|s| s.hesap == h)
        .map(|s| (s.borc, s.alacak)).next();
    assert_eq!(bul("100"), Some((l(300), 0)), "nakit tutarı 100 Kasa'ya borç yazılmalı");
    assert_eq!(bul("102"), Some((l(900), 0)), "havale tutarı 102 Bankalar'a borç yazılmalı");
    assert_eq!(bul("120"), Some((0, l(1_200))), "karşı bacak 120 Alıcılar, toplam tutarla");

    // Fiş kendi içinde DENK olmalı — satır sayısı arttı, denge bozulmamalı.
    let b: i64 = fis.satirlar.iter().map(|s| s.borc).sum();
    let a: i64 = fis.satirlar.iter().map(|s| s.alacak).sum();
    assert_eq!(b, a, "karma ödeme fişi dengesiz");
    assert_eq!(b, l(1_200));
}

// ═════════════════════════════════════════════════════════════════════════════
// ORTAK · KURUŞ BÜTÜNLÜĞÜ — bölünen tahsilatta tek kuruş bile kaybolmamalı
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn kurus_kaybi_olmaz() {
    // Kuruş artıklı tutarlar: 333,33 · 333,33 · 333,34 → toplam 1.000,00
    let mk = |no: &str, tarih: &str, net: i64| {
        let mut f = temel_fatura();
        f.no = no.into(); f.tarih = tarih.into(); f.karsi = "LAMBDA A.Ş.".into();
        f.matrah = net; f.kdv = 0; f.kdv_oran = 0; f.istisna_kod = "301".into();
        f.toplam = net; f.net_borc = net; f.kalan = net;
        f
    };
    let f = vec![
        mk("FT-1", "10.01.2026", 33_333),
        mk("FT-2", "11.01.2026", 33_333),
        mk("FT-3", "12.01.2026", 33_334),
    ];
    let t = vec![tahsilat("T-1", "20.01.2026", "LAMBDA A.Ş.", 1_000)]; // 100.000 kuruş
    let d = kos("KURUŞ", f, t, &HashMap::new(), &Donmus::new());

    let toplam_borc = 33_333 + 33_333 + 33_334;
    let dagitilan: i64 = d.faturalar.iter().map(|f| f.tahsil).sum();
    assert_eq!(dagitilan, toplam_borc, "kuruş kaybı/fazlası var");
    for no in ["FT-1", "FT-2", "FT-3"] {
        assert_eq!(acik(&d, no), 0, "{no} kapanmadı");
    }
}
