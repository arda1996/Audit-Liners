# Backend Kurgusu (dökümantasyon → modül haritası)

33 doküman analizinden (`docs/analiz/`) türeyen mimari. İlke: **domain saf ve bağımsız (hexagonal merkez);
her modül domain'e bağlanır. VUK/TDHP = çekirdek defter; TMS = üstüne binen overlay katman.**

## Crate / modül haritası
| Crate/modül | Sorumluluk | Durum |
|-------------|-----------|-------|
| `domain` (saf) | Çift taraflı defter, TDHP türetme, muavin, cari, dönem kapanış/açılış virman (vergi karşılıklı), mali tablolar, kurallar. Para=kuruş(i64). | **VAR** (29 test) |
| `domain::vergi` | Vergiyi doğuran olay → KDV/stopaj/damga satırı (matrah×oran); aylık KDV mahsubu (391→191, fark 360/190); tevkifat. Ticari kâr→mali kâr köprüsü (KKEG/istisna). | sıradaki |
| `domain::degerleme` | Dönem sonu: amortisman (VUK+TMS dual), reeskont (simetri zorunlu), şüpheli alacak, kur (parasal bayrağı), karşılık → fiş üreticileri. | plan |
| `domain::bordro` | Ücret brüt→net, SGK, GV stopajı (kümülatif matrah), damga; yıl-etkin parametre tablosu. | plan |
| `domain::tms` (overlay) | Dual-değer (vuk_degeri/tms_degeri), geçici/kalıcı fark → ertelenmiş vergi (TMS12), dipnot. | plan (ayrıştırıcı) |
| `domain::denetim` | BDS/analitik testler (reeskont simetrisi, anormal yevmiye), ilişkili taraf (TMS24), değişmez denetim izi. | plan |
| `db` | sqlx + gömülü Postgres adaptörü (repository). | plan |
| `api` (axum) | Tüm modülleri HTTP uçlarına açar; bellek→db. | **VAR** (bellek) |
| `data/` | tdhp.csv, hesap-kurallari.json, hesap-aciklamalari.json, **parametreler.json** (yıl-etkin oranlar — plan). | kısmen |

## Standart-farkındalıklı kural motoru (ayrıştırıcı)
Bulgulardaki "yapılmaması gerekenler" doğrudan **gömülü validasyon**a dönüşür:
LIFO'yu engelle · stok sınıf-bazı NGD indirgemeyi blokla · reeskont simetrisi zorla ·
şerefiye değer düşüklüğü iptalini yasakla · kapalı döneme kayıt engelle · dayanaksız fiş uyar.

## Katman kararı: VUK çekirdek + TMS overlay
- Çekirdek defter **VUK/TDHP** (herkes; SMMM günlük işi).
- **TMS opsiyonel overlay**: her varlık/hesap için `vuk_degeri` + `tms_degeri`; fark → ertelenmiş vergi.
- Firma profilinde `raporlama_cercevesi` (VUK / BOBİ FRS / TMS) bayrağı motorları dallandırır.
- Enflasyon altyapısı: hesaba `parasal:bool` + `iktisap_tarihi` (VUK Geç.37 ile 2025-27 durdurma; 2028 için hazır).
