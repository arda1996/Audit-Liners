import { Fragment, useCallback, useEffect, useMemo, useState } from "react";
import { useLocation, useNavigate } from "react-router-dom";
import HesapSecici from "./HesapSecici";
import AciklamaPanel from "./AciklamaPanel";
import Sidebar from "./Sidebar";
import AciklamaSecici from "./AciklamaSecici";
import CommandPalette from "./CommandPalette";
import Rehber from "./Rehber";
import Login, { type Kullanici } from "./Login";
import Banka from "./Banka";
import Belgeler from "./Belgeler";
import Havada from "./Havada";
import BelgeAlim from "./BelgeAlim";

const API = "http://127.0.0.1:8787";

type Hesap = { kod: string; ad: string; tip: string; dogasi: string; yaprak: boolean; seviye: number };
type Satir = { hesap_kod: string; aciklama: string; borc: string; alacak: string };
type MizanSatir = { kebir: string; kod: string; ad: string; aciklama: string; borc: number; alacak: number; borc_bakiye: number; alacak_bakiye: number };
type FisOzet = { id: number; yevmiye_no: number | null; fis_no: string; tip: string; tarih: string; aciklama: string; tutar: number; dayanaksiz: boolean; durum: string };

// Fiş şablonları — en sık kayıtlar (ornek-kayitlar.md). Tutarları kullanıcı girer;
// KDV satırı "kdvOran" ile işaretli: matrah satırı girilince otomatik hesaplanır.
type Sablon = { ad: string; tip: string; sektorler: string[]; kullanici?: boolean; kullanim?: number; satirlar: { kod: string; aciklama: string; taraf: "B" | "A"; kdvOran?: number }[] };
type SatirDetay = { kod: string; ad: string; aciklama: string; borc: number; alacak: number };
type FisDetay = { id: number; fis_no: string; tip: string; tarih: string; aciklama: string; belge_tipi: string | null; belge_no: string | null; dayanaksiz: boolean; satirlar: SatirDetay[] };

const SERI: Record<string, string> = { Tahsil: "TAH", Tediye: "TED", Mahsup: "MAH", Acilis: "AÇ", Kapanis: "KAP" };
const kurus = (tl: string) => {
  let t = String(tl).trim().replace(/\s/g, "");
  if (t.includes(",")) t = t.replace(/\./g, "").replace(",", "."); // TR: nokta=binlik, virgül=ondalık
  return Math.round((parseFloat(t) || 0) * 100);
};
const tl = (k: number) => (k / 100).toLocaleString("tr-TR", { minimumFractionDigits: 2, maximumFractionDigits: 2 });
const tlk = (k: number) => "₺ " + tl(k);
const bosSatir = (): Satir => ({ hesap_kod: "", aciklama: "", borc: "", alacak: "" });

const atalar = (kod: string): string[] => {
  const ana = kod.split(".")[0];
  const res: string[] = [];
  for (let l = 1; l < ana.length; l++) res.push(ana.slice(0, l));
  const parts = kod.split(".");
  let acc = parts[0];
  for (let i = 1; i < parts.length - 1; i++) { acc += "." + parts[i]; res.push(acc); }
  return res;
};

// Modül haritası (docs/analiz/03-frontend-todo.md) — tasarım: Genel Bakış.dc.html
// Sidebar SADE: her grup tek satır, üzerine gelince uçan pencere açılır.
// Belgeler/Banka/Cari ayrı bir "Finans" başlığı değil — muhasebenin FONKSİYONLARI (kayda giden veri).
const NAV: { grup: string; kok?: string; ikon?: string; items: { id: string; ad: string; faz?: string }[] }[] = [
  { grup: "Genel Bakış", kok: "dashboard", items: [{ id: "dashboard", ad: "Genel Bakış" }] },
  { grup: "Muhasebe", kok: "muhasebe", items: [
    { id: "muhasebe", ad: "Kayıt & defterler" },
    { id: "belgeler", ad: "Belgeler (e-Fatura)" },
    { id: "belgealim", ad: "Belge yükle → kayıt" },
    { id: "banka", ad: "Banka" },
    { id: "havada", ad: "Havada bakiye" },
    { id: "hesaplar", ad: "Hesap planı" },
    { id: "cari", ad: "Cari kartlar", faz: "F2" },
  ]},
  // VUK/GİB katmanı — mükellefin beyan yükümlülüğü (vergi matrahı, beyanname, dönem sonu).
  { grup: "Vergi & Raporlama", kok: "vergi", ikon: "rapor", items: [
    { id: "vergi", ad: "Vergi (KDV · geçici)" },
    { id: "analiz", ad: "Finansal analiz" },
    { id: "muhtasar", ad: "Muhtasar", faz: "F2" },
    { id: "bordro", ad: "Bordro", faz: "F2" },
    { id: "degerleme", ad: "Dönem sonu", faz: "F2" },
  ]},
  // BDS/TFRS katmanı — AYRI sistem, ama vergi katmanıyla karşılıklı besleme var:
  // VUK mizanı denetime girdi olur (UFRS WorkSheet'te TFRS'ye köprülenir, deftere İŞLEMEZ);
  // denetim bulguları da düzeltme/beyan revizesi olarak vergi tarafına döner.
  { grup: "Bağımsız Denetim", kok: "denetim", ikon: "denetim", items: [
    { id: "denetim", ad: "Çalışma programları" },
    { id: "ufrs", ad: "UFRS WorkSheet" },
  ]},
  { grup: "Tanımlar", kok: "hesaplar", ikon: "tanimlar", items: [
    { id: "hesaplar", ad: "Hesap planı" }, { id: "firma", ad: "Firma / Mükellef", faz: "F1" }, { id: "parametre", ad: "Parametreler", faz: "F1" },
  ]},
];
const GERCEK = ["dashboard", "muhasebe", "hesaplar", "vergi", "banka", "belgeler", "analiz", "denetim", "ufrs", "yonetim"];
const BASLIK: Record<string, [string, string]> = {
  dashboard: ["Genel Bakış", "Dönem özeti ve kayıt durumu"],
  denetim: ["Bağımsız Denetim", "BDS katmanı — sektörel çalışma programları → çalışma kağıtları (BDS 230, TMS/TFRS dayanaklı). Vergi beyanından ayrı sistem."],
  yonetim: ["Yönetim", "Kullanıcılar ve mükellef erişim yetkileri (yalnız admin)"],
  vergi: ["Vergi & Raporlama", "VUK katmanı — KDV beyanname taslağı · geçici vergi hesap kağıdı · vergi takvimi"],
  muhasebe: ["Muhasebe", "Kayıt → fişler → yevmiye → kebir → muavin → mizan — tek ekran, kronolojik bütünlük"],
  fisler: ["Fişler", "Kesinleşen kayıtlar ve detayları"],
  mizan: ["Mizan", "Hesap bazında borç / alacak / bakiye"],
  hesaplar: ["Hesap planı", "Tek Düzen — alt kırılım ve resmi işleyiş"],
  kdv: ["KDV", "Aylık 191/391 mahsubu — 360 Ödenecek / 190 Devreden"],
  yevmiye: ["Yevmiye defteri", "Müteselsil maddeler — tarih sırasıyla (VUK)"],
  kebir: ["Defter-i kebir", "Hesap bazında hareketler + yürüyen bakiye"],
  muavin: ["Muavin defteri", "Alt hesap düzeyinde döküm — TXT dışa aktarılabilir"],
  bilanco: ["Bilanço", "Aktif = Pasif — dönem kârı öz kaynakta"],
  gelirt: ["Gelir tablosu", "Brüt satıştan dönem net kârına"],
  banka: ["Banka", "Ekstre yükle → kayıtlarla eşleştir (tutar + tarih önerili)"],
  havada: ["Havada bakiye", "Eşleşmemiş para hareketleri — faturaya bağla veya fatura bekleniyor işaretle"],
  belgealim: ["Belge yükle → muhasebe kaydı", "Fatura/mutabakat/dekont yükle · alanları doğrula · fişi kur (VUK 227: kayıt belgeye dayanır)"],
  belgeler: ["Belgeler", "e-Fatura / e-Arşiv — tek liste: tip · durum · tahsilat · iz-sür"],
  analiz: ["Finansal analiz", "Oranlar + banka görünümü · bilanço/GT (kebir) · dönem kıyası · kur bağlamı"],
  ufrs: ["UFRS WorkSheet", "VUK mizan → denetçi çalışmaları (AJE/RJE, dayanak + not) → WTB → TFRS bakiye — deftere işlemez"],
};
const aktifAd = (id: string) => NAV.flatMap((g) => g.items).find((i) => i.id === id)?.ad ?? id;

