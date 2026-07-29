//! Önerilen Kayıt (bildirim) — **sistem KARAR VERMEZ, önerir; muhasebeci karar verir.**
//!
//! Tahakkuk esasına göre atılması gereken kayıtları hesaplayan motorlar (kredi, ekstre→fiş,
//! dönem sonu değerleme…) çıktılarını doğrudan deftere YAZMAZ; her biri bir `OnerilenKayit`
//! olarak muhasebeciye sunulur. Muhasebeci onaylarsa taslak fiş olur, kesinleştirirse deftere geçer.
//! Vade dilimi (300/303/400) gibi ince ayrımlar da öneri + gerekçe olarak taşınır; kararı meslek
//! mensubu verir. Tasarım: docs/tasarim/banka-veri-hatti.md §4-§6.

use crate::fis::{Fis, Tarih};

/// Öneriye ne kadar güvendiğimiz → UI'da otomatik-taslak vs onay-bekle ayrımı.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Guven {
    /// Deterministik/tek yorum (ör. taksit planı satırı) — otomatik taslak önerilebilir.
    Yuksek,
    /// Makul ama muhasebeci teyidi ister (ör. cari isim eşleşmesi).
    Orta,
    /// Gri bölge; ek bilgi/karar gerekir (ör. anapara/faiz ayrımı, vade dilimi).
    Dusuk,
}

/// Önerinin muhasebeci tarafındaki durumu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KararDurum {
    Bekliyor,
    Onaylandi,
    Reddedildi,
}

/// Öneri türü — bildirim listesini kategorize eder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OneriTuru {
    KrediKullanim,
    KrediTaksit,
    FaizTahakkuku,
    VadeDevir,
    KurFarki,
    BankaMasrafi,
    EkstreFis,
    Diger,
}

/// Muhasebeciye sunulan tek öneri: "şu tarihte şu kayıt atılmalı, şu yüzden".
#[derive(Debug, Clone)]
pub struct OnerilenKayit {
    pub tur: OneriTuru,
    pub tarih: Tarih,
    /// Kısa başlık — "Kredi taksiti #3 (15.07.2026)".
    pub baslik: String,
    /// Neden öneriliyor — tahakkuk esası/mevzuat gerekçesi (muhasebeci görsün diye).
    pub gerekce: String,
    /// Hazır taslak fiş (satırlarıyla); muhasebeci düzenleyip kesinleştirir.
    pub onerilen_fis: Fis,
    pub guven: Guven,
    pub karar: KararDurum,
    /// Dayanak: ekstre satır referansı / kredi no / hash — izlenebilirlik.
    pub kaynak_ref: Option<String>,
}

impl OnerilenKayit {
    pub fn yeni(
        tur: OneriTuru,
        tarih: Tarih,
        baslik: impl Into<String>,
        gerekce: impl Into<String>,
        onerilen_fis: Fis,
        guven: Guven,
    ) -> Self {
        OnerilenKayit {
            tur,
            tarih,
            baslik: baslik.into(),
            gerekce: gerekce.into(),
            onerilen_fis,
            guven,
            karar: KararDurum::Bekliyor,
            kaynak_ref: None,
        }
    }

    pub fn ile_kaynak(mut self, r: impl Into<String>) -> Self {
        self.kaynak_ref = Some(r.into());
        self
    }
}
