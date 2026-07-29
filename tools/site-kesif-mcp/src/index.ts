#!/usr/bin/env node
/**
 * site-kesif-mcp — Audit-Liners dış veri katmanı keşif aracı.
 *
 * Amaç: bir veri kaynağının (TCMB, EVDS, GİB, Damodaran, borsa vb.) web arayüzünü kendi Chrome'unla
 * gezerken ARKA PLANDAKİ ağ trafiğini dinlemek, XHR/fetch endpoint'lerini çıkarmak, seçilen çağrıyı
 * program dışından TAKLİT ETMEK (call/callback) ve sonucu ingest adaptörüne / veri-kaynaklari.json'a
 * uygun bir profile dökmek. DevTools "Copy as fetch" akışının otomatik ve tekrarlanabilir hâli.
 *
 * Yetki ilkesi: kendi tarayıcına bağlanır, senin açtığın sayfaların gördüğü trafiği gözlemler.
 * Yalnız entegre etmeye YETKİLİ olduğun (kamuya açık / kendi hesabın olan) kaynaklara yönelt;
 * kimlik doğrulama aşma, hız sınırı zorlama, kitlesel kazıma amaçlı KULLANILMAZ — kaynağın
 * kullanım şartlarına ve robots kurallarına uy.
 */

import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { chromium, type Browser, type BrowserContext, type Page } from "playwright-core";
import { z } from "zod";

// ── Durum (sunucu ömrü boyunca bellekte) ─────────────────────────────────────
type Yakalanan = {
  no: number;
  zaman: number;
  tur: string; // resourceType: xhr | fetch | document | script …
  yontem: string;
  url: string;
  istek_baslik: Record<string, string>;
  istek_govde: string | null;
  durum: number | null;
  yanit_tur: string | null; // content-type
  yanit_baslik: Record<string, string>;
  yanit_govde: string | null; // kırpılmış
  yanit_boyut: number | null;
};

const GOVDE_LIMIT = 20_000; // saklanan gövde üst sınırı (kanıt için yeter, bağlamı taşırmaz)
let browser: Browser | null = null;
let ctx: BrowserContext | null = null;
let sayfa: Page | null = null;
let yakalananlar: Yakalanan[] = [];
let sayac = 0;
let dinliyor = false;

const kucukBaslik = (h: Record<string, string>) => {
  const o: Record<string, string> = {};
  for (const [k, v] of Object.entries(h)) o[k.toLowerCase()] = v;
  return o;
};

function dinlemeyeBasla(p: Page) {
  if (dinliyor) return;
  dinliyor = true;
  p.on("response", async (resp) => {
    try {
      const req = resp.request();
      const tur = req.resourceType();
      if (!["xhr", "fetch", "document"].includes(tur)) return; // varlıklar (img/css/font) hariç
      let govde: string | null = null;
      let boyut: number | null = null;
      try {
        const buf = await resp.body();
        boyut = buf.length;
        govde = buf.toString("utf8").slice(0, GOVDE_LIMIT);
      } catch {
        /* gövde okunamadı (yönlendirme/302, boş) — sorun değil */
      }
      sayac += 1;
      yakalananlar.push({
        no: sayac,
        zaman: Date.now(),
        tur,
        yontem: req.method(),
        url: req.url(),
        istek_baslik: kucukBaslik(await req.allHeaders()),
        istek_govde: req.postData() ?? null,
        durum: resp.status(),
        yanit_tur: resp.headers()["content-type"] ?? null,
        yanit_baslik: kucukBaslik(resp.headers()),
        yanit_govde: govde,
        yanit_boyut: boyut,
      });
    } catch {
      /* tek bir yanıt kaçarsa dinleme sürsün */
    }
  });
}

// Bağlam olmadan çağrılan araçlar için ortak uyarı
const CHROME_YOK =
  "Chrome'a bağlı değil. Önce `chrome_baslat` ile hata ayıklama portlu Chrome aç, sonra `chrome_baglan` çağır.";

const metin = (s: string) => ({ content: [{ type: "text" as const, text: s }] });

