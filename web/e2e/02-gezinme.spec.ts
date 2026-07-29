import { test, expect } from "@playwright/test";
import { giris, sayfaAc, muhasebeBozulmadi } from "./yardim";

test.describe("01 · Giriş ve gezinme", () => {
  test.beforeEach(async ({ page }) => { await giris(page); });

  test("sidebar sade — Finans ayrı başlık DEĞİL, Muhasebe altında", async ({ page }) => {
    // Sabit sayı iddia etmiyoruz (admin'de "Sistem" grubu da eklenir) — DAVRANIŞ ölçüyoruz:
    // gruplar birleştirildi mi, Banka/Belgeler Muhasebe'nin altına mı alındı?
    const yan = page.locator("aside.side");
    for (const g of [/^Muhasebe/, /Vergi & Raporlama/, /Bağımsız Denetim/, /^Tanımlar/]) {
      await expect(yan.getByRole("button", { name: g })).toBeVisible();
    }
    await expect(yan.getByRole("button", { name: /^Finans$/ }), "Finans ayrı başlık olmamalı").toHaveCount(0);

    // Banka ve Belgeler artık Muhasebe'nin uçan penceresinde
    await yan.getByRole("button", { name: /^Muhasebe/ }).hover();
    await expect(page.getByRole("button", { name: /^Banka$/ })).toBeVisible();
    await expect(page.getByRole("button", { name: /Belgeler/ })).toBeVisible();
  });

  test("uçan menü sayfa içeriğinin ÜSTÜNDE kalıyor", async ({ page }) => {
    // Regresyon: menü sidebar'ın yığılma bağlamına hapsolup sayfa kutularının altında kalıyordu.
    // Portal + z-index ile çözüldü; burada gerçekten tıklanabilir mi diye bakıyoruz.
    await page.locator("aside.side").getByRole("button", { name: /Vergi & Raporlama/i }).hover();
    const oge = page.getByRole("button", { name: /^Vergi \(KDV/i });
    await expect(oge).toBeVisible();
    await oge.click();                        // örtülüyse Playwright tıklayamaz → test kırılır
    await expect(page.locator("h1")).toContainText(/Vergi/i);
  });

  test("Muhasebe menüsünden Havada bakiye sayfası açılıyor", async ({ page, request }) => {
    await sayfaAc(page, /^Muhasebe/i, /Havada bakiye/i);
    await expect(page.locator("h1")).toContainText(/Havada/i);
    await muhasebeBozulmadi(request, "gezinme sonrası");
  });
});
