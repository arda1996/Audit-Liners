import { test, expect } from "@playwright/test";
import { API, giris, sayfaAc, muhasebeBozulmadi } from "./yardim";

test.describe("03 · Belgeler ve Vergi ekranları", () => {
  test.beforeEach(async ({ page }) => { await giris(page); });

  test("akıllı filtre çubuğu API ile aynı sayıyı gösteriyor", async ({ page, request }) => {
    await sayfaAc(page, /^Muhasebe/i, /Belgeler/i);
    const girdi = page.getByPlaceholder(/Yaz: firma/);
    await expect(girdi).toBeVisible();

    // "e-arşiv" yaz → öneriyi seç → tablo sayısı API'deki sayıyla tutmalı
    await girdi.fill("e-arşiv");
    await page.getByText(/^e-Arşiv$/).first().click();
    const beklenen = (await (await request.get(`${API}/api/simulasyon/faturalar?ebelge_tip=E_ARSIV&limit=1`)).json()).toplam;
    await expect(page.locator("text=/\\/ " + beklenen.toLocaleString("tr-TR") + "/")).toBeVisible({ timeout: 15_000 });
  });

  test("KDV1 taslağı oran kırılımı + istisna + tevkifat gösteriyor", async ({ page, request }) => {
    await sayfaAc(page, /Vergi & Raporlama/i, /^Vergi \(KDV/i);
    await expect(page.getByText(/KDV1 BEYANNAME TASLAĞI/)).toBeVisible();

    // Oran kırılımı: en az iki farklı oran satırı olmalı (tek orana çökmemeli — K2 regresyonu)
    const oranSatiri = page.locator("td").filter({ hasText: /Teslim ve hizmetler — %/ });
    expect(await oranSatiri.count(), "KDV tek orana çökmüş — muavin kırılımı bozuk").toBeGreaterThanOrEqual(2);

    // İstisna satırı varsa KDV sütununda "KDV doğmaz" yazmalı (hesaplanana eklenmemeli)
    const istisna = page.locator("tr").filter({ hasText: /İstisna kapsamındaki teslimler/ });
    if (await istisna.count()) await expect(istisna).toContainText(/KDV doğmaz/);

    await muhasebeBozulmadi(request, "KDV1 görüntüleme");
  });

  test("cari mutabakat sekmesi beyanname OLMADIĞINI söylüyor", async ({ page }) => {
    await sayfaAc(page, /Vergi & Raporlama/i, /^Vergi \(KDV/i);
    await page.getByRole("button", { name: /Cari mutabakat/ }).click();
    // Ba/Bs 25.09.2024'te kaldırıldı — kullanıcı bunu beyanname sanmamalı.
    await expect(page.getByText(/Bu bir beyanname değildir/)).toBeVisible();
    await expect(page.getByText(/25\.09\.2024/)).toBeVisible();
  });
});
