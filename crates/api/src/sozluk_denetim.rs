//! SÖZLÜK BÜTÜNLÜĞÜ — sözlük büyüdükçe sessizce çürümesin.
//!
//! ## Neden gerekli
//!
//! Sözlük ürünün en çok büyüyecek parçası: her yeni belge türü, her yeni dil, her yeni
//! eş adlandırma buraya eklenir. Ama sözlükteki bir hata **sessizdir** — yanlış yazılmış
//! bir terim hata vermez, sadece hiç eşleşmez ve alan boş kalır.
//!
//! Ölçüldü: bu testler yazılmadan önce `alan_etiketleri`'ndeki **221 girdinin 48'i (%22)
//! hiç eşleşemiyordu.** Türkçe karakter, aksan, tire veya `°` içeren her girdi ölüydü —
//! çünkü karşılaştırma sadeleştirilmiş metinde yapılıyor, sözlük ise doğal yazımda.
//! Yani sözlüğe terim eklemek işe yarıyor *sanılıyordu*.
//!
//! ## Sözleşme
//!
//! 1. Sözlük **doğal yazımla** yazılır (é, ß, ı, №, tire serbest) — yazan kişinin
//!    kodlama kuralını bilmesi gerekmez.
//! 2. Sadeleştirme **sınırda, tek yerde** yapılır (`belge_oku::alan_esanlamlilari`).
//! 3. Bu testler o sözleşmeyi korur: sadeleştirmeden sonra kullanılamaz hale gelen
//!    (boşalan, tek harfe düşen, mükerrer olan) girdileri yakalar.
//!
//! ## Kural: çekim EKLENMEZ
//!
//! `dilbilgisi` katmanı Türkçe/İngilizce çekim eklerini kuralla çözüyor. Sözlüğe
//! "miktarı", "tutarları", "prices" gibi çekimli biçim eklemek gereksizdir ve listeyi
//! şişirir. Test bunu da yakalar: kökü zaten listede olan bir çekimli biçim hatadır.

#[cfg(test)]
mod testler {
    use crate::belge_oku::sadelestir;

    const BELGE: &str =
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/belge-sozlugu.json"));
    const FINANS: &str =
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/finans-sozlugu.json"));

    fn belge() -> serde_json::Value { serde_json::from_str(BELGE).unwrap() }
    fn finans() -> serde_json::Value { serde_json::from_str(FINANS).unwrap() }

    /// Her (bölüm, anahtar, dil, terim) dörtlüsünü gezer.
    fn gez(v: &serde_json::Value, bolum: &str, f: &mut dyn FnMut(&str, &str, &str)) {
        let Some(o) = v[bolum].as_object() else { return };
        for (anahtar, diller) in o {
            if anahtar.starts_with('_') { continue; }
            // belge_turleri'nde diller `anahtar` alanının altında
            let d = if diller.get("anahtar").is_some() { &diller["anahtar"] } else { diller };
            let Some(dm) = d.as_object() else { continue };
            for (dil, liste) in dm {
                if dil.starts_with('_') { continue; }
                for t in liste.as_array().into_iter().flatten() {
                    if let Some(s) = t.as_str() { f(anahtar, dil, s); }
                }
            }
        }
    }

    /// **Sadeleştirmeden sonra kullanılamaz hale gelen girdi olmamalı.**
    /// Bu, %22'lik sessiz kaybın testi.
    #[test]
    fn hicbir_girdi_sadelestirmede_kaybolmaz() {
        let mut kotu = Vec::new();
        for (v, ad) in [(belge(), "belge-sozlugu"), (finans(), "finans-sozlugu")] {
            for bolum in ["alan_etiketleri", "sutun_basliklari", "belge_turleri", "korunan"] {
                gez(&v, bolum, &mut |anahtar, dil, t| {
                    let s = sadelestir(t);
                    if s.chars().count() < 2 {
                        kotu.push(format!("{ad}/{bolum}/{anahtar}/{dil}: {t:?} → {s:?} (kullanılamaz)"));
                    }
                });
            }
        }
        assert!(kotu.is_empty(),
            "sadeleştirmede kullanılamaz hale gelen {} girdi:\n{}", kotu.len(), kotu.join("\n"));
    }

    /// Aynı anahtar+dil içinde sadeleştirilmiş biçimi ÇAKIŞAN girdi olmamalı —
    /// listeyi şişirir, hiçbir şey kazandırmaz.
    #[test]
    fn ayni_alanda_mukerrer_terim_olmaz() {
        let mut kotu = Vec::new();
        for bolum in ["alan_etiketleri", "sutun_basliklari"] {
            let mut gorulen: std::collections::HashMap<String, String> = Default::default();
            gez(&belge(), bolum, &mut |anahtar, dil, t| {
                let k = format!("{bolum}/{anahtar}/{dil}/{}", sadelestir(t));
                if let Some(onceki) = gorulen.insert(k.clone(), t.to_string()) {
                    if onceki != t { kotu.push(format!("{k}: {onceki:?} ile {t:?} aynı")); }
                }
            });
        }
        assert!(kotu.is_empty(), "mükerrer terim:\n{}", kotu.join("\n"));
    }

