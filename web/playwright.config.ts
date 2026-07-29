import { defineConfig, devices } from "@playwright/test";

// E2E yapılandırması — arayüz + defter BİRLİKTE test edilir.
// API'nin (cargo run -p api) ayakta olması gerekir; vite sunucusunu Playwright başlatır.
export default defineConfig({
  testDir: "./e2e",
  timeout: 45_000,
  expect: { timeout: 10_000 },
  // Testler ORTAK defteri değiştiriyor (eşleştirme, aktarım) — paralel koşarsa
  // birbirlerinin muhasebe durumunu bozar ve sonuç güvenilmez olur.
  fullyParallel: false,
  workers: 1,
  retries: 0,
  reporter: [["list"]],
  use: {
    baseURL: process.env.WEB_URL || "http://localhost:5174",
    trace: "retain-on-failure",
    screenshot: "only-on-failure",
    locale: "tr-TR",
    timezoneId: "Europe/Istanbul",
  },
  projects: [{ name: "chromium", use: { ...devices["Desktop Chrome"] } }],
  webServer: {
    command: "npm run dev -- --port 5174 --strictPort",
    url: "http://localhost:5174",
    reuseExistingServer: true,
    timeout: 60_000,
  },
});
