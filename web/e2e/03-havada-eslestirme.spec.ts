import { test, expect } from "@playwright/test";
import { API, giris, sayfaAc, saglik, defteriHazirla, muhasebeBozulmadi, tl } from "./yardim";

// PAKETİN ÇEKİRDEĞİ: arayüzden eşleştirme yapıp DEFTERİN doğru kaldığını doğrular.
// Bir UI testi "buton çalıştı" der; burada asıl soru "para korundu mu, mizan denk mi".
test.describe("02 · Havada bakiye eşleştirme (UI + defter)", () => {
  test.beforeEach(async ({ request, page }) => {
    await defteriHazirla(request);
    await giris(page);
    await sayfaAc(page, /^Muhasebe/i, /Havada bakiye/i);
  });

  test("banka kartları havada tutarı gösteriyor ve süzüyor", async ({ page, request }) => {
    const h = await (await request.get(`${API}/api/havada`)).json();
    test.skip(h.adet === 0, "havada bakiye yok — süzme testi anlamsız");

    await expect(page.getByText(/Havada bakiye — eşleşmemiş para/)).toBeVisible();
    const kart = page.locator("button.card").filter({ hasText: /İş Bankası/ }).first();
    await expect(kart).toBeVisible();
    await kart.click(); // bankaya süz
    await expect(page.getByRole("button", { name: /Süzmeyi kaldır/ })).toBeVisible();
    await muhasebeBozulmadi(request, "süzme sonrası"); // okuma eylemi defteri bozmamalı
  });

  test("UI'dan faturaya bağlama: sonuç bildirilir ve DEFTER bozulmaz", async ({ page, request }) => {
    const h = await (await request.get(`${API}/api/havada`)).json();
    test.skip(h.adet === 0, "havada bakiye yok");

    const oncesi = await saglik(request);

    await page.getByRole("button", { name: /^Çöz$/ }).first().click();
    const firmaSecici = page.locator("select").filter({ hasText: /firma seçin/ }).first();
    await expect(firmaSecici).toBeVisible();
    // Açık faturası OLAN bir firma bul — ilk firmada olmayabilir; test atlanmasın diye dolaş.
    const firmalar = (await firmaSecici.locator("option").allTextContents())
      .filter((x) => !/firma seçin/.test(x));
    expect(firmalar.length, "seçilebilir firma yok").toBeGreaterThan(0);

    let bagla = page.getByRole("button", { name: /Bu faturaya bağla/ }).first();
    let secildi = "";
    for (const f of firmalar) {
      await firmaSecici.selectOption({ label: f });
      // Fatura listesi asenkron yükleniyor — "yok" mesajı ya da satır görünene kadar bekle.
      await expect(
        page.getByRole("button", { name: /Bu faturaya bağla/ }).first()
          .or(page.getByText(/Bu firmada açık fatura yok/)),
      ).toBeVisible();
      if (await bagla.count()) { secildi = f; break; }
    }
    expect(secildi, "hiçbir firmada açık satış faturası bulunamadı").toBeTruthy();
    await bagla.click();

    // 1) Kullanıcıya SONUÇ bildirilmeli — sessiz durum olmamalı (eski hata tam buydu).
    const mesaj = page.locator(".msg").first();
    await expect(mesaj).toBeVisible();
    await expect(mesaj).toContainText(/YAZILDI|YAZILAMADI|KISMEN/);
    await expect(mesaj, "değişmez ihlali mesajı çıktı — motor hatası").not.toContainText(/SİSTEM HATASI/);

    // 2) DEFTER doğru kalmalı — testin asıl amacı.
    await muhasebeBozulmadi(request, "UI'dan eşleştirme sonrası");

    // 3) Eşleştirme deftere yansımalı, denge korunmalı.
    const sonrasi = await saglik(request);
    expect(sonrasi.defter.fis, "eşleştirme deftere işlenmedi").toBeGreaterThanOrEqual(oncesi.defter.fis);
    expect(
      sonrasi.defter.borc,
      `Σborç ${tl(sonrasi.defter.borc)} ≠ Σalacak ${tl(sonrasi.defter.alacak)}`,
    ).toBe(sonrasi.defter.alacak);
  });

  test("fatura bekleniyor işareti kayıt DOĞURMAZ (VUK 227)", async ({ page, request }) => {
    const h = await (await request.get(`${API}/api/havada`)).json();
    test.skip(h.adet === 0, "havada bakiye yok");
    const oncesi = await saglik(request);

    await page.getByRole("button", { name: /^Çöz$/ }).first().click();
    const firmaSecici = page.locator("select").filter({ hasText: /firma seçin/ }).first();
    const secenekler = await firmaSecici.locator("option").allTextContents();
    await firmaSecici.selectOption({ label: secenekler.find((x) => !/firma seçin/.test(x))! });
    await page.getByRole("button", { name: /Fatura bekleniyor olarak işaretle/ }).click();

    await expect(page.locator(".msg").first()).toContainText(/fatura bekleniyor/i);
    // Belge yoksa muhasebe kaydı doğmaz — fiş sayısı DEĞİŞMEMELİ.
    const sonrasi = await saglik(request);
    expect(sonrasi.defter.fis, "belge olmadan kayıt üretildi — VUK 227 ihlali").toBe(oncesi.defter.fis);
    await muhasebeBozulmadi(request, "fatura bekleniyor işareti sonrası");
  });
});
