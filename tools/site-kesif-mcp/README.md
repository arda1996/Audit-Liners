# site-kesif-mcp — Site Keşif MCP Sunucusu

Audit-Liners **dış veri katmanının** (docs/tasarim/dis-veri-katmani.md) keşif aracı. Bir veri kaynağının
(TCMB, EVDS, GİB, Damodaran, borsa…) web arayüzünü **kendi Chrome'unla** gezerken arka plandaki ağ
trafiğini dinler, XHR/fetch **endpoint'lerini çıkarır**, seçilen çağrıyı **taklit eder** ve sonucu
`data/veri-kaynaklari.json` ile uyumlu bir **profile** döker. DevTools "Copy as fetch" akışının otomatik hâli.

## Yetki ve sınırlar (önemli)
- Yalnız **senin tarayıcına** bağlanır; yalnız **senin açtığın** sayfaların gördüğü trafiği gözlemler.
- Yalnız entegre etmeye **yetkili** olduğun (kamuya açık / kendi hesabın olan) kaynaklara yönelt.
- Kimlik doğrulama aşma, hız sınırı zorlama, kitlesel kazıma için **değildir**; kaynağın kullanım
  şartlarına ve robots kurallarına uy. Tetiği kullanıcı çeker (hedefi sen gösterirsin).

## Akış
1. **`chrome_baslat`** → verdiği komutu terminalde çalıştır (kalıcı keşif profili + hata ayıklama portu).
   Girişli kaynak varsa (EVDS gibi) o pencerede bir kez giriş yap.
2. **`chrome_baglan`** → Chrome'a bağlan, dinleme açılır.
3. **`hedef_ac`** (veya tarayıcıda elle gez) → kaynağın veri sayfasına git.
4. **`yakalananlar`** → benzersiz endpoint listesi (json/xml önce). `suzgec` ile daralt.
5. **`cagri_detay`** → bir endpoint'in tam istek/yanıt + şema özeti.
6. **`cagri_tekrarla`** → çağrıyı program dışından taklit et (parametre değiştir, callback dene).
7. **`profil_uret`** → `veri-kaynaklari.json` taslağı üret (kod, erişim, veri alanları, kullanım).

## Kurulum
```
cd tools/site-kesif-mcp
npm install
npm run build
```
`.mcp.json` projeye kayıtlı (`site-kesif`). Claude Code yeniden başlatınca araçlar yüklenir.

## Mimari notu
Bu araç dış veri katmanının **keşif ucu**dur: çıktısı (endpoint + profil) → `crates/ingest` adaptörünün
(J.2) girdisi olur. Dinamik token / imza gerektiren (çerezle çözülemeyen) endpoint'ler için çağrı
tarayıcı içinden yapılmalı — bunu ingest adaptörü CDP üzerinden üstlenir (rota: J.3+).