// ── Sunucu ───────────────────────────────────────────────────────────────────
const server = new McpServer({ name: "site-kesif", version: "0.1.0" });

server.registerTool(
  "chrome_baslat",
  {
    title: "Chrome'u hata ayıklama portuyla başlat",
    description:
      "Kalıcı bir profille (girişlerin saklanır — EVDS gibi kaynaklar için) ve --remote-debugging-port ile " +
      "Chrome başlatmak için gereken komutu döner. Komut kullanıcının terminalinde çalıştırılmalı; bu araç " +
      "yalnız komutu ve profil yolunu verir (tarayıcıyı doğrudan açmaz, güvenlik için tetiği kullanıcı çeker).",
    inputSchema: { port: z.number().int().min(1024).max(65535).default(9222).describe("CDP portu") },
    annotations: { readOnlyHint: true, openWorldHint: false },
  },
  async ({ port }) => {
    const profil = `${process.env.HOME}/.site-kesif-chrome`;
    const komut = `"/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" --remote-debugging-port=${port} --user-data-dir="${profil}"`;
    return metin(
      `Aşağıdaki komutu terminalde çalıştır (ana Chrome profilinden ayrı, kalıcı bir keşif profili açar):\n\n` +
        `${komut}\n\n` +
        `Açılan pencerede hedef kaynağa (varsa) giriş yap; giriş bilgileri "${profil}" altında saklanır. ` +
        `Sonra \`chrome_baglan\` (port ${port}) çağır.`
    );
  }
);

server.registerTool(
  "chrome_baglan",
  {
    title: "Çalışan Chrome'a CDP ile bağlan",
    description:
      "Hata ayıklama portunda çalışan Chrome'a bağlanır ve ağ dinlemesini başlatır. Bağlantı kurulunca " +
      "senin gezdiğin her sayfanın XHR/fetch trafiği yakalanmaya başlar.",
    inputSchema: { port: z.number().int().min(1024).max(65535).default(9222) },
    annotations: { readOnlyHint: false, openWorldHint: true },
  },
  async ({ port }) => {
    try {
      browser = await chromium.connectOverCDP(`http://localhost:${port}`);
      ctx = browser.contexts()[0] ?? (await browser.newContext());
      sayfa = ctx.pages()[0] ?? (await ctx.newPage());
      dinliyor = false;
      dinlemeyeBasla(sayfa);
      // Yeni sekmeler de dinlensin
      ctx.on("page", (p) => {
        dinliyor = false;
        sayfa = p;
        dinlemeyeBasla(p);
      });
      const url = sayfa.url();
      return metin(
        `Bağlandı (port ${port}). Aktif sekme: ${url || "boş"}. Ağ dinlemesi AÇIK. ` +
          `Şimdi \`hedef_ac\` ile bir kaynağa git ya da tarayıcıda elle gez; sonra \`yakalananlar\` ile endpoint'leri listele.`
      );
    } catch (e) {
      return metin(
        `Bağlanamadı: ${(e as Error).message}\n${CHROME_YOK}\n` +
          `(Chrome'un o portta gerçekten açık ve başka bir örnekçe kilitlenmemiş olduğundan emin ol.)`
      );
    }
  }
);

server.registerTool(
  "hedef_ac",
  {
    title: "Hedef adrese git ve trafiği yakala",
    description:
      "Aktif sekmeyi verilen adrese götürür, ağ boşalana kadar bekler. Sayfanın açılışta yaptığı tüm " +
      "veri çağrıları yakalanır. Kullanıcının işaret ettiği hedefe bu araçla gidilir.",
    inputSchema: {
      url: z.string().url().describe("Gidilecek adres (kaynağın veri sayfası)"),
      bekleme_ms: z.number().int().min(0).max(60000).default(3500).describe("Ağ boşalması için ek bekleme"),
    },
    annotations: { readOnlyHint: false, openWorldHint: true },
  },
  async ({ url, bekleme_ms }) => {
    if (!sayfa) return metin(CHROME_YOK);
    try {
      const oncesi = yakalananlar.length;
      await sayfa.goto(url, { waitUntil: "domcontentloaded", timeout: 45000 });
      await sayfa.waitForTimeout(bekleme_ms).catch(() => {});
      const yeni = yakalananlar.length - oncesi;
      return metin(`Gidildi: ${url}\nBu gezinmede ${yeni} yeni çağrı yakalandı. \`yakalananlar\` ile listele.`);
    } catch (e) {
      return metin(`Gidilemedi: ${(e as Error).message}`);
    }
  }
);

