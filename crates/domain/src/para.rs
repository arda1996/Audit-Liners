//! Para birimi: **tamsayı kuruş**. Float yasak (yuvarlama hatası muhasebede kabul edilemez).
//! 1 TL = 100 Kurus. UI'da /100 gösterilir; çekirdek her zaman i64 ile çalışır.

use std::ops::Add;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Kurus(pub i64);

impl Kurus {
    pub const SIFIR: Kurus = Kurus(0);

    pub fn yeni(k: i64) -> Self {
        Kurus(k)
    }

    pub fn pozitif(&self) -> bool {
        self.0 > 0
    }

    pub fn negatif(&self) -> bool {
        self.0 < 0
    }

    pub fn deger(&self) -> i64 {
        self.0
    }
}

impl Add for Kurus {
    type Output = Kurus;
    fn add(self, o: Kurus) -> Kurus {
        Kurus(self.0 + o.0)
    }
}

impl std::iter::Sum for Kurus {
    fn sum<I: Iterator<Item = Kurus>>(iter: I) -> Kurus {
        Kurus(iter.map(|k| k.0).sum())
    }
}
