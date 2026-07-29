# Çoklu Mükellef — Veritabanı Ayrıştırma Kararı

> Karar tarihi: 2026-07-09. Kalıcılık fazına kadar (🔒 muhasebe A-Z) uygulanmaz; ama `db` crate'ini
> doğru tasarlamak için model şimdi sabitlendi. Bugünkü bellek-içi "aktif+arşiv swap" bunun ön provası.

## Karar: DB-per-mükellef + registry DB + dönem-partition

Her mükellefin defteri **ayrı bir PostgreSQL veritabanında** tutulur. Küçük bir **registry (kayıt)
veritabanı** mükellef listesini, profillerini (unvan/VKN/sektör/maliyet seçeneği) ve sektör kataloğunu
tutar; hangi client DB'ye bağlanılacağını yönlendirir. Dönem (yıl) mükellef DB'sinin **içinde** partition
olarak yaşar (DB-per-mükellef-per-yıl DEĞİL).

## Neden (bağlamımız)
- **İzolasyon sert gereklilik:** iki müşterinin defteri fiziksel olarak birbirini göremez → sızıntı
  imkânsız. Muhasebe/hukuk açısından zorunlu.
- **Domain modeline birebir:** her mükellef ayrı tüzel kişilik + ayrı defter; SMMM pratiğinde ayrı firma
  dosyası. Çapraz sorgu neredeyse hiç gerekmez.
- **Müşteri yaşam döngüsü:** yeni = yeni DB; kapanan = pg_dump ile arşivle/DROP; devir = tek dump ile teslim.
- **Mevcut swap deseninin ikizi:** `profiller`=registry, `arsiv`=per-mükellef defter. Postgres'te swap =
  "aktif mükellefin DB'sine bağlan". Geçiş temiz.

## Maliyet (kabul edilen)
| Kalem | Etki | Karşılık |
|-------|------|----------|
| Migrasyon | N DB'de döngü | Tek seferlik "migrasyon koşucusu" altyapısı |
| Depolama | DB başına ~7-8MB katalog (300 mükellef ≈ 2GB boş) | Masaüstünde kabul edilebilir |
| Çapraz raporlama | Portföy görünümü N DB'ye fan-out | Nadir; ayrı toplayıcı |
| Bağlantı | Mükellef değişince reconnect | Masaüstü tek kullanıcıda ihmal edilebilir |

## Reddedilen alternatifler
- **Schema-per-mükellef:** izolasyon mantıksal (search_path hatası sızdırır); yedek/devir daha zor. Depolama
  avantajı var ama izolasyon garantisi bizim için yetersiz.
- **Tek şema + mukellef_id kolonu:** en ucuz + çapraz sorgu kolay AMA izolasyon yalnız uygulama katmanında;
  unutulan bir WHERE = sızıntı. Muhasebe ürününde kabul edilemez risk (RLS ile hafifler, yine de fiziksel değil).

## Çekince (gelecek)
Bulut/SaaS binlerce kiracıya çıkarsa (hedef DEĞİL — masaüstü öncelikli) o kanal için schema-per veya
paylaşılan-şema + RLS yeniden değerlendirilir. Karar masaüstü + gömülü Postgres olduğundan DB-per doğru.

## Mimari etki (hexagonal)
- Domain'e DOKUNMAZ. `db` crate bir `DefterDeposu` port'u sunar; "hangi DB" bir altyapı ayrıntısıdır.
- Bugünkü bellek-swap ile yarınki DB-per **aynı port'un iki uygulaması** — domain fark etmez.
- Registry DB: `mukellef(id, unvan, vkn, sektor_kodlari, maliyet_secenegi, db_adi, olusturma)`.
- Client DB şeması: fis/yevmiye_satiri/hesap_muavin/... + dönem partition anahtarı.
- Bağlantı yönlendirme: aktif mükellef → connection pool o DB'ye; değişince pool swap (bugünkü
  `mukellef_aktif` ucunun kalıcı karşılığı).

## Sıra
Kalıcılık turu (F.2) geldiğinde: (1) registry DB + migrasyon koşucusu, (2) client DB şeması, (3) bağlantı
yönlendirici, (4) bellek-swap → DB-per geçişi (aynı port). Şimdilik bellek-swap yeterli.
