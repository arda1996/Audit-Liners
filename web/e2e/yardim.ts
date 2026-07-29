// E2E test yardımcıları.
//
// ÇEKİRDEK FİKİR: Bu testler yalnız arayüzü değil, arayüzün DEFTERE ne yaptığını da doğrular.
// Genel bir test aracı "buton çalıştı" der; biz her UI eyleminden sonra
// "para korundu mu, mizan denk mi, ters bakiye doğdu mu" diye soruyoruz.
// Muhasebede asıl hatalar ekranda değil defterde görünür — bu ay bulduğumuz her gerçek
// hata (iptal/iade, ÖTV zinciri, sessiz eşleştirme, ters bakiye) böyleydi.

import { expect, type Page, type APIRequestContext } from "@playwright/test";

export const API = process.env.API_URL ?? "http://127.0.0.1:8787";

// Geliştirme ortamının tohum kullanıcısı (crates/api/src/main.rs içinde seed edilir).
// Gerçek bir sır değildir; ortam değişkeniyle geçersiz kılınabilir.
export const KULLANICI = process.env.TEST_KULLANICI ?? "admin";
export const PAROLA = process.env.TEST_PAROLA ?? "audit2026";

/** Muhasebe sağlık fotoğrafı — değişmezler + defter dengesi. */
export type Saglik = {
  saglikli: boolean;
  degismez_ihlali: number;
  ters_bakiye: number;
  degismezler: { kod: string; mesaj: string }[];
  ters_bakiyeler: { kod: string; ad: string; bakiye: number }[];
  defter: { fis: number; borc: number; alacak: number; denge_ok: boolean };
};

export async function saglik(req: APIRequestContext): Promise<Saglik> {
  const r = await req.get(`${API}/api/kontrol/saglik`);
  expect(r.ok(), "sağlık ucu yanıt vermedi — API ayakta mı?").toBeTruthy();
  return r.json();
}

/**
 * Muhasebe DEĞİŞMEZLERİ bozulmamış olmalı. Her UI eyleminden sonra çağrılır.
 * Bozulursa hata mesajı hangi kuralın kırıldığını söyler — "test failed" demek yetmez.
 */
export async function muhasebeBozulmadi(req: APIRequestContext, baglam = "") {
  const s = await saglik(req);
  const ek = baglam ? ` (${baglam})` : "";
  expect(s.degismezler.map((x) => `[${x.kod}] ${x.mesaj}`), `DEĞİŞMEZ İHLALİ${ek} — motor hatası`).toEqual([]);
  expect(s.defter.borc, `Defter dengesi bozuldu${ek}`).toBe(s.defter.alacak);
  expect(
    s.ters_bakiyeler.map((x) => `${x.kod} ${x.ad}: ${(x.bakiye / 100).toFixed(2)}`),
    `TERS BAKİYE${ek} — her ters bakiyenin açıklanabilir sebebi olmalı`,
  ).toEqual([]);
}

/** Defteri sıfırdan kurar (senkronize + aktar). Testler aynı taban veriden başlasın diye. */
export async function defteriHazirla(req: APIRequestContext) {
  await req.post(`${API}/api/tahsis/senkronize`);
  await req.post(`${API}/api/muhasebe/aktar`);
}

/** Uygulamaya giriş yapar ve ana ekranın yüklendiğini doğrular. */
export async function giris(page: Page) {
  await page.goto("/");
  // Zaten girilmişse tekrar deneme.
  const girisBtn = page.getByRole("button", { name: /giriş yap/i });
  if (await girisBtn.count()) {
    // Etiketler input'a bağlı değil (htmlFor yok) — placeholder ve tip ile seçiyoruz.
    await page.getByPlaceholder("kullanıcı adı").fill(KULLANICI);
    await page.locator('input[type="password"]').fill(PAROLA);
    await girisBtn.click();
  }
  await expect(page.locator("aside.side"), "giriş sonrası kenar çubuğu görünmeli").toBeVisible({ timeout: 15_000 });
}

/** Sidebar'daki bir gruba gelip alt menüden sayfa seçer (uçan pencere deseni). */
export async function sayfaAc(page: Page, grup: RegExp, sayfa: RegExp) {
  await page.locator("aside.side").getByRole("button", { name: grup }).hover();
  await page.getByRole("button", { name: sayfa }).click();
}

/** Kuruş → okunur TL (test mesajlarında kullanmak için). */
export const tl = (k: number) => (k / 100).toLocaleString("tr-TR", { minimumFractionDigits: 2 });