    /// **Aynı terim iki farklı SÜTUN ANLAMINA bağlanmamalı.** Bağlanırsa hangi anlamın
    /// kazanacağı sözlük sırasına kalır — belirsizlik, sessiz yanlış eşleşme demektir.
    #[test]
    fn bir_terim_iki_sutun_anlamina_baglanmaz() {
        let v = belge();
        let mut sahip: std::collections::HashMap<String, String> = Default::default();
        let mut kotu = Vec::new();
        gez(&v, "sutun_basliklari", &mut |anlam, dil, t| {
            let k = format!("{dil}/{}", sadelestir(t));
            match sahip.get(&k) {
                Some(o) if o != anlam => kotu.push(format!("{t:?} hem {o} hem {anlam}")),
                _ => { sahip.insert(k, anlam.to_string()); }
            }
        });
        assert!(kotu.is_empty(), "çakışan sütun anlamı:\n{}", kotu.join("\n"));
    }

    /// **Çekimli biçim EKLENMEZ** — `dilbilgisi` katmanı onları kuralla üretiyor.
    /// Kökü zaten listede olan bir çekimli biçim, listeyi şişirmekten başka işe yaramaz.
    #[test]
    fn sozlukte_gereksiz_cekim_bulunmaz() {
        let v = belge();
        let mut liste: std::collections::HashMap<(String, String), Vec<String>> = Default::default();
        gez(&v, "sutun_basliklari", &mut |anlam, dil, t| {
            liste.entry((anlam.into(), dil.into())).or_default().push(sadelestir(t));
        });
        let mut kotu = Vec::new();
        for ((anlam, dil), terimler) in &liste {
            if dil != "tr" && dil != "en" { continue; } // kural katmanı yalnız TR+EN
            for t in terimler {
                for kok in crate::dilbilgisi::kokler(t, dil) {
                    if &kok != t && terimler.contains(&kok) {
                        kotu.push(format!("{anlam}/{dil}: {t:?} gereksiz — kökü {kok:?} zaten var"));
                    }
                }
            }
        }
        assert!(kotu.is_empty(),
            "dil bilgisi katmanının zaten ürettiği {} çekimli biçim:\n{}", kotu.len(), kotu.join("\n"));
    }

    /// Türkçe ve İngilizce **birincil dillerdir** — her anahtarda ikisi de dolu olmalı.
    /// Almanca/Fransızca/Rusça ikinci halkadır, boş olabilir.
    #[test]
    fn birincil_diller_her_anahtarda_dolu() {
        let v = belge();
        let mut eksik = Vec::new();
        for bolum in ["alan_etiketleri", "sutun_basliklari"] {
            let Some(o) = v[bolum].as_object() else { continue };
            for (anahtar, diller) in o {
                if anahtar.starts_with('_') { continue; }
                for d in ["tr", "en"] {
                    let n = diller[d].as_array().map(|a| a.len()).unwrap_or(0);
                    if n == 0 { eksik.push(format!("{bolum}/{anahtar}: {d} boş")); }
                }
            }
        }
        assert!(eksik.is_empty(), "birincil dilde boş:\n{}", eksik.join("\n"));
    }

    /// `belge-parametreleri.json`'da tanımlı HER alan kodunun etiket karşılığı olmalı.
    /// Yoksa o alan taramada hiç aranmaz ve sessizce boş kalır.
    #[test]
    fn her_parametre_alaninin_etiketi_var() {
        const PARAM: &str =
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/belge-parametreleri.json"));
        let p: serde_json::Value = serde_json::from_str(PARAM).unwrap();
        let mut eksik = Vec::new();
        for (tur, veri) in p["belge_turleri"].as_object().into_iter().flatten() {
            for alan in veri["parametreler"].as_array().into_iter().flatten() {
                let Some(kod) = alan["kod"].as_str() else { continue };
                // SECENEK tipinde etiket aranmaz — değeri kullanıcı seçer.
                if alan["tip"].as_str() == Some("SECENEK") { continue; }
                if crate::belge_oku::alan_esanlamlilari(kod, "tr").is_empty() {
                    eksik.push(format!("{tur}.{kod}"));
                }
            }
        }
        eksik.sort();
        eksik.dedup();
        assert!(eksik.is_empty(),
            "etiket sözlüğünde karşılığı OLMAYAN {} alan — bunlar taramada hiç aranmaz:\n  {}",
            eksik.len(), eksik.join("\n  "));
    }
}