server.registerTool(
  "yakalananlar",
  {
    title: "Yakalanan çağrıları listele",
    description:
      "Yakalanan ağ çağrılarını özet listeler (no, yöntem, durum, içerik türü, URL). Aynı endpoint'in " +
      "tekrarları teklenir. Veri dönen (json/xml) çağrılar önce gelir. Detay için `cagri_detay` kullan.",
    inputSchema: {
      suzgec: z.string().optional().describe("URL/içerik içinde geçen metin (ör. 'kur', 'evds', '.json')"),
      yalniz_veri: z.boolean().default(true).describe("Yalnız json/xml/text yanıtları göster"),
      limit: z.number().int().min(1).max(200).default(50),
    },
    annotations: { readOnlyHint: true, openWorldHint: false },
  },
  async ({ suzgec, yalniz_veri, limit }) => {
    let liste = [...yakalananlar];
    if (yalniz_veri)
      liste = liste.filter((x) => /json|xml|text|csv/i.test(x.yanit_tur ?? "") || x.tur === "xhr" || x.tur === "fetch");
    if (suzgec) {
      const s = suzgec.toLowerCase();
      liste = liste.filter((x) => x.url.toLowerCase().includes(s) || (x.yanit_govde ?? "").toLowerCase().includes(s));
    }
    // teklendir: yöntem+path (query hariç) — son görüleni tut
    const gorulen = new Map<string, Yakalanan>();
    for (const x of liste) {
      const path = x.url.split("?")[0];
      gorulen.set(`${x.yontem} ${path}`, x);
    }
    const teklenmis = [...gorulen.values()].sort((a, b) => {
      const av = /json|xml/i.test(a.yanit_tur ?? "") ? 0 : 1;
      const bv = /json|xml/i.test(b.yanit_tur ?? "") ? 0 : 1;
      return av - bv || a.no - b.no;
    });
    const satirlar = teklenmis.slice(0, limit).map((x) => {
      const kb = x.yanit_boyut ? `${(x.yanit_boyut / 1024).toFixed(1)}KB` : "?";
      return `#${x.no} ${x.yontem} ${x.durum ?? "-"} [${(x.yanit_tur ?? "?").split(";")[0]}, ${kb}] ${x.url}`;
    });
    const govde = satirlar.length
      ? satirlar.join("\n")
      : "Henüz uygun çağrı yakalanmadı. `hedef_ac` ile bir sayfaya git veya tarayıcıda gez.";
    return {
      content: [{ type: "text" as const, text: `${teklenmis.length} benzersiz endpoint (ilk ${limit}):\n\n${govde}` }],
      structuredContent: {
        toplam: teklenmis.length,
        endpointler: teklenmis.slice(0, limit).map((x) => ({
          no: x.no,
          yontem: x.yontem,
          durum: x.durum,
          icerik_turu: x.yanit_tur,
          url: x.url,
        })),
      },
    };
  }
);

