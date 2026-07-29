# e-Fatura / e-Arşiv Entegrasyonu + Fatura Sayfası — Tasarım (2 ajan araştırması, 2026-07-21)

> Kaynak: GİB e-Fatura Entegrasyon Kılavuzu v1.9, e-Arşiv Portal Özel Entegratör Kılavuzu, İzibiz/Turkcell/
> Uyumsoft/EDM/SOVOS API dokümanları, Paraşüt/Logo/Mikro UX. Belge daima **UBL-TR 1.2 XML**.

## 1. İletim yolları (üçü de aynı UBL-TR'yi üretir, ileti farklı)
| Yol | Kime | Mühür | Bizim için |
|-----|------|-------|-----------|
| **GİB Portal** (earsivportal/ivd) | düşük hacim | portalda | resmî API'si yok (dispatch reverse) → **üretimde kullanma** |
| **Doğrudan entegrasyon** (EF-VAP, SOAP/zarf) | yüksek hacim, kendi BT | mükellefin mührü | BİS raporu+test+7/24 → ağır, sonraya |
| **Özel entegratör** (İzibiz/Turkcell/Uyumsoft/EDM/SOVOS) | çoğunluk | genelde entegratörde | ✅ **bizim yol** — WS'e bağlan |

**Zorunluluk (509 VUK):** e-Fatura ciro ≥ **3M TL** (e-ticaret/gayrimenkul/taşıt **500k**); e-Fatura mükellefi
otomatik e-Arşiv'e de girer. Karşı taraf **e-Fatura mükellefiyse e-Fatura, değilse e-Arşiv** (otomatik yönlendirme).

## 2. Entegratör API deseni — 5 fiil (hepsinde aynı)
`login/auth → send → list(inbox) → getWithType(indir UBL/PDF/HTML) → getStatus`
- **İzibiz/Uyumsoft/EDM:** SOAP/WSDL (`Login`, `SendInvoice`, `GetInvoice`, `GetInvoiceWithType`, `GetInvoiceStatus`, `MarkInvoice`, `SendInvoiceResponseWithServerSign`).
- **Turkcell/SOVOS:** REST/JSON — **en temiz şablon:** `POST /v1/outboxinvoice/create` · `GET /v1/inboxinvoice/list` · `GET /v1/outboxinvoice/{id}/pdf|html|ubl` · `GET /v1/outboxinvoice/{id}/status`.

## 3. İç API sözleşmesi (bizim) — Turkcell REST desenini taklit et + adapter
UI tek iç API görür; her entegratör için **adapter** yazılır (entegratör değiştirmek = adapter değiştirmek).
```
/api/efatura/giden        (liste)       /api/efatura/gelen     (inbox)
/api/efatura/:id          (detay)       /api/efatura/:id/ubl|pdf|html (indir)
/api/efatura/:id/durum    (status)      POST /api/efatura/olustur / :id/gonder / :id/kabul|ret|iptal
```
`trait EFaturaEntegrator { login; gonder(ubl); gelen_listesi; detay; durum; yanit(kabul/ret) }` → Izibiz/Turkcell impl.

## 4. Fatura sayfası UX (Paraşüt/Logo deseni)
- **Sekmeler:** **Giden · Gelen · e-Arşiv · Taslak**.
- **Kolonlar:** no · tarih · cari · tutar · **durum rozeti** · tip (e-Fatura/e-Arşiv).
- **Durum enum (GİB kodları → dahili):** giden 100–141, gelen 122–133 →
  `TASLAK · GONDERILDI · ALINDI(temel) · ONAY_BEKLIYOR(ticari 7g) · KABUL · RED · IADE · IPTAL`.
- **Oluşturma formu:** cari seç · kalem (ürün/miktar/fiyat/KDV) · **KDV muafiyet sebebi (KDV=0 zorunlu)** ·
  **e-Fatura/e-Arşiv otomatik yönlendirme** (VKN sorgusu).
- **Detay:** PDF / HTML / **UBL** üç format.
- **Aksiyonlar:** Görüntüle · Kabul/Ret (gelen ticari) · İndir · **Kayda dönüştür (muhasebeleştir)** · Gönder · İptal/İade · Yazdır.

## 5. Fatura ↔ Belgeler ayrımı (bize birebir uyuyor)
- **Belgeler** = entegratörden düşen **ham gelen kutusu** (değişmez, dayanak/otorite; henüz muhasebeleşmemiş).
- **Faturalar** = bu belgeden türeyen **muhasebe/iş kaydı** (cari+stok hareketi doğuran). Belge→fatura **izli** dönüşüm
  ([[beyanname-doktrini]]: muhasebe kaydı=dayanak, belge=otorite; deftere manuel müdahale katmanı burada).

## 6. Uygulama sırası
1. **(bu tur) Fatura sayfası UI** — sekmeler + durum rozeti + liste, simülasyon verisiyle (e-tip + e-durum eklendi).
2. Oluşturma formu + otomatik e-Fatura/e-Arşiv yönlendirme.
3. `EFaturaEntegrator` trait + mock adapter (gerçek entegratör credential'ı gelince Izibiz/Turkcell impl).
4. Belge→fatura izli dönüşüm + kayda dönüştürme.
> DOĞRULANMADI (araştırmadan): doğrudan entegrasyon EK-3 kesin WSDL metod adları; e-Arşiv tekil eşiği güncel TL; iptal/itiraz noter/KEP süresi. Gerçek entegrasyonda GİB EK-3 + güncel tebliğden teyit.
