import { test, expect } from "@playwright/test";
import { API, giris, sayfaAc, saglik, muhasebeBozulmadi } from "./yardim";

// Belge → muhasebe kaydı akışının UI tarafı. Buradaki asıl soru yine defterde:
// arayüzden kurulan fiş DOĞRU HESAPLARA mı gidiyor, mizan denk kalıyor mu.
/** Sıradaki alanı doldurur ve onaylar. Alanın açılır liste mi metin mi olduğunu EKRANDAN
 *  anlar — testin alan sırasını ezberlemesi gerekmez, sözleşme değişince kırılmaz. */
async function doldur(page: import("@playwright/test").Page, deger: string) {
  const kutu = page.getByTestId("aktif-alan");
  await expect(kutu).toBeVisible();
  // Hangi alandayız? Onaydan sonra bu DEĞİŞMELİ — değişmezse akış tıkanmıştır ve
  // sıradaki değer yanlış alana yazılır. Eskiden bunu beklemiyorduk ve test, ikinci turda
  // "yön" açılır listesine firma adı seçmeye çalışıp zaman aşımına düşüyordu.
  const oncekiBaslik = (await kutu.locator("div").first().textContent())?.trim() ?? "";

  const secenek = kutu.getByTestId("alan-secenek");
  if (deger !== "") {
    if (await secenek.count()) await secenek.selectOption(deger);
    else await kutu.getByTestId("alan-metin").fill(deger);
    await page.getByRole("button", { name: /^Değeri al$/ }).click();
  }
  // Boş değer = "bu belgede yok"; isteğe bağlı alan atlanabilmeli.
  const onay = page.getByRole("button", { name: /Onayla ve devam|Bu belgede yok/ });
  await expect(onay, `'${oncekiBaslik}' için değer alınmadı — onay düğmesi açılmadı`).toBeEnabled();
  await onay.click();

  // Ya sıradaki alana geçilmiş olmalı ya da alanlar bitmiş olmalı.
  await expect(async () => {
    if (!(await kutu.count())) return; // alanlar bitti
    const simdiki = (await kutu.locator("div").first().textContent())?.trim() ?? "";
    expect(simdiki, `'${oncekiBaslik}' onaylandıktan sonra sıradaki alana geçilmedi`)
      .not.toBe(oncekiBaslik);
  }).toPass({ timeout: 10_000 });
}

test.describe("04 · Belge alım (UI) — fatura → fiş", () => {
  test.beforeEach(async ({ page }) => {
    await giris(page);
    await sayfaAc(page, /^Muhasebe/i, /Belge yükle/i);
  });

  test("parametreler SIRAYLA sunuluyor ve ilk alan yönlendiriliyor", async ({ page }) => {
    await expect(page.getByRole("heading", { name: /Belge alım/i })).toBeVisible();
    await page.getByRole("button", { name: /Alımı başlat/i }).click();

    // Fatura varsayılan tür — sıra sözleşmesi ekranda görünmeli.
    const satirlar = page.locator("table tbody tr");
    await expect(satirlar.first()).toContainText("Fatura yönü");
    await expect(satirlar.nth(1)).toContainText("Karşı taraf");

    // Kullanıcı ilk alana yönlendirilmeli, ipucuyla birlikte.
    await expect(page.getByText(/1\. Fatura yönü/)).toBeVisible();
    await expect(page.getByText(/bize mi kesildi/i)).toBeVisible();
  });

  test("seçim ONAYSIZ ilerlemiyor — 'Muhasebe kaydını kur' erken çıkmıyor", async ({ page }) => {
    await page.getByRole("button", { name: /Alımı başlat/i }).click();
    // Zorunlu alanlar dururken kayıt kurma düğmesi hiç görünmemeli.
    await expect(page.getByRole("button", { name: /Muhasebe kaydını kur/i })).toHaveCount(0);
  });

  test("ALIŞ faturası doldurulunca fiş 153/191/320'ye kuruluyor ve DEFTER bozulmuyor",
    async ({ page, request }) => {
    const oncesi = await saglik(request);
    await page.getByRole("button", { name: /Alımı başlat/i }).click();

    for (const d of ["ALIS", "EGE TEKSTİL A.Ş.", "1234567890", "EGE2026000123",
                     "15.03.2026", "10.000,00", "2.000,00", "12.000,00",
                     ""]) { // tevkifat yok — isteğe bağlı alan ATLANABİLMELİ
      await doldur(page, d);
    }

    await page.getByRole("button", { name: /Muhasebe kaydını kur/i }).click();

    // Fiş taslağı DOĞRU hesapları göstermeli — alışta stok + indirilecek KDV + satıcı borcu.
    const taslak = page.getByTestId("fis-taslak");
    await expect(taslak).toContainText("153");
    await expect(taslak).toContainText("191.06");   // %20 KDV muavini ARDIŞIK koddur
    await expect(taslak).toContainText("320");
    await expect(taslak).toContainText("(denk)");

    // Okuma/hazırlık defteri değiştirmemeli — fiş henüz yazılmadı.
    const sonrasi = await saglik(request);
    expect(sonrasi.defter.fis, "taslak üretmek deftere kayıt attı").toBe(oncesi.defter.fis);
    await muhasebeBozulmadi(request, "fiş taslağı sonrası");
  });

  test("tutarsız fatura kayda geçmiyor (matrah + KDV ≠ toplam)", async ({ page, request }) => {
    await page.getByRole("button", { name: /Alımı başlat/i }).click();
    for (const d of ["SATIS", "X A.Ş.", "1234567890", "N-1", "01.02.2026",
                     "1.000,00", "200,00", "1.500,00", ""]) { // 1000+200 ≠ 1500
      await doldur(page, d);
    }
    // Denklem ihlali ekranda ENGEL olarak görünmeli.
    await expect(page.getByText(/A21/)).toBeVisible();
    await expect(page.getByText(/TUTMUYOR/)).toBeVisible();
    await muhasebeBozulmadi(request, "tutarsız belge sonrası");
  });

  test("API sözleşmesi: şablon uçları arayüzün beklediği alanları veriyor", async ({ request }) => {
    const s = await (await request.get(`${API}/api/belge/alim/sablonlar`)).json();
    expect(Object.keys(s.modlar)).toEqual(expect.arrayContaining(["OTOMATIK", "SECIMLI", "MANUEL"]));
    const fatura = s.belge_turleri.FATURA;
    expect(fatura, "FATURA şablonu yok").toBeTruthy();
    const kodlar = fatura.parametreler.map((p: any) => p.kod);
    expect(kodlar[0], "ilk sorulan alan yön olmalı — fiş yönü buna bağlı").toBe("yon");
    expect(kodlar).toEqual(expect.arrayContaining(["unvan", "belge_no", "tarih", "matrah", "kdv", "toplam"]));
    // Her alanın ipucu olmalı; kullanıcı "ne seçeceğim" diye sormamalı.
    for (const p of fatura.parametreler) {
      expect(p.ipucu?.length ?? 0, `${p.kod} için ipucu yok`).toBeGreaterThan(5);
    }
  });
});
