# Yetkilendirme — Departman + Hiyerarşi Tasarımı (2026-07-09)

> Kullanıcı: "departmanları ilgilendiren bölümler hangi departman için geçerliyse o kısımları görecek;
> departman içi hiyerarşiye göre işlem içerikleri değişecek." Bu tur yalnız TASARIM (kod yok).
> Kaynak: RBAC/ABAC + maker-checker + SoD araştırması. Mevcut temel: rol (admin|kullanici) + mükellef ataması.

## Model: RBAC (rol) × Departman (görünürlük) × Kademe (işlem yetkisi) + SoD

Üç eksen birlikte:
1. **Departman** = hangi MODÜLLERİ görür (görünürlük/kapsam). Muhasebe departmanı defter/fiş görür, Bordro
   departmanı ücret/SGK görür, Denetim departmanı çalışma kağıtları görür.
2. **Kademe** (departman içi hiyerarşi) = o modülde NE YAPABİLİR (işlem derinliği). Aynı Muhasebe modülünde
   eleman fiş TASLAĞI girer, sorumlu KESİNLEŞTİRİR/onaylar, müdür DÖNEM KAPATIR.
3. **SoD (görevler ayrılığı)** = kaydı GİREN ≠ ONAYLAYAN (maker ≠ checker). Denetim standardı; toksik
   kombinasyonu (kendi kaydını onaylama) engeller.

## Departmanlar (SMMM/şirket bağlamı)
| Departman | Gördüğü modüller |
|-----------|------------------|
| Muhasebe | Muhasebe (kayıt/defter/mizan), Banka, Belgeler, Hesap planı |
| Vergi | Vergi (KDV/geçici/takvim/parametre), Mali tablolar |
| Bordro/İK | Ücret tahakkuku, SGK, muhtasar (E fazı) |
| Cari/Tahsilat | Cari kartlar, tahsilat/ödeme, banka mutabakat |
| Denetim | Denetim çalışma programları/kağıtları, analiz |
| Yönetim | Genel bakış, tüm raporlar (okuma), yetki yönetimi |

## Kademeler (maker-checker-approver)
| Kademe | İşlem yetkisi |
|--------|---------------|
| Eleman (maker) | Taslak fiş gir/düzelt; kesinleştiremez |
| Sorumlu (checker) | Taslağı kontrol et → KESİNLEŞTİR; kendi girdiğini onaylayamaz (SoD) |
| Müdür (approver) | İptal (VUK 217), dönem kapatma, tutar eşiği üstü onay |
| Yönetici (admin) | Kullanıcı/departman/yetki yönetimi (mevcut) |

## Veri modeli (data-driven — proaktif ilke)
- **data/departmanlar.json:** departman → görünür modül id'leri (GERCEK/NAV ile eşleşir).
- **data/kademeler.json:** kademe → izin verilen işlemler (taslak_gir, kesinlestir, iptal, donem_kapat,
  kullanici_yonet, onay_esigi_kurus).
- **Kullanıcı genişletmesi:** mevcut Kullanici'ye `departman` + `kademe` alanları. `mukellef_idleri` (var)
  hangi firmalarda çalıştığını; departman+kademe o firmalarda ne yapabildiğini belirler.
- **ABAC dokunuşu:** onay tutar eşiği (kademeye/departmana göre), gider yeri/cost center kapsamı.

## SoD uygulaması (mevcut mimariyle uyum)
Zaten var olan **taslak → kesinleştir** akışı SoD'un doğal iskeleti:
- Maker `taslak` fiş üretir (B.7 Taslak CRUD önkoşul).
- Checker `kesinlestir` çağırır — sistem "kaydı giren ≠ kesinleştiren" kontrolü yapar (yeni kural).
- Kesin fiş değişmezliği + VUK 217 iptal (müdür yetkisi) zaten denetim izini taşıyor.
- Kullanıcı+zaman damgası (B işi) her fişe olusturan/kesinlestiren yazar → SoD denetlenebilir olur.

## UI etkisi
- Sidebar NAV departmana göre süzülür (görünürlük).
- Fiş ekranında kademeye göre düğmeler: eleman "Taslak kaydet", sorumlu "Kesinleştir", müdür "İptal/Kapat".
- Yönetim paneli genişler: kullanıcıya departman + kademe atama; departman→modül ve kademe→işlem matrisleri.

## Sıra (uygulama)
1. Veri modeli: departmanlar.json + kademeler.json + Kullanici'ye departman/kademe. (B.7 Taslak CRUD ile birlikte)
2. Backend: kesinleştirmede SoD kontrolü + kademe işlem yetkisi + olusturan/kesinlestiren damgası.
3. UI: NAV departman süzme + kademeye göre aksiyon düğmeleri + yönetim paneli matrisleri.
> Not: SoD gerçek anlamını kalıcılık + kullanıcı damgası gelince kazanır; şimdilik model + görünürlük.
