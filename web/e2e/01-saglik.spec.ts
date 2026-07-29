import { test, expect } from "@playwright/test";
import { API, saglik, defteriHazirla, muhasebeBozulmadi } from "./yardim";

// Tüm E2E paketinin ÖN KOŞULU: sistem daha hiçbir UI eylemi yapılmadan sağlıklı olmalı.
// Bozuksa sonraki testlerin "kırıldı" demesi yanıltıcı olur — hata zaten vardı.
test.describe("00 · Taban sağlık", () => {
  test("API ayakta ve defter kurulabiliyor", async ({ request }) => {
    await defteriHazirla(request);
    const s = await saglik(request);
    expect(s.defter.fis, "defter boş — aktarım çalışmıyor").toBeGreaterThan(0);
    expect(s.defter.denge_ok, "Σborç ≠ Σalacak").toBe(true);
  });

  test("hiçbir değişmez ihlali ve ters bakiye yok", async ({ request }) => {
    await muhasebeBozulmadi(request, "taban durum");
  });

  test("KDV1 taslağı 12 ayın hepsinde üretiliyor", async ({ request }) => {
    // LCG dağılım hatası regresyonu: bir ara tek aylar dolu, çift aylar boştu.
    const bos: number[] = [];
    for (let ay = 1; ay <= 12; ay++) {
      const d = await (await request.get(`${API}/api/kdv-beyanname?ay=${ay}`)).json();
      if ((d.hesaplanan_toplam ?? 0) === 0) bos.push(ay);
    }
    expect(bos, "şu aylarda beyanname boş").toEqual([]);
  });
});