server.registerTool(
  "cagri_detay",
  {
    title: "Bir çağrının tam detayı",
    description:
      "Seçilen çağrının (no ile) tam istek/yanıt detayını verir: yöntem, URL, başlıklar, istek gövdesi, " +
      "yanıt gövdesi (kırpılmış) ve JSON ise şema özeti. Endpoint'i anlamak ve taklit için buradan bakılır.",
    inputSchema: { no: z.number().int().describe("`yakalananlar`daki çağrı numarası") },
    annotations: { readOnlyHint: true, openWorldHint: false },
  },
  async ({ no }) => {
    const x = yakalananlar.find((y) => y.no === no);
    if (!x) return metin(`#${no} bulunamadı. \`yakalananlar\` ile numaralara bak.`);
    let sema = "";
    if (/json/i.test(x.yanit_tur ?? "") && x.yanit_govde) {
      try {
        sema = "\n\nYanıt şeması (özet):\n" + semaOzet(JSON.parse(x.yanit_govde));
      } catch {
        /* kısmi/kırpık json */
      }
    }
    const g = [
      `#${x.no} — ${x.yontem} ${x.url}`,
      `Durum: ${x.durum} · İçerik: ${x.yanit_tur ?? "?"} · Boyut: ${x.yanit_boyut ?? "?"} bayt`,
      ``,
      `İstek başlıkları:\n${JSON.stringify(x.istek_baslik, null, 2)}`,
      x.istek_govde ? `\nİstek gövdesi:\n${x.istek_govde.slice(0, 2000)}` : `\n(istek gövdesi yok)`,
      ``,
      `Yanıt gövdesi (ilk ${GOVDE_LIMIT} bayt):\n${x.yanit_govde ?? "(okunamadı)"}`,
      sema,
    ].join("\n");
    return metin(g);
  }
);

server.registerTool(
  "cagri_tekrarla",
  {
    title: "Çağrıyı taklit et (call/callback)",
    description:
      "Yakalanan bir çağrıyı program dışından yeniden yapar — endpoint'in bağımsız çalışıp çalışmadığını, " +
      "parametre değiştirince ne döndüğünü test etmek için. İstenirse URL/başlık/gövde geçersiz kılınır. " +
      "Tarayıcıdaki oturum başlıkları (çerez dâhil) korunur ki girişli kaynaklar da yanıt versin.",
    inputSchema: {
      no: z.number().int().describe("Taklit edilecek yakalanmış çağrının numarası"),
      url: z.string().url().optional().describe("URL'i değiştir (parametre denemek için)"),
      govde: z.string().optional().describe("İstek gövdesini değiştir (POST için)"),
      baslik_ekle: z.record(z.string()).optional().describe("Ek/değişen başlıklar"),
    },
    annotations: { readOnlyHint: false, idempotentHint: false, openWorldHint: true },
  },
  async ({ no, url, govde, baslik_ekle }) => {
    const x = yakalananlar.find((y) => y.no === no);
    if (!x) return metin(`#${no} bulunamadı.`);
    const hedefUrl = url ?? x.url;
    // tarayıcı-yönetimli / güvensiz başlıkları ele
    const yasak = new Set(["host", "content-length", "connection", "accept-encoding", ":authority", ":method", ":path", ":scheme"]);
    const baslik: Record<string, string> = {};
    for (const [k, v] of Object.entries(x.istek_baslik)) if (!yasak.has(k) && !k.startsWith(":")) baslik[k] = v;
    Object.assign(baslik, baslik_ekle ?? {});
    const govdeSon = govde ?? x.istek_govde ?? undefined;
    try {
      const t0 = Date.now();
      const r = await fetch(hedefUrl, {
        method: x.yontem,
        headers: baslik,
        body: ["GET", "HEAD"].includes(x.yontem) ? undefined : govdeSon,
      });
      const metinYanit = await r.text();
      const sure = Date.now() - t0;
      return metin(
        `Taklit: ${x.yontem} ${hedefUrl}\nSonuç: ${r.status} ${r.statusText} · ${metinYanit.length} bayt · ${sure}ms · ${r.headers.get("content-type") ?? "?"}\n\n` +
          metinYanit.slice(0, GOVDE_LIMIT) +
          (metinYanit.length > GOVDE_LIMIT ? `\n… (${metinYanit.length - GOVDE_LIMIT} bayt daha)` : "")
      );
    } catch (e) {
      return metin(
        `Taklit başarısız: ${(e as Error).message}\n` +
          `İpucu: çağrı çerez/oturum ya da özel başlık (imza, token) gerektiriyor olabilir. cagri_detay ile başlıklara bak; ` +
          `token dinamikse endpoint tarayıcı içinden çağrılmalı (ingest adaptörü CDP üzerinden yapabilir).`
      );
    }
  }
);