export default function App() {
  const [gorunum, setGorunum] = useState("dashboard");
  // ── URL routing köprüsü (frontend-routing-plani.md, Adım 2) — tek kaynak = URL ──
  // git(id): sidebar/komut/rehber → URL yazar; gorunum URL'den TÜRETİLİR (aşağıdaki effect).
  const navigate = useNavigate();
  const location = useLocation();
  const git = useCallback((g: string) => navigate("/" + (g === "dashboard" ? "panel" : g)), [navigate]);
  useEffect(() => {
    const seg = location.pathname.split("/")[1] || "panel";
    setGorunum(seg === "panel" ? "dashboard" : seg);
  }, [location.pathname]);
  const [muhAlt, setMuhAlt] = useState("yeni");
  const [hesaplar, setHesaplar] = useState<Hesap[]>([]);
  const [tip, setTip] = useState("Tediye");
  const [tarih, setTarih] = useState("01.03.2026");
  const [belgeTipi, setBelgeTipi] = useState("Fatura");
  const [belgeNo, setBelgeNo] = useState("");
  const [satirlar, setSatirlar] = useState<Satir[]>([bosSatir(), bosSatir()]);
  const [gonderiliyor, setGonderiliyor] = useState(false);
  const [suruklenen, setSuruklenen] = useState<number | null>(null); // sürüklenen satır indeksi
  // Satırı from→to taşır; toplam borç/alacak zaten satirlar'dan türediği için hesap CANLI güncellenir.
  const satirTasi = (from: number, to: number) => {
    if (from === to) return;
    setSatirlar((x) => { const a = [...x]; const [t] = a.splice(from, 1); a.splice(to, 0, t); return a; });
  };
  const [mizan, setMizan] = useState<MizanSatir[]>([]);
  const [fisler, setFisler] = useState<FisOzet[]>([]);
  const [seciliFis, setSeciliFis] = useState<FisDetay | null>(null);
  // Spotlight öğretici rehber — ilk açılışta otomatik, sonra başlıktaki düğmeyle aç/kapat
  const [rehberAcik, setRehberAcik] = useState(() => localStorage.getItem("rehber-gezildi") !== "1");
  const rehberKapat = () => { setRehberAcik(false); localStorage.setItem("rehber-gezildi", "1"); };
  // --- Kullanıcı girişi + yetki ---
  const [token, setToken] = useState<string | null>(() => localStorage.getItem("oturum-token"));
  const [kullanici, setKullanici] = useState<Kullanici | null>(null);
  const [oturumHazir, setOturumHazir] = useState(false);
  const authFetch = (u: string, opts: RequestInit = {}) =>
    fetch(u, { ...opts, headers: { ...(opts.headers || {}), ...(token ? { authorization: "Bearer " + token } : {}) } });
  useEffect(() => {
    if (!token) { setOturumHazir(true); return; }
    fetch(`${API}/api/oturum`, { headers: { authorization: "Bearer " + token } })
      .then((r) => (r.ok ? r.json() : Promise.reject()))
      .then((k) => setKullanici(k))
      .catch(() => { localStorage.removeItem("oturum-token"); setToken(null); })
      .finally(() => setOturumHazir(true));
  }, [token]);
  const girisYap = (tok: string, k: Kullanici) => { localStorage.setItem("oturum-token", tok); setToken(tok); setKullanici(k); yenileMukellef(tok); };
  const cikisYap = () => { authFetch(`${API}/api/cikis`, { method: "POST" }); localStorage.removeItem("oturum-token"); setToken(null); setKullanici(null); };
  // Admin: kullanıcı yönetimi
  const [kullanicilar, setKullanicilar] = useState<Kullanici[]>([]);
  const [ykAd, setYkAd] = useState(""); const [ykKa, setYkKa] = useState(""); const [ykSifre, setYkSifre] = useState("");
  const [ykRol, setYkRol] = useState("kullanici"); const [ykMuk, setYkMuk] = useState<string[]>([]);
  const [ykDep, setYkDep] = useState("MUHASEBE"); const [ykKademe, setYkKademe] = useState("ELEMAN");
  const [ykMesaj, setYkMesaj] = useState<{ ok: boolean; t: string } | null>(null);
  type Katalog = { kod: string; ad: string; aciklama?: string; moduller?: string[]; islemler?: string[]; onay_esigi_kurus?: number };
  const [departmanlar, setDepartmanlar] = useState<Katalog[]>([]);
  const [kademeler, setKademeler] = useState<Katalog[]>([]);
  useEffect(() => {
    if (!kullanici) return;
    fetch(`${API}/api/departmanlar`).then((r) => r.json()).then((d) => setDepartmanlar(d.departmanlar ?? [])).catch(() => {});
    fetch(`${API}/api/kademeler`).then((r) => r.json()).then((d) => setKademeler(d.kademeler ?? [])).catch(() => {});
  }, [kullanici]);
  // Oturumdaki kullanıcının görünür modülleri (departman) + izinli işlemleri (kademe). admin = tümü.
  const yoneticiMi = kullanici?.rol === "admin";
  const gorunurModuller = yoneticiMi ? ["*"] : (departmanlar.find((d) => d.kod === kullanici?.departman)?.moduller ?? ["*"]);
  const benimIslemler = yoneticiMi ? ["*"] : (kademeler.find((k) => k.kod === kullanici?.kademe)?.islemler ?? []);
  const yetkiVar = (islem: string) => benimIslemler.includes("*") || benimIslemler.includes(islem);
  const yenileKullanicilar = () => authFetch(`${API}/api/kullanicilar`).then((r) => (r.ok ? r.json() : [])).then(setKullanicilar).catch(() => {});
  useEffect(() => { if (gorunum === "yonetim" && kullanici?.rol === "admin") yenileKullanicilar(); }, [gorunum]);
  const kullaniciEkle = async () => {
    setYkMesaj(null);
    const r = await authFetch(`${API}/api/kullanici`, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ kullanici_adi: ykKa.trim(), ad: ykAd.trim(), sifre: ykSifre, rol: ykRol, mukellef_idleri: ykMuk, departman: ykDep, kademe: ykKademe }) });
    const d = await r.json();
    if (r.ok) { setYkMesaj({ ok: true, t: "Kullanıcı oluşturuldu: " + ykKa }); setYkAd(""); setYkKa(""); setYkSifre(""); setYkMuk([]); yenileKullanicilar(); }
    else setYkMesaj({ ok: false, t: d.hata ?? "hata" });
  };
  const [mesaj, setMesaj] = useState<{ ok: boolean; text: string } | null>(null);
  const [aciklamaKod, setAciklamaKod] = useState<string | null>(null);
  const [plan, setPlan] = useState<Hesap[]>([]);
  const [acik, setAcik] = useState<Set<string>>(new Set());
  const [hpFiltre, setHpFiltre] = useState("");
  const [muavinAna, setMuavinAna] = useState<string | null>(null);
  const [muavinAlt, setMuavinAlt] = useState("");
  const [muavinAd, setMuavinAd] = useState("");
  /// Alt hesap fiş satırından istendiyse: ekleme sonrası o satıra yaz.
  const [muavinHedef, setMuavinHedef] = useState<number | null>(null);
  const [kdv, setKdv] = useState<{ indirilecek: number; devreden: number; hesaplanan: number; fark: number } | null>(null);
  const [palet, setPalet] = useState(false);
  useEffect(() => {
    const h = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") { e.preventDefault(); setPalet((p) => !p); }
    };
    window.addEventListener("keydown", h);
    return () => window.removeEventListener("keydown", h);
  }, []);
  const yenileKdv = () => fetch(`${API}/api/kdv`).then((r) => r.json()).then(setKdv).catch(() => {});
  const kdvMahsup = async () => {
    const r = await fetch(`${API}/api/kdv-mahsup`, { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify({ tarih: tarih }) });
    const d = await r.json();
    alert(r.ok ? `Mahsup fişi kesinleşti: ${d.fis_no}` : d.hata ?? "hata");
    yenileKdv(); yenileMizan(); yenileFisler();
  };

  const yenileHesaplar = () => fetch(`${API}/api/hesaplar`).then((r) => r.json()).then(setHesaplar).catch(() => {});
  const yenilePlan = () => fetch(`${API}/api/hesap-plani`).then((r) => r.json()).then(setPlan).catch(() => {});
  const yenileMizan = () => fetch(`${API}/api/mizan`).then((r) => r.json()).then(setMizan).catch(() => {});
  const yenileFisler = () => fetch(`${API}/api/fisler`).then((r) => r.json()).then(setFisler).catch(() => {});
  const acFis = (id: number) => fetch(`${API}/api/fis/${id}`).then((r) => r.json()).then(setSeciliFis).catch(() => {});

  useEffect(() => { yenileHesaplar(); yenilePlan(); yenileMizan(); yenileFisler(); }, []);
  // Çoklu mükellef (görev #5) — aktif mükellef + seçici
  type Mukellef = { id: string; unvan: string; vkn: string; sektor_kodlari: string[]; maliyet_secenegi: string };
  type Sektor = { kod: string; ad: string };
  const [mukellefler, setMukellefler] = useState<Mukellef[]>([]);
  const [aktifMuk, setAktifMuk] = useState("m1");
  const [mukAcik, setMukAcik] = useState(false);
  const [sektorList, setSektorList] = useState<Sektor[]>([]);
  const [yeniUnvan, setYeniUnvan] = useState("");
  const [yeniVkn, setYeniVkn] = useState("");
  const [yeniSektor, setYeniSektor] = useState("TIC");
  const yenileMukellef = (tok?: string) => fetch(`${API}/api/mukellefler`, { headers: { authorization: "Bearer " + (tok ?? token ?? "") } }).then((r) => r.json()).then((d) => { setMukellefler(d.mukellefler); setAktifMuk(d.aktif); }).catch(() => {});
  const [sablonlar, setSablonlar] = useState<Sablon[]>([]);
  const yenileSablonlar = () => fetch(`${API}/api/sablonlar`).then((r) => r.json()).then((d) => setSablonlar(d.sablonlar ?? [])).catch(() => {});
  useEffect(() => { yenileSablonlar(); /* eslint-disable-next-line */ }, []);
  const [sablonYonet, setSablonYonet] = useState(false);
  type SablonTaslakSatir = { kod: string; ad: string; aciklama: string; taraf: "B" | "A" };
  const [sablonTaslak, setSablonTaslak] = useState<{ ad: string; duzenlenen?: string; satirlar: SablonTaslakSatir[] } | null>(null);
  const [sablonHata, setSablonHata] = useState<string | null>(null);
  // Şablon kimliği = hesap+taraf kümesi (sıra önemsiz). Aynı imzalı şablon zaten varsa
  // muhasebeciye "bu zaten var" denir — mükerrer şablon birikmesini önler.
  const sablonImza = (s: { kod: string; taraf: string }[]) =>
    s.map((r) => `${r.kod}:${r.taraf}`).sort().join("|");
  const mevcutSablon = sablonTaslak
    ? sablonlar.find((s) => s.ad !== sablonTaslak.duzenlenen // düzenlenen şablon kendini "mükerrer" saymasın
        && sablonImza(s.satirlar.map((r) => ({ kod: r.kod, taraf: r.taraf }))) === sablonImza(sablonTaslak.satirlar))
    : undefined;
  // Aktif mükellefin sektörüne göre şablonları süz — üreticiye 150, tüccara 153 önerilir.
  const aktifSektorler = mukellefler.find((m) => m.id === aktifMuk)?.sektor_kodlari ?? [];
  const gorunenSablonlar = sablonlar.filter((sb) => sb.sektorler.includes("*") || sb.sektorler.some((k) => aktifSektorler.includes(k)));
  // Sık kullanılanlar öne — muhasebecinin ilk birkaç şablonu işinin çoğunu görür.
  const sikKullanilan = [...gorunenSablonlar].filter((s) => (s.kullanim ?? 0) > 0)
    .sort((a, b) => (b.kullanim ?? 0) - (a.kullanim ?? 0)).slice(0, 5);
  useEffect(() => { if (kullanici) yenileMukellef(); fetch(`${API}/api/sektorler`).then((r) => r.json()).then((d) => setSektorList(d.sektorler ?? [])).catch(() => {}); }, [kullanici]);
  // Aktif mükellef değişince TÜM aktif-defter verisi yeniden çekilir (çalışma seti swap oldu).
  const hepsiniTazele = () => { yenileFisler(); yenileMizan(); yenileHesaplar(); yenilePlan(); yenileKdv(); };
  const mukellefSec = async (id: string) => {
    if (id === aktifMuk) { setMukAcik(false); return; }
    await authFetch(`${API}/api/mukellef/${id}/aktif`, { method: "POST" });
    setAktifMuk(id); setMukAcik(false); hepsiniTazele();
  };
  const mukellefEkle = async () => {
    if (!yeniUnvan.trim()) return;
    const r = await authFetch(`${API}/api/mukellef`, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ unvan: yeniUnvan.trim(), vkn: yeniVkn.trim(), sektor_kodlari: [yeniSektor], maliyet_secenegi: "" }) });
    const d = await r.json();
    setYeniUnvan(""); setYeniVkn(""); await yenileMukellef();
    if (d.id) await mukellefSec(d.id);
  };
  // --- Vergi sayfası (G ekseni): KDV1 taslağı + geçici vergi kağıdı ---
  const [vergiAlt, setVergiAlt] = useState("kdv");
  type KdvManuel = { ad: string; yon: string; matrah: number; kdv: number; kaynak: string };
  type KdvBey = { ay: number; donem: string; matrahlar: { kod: string; ad: string; oran: number; matrah: number; hesaplanan: number; kaynak: string }[]; manuel: KdvManuel[]; hesaplanan_toplam: number; onceki_devreden: number; bu_donem_indirilecek: number; indirimler_toplam: number; odenecek: number; devreden: number; kaynak_notlari: string[]; indirim_dagilimi: { kod: string; ad: string; oran: number; matrah: number; hesaplanan: number }[]; istisna: { matrah: number; belge_sayisi: number; kaynak: string }; tevkifat: { matrah: number; toplam_kdv: number; tevkif_edilen: number; beyan_edilen: number; belge_sayisi: number; kaynak: string } };
  type GvM = { ad: string; tutar: number };
  type Gv = { ceyrek: number; donem: string; ticari_kar: number; kkeg_otomatik: number; kkeg_manuel: GvM[]; istisna_manuel: GvM[]; matrah: number; oran: number; hesaplanan: number; onceki_mahsup: number; odenecek: number };
  const [kdvBeyAy, setKdvBeyAy] = useState(12);
  const [kdvBey, setKdvBey] = useState<KdvBey | null>(null);
  type BabsSatir = { vkn: string; unvan: string; belge_sayisi: number; tutar: number; hadde_girdi: boolean };
  type Babs = { ay: number; donem: string; had: number; yururlukte: boolean; kaldirilma_tarihi: string; dayanak: string;
    ba: { aciklama: string; bildirilecek_cari: number; toplam: number; satirlar: BabsSatir[] };
    bs: { aciklama: string; bildirilecek_cari: number; toplam: number; satirlar: BabsSatir[] };
    vkn_eksik: string[]; notlar: string[] };
  const [babs, setBabs] = useState<Babs | null>(null);
  const [gvCeyrek, setGvCeyrek] = useState(12);
  const [gv, setGv] = useState<Gv | null>(null);
  const [gvKkeg, setGvKkeg] = useState<GvM[]>([]);
  const [gvIst, setGvIst] = useState<GvM[]>([]);
  const [gvYeniAd, setGvYeniAd] = useState("");
  const [gvYeniTutar, setGvYeniTutar] = useState("");
  const [gvYeniTip, setGvYeniTip] = useState("kkeg");
  const gvYukle = (c: number) =>
    fetch(`${API}/api/gecici-vergi?ceyrek=${c}`).then((r) => r.json()).then((d) => { setGv(d); setGvKkeg(d.kkeg_manuel); setGvIst(d.istisna_manuel); }).catch(() => {});
  useEffect(() => {
    if (gorunum !== "vergi") return;
    yenileKdv();
    fetch(`${API}/api/kdv-beyanname?ay=${kdvBeyAy}`).then((r) => r.json()).then((d) => { setKdvBey(d); setKdvManuel(d.manuel ?? []); }).catch(() => {});
    fetch(`${API}/api/beyanname/babs?ay=${kdvBeyAy}`).then((r) => r.json()).then(setBabs).catch(() => {});
    gvYukle(gvCeyrek);
  }, [gorunum, kdvBeyAy, gvCeyrek, fisler.length]);
  const [kdvManuel, setKdvManuel] = useState<KdvManuel[]>([]);
  const [kmAd, setKmAd] = useState(""); const [kmYon, setKmYon] = useState("indirim");
  const [kmMatrah, setKmMatrah] = useState(""); const [kmKdv, setKmKdv] = useState(""); const [kmKaynak, setKmKaynak] = useState("");
  const kdvManuelKaydet = (satirlar: KdvManuel[]) =>
    fetch(`${API}/api/kdv-beyanname/duzenle?ay=${kdvBeyAy}`, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ satirlar }) })
      .then(() => fetch(`${API}/api/kdv-beyanname?ay=${kdvBeyAy}`).then((r) => r.json()).then((d) => { setKdvBey(d); setKdvManuel(d.manuel ?? []); }));
  const kdvManuelEkle = () => {
    const m = Math.round((parseFloat(kmMatrah) || 0) * 100), k = Math.round((parseFloat(kmKdv) || 0) * 100);
    if (!kmAd.trim() || k <= 0) return;
    kdvManuelKaydet([...kdvManuel, { ad: kmAd.trim(), yon: kmYon, matrah: m, kdv: k, kaynak: kmKaynak.trim() }]);
    setKmAd(""); setKmMatrah(""); setKmKdv(""); setKmKaynak("");
  };
  const gvKaydet = () =>
    fetch(`${API}/api/gecici-vergi/duzenle?ceyrek=${gvCeyrek}`, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ kkeg: gvKkeg, istisna: gvIst }) }).then(() => gvYukle(gvCeyrek));
  type VParam = { kod: string; ad: string; alanlar: { ad: string; deger: string; dogrulandi: boolean; programda?: string }[] };
  const [vparam, setVparam] = useState<VParam[]>([]);
  useEffect(() => {
    if (gorunum === "vergi" && vergiAlt === "param" && vparam.length === 0)
      fetch(`${API}/api/vergi-parametreler`).then((r) => r.json()).then((d) => setVparam(d.kanunlar ?? [])).catch(() => {});
  }, [gorunum, vergiAlt]);
  const gvEkle = () => {
    const t = Math.round((parseFloat(gvYeniTutar) || 0) * 100);
    if (!gvYeniAd.trim() || t <= 0) return;
    if (gvYeniTip === "kkeg") setGvKkeg([...gvKkeg, { ad: gvYeniAd.trim(), tutar: t }]);
    else setGvIst([...gvIst, { ad: gvYeniAd.trim(), tutar: t }]);
    setGvYeniAd(""); setGvYeniTutar("");
  };

  // Defterler & mali tablolar
  type Madde = { yevmiye_no: number | null; fis_no: string; tarih: string; tip: string; aciklama: string; belge: string | null; toplam: number; satirlar: SatirDetay[] };
  const [maddeler, setMaddeler] = useState<Madde[]>([]);
  const [kebirKod, setKebirKod] = useState("");
  const [kebirData, setKebirData] = useState<{ kod: string; ad: string; hareketler: { tarih: string; yevmiye_no: number | null; fis_no: string; aciklama: string; karsi: string; borc: number; alacak: number; yuruyen_bakiye: number }[] } | null>(null);
  const [mizanSeviye, setMizanSeviye] = useState<"muavin" | "kebir">("muavin");
  const [mizanFiltre, setMizanFiltre] = useState("");
  // Banka → izole Banka.tsx · Belgeler → izole Belgeler.tsx (state kendi dosyalarında)

  // Finansal analiz
  type Oran = { grup: string; ad: string; deger: number; birim: string; durum: string; yorum: string; banka: string };
  type Dikey = { ad: string; tutar: number; oran: number };
  type Aylik = { ay: number; net_satis: number; net_kar: number; cari_oran: number; aktif: number };
  type KebirB = { kod: string; ad: string; taraf: string; bakiye: number };
  type KurAy = { ay: number; usd: number; eur: number; usd_eur: number };
  type GtD = Record<string, number>;
  type AnalizD = { ay: number; oranlar: Oran[]; kebir_bilanco: KebirB[]; gt: GtD; dikey_bilanco: Dikey[]; dikey_gt: Dikey[]; aylik: Aylik[]; kur: KurAy[]; kiyas: { ay: number; oranlar: Oran[]; kebir_bilanco: KebirB[]; gt: GtD; yorumlar: string[] } | null };
  const [analiz, setAnaliz] = useState<AnalizD | null>(null);
  const [analizAy, setAnalizAy] = useState(12);
  const [analizKiyas, setAnalizKiyas] = useState<number | "">(9); // tx-1 varsayılan (Q4 → Q3)
  const [analizAlt, setAnalizAlt] = useState("oranlar");
  useEffect(() => {
    if (gorunum === "analiz") fetch(`${API}/api/analiz?ay=${analizAy}${analizKiyas ? `&kiyas=${analizKiyas}` : ""}`).then((r) => r.json()).then(setAnaliz).catch(() => {});
  }, [gorunum, fisler.length, analizAy, analizKiyas]);
  const fmt = (v: number, birim: string) => birim === "x" ? v.toFixed(2) : birim === "gün" ? Math.round(v) + " gün" : v.toFixed(1) + "%";
  const durumPil = (d: string) => d === "iyi" ? "pos" : d === "orta" ? "warn" : "neg";
  const AYAD = ["", "Oca", "Şub", "Mar", "Nis", "May", "Haz", "Tem", "Ağu", "Eyl", "Eki", "Kas", "Ara"];
  // Raporlama dönemleri (dönemlik mali tablo sunum esası): Q1=31.03 Q2=30.06 Q3=30.09 Q4=31.12
  const QSON: Record<number, string> = { 3: "31.03.2026", 6: "30.06.2026", 9: "30.09.2026", 12: "31.12.2026" };
  const qlab = (ay: number) => QSON[ay] ? `Q${ay / 3} · ${QSON[ay]}` : AYAD[ay];
  // tx her zaman tx-1 (bir önceki dönem) ile karşılaştırılır; Q1'in öncesi bu deftered yok.
  const oncekiCeyrek = (ay: number) => (ay > 3 ? ay - 3 : null);
  const donemSec = (ay: number) => { setAnalizAy(ay); const o = oncekiCeyrek(ay); setAnalizKiyas(o ?? ""); };
  // --- Denetim: sektörel programlar + çalışma kağıdı (H.4) ---
  type ProgCalisma = { id: string; ad: string; bds: string; motor: string; siddet: string; hesaplar: string[]; anlatim: string };
  type ProgSektor = { kod: string; ad: string; devralir?: string; calismalar: ProgCalisma[] };
  type ManuelSatir = { kod: string; ad: string; borc: number; alacak: number; not: string };
  type KagitT = { id: string; ad: string; sektor: string; standart: string; siddet: string; amac: string; ref_no: string; donem: string; tarih: string; hazirlayan: string; onekler: string[]; nitelikler: { onek: string; ad: string; tanim: string }[]; hesaplar: { kod: string; ad: string; borc: number; alacak: number; borc_bakiye: number; alacak_bakiye: number; hareket: number }[]; testler: { motor: string; ad: string; durum: string; bulgular: string[] }[]; not_metni: string; duzenlemeler: { hucreler?: Record<string, number | string>; satirlar?: ManuelSatir[] } };
  const [denetimSektorler, setDenetimSektorler] = useState<ProgSektor[]>([]);
  const [denetimSektor, setDenetimSektor] = useState("TIC");
  const [kagit, setKagit] = useState<KagitT | null>(null);
  const [kagitNot, setKagitNot] = useState("");
  // Excel-benzeri kağıt grid'i: kilit (güvenli varsayılan) + hücre müdahaleleri + manuel satırlar.
  // İlke: müdahale DEFTERE dokunmaz; sistem değeri korunur, yanında saklanır (denetim izi).
  const [kagitKilit, setKagitKilit] = useState(true);
  const [kagitHucre, setKagitHucre] = useState<Record<string, number | string>>({});
  const [kagitManuel, setKagitManuel] = useState<ManuelSatir[]>([]);
  // --- UFRS WorkSheet (görev #26): WTB + denetçi çalışmaları + kayıt defteri ---
  // Model: top-side entries — deftere İŞLEMEZ; kayıt = dayanak (ekspertiz/piyasa/hesaplama) + denetçi notu + iz.
  type UfrsSatirT = { hesap: string; borc: number; alacak: number };
  type UfrsKayitT = { no: string; tur: string; standart: string; kaynak_ws: string; aciklama: string; satirlar: UfrsSatirT[]; dayanak_tur: string; dayanak_ref: string; denetci_notu: string; degerleme_yontemi: string; degerleme_bazi: string; durum: string; hazirlayan: string; tarih: string; donem: string; devir: string; devir_kaynak?: string | null };
  type UfrsParamT = { anahtar: string; ad: string; tip: string; varsayilan: string; aciklama: string };
  type UfrsHesapAdayT = { kod: string; ad: string; tur?: string };
  type UfrsCalismaT = { id: string; standart: string; tur: string; ad: string; anlatim: string; dayanak_onerisi: string; hesaplar: string[]; sira: number; cati?: boolean; uyuyan?: boolean; kayit_sayisi?: number; uygun?: boolean; devir?: string; sektorler?: string[]; parametreler?: UfrsParamT[]; hesapla?: boolean; kisa_ad?: string; senaryolar?: { kod: string; ad: string; aciklama: string; tur: string; ev_kanal?: string; ev_yon?: string; bacaklar?: { hesap: string; yon: string; pay?: string; kanal?: string }[]; hedef_onekleri?: string[]; tutar_kaynak?: string; kurallar?: string[]; cift_tutar?: boolean; ikinci_tutar_ad?: string }[]; kayit_hesaplari?: { borc: UfrsHesapAdayT[]; alacak: UfrsHesapAdayT[] }; degerleme_yontemleri?: string[] };
  type DegerlemeYontemT = { kod: string; ad: string; aciklama: string };
  type UfrsHesapT = { ara_tablo: { kalem: string; detay: string; tutar: number; kaynak?: string; kaynak_ref?: string; mudahale?: boolean }[]; uyarilar: string[]; onerilen: null | { tur: string; satirlar: { hesap: string; borc: number; alacak: number; serbest: boolean }[]; aciklama: string; dayanak_tur: string; not_taslagi: string; degerleme_yontemi: string; degerleme_bazi_taslagi: string } };
  type DayanakTurT = { kod: string; ad: string; aciklama: string };
  // Big-4 değer zinciri: VUK (Per Books) → AJE → Düzeltilmiş → RJE → CF (devir) → TFRS (Final)
  type WtbSatirT = { kod: string; ad: string; tdhp: boolean; sinif: string; vuk: number; aje: number; duzeltilmis: number; rje: number; cf: number; tfrs: number; aje_refs: string[]; rje_refs: string[]; cf_refs: string[] };
  type WtbT = { donem: string; kesin: boolean; satirlar: WtbSatirT[]; kontrol: { vuk_sifir: boolean; aje_sifir: boolean; rje_sifir: boolean; cf_sifir: boolean; tfrs_sifir: boolean; vuk: number; aje: number; rje: number; cf: number; tfrs: number }; kar_koprusu: { vuk_kar: number; aje_etkisi: number; tfrs_kar: number; not: string } };
  // Denetim işaret geleneği: tek net sütun, borç +, alacak (parantezli) — Big-4/CaseWare standardı
  const wtl = (v: number) => (v === 0 ? "—" : v < 0 ? `(${tl(-v)})` : tl(v));
  const SINIF_AD: Record<string, string> = { "1": "DÖNEN VARLIKLAR", "2": "DURAN VARLIKLAR", "3": "KISA VADELİ YABANCI KAYNAKLAR", "4": "UZUN VADELİ YABANCI KAYNAKLAR", "5": "ÖZ KAYNAKLAR", "6": "GELİR TABLOSU", "7": "MALİYET HESAPLARI", "8": "SERBEST", "9": "NAZIM", T: "TFRS KALEMLERİ (TDHP DIŞI)" };
  const [ufrsSekme, setUfrsSekme] = useState("wtb");
  const [wtb, setWtb] = useState<WtbT | null>(null);
  const [ufrsKatalog, setUfrsKatalog] = useState<{ dayanak_turleri: DayanakTurT[]; degerleme_yontemleri: DegerlemeYontemT[]; calismalar: UfrsCalismaT[]; ev_hesaplari?: Record<string, string> } | null>(null);
  const [ufrsWs, setUfrsWs] = useState("");
  const [ufrsDetay, setUfrsDetay] = useState<{ tanim: UfrsCalismaT; girdi_hesaplar: { kod: string; ad: string; kebir: string; borc_bakiye: number; alacak_bakiye: number; net: number; hareket: number; kimlik?: string | null; vuk_omur?: number | null; tfrs_omur?: number | null; tamamlayici_of?: string | null }[]; kayitlar: UfrsKayitT[] } | null>(null);
  const [ufrsKayitlar, setUfrsKayitlar] = useState<UfrsKayitT[]>([]);
  const [ukTur, setUkTur] = useState("AJE");
  const [ukSatirlar, setUkSatirlar] = useState<{ hesap: string; borc: string; alacak: string }[]>([{ hesap: "", borc: "", alacak: "" }, { hesap: "", borc: "", alacak: "" }]);
  // Hedef hesap: "bu kayıt HANGİ hesap için?" — girdi tablosundan tıkla ya da serbest seç (üst ya da alt kırılım).
  const [ufrsHedef, setUfrsHedef] = useState<{ kod: string; ad: string } | null>(null);
  const [senaryoKod, setSenaryoKod] = useState("");
  const [senaryoTutar, setSenaryoTutar] = useState("");
  const [senaryoOran, setSenaryoOran] = useState("25");
  const hedefSec = (kod: string, ad: string) => {
    // D3 düzeltmesi: hedef değişince ÖNCEKİ hedefin yazdığı satırı güncelle (bayat bacak bırakma);
    // ilk seçimde boş satıra yaz; form doluysa ezme — yeni satır aç.
    const oncekiKod = ufrsHedef?.kod;
    setUfrsHedef({ kod, ad });
    setUkSatirlar((prev) => {
      const onceki = oncekiKod ? prev.findIndex((x) => x.hesap.trim() === oncekiKod) : -1;
      const bos = prev.findIndex((x) => !x.hesap.trim());
      const i = onceki >= 0 ? onceki : bos;
      if (i < 0) return [...prev, { hesap: kod, borc: "", alacak: "" }];
      return prev.map((x, j) => (j === i ? { ...x, hesap: kod } : x));
    });
  };
  const [ukAciklama, setUkAciklama] = useState("");
  const [ukDayanakTur, setUkDayanakTur] = useState("ekspertiz");
  const [ukDayanakRef, setUkDayanakRef] = useState("");
  const [ukNot, setUkNot] = useState("");
  const [ukYontem, setUkYontem] = useState("");
  const [ukBaz, setUkBaz] = useState("");
  const [ukMesaj, setUkMesaj] = useState("");
  // Açılır pencereler: kayıt listesi (delinebilir tutar) veya yeni kayıt formu
  const [ufrsModal, setUfrsModal] = useState<null | { tip: "kayitlar"; baslik: string; nolar: string[] } | { tip: "yeni" }>(null);
  const [ufrsParams, setUfrsParams] = useState<Record<string, string>>({});
  const [ufrsHesap, setUfrsHesap] = useState<UfrsHesapT | null>(null);
  const [ufrsHesapMesaj, setUfrsHesapMesaj] = useState("");
  const [wtbAcik, setWtbAcik] = useState<Record<string, { kod: string; ad: string; bakiye: number }[] | "yok">>({});
  const yenileUfrs = () => {
    if (plan.length === 0) yenilePlan(); // hedef/kayıt seçicisi için tam ağaç (üst hesaplar dahil)
    fetch(`${API}/api/ufrs/calismalar`).then((r) => r.json()).then(setUfrsKatalog).catch(() => {});
    fetch(`${API}/api/ufrs/wtb`).then((r) => r.json()).then(setWtb).catch(() => {});
    fetch(`${API}/api/ufrs/kayitlar`).then((r) => r.json()).then(setUfrsKayitlar).catch(() => {});
  };
  useEffect(() => { if (gorunum === "ufrs") yenileUfrs(); }, [gorunum]);
  const acUfrsWs = (id: string, tur?: string) => {
    // D2 düzeltmesi: çalışma değişince TÜM form state'i sıfırlanır — önceki çalışmanın
    // satırları/notları yeni kaynak_ws altında gönderilemesin.
    setUfrsWs(id); setUkMesaj(""); setUfrsHesap(null); setUfrsHesapMesaj(""); setUfrsHedef(null); setSenaryoKod(""); setSenaryoTutar(""); setSenaryoTutar2(""); setSenaryoOran("25");
    setUkSatirlar([{ hesap: "", borc: "", alacak: "" }, { hesap: "", borc: "", alacak: "" }]);
    setUkAciklama(""); setUkDayanakRef(""); setUkNot("");
    if (tur) setUkTur(tur);
    fetch(`${API}/api/ufrs/calisma/${id}`).then((r) => r.json()).then((d) => {
      setUfrsDetay(d);
      if (d?.tanim?.dayanak_onerisi) setUkDayanakTur(d.tanim.dayanak_onerisi);
      const ilk: Record<string, string> = {};
      (d?.tanim?.parametreler ?? []).forEach((pr: UfrsParamT) => { ilk[pr.anahtar] = pr.varsayilan; });
      setUfrsParams(ilk);
      setUkYontem(d?.tanim?.degerleme_yontemleri?.[0] ?? "");
      setUkBaz("");
    }).catch(() => {});
  };
  // D4 düzeltmesi: TEK sezgi (backend ile aynı) — ilk karakter rakamsa TDHP kodu, değilse serbest
  // TFRS kalemi. Çok seviyeli muavin (252.90.1) artık yanlışlıkla serbest sayılmaz.
  const serbestMi = (h: string) => !/^\d/.test(h.trim());
  const tlGir = (k: number) => (k / 100).toFixed(2).replace(".", ","); // kuruş → form TL metni
  const ufrsHesaplaCalistir = async () => {
    setUfrsHesapMesaj("… hesaplanıyor"); setUfrsHesap(null);
    try {
      const r = await fetch(`${API}/api/ufrs/calisma/${ufrsWs}/hesapla`, {
        method: "POST", headers: { "content-type": "application/json" },
        body: JSON.stringify({ parametreler: ufrsParams }),
      });
      const d = await r.json();
      if (r.ok) { setUfrsHesap(d); setUfrsHesapMesaj(""); }
      else setUfrsHesapMesaj("⚠ " + (d.hata ?? "hesaplanamadı"));
    } catch { setUfrsHesapMesaj("⚠ sunucuya ulaşılamadı"); }
  };
  // v7: hedef yalnız @hedef bacaklı senaryoda gerekir; karma senaryolar ikinci tutar (tutar2) ister
  const [senaryoTutar2, setSenaryoTutar2] = useState("");
  const seciliSenaryo = ufrsDetay?.tanim.senaryolar?.find((s) => s.kod === senaryoKod);
  const senaryoHedefIster = !!seciliSenaryo?.bacaklar?.some((b) => b.hesap === "@hedef");
  // KAPALI DÜNYA: standart hangi hesapları ilgilendiriyorsa seçici YALNIZ onları gösterir.
  // Kaynak: çalışmanın hesap önekleri + kayit_hesaplari + senaryo bacakları + EV hesapları.
  // Denetçi başka hesaba yönlendirilmez — sınıflandırma gereği başka hesaba kayıt gereksinimi yoktur.
  const wsHesaplar = useMemo(() => {
    const tum = plan.length ? plan : hesaplar;
    const t = ufrsDetay?.tanim;
    if (!t) return tum;
    const onekler = (t.hesaplar ?? []).map(String);
    const sabitKebir = new Set<string>();
    const ekle = (kod?: string) => { if (kod && /^\d/.test(kod)) sabitKebir.add(kod.split(".")[0]); };
    (t.kayit_hesaplari?.borc ?? []).forEach((h) => ekle(h.kod));
    (t.kayit_hesaplari?.alacak ?? []).forEach((h) => ekle(h.kod));
    (t.senaryolar ?? []).forEach((s) => (s.bacaklar ?? []).forEach((b) => { if (b.hesap !== "@hedef") ekle(b.hesap); }));
    Object.values(ufrsKatalog?.ev_hesaplari ?? {}).forEach((k) => ekle(k));
    return tum.filter((h) => h.kod.length >= 3 &&
      (onekler.some((o) => h.kod.startsWith(o)) || sabitKebir.has(h.kod.split(".")[0])));
  }, [plan, hesaplar, ufrsDetay, ufrsKatalog]);
  const senaryoUygula = async () => {
    if (!senaryoKod) { setUfrsHesapMesaj("⚠ önce bir senaryo seçin"); return; }
    if (senaryoHedefIster && !ufrsHedef) { setUfrsHesapMesaj("⚠ önce hedef hesabı seçin (girdi tablosundan tıkla)"); return; }
    setUfrsHesapMesaj("… senaryo uygulanıyor"); setUfrsHesap(null);
    try {
      const r = await fetch(`${API}/api/ufrs/calisma/${ufrsWs}/senaryo`, {
        method: "POST", headers: { "content-type": "application/json" },
        body: JSON.stringify({ senaryo_kod: senaryoKod, hedef: senaryoHedefIster ? ufrsHedef!.kod : "", tutar: senaryoTutar, tutar2: senaryoTutar2, kv_orani: senaryoOran ? "%" + senaryoOran : "" }),
      });
      const d = await r.json();
      if (r.ok) { setUfrsHesap(d); setUfrsHesapMesaj(""); }
      else setUfrsHesapMesaj("⚠ " + (d.hata ?? "olmadı"));
    } catch { setUfrsHesapMesaj("⚠ sunucuya ulaşılamadı"); }
  };
  const oneriFormaAktar = () => {
    const o = ufrsHesap?.onerilen;
    if (!o) return;
    setUkTur(o.tur);
    setUkSatirlar(o.satirlar.map((x) => ({ hesap: x.hesap, borc: x.borc ? tlGir(x.borc) : "", alacak: x.alacak ? tlGir(x.alacak) : "" })));
    setUkAciklama(o.aciklama);
    setUkDayanakTur(o.dayanak_tur);
    setUkNot(o.not_taslagi);
    setUkYontem(o.degerleme_yontemi);
    setUkBaz(o.degerleme_bazi_taslagi);
    setUkMesaj("Öneri forma aktarıldı — dayanak referansını tamamlayıp kaydedin.");
  };
  const wtbKirilim = async (kod: string) => {
    if (wtbAcik[kod]) { const y = { ...wtbAcik }; delete y[kod]; setWtbAcik(y); return; }
    try {
      const r = await fetch(`${API}/api/muavin?kod=${kod}`);
      const d: { kod: string; ad: string; bakiye: number }[] = await r.json();
      const alt = d.filter((m) => m.kod !== kod && m.bakiye !== 0);
      setWtbAcik({ ...wtbAcik, [kod]: alt.length ? alt : "yok" });
    } catch { /* sessiz */ }
  };
  const ukTL = (s: string) => { const v = parseFloat(s.replace(/\./g, "").replace(",", ".")); return isNaN(v) ? 0 : Math.round(v * 100); };
  const ufrsKayitGonder = async () => {
    const ws = ufrsKatalog?.calismalar.find((c) => c.id === ufrsWs);
    if (!ws) { setUkMesaj("⚠ önce bir çalışma seçin"); return; }
    const satirlar = ukSatirlar.filter((s) => s.hesap.trim()).map((s) => ({ hesap: s.hesap.trim(), borc: ukTL(s.borc), alacak: ukTL(s.alacak), serbest: serbestMi(s.hesap) }));
    try {
      const r = await authFetch(`${API}/api/ufrs/kayit`, {
        method: "POST", headers: { "content-type": "application/json" },
        body: JSON.stringify({ kaynak_ws: ufrsWs, tur: ukTur, standart: ws.standart, aciklama: ukAciklama, satirlar, dayanak_tur: ukDayanakTur, dayanak_ref: ukDayanakRef, denetci_notu: ukNot, degerleme_yontemi: ukYontem, degerleme_bazi: ukBaz }),
      });
      const d = await r.json();
      if (r.ok) {
        // D5 düzeltmesi: önce reset (acUfrsWs ukMesaj'ı boşaltır), SONRA başarı mesajı — mesaj yutulmasın.
        acUfrsWs(ufrsWs); yenileUfrs();
        const uy = (d.uyarilar as string[] | undefined)?.length ? " · " + (d.uyarilar as string[]).join(" ") : "";
        setUkMesaj(`✓ ${d.no} kaydedildi — WTB güncellendi${uy}`);
      } else setUkMesaj("⚠ " + (d.hata ?? "kayıt başarısız"));
    } catch { setUkMesaj("⚠ sunucuya ulaşılamadı"); }
  };
  const ufrsVazgec = async (no: string) => {
    // D6 düzeltmesi: yanıt kontrol edilir — kesin dönem/CF reddi sessiz kalmasın.
    try {
      const r = await authFetch(`${API}/api/ufrs/kayit/${no}/vazgec`, { method: "POST" });
      if (!r.ok) {
        const d = await r.json().catch(() => ({} as { hata?: string }));
        setUfrsAksiyonMesaj("⚠ " + ((d as { hata?: string }).hata ?? `${no} vazgeçilemedi`));
        return;
      }
    } catch { setUfrsAksiyonMesaj("⚠ sunucuya ulaşılamadı"); return; }
    yenileUfrs(); if (ufrsWs) acUfrsWs(ufrsWs);
  };
  const [ufrsAksiyonMesaj, setUfrsAksiyonMesaj] = useState("");
  const ufrsKesinlestir = async () => {
    if (!confirm(`${wtb?.donem} dönemi KESİNLEŞTİRİLSİN mi? Kesin döneme kayıt atılamaz; devir üretiminin kaynağı olur.`)) return;
    const r = await authFetch(`${API}/api/ufrs/kesinlestir`, { method: "POST" });
    const d = await r.json();
    setUfrsAksiyonMesaj(r.ok ? `✓ ${d.donem} kesinleşti (${d.kesinlesen_kayit} kayıt)` : "⚠ " + (d.hata ?? "olmadı"));
    yenileUfrs();
  };
  const ufrsDevirGetir = async () => {
    const r = await authFetch(`${API}/api/ufrs/devir`, { method: "POST" });
    const d = await r.json();
    setUfrsAksiyonMesaj(r.ok
      ? `✓ Devir ${d.kaynak}→${d.hedef}: ${d.uretilen_cf.length} CF üretildi, ${d.atlanan.length} atlandı${d.yeniden_olcum_gerekli.length ? " · yeniden ölçüm: " + d.yeniden_olcum_gerekli.join(", ") : ""}`
      : "⚠ " + (d.hata ?? "olmadı"));
    yenileUfrs();
  };
  type KSatir = { kod: string; ad: string; borc: number; alacak: number; hareket: number | null; manuel: boolean; mi: number; ovB: boolean; ovA: boolean; not: string };
  const kagitSatirlar = (): KSatir[] => {
    if (!kagit) return [];
    const ov = (k: string, a: string) => kagitHucre[k + ":" + a];
    return [
      ...kagit.hesaplar.map((h) => ({
        kod: h.kod, ad: h.ad,
        borc: ov(h.kod, "borc") !== undefined ? Number(ov(h.kod, "borc")) : h.borc,
        alacak: ov(h.kod, "alacak") !== undefined ? Number(ov(h.kod, "alacak")) : h.alacak,
        hareket: h.hareket, manuel: false, mi: -1,
        ovB: ov(h.kod, "borc") !== undefined, ovA: ov(h.kod, "alacak") !== undefined,
        not: String(ov(h.kod, "not") ?? ""),
      })),
      ...kagitManuel.map((m, i) => ({ kod: m.kod, ad: m.ad, borc: m.borc, alacak: m.alacak, hareket: null, manuel: true, mi: i, ovB: false, ovA: false, not: m.not })),
    ];
  };
  const sistemDeger = (kod: string, alan: "borc" | "alacak") => kagit?.hesaplar.find((h) => h.kod === kod)?.[alan] ?? 0;
  const hucreGeriAl = (kod: string, alan: string) => { const c = { ...kagitHucre }; delete c[kod + ":" + alan]; setKagitHucre(c); };
  const manuelGuncelle = (i: number, alan: string, v: string | number) => setKagitManuel(kagitManuel.map((m, j) => (j === i ? { ...m, [alan]: v } : m)));
  const paraGir = (r: KSatir, alan: "borc" | "alacak", sval: string) => {
    const kurus = Math.round((parseFloat(sval || "0") || 0) * 100);
    if (r.manuel) manuelGuncelle(r.mi, alan, kurus);
    else if (kurus === sistemDeger(r.kod, alan)) hucreGeriAl(r.kod, alan);
    else setKagitHucre({ ...kagitHucre, [r.kod + ":" + alan]: kurus });
  };
  const kagitDuzenleKaydet = () => {
    if (!kagit) return;
    fetch(`${API}/api/denetim/kagit/${kagit.id}/duzenle`, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ hucreler: kagitHucre, satirlar: kagitManuel }) })
      .then(() => setKagitKilit(true));
  };
  // Hesap satırından kayıtlara iniş: bakiyeyi oluşturan hareketler → fiş + dayanak → VUK 217 düzeltme.
  const [kagitDetayKod, setKagitDetayKod] = useState<string | null>(null);
  const [kagitHar, setKagitHar] = useState<Hareket[]>([]);
  const [kagitHarToplam, setKagitHarToplam] = useState(0);
  const [kagitHarOffset, setKagitHarOffset] = useState(0);
  const [kagitFis, setKagitFis] = useState<FisDetay | null>(null);
  const HARLIMIT = 15;
  const kagitHarYukle = (kod: string, offset: number) =>
    fetch(`${API}/api/muavin/hareket/${kod}?offset=${offset}&limit=${HARLIMIT}`).then((r) => r.json())
      .then((d) => { setKagitHar(d.hareketler); setKagitHarToplam(d.toplam); setKagitHarOffset(offset); }).catch(() => {});
  const kodTikla = (kod: string) => {
    setKagitFis(null);
    if (kagitDetayKod === kod) { setKagitDetayKod(null); return; }
    setKagitDetayKod(kod); setKagitHar([]); kagitHarYukle(kod, 0);
  };
  const kagitHareketAc = (h: Hareket) => {
    if (h.fis_id == null) return;
    if (kagitFis && kagitFis.id === h.fis_id) { setKagitFis(null); return; }
    fetch(`${API}/api/fis/${h.fis_id}`).then((r) => r.json()).then(setKagitFis).catch(() => {});
  };
  const kagitFisIptal = async (id: number) => {
    const gerekce = prompt("İptal gerekçesi (VUK md.217 — düzeltme kaydına yazılır):");
    if (!gerekce || !gerekce.trim()) return;
    const t = prompt("Hatanın fark edildiği tarih (gg.aa.yyyy):", "31.12.2026");
    if (!t) return;
    const r = await authFetch(`${API}/api/fis/${id}/iptal`, { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify({ tarih: t.trim(), gerekce: gerekce.trim() }) });
    const d = await r.json();
    alert(r.ok ? `Düzeltme (iptal) fişi kesinleşti: ${d.fis_no}` : d.hata ?? "hata");
    if (r.ok && kagit) { setKagitFis(null); kagitAc(kagit.id); if (kagitDetayKod) kagitHarYukle(kagitDetayKod, 0); }
  };
  useEffect(() => {
    if (gorunum === "denetim" && denetimSektorler.length === 0)
      fetch(`${API}/api/denetim/programlar`).then((r) => r.json()).then((d) => setDenetimSektorler(d.sektorler ?? [])).catch(() => {});
  }, [gorunum]);
  const denetimCalismalar = (kod: string): ProgCalisma[] => {
    const sk = denetimSektorler.find((x) => x.kod === kod);
    if (!sk) return [];
    return [...(sk.devralir ? denetimCalismalar(sk.devralir) : []), ...sk.calismalar];
  };
  const kagitAc = (id: string) =>
    fetch(`${API}/api/denetim/kagit/${id}`).then((r) => r.json()).then((k) => {
      setKagit(k); setKagitNot(k.not_metni ?? "");
      setKagitHucre(k.duzenlemeler?.hucreler ?? {}); setKagitManuel(k.duzenlemeler?.satirlar ?? []);
      setKagitKilit(true); setKagitDetayKod(null); setKagitFis(null);
    }).catch(() => {});
  const kagitNotKaydet = () => {
    if (!kagit) return;
    fetch(`${API}/api/denetim/kagit/${kagit.id}/not`, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ not_metni: kagitNot }) })
      .then(() => setKagit({ ...kagit, not_metni: kagitNot }));
  };
  const kagitTxt = () => {
    if (!kagit) return;
    const para = (v: number) => (v / 100).toLocaleString("tr-TR", { minimumFractionDigits: 2 });
    const L: string[] = [
      `ÇALIŞMA KAĞIDI ${kagit.ref_no}`,
      `Çalışma    : ${kagit.ad}`,
      `Sektör     : ${kagit.sektor}`,
      `Dayanak    : ${kagit.standart}`,
      `Dönem      : ${kagit.donem}`,
      `Hazırlayan : ${kagit.hazirlayan} · ${kagit.tarih}`,
      `Amaç       : ${kagit.amac}`,
      "",
      "İLGİLİ HESAPLAR (defterden önek taramasıyla tespit)",
    ];
    const satirlar = kagitSatirlar();
    satirlar.forEach((r) => {
      const isaret = r.manuel ? " (M)" : r.ovB || r.ovA ? " (D)" : "";
      L.push(`${r.kod.padEnd(10)} ${r.ad.slice(0, 30).padEnd(30)} Borç: ${para(r.borc).padStart(16)}  Alacak: ${para(r.alacak).padStart(16)}  Bakiye: ${para(r.borc - r.alacak).padStart(16)}${r.not ? "  Not: " + r.not : ""}${isaret}`);
    });
    L.push(`${"TOPLAM".padEnd(41)} Borç: ${para(satirlar.reduce((t, r) => t + r.borc, 0)).padStart(16)}  Alacak: ${para(satirlar.reduce((t, r) => t + r.alacak, 0)).padStart(16)}`);
    L.push("(M) manuel satır · (D) sistem değeri üzerinde manuel düzeltme — defter kayıtları değişmez");
    L.push("", "TESTLER");
    kagit.testler.forEach((t) => {
      L.push(`[${t.motor}] ${t.ad} — ${t.durum}`);
      t.bulgular.forEach((b) => L.push(`  • ${b}`));
      if (t.durum === "çalıştı" && t.bulgular.length === 0) L.push("  • Bulgu yok.");
    });
    L.push("", "SONUÇ / NOT", kagitNot || "-");
    const blob = new Blob(["\uFEFF" + L.join("\n")], { type: "text/plain;charset=utf-8" });
    const a = document.createElement("a");
    a.href = URL.createObjectURL(blob);
    a.download = `${kagit.ref_no.replace(/\//g, "-")}.txt`;
    a.click();
  };
  type Hareket = { tarih: string; yevmiye_no: number | null; fis_no: string; aciklama: string; karsi: string; borc: number; alacak: number; yuruyen_bakiye: number; fis_id?: number | null; belge?: string };
  type MuavinOzet = { kod: string; ad: string; hareket_sayisi: number; toplam_borc: number; toplam_alacak: number; bakiye: number };
  const [muavinOzet, setMuavinOzet] = useState<MuavinOzet[]>([]);
  const [muavinFiltre, setMuavinFiltre] = useState("");
  // Tembel yükleme: yalnız açılan hesabın hareketleri, sayfalı (offset = sondan geriye).
  const [acikHesap, setAcikHesap] = useState<Record<string, { toplam: number; hareketler: Hareket[] }>>({});
  const hareketYukle = (kod: string, offset: number) => {
    fetch(`${API}/api/muavin/hareket/${kod}?offset=${offset}&limit=200`)
      .then((r) => r.json())
      .then((d: { toplam: number; hareketler: Hareket[] }) => {
        setAcikHesap((s) => ({
          ...s,
          [kod]: { toplam: d.toplam, hareketler: [...d.hareketler, ...(s[kod]?.hareketler ?? [])] },
        }));
      }).catch(() => {});
  };
  const hesapToggle = (kod: string) => {
    setAcikHesap((s) => {
      if (s[kod]) { const n = { ...s }; delete n[kod]; return n; }
      return s;
    });
    if (!acikHesap[kod]) hareketYukle(kod, 0);
  };
  const [bilancoD, setBilancoD] = useState<{ donen: number; duran: number; aktif: number; kvyk: number; uvyk: number; oz: number; kar: number; pasif: number } | null>(null);
  const [gt, setGt] = useState<Record<string, number> | null>(null);
  useEffect(() => {
    if (gorunum === "muhasebe" && muhAlt === "yevmiye") fetch(`${API}/api/yevmiye`).then((r) => r.json()).then(setMaddeler).catch(() => {});
    if (gorunum === "muhasebe" && muhAlt === "muavin") { setAcikHesap({}); fetch(`${API}/api/muavin${muavinFiltre ? `?kod=${muavinFiltre}` : ""}`).then((r) => r.json()).then(setMuavinOzet).catch(() => {}); }
    if (gorunum === "bilanco") fetch(`${API}/api/bilanco`).then((r) => r.json()).then(setBilancoD).catch(() => {});
    if (gorunum === "gelirt") fetch(`${API}/api/gelir-tablosu`).then((r) => r.json()).then(setGt).catch(() => {});
  }, [gorunum, muhAlt, fisler.length, muavinFiltre]);
  useEffect(() => {
    if (gorunum === "muhasebe" && muhAlt === "kebir" && kebirKod) fetch(`${API}/api/kebir/${kebirKod}`).then((r) => r.json()).then(setKebirData).catch(() => {});
  }, [gorunum, muhAlt, kebirKod, fisler.length]);

  const ataToggle = (kod: string) => setAcik((s) => { const n = new Set(s); n.has(kod) ? n.delete(kod) : n.add(kod); return n; });

  const muavinEkle = async (anaKod: string) => {
    if (!muavinAlt.trim() || !muavinAd.trim()) return;
    const r = await fetch(`${API}/api/hesap/muavin`, { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify({ ana_kod: anaKod, alt_kod: muavinAlt.trim(), ad: muavinAd.trim() }) });
    if (r.ok) {
      const yeniKod = `${anaKod}.${muavinAlt.trim()}`;
      if (muavinHedef !== null) { guncelle(muavinHedef, "hesap_kod", yeniKod); setMuavinHedef(null); }
      setMuavinAna(null); setMuavinAlt(""); setMuavinAd(""); setAcik((s) => new Set(s).add(anaKod)); yenilePlan(); yenileHesaplar();
    } else { const d = await r.json(); alert(d.hata ?? "hata"); }
  };

  const gorunenPlan = useMemo(() => {
    const q = hpFiltre.trim().toLocaleLowerCase("tr");
    if (q) return plan.filter((h) => h.kod.includes(q) || h.ad.toLocaleLowerCase("tr").includes(q));
    return plan.filter((h) => atalar(h.kod).every((a) => acik.has(a)));
  }, [plan, acik, hpFiltre]);

  // Mizan: seviyeye göre satırlar (kebir = 3 haneli özet — muavinler ana koda toplanır)
  const mizanGoster = useMemo<MizanSatir[]>(() => {
    // Kebir filtresi: kod/kebir önekine göre (ör. "153" → 153 + tüm muavinleri)
    const f = mizanFiltre.trim();
    const kaynak = f ? mizan.filter((m) => m.kod.startsWith(f) || m.kebir.startsWith(f)) : mizan;
    if (mizanSeviye === "muavin") return kaynak;
    const g = new Map<string, MizanSatir>();
    for (const m of kaynak) {
      const e = g.get(m.kebir);
      if (e) { e.borc += m.borc; e.alacak += m.alacak; e.borc_bakiye += m.borc_bakiye; e.alacak_bakiye += m.alacak_bakiye; }
      else {
        const anaAd = plan.find((h) => h.kod === m.kebir)?.ad ?? m.ad;
        g.set(m.kebir, { ...m, kod: m.kebir, ad: anaAd });
      }
    }
    return [...g.values()];
  }, [mizan, mizanSeviye, mizanFiltre, plan]);

  const toplamBorc = useMemo(() => satirlar.reduce((t, s) => t + kurus(s.borc), 0), [satirlar]);
  const toplamAlacak = useMemo(() => satirlar.reduce((t, s) => t + kurus(s.alacak), 0), [satirlar]);
  const dengeli = toplamBorc > 0 && toplamBorc === toplamAlacak && satirlar.length >= 2;

  const guncelle = (i: number, alan: keyof Satir, deger: string) => {
    setSatirlar((s) => s.map((r, j) => (j === i ? { ...r, [alan]: deger } : r)));
    if (alan === "hesap_kod" && deger) setAciklamaKod(deger);
  };

  const sablonUygula = (ad: string) => {
    const s = sablonlar.find((x) => x.ad === ad);
    if (!s) return;
    if (satirlar.some((r) => r.hesap_kod || r.borc || r.alacak) && !confirm("Mevcut satırlar şablonla değiştirilsin mi?")) return;
    const tipDegisti = s.tip !== tip; // şablon fiş tipini sessizce değiştirmesin — muhasebeciye söyle
    setTip(s.tip);
    setSatirlar(s.satirlar.map((r) => ({ hesap_kod: r.kod, aciklama: r.aciklama, borc: "", alacak: "" })));
    setMesaj({
      ok: true,
      text: `“${s.ad}” uygulandı — tutarları sen gireceksin.` +
        (tipDegisti ? ` Fiş tipi ${s.tip} olarak değiştirildi (seri ${SERI[s.tip]}).` : ""),
    });
    // Kullanım sayacı — sık kullanılan şablon listenin başına gelsin.
    fetch(`${API}/api/sablon/${encodeURIComponent(ad)}/kullan`, { method: "POST" })
      .then(() => yenileSablonlar()).catch(() => {});
  };

  // Şablon kaydetme penceresini açar. BAKİYE ŞART DEĞİL: tutar girilmemiş satırlar da şablona
  // girer — taraf (B/A) tutardan çıkarılamazsa hesabın doğasından öneriler, kullanıcı çevirebilir.
  const sablonKayitAc = () => {
    const dolu = satirlar.filter((r) => r.hesap_kod);
    if (dolu.length < 2) { setMesaj({ ok: false, text: "Şablon için en az 2 hesap satırı gerekli (tutar girmene gerek yok)." }); return; }
    setSablonTaslak({
      ad: "",
      satirlar: dolu.map((r) => {
        const h = hesaplar.find((x) => x.kod === r.hesap_kod);
        const taraf: "B" | "A" = kurus(r.borc) > 0 ? "B" : kurus(r.alacak) > 0 ? "A" : h?.dogasi === "Alacak" ? "A" : "B";
        return { kod: r.hesap_kod, ad: h?.ad ?? "", aciklama: r.aciklama, taraf };
      }),
    });
    setSablonHata(null);
  };

  const sablonKaydet = async () => {
    if (!sablonTaslak) return;
    const ad = sablonTaslak.ad.trim();
    if (!ad) { setSablonHata("Şablon adı zorunlu."); return; }
    const govde = {
      ad, tip, sektorler: ["*"],
      satirlar: sablonTaslak.satirlar.map((r) => ({ kod: r.kod, aciklama: r.aciklama, taraf: r.taraf })),
    };
    const r = await fetch(`${API}/api/sablon`, {
      method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify(govde),
    });
    let d = await r.json();
    if (!r.ok) { setSablonHata(d.hata ?? "Şablon kaydedilemedi."); return; }
    // Düzenlemede ad değiştiyse eski kayıt kalmasın (POST yeni ad açar, eskisini silmez).
    const eski = sablonTaslak.duzenlenen;
    if (eski && eski !== ad) {
      const rs = await fetch(`${API}/api/sablon/${encodeURIComponent(eski)}`, { method: "DELETE" });
      if (rs.ok) d = await rs.json();
    }
    setSablonlar(d.sablonlar ?? []);
    setSablonTaslak(null);
    setMesaj({
      ok: true,
      text: eski
        ? `“${ad}” şablonu güncellendi.`
        : `“${ad}” şablonu kaydedildi — tutarlar saklanmadı, her kayıtta sen girersin.`,
    });
  };

  // ⌘⇧S / Ctrl+Shift+S — fiş formundayken şablon kaydetme penceresini açar.
  useEffect(() => {
    if (gorunum !== "muhasebe" || muhAlt !== "yeni") return;
    const f = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.shiftKey && e.key.toLowerCase() === "s") {
        e.preventDefault();
        if (!sablonTaslak) sablonKayitAc();
      }
    };
    window.addEventListener("keydown", f);
    return () => window.removeEventListener("keydown", f);
    // eslint-disable-next-line
  }, [gorunum, muhAlt, satirlar, hesaplar, sablonTaslak]);

  // Kendi şablonunu düzenle — aynı pencere, ad ve satırlar dolu gelir.
  const sablonDuzenle = (s: Sablon) => {
    setSablonTaslak({
      ad: s.ad, duzenlenen: s.ad,
      satirlar: s.satirlar.map((r) => ({
        kod: r.kod, ad: hesaplar.find((x) => x.kod === r.kod)?.ad ?? "", aciklama: r.aciklama, taraf: r.taraf,
      })),
    });
    setSablonHata(null);
  };

  const sablonSil = async (ad: string) => {
    if (!confirm(`“${ad}” şablonu silinsin mi?`)) return;
    const r = await fetch(`${API}/api/sablon/${encodeURIComponent(ad)}`, { method: "DELETE" });
    const d = await r.json();
    if (!r.ok) { setMesaj({ ok: false, text: d.hata ?? "Silinemedi." }); return; }
    setSablonlar(d.sablonlar ?? []);
  };

  // Kalan denge farkını verilen satıra yazar (muhasebeci kısayolu).
  const farkiDoldur = (i: number) => {
    const fark = toplamBorc - toplamAlacak;
    if (fark === 0) return;
    const s = satirlar[i];
    if (fark > 0) guncelle(i, "alacak", String((kurus(s.alacak) + fark) / 100));
    else guncelle(i, "borc", String((kurus(s.borc) - fark) / 100));
  };

  // Ters bakiye uyarısı: kayıt sonrası hesap, doğasına TERS bakiyeye düşüyorsa (MSUGT — ör. kasa alacak veremez).
  const tersBakiyeUyari = useMemo(() => {
    const uyarilar: string[] = [];
    for (const s of satirlar) {
      if (!s.hesap_kod) continue;
      const h = hesaplar.find((x) => x.kod === s.hesap_kod);
      if (!h) continue;
      const m = mizan.find((x) => x.kod === s.hesap_kod);
      const mevcut = (m?.borc_bakiye ?? 0) - (m?.alacak_bakiye ?? 0);
      const yeni = mevcut + kurus(s.borc) - kurus(s.alacak);
      if (h.dogasi === "Borç" && yeni < 0) uyarilar.push(`${s.hesap_kod} ${h.ad}: alacak bakiyesine düşer (${tl(yeni)}) — doğası BORÇ`);
      if (h.dogasi === "Alacak" && yeni > 0) uyarilar.push(`${s.hesap_kod} ${h.ad}: borç bakiyesine düşer (${tl(yeni)}) — doğası ALACAK`);
    }
    return uyarilar;
  }, [satirlar, hesaplar, mizan]);

  // VUK 217: iptal fişi hatanın FARK EDİLDİĞİ tarihle atılır; gerekçe zorunludur.
  const fisIptal = async (id: number) => {
    const gerekce = prompt("İptal gerekçesi (zorunlu — düzeltme kaydı açıklamasına yazılır):");
    if (!gerekce || !gerekce.trim()) return;
    const bugun = new Date().toISOString().slice(0, 10);
    const r = await authFetch(`${API}/api/fis/${id}/iptal`, {
      method: "POST", headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ tarih: bugun, gerekce: gerekce.trim() }),
    });
    const d = await r.json();
    alert(r.ok ? `İptal fişi kesinleşti: ${d.fis_no}` : d.hata ?? "hata");
    setSeciliFis(null); yenileFisler(); yenileMizan();
  };

  // Belge no'nun sonundaki sayıyı bir artırır, hane genişliğini korur (FT-2026-0001 → FT-2026-0002).
  const sonrakiBelgeNo = (no: string) => no.replace(/(\d+)(?!.*\d)/, (m) => String(Number(m) + 1).padStart(m.length, "0"));

  const kesinlestir = async () => {
    if (gonderiliyor || !dengeli) return;   // çift tıklama = mükerrer kesin fiş engeli
    setGonderiliyor(true); setMesaj(null);
    const govde = { tip, tarih, belge_tipi: belgeTipi, belge_no: belgeNo, satirlar: satirlar.map((s) => ({ hesap_kod: s.hesap_kod, borc: kurus(s.borc), alacak: kurus(s.alacak), aciklama: s.aciklama })) };
    try {
      const r = await authFetch(`${API}/api/fis`, { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify(govde) });
      const d = await r.json();
      if (r.ok) {
        setMesaj({ ok: true, text: `✓ Fiş kesinleşti: ${d.fis_no} — yeni kayda hazır.` });
        setSatirlar([bosSatir(), bosSatir()]);          // form temizlenir (tarih/tip korunur — seri kayıt hızı)
        setBelgeNo((n) => (n ? sonrakiBelgeNo(n) : n));  // belge no otomatik artar → aynı belgeyle ikinci kayıt önlenir
        yenileMizan(); yenileFisler();
        setTimeout(() => document.querySelector<HTMLButtonElement>(".combo-btn")?.focus(), 30);
      } else setMesaj({ ok: false, text: d.hata ?? "hata" });
    } catch {
      setMesaj({ ok: false, text: "Sunucuya ulaşılamadı — API çalışıyor mu?" });
    } finally { setGonderiliyor(false); }
  };

  // İlk kullanıcı için: boş deftere 20.000 kayıtlık örnek e-ticaret defterini yükler.
  const ornekYukle = async () => {
    if (gonderiliyor) return;
    setGonderiliyor(true);
    try { const r = await fetch(`${API}/api/ornek-veri`, { method: "POST" }); if (r.ok) { yenileFisler(); yenileMizan(); } }
    catch { /* sessiz */ } finally { setGonderiliyor(false); }
  };

  // Dashboard verileri (gerçek API'den)
  const dash = useMemo(() => {
    const tb = mizan.reduce((t, m) => t + m.borc, 0);
    const ta = mizan.reduce((t, m) => t + m.alacak, 0);
    const bak = (k: string) => { const m = mizan.find((x) => x.kod === k); return m ? m.borc_bakiye - m.alacak_bakiye : 0; };
    const kdvNet = bak("191") + bak("190") + bak("391"); // borç(+) yönlü net
    return { tb, ta, kdvNet, fisSayisi: fisler.length, dengeli: tb === ta };
  }, [mizan, fisler]);

  const [b1, b2] = BASLIK[gorunum] ?? [aktifAd(gorunum), "Standart analizinden türetilen modül"];

  if (!oturumHazir) return <div className="giris-ekran" />;
  if (!kullanici) return <Login onGiris={girisYap} />;
  const navTum = kullanici.rol === "admin" ? [...NAV, { grup: "Sistem", items: [{ id: "yonetim", ad: "Yönetim" }] }] : NAV;
  // Departmana göre süz: kullanıcı yalnız departmanının modüllerini görür (görünür değilse aktif sayfayı dashboard'a çek).
  const navGosterilen = gorunurModuller.includes("*") ? navTum
    : navTum.map((g) => ({ ...g, items: g.items.filter((it) => gorunurModuller.includes(it.id)) })).filter((g) => g.items.length > 0);
  return (
    <div className="app">
      <Sidebar nav={navGosterilen} aktif={gorunum} sec={git} fisSayisi={fisler.length} paletAc={() => setPalet(true)} />
      <CommandPalette acik={palet} kapat={() => setPalet(false)} nav={navGosterilen} git={git}
        hesaplar={plan} hesapSec={(k) => { git("hesaplar"); setAciklamaKod(k); setHpFiltre(k); }} />

      <main className="main">
        <header className="hdr">
          <div><h1>{b1}</h1><span className="hdr-sub">{b2}</span></div>
          <div className="hdr-right">
            <button className="chip" title="Öğretici rehberi aç/kapat" onClick={() => (rehberAcik ? rehberKapat() : setRehberAcik(true))}>❔ Rehber</button>
            <button className="chip" title={`${kullanici.ad} (${kullanici.rol}) — çıkış yap`} onClick={cikisYap}>👤 {kullanici.ad.split(" ")[0]} · Çıkış</button>
            <div className="combo" style={{ position: "relative" }}>
              <button className="chip" title="Mükellef değiştir / ekle" onClick={() => setMukAcik(!mukAcik)}>
                🏢 {mukellefler.find((m) => m.id === aktifMuk)?.unvan ?? "Mükellef"} ▾
              </button>
              {mukAcik && (
                <>
                  <div className="backdrop" onClick={() => setMukAcik(false)} />
                  <div className="combo-pop" style={{ right: 0, left: "auto", minWidth: 300 }}>
                    <div className="lbl2" style={{ marginTop: 0 }}>Mükellefler</div>
                    {mukellefler.map((m) => (
                      <div key={m.id} className={"combo-opt" + (m.id === aktifMuk ? " akt" : "")} onClick={() => mukellefSec(m.id)}>
                        <div style={{ fontWeight: 600 }}>{m.unvan}</div>
                        <div style={{ fontSize: 11.5, color: "var(--mut)" }}>VKN {m.vkn || "—"} · {m.sektor_kodlari.map((k) => sektorList.find((x) => x.kod === k)?.ad ?? k).join(", ") || "sektörsüz"}</div>
                      </div>
                    ))}
                    <div style={{ borderTop: "1px solid var(--border2)", marginTop: 6, paddingTop: 8 }}>
                      <div className="lbl2" style={{ marginTop: 0 }}>Yeni mükellef</div>
                      <input placeholder="Unvan" value={yeniUnvan} onChange={(e) => setYeniUnvan(e.target.value)} style={{ marginBottom: 6 }} />
                      <input placeholder="VKN" value={yeniVkn} onChange={(e) => setYeniVkn(e.target.value)} style={{ marginBottom: 6 }} />
                      <select value={yeniSektor} onChange={(e) => setYeniSektor(e.target.value)} style={{ marginBottom: 8 }}>
                        {sektorList.map((sk) => <option key={sk.kod} value={sk.kod}>{sk.ad}</option>)}
                      </select>
                      <button className="btn-dark" style={{ borderRadius: 9, width: "100%", justifyContent: "center" }} onClick={mukellefEkle}>+ Mükellef oluştur</button>
                    </div>
                  </div>
                </>
              )}
            </div>
            <button className="chip"><span className="dot" /> Dönem 2026</button>
            <button className="btn-dark" data-rehber="yeni-kayit" onClick={() => { git("muhasebe"); setMuhAlt("yeni"); }}>+ Yeni kayıt</button>
          </div>
        </header>

        <Rehber gorunum={gorunum} acik={rehberAcik} kapat={rehberKapat} git={(g) => setGorunum(g)} />
        <div className="content">
          {gorunum === "dashboard" && (
            <>
              {fisler.length === 0 && (
                <div className="card" style={{ display: "flex", alignItems: "center", justifyContent: "space-between", gap: 16, flexWrap: "wrap", background: "var(--warn-bg)", borderColor: "var(--warn)" }}>
                  <div style={{ fontSize: 13.5, lineHeight: 1.5 }}><b>Defter boş.</b> Programı denemek için 20.000 kayıtlık örnek e-ticaret defterini yükle ya da kendi ilk fişini gir.</div>
                  <div style={{ display: "flex", gap: 8 }}>
                    <button className="btn" disabled={gonderiliyor} onClick={ornekYukle}>{gonderiliyor ? "Yükleniyor…" : "Örnek defter yükle"}</button>
                    <button className="btn-dark" style={{ borderRadius: 9 }} onClick={() => { git("muhasebe"); setMuhAlt("yeni"); }}>+ Yeni fiş</button>
                  </div>
                </div>
              )}
              <section className="kpis">
                <div className="kpi"><div className="kpi-lbl">Toplam borç <span className="kpi-dot" style={{ background: "var(--pos)" }} /></div>
                  <div className="kpi-val">{tlk(dash.tb)}</div>
                  <div><span className={"pill " + (dash.dengeli ? "pos" : "neg")}>{dash.dengeli ? "✓ Denge" : "Denge bozuk"}</span></div></div>
                <div className="kpi"><div className="kpi-lbl">Toplam alacak <span className="kpi-dot" style={{ background: "var(--neg)" }} /></div>
                  <div className="kpi-val">{tlk(dash.ta)}</div>
                  <div><span className="pill pos">Σborç = Σalacak</span></div></div>
                <div className="kpi"><div className="kpi-lbl">KDV pozisyonu <span className="kpi-dot" style={{ background: "var(--warn)" }} /></div>
                  <div className="kpi-val">{tlk(Math.abs(dash.kdvNet))}</div>
                  <div><span className={"pill " + (dash.kdvNet >= 0 ? "warn" : "neg")}>{dash.kdvNet >= 0 ? "Devreden yönlü" : "Ödenecek yönlü"}</span></div></div>
                <div className="kpi"><div className="kpi-lbl">Kesin fiş <span className="kpi-dot" style={{ background: "var(--nav)" }} /></div>
                  <div className="kpi-val">{dash.fisSayisi}</div>
                  <div><span className="pill pos">ayrı seri no</span></div></div>
              </section>

              <section style={{ display: "grid", gridTemplateColumns: "1.55fr 1fr", gap: 14 }}>
                <div className="card">
                  <div style={{ display: "flex", justifyContent: "space-between", marginBottom: 12 }}>
                    <span className="card-title">Son işlemler</span>
                    <span style={{ fontSize: 12.5, color: "var(--mut)", cursor: "pointer" }} onClick={() => { git("muhasebe"); setMuhAlt("fisler"); }}>Tümünü gör →</span>
                  </div>
                  <table>
                    <thead><tr><th>Tarih</th><th>Fiş no</th><th>Tip</th><th className="num">Tutar</th><th style={{ textAlign: "right" }}>Durum</th></tr></thead>
                    <tbody>
                      {fisler.length === 0 && <tr><td colSpan={5} style={{ color: "var(--mut)" }}>Henüz kayıt yok — "+ Yeni fiş" ile başlayın.</td></tr>}
                      {[...fisler].slice(-6).reverse().map((f) => (
                        <tr key={f.id} className="clickable" onClick={() => { git("muhasebe"); setMuhAlt("fisler"); acFis(f.id); }}>
                          <td style={{ color: "var(--mut)" }}>{f.tarih}</td>
                          <td className="mono">{f.fis_no}</td><td>{f.tip}</td>
                          <td className="num" style={{ fontWeight: 600 }}>{tlk(f.tutar)}</td>
                          <td style={{ textAlign: "right" }}><span className={"pill " + (f.dayanaksiz ? "warn" : "pos")}>{f.dayanaksiz ? "dayanaksız" : "Kesin"}</span></td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
                <div className="card">
                  <div style={{ display: "flex", justifyContent: "space-between", marginBottom: 12 }}>
                    <span className="card-title" data-rehber="takvim-karti">Vergi & beyanname takvimi</span>
                    <span className="pill warn">aylık</span>
                  </div>
                  {[["28", "KDV beyannamesi", "İzleyen ayın 28'ine kadar beyan/ödeme"], ["26", "Muhtasar (stopaj)", "Aylık — ücret/kira stopajı"], ["17", "Geçici vergi", "3 aylık dönemleri izleyen 2. ayın 17'si"], ["30", "Kurumlar vergisi", "Nisan sonu — yıllık beyan"]].map(([g, t, s], i) => (
                    <div key={i} style={{ display: "flex", gap: 12, alignItems: "center", padding: "10px 0", borderBottom: "1px solid var(--border2)", cursor: "pointer" }} title="Vergi sayfasındaki takvimi aç" onClick={() => { setVergiAlt("takvim"); git("vergi"); }}>
                      <div style={{ width: 38, height: 38, borderRadius: 10, background: "var(--warn-bg)", color: "var(--warn)", display: "flex", alignItems: "center", justifyContent: "center", fontWeight: 700, fontSize: 13 }}>{g}</div>
                      <div style={{ lineHeight: 1.3 }}><div style={{ fontSize: 13, fontWeight: 600 }}>{t}</div><div style={{ fontSize: 11.5, color: "var(--mut2)" }}>{s}</div></div>
                    </div>
                  ))}
                </div>
              </section>
            </>
          )}

          {gorunum === "muhasebe" && (
            <div className="pills" data-rehber="muh-pills">
              {([["yeni", "Kayıt"], ["fisler", `Fişler (${fisler.length})`], ["yevmiye", "Yevmiye"], ["muavin", "Muavin"], ["mizan", "Mizan"]] as [string, string][]).map(([id, ad]) => (
                <button key={id} className={"pill-tab" + (muhAlt === id ? " akt" : "")} onClick={() => setMuhAlt(id)}>{ad}</button>
              ))}
              <span style={{ marginLeft: "auto", fontSize: 12, color: "var(--mut)", alignSelf: "center" }}>
                Kayıt → Yevmiye → Kebir → Muavin → Mizan · kronolojik bütünlük
              </span>
            </div>
          )}

          {gorunum === "muhasebe" && muhAlt === "yeni" && (
            <div className="card">
              <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 16, gap: 12, flexWrap: "wrap" }}>
                <span className="card-title" data-rehber="fis-form">Fiş bilgileri</span>
                <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
                  <select data-rehber="sablon" style={{ width: 220 }} value="" onChange={(e) => e.target.value && sablonUygula(e.target.value)}>
                    <option value="">Şablondan başla…</option>
                    {sikKullanilan.length > 0 && (
                      <optgroup label="★ Sık kullandıkların">
                        {sikKullanilan.map((s) => <option key={"sik-" + s.ad} value={s.ad}>{s.ad} ({s.kullanim}×)</option>)}
                      </optgroup>
                    )}
                    <optgroup label="Hazır şablonlar">
                      {gorunenSablonlar.filter((s) => !s.kullanici).map((s) => <option key={s.ad} value={s.ad}>{s.ad}</option>)}
                    </optgroup>
                    {gorunenSablonlar.some((s) => s.kullanici) && (
                      <optgroup label="Kendi şablonlarım">
                        {gorunenSablonlar.filter((s) => s.kullanici).map((s) => <option key={s.ad} value={s.ad}>{s.ad}</option>)}
                      </optgroup>
                    )}
                  </select>
                  <button type="button" className="btn" style={{ fontSize: 12 }} onClick={sablonKayitAc}
                    title="Formdaki satırları şablon olarak kaydet (tutar girmene gerek yok) — ⌘⇧S">+ Şablon kaydet</button>
                  <button type="button" className="btn" style={{ fontSize: 12 }} onClick={() => setSablonYonet(!sablonYonet)}
                    title="Kendi şablonlarını yönet">Yönet</button>
                  <span className="rozet mono">{SERI[tip]}-• taslak</span>
                </div>
              </div>

              {/* Şablon kaydetme penceresi — isim ZORUNLU, tutar gerekmez, mükerrer uyarısı burada. */}
              {sablonTaslak && (
                <div className="modal-bg" onClick={() => setSablonTaslak(null)}>
                  <div className="modal" style={{ width: 560 }} onClick={(e) => e.stopPropagation()}>
                    <div className="modal-hd">
                      <div>
                        <div style={{ fontWeight: 600, fontSize: 15 }}>{sablonTaslak.duzenlenen ? "Şablonu düzenle" : "Şablon kaydet"}</div>
                        <div style={{ fontSize: 12, color: "var(--mut)", marginTop: 2 }}>
                          Hesaplar ve borç/alacak tarafı saklanır — <b>tutarlar saklanmaz</b>.
                        </div>
                      </div>
                      <button type="button" className="modal-x" onClick={() => setSablonTaslak(null)}>✕</button>
                    </div>
                    <div className="modal-body">
                      {/* Mükerrer uyarısı — aynı hesap+taraf kümesi zaten kayıtlıysa */}
                      {mevcutSablon && (
                        <div className="msg" style={{ background: "var(--warn-bg)", color: "var(--warn)", marginBottom: 12 }}>
                          <div>Bu şablon zaten mevcut: <b>“{mevcutSablon.ad}”</b>{!mevcutSablon.kullanici && " (hazır şablon)"}</div>
                          <button type="button" className="btn" style={{ fontSize: 12, marginTop: 8 }}
                            onClick={() => { sablonUygula(mevcutSablon.ad); setSablonTaslak(null); }}>
                            Bu şablonu kullan →
                          </button>
                        </div>
                      )}

                      <label>Şablon adı <span style={{ color: "var(--neg)" }}>*</span></label>
                      <input autoFocus value={sablonTaslak.ad} placeholder="ör. Kira ödemesi — stopajlı"
                        onChange={(e) => { setSablonTaslak({ ...sablonTaslak, ad: e.target.value }); setSablonHata(null); }}
                        onKeyDown={(e) => { if (e.key === "Enter") sablonKaydet(); }}
                        style={{ width: "100%", borderColor: sablonHata ? "var(--neg)" : undefined }} />
                      {sablonlar.some((s) => s.ad === sablonTaslak.ad.trim() && s.kullanici) && (
                        <div style={{ fontSize: 12, color: "var(--warn)", marginTop: 4 }}>Bu adda şablonun var — üzerine yazılacak.</div>
                      )}

                      <div className="lbl2">Satırlar ({sablonTaslak.satirlar.length}) — tarafı değiştirebilirsin</div>
                      <table>
                        <thead><tr><th>Hesap</th><th>Açıklama</th><th style={{ textAlign: "right" }}>Taraf</th></tr></thead>
                        <tbody>
                          {sablonTaslak.satirlar.map((r, i) => (
                            <tr key={i}>
                              <td className="mono">{r.kod} <span style={{ color: "var(--mut2)", fontSize: 12 }}>{r.ad}</span></td>
                              <td style={{ color: "var(--mut)", fontSize: 12.5 }}>{r.aciklama || "—"}</td>
                              <td style={{ textAlign: "right" }}>
                                <button type="button" className="btn" style={{ fontSize: 12, minWidth: 74, color: r.taraf === "B" ? "var(--pos)" : "var(--neg)" }}
                                  title="Borç ↔ Alacak çevir"
                                  onClick={() => setSablonTaslak({
                                    ...sablonTaslak,
                                    satirlar: sablonTaslak.satirlar.map((x, j) => (j === i ? { ...x, taraf: x.taraf === "B" ? "A" : "B" } : x)),
                                  })}>
                                  {r.taraf === "B" ? "Borç" : "Alacak"} ⇄
                                </button>
                              </td>
                            </tr>
                          ))}
                        </tbody>
                      </table>
                      {(!sablonTaslak.satirlar.some((r) => r.taraf === "B") || !sablonTaslak.satirlar.some((r) => r.taraf === "A")) && (
                        <div className="msg" style={{ background: "var(--warn-bg)", color: "var(--warn)", marginTop: 10 }}>
                          Şablonda en az bir <b>borç</b> ve bir <b>alacak</b> satırı olmalı — tek taraflı şablondan denk fiş üretilemez (çift taraflı kayıt).
                        </div>
                      )}
                      {sablonHata && <div className="msg" style={{ background: "var(--neg-bg, #fdeaea)", color: "var(--neg)", marginTop: 10 }}>{sablonHata}</div>}

                      <div style={{ display: "flex", gap: 8, marginTop: 16 }}>
                        <button type="button" className="btn" style={{ fontSize: 13 }} onClick={sablonKaydet}>{sablonTaslak.duzenlenen ? "Güncelle" : "Kaydet"}</button>
                        <button type="button" className="btn" style={{ fontSize: 13 }} onClick={() => setSablonTaslak(null)}>Vazgeç</button>
                      </div>
                    </div>
                  </div>
                </div>
              )}

              {/* Kendi şablonlarını yönet — hazır şablonlar burada listelenmez, onlar silinemez. */}
              {sablonYonet && (
                <div className="detay" style={{ marginTop: 0, marginBottom: 16 }}>
                  <div style={{ fontSize: 12.5, fontWeight: 600, marginBottom: 8 }}>Kendi şablonlarım</div>
                  {sablonlar.filter((s) => s.kullanici).length === 0 ? (
                    <p style={{ fontSize: 12.5, color: "var(--mut)", margin: 0 }}>
                      Henüz şablonun yok. Sık attığın bir kaydı forma gir, <b>+ Şablon kaydet</b>'e bas — hesaplar ve borç/alacak tarafı saklanır, tutarlar saklanmaz.
                    </p>
                  ) : (
                    <table>
                      <thead><tr><th>Ad</th><th>Fiş tipi</th><th>Satırlar</th><th className="num">Kullanım</th><th></th></tr></thead>
                      <tbody>
                        {sablonlar.filter((s) => s.kullanici).map((s) => (
                          <tr key={s.ad}>
                            <td style={{ fontWeight: 600 }}>{s.ad}</td>
                            <td style={{ color: "var(--mut)" }}>{s.tip}</td>
                            <td className="mono" style={{ fontSize: 12 }}>
                              {s.satirlar.map((r) => `${r.kod}${r.taraf === "B" ? " B" : " A"}`).join(" · ")}
                            </td>
                            <td className="num" style={{ color: "var(--mut)" }}>{s.kullanim ? `${s.kullanim}×` : "—"}</td>
                            <td style={{ textAlign: "right", whiteSpace: "nowrap" }}>
                              <button type="button" className="btn" style={{ fontSize: 12 }} onClick={() => sablonDuzenle(s)}>Düzenle</button>{" "}
                              <button type="button" className="btn" style={{ fontSize: 12, borderColor: "var(--neg)", color: "var(--neg)" }} onClick={() => sablonSil(s.ad)}>Sil</button>
                            </td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  )}
                </div>
              )}
              <div className="row">
                <div><label>Fiş tipi</label>
                  <select value={tip} onChange={(e) => setTip(e.target.value)}>
                    <option value="Tahsil">Tahsil</option><option value="Tediye">Tediye</option><option value="Mahsup">Mahsup</option><option value="Acilis">Açılış</option><option value="Kapanis">Kapanış</option>
                  </select></div>
                <div><label>Tarih</label><input placeholder="gg.aa.yyyy" value={tarih} onChange={(e) => setTarih(e.target.value)} /></div>
                <div><label>Belge tipi (dayanak)</label>
                  <select value={belgeTipi} onChange={(e) => setBelgeTipi(e.target.value)}>
                    <option>Fatura</option><option>e-Fatura</option><option>Makbuz</option><option>Dekont</option><option>Sözleşme</option><option value="">— (dayanaksız)</option>
                  </select></div>
                <div><label>Belge no</label><input placeholder="ör. FT-2026-0001" value={belgeNo} onChange={(e) => setBelgeNo(e.target.value)} /></div>
              </div>

              {mesaj && <div className="msg" style={{ background: mesaj.ok ? "var(--pos-bg)" : "var(--neg-bg)", color: mesaj.ok ? "var(--pos)" : "var(--neg)" }}>{mesaj.text}</div>}

              <table>
                <thead><tr><th style={{ width: 26 }}></th><th style={{ width: 64 }}>Kebir</th><th style={{ width: "36%" }}>Hesap kodu · adı</th><th>Açıklama</th><th className="num" style={{ width: "14%" }}>Borç</th><th className="num" style={{ width: "14%" }}>Alacak</th><th style={{ width: 68 }}></th></tr></thead>
                <tbody>
                  {satirlar.map((s, i) => (
                    <tr key={i}
                      className={suruklenen === i ? "surukle-akt" : ""}
                      onDragOver={(e) => { if (suruklenen !== null && suruklenen !== i) e.preventDefault(); }}
                      onDrop={(e) => { e.preventDefault(); if (suruklenen !== null) satirTasi(suruklenen, i); setSuruklenen(null); }}>
                      <td className="tutamak" draggable onDragStart={() => setSuruklenen(i)} onDragEnd={() => setSuruklenen(null)} title="Satırı sürükleyip sırayı değiştir">⠿</td>
                      <td className="mono" style={{ color: "var(--mut)" }}>{s.hesap_kod.split(".")[0] || "—"}</td>
                      <td><HesapSecici hesaplar={hesaplar} value={s.hesap_kod} onChange={(k) => guncelle(i, "hesap_kod", k)}
                        onYeniAlt={(ana) => { setMuavinAna(ana || s.hesap_kod.split(".")[0] || ""); setMuavinAlt(""); setMuavinAd(""); setMuavinHedef(i); }} /></td>
                      <td><AciklamaSecici hesapKod={s.hesap_kod} value={s.aciklama} onChange={(v) => guncelle(i, "aciklama", v)} /></td>
                      <td><input className="num" inputMode="decimal" placeholder="0,00" value={s.borc} onChange={(e) => guncelle(i, "borc", e.target.value)} /></td>
                      <td><input className="num" inputMode="decimal" placeholder="0,00" value={s.alacak} onChange={(e) => guncelle(i, "alacak", e.target.value)} /></td>
                      <td style={{ whiteSpace: "nowrap" }}>
                        <button className="ikon-btn" title="Kalan denge farkını bu satıra yaz" onClick={() => farkiDoldur(i)}>±</button>{" "}
                        <button className="ikon-btn sil" title="Satırı sil" onClick={() => setSatirlar((x) => x.filter((_, j) => j !== i))}>✕</button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>

              {tersBakiyeUyari.length > 0 && (
                <div className="msg" style={{ background: "var(--warn-bg)", color: "var(--warn)", marginTop: 10 }}>
                  ⚠ Ters bakiye uyarısı (engel değil): {tersBakiyeUyari.join(" · ")}
                </div>
              )}

              {muavinAna !== null && (
                <div className="detay">
                  <div style={{ marginBottom: 8, fontWeight: 600, fontSize: 13 }}>Alt hesap ekle {muavinHedef !== null && "(satıra yazılacak)"}</div>
                  <div style={{ display: "flex", gap: 8, alignItems: "center", flexWrap: "wrap" }}>
                    <input placeholder="Ana (ör. 153)" value={muavinAna} onChange={(e) => setMuavinAna(e.target.value)} style={{ width: 110 }} />
                    <input placeholder="Alt kod (ör. 01)" value={muavinAlt} onChange={(e) => setMuavinAlt(e.target.value)} style={{ width: 120 }} />
                    <input placeholder="Ad (ör. X Ticari Mal)" value={muavinAd} onChange={(e) => setMuavinAd(e.target.value)} style={{ flex: 1, minWidth: 160 }} />
                    <span className="mono" style={{ color: "var(--mut)", fontSize: 13 }}>→ {muavinAna || "?"}.{muavinAlt || "??"}</span>
                    <button className="primary" onClick={() => muavinAna && muavinEkle(muavinAna)}>Ekle</button>
                    <button className="btn" onClick={() => { setMuavinAna(null); setMuavinHedef(null); }}>Vazgeç</button>
                  </div>
                </div>
              )}
              <button className="btn" style={{ marginTop: 10 }} onClick={() => setSatirlar((s) => [...s, bosSatir()])}>+ Satır ekle</button>

              <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginTop: 16, paddingTop: 16, borderTop: "1px solid var(--border2)", flexWrap: "wrap", gap: 12 }}>
                <div style={{ display: "flex", gap: 24 }}>
                  <div><label>Toplam borç</label><div className="kpi-val" style={{ fontSize: 18 }}>{tl(toplamBorc)}</div></div>
                  <div><label>Toplam alacak</label><div className="kpi-val" style={{ fontSize: 18 }}>{tl(toplamAlacak)}</div></div>
                </div>
                <div style={{ display: "flex", gap: 10, alignItems: "center" }}>
                  <span className={"pill " + (dengeli ? "pos" : "neg")}>{dengeli ? "● Dengeli" : "● Fark " + tl(Math.abs(toplamBorc - toplamAlacak))}</span>
                  {yetkiVar("kesinlestir")
                    ? <button className="primary" data-rehber="kesinlestir" disabled={!dengeli || gonderiliyor} onClick={kesinlestir}>{gonderiliyor ? "Kaydediliyor…" : "Kesinleştir"}</button>
                    : <span className="pill warn" title="Kesinleştirme yetkisi sorumlu/müdür kademesindedir">Bu kademe kesinleştiremez</span>}
                </div>
              </div>

              {aciklamaKod && (
                <div style={{ marginTop: 16 }}>
                  <div style={{ display: "flex", justifyContent: "space-between", marginBottom: 6 }}>
                    <span className="lbl2">Seçili hesabın resmi işleyişi</span>
                    <button className="btn" style={{ padding: "3px 10px", fontSize: 12 }} onClick={() => setAciklamaKod(null)}>Gizle</button>
                  </div>
                  <AciklamaPanel kod={aciklamaKod} onKarsi={(k) => setAciklamaKod(k)} />
                </div>
              )}
            </div>
          )}

          {gorunum === "muhasebe" && muhAlt === "fisler" && (
            <div className="card" style={{ padding: 0, overflow: "hidden" }}>
              <div style={{ display: "flex", justifyContent: "space-between", padding: "16px 20px", borderBottom: "1px solid var(--border2)" }}>
                <span className="card-title">Fişler</span>
                <button className="btn" onClick={yenileFisler}>Yenile</button>
              </div>
              <table>
                <thead><tr><th style={{ paddingLeft: 20, width: 60 }}>Yevmiye</th><th>Fiş no</th><th>Tip</th><th>Tarih</th><th>Açıklama</th><th className="num">Tutar</th><th style={{ width: 100 }}>Durum</th><th style={{ paddingRight: 20 }}></th></tr></thead>
                <tbody>
                  {fisler.length === 0 && <tr><td colSpan={8} style={{ color: "var(--mut)", padding: 20 }}>Henüz fiş yok.</td></tr>}
                  {fisler.map((f) => (
                    <tr key={f.id} className="clickable" onClick={() => acFis(f.id)} style={f.durum === "İptal edildi" ? { opacity: 0.55 } : undefined}>
                      <td className="mono" style={{ paddingLeft: 20, color: "var(--mut)" }}>{f.yevmiye_no ?? "—"}</td>
                      <td className="mono">{f.fis_no}</td>
                      <td>{f.tip}</td><td style={{ color: "var(--mut)" }}>{f.tarih}</td>
                      <td>{f.aciklama || "—"} {f.dayanaksiz && <span className="pill warn">dayanaksız</span>}</td>
                      <td className="num" style={{ fontWeight: 600 }}>{tlk(f.tutar)}</td>
                      <td><span className={"pill " + (f.durum === "Kesin" ? "pos" : f.durum === "İptal edildi" ? "neg" : "warn")}>{f.durum}</span></td>
                      <td style={{ textAlign: "right", paddingRight: 20, color: "var(--nav)", fontWeight: 600, fontSize: 12.5 }}>Detay →</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}

          {gorunum === "muhasebe" && muhAlt === "mizan" && (
            <div className="card" style={{ padding: 0, overflow: "hidden" }}>
              <div style={{ display: "flex", justifyContent: "space-between", padding: "16px 20px", borderBottom: "1px solid var(--border2)" }}>
                <span className="card-title">Mizan</span>
                <div style={{ display: "flex", gap: 8 }}>
                  <input placeholder="Kebir filtresi (ör. 153, 108)" value={mizanFiltre} onChange={(e) => setMizanFiltre(e.target.value)} style={{ width: 170 }} />
                  <select style={{ width: 190 }} value={mizanSeviye} onChange={(e) => setMizanSeviye(e.target.value as "muavin" | "kebir")}>
                    <option value="muavin">Muavin (tüm hesaplar)</option>
                    <option value="kebir">Kebir (3 haneli özet)</option>
                  </select>
                  <button className="btn" onClick={yenileMizan}>Yenile</button>
                </div>
              </div>
              <table>
                <thead><tr>
                  <th style={{ paddingLeft: 20, width: 60 }}>Kebir</th><th style={{ width: 90 }}>Hesap kodu</th><th>Hesap adı</th><th>Açıklama</th>
                  <th className="num">Borç</th><th className="num">Alacak</th><th className="num">Borç bakiye</th><th className="num">Alacak bakiye</th><th style={{ width: 76 }}></th>
                </tr></thead>
                <tbody>
                  {mizanGoster.length === 0 && <tr><td colSpan={8} style={{ color: "var(--mut)", padding: 20 }}>Henüz kayıt yok.</td></tr>}
                  {mizanGoster.map((m, i) => (
                    <tr key={m.kod} className="clickable" onClick={() => setAciklamaKod(m.kod)}>
                      <td className="mono" style={{ paddingLeft: 20, color: "var(--mut)" }}>{i === 0 || mizanGoster[i - 1].kebir !== m.kebir ? m.kebir : ""}</td>
                      <td className="mono">{m.kod}</td><td>{m.ad}</td>
                      <td style={{ color: "var(--mut2)", fontSize: 12 }}>{m.aciklama}</td>
                      <td className="num">{tl(m.borc)}</td><td className="num">{tl(m.alacak)}</td>
                      <td className="num" style={{ fontWeight: 600 }}>{m.borc_bakiye ? tl(m.borc_bakiye) : "—"}</td>
                      <td className="num" style={{ fontWeight: 600 }}>{m.alacak_bakiye ? tl(m.alacak_bakiye) : "—"}</td>
                      <td style={{ paddingRight: 20, textAlign: "right" }}>
                        <button className="btn" style={{ fontSize: 11.5, padding: "3px 9px" }}
                          onClick={(e) => { e.stopPropagation(); setKebirKod(m.kod); setMuhAlt("kebir"); }}>Kebir →</button>
                      </td>
                    </tr>
                  ))}
                  {mizanGoster.length > 0 && (
                    <tr style={{ fontWeight: 600, background: "var(--bg2)" }}>
                      <td style={{ paddingLeft: 20 }} colSpan={4}>Toplam</td>
                      <td className="num">{tl(mizanGoster.reduce((t, m) => t + m.borc, 0))}</td>
                      <td className="num">{tl(mizanGoster.reduce((t, m) => t + m.alacak, 0))}</td>
                      <td className="num">{tl(mizanGoster.reduce((t, m) => t + m.borc_bakiye, 0))}</td>
                      <td className="num">{tl(mizanGoster.reduce((t, m) => t + m.alacak_bakiye, 0))}</td><td></td>
                    </tr>
                  )}
                </tbody>
              </table>
              {aciklamaKod && <div style={{ padding: 16 }}><AciklamaPanel kod={aciklamaKod} onKarsi={(k) => setAciklamaKod(k)} /></div>}
            </div>
          )}

          {gorunum === "hesaplar" && (
            <div className="card">
              <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 12 }}>
                <span className="card-title">Hesap planı <span style={{ color: "var(--mut2)", fontWeight: 500, fontSize: 12 }}>· {plan.length} hesap</span></span>
                <input placeholder="Kod veya ad ara…" value={hpFiltre} onChange={(e) => setHpFiltre(e.target.value)} style={{ width: 230 }} />
              </div>
              <div style={{ display: "grid", gridTemplateColumns: aciklamaKod ? "1fr 360px" : "1fr", gap: 16 }}>
                <table>
                  <tbody>
                    {gorunenPlan.map((h) => (
                      <tr key={h.kod} className="clickable" onClick={() => setAciklamaKod(h.kod)}>
                        <td style={{ paddingLeft: 8 + (h.seviye - 1) * 18 }}>
                          {!h.yaprak && !hpFiltre
                            ? <button onClick={(e) => { e.stopPropagation(); ataToggle(h.kod); }} style={{ border: "none", background: "none", cursor: "pointer", marginRight: 4, color: "var(--mut2)" }}>{acik.has(h.kod) ? "▾" : "▸"}</button>
                            : <span style={{ display: "inline-block", width: 18 }} />}
                          <span className="mono">{h.kod}</span> <span style={{ fontWeight: h.seviye <= 2 ? 600 : 400 }}>{h.ad}</span>
                        </td>
                        <td style={{ color: "var(--mut2)", fontSize: 12 }}>{h.yaprak ? `${h.tip} · ${h.dogasi}` : ""}</td>
                        <td style={{ textAlign: "right" }}>
                          {h.seviye >= 3 && muavinAna !== h.kod && (
                            <button className="btn" style={{ fontSize: 12, padding: "4px 10px" }} onClick={(e) => { e.stopPropagation(); setMuavinAna(h.kod); setMuavinAlt(""); setMuavinAd(""); }}>+ Alt hesap</button>
                          )}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
                {aciklamaKod && (
                  <div>
                    <div style={{ display: "flex", justifyContent: "space-between", marginBottom: 6 }}>
                      <span className="lbl2">Resmi işleyiş</span>
                      <button className="btn" style={{ padding: "3px 10px", fontSize: 12 }} onClick={() => setAciklamaKod(null)}>Kapat</button>
                    </div>
                    <AciklamaPanel kod={aciklamaKod} onKarsi={(k) => setAciklamaKod(k)} />
                  </div>
                )}
              </div>

              {muavinAna && (
                <div className="detay">
                  <div style={{ marginBottom: 8, fontWeight: 600, fontSize: 13 }}>{muavinAna} altına alt hesap (muavin) ekle</div>
                  <div style={{ display: "flex", gap: 8, alignItems: "center", flexWrap: "wrap" }}>
                    <input placeholder="Alt kod (ör. 01)" value={muavinAlt} onChange={(e) => setMuavinAlt(e.target.value)} style={{ width: 130 }} />
                    <input placeholder="Ad / unvan" value={muavinAd} onChange={(e) => setMuavinAd(e.target.value)} style={{ flex: 1, minWidth: 180 }} />
                    <span className="mono" style={{ color: "var(--mut)", fontSize: 13 }}>→ {muavinAna}.{muavinAlt || "??"}</span>
                    <button className="primary" onClick={() => muavinEkle(muavinAna)}>Ekle</button>
                    <button className="btn" onClick={() => setMuavinAna(null)}>Vazgeç</button>
                  </div>
                </div>
              )}
            </div>
          )}

          {gorunum === "vergi" && (
            <>
              <div className="pills" data-rehber="vergi-pills">
                {([["kdv", "KDV (KDV1 taslak)"], ["mutabakat", "Cari mutabakat"], ["gecici", "Geçici vergi"], ["takvim", "Vergi takvimi"], ["param", "Parametreler"]] as [string, string][]).map(([id, ad]) => (
                  <button key={id} className={"pill-tab" + (vergiAlt === id ? " akt" : "")} onClick={() => setVergiAlt(id)}>{ad}</button>
                ))}
              </div>

              {vergiAlt === "gecici" && gv && (
                <>
                  <div style={{ display: "flex", gap: 8, alignItems: "center", flexWrap: "wrap" }}>
                    {[3, 6, 9, 12].map((c) => (
                      <button key={c} className={"pill-tab" + (gvCeyrek === c ? " akt" : "")} onClick={() => setGvCeyrek(c)}>Q{c / 3} · {QSON[c]}</button>
                    ))}
                    <span style={{ flex: 1 }} />
                    <button className="pill-tab" onClick={() => { donemSec(gvCeyrek); setAnalizAlt("mali"); git("analiz"); }}>Mali tablolar →</button>
                  </div>
                  <div className="card" style={{ padding: 0, overflow: "hidden", maxWidth: 860 }}>
                    <div style={{ padding: "14px 20px", borderBottom: "1px solid var(--border2)", textAlign: "center" }}>
                      <div style={{ fontWeight: 700, fontSize: 14 }}>GEÇİCİ VERGİ HESAP KAĞIDI — Q{gv.ceyrek / 3}</div>
                      <div style={{ fontSize: 12, color: "var(--mut)" }}>{gv.donem} (kümülatif — KVK mük.120 esası) · oran %{gv.oran} · TL</div>
                    </div>
                    <table>
                      <tbody>
                        <tr><td style={{ paddingLeft: 20 }}>Ticari kâr (gelir tablosu — dönem kârı, vergi öncesi)</td><td className="num" style={{ paddingRight: 20, fontWeight: 600 }}>{tl(gv.ticari_kar)}</td></tr>
                        <tr><td style={{ paddingLeft: 20 }}>(+) KKEG — otomatik <span style={{ fontSize: 11.5, color: "var(--mut2)" }}>(689* önek taraması)</span></td><td className="num" style={{ paddingRight: 20 }}>{tl(gv.kkeg_otomatik)}</td></tr>
                        {gvKkeg.map((m, i) => (
                          <tr key={"k" + i}><td style={{ paddingLeft: 36 }}>(+) KKEG — {m.ad} <button className="xg-geri" onClick={() => setGvKkeg(gvKkeg.filter((_, j) => j !== i))}>×</button></td><td className="num" style={{ paddingRight: 20 }}>{tl(m.tutar)}</td></tr>
                        ))}
                        {gvIst.map((m, i) => (
                          <tr key={"i" + i}><td style={{ paddingLeft: 36 }}>(−) İstisna/indirim — {m.ad} <button className="xg-geri" onClick={() => setGvIst(gvIst.filter((_, j) => j !== i))}>×</button></td><td className="num" style={{ paddingRight: 20, color: "var(--pos)" }}>−{tl(m.tutar)}</td></tr>
                        ))}
                        <tr style={{ fontWeight: 700, background: "var(--bg2)" }}><td style={{ paddingLeft: 20 }}>= GEÇİCİ VERGİ MATRAHI</td><td className="num" style={{ paddingRight: 20 }}>{tl(gv.matrah)}</td></tr>
                        <tr><td style={{ paddingLeft: 20 }}>Hesaplanan geçici vergi (%{gv.oran})</td><td className="num" style={{ paddingRight: 20 }}>{tl(gv.hesaplanan)}</td></tr>
                        <tr><td style={{ paddingLeft: 20 }}>(−) Önceki dönemlerde hesaplanan geçici vergi mahsubu</td><td className="num" style={{ paddingRight: 20, color: "var(--pos)" }}>−{tl(gv.onceki_mahsup)}</td></tr>
                        <tr style={{ fontWeight: 700, background: "var(--bg2)" }}><td style={{ paddingLeft: 20 }}>= ÖDENECEK GEÇİCİ VERGİ</td><td className="num" style={{ paddingRight: 20 }}>{tl(gv.odenecek)}</td></tr>
                      </tbody>
                    </table>
                    <div style={{ padding: "12px 20px", borderTop: "1px solid var(--border2)", display: "flex", gap: 8, alignItems: "center", flexWrap: "wrap" }}>
                      <select value={gvYeniTip} onChange={(e) => setGvYeniTip(e.target.value)} style={{ fontSize: 12.5 }}>
                        <option value="kkeg">(+) KKEG</option><option value="istisna">(−) İstisna/indirim</option>
                      </select>
                      <input placeholder="açıklama (ör. binek oto MTV — KVK 11)" value={gvYeniAd} onChange={(e) => setGvYeniAd(e.target.value)} style={{ flex: 1, minWidth: 200, fontSize: 12.5 }} />
                      <input placeholder="tutar (TL)" type="number" step="0.01" value={gvYeniTutar} onChange={(e) => setGvYeniTutar(e.target.value)} style={{ width: 130, fontSize: 12.5 }} />
                      <button className="btn" onClick={gvEkle}>Ekle</button>
                      <button className="btn-dark" style={{ borderRadius: 9 }} onClick={gvKaydet}>Kaydet</button>
                    </div>
                  </div>
                  <p style={{ fontSize: 11.5, color: "var(--mut2)", margin: 0 }}>
                    Ticari kâr → mali kâr köprüsü: KKEG eklenir, istisna/indirim düşülür (KVK md.5, 10, 11). Oran %25 — firma/yıl parametresine (E.5) taşınacak. Manuel satırlar deftere işlemez, hesap kağıdında saklanır.
                  </p>
                </>
              )}

              {vergiAlt === "takvim" && (
                <div className="card" style={{ padding: 0, overflow: "hidden", maxWidth: 760 }}>
                  <div style={{ padding: "14px 20px", borderBottom: "1px solid var(--border2)" }}>
                    <span className="card-title">Vergi takvimi — genel kurallar <span style={{ fontSize: 11.5, color: "var(--mut2)" }}>· kanuni süreler GİB takviminden doğrulanmalı; firma/yıl parametresine taşınacak</span></span>
                  </div>
                  <table>
                    <thead><tr><th style={{ paddingLeft: 20 }}>Beyanname</th><th>Dönem</th><th style={{ paddingRight: 20 }}>Genel süre</th></tr></thead>
                    <tbody>
                      {([["KDV (KDV1)", "Aylık", "İzleyen ayın 28'i (beyan + ödeme)"], ["Muhtasar ve prim hizmet", "Aylık", "İzleyen ayın 26'sı"], ["Geçici vergi", "Q1–Q4 (kümülatif)", "Dönemi izleyen 2. ayın 17'si"], ["Kurumlar vergisi (yıllık)", "Takvim yılı", "İzleyen yıl 1–30 Nisan"], ["Damga vergisi (sürekli)", "Aylık", "İzleyen ayın 26'sı"], ["2 No.lu KDV (tevkifat/sorumlu sıfatıyla)", "Aylık", "İzleyen ayın 28'i"]] as [string, string, string][]).map(([b, d, t]) => (
                        <tr key={b}><td style={{ paddingLeft: 20, fontWeight: 600 }}>{b}</td><td>{d}</td><td style={{ paddingRight: 20 }}>{t}</td></tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              )}

              {vergiAlt === "param" && (
                <div className="card" style={{ padding: 0, overflow: "hidden", maxWidth: 900 }}>
                  <div style={{ padding: "14px 20px", borderBottom: "1px solid var(--border2)" }}>
                    <span className="card-title">Vergi parametreleri — 2026 <span style={{ fontSize: 11.5, color: "var(--mut2)" }}>· data/vergi-parametreleri.json (sürümlü) · kod hardcode etmez, buradan okur · ⚠ = tebliğ teyidi bekliyor</span></span>
                  </div>
                  {vparam.map((k) => (
                    <div key={k.kod} style={{ borderBottom: "1px solid var(--border2)" }}>
                      <div style={{ padding: "10px 20px 4px", fontWeight: 600, fontSize: 13 }}>{k.ad} <span className="mono" style={{ fontSize: 11, color: "var(--mut2)" }}>{k.kod}</span></div>
                      <table style={{ fontSize: 12.5 }}>
                        <tbody>
                          {k.alanlar.map((a, i) => (
                            <tr key={i}>
                              <td style={{ paddingLeft: 20, width: "38%" }}>{a.ad}{!a.dogrulandi && <span title="Tebliğ teyidi bekliyor" style={{ color: "var(--warn)", marginLeft: 5 }}>⚠</span>}</td>
                              <td style={{ fontWeight: 500 }}>{a.deger}</td>
                              <td style={{ paddingRight: 20, color: "var(--mut2)", fontSize: 11.5 }}>{a.programda ?? ""}</td>
                            </tr>
                          ))}
                        </tbody>
                      </table>
                    </div>
                  ))}
                </div>
              )}

              {vergiAlt === "kdv" && kdvBey && (
                <div className="card" style={{ padding: 0, overflow: "hidden", maxWidth: 860 }}>
                  <div style={{ padding: "14px 20px", borderBottom: "1px solid var(--border2)", display: "flex", alignItems: "center", gap: 12, flexWrap: "wrap" }}>
                    <span className="card-title" data-rehber="kdv-karti">KDV1 BEYANNAME TASLAĞI</span>
                    <select value={kdvBeyAy} onChange={(e) => setKdvBeyAy(Number(e.target.value))} style={{ fontSize: 12.5 }}>
                      {AYAD.slice(1).map((a, i) => <option key={i + 1} value={i + 1}>{a} 2026</option>)}
                    </select>
                    <span style={{ fontSize: 11.5, color: "var(--mut2)" }}>{kdvBey.donem} · matrahlar 391 muavin kırılımından (oran = muavin kodu)</span>
                  </div>
                  <table>
                    <thead><tr><th style={{ paddingLeft: 20 }}>Kalem</th><th className="num">Matrah</th><th className="num" style={{ paddingRight: 20 }}>KDV</th></tr></thead>
                    <tbody>
                      {kdvBey.matrahlar.map((m) => (
                        <tr key={m.kod}><td style={{ paddingLeft: 20 }}>Teslim ve hizmetler — %{m.oran} <span className="mono" style={{ fontSize: 11, color: "var(--mut2)" }}>({m.kod})</span></td><td className="num">{tl(m.matrah)}</td><td className="num" style={{ paddingRight: 20 }}>{tl(m.hesaplanan)}</td></tr>
                      ))}
                      {kdvManuel.map((m, i) => (
                        <tr key={"m" + i} style={{ background: "var(--acc-bg)" }}>
                          <td style={{ paddingLeft: 20 }}><span className="pill warn" style={{ fontSize: 10, marginRight: 6 }}>M</span>{m.yon === "indirim" ? "(−) " : "(+) "}{m.ad} <span style={{ fontSize: 11, color: "var(--mut2)" }}>{m.kaynak && `— dayanak: ${m.kaynak}`}</span> <button className="xg-geri" title="Satırı kaldır" onClick={() => kdvManuelKaydet(kdvManuel.filter((_, j) => j !== i))}>×</button></td>
                          <td className="num">{m.matrah ? tl(m.matrah) : ""}</td>
                          <td className="num" style={{ paddingRight: 20, color: m.yon === "indirim" ? "var(--pos)" : undefined }}>{m.yon === "indirim" ? "−" : ""}{tl(m.kdv)}</td>
                        </tr>
                      ))}
                      <tr style={{ fontWeight: 600, background: "var(--bg2)" }}><td style={{ paddingLeft: 20 }}>Hesaplanan KDV toplamı</td><td></td><td className="num" style={{ paddingRight: 20 }}>{tl(kdvBey.hesaplanan_toplam)}</td></tr>
                      {kdvBey.istisna.belge_sayisi > 0 && (
                        <tr title="KDVK 11/12 — matrahı beyan edilir, KDV DOĞMAZ; hesaplanan KDV'ye eklenmez">
                          <td style={{ paddingLeft: 20 }}>İstisna kapsamındaki teslimler <span style={{ fontSize: 11, color: "var(--mut2)" }}>({kdvBey.istisna.belge_sayisi} belge · ihracat)</span></td>
                          <td className="num">{tl(kdvBey.istisna.matrah)}</td>
                          <td className="num" style={{ paddingRight: 20, color: "var(--mut2)" }}>KDV doğmaz</td>
                        </tr>
                      )}
                      {kdvBey.tevkifat.belge_sayisi > 0 && (
                        <>
                          <tr title="KDVK 9 — KDV'nin bir kısmını alıcı sorumlu sıfatıyla 2 No.lu KDV ile beyan eder">
                            <td style={{ paddingLeft: 20 }}>Tevkifat uygulanan işlemler <span style={{ fontSize: 11, color: "var(--mut2)" }}>({kdvBey.tevkifat.belge_sayisi} belge)</span></td>
                            <td className="num">{tl(kdvBey.tevkifat.matrah)}</td>
                            <td className="num" style={{ paddingRight: 20 }}>{tl(kdvBey.tevkifat.toplam_kdv)}</td>
                          </tr>
                          <tr style={{ fontSize: 12.5, color: "var(--mut)" }}>
                            <td style={{ paddingLeft: 36 }}>├ beyan edilen (391'e yazılan)</td><td></td>
                            <td className="num" style={{ paddingRight: 20 }}>{tl(kdvBey.tevkifat.beyan_edilen)}</td>
                          </tr>
                          <tr style={{ fontSize: 12.5, color: "var(--mut)" }}>
                            <td style={{ paddingLeft: 36 }}>└ tevkif edilen (alıcı 2 No.lu KDV ile beyan eder)</td><td></td>
                            <td className="num" style={{ paddingRight: 20 }}>{tl(kdvBey.tevkifat.tevkif_edilen)}</td>
                          </tr>
                        </>
                      )}
                      <tr><td style={{ paddingLeft: 20 }}>(−) Önceki dönemden devreden KDV (190)</td><td></td><td className="num" style={{ paddingRight: 20 }}>{tl(kdvBey.onceki_devreden)}</td></tr>
                      <tr><td style={{ paddingLeft: 20 }}>(−) Bu döneme ait indirilecek KDV (191)</td><td></td><td className="num" style={{ paddingRight: 20 }}>{tl(kdvBey.bu_donem_indirilecek)}</td></tr>
                      {/* KDV1 formunun ayrı bölümü: indirilecek KDV'nin oranlara göre dağılımı */}
                      {kdvBey.indirim_dagilimi.map((m) => (
                        <tr key={"i" + m.kod} style={{ fontSize: 12.5, color: "var(--mut)" }}>
                          <td style={{ paddingLeft: 36 }}>└ %{m.oran} — alınan mal ve hizmet bedeli <span className="mono" style={{ fontSize: 11, color: "var(--mut2)" }}>({m.kod})</span></td>
                          <td className="num">{tl(m.matrah)}</td>
                          <td className="num" style={{ paddingRight: 20 }}>{tl(m.hesaplanan)}</td>
                        </tr>
                      ))}
                      <tr style={{ fontWeight: 700, background: "var(--bg2)" }}>
                        <td style={{ paddingLeft: 20 }}>{kdvBey.odenecek > 0 ? "= ÖDENECEK KDV (→ 360)" : "= SONRAKİ DÖNEME DEVREDEN KDV (→ 190)"}</td><td></td>
                        <td className="num" style={{ paddingRight: 20, color: kdvBey.odenecek > 0 ? "var(--neg)" : "var(--pos)" }}>{tl(kdvBey.odenecek > 0 ? kdvBey.odenecek : kdvBey.devreden)}</td>
                      </tr>
                    </tbody>
                  </table>
                  <div style={{ padding: "12px 20px", borderTop: "1px solid var(--border2)", display: "flex", gap: 8, alignItems: "center", flexWrap: "wrap" }}>
                    <select value={kmYon} onChange={(e) => setKmYon(e.target.value)} style={{ width: 150, fontSize: 12.5 }}>
                      <option value="indirim">(−) İndirim/istisna</option><option value="hesaplanan">(+) İlave matrah</option>
                    </select>
                    <input placeholder="açıklama (ör. tevkifat düzeltmesi — KDVK 9)" value={kmAd} onChange={(e) => setKmAd(e.target.value)} style={{ flex: 1, minWidth: 180, fontSize: 12.5 }} />
                    <input placeholder="matrah (TL)" type="number" step="0.01" value={kmMatrah} onChange={(e) => setKmMatrah(e.target.value)} style={{ width: 110, fontSize: 12.5 }} />
                    <input placeholder="KDV (TL)" type="number" step="0.01" value={kmKdv} onChange={(e) => setKmKdv(e.target.value)} style={{ width: 100, fontSize: 12.5 }} />
                    <input placeholder="dayanak (fiş/belge)" value={kmKaynak} onChange={(e) => setKmKaynak(e.target.value)} style={{ width: 170, fontSize: 12.5 }} />
                    <button className="btn" onClick={kdvManuelEkle}>Ekle</button>
                  </div>
                  <div style={{ padding: "8px 20px 14px", fontSize: 11.5, color: "var(--mut2)", lineHeight: 1.7 }}>
                    {kdvBey.kaynak_notlari.map((n, i) => <div key={i}>• {n}</div>)}
                  </div>
                </div>
              )}

              {vergiAlt === "mutabakat" && babs && (
                <div className="card" style={{ padding: 0, overflow: "hidden", maxWidth: 980 }}>
                  <div style={{ padding: "14px 20px", borderBottom: "1px solid var(--border2)", display: "flex", alignItems: "center", gap: 12, flexWrap: "wrap" }}>
                    <span className="card-title">CARİ BAZLI ALIM / SATIM MUTABAKATI</span>
                    <select value={kdvBeyAy} onChange={(e) => setKdvBeyAy(Number(e.target.value))} style={{ fontSize: 12.5 }}>
                      {AYAD.slice(1).map((a, i) => <option key={i + 1} value={i + 1}>{a} 2026</option>)}
                    </select>
                    <span style={{ fontSize: 11.5, color: "var(--mut2)" }}>{babs.donem} · KDV hariç · iade ve iskonto düşülmüş</span>
                  </div>

                  {/* Yürürlük uyarısı — bu bir beyanname DEĞİL. Kullanıcı yanlış sanmasın. */}
                  <div className="msg" style={{ margin: "12px 20px", background: "var(--warn-bg)", color: "var(--warn)" }}>
                    <b>Bu bir beyanname değildir.</b> Ba/Bs bildirim zorunluluğu <b>{babs.kaldirilma_tarihi}</b> tarihinde
                    kaldırıldı ({babs.dayanak}); GİB alım/satım verisine e-Fatura/e-Arşiv üzerinden doğrudan eriştiği için
                    ayrı bildirime gerek kalmadı. Bu tablo <b>iç mutabakat</b> ve <b>çapraz kontrol</b> içindir:
                    satışlarımız karşı tarafın alış kaydıyla tutmalıdır.
                  </div>

                  {babs.vkn_eksik.length > 0 && (
                    <div className="msg" style={{ margin: "0 20px 12px", background: "var(--neg-bg, #fdeaea)", color: "var(--neg)" }}>
                      VKN'si eksik cari: {babs.vkn_eksik.join(", ")} — çapraz kontrol yapılamaz.
                    </div>
                  )}

                  {([["bs", babs.bs, "SATIŞLAR"], ["ba", babs.ba, "ALIMLAR"]] as [string, Babs["bs"], string][]).map(([id, blok, baslik]) => (
                    <div key={id}>
                      <div style={{ padding: "10px 20px", background: "var(--bg2)", fontSize: 12.5, fontWeight: 600 }}>
                        {baslik} <span style={{ color: "var(--mut2)", fontWeight: 500 }}>· {blok.aciklama} · {blok.bildirilecek_cari} cari · {tl(blok.toplam)} ₺</span>
                      </div>
                      <table>
                        <thead><tr><th style={{ paddingLeft: 20 }}>VKN</th><th>Ünvan</th><th className="num">Belge</th><th className="num" style={{ paddingRight: 20 }}>Tutar (KDV hariç)</th></tr></thead>
                        <tbody>
                          {blok.satirlar.length === 0 && <tr><td colSpan={4} style={{ color: "var(--mut)", padding: 16 }}>Bu ayda kayıt yok.</td></tr>}
                          {blok.satirlar.map((x) => (
                            <tr key={id + x.vkn}>
                              <td className="mono" style={{ paddingLeft: 20 }}>{x.vkn || <span style={{ color: "var(--neg)" }}>eksik</span>}</td>
                              <td>{x.unvan}</td>
                              <td className="num" style={{ color: "var(--mut)" }}>{x.belge_sayisi}</td>
                              <td className="num" style={{ paddingRight: 20, fontWeight: 600 }}>{tl(x.tutar)}</td>
                            </tr>
                          ))}
                        </tbody>
                      </table>
                    </div>
                  ))}
                </div>
              )}

              {vergiAlt === "kdv" && !kdv && <div className="card" style={{ color: "var(--mut)" }}>Yükleniyor… <button className="btn" onClick={yenileKdv}>Getir</button></div>}
              {vergiAlt === "kdv" && kdv && (
                <>
                  <section className="kpis" style={{ gridTemplateColumns: "repeat(4, 1fr)" }}>
                    <div className="kpi"><div className="kpi-lbl">191 İndirilecek KDV (mahsup edilmemiş)</div><div className="kpi-val">{tlk(kdv.indirilecek)}</div></div>
                    <div className="kpi"><div className="kpi-lbl">190 Devreden (önceki)</div><div className="kpi-val">{tlk(kdv.devreden)}</div></div>
                    <div className="kpi"><div className="kpi-lbl">391 Hesaplanan KDV</div><div className="kpi-val">{tlk(kdv.hesaplanan)}</div></div>
                    <div className="kpi"><div className="kpi-lbl">Sonuç</div>
                      <div className="kpi-val" style={{ color: kdv.fark > 0 ? "var(--neg)" : "var(--pos)" }}>{tlk(Math.abs(kdv.fark))}</div>
                      <div><span className={"pill " + (kdv.fark > 0 ? "neg" : "pos")}>{kdv.fark > 0 ? "360 Ödenecek" : kdv.fark < 0 ? "190 Devreden" : "—"}</span></div></div>
                  </section>
                  <div className="card" style={{ display: "flex", justifyContent: "space-between", alignItems: "center", gap: 12, flexWrap: "wrap" }}>
                    <div style={{ fontSize: 13, color: "var(--lbl)", lineHeight: 1.5 }}>
                      Ay sonu mahsubu: <span className="mono">391 borç / 191+190 alacak</span>; fark <span className="mono">360</span>'a (ödenecek) veya <span className="mono">190</span>'a (devreden) yazılır.
                    </div>
                    <button className="primary" disabled={kdv.indirilecek + kdv.devreden + kdv.hesaplanan === 0} onClick={kdvMahsup}>Mahsup fişi oluştur</button>
                  </div>
                </>
              )}
            </>
          )}

          {gorunum === "muhasebe" && muhAlt === "yevmiye" && (
            <div className="card" style={{ padding: 0, overflow: "hidden" }}>
              <div style={{ padding: "16px 20px", borderBottom: "1px solid var(--border2)" }}><span className="card-title">Yevmiye defteri</span></div>
              {maddeler.length === 0 && <div style={{ padding: 20, color: "var(--mut)" }}>Henüz kesin madde yok.</div>}
              {maddeler.map((m) => (
                <div key={m.fis_no} style={{ borderBottom: "1px solid var(--border2)", padding: "10px 20px" }}>
                  <div style={{ display: "flex", gap: 12, fontSize: 12.5, color: "var(--mut)", marginBottom: 6, flexWrap: "wrap" }}>
                    <span className="mono" style={{ fontWeight: 600, color: "var(--ink)" }}>Madde {m.yevmiye_no}</span>
                    <span>{m.tarih}</span><span className="mono">{m.fis_no}</span><span>{m.tip}</span>
                    {m.belge && <span className="rozet">{m.belge}</span>}
                    <span style={{ flex: 1, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{m.aciklama}</span>
                    <span className="mono" style={{ fontWeight: 600, color: "var(--ink)" }}>{tl(m.toplam)}</span>
                  </div>
                  <table style={{ fontSize: 12.5 }}>
                    <tbody>
                      {m.satirlar.map((s, i) => (
                        <tr key={i}>
                          <td className="mono" style={{ width: 90, paddingLeft: s.borc ? 0 : 28 }}>{s.kod}</td>
                          <td>{s.ad}</td>
                          <td className="num" style={{ width: 120 }}>{s.borc ? tl(s.borc) : ""}</td>
                          <td className="num" style={{ width: 120 }}>{s.alacak ? tl(s.alacak) : ""}</td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              ))}
              {maddeler.length > 0 && (
                <div style={{ display: "flex", justifyContent: "space-between", padding: "12px 20px", background: "var(--bg2)", fontWeight: 600, fontSize: 13 }}>
                  <span>Genel toplam (nakli yekûn) — {maddeler.length} madde</span>
                  <span className="mono">{tl(maddeler.reduce((t, m) => t + m.toplam, 0))}</span>
                </div>
              )}
            </div>
          )}

          {gorunum === "muhasebe" && muhAlt === "kebir" && (
            <div className="card">
              <div style={{ display: "flex", gap: 12, alignItems: "center", marginBottom: 14 }}>
                <button className="btn" style={{ fontSize: 12 }} onClick={() => setMuhAlt("mizan")}>← Mizan</button>
                <span className="card-title">Defter-i kebir</span>
                <div style={{ width: 320 }}><HesapSecici hesaplar={hesaplar} value={kebirKod} onChange={setKebirKod} /></div>
              </div>
              {!kebirKod && <div style={{ color: "var(--mut)", fontSize: 13 }}>Hesap seçin — hareketler ve yürüyen bakiye listelenir.</div>}
              {kebirData && kebirKod && (
                <table>
                  <thead><tr><th>Tarih</th><th style={{ width: 66 }}>Yevmiye</th><th>Fiş no</th><th>Açıklama</th><th>Karşı hesap</th><th className="num">Borç</th><th className="num">Alacak</th><th className="num">Bakiye</th></tr></thead>
                  <tbody>
                    {kebirData.hareketler.length === 0 && <tr><td colSpan={8} style={{ color: "var(--mut)" }}>Bu hesapta hareket yok.</td></tr>}
                    {kebirData.hareketler.map((h, i) => (
                      <tr key={i}>
                        <td style={{ color: "var(--mut)" }}>{h.tarih}</td>
                        <td className="mono" style={{ color: "var(--mut)" }}>{h.yevmiye_no ?? "—"}</td>
                        <td className="mono">{h.fis_no}</td><td>{h.aciklama || "—"}</td>
                        <td className="mono" style={{ fontSize: 12, color: "var(--mut)" }}>{h.karsi}</td>
                        <td className="num">{h.borc ? tl(h.borc) : ""}</td><td className="num">{h.alacak ? tl(h.alacak) : ""}</td>
                        <td className="num" style={{ fontWeight: 600 }}>{tl(Math.abs(h.yuruyen_bakiye))} <span style={{ fontSize: 11, color: "var(--mut)" }}>{h.yuruyen_bakiye >= 0 ? "B" : "A"}</span></td>
                      </tr>
                    ))}
                    {kebirData.hareketler.length > 0 && (
                      <tr style={{ fontWeight: 600, background: "var(--bg2)" }}>
                        <td colSpan={5}>Toplam</td>
                        <td className="num">{tl(kebirData.hareketler.reduce((t, h) => t + h.borc, 0))}</td>
                        <td className="num">{tl(kebirData.hareketler.reduce((t, h) => t + h.alacak, 0))}</td>
                        <td className="num">{(() => { const b = kebirData.hareketler[kebirData.hareketler.length - 1].yuruyen_bakiye; return <>{tl(Math.abs(b))} <span style={{ fontSize: 11 }}>{b >= 0 ? "B" : "A"}</span></>; })()}</td>
                      </tr>
                    )}
                  </tbody>
                </table>
              )}
            </div>
          )}

          {gorunum === "muhasebe" && muhAlt === "muavin" && (
            <div className="card" style={{ padding: 0, overflow: "hidden" }}>
              <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", padding: "16px 20px", borderBottom: "1px solid var(--border2)", gap: 12, flexWrap: "wrap" }}>
                <span className="card-title">Muavin defteri <span style={{ color: "var(--mut2)", fontWeight: 500, fontSize: 12 }}>· {muavinOzet.length} hesap · hesaba tıklayınca hareketler yüklenir</span></span>
                <div style={{ display: "flex", gap: 8 }}>
                  <input placeholder="Hesap filtresi (ör. 120 veya 153.01)" value={muavinFiltre} onChange={(e) => setMuavinFiltre(e.target.value.trim())} style={{ width: 220 }} />
                  <button className="primary" onClick={() => window.open(`${API}/api/muavin/txt${muavinFiltre ? `?kod=${muavinFiltre}` : ""}`, "_blank")}>TXT indir</button>
                </div>
              </div>
              {muavinOzet.length === 0 && <div style={{ padding: 20, color: "var(--mut)" }}>Hareket gören hesap yok.</div>}
              {muavinOzet.map((b) => {
                const acikD = acikHesap[b.kod];
                return (
                  <div key={b.kod} style={{ borderBottom: "1px solid var(--border2)" }}>
                    <div className="clickable" onClick={() => hesapToggle(b.kod)}
                      style={{ display: "flex", justifyContent: "space-between", gap: 12, padding: "10px 20px", background: "var(--bg2)", fontWeight: 600, fontSize: 13, cursor: "pointer" }}>
                      <span>{acikD ? "▾" : "▸"} <span className="mono">{b.kod}</span> {b.ad}
                        <span style={{ fontWeight: 400, color: "var(--mut2)", fontSize: 12 }}> · {b.hareket_sayisi.toLocaleString("tr-TR")} hareket</span></span>
                      <span style={{ display: "flex", gap: 18 }}>
                        <span className="mono" style={{ color: "var(--mut)" }}>B {tl(b.toplam_borc)}</span>
                        <span className="mono" style={{ color: "var(--mut)" }}>A {tl(b.toplam_alacak)}</span>
                        <span className="mono">{tl(Math.abs(b.bakiye))} {b.bakiye >= 0 ? "B" : "A"}</span>
                      </span>
                    </div>
                    {acikD && (
                      <>
                        {acikD.hareketler.length < acikD.toplam && (
                          <div style={{ padding: "8px 20px" }}>
                            <button className="btn" style={{ fontSize: 12 }} onClick={() => hareketYukle(b.kod, acikD.hareketler.length)}>
                              ↑ Daha eski 200 hareketi yükle ({acikD.hareketler.length.toLocaleString("tr-TR")} / {acikD.toplam.toLocaleString("tr-TR")})
                            </button>
                          </div>
                        )}
                        <table style={{ fontSize: 12.5 }}>
                          <thead><tr><th style={{ paddingLeft: 20 }}>Tarih</th><th>Yevmiye</th><th>Fiş no</th><th>Açıklama</th><th>Karşı hesap</th><th className="num">Borç</th><th className="num">Alacak</th><th className="num" style={{ paddingRight: 20 }}>Bakiye</th></tr></thead>
                          <tbody>
                            {acikD.hareketler.map((h, i) => (
                              <tr key={i}>
                                <td style={{ paddingLeft: 20, color: "var(--mut)" }}>{h.tarih}</td>
                                <td className="mono" style={{ color: "var(--mut)" }}>{h.yevmiye_no ?? "—"}</td>
                                <td className="mono">{h.fis_no}</td><td>{h.aciklama || "—"}</td>
                                <td className="mono" style={{ fontSize: 11.5, color: "var(--mut)" }}>{h.karsi}</td>
                                <td className="num">{h.borc ? tl(h.borc) : ""}</td><td className="num">{h.alacak ? tl(h.alacak) : ""}</td>
                                <td className="num" style={{ paddingRight: 20, fontWeight: 600 }}>{tl(Math.abs(h.yuruyen_bakiye))} <span style={{ fontSize: 10.5, color: "var(--mut)" }}>{h.yuruyen_bakiye >= 0 ? "B" : "A"}</span></td>
                              </tr>
                            ))}
                          </tbody>
                        </table>
                      </>
                    )}
                  </div>
                );
              })}
            </div>
          )}

          {gorunum === "bilanco" && bilancoD && (
            <section style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 14 }}>
              <div className="card">
                <span className="card-title">Aktif (Varlıklar)</span>
                <table style={{ marginTop: 10 }}>
                  <tbody>
                    <tr><td>1 Dönen varlıklar</td><td className="num">{tl(bilancoD.donen)}</td></tr>
                    <tr><td>2 Duran varlıklar</td><td className="num">{tl(bilancoD.duran)}</td></tr>
                    <tr style={{ fontWeight: 600, background: "var(--bg2)" }}><td>Aktif toplam</td><td className="num">{tl(bilancoD.aktif)}</td></tr>
                  </tbody>
                </table>
              </div>
              <div className="card">
                <span className="card-title">Pasif (Kaynaklar)</span>
                <table style={{ marginTop: 10 }}>
                  <tbody>
                    <tr><td>3 Kısa vadeli yabancı kaynaklar</td><td className="num">{tl(bilancoD.kvyk)}</td></tr>
                    <tr><td>4 Uzun vadeli yabancı kaynaklar</td><td className="num">{tl(bilancoD.uvyk)}</td></tr>
                    <tr><td>5 Öz kaynaklar</td><td className="num">{tl(bilancoD.oz)}</td></tr>
                    <tr><td style={{ paddingLeft: 24, color: "var(--mut)" }}>Dönem kârı/zararı</td><td className="num">{tl(bilancoD.kar)}</td></tr>
                    <tr style={{ fontWeight: 600, background: "var(--bg2)" }}><td>Pasif toplam</td><td className="num">{tl(bilancoD.pasif)}</td></tr>
                  </tbody>
                </table>
                <div style={{ marginTop: 10 }}>
                  <span className={"pill " + (bilancoD.aktif === bilancoD.pasif ? "pos" : "neg")}>{bilancoD.aktif === bilancoD.pasif ? "✓ Aktif = Pasif" : "Denklik bozuk!"}</span>
                </div>
              </div>
            </section>
          )}

          {gorunum === "gelirt" && gt && (
            <div className="card" style={{ maxWidth: 560 }}>
              <span className="card-title">Gelir tablosu</span>
              <table style={{ marginTop: 10 }}>
                <tbody>
                  {([["Brüt satışlar", gt.brut_satislar], ["Satış indirimleri (−)", -gt.satis_indirimleri], ["NET SATIŞLAR", gt.net_satislar], ["Satışların maliyeti (−)", -gt.smm], ["BRÜT SATIŞ KÂRI", gt.brut_kar], ["Faaliyet giderleri (−)", -gt.faaliyet_giderleri], ["FAALİYET KÂRI", gt.faaliyet_kari], ["Diğer olağan gelirler", gt.diger_gelir], ["Diğer olağan giderler (−)", -gt.diger_gider], ["Finansman giderleri (−)", -gt.finansman], ["OLAĞAN KÂR", gt.olagan_kar], ["Olağandışı gelirler", gt.od_gelir], ["Olağandışı giderler (−)", -gt.od_gider], ["DÖNEM KÂRI (vergi öncesi)", gt.donem_kari], ["Vergi ve yasal yük. karşılığı (−)", -gt.vergi_karsiligi], ["DÖNEM NET KÂRI", gt.donem_net_kari]] as [string, number][]).map(([ad, deger]) => (
                    <tr key={ad} style={ad === ad.toUpperCase() && ad.length > 6 ? { fontWeight: 600, background: "var(--bg2)" } : undefined}>
                      <td>{ad}</td><td className="num" style={{ color: deger < 0 ? "var(--neg)" : "var(--ink)" }}>{tl(deger)}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}

          {gorunum === "banka" && <Banka />}

          {gorunum === "havada" && <Havada />}
          {gorunum === "belgealim" && <BelgeAlim />}

          {gorunum === "belgeler" && <Belgeler />}

          {gorunum === "analiz" && analiz && (
            <>
              <div className="pills" data-rehber="analiz-pills">
                {([["oranlar", "Oranlar"], ["mali", "Mali tablolar"], ["aylik", "Aylık & kur"]] as [string, string][]).map(([id, ad]) => (
                  <button key={id} className={"pill-tab" + (analizAlt === id ? " akt" : "")} onClick={() => setAnalizAlt(id)}>{ad}</button>
                ))}
                <span style={{ marginLeft: "auto", display: "flex", gap: 8, alignItems: "center" }}>
                  <label style={{ margin: 0, fontSize: 12 }}><span data-rehber="donem-secici">Raporlama dönemi</span></label>
                  <select style={{ width: 160 }} value={analizAy} onChange={(e) => donemSec(Number(e.target.value))}>
                    {[3, 6, 9, 12].map((a) => <option key={a} value={a}>{qlab(a)}</option>)}
                  </select>
                  <span style={{ fontSize: 11.5, color: "var(--mut)" }}>
                    {oncekiCeyrek(analizAy) ? `karşılaştırma: tx-1 = ${qlab(oncekiCeyrek(analizAy)!)}` : "tx-1 yok (önceki yıl verisi açılınca gelecek)"}
                  </span>
                </span>
              </div>

              {analiz.kiyas && analizAlt === "oranlar" && (
                <div className="card" style={{ borderLeft: "3px solid var(--warn)" }}>
                  <span className="card-title">Dönem karşılaştırma yorumu (tx-1 {qlab(analiz.kiyas.ay)} → tx {qlab(analiz.ay)})</span>
                  <ul style={{ margin: "8px 0 0", paddingLeft: 18, fontSize: 12.5, color: "var(--lbl)", lineHeight: 1.7 }}>
                    {analiz.kiyas.yorumlar.map((y, i) => <li key={i}>{y}</li>)}
                  </ul>
                </div>
              )}

              {analizAlt === "oranlar" && ["Likidite", "Finansal yapı", "Devir hızları", "Kârlılık"].map((grup) => (
                <div key={grup}>
                  <div style={{ fontSize: 13, fontWeight: 600, color: "var(--lbl)", margin: "4px 0 8px" }}>{grup}</div>
                  <section className="kpis" style={{ gridTemplateColumns: "repeat(auto-fit, minmax(250px, 1fr))" }}>
                    {analiz.oranlar.filter((o) => o.grup === grup).map((o) => {
                      const onceki = analiz.kiyas?.oranlar.find((x) => x.ad === o.ad);
                      return (
                        <div key={o.ad} className="kpi">
                          <div className="kpi-lbl">{o.ad}</div>
                          <div style={{ display: "flex", alignItems: "center", gap: 10, flexWrap: "wrap" }}>
                            <span className="kpi-val" style={{ fontSize: 20 }}>{fmt(o.deger, o.birim)}</span>
                            <span className={"pill " + durumPil(o.durum)}>{o.durum}</span>
                            {onceki && <span className="pill" style={{ background: "var(--bg2)", color: "var(--nav)" }}>tx-1: {fmt(onceki.deger, o.birim)}</span>}
                          </div>
                          <div style={{ fontSize: 12, color: "var(--lbl)", lineHeight: 1.45 }}>{o.yorum}</div>
                          {o.banka && <div style={{ fontSize: 11.5, color: "var(--mut)", lineHeight: 1.4, borderTop: "1px dashed var(--border)", paddingTop: 6 }}>Banka görünümü: {o.banka}</div>}
                        </div>
                      );
                    })}
                  </section>
                </div>
              ))}

              {analizAlt === "mali" && (() => {
                // Sunum esasları (PDF): hesap tipi bilanço — aktif|pasif KARŞILIKLI (Genel Muh. Şekil 2.3);
                // karşılaştırmalı kolonlar tx-1|tx (MSUGT); GT dönem başından kümülatif (TMS 34).
                const grupAd = (g: string) => plan.find((h) => h.kod === g)?.ad ?? "";
                const tarafSatirlar = (taraf: string) => {
                  const simdi = analiz.kebir_bilanco.filter((k) => k.taraf === taraf);
                  const once = analiz.kiyas?.kebir_bilanco.filter((k) => k.taraf === taraf) ?? [];
                  const kodlar = [...new Set([...once.map((k) => k.kod), ...simdi.map((k) => k.kod)])].sort();
                  return kodlar.map((kod) => ({
                    kod,
                    ad: simdi.find((k) => k.kod === kod)?.ad || once.find((k) => k.kod === kod)?.ad || "",
                    tx: simdi.find((k) => k.kod === kod)?.bakiye ?? 0,
                    tx1: once.find((k) => k.kod === kod)?.bakiye ?? null,
                  }));
                };
                const TarafTablo = ({ taraf }: { taraf: string }) => {
                  const satirlar = tarafSatirlar(taraf);
                  let sonGrup = "";
                  return (
                    <table style={{ fontSize: 12.5 }}>
                      <thead><tr>
                        <th style={{ paddingLeft: 12 }}>{taraf === "Aktif" ? "AKTİF (Varlıklar)" : "PASİF (Kaynaklar)"}</th>
                        {analiz.kiyas && <th className="num">tx-1</th>}<th className="num" style={{ paddingRight: 12 }}>tx</th>
                      </tr></thead>
                      <tbody>
                        {satirlar.map((r) => {
                          const grup = r.kod.slice(0, 2);
                          const baslikGerek = /^\d/.test(r.kod) && grup !== sonGrup;
                          if (baslikGerek) sonGrup = grup;
                          return [
                            baslikGerek ? (
                              <tr key={"g" + grup} style={{ background: "var(--bg2)" }}>
                                <td colSpan={analiz.kiyas ? 3 : 2} style={{ paddingLeft: 12, fontWeight: 600, fontSize: 11.5, color: "var(--lbl)" }}>{grup} {grupAd(grup).toLocaleUpperCase("tr")}</td>
                              </tr>
                            ) : null,
                            <tr key={r.kod}>
                              <td style={{ paddingLeft: 24 }}><span className="mono">{r.kod}</span> {r.ad}</td>
                              {analiz.kiyas && <td className="num" style={{ color: "var(--mut)" }}>{r.tx1 != null ? tl(r.tx1) : "—"}</td>}
                              <td className="num" style={{ paddingRight: 12, fontWeight: 600 }}>{tl(r.tx)}</td>
                            </tr>,
                          ];
                        })}
                        <tr style={{ fontWeight: 700, background: "var(--bg2)" }}>
                          <td style={{ paddingLeft: 12 }}>{taraf.toLocaleUpperCase("tr")} TOPLAMI</td>
                          {analiz.kiyas && <td className="num">{tl((analiz.kiyas.kebir_bilanco.filter((k) => k.taraf === taraf)).reduce((t, k) => t + k.bakiye, 0))}</td>}
                          <td className="num" style={{ paddingRight: 12 }}>{tl(analiz.kebir_bilanco.filter((k) => k.taraf === taraf).reduce((t, k) => t + k.bakiye, 0))}</td>
                        </tr>
                      </tbody>
                    </table>
                  );
                };
                return (
                  <>
                    <div className="card" style={{ padding: 0, overflow: "hidden" }}>
                      <div style={{ padding: "14px 20px", borderBottom: "1px solid var(--border2)", textAlign: "center" }}>
                        <div style={{ fontWeight: 700, fontSize: 14 }}>KARŞILAŞTIRMALI BİLANÇO</div>
                        <div style={{ fontSize: 12, color: "var(--mut)" }}>{QSON[analiz.ay]} itibarıyla{analiz.kiyas ? ` · tx-1: ${QSON[analiz.kiyas.ay]}` : ""} · hesap tipi (aktif–pasif karşılıklı) · TL</div>
                      </div>
                      <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 0, borderBottom: "1px solid var(--border2)" }}>
                        <div style={{ borderRight: "1px solid var(--border2)" }}><TarafTablo taraf="Aktif" /></div>
                        <div><TarafTablo taraf="Pasif" /></div>
                      </div>
                    </div>
                    <div className="card" style={{ padding: 0, overflow: "hidden", maxWidth: 760 }}>
                      <div style={{ padding: "14px 20px", borderBottom: "1px solid var(--border2)", textAlign: "center", position: "relative" }}>
                        <div style={{ fontWeight: 700, fontSize: 14 }}>KARŞILAŞTIRMALI GELİR TABLOSU</div>
                        <div style={{ fontSize: 12, color: "var(--mut)" }}>01.01.2026 – {QSON[analiz.ay]}{analiz.kiyas ? ` · tx-1: 01.01.2026 – ${QSON[analiz.kiyas.ay]}` : ""} (kümülatif) · TL</div>
                        <button className="pill-tab" style={{ position: "absolute", right: 14, top: 12, fontSize: 12 }} title="Bu dönemin kârından geçici vergi hesap kağıdına geç"
                          onClick={() => { setGvCeyrek(analiz.ay); setVergiAlt("gecici"); git("vergi"); }}>Geçici vergi (Q{analiz.ay / 3}) →</button>
                      </div>
                      <table>
                        <thead><tr><th style={{ paddingLeft: 20 }}>Kalem</th>{analiz.kiyas && <th className="num">tx-1</th>}<th className="num">tx</th>{analiz.kiyas && <th className="num" style={{ paddingRight: 20 }}>Δ%</th>}</tr></thead>
                        <tbody>
                          {([["Brüt satışlar", "brut_satislar"], ["Satış indirimleri (−)", "satis_indirimleri"], ["NET SATIŞLAR", "net_satislar"], ["Satışların maliyeti (−)", "smm"], ["BRÜT SATIŞ KÂRI", "brut_kar"], ["Faaliyet giderleri (−)", "faaliyet_giderleri"], ["FAALİYET KÂRI", "faaliyet_kari"], ["Finansman giderleri (−)", "finansman"], ["DÖNEM KÂRI (vergi öncesi)", "donem_kari"], ["Vergi karşılığı (−)", "vergi_karsiligi"], ["DÖNEM NET KÂRI", "donem_net_kari"]] as [string, string][]).map(([ad, key]) => {
                            const v = analiz.gt[key] ?? 0;
                            const o = analiz.kiyas?.gt[key];
                            const dlt = o ? (100 * (v - o)) / Math.abs(o || 1) : null;
                            const kalin = ad === ad.toUpperCase() && ad.length > 6;
                            return (
                              <tr key={ad} style={kalin ? { fontWeight: 600, background: "var(--bg2)" } : undefined}>
                                <td style={{ paddingLeft: 20 }}>{ad}</td>
                                {analiz.kiyas && <td className="num" style={{ color: "var(--mut)" }}>{o != null ? tl(o) : "—"}</td>}
                                <td className="num">{tl(v)}</td>
                                {analiz.kiyas && <td className="num" style={{ paddingRight: 20, color: dlt == null ? "var(--mut2)" : dlt >= 0 ? "var(--pos)" : "var(--neg)" }}>{dlt == null ? "—" : (dlt >= 0 ? "+" : "") + dlt.toFixed(1) + "%"}</td>}
                              </tr>
                            );
                          })}
                        </tbody>
                      </table>
                    </div>
                  </>
                );
              })()}

              {analizAlt === "aylik" && (
                <div className="card" style={{ padding: 0, overflow: "hidden" }}>
                  <div style={{ padding: "16px 20px", borderBottom: "1px solid var(--border2)" }}>
                    <span className="card-title">Aylık performans + kur bağlamı <span style={{ fontSize: 11.5, color: "var(--mut2)" }}>· kur: örnek seri (TCMB EVDS bağlantısı planda)</span></span>
                  </div>
                  <table>
                    <thead><tr><th style={{ paddingLeft: 20 }}>Ay</th><th className="num">Net satış</th><th className="num">Net kâr</th><th className="num">Net marj</th><th className="num">Cari oran</th><th className="num">USD/TL</th><th className="num">EUR/TL</th><th className="num" style={{ paddingRight: 20 }}>USD/EUR</th></tr></thead>
                    <tbody>
                      {analiz.aylik.map((m) => {
                        const k = analiz.kur.find((x) => x.ay === m.ay);
                        return (
                          <tr key={m.ay}>
                            <td style={{ paddingLeft: 20, fontWeight: 600 }}>{AYAD[m.ay]}</td>
                            <td className="num">{tl(m.net_satis)}</td>
                            <td className="num" style={{ color: m.net_kar < 0 ? "var(--neg)" : "var(--pos)", fontWeight: 600 }}>{tl(m.net_kar)}</td>
                            <td className="num">{m.net_satis > 0 ? ((100 * m.net_kar) / m.net_satis).toFixed(1) + "%" : "—"}</td>
                            <td className="num">{m.cari_oran.toFixed(2)}</td>
                            <td className="num">{k?.usd.toFixed(2) ?? "—"}</td>
                            <td className="num">{k?.eur.toFixed(2) ?? "—"}</td>
                            <td className="num" style={{ paddingRight: 20 }}>{k?.usd_eur.toFixed(4) ?? "—"}</td>
                          </tr>
                        );
                      })}
                    </tbody>
                  </table>
                </div>
              )}

              <p style={{ fontSize: 11.5, color: "var(--mut2)", margin: 0 }}>
                Oran eşikleri finans doktrini genel kabulleri; banka görünümü kurumsal kredi tahsis pratiğine dayalı geneldir, bankaya göre değişir. Sektörel kıyas ve gerçek kur (TCMB) ileride.
              </p>
            </>
          )}

          {gorunum === "denetim" && !kagit && (
            <>
              <div className="pills" data-rehber="denetim-pills">
                {denetimSektorler.map((sk) => (
                  <button key={sk.kod} className={"pill-tab" + (denetimSektor === sk.kod ? " akt" : "")} onClick={() => setDenetimSektor(sk.kod)}>{sk.ad}</button>
                ))}
              </div>
              <div className="card" style={{ padding: 0, overflow: "hidden" }}>
                <div style={{ padding: "14px 20px", borderBottom: "1px solid var(--border2)" }}>
                  <span className="card-title" data-rehber="denetim-tablo">Çalışma programı — {denetimSektorler.find((x) => x.kod === denetimSektor)?.ad}</span>
                  <span style={{ fontSize: 11.5, color: "var(--mut2)", marginLeft: 10 }}>satıra tıkla → çalışma kağıdı üretilir (ilgili hesaplar defterden tespit edilir)</span>
                </div>
                <table>
                  <thead><tr><th style={{ paddingLeft: 20 }}>No</th><th>Çalışma</th><th>Dayanak</th><th>Motor</th><th>Ciddiyet</th><th style={{ paddingRight: 20 }}>Hesaplar</th></tr></thead>
                  <tbody>
                    {denetimCalismalar(denetimSektor).map((cl) => (
                      <tr key={cl.id} style={{ cursor: "pointer" }} onClick={() => kagitAc(cl.id)}>
                        <td style={{ paddingLeft: 20 }} className="mono">{cl.id}</td>
                        <td style={{ fontWeight: 600 }}>{cl.ad}</td>
                        <td>{cl.bds}</td>
                        <td className="mono">{cl.motor}</td>
                        <td><span className={"pill " + (cl.siddet === "yuksek" ? "neg" : cl.siddet === "orta" ? "warn" : "")}>{cl.siddet}</span></td>
                        <td style={{ paddingRight: 20 }} className="mono">{cl.hesaplar.join(" ")}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </>
          )}

          {gorunum === "denetim" && kagit && (
            <>
              <div style={{ display: "flex", gap: 10, alignItems: "center", flexWrap: "wrap" }}>
                <button className="pill-tab" onClick={() => setKagit(null)}>← Çalışma programı</button>
                <button className="pill-tab" onClick={kagitTxt}>TXT indir</button>
                <span style={{ flex: 1 }} />
                <button className="pill-tab" onClick={() => setKagitKilit(!kagitKilit)} title="Kağıt varsayılan kilitlidir; müdahale için bilinçli olarak açılır">
                  {kagitKilit ? "🔒 Kilitli — düzenlemeyi aç" : "🔓 Düzenleme açık"}
                </button>
                {!kagitKilit && <button className="pill-tab" onClick={() => setKagitManuel([...kagitManuel, { kod: "", ad: "", borc: 0, alacak: 0, not: "" }])}>+ Satır ekle</button>}
                {!kagitKilit && <button className="btn-dark" style={{ borderRadius: 9 }} onClick={kagitDuzenleKaydet}>Kaydet ve kilitle</button>}
              </div>
              <div className="card" style={{ padding: 20 }}>
                <div style={{ display: "flex", justifyContent: "space-between", flexWrap: "wrap", gap: 8 }}>
                  <div>
                    <div style={{ fontWeight: 700, fontSize: 15 }}>ÇALIŞMA KAĞIDI · {kagit.ref_no}</div>
                    <div style={{ fontSize: 13, marginTop: 4 }}>{kagit.ad} <span style={{ color: "var(--mut)" }}>— {kagit.sektor}</span></div>
                  </div>
                  <div style={{ fontSize: 12, color: "var(--mut)", textAlign: "right" }}>
                    <div>Dayanak: <b>{kagit.standart}</b> · Ciddiyet: {kagit.siddet}</div>
                    <div>Dönem: {kagit.donem}</div>
                    <div>Hazırlayan: {kagit.hazirlayan} · {kagit.tarih}</div>
                  </div>
                </div>
                <div style={{ fontSize: 12.5, color: "var(--mut)", marginTop: 10, borderTop: "1px solid var(--border2)", paddingTop: 10 }}><b>Amaç (yapılan iş):</b> {kagit.amac}</div>
              </div>
              {(() => {
                const satirlar = kagitSatirlar();
                const tb = satirlar.reduce((t, r) => t + r.borc, 0), ta = satirlar.reduce((t, r) => t + r.alacak, 0);
                return (
                  <div className="card" style={{ padding: 0, overflow: "hidden" }}>
                    <div style={{ padding: "12px 20px", borderBottom: "1px solid var(--border2)", display: "flex", gap: 10, alignItems: "baseline", flexWrap: "wrap" }}>
                      <span className="card-title">İlgili hesaplar <span style={{ fontSize: 11.5, color: "var(--mut2)" }}>· önek taraması: {kagit.onekler.join(", ")}</span></span>
                      <span style={{ fontSize: 11.5, color: "var(--mut2)" }}>Sarı hücre = manuel düzeltme (sistem değeri saklanır) · Mavi satır = manuel eklenen · Müdahale deftere işlemez</span>
                    </div>
                    <div style={{ overflowX: "auto" }}>
                    <table className="xgrid">
                      <thead>
                        <tr><th className="rn">#</th><th style={{ width: 90 }}>A · Kod</th><th>B · Hesap adı</th><th className="num" style={{ width: 150 }}>C · Borç</th><th className="num" style={{ width: 150 }}>D · Alacak</th><th className="num" style={{ width: 140 }}>E · Bakiye</th><th className="num" style={{ width: 80 }}>F · Hareket</th><th style={{ width: 190 }}>G · Not</th>{!kagitKilit && <th className="rn"></th>}</tr>
                      </thead>
                      <tbody>
                        {satirlar.map((r, ri) => [
                          <tr key={(r.manuel ? "m" : "s") + ri} className={r.manuel ? "man" : ""}>
                            <td className="rn">{ri + 1}</td>
                            <td className="mono" style={!r.manuel ? { cursor: "pointer", whiteSpace: "nowrap" } : undefined} onClick={() => !r.manuel && kodTikla(r.kod)} title={!r.manuel ? "Kayıtlara in: bakiyeyi oluşturan fişler + dayanaklar" : undefined}>
                              {!r.manuel && <span style={{ color: "var(--mut2)" }}>{kagitDetayKod === r.kod ? "▾ " : "▸ "}</span>}
                              {r.manuel && !kagitKilit ? <input value={r.kod} onChange={(e) => manuelGuncelle(r.mi, "kod", e.target.value)} placeholder="kod" /> : r.kod}
                              {r.manuel && kagitKilit && <span className="pill warn" style={{ marginLeft: 6, fontSize: 10 }}>M</span>}
                            </td>
                            <td>{r.manuel && !kagitKilit ? <input value={r.ad} onChange={(e) => manuelGuncelle(r.mi, "ad", e.target.value)} placeholder="satır adı (ör. düzeltme kalemi)" /> : r.ad}</td>
                            <td className={"num" + (r.ovB ? " ov" : "") + (!kagitKilit ? " edit" : "")}>
                              {kagitKilit ? tl(r.borc) : <input type="number" step="0.01" value={r.borc / 100} onChange={(e) => paraGir(r, "borc", e.target.value)} />}
                              {r.ovB && <span className="xg-sys">sistem: {tl(sistemDeger(r.kod, "borc"))} {!kagitKilit && <button className="xg-geri" onClick={() => hucreGeriAl(r.kod, "borc")} title="Sistem değerine dön">↺</button>}</span>}
                            </td>
                            <td className={"num" + (r.ovA ? " ov" : "") + (!kagitKilit ? " edit" : "")}>
                              {kagitKilit ? tl(r.alacak) : <input type="number" step="0.01" value={r.alacak / 100} onChange={(e) => paraGir(r, "alacak", e.target.value)} />}
                              {r.ovA && <span className="xg-sys">sistem: {tl(sistemDeger(r.kod, "alacak"))} {!kagitKilit && <button className="xg-geri" onClick={() => hucreGeriAl(r.kod, "alacak")} title="Sistem değerine dön">↺</button>}</span>}
                            </td>
                            <td className="num" style={{ fontWeight: 600 }}>{tl(r.borc - r.alacak)}</td>
                            <td className="num">{r.hareket === null ? "—" : r.hareket.toLocaleString("tr-TR")}</td>
                            <td className={!kagitKilit ? "edit" : ""}>
                              {kagitKilit ? <span style={{ fontSize: 12, color: "var(--mut)" }}>{r.not}</span> : <input value={r.not} placeholder="hücre notu…" onChange={(e) => (r.manuel ? manuelGuncelle(r.mi, "not", e.target.value) : setKagitHucre({ ...kagitHucre, [r.kod + ":not"]: e.target.value }))} />}
                            </td>
                            {!kagitKilit && <td className="rn">{r.manuel && <button className="xg-geri" onClick={() => setKagitManuel(kagitManuel.filter((_, j) => j !== r.mi))} title="Manuel satırı sil">×</button>}</td>}
                          </tr>,
                          !r.manuel && kagitDetayKod === r.kod ? (
                            <tr key={"detay" + ri}>
                              <td colSpan={kagitKilit ? 8 : 9} style={{ padding: 0, background: "#FAFBFC" }}>
                                <div style={{ padding: "10px 16px" }}>
                                  <div style={{ fontSize: 11.5, color: "var(--mut)", marginBottom: 6 }}>
                                    <b className="mono">{r.kod}</b> — bakiyeyi oluşturan kayıtlar · {kagitHarToplam.toLocaleString("tr-TR")} hareket · satıra tıkla → fiş detayı + dayanak
                                  </div>
                                  <table style={{ fontSize: 12 }}>
                                    <thead><tr><th>Tarih</th><th>Yevmiye</th><th>Fiş</th><th>Açıklama</th><th>Karşı</th><th>Belge (dayanak)</th><th className="num">Borç</th><th className="num">Alacak</th><th className="num">Yürüyen bk.</th></tr></thead>
                                    <tbody>
                                      {kagitHar.map((h, hi) => (
                                        <tr key={hi} style={{ cursor: h.fis_id != null ? "pointer" : undefined, background: kagitFis && kagitFis.id === h.fis_id ? "#EEF3F8" : undefined }} onClick={() => kagitHareketAc(h)}>
                                          <td style={{ whiteSpace: "nowrap" }}>{h.tarih}</td><td className="mono">{h.yevmiye_no ?? ""}</td><td className="mono">{h.fis_no}</td>
                                          <td>{h.aciklama}</td><td className="mono">{h.karsi}</td><td>{h.belge}</td>
                                          <td className="num">{tl(h.borc)}</td><td className="num">{tl(h.alacak)}</td><td className="num">{tl(h.yuruyen_bakiye)}</td>
                                        </tr>
                                      ))}
                                    </tbody>
                                  </table>
                                  <div style={{ display: "flex", gap: 8, marginTop: 8, alignItems: "center" }}>
                                    <button className="pill-tab" style={{ opacity: kagitHarOffset + HARLIMIT >= kagitHarToplam ? 0.4 : 1 }} disabled={kagitHarOffset + HARLIMIT >= kagitHarToplam} onClick={() => kagitHarYukle(r.kod, kagitHarOffset + HARLIMIT)}>← daha eski</button>
                                    <button className="pill-tab" style={{ opacity: kagitHarOffset === 0 ? 0.4 : 1 }} disabled={kagitHarOffset === 0} onClick={() => kagitHarYukle(r.kod, Math.max(0, kagitHarOffset - HARLIMIT))}>daha yeni →</button>
                                    <span style={{ fontSize: 11.5, color: "var(--mut2)" }}>{(kagitHarToplam - kagitHarOffset - kagitHar.length + 1).toLocaleString("tr-TR")}–{(kagitHarToplam - kagitHarOffset).toLocaleString("tr-TR")} / {kagitHarToplam.toLocaleString("tr-TR")}</span>
                                  </div>
                                  {kagitFis && (
                                    <div style={{ marginTop: 10, padding: 14, border: "1px solid var(--border3)", borderRadius: 10, background: "#fff" }}>
                                      <div style={{ display: "flex", justifyContent: "space-between", flexWrap: "wrap", gap: 8, alignItems: "center" }}>
                                        <div style={{ fontSize: 13 }}><b>{kagitFis.fis_no}</b> · {kagitFis.tip} · {kagitFis.tarih} <span style={{ color: "var(--mut)" }}>— dayanak: {kagitFis.belge_tipi ? `${kagitFis.belge_tipi} ${kagitFis.belge_no}` : kagitFis.dayanaksiz ? "DAYANAKSIZ ⚠" : "—"}</span></div>
                                        {yetkiVar("iptal") && <button className="pill-tab" style={{ color: "var(--neg)" }} onClick={() => kagitFisIptal(kagitFis.id)}>Fişi iptal et (VUK 217 düzeltme)</button>}
                                      </div>
                                      <table style={{ fontSize: 12, marginTop: 8 }}>
                                        <thead><tr><th>Hesap</th><th>Açıklama</th><th className="num">Borç</th><th className="num">Alacak</th></tr></thead>
                                        <tbody>{kagitFis.satirlar.map((sd, si) => <tr key={si}><td><span className="mono">{sd.kod}</span> {sd.ad}</td><td>{sd.aciklama}</td><td className="num">{tl(sd.borc)}</td><td className="num">{tl(sd.alacak)}</td></tr>)}</tbody>
                                      </table>
                                      <div style={{ fontSize: 11.5, color: "var(--mut2)", marginTop: 6 }}>Kesin fiş değiştirilemez — değişiklik VUK md.217 gereği fark edilme tarihli ters kayıtla yapılır; düzeltme fişinin dayanağı bu fiştir. Kağıt ve mizan iptal sonrası kendiliğinden güncellenir.</div>
                                    </div>
                                  )}
                                </div>
                              </td>
                            </tr>
                          ) : null,
                        ])}
                        {satirlar.length === 0 && <tr><td colSpan={kagitKilit ? 8 : 9} style={{ padding: 20, color: "var(--mut2)" }}>Bu öneklere uyan hareketli hesap bulunamadı.</td></tr>}
                        <tr style={{ fontWeight: 700, background: "var(--bg2)" }}>
                          <td className="rn"></td><td colSpan={2}>TOPLAM</td>
                          <td className="num">{tl(tb)}</td><td className="num">{tl(ta)}</td><td className="num">{tl(tb - ta)}</td><td></td><td></td>{!kagitKilit && <td></td>}
                        </tr>
                      </tbody>
                    </table>
                    </div>
                  </div>
                );
              })()}

              <div className="card" style={{ padding: 20 }}>
                <span className="card-title">Testler ve bulgular</span>
                {kagit.testler.map((t) => (
                  <div key={t.motor + t.ad} style={{ marginTop: 12 }}>
                    <div style={{ fontSize: 13 }}><span className="mono" style={{ background: "var(--bg2)", padding: "1px 6px", borderRadius: 4 }}>{t.motor}</span> <b>{t.ad}</b> <span style={{ color: "var(--mut)" }}>— {t.durum}</span></div>
                    {t.bulgular.length > 0 ? (
                      <ul style={{ margin: "6px 0 0 20px", fontSize: 12.5, color: "var(--neg)" }}>{t.bulgular.map((b, i) => <li key={i}>{b}</li>)}</ul>
                    ) : t.durum === "çalıştı" ? <div style={{ fontSize: 12.5, color: "var(--pos)", marginTop: 4 }}>Bulgu yok.</div> : null}
                  </div>
                ))}
              </div>
              {kagit.nitelikler.length > 0 && (
                <div className="card" style={{ padding: 20 }}>
                  <span className="card-title">Hesap nitelikleri (MSUGT resmi tanım)</span>
                  {kagit.nitelikler.map((n) => (
                    <div key={n.onek} style={{ fontSize: 12.5, marginTop: 8 }}><b className="mono">{n.onek}</b> <b>{n.ad}</b> — <span style={{ color: "var(--mut)" }}>{n.tanim}</span></div>
                  ))}
                </div>
              )}
              <div className="card" style={{ padding: 20 }}>
                <span className="card-title">Sonuç / SMMM notu</span>
                <textarea value={kagitNot} onChange={(e) => setKagitNot(e.target.value)} rows={4} style={{ width: "100%", marginTop: 10, fontFamily: "inherit", fontSize: 13 }} placeholder="Çalışmanın sonucu, yapılan ek incelemeler, dayanaklar…" />
                <div style={{ marginTop: 8, display: "flex", gap: 8, alignItems: "center" }}>
                  <button className="btn" onClick={kagitNotKaydet}>Notu kaydet</button>
                  {kagit.not_metni === kagitNot && kagitNot !== "" && <span style={{ fontSize: 12, color: "var(--pos)" }}>Kaydedildi</span>}
                </div>
              </div>
            </>
          )}

          {gorunum === "ufrs" && (
            <>
              <div className="pills">
                {([["wtb", "WTB — Dönüşüm Mizanı"], ["calismalar", "Çalışmalar"], ["defter", `Kayıt Defteri (${ufrsKayitlar.length})`]] as [string, string][]).map(([id, ad]) => (
                  <button key={id} className={"pill-tab" + (ufrsSekme === id ? " akt" : "")} onClick={() => setUfrsSekme(id)}>{ad}</button>
                ))}
              </div>

              {ufrsSekme === "wtb" && wtb && (
                <>
                  {/* Başlık bloğu — denetim kağıdı geleneği: müşteri / dönem / durum / aksiyonlar */}
                  <div className="card" style={{ padding: "12px 18px", display: "flex", gap: 18, alignItems: "center", flexWrap: "wrap" }}>
                    <div>
                      <div style={{ fontWeight: 700, fontSize: 14 }}>WORKING TRIAL BALANCE — {wtb.donem}</div>
                      <div style={{ fontSize: 11.5, color: "var(--mut)" }}>VUK (Per Books) → AJE → Düzeltilmiş → RJE → Devir (CF) → TFRS (Final) · net gösterim: borç +, alacak (parantez)</div>
                    </div>
                    <span className={"pill " + (wtb.kesin ? "warn" : "ok")}>{wtb.kesin ? "🔒 KESİN" : "AÇIK — taslak"}</span>
                    <span style={{ flex: 1 }} />
                    {!wtb.kesin && yetkiVar("kesinlestir") && <button className="pill-tab" onClick={ufrsKesinlestir}>Dönemi kesinleştir 🔒</button>}
                    {!wtb.kesin && <button className="pill-tab" onClick={ufrsDevirGetir} title="Önceki kesin dönemin AJE'lerini taşı-bırak kuralıyla getirir">⤵ Devri getir (CF)</button>}
                    <button className="primary" onClick={() => { setUkMesaj(""); setUfrsModal({ tip: "yeni" }); }} disabled={wtb.kesin}>+ Kayıt (pencere)</button>
                  </div>
                  {ufrsAksiyonMesaj && <div style={{ fontSize: 12.5 }}>{ufrsAksiyonMesaj}</div>}
                  {/* Kontrol satırı — her sütun 0'a kapanmalı (denge) */}
                  <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
                    {([["VUK", wtb.kontrol.vuk, wtb.kontrol.vuk_sifir], ["AJE", wtb.kontrol.aje, wtb.kontrol.aje_sifir], ["RJE", wtb.kontrol.rje, wtb.kontrol.rje_sifir], ["CF", wtb.kontrol.cf, wtb.kontrol.cf_sifir], ["TFRS", wtb.kontrol.tfrs, wtb.kontrol.tfrs_sifir]] as [string, number, boolean][]).map(([ad, v, ok]) => (
                      <span key={ad} className={"pill " + (ok ? "ok" : "warn")}>{ad} {ok ? "= 0 ✓" : "≠ 0: " + tl(v)}</span>
                    ))}
                    <span className="pill">Kâr köprüsü: {wtl(wtb.kar_koprusu.vuk_kar)} {wtb.kar_koprusu.aje_etkisi >= 0 ? "+" : "−"} {tl(Math.abs(wtb.kar_koprusu.aje_etkisi))} = <b style={{ marginLeft: 4 }}>{wtl(wtb.kar_koprusu.tfrs_kar)}</b></span>
                  </div>
                  {/* Değer zinciri tablosu — sınıf bölümleri + ara toplamlar; AJE/RJE/CF tutarları DELİNEBİLİR */}
                  <div className="card" style={{ padding: 0, overflow: "auto" }}>
                    <table>
                      <thead><tr><th style={{ width: 54 }}>Kod</th><th>Hesap / TFRS Kalemi</th><th className="num">VUK Bakiye</th><th className="num">AJE</th><th className="num">Düzeltilmiş</th><th className="num">RJE</th><th className="num">Devir (CF)</th><th className="num">TFRS Bakiye</th></tr></thead>
                      <tbody>
                        {Object.entries(SINIF_AD).map(([sinif, sinifAd]) => {
                          const grup = wtb.satirlar.filter((r) => r.sinif === sinif);
                          if (grup.length === 0) return null;
                          const t = (f: (r: WtbSatirT) => number) => grup.reduce((x, r) => x + f(r), 0);
                          return (
                            <Fragment key={sinif}>
                              <tr style={{ background: "var(--bg2, rgba(0,0,0,.03))" }}><td colSpan={8} style={{ fontWeight: 700, fontSize: 11.5, letterSpacing: 0.5, padding: "8px 12px" }}>{sinif !== "T" ? sinif + " — " : ""}{sinifAd}</td></tr>
                              {grup.map((r, i) => (
                                <Fragment key={sinif + i}>
                                <tr>
                                  <td>{r.tdhp ? (
                                    <button className="pill-tab" style={{ padding: "0 6px", fontSize: 11.5 }} title="alt kırılım (muavin)"
                                      onClick={() => wtbKirilim(r.kod)}>{wtbAcik[r.kod] ? "▾" : "▸"} {r.kod}</button>
                                  ) : <span className="pill warn" style={{ fontSize: 10 }}>TFRS</span>}</td>
                                  <td>{r.ad}</td>
                                  <td className="num">{wtl(r.vuk)}</td>
                                  <td className="num">{r.aje !== 0 ? (
                                    <button className="pill-tab" style={{ padding: "1px 8px", fontSize: 12 }} title={r.aje_refs.join(", ")}
                                      onClick={() => setUfrsModal({ tip: "kayitlar", baslik: `${r.kod !== "—" ? r.kod + " " : ""}${r.ad} — AJE kayıtları`, nolar: r.aje_refs })}>{wtl(r.aje)}</button>
                                  ) : "—"}</td>
                                  <td className="num" style={{ color: "var(--mut)" }}>{wtl(r.duzeltilmis)}</td>
                                  <td className="num">{r.rje !== 0 ? (
                                    <button className="pill-tab" style={{ padding: "1px 8px", fontSize: 12 }} title={r.rje_refs.join(", ")}
                                      onClick={() => setUfrsModal({ tip: "kayitlar", baslik: `${r.ad} — RJE kayıtları`, nolar: r.rje_refs })}>{wtl(r.rje)}</button>
                                  ) : "—"}</td>
                                  <td className="num">{r.cf !== 0 ? (
                                    <button className="pill-tab" style={{ padding: "1px 8px", fontSize: 12 }} title={r.cf_refs.join(", ")}
                                      onClick={() => setUfrsModal({ tip: "kayitlar", baslik: `${r.ad} — Devir (CF) kayıtları`, nolar: r.cf_refs })}>{wtl(r.cf)}</button>
                                  ) : "—"}</td>
                                  <td className="num" style={{ fontWeight: 600 }}>{wtl(r.tfrs)}</td>
                                </tr>
                                {wtbAcik[r.kod] === "yok" && (
                                  <tr style={{ background: "var(--bg2, rgba(0,0,0,.02))" }}><td /><td colSpan={7} style={{ fontSize: 11.5, color: "var(--mut2)", paddingLeft: 26 }}>alt kırılım yok (muavin açılmamış)</td></tr>
                                )}
                                {Array.isArray(wtbAcik[r.kod]) && (wtbAcik[r.kod] as { kod: string; ad: string; bakiye: number }[]).map((m, mi) => (
                                  <tr key={sinif + i + "m" + mi} style={{ background: "var(--bg2, rgba(0,0,0,.02))" }}>
                                    <td className="mono" style={{ paddingLeft: 26, fontSize: 11.5 }}>{m.kod}</td>
                                    <td style={{ fontSize: 11.5, color: "var(--mut)" }}>{m.ad}</td>
                                    <td className="num" style={{ fontSize: 11.5 }}>{wtl(m.bakiye)}</td>
                                    <td className="num" colSpan={4} style={{ color: "var(--mut2)", fontSize: 11 }}>düzeltmeler kebir düzeyinde</td>
                                    <td className="num" style={{ fontSize: 11.5, color: "var(--mut)" }}>{wtl(m.bakiye)}</td>
                                  </tr>
                                ))}
                                </Fragment>
                              ))}
                              <tr style={{ borderTop: "1.5px solid var(--border2)" }}>
                                <td /><td style={{ fontWeight: 650, fontSize: 12 }}>Ara toplam</td>
                                <td className="num" style={{ fontWeight: 650 }}>{wtl(t((r) => r.vuk))}</td>
                                <td className="num" style={{ fontWeight: 650 }}>{wtl(t((r) => r.aje))}</td>
                                <td className="num" style={{ fontWeight: 650, color: "var(--mut)" }}>{wtl(t((r) => r.duzeltilmis))}</td>
                                <td className="num" style={{ fontWeight: 650 }}>{wtl(t((r) => r.rje))}</td>
                                <td className="num" style={{ fontWeight: 650 }}>{wtl(t((r) => r.cf))}</td>
                                <td className="num" style={{ fontWeight: 700 }}>{wtl(t((r) => r.tfrs))}</td>
                              </tr>
                            </Fragment>
                          );
                        })}
                      </tbody>
                    </table>
                  </div>
                  <div style={{ fontSize: 11.5, color: "var(--mut2)" }}>Her AJE/RJE/CF tutarı delinebilir — tıklayınca kaynak kayıtlar açılır. Düzeltmeler deftere İŞLEMEZ. Devir (CF): önceki kesin dönemin AJE'leri taşı-bırak kuralıyla (P/L → 570) — firmalar kümüle ilerler, dönemler kopmaz.</div>
                </>
              )}

              {ufrsSekme === "calismalar" && ufrsKatalog && (
                <>
                  {/* Çalışma sekmeleri — yatay şerit; sidebar yok, tam genişlik kağıt */}
                  <div style={{ display: "flex", gap: 6, flexWrap: "wrap", padding: "2px 0" }}>
                    {[...ufrsKatalog.calismalar].sort((a, b) => a.sira - b.sira).map((c) => {
                      const kapali = !!c.uyuyan || c.uygun === false;
                      const akt = ufrsWs === c.id;
                      return (
                        <button key={c.id} className={"pill-tab" + (akt ? " akt" : "")} disabled={kapali}
                          title={kapali ? (c.uyuyan ? "Uyuyan çalışma" : "Bu sektörde uygulanmaz") : c.ad}
                          style={{ opacity: kapali ? 0.4 : 1, cursor: kapali ? "not-allowed" : "pointer", display: "flex", gap: 5, alignItems: "center" }}
                          onClick={() => !kapali && acUfrsWs(c.id, c.tur)}>
                          <b>{c.standart}</b> <span style={{ fontWeight: 400 }}>— {c.kisa_ad ?? c.ad}</span>
                          {!!c.kayit_sayisi && <span className="pill ok" style={{ fontSize: 9.5, padding: "0 5px" }}>{c.kayit_sayisi}</span>}
                        </button>
                      );
                    })}
                  </div>
                  {!ufrsDetay && <div className="card" style={{ padding: 28, color: "var(--mut)" }}>Yukarıdan bir çalışma seç — girdi hesapları, parametreler ve kayıt formu tam genişlikte açılır.</div>}
                  {ufrsDetay && (
                    <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
                      <div className="card" style={{ padding: "14px 18px" }}>
                        <div style={{ display: "flex", gap: 8, alignItems: "center", flexWrap: "wrap" }}>
                          <span style={{ fontWeight: 700, fontSize: 15 }}>{ufrsDetay.tanim.standart} — {ufrsDetay.tanim.kisa_ad ?? ufrsDetay.tanim.ad}</span>
                          <span className="pill" style={{ fontSize: 10.5 }}>{ufrsDetay.tanim.tur}</span>
                          <span className="pill" style={{ fontSize: 10.5 }}>devir: {ufrsDetay.tanim.devir}</span>
                          {ufrsDetay.tanim.cati && <span className="pill warn" style={{ fontSize: 10.5 }}>çatı çalışma</span>}
                        </div>
                        <div style={{ fontSize: 12.5, color: "var(--mut)", marginTop: 6, lineHeight: 1.5 }}>{ufrsDetay.tanim.anlatim}</div>
                      </div>
                      {/* Girdi + Parametre yan yana (geniş ekran) */}
                      <div style={{ display: "grid", gridTemplateColumns: ufrsDetay.tanim.hesapla ? "1fr 1fr" : "1fr", gap: 12, alignItems: "start" }}>
                        {ufrsDetay.girdi_hesaplar.length > 0 && (
                          <div className="card" style={{ padding: 0, overflow: "auto", maxHeight: 300 }}>
                            <div style={{ padding: "10px 14px 4px", fontWeight: 650, fontSize: 12.5 }}>① Girdi — hesabı seç (kayıt onun için atılacak) <button className="pill-tab" style={{ marginLeft: 8, fontSize: 11 }} onClick={() => acUfrsWs(ufrsWs)}>⟳</button></div>
                            <table>
                              <thead><tr><th>Alt hesap</th><th>Varlık / kalem — kimlik</th><th className="num">Net bakiye</th><th className="num">Ömür</th></tr></thead>
                              <tbody>
                                {ufrsDetay.girdi_hesaplar.map((h, i) => (
                                  <tr key={i} className="clickable" onClick={() => hedefSec(h.kod, h.ad)}
                                    style={ufrsHedef?.kod === h.kod ? { background: "var(--brand-bg)", boxShadow: "inset 3px 0 0 var(--brand)" } : undefined}>
                                    <td className="mono" style={h.tamamlayici_of ? { paddingLeft: 18, color: "var(--mut)" } : undefined}>{h.tamamlayici_of ? "↳ " : ""}{h.kod}</td>
                                    <td>
                                      {h.ad}
                                      {h.kimlik && <div style={{ fontSize: 10.5, color: "var(--mut2)" }}>{h.kimlik}</div>}
                                      {h.tamamlayici_of && <div style={{ fontSize: 10.5, color: "var(--mut2)" }}>tamamlayıcı → {h.tamamlayici_of}</div>}
                                    </td>
                                    <td className="num">{wtl(h.net)}</td>
                                    <td className="num" style={{ fontSize: 11, color: "var(--mut2)" }}>
                                      {h.vuk_omur ? `VUK ${h.vuk_omur}y / TFRS ${h.tfrs_omur}y` : (h.hareket ? h.hareket + " hrk" : "—")}
                                    </td>
                                  </tr>
                                ))}
                              </tbody>
                            </table>
                          </div>
                        )}
                        {/* ② Senaryo — denetçi hesap aramaz, DURUM seçer; standart kaydın şeklini + ertelenmiş vergiyi kurar */}
                        {ufrsDetay.tanim.senaryolar && ufrsDetay.tanim.senaryolar.length > 0 && (
                          <div className="card" style={{ padding: "14px 18px", display: "flex", flexDirection: "column", gap: 10 }}>
                            <div style={{ fontWeight: 650, fontSize: 12.5 }}>② Senaryo — <span style={{ fontWeight: 400, color: "var(--mut2)" }}>durumu seç, standart kaydı + ertelenmiş vergiyi kursun</span></div>
                            {senaryoHedefIster && (
                              <div style={{ display: "flex", gap: 10, alignItems: "center", padding: "8px 10px", background: "var(--brand-bg)", borderRadius: 8, flexWrap: "wrap" }}>
                                <span style={{ fontSize: 11.5, fontWeight: 650, color: "var(--brand-dark)" }}>Hedef hesap</span>
                                <div style={{ width: 300 }}>
                                  <HesapSecici hesaplar={wsHesaplar} value={ufrsHedef?.kod ?? ""} serbest
                                    oneriler={(seciliSenaryo?.hedef_onekleri ?? []).flatMap((on) => wsHesaplar.filter((h) => h.kod.length === 3 && h.kod.startsWith(on)).map((h) => ({ kod: h.kod, ad: h.ad })))}
                                    onChange={(k) => hedefSec(k, wsHesaplar.find((x) => x.kod === k)?.ad ?? "")} />
                                </div>
                                {ufrsHedef
                                  ? <span style={{ fontSize: 11.5, color: "var(--mut)" }}>→ {ufrsHedef.ad}</span>
                                  : <span style={{ fontSize: 11.5, color: "var(--warn)" }}>bu senaryo hedef ister — girdi tablosundan da tıklayabilirsin</span>}
                              </div>
                            )}
                            <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
                              {ufrsDetay.tanim.senaryolar.map((sc) => (
                                <button key={sc.kod} className="card" onClick={() => { setSenaryoKod(sc.kod); if (senaryoKod && senaryoKod !== sc.kod) { setSenaryoTutar(""); setSenaryoTutar2(""); } }}
                                  style={{ textAlign: "left", padding: "9px 12px", cursor: "pointer", border: senaryoKod === sc.kod ? "1.5px solid var(--brand)" : undefined, boxShadow: "none" }}>
                                  <div style={{ display: "flex", gap: 6, alignItems: "center" }}>
                                    <b style={{ fontSize: 13 }}>{sc.ad}</b>
                                    <span className="pill" style={{ fontSize: 9.5 }}>{sc.tur}</span>
                                    {sc.ev_kanal && sc.ev_kanal !== "yok" && <span className={"pill " + (sc.ev_kanal === "kz" ? "acc" : "warn")} style={{ fontSize: 9.5 }}>EV → {sc.ev_kanal === "kz" ? "K/Z" : "OCI"}</span>}
                                    {sc.ev_kanal === "yok" && <span className="pill" style={{ fontSize: 9.5 }}>EV yok</span>}
                                    {sc.cift_tutar && <span className="pill brand" style={{ fontSize: 9.5 }}>2 tutar</span>}
                                  </div>
                                  <div style={{ fontSize: 11, color: "var(--mut)", marginTop: 3 }}>{sc.aciklama}</div>
                                </button>
                              ))}
                            </div>
                            <div style={{ display: "flex", gap: 8, alignItems: "center", flexWrap: "wrap" }}>
                              <label style={{ fontSize: 11.5, color: "var(--mut)" }}>Tutar (TL)
                                <input className="num" style={{ width: 150, marginLeft: 6 }} placeholder="ör. 100.000" value={senaryoTutar} onChange={(e) => setSenaryoTutar(e.target.value)} /></label>
                              {seciliSenaryo?.cift_tutar && (
                                <label style={{ fontSize: 11.5, color: "var(--warn)" }} title={seciliSenaryo.ikinci_tutar_ad}>
                                  {seciliSenaryo.ikinci_tutar_ad ?? "İkinci tutar"} (TL)
                                  <input className="num" style={{ width: 150, marginLeft: 6 }} placeholder="simetri kısmı" value={senaryoTutar2} onChange={(e) => setSenaryoTutar2(e.target.value)} /></label>
                              )}
                              <label style={{ fontSize: 11.5, color: "var(--mut)" }}>KV oranı %
                                <input className="num" style={{ width: 70, marginLeft: 6 }} value={senaryoOran} onChange={(e) => setSenaryoOran(e.target.value)} /></label>
                              <button className="primary" onClick={senaryoUygula} disabled={!senaryoKod || (senaryoHedefIster && !ufrsHedef)}>Senaryoyu uygula →</button>
                              {ufrsHesapMesaj && <span style={{ fontSize: 12.5 }}>{ufrsHesapMesaj}</span>}
                            </div>
                          </div>
                        )}
                        {ufrsDetay.tanim.hesapla && ufrsDetay.tanim.parametreler && (
                          <div className="card" style={{ padding: "14px 18px", display: "flex", flexDirection: "column", gap: 10 }}>
                            <div style={{ fontWeight: 650, fontSize: 12.5 }}>② Parametreler → Hesapla</div>
                            <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(200px, 1fr))", gap: 10 }}>
                              {ufrsDetay.tanim.parametreler.map((pr) => (
                                <label key={pr.anahtar} style={{ fontSize: 11.5, color: "var(--mut)", display: "flex", flexDirection: "column", gap: 3 }} title={pr.aciklama}>
                                  {pr.ad}
                                  {pr.tip === "secim" ? (
                                    <select value={ufrsParams[pr.anahtar] ?? pr.varsayilan} onChange={(e) => setUfrsParams({ ...ufrsParams, [pr.anahtar]: e.target.value })}>
                                      <option value="evet">evet</option><option value="hayır">hayır</option>
                                    </select>
                                  ) : (
                                    <input className={pr.tip === "tutar" || pr.tip === "yuzde" ? "num" : ""} placeholder={pr.varsayilan || "—"}
                                      value={ufrsParams[pr.anahtar] ?? ""} onChange={(e) => setUfrsParams({ ...ufrsParams, [pr.anahtar]: e.target.value })} />
                                  )}
                                </label>
                              ))}
                            </div>
                            <div style={{ display: "flex", gap: 10, alignItems: "center" }}>
                              <button className="primary" onClick={ufrsHesaplaCalistir}>HESAPLA</button>
                              {ufrsHesapMesaj && <span style={{ fontSize: 12.5 }}>{ufrsHesapMesaj}</span>}
                            </div>
                          </div>
                        )}
                      </div>
                      {/* Hesap tablosu + öneri yan yana */}
                      {ufrsHesap && (
                        <div style={{ display: "grid", gridTemplateColumns: ufrsHesap.onerilen ? "1.4fr 1fr" : "1fr", gap: 12, alignItems: "start" }}>
                          <div className="card" style={{ padding: 0, overflow: "auto", maxHeight: 340 }}>
                            <div style={{ padding: "10px 14px 4px", fontWeight: 650, fontSize: 12.5 }}>③ Hesap tablosu (denetim izi)
                              <span style={{ fontWeight: 400, color: "var(--mut2)", fontSize: 11, marginLeft: 8 }}>köken: <b style={{color:"#2563eb"}}>◆ defter</b> · <b style={{color:"#16a34a"}}>◆ parametre</b> · <b style={{color:"var(--mut)"}}>◆ formül/sonuç</b></span>
                            </div>
                            {ufrsHesap.uyarilar.map((u, i) => <div key={i} style={{ padding: "0 14px 6px" }}><span className="pill warn" style={{ fontSize: 11 }}>{u}</span></div>)}
                            <table>
                              <thead><tr><th style={{ width: 20 }}></th><th>Kalem</th><th>Köken</th><th className="num">Tutar</th></tr></thead>
                              <tbody>
                                {ufrsHesap.ara_tablo.map((r, i) => {
                                  const renk = r.kaynak === "defter" ? "#2563eb" : r.kaynak === "parametre" ? "#16a34a" : "var(--mut2)";
                                  const vurgu = r.kalem.startsWith("FARK") || r.kalem.includes("TOPLAM") || r.kalem.includes("kayıt tutarı");
                                  return (
                                    <tr key={i} style={vurgu ? { fontWeight: 650, borderTop: "1.5px solid var(--border2)" } : undefined}
                                      title={r.mudahale ? "Denetçi müdahale edebilir (parametre bloğundan)" : r.kaynak === "defter" ? "Defter bakiyesi — değişmez; düzeltme başka çalışmada" : ""}>
                                      <td style={{ color: renk, fontSize: 10 }}>◆</td>
                                      <td>{r.kalem}{r.mudahale && <span className="pill" style={{ fontSize: 9, marginLeft: 6, padding: "0 5px" }}>müdahale</span>}</td>
                                      <td style={{ fontSize: 11, color: "var(--mut)" }}>{r.detay}</td>
                                      <td className="num">{r.tutar ? wtl(r.tutar) : "—"}</td>
                                    </tr>
                                  );
                                })}
                              </tbody>
                            </table>
                          </div>
                          {ufrsHesap.onerilen && (
                            <div className="card" style={{ padding: "14px 18px", border: "1.5px solid #16a34a55" }}>
                              <div style={{ display: "flex", gap: 8, alignItems: "center", flexWrap: "wrap" }}>
                                <span style={{ fontWeight: 700, fontSize: 13 }}>④ Önerilen kayıt</span>
                                <span className="pill ok" style={{ fontSize: 10.5 }}>{ufrsHesap.onerilen.tur}</span>
                              </div>
                              <div style={{ fontSize: 11.5, color: "var(--mut)", marginTop: 4 }}>{ufrsHesap.onerilen.aciklama}</div>
                              <table style={{ marginTop: 8 }}>
                                <tbody>
                                  {ufrsHesap.onerilen.satirlar.map((x, i) => (
                                    <tr key={i}>
                                      <td style={{ width: 22, color: "var(--mut2)" }}>{x.borc > 0 ? "B" : "A"}</td>
                                      <td className="mono" style={{ paddingLeft: x.borc > 0 ? 0 : 16 }}>{x.hesap}</td>
                                      <td className="num">{tl(x.borc > 0 ? x.borc : x.alacak)}</td>
                                    </tr>
                                  ))}
                                </tbody>
                              </table>
                              <div style={{ display: "flex", gap: 10, marginTop: 10 }}>
                                <button className="primary" onClick={oneriFormaAktar}>Formu doldur →</button>
                                <button className="pill-tab" onClick={() => setUfrsHesap(null)}>Yoksay</button>
                              </div>
                            </div>
                          )}
                        </div>
                      )}
                      {ufrsHesap && !ufrsHesap.onerilen && (
                        <div className="card" style={{ padding: "12px 18px", color: "var(--mut)", fontSize: 12.5 }}>Fark yok — bu dönem kayıt gerekmiyor.</div>
                      )}
                      {/* ⑤ Kayıt formu — tam genişlik */}
                      <div className="card" style={{ padding: "14px 18px", display: "flex", flexDirection: "column", gap: 10 }}>
                        <div style={{ display: "flex", gap: 10, alignItems: "center", flexWrap: "wrap" }}>
                          <span style={{ fontWeight: 650, fontSize: 13 }}>⑤ Kayıt — deftere işlemez, WTB'ye akar</span>
                          <select value={ukTur} onChange={(e) => setUkTur(e.target.value)} style={{ width: 220 }}>
                            <option value="AJE">AJE — düzeltme (kâr etkiler)</option>
                            <option value="RJE">RJE — sınıflama (tutar-nötr)</option>
                          </select>
                          <input style={{ flex: 1, minWidth: 260 }} placeholder="açıklama" value={ukAciklama} onChange={(e) => setUkAciklama(e.target.value)} />
                        </div>
                        {/* Satırlar — başlıklı tablo; çalışmanın tipik hesapları seçicinin İÇİNDE (★ grubu) */}
                        {(() => {
                          const uygun = (h: UfrsHesapAdayT) => !h.tur || h.tur === "*" || h.tur === ukTur;
                          const oneriListe = [
                            ...(ufrsDetay.tanim.kayit_hesaplari?.borc.filter(uygun) ?? []).map((h) => ({ kod: h.kod, ad: h.ad, grup: "BORÇ" })),
                            ...(ufrsDetay.tanim.kayit_hesaplari?.alacak.filter(uygun) ?? []).map((h) => ({ kod: h.kod, ad: h.ad, grup: "ALACAK" })),
                          ];
                          return (
                            <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
                              <div style={{ display: "flex", gap: 8, fontSize: 10.5, fontWeight: 650, color: "var(--mut2)", textTransform: "uppercase", letterSpacing: ".04em" }}>
                                <span style={{ flex: 1, maxWidth: 420 }}>Hesap <span style={{ fontWeight: 400, textTransform: "none" }}>— {ufrsDetay.tanim.standart} kapsamı: {wsHesaplar.length} hesap (standart başka hesabı ilgilendirmez)</span></span>
                                <span style={{ width: 150, textAlign: "right" }}>Borç (TL)</span>
                                <span style={{ width: 150, textAlign: "right" }}>Alacak (TL)</span>
                                <span style={{ width: 24 }} />
                              </div>
                              {ukSatirlar.map((s2, i) => (
                                <div key={i} style={{ display: "flex", gap: 8, alignItems: "center" }}>
                                  <div style={{ flex: 1, maxWidth: 420, position: "relative" }}>
                                    <HesapSecici hesaplar={wsHesaplar} value={s2.hesap} serbest oneriler={oneriListe}
                                      onChange={(k) => setUkSatirlar(ukSatirlar.map((x, j) => (j === i ? { ...x, hesap: k } : x)))} />
                                    {s2.hesap && serbestMi(s2.hesap) && <span className="pill warn" style={{ fontSize: 9, position: "absolute", right: 6, top: "50%", transform: "translateY(-50%)" }}>TFRS</span>}
                                  </div>
                                  <input style={{ width: 150 }} className="num" placeholder="0,00" value={s2.borc} onChange={(e) => setUkSatirlar(ukSatirlar.map((x, j) => (j === i ? { ...x, borc: e.target.value } : x)))} />
                                  <input style={{ width: 150 }} className="num" placeholder="0,00" value={s2.alacak} onChange={(e) => setUkSatirlar(ukSatirlar.map((x, j) => (j === i ? { ...x, alacak: e.target.value } : x)))} />
                                  <button className="xg-geri" title="satırı sil" disabled={ukSatirlar.length <= 2}
                                    style={{ width: 24, opacity: ukSatirlar.length <= 2 ? 0.3 : 1 }}
                                    onClick={() => setUkSatirlar(ukSatirlar.filter((_, j) => j !== i))}>×</button>
                                </div>
                              ))}
                            </div>
                          );
                        })()}
                        <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
                          <button className="pill-tab" onClick={() => setUkSatirlar([...ukSatirlar, { hesap: "", borc: "", alacak: "" }])}>+ satır</button>
                          {(() => {
                            const b = ukSatirlar.reduce((t, x) => t + ukTL(x.borc), 0);
                            const a = ukSatirlar.reduce((t, x) => t + ukTL(x.alacak), 0);
                            const fark = b - a;
                            if (fark === 0) return null;
                            return <button className="pill-tab" onClick={() => {
                              const i = ukSatirlar.findIndex((x) => x.hesap.trim() && !ukTL(x.borc) && !ukTL(x.alacak));
                              if (i < 0) return;
                              const val = tlGir(Math.abs(fark));
                              setUkSatirlar(ukSatirlar.map((x, j) => (j === i ? (fark > 0 ? { ...x, alacak: val } : { ...x, borc: val }) : x)));
                            }}>⇄ farkı yaz ({tl(Math.abs(fark))})</button>;
                          })()}
                        </div>
                        {/* Değerleme + dayanak yan yana (geniş ekran) */}
                        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 12 }}>
                          <div style={{ display: "flex", flexDirection: "column", gap: 8, padding: "10px 12px", border: "1px solid var(--border2)", borderRadius: 8 }}>
                            <div style={{ fontSize: 11.5, fontWeight: 650 }}>Değerleme — kaydın gerekçesi</div>
                            <select value={ukYontem} onChange={(e) => setUkYontem(e.target.value)}>
                              <option value="">— yöntem seç (ZORUNLU) —</option>
                              {(ufrsDetay.tanim.degerleme_yontemleri ?? ufrsKatalog.degerleme_yontemleri.map((y) => y.kod)).map((kod) => {
                                const y = ufrsKatalog.degerleme_yontemleri.find((x) => x.kod === kod);
                                return <option key={kod} value={kod}>{y?.ad ?? kod}</option>;
                              })}
                            </select>
                            {ukYontem && <div style={{ fontSize: 11, color: "var(--mut)" }}>{ufrsKatalog.degerleme_yontemleri.find((y) => y.kod === ukYontem)?.aciklama}</div>}
                            <textarea rows={3} placeholder="neyi baz aldı? — ZORUNLU (ör. 31.12.2026 ekspertiz raporu m² birim değeri)" value={ukBaz} onChange={(e) => setUkBaz(e.target.value)} />
                          </div>
                          <div style={{ display: "flex", flexDirection: "column", gap: 8, padding: "10px 12px", border: "1px solid var(--border2)", borderRadius: 8 }}>
                            <div style={{ fontSize: 11.5, fontWeight: 650 }}>Dayanak + denetçi notu</div>
                            <select value={ukDayanakTur} onChange={(e) => setUkDayanakTur(e.target.value)}>
                              {ufrsKatalog.dayanak_turleri.map((dt) => <option key={dt.kod} value={dt.kod}>{dt.ad}</option>)}
                            </select>
                            <input placeholder="dayanak referansı — ZORUNLU" value={ukDayanakRef} onChange={(e) => setUkDayanakRef(e.target.value)} />
                            <textarea rows={3} placeholder="denetçi notu — ZORUNLU (yapılan iş + sonuç; BDS 230)" value={ukNot} onChange={(e) => setUkNot(e.target.value)} />
                          </div>
                        </div>
                        <div style={{ display: "flex", gap: 10, alignItems: "center" }}>
                          <button className="primary" onClick={ufrsKayitGonder}>Kaydı at → WTB</button>
                          {(() => { const b = ukSatirlar.reduce((t, x) => t + ukTL(x.borc), 0); const a = ukSatirlar.reduce((t, x) => t + ukTL(x.alacak), 0); return <span className={"pill " + (b === a && b > 0 ? "ok" : "warn")}>Σ B {tl(b)} / A {tl(a)}</span>; })()}
                          {ukMesaj && <span style={{ fontSize: 12.5 }}>{ukMesaj}</span>}
                        </div>
                      </div>
                      {ufrsDetay.kayitlar.length > 0 && (
                        <div className="card" style={{ padding: 0, overflow: "auto" }}>
                          <div style={{ padding: "10px 14px 4px", fontWeight: 650, fontSize: 12.5 }}>Bu çalışmanın kayıtları</div>
                          <table>
                            <thead><tr><th>No</th><th>Açıklama</th><th>Değerleme</th><th>Dayanak</th><th className="num">Tutar</th><th>Durum</th><th></th></tr></thead>
                            <tbody>
                              {ufrsDetay.kayitlar.map((k) => (
                                <tr key={k.no} style={k.durum === "vazgecildi" ? { opacity: 0.5, textDecoration: "line-through" } : undefined}>
                                  <td className="mono">{k.no}</td><td>{k.aciklama}</td>
                                  <td style={{ fontSize: 11.5 }}>{ufrsKatalog.degerleme_yontemleri.find((y) => y.kod === k.degerleme_yontemi)?.ad ?? "—"}</td>
                                  <td style={{ fontSize: 11.5 }}>{k.dayanak_ref}</td>
                                  <td className="num">{tl(k.satirlar.reduce((t, x) => t + x.borc, 0))}</td>
                                  <td><span className={"pill " + (k.durum === "onerildi" ? "ok" : "warn")}>{k.durum}</span></td>
                                  <td>{k.durum === "onerildi" && <button className="xg-geri" onClick={() => ufrsVazgec(k.no)}>vazgeç</button>}</td>
                                </tr>
                              ))}
                            </tbody>
                          </table>
                        </div>
                      )}
                    </div>
                  )}
                </>
              )}

              {ufrsSekme === "defter" && (
                <div className="card" style={{ padding: 0, overflow: "auto" }}>
                  <table>
                    <thead><tr><th>No</th><th>Tarih</th><th>Standart</th><th>Çalışma</th><th>Açıklama</th><th>Satırlar</th><th>Değerleme</th><th>Dayanak</th><th>Denetçi notu</th><th>Hazırlayan</th><th>Durum</th></tr></thead>
                    <tbody>
                      {ufrsKayitlar.length === 0 && <tr><td colSpan={11} style={{ color: "var(--mut)", padding: 16 }}>Henüz kayıt yok — "Çalışmalar" sekmesinden dayanaklı AJE/RJE oluştur.</td></tr>}
                      {ufrsKayitlar.map((k) => (
                        <tr key={k.no} style={k.durum === "vazgecildi" ? { opacity: 0.5 } : undefined}>
                          <td style={{ whiteSpace: "nowrap" }}>{k.no}</td><td>{k.tarih}</td><td>{k.standart}</td><td style={{ fontSize: 11.5 }}>{k.kaynak_ws}</td><td>{k.aciklama}</td>
                          <td style={{ fontSize: 11.5 }}>{k.satirlar.map((s, i) => <div key={i}>{s.borc > 0 ? `B ${s.hesap} ${tl(s.borc)}` : `A ${s.hesap} ${tl(s.alacak)}`}</div>)}</td>
                          <td style={{ fontSize: 11.5, maxWidth: 180 }}>
                            {k.degerleme_yontemi ? (
                              <><b>{ufrsKatalog?.degerleme_yontemleri.find((y) => y.kod === k.degerleme_yontemi)?.ad ?? k.degerleme_yontemi}</b>
                                <div style={{ color: "var(--mut2)", fontSize: 11 }}>{k.degerleme_bazi.length > 60 ? k.degerleme_bazi.slice(0, 60) + "…" : k.degerleme_bazi}</div></>
                            ) : <span style={{ color: "var(--mut2)" }}>—</span>}
                          </td>
                          <td style={{ fontSize: 11.5 }}>{k.dayanak_tur}: {k.dayanak_ref}</td>
                          <td style={{ fontSize: 11.5, maxWidth: 220 }}>{k.denetci_notu}</td>
                          <td style={{ fontSize: 11.5 }}>{k.hazirlayan}</td>
                          <td><span className={"pill " + (k.durum === "onerildi" ? "ok" : "warn")}>{k.durum}</span></td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              )}

              {/* ═══ Açılır pencereler (modal) — kayıt detayı + yeni kayıt ═══ */}
              {ufrsModal && (
                <div onClick={() => setUfrsModal(null)}
                  style={{ position: "fixed", inset: 0, background: "rgba(16,19,23,.45)", zIndex: 60, display: "flex", alignItems: "center", justifyContent: "center", padding: 24 }}>
                  <div onClick={(e) => e.stopPropagation()} className="card"
                    style={{ width: "min(760px, 94vw)", maxHeight: "86vh", overflow: "auto", padding: 0, boxShadow: "0 18px 60px rgba(0,0,0,.35)" }}>
                    <div style={{ padding: "13px 18px", borderBottom: "1px solid var(--border2)", display: "flex", alignItems: "center", gap: 10 }}>
                      <div style={{ fontWeight: 700, fontSize: 13.5, flex: 1 }}>
                        {ufrsModal.tip === "kayitlar" ? ufrsModal.baslik : "Yeni kayıt — çalışmadan WTB'ye"}
                      </div>
                      <button className="pill-tab" onClick={() => setUfrsModal(null)}>✕ Kapat</button>
                    </div>
                    <div style={{ padding: 18, display: "flex", flexDirection: "column", gap: 12 }}>
                      {ufrsModal.tip === "kayitlar" && ufrsKayitlar.filter((k) => ufrsModal.nolar.includes(k.no)).map((k) => (
                        <div key={k.no} className="card" style={{ padding: "12px 14px", margin: 0 }}>
                          <div style={{ display: "flex", gap: 8, alignItems: "center", flexWrap: "wrap" }}>
                            <b>{k.no}</b><span className="pill" style={{ fontSize: 10.5 }}>{k.standart}</span>
                            <span className="pill" style={{ fontSize: 10.5 }}>{k.kaynak_ws}</span>
                            <span className={"pill " + (k.durum === "onerildi" ? "ok" : "warn")} style={{ fontSize: 10.5 }}>{k.durum}</span>
                            <span style={{ flex: 1 }} />
                            <span style={{ fontSize: 11.5, color: "var(--mut)" }}>{k.hazirlayan} · {k.tarih} · dönem {k.donem}</span>
                          </div>
                          <div style={{ fontSize: 12.5, marginTop: 6 }}>{k.aciklama}</div>
                          <table style={{ marginTop: 8 }}>
                            <tbody>
                              {k.satirlar.map((sat, i) => (
                                <tr key={i}>
                                  <td style={{ width: 24, color: "var(--mut2)" }}>{sat.borc > 0 ? "B" : "A"}</td>
                                  <td style={{ paddingLeft: sat.borc > 0 ? 0 : 18 }}>{sat.hesap}</td>
                                  <td className="num">{tl(sat.borc > 0 ? sat.borc : sat.alacak)}</td>
                                </tr>
                              ))}
                            </tbody>
                          </table>
                          {k.degerleme_yontemi && (
                            <div style={{ fontSize: 11.5, marginTop: 8, padding: "8px 10px", background: "var(--bg2, rgba(0,0,0,.02))", borderRadius: 6 }}>
                              <div><b>Değerleme yöntemi:</b> {ufrsKatalog?.degerleme_yontemleri.find((y) => y.kod === k.degerleme_yontemi)?.ad ?? k.degerleme_yontemi}</div>
                              <div style={{ marginTop: 3 }}><b>Baz alınan:</b> {k.degerleme_bazi}</div>
                            </div>
                          )}
                          <div style={{ fontSize: 11.5, color: "var(--mut)", marginTop: 8 }}>
                            <b>Dayanak:</b> {k.dayanak_tur} — {k.dayanak_ref}
                            {k.devir_kaynak ? <> · <b>Devir kaynağı:</b> {k.devir_kaynak}</> : null}
                          </div>
                          <div style={{ fontSize: 11.5, marginTop: 4 }}><b>Denetçi notu:</b> {k.denetci_notu}</div>
                        </div>
                      ))}
                      {ufrsModal.tip === "yeni" && ufrsKatalog && (
                        <>
                          <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
                            <select value={ufrsWs} onChange={(e) => acUfrsWs(e.target.value, ufrsKatalog.calismalar.find((c) => c.id === e.target.value)?.tur)}>
                              <option value="">— çalışma seç —</option>
                              {ufrsKatalog.calismalar.filter((c) => !c.uyuyan && c.uygun !== false).sort((a, b) => a.sira - b.sira).map((c) => (
                                <option key={c.id} value={c.id}>{c.ad} · {c.standart}</option>
                              ))}
                            </select>
                            <select value={ukTur} onChange={(e) => setUkTur(e.target.value)}>
                              <option value="AJE">AJE — düzeltme</option>
                              <option value="RJE">RJE — sınıflama</option>
                            </select>
                          </div>
                          {ufrsWs && ufrsDetay && (
                            <>
                              <div style={{ fontSize: 11.5, color: "var(--mut)" }}>{ufrsDetay.tanim.anlatim}</div>
                              <input placeholder="açıklama" value={ukAciklama} onChange={(e) => setUkAciklama(e.target.value)} />
                              {ukSatirlar.map((sat, i) => (
                                <div key={i} style={{ display: "flex", gap: 8 }}>
                                  <input style={{ flex: 1 }} placeholder="hesap kodu veya TFRS kalemi" value={sat.hesap} onChange={(e) => setUkSatirlar(ukSatirlar.map((x, j) => (j === i ? { ...x, hesap: e.target.value } : x)))} />
                                  <input style={{ width: 120 }} className="num" placeholder="borç (TL)" value={sat.borc} onChange={(e) => setUkSatirlar(ukSatirlar.map((x, j) => (j === i ? { ...x, borc: e.target.value } : x)))} />
                                  <input style={{ width: 120 }} className="num" placeholder="alacak (TL)" value={sat.alacak} onChange={(e) => setUkSatirlar(ukSatirlar.map((x, j) => (j === i ? { ...x, alacak: e.target.value } : x)))} />
                                </div>
                              ))}
                              <div><button className="pill-tab" onClick={() => setUkSatirlar([...ukSatirlar, { hesap: "", borc: "", alacak: "" }])}>+ satır</button></div>
                              <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
                                <select value={ukDayanakTur} onChange={(e) => setUkDayanakTur(e.target.value)}>
                                  {ufrsKatalog.dayanak_turleri.map((dt) => <option key={dt.kod} value={dt.kod}>{dt.ad}</option>)}
                                </select>
                                <input style={{ flex: 1, minWidth: 180 }} placeholder="dayanak referansı — ZORUNLU" value={ukDayanakRef} onChange={(e) => setUkDayanakRef(e.target.value)} />
                              </div>
                              <textarea rows={2} placeholder="denetçi notu — ZORUNLU" value={ukNot} onChange={(e) => setUkNot(e.target.value)} />
                              {/* D1 düzeltmesi: backend'in zorunlu kıldığı değerleme yöntemi + bazı bu formda da girilebilsin */}
                              <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
                                <select value={ukYontem} onChange={(e) => setUkYontem(e.target.value)}>
                                  {(ufrsDetay.tanim.degerleme_yontemleri ?? []).map((y) => (
                                    <option key={y} value={y}>{ufrsKatalog.degerleme_yontemleri.find((k) => k.kod === y)?.ad ?? y}</option>
                                  ))}
                                </select>
                                <input style={{ flex: 1, minWidth: 180 }} placeholder="değerlemenin baz aldığı veri — ZORUNLU" value={ukBaz} onChange={(e) => setUkBaz(e.target.value)} />
                              </div>
                              <div style={{ display: "flex", gap: 10, alignItems: "center" }}>
                                <button className="primary" onClick={ufrsKayitGonder}>Kaydı at → WTB</button>
                                {(() => { const bt = ukSatirlar.reduce((t2, x) => t2 + ukTL(x.borc), 0); const at = ukSatirlar.reduce((t2, x) => t2 + ukTL(x.alacak), 0); return <span className={"pill " + (bt === at && bt > 0 ? "ok" : "warn")}>Σ B {tl(bt)} / A {tl(at)}</span>; })()}
                                {ukMesaj && <span style={{ fontSize: 12.5 }}>{ukMesaj}</span>}
                              </div>
                            </>
                          )}
                        </>
                      )}
                    </div>
                  </div>
                </div>
              )}
            </>
          )}

          {gorunum === "yonetim" && kullanici.rol === "admin" && (
            <>
              <div className="card" style={{ maxWidth: 720 }}>
                <span className="card-title">Kullanıcılar</span>
                <table style={{ marginTop: 10 }}>
                  <thead><tr><th>Kullanıcı adı</th><th>Ad</th><th>Departman</th><th>Kademe</th><th>Rol</th><th>Erişebildiği mükellefler</th></tr></thead>
                  <tbody>
                    {kullanicilar.map((u) => (
                      <tr key={u.id}>
                        <td className="mono">{u.kullanici_adi}</td><td>{u.ad}</td>
                        <td style={{ fontSize: 12.5 }}>{departmanlar.find((d) => d.kod === u.departman)?.ad ?? u.departman ?? "—"}</td>
                        <td style={{ fontSize: 12.5 }}>{kademeler.find((k) => k.kod === u.kademe)?.ad ?? u.kademe ?? "—"}</td>
                        <td><span className={"pill " + (u.rol === "admin" ? "warn" : "")}>{u.rol}</span></td>
                        <td style={{ fontSize: 12.5, color: "var(--mut)" }}>{u.rol === "admin" ? "Tüm mükellefler" : (u.mukellef_idleri.map((id) => mukellefler.find((m) => m.id === id)?.unvan ?? id).join(", ") || "—")}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
              <div className="card" style={{ maxWidth: 720 }}>
                <span className="card-title">Yeni kullanıcı</span>
                <div style={{ fontSize: 12, color: "var(--mut)", marginBottom: 12 }}>Kayıt yoktur — kullanıcıları yalnız yönetici (admin) buradan açar ve mükellef erişimini atar.</div>
                <div className="row">
                  <div><label>Ad soyad</label><input value={ykAd} onChange={(e) => setYkAd(e.target.value)} placeholder="Ayşe Yılmaz" /></div>
                  <div><label>Kullanıcı adı</label><input value={ykKa} onChange={(e) => setYkKa(e.target.value)} placeholder="ayse" /></div>
                  <div><label>Parola</label><input type="password" value={ykSifre} onChange={(e) => setYkSifre(e.target.value)} placeholder="••••••" /></div>
                  <div><label>Rol</label><select value={ykRol} onChange={(e) => setYkRol(e.target.value)}><option value="kullanici">Kullanıcı</option><option value="admin">Yönetici (admin)</option></select></div>
                  <div><label>Departman <span style={{ color: "var(--mut2)" }}>(görürlük)</span></label><select value={ykDep} onChange={(e) => setYkDep(e.target.value)}>{departmanlar.map((d) => <option key={d.kod} value={d.kod}>{d.ad}</option>)}</select></div>
                  <div><label>Kademe <span style={{ color: "var(--mut2)" }}>(yetki)</span></label><select value={ykKademe} onChange={(e) => setYkKademe(e.target.value)}>{kademeler.map((k) => <option key={k.kod} value={k.kod}>{k.ad}</option>)}</select></div>
                </div>
                {ykRol === "kullanici" && (
                  <div style={{ marginBottom: 12 }}>
                    <label>Erişebileceği mükellefler</label>
                    <div style={{ display: "flex", flexWrap: "wrap", gap: 8, marginTop: 4 }}>
                      {mukellefler.map((m) => (
                        <label key={m.id} style={{ display: "flex", alignItems: "center", gap: 6, fontSize: 12.5, border: "1px solid var(--border3)", borderRadius: 8, padding: "5px 9px", cursor: "pointer" }}>
                          <input type="checkbox" style={{ width: "auto", height: "auto" }} checked={ykMuk.includes(m.id)} onChange={(e) => setYkMuk(e.target.checked ? [...ykMuk, m.id] : ykMuk.filter((x) => x !== m.id))} />
                          {m.unvan}
                        </label>
                      ))}
                    </div>
                  </div>
                )}
                {ykMesaj && <div className="msg" style={{ background: ykMesaj.ok ? "var(--pos-bg)" : "var(--neg-bg)", color: ykMesaj.ok ? "var(--pos)" : "var(--neg)" }}>{ykMesaj.t}</div>}
                <button className="btn-dark" style={{ borderRadius: 9 }} disabled={!ykKa.trim() || !ykSifre} onClick={kullaniciEkle}>+ Kullanıcı oluştur</button>
              </div>
            </>
          )}

          {gorunum !== "yonetim" && !GERCEK.includes(gorunum) && (
            <div className="yakinda">
              <div style={{ fontSize: 15, fontWeight: 600, marginBottom: 6, color: "var(--ink)" }}>{aktifAd(gorunum)}</div>
              Bu modül standart analizinden türetildi — geliştirme sırasında. bkz. docs/analiz/03-frontend-todo.md
            </div>
          )}
        </div>
      </main>

      {seciliFis && (
        <div className="modal-bg" onClick={() => setSeciliFis(null)}>
          <div className="modal" onClick={(e) => e.stopPropagation()}>
            <div className="modal-hd">
              <div>
                <div style={{ fontSize: 16, fontWeight: 700 }}>{seciliFis.fis_no} · {seciliFis.tip}</div>
                <div style={{ fontSize: 12.5, color: "var(--mut)", marginTop: 3 }}>
                  {seciliFis.tarih}
                  {seciliFis.belge_tipi && <> · {seciliFis.belge_tipi} {seciliFis.belge_no}</>}
                  {seciliFis.dayanaksiz && <span className="pill warn" style={{ marginLeft: 8 }}>dayanaksız</span>}
                </div>
              </div>
              <div style={{ display: "flex", gap: 8 }}>
                {yetkiVar("iptal") && <button className="btn" style={{ color: "var(--neg)" }} onClick={() => fisIptal(seciliFis.id)}>İptal fişi kes</button>}
                <button className="modal-x" onClick={() => setSeciliFis(null)}>✕</button>
              </div>
            </div>
            <div className="modal-body">
              <table>
                <thead><tr><th>Hesap</th><th>Açıklama</th><th className="num">Borç</th><th className="num">Alacak</th></tr></thead>
                <tbody>
                  {seciliFis.satirlar.map((s, i) => (
                    <tr key={i}>
                      <td><span className="mono">{s.kod}</span> {s.ad}</td>
                      <td style={{ color: "var(--mut)" }}>{s.aciklama || "—"}</td>
                      <td className="num" style={{ fontWeight: 600 }}>{s.borc ? tl(s.borc) : ""}</td>
                      <td className="num" style={{ fontWeight: 600 }}>{s.alacak ? tl(s.alacak) : ""}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
