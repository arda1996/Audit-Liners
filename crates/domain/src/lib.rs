//! # Audit-Liners — Muhasebe Çekirdeği (domain)
//!
//! Bağımlılıksız saf domain: çift taraflı defter kuralları. Hexagonal mimarinin merkezi —
//! her modül (db, api, ingest, reporting, audit) buna bağlanır; bu hiçbir şeye bağlanmaz.
//!
//! Tasarım: `docs/tasarim/muhasebe-cekirdek-modeli.md`, `docs/tasarim/02-kayit-kurallari.md`.

pub mod cari;
pub mod defter;
pub mod donem;
pub mod ekstre;
pub mod isim;
pub mod mutabakat;
pub mod fatura;
pub mod fis;
pub mod hesap;
pub mod hesap_plani;
pub mod kredi;
pub mod kurallar;
pub mod mali_tablo;
pub mod oneri;
pub mod para;
pub mod vergi;

pub use fis::{
    BelgeTipi, Dayanak, Fis, FisDurum, FisId, FisTipi, MasrafYeriId, Tarih, YevmiyeSatiri,
};
pub use hesap::{Hesap, HesapDogasi, HesapId, HesapTipi};
pub use hesap_plani::{csv_yukle, hesap_olustur, muavin_olustur};
pub use kurallar::{
    acilis_fisi, dogrula, fis_no, iptal_fisi, kesinlestir, Baglam, Donem, DonemDurum, KayitHatasi,
    SeriSayac,
};
pub use cari::{cari_olustur, Cari, CariTip};
pub use defter::{Defter, KebirHareket, MizanSatiri};
pub use donem::{acilis, bilanco_kapanis, donem_sonucu, kapanis};
pub use fatura::{fatura_fisi, Fatura, FaturaDurum, FaturaSatiri, FaturaYonu};
pub use mali_tablo::{bilanco, gelir_tablosu, Bilanco, GelirTablosu};
pub use para::Kurus;
pub use vergi::kdv_mahsup;
pub use kredi::{
    ay_ekle, odeme_plani, oneriler, oneriler_plandan, FaizDilimi, Kredi, OdemeYontemi, TaksitSatiri,
};
pub use oneri::{Guven, KararDurum, OneriTuru, OnerilenKayit};
// D.5-D.7 uyuyan modüller (takipsistemiv2 damıtması) — iş akışında gerekince API/UI'a bağlanır.
pub use ekstre::{devir_dogrula, hareket_anahtari, ters_isle, zincir_dogrula, Ekstre, EkstreHareket, HareketYonu, ZincirBulgu};
pub use isim::{isim_skoru, kanonik_isim, tr_katla};
pub use mutabakat::{mutabakat, ArtikKategori, MutabakatKayit, MutabakatSonuc};
