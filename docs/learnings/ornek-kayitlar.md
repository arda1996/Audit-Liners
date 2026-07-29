# Örnek Muhasebe Kayıtları (en sık kullanılanlar)

En yaygın yevmiye kayıtları — hem öğrenme referansı hem **domain test fixture** kaynağı.
KDV genel oran **%20** alınmıştır. Hesaplar `data/tdhp.csv`'den.
Kaynaklar (araştırma için): muhasebedersleri.com, vergidosyasi.com, muhasebenews.com.

> Kural hatırlatma: Aktif/Gider artış=**Borç**; Pasif/Gelir artış=**Alacak**; her fişte Σborç=Σalacak.

## 1. Peşin mal alışı (Tediye fişi)
KDV hariç 10.000, %20 KDV, kasadan nakit.
| Hesap | Borç | Alacak |
|------|-----:|-------:|
| 153 Ticari Mallar | 10.000 | |
| 191 İndirilecek KDV | 2.000 | |
| 100 Kasa | | 12.000 |

## 2. Veresiye (kredili) mal alışı (Mahsup fişi)
| Hesap | Borç | Alacak |
|------|-----:|-------:|
| 153 Ticari Mallar | 10.000 | |
| 191 İndirilecek KDV | 2.000 | |
| 320 Satıcılar | | 12.000 |

## 3. Peşin mal satışı (Tahsil fişi)
Satış 15.000 + %20 KDV, tahsilat kasaya.
| Hesap | Borç | Alacak |
|------|-----:|-------:|
| 100 Kasa | 18.000 | |
| 600 Yurtiçi Satışlar | | 15.000 |
| 391 Hesaplanan KDV | | 3.000 |

## 4. Veresiye satış (Mahsup fişi)
| Hesap | Borç | Alacak |
|------|-----:|-------:|
| 120 Alıcılar | 18.000 | |
| 600 Yurtiçi Satışlar | | 15.000 |
| 391 Hesaplanan KDV | | 3.000 |

## 5. Satılan malın maliyeti (eş zamanlı — aralıksız envanter)
Satılan malın maliyeti 10.000.
| Hesap | Borç | Alacak |
|------|-----:|-------:|
| 621 Satılan Ticari Mallar Maliyeti | 10.000 | |
| 153 Ticari Mallar | | 10.000 |

## 6. Alıcıdan banka yoluyla tahsilat
| Hesap | Borç | Alacak |
|------|-----:|-------:|
| 102 Bankalar | 18.000 | |
| 120 Alıcılar | | 18.000 |

## 7. Satıcıya banka yoluyla ödeme
| Hesap | Borç | Alacak |
|------|-----:|-------:|
| 320 Satıcılar | 12.000 | |
| 102 Bankalar | | 12.000 |

## 8. Ay sonu KDV tahakkuku (191 ↔ 391 mahsup)
Hesaplanan 3.000 > İndirilecek 2.000 → 1.000 ödenecek.
| Hesap | Borç | Alacak |
|------|-----:|-------:|
| 391 Hesaplanan KDV | 3.000 | |
| 191 İndirilecek KDV | | 2.000 |
| 360 Ödenecek Vergi ve Fonlar | | 1.000 |
*(İndirilecek > Hesaplanan olsaydı fark 190 Devreden KDV'ye borç yazılır.)*

## 9. Ücret/bordro tahakkuku (basitleştirilmiş)
Brüt 10.000; net 7.000; GV+damga 1.500; SGK kesintisi 1.500.
| Hesap | Borç | Alacak |
|------|-----:|-------:|
| 770 Genel Yönetim Giderleri | 10.000 | |
| 335 Personele Borçlar | | 7.000 |
| 360 Ödenecek Vergi ve Fonlar | | 1.500 |
| 361 Ödenecek Sosyal Güvenlik Kesintileri | | 1.500 |
*(Tam bordroda SGK işveren payı ayrıca gider+361 alacak olarak eklenir.)*

## 10. Amortisman ayrılması (dönem sonu)
| Hesap | Borç | Alacak |
|------|-----:|-------:|
| 770 Genel Yönetim Giderleri | 5.000 | |
| 257 Birikmiş Amortismanlar | | 5.000 |

## 11. Kasadan bankaya yatırma
| Hesap | Borç | Alacak |
|------|-----:|-------:|
| 102 Bankalar | 5.000 | |
| 100 Kasa | | 5.000 |

## 12. Alıcıdan çek alınması
| Hesap | Borç | Alacak |
|------|-----:|-------:|
| 101 Alınan Çekler | 18.000 | |
| 120 Alıcılar | | 18.000 |

---

# Üretim İşletmesi — 7/A Maliyet Akışı (sektörel)

## 13. Direkt ilk madde kullanımı (üretime sevk)
| Hesap | Borç | Alacak |
|------|-----:|-------:|
| 710 Direkt İlk Madde ve Malzeme Giderleri | 8.000 | |
| 150 İlk Madde ve Malzeme | | 8.000 |

## 14. Direkt işçilik tahakkuku
| Hesap | Borç | Alacak |
|------|-----:|-------:|
| 720 Direkt İşçilik Giderleri | 5.000 | |
| 335 Personele Borçlar / 360 / 361 | | 5.000 |

## 15. Genel üretim gideri (örn. amortisman)
| Hesap | Borç | Alacak |
|------|-----:|-------:|
| 730 Genel Üretim Giderleri | 2.000 | |
| 257 Birikmiş Amortismanlar | | 2.000 |

## 16. Yansıtma → Yarı mamul/mamul (dönem sonu)
| Hesap | Borç | Alacak |
|------|-----:|-------:|
| 151 Yarı Mamuller | 15.000 | |
| 711 Direkt İlk Madde Yansıtma | | 8.000 |
| 721 Direkt İşçilik Yansıtma | | 5.000 |
| 731 Genel Üretim Gid. Yansıtma | | 2.000 |

Sonra **152 Mamuller / 151**, satışta **620 Satılan Mamuller Maliyeti / 152**.

## 17. Vergiyi doğuran olay örneği — kira stopajı + KDV (tevkifat dışı)
Brüt kira 10.000; %20 stopaj (GV); %20 KDV (sorumlu sıfatıyla 2 no.lu KDV varsa ayrıca).
| Hesap | Borç | Alacak |
|------|-----:|-------:|
| 770 Genel Yönetim Giderleri | 10.000 | |
| 191 İndirilecek KDV | 2.000 | |
| 100 Kasa / 102 Banka | | 10.000 |
| 360 Ödenecek Vergi ve Fonlar (stopaj) | | 2.000 |
*(Vergiyi doğuran olay: kira tahakkuku → stopaj 360'a; KDV teslim/ifa anında.)*