server.registerTool(
  "profil_uret",
  {
    title: "Endpoint'ten veri kaynağı profili üret",
    description:
      "Seçilen çağrıdan data/veri-kaynaklari.json ile uyumlu bir kaynak profili taslağı üretir " +
      "(kod, erişim, veri alanları, kullanım). Keşif sonucunu ingest adaptörüne / dış veri iskeletine bağlar.",
    inputSchema: {
      no: z.number().int(),
      kod: z.string().describe("Kaynak kısa kodu (ör. 'evds-tufe', 'bist-tlref')"),
      eksen: z.enum(["vergi", "bagimsiz-denetim", "ortak"]).default("ortak"),
      kullanim: z.string().optional().describe("Hangi standart/motor kullanacak (ör. 'TMS 29 endeksleme')"),
    },
    annotations: { readOnlyHint: true, openWorldHint: false },
  },
  async ({ no, kod, eksen, kullanim }) => {
    const x = yakalananlar.find((y) => y.no === no);
    if (!x) return metin(`#${no} bulunamadı.`);
    let alanlar: string[] = [];
    if (/json/i.test(x.yanit_tur ?? "") && x.yanit_govde) {
      try {
        alanlar = ustAlanlar(JSON.parse(x.yanit_govde));
      } catch {
        /* */
      }
    }
    const profil = {
      kod,
      ad: kod,
      eksen,
      erisim: {
        yontem: /json/i.test(x.yanit_tur ?? "") ? "rest-api" : x.yanit_tur?.includes("xml") ? "xml" : "http",
        url: x.url.split("?")[0],
        ornek_tam_url: x.url,
        http_yontem: x.yontem,
        kimlik: x.istek_baslik["authorization"] || x.istek_baslik["cookie"] ? "gerekli (token/çerez)" : "yok",
        dogrulandi: false,
        canli_test: `#${no}: ${x.durum} ${x.yanit_tur ?? ""}`,
      },
      veri: alanlar.length ? alanlar : ["(şema çıkarılamadı — cagri_detay'a bak)"],
      guncelleme: "TESPİT EDİLECEK",
      kullanim: kullanim ? [kullanim] : [],
      not: "site-kesif ile üretildi; data/veri-kaynaklari.json'a eklemeden önce erişim/kimlik doğrula.",
    };
    return metin(
      `data/veri-kaynaklari.json 'kaynaklar' dizisine eklenecek taslak:\n\n${JSON.stringify(profil, null, 2)}`
    );
  }
);

server.registerTool(
  "temizle",
  {
    title: "Yakalama tamponunu temizle",
    description: "Yakalanan çağrı listesini sıfırlar (yeni bir kaynağa geçerken karışmasın diye).",
    inputSchema: {},
    annotations: { readOnlyHint: false, destructiveHint: true, idempotentHint: true, openWorldHint: false },
  },
  async () => {
    const n = yakalananlar.length;
    yakalananlar = [];
    sayac = 0;
    return metin(`${n} kayıt temizlendi.`);
  }
);

// ── JSON şema özeti yardımcıları ──────────────────────────────────────────────
function tip(v: unknown): string {
  if (Array.isArray(v)) return `dizi[${v.length}]`;
  if (v === null) return "null";
  return typeof v;
}
function ustAlanlar(v: unknown): string[] {
  const kok = Array.isArray(v) ? v[0] : v;
  if (kok && typeof kok === "object") return Object.keys(kok as object);
  return [];
}
function semaOzet(v: unknown, derinlik = 0): string {
  if (derinlik > 2) return "…";
  const kok = Array.isArray(v) ? v[0] : v;
  if (kok && typeof kok === "object") {
    return Object.entries(kok as Record<string, unknown>)
      .slice(0, 25)
      .map(([k, vv]) => `${"  ".repeat(derinlik)}- ${k}: ${tip(vv)}`)
      .join("\n");
  }
  return `${tip(v)}`;
}

// ── Başlat ────────────────────────────────────────────────────────────────────
async function main() {
  const transport = new StdioServerTransport();
  await server.connect(transport);
  process.stderr.write("site-kesif-mcp hazır (stdio).\n");
}
main().catch((e) => {
  process.stderr.write(`site-kesif-mcp hata: ${e}\n`);
  process.exit(1);
});
