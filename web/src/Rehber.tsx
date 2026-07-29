import { useEffect, useState } from "react";

/** Spotlight öğretici rehber — ilk kullanım kılavuzu.
 *  İçerik veriden sürülür (adım ekle/değiştir = bu dosyada veri düzenle, bileşen değişmez).
 *  hedef: CSS seçici (data-rehber). Hedef yoksa/bulunamazsa kart ortada gösterilir (adım kaybolmaz).
 *  Spotlight tıklamayı ENGELLEMEZ — kullanıcı anlatılan düğmeye gerçekten basabilir (yönlendirici tur). */

export type RehberAdim = { hedef?: string; baslik: string; metin: string };

const REHBER: Record<string, RehberAdim[]> = {
  dashboard: [
    { baslik: "Audit-Liners'a hoş geldin", metin: "Bu program muhasebe kaydından vergiye ve denetime giden yolun tamamını tek çatıda toplar. Bu rehber her sayfada ne yapıldığını adım adım gösterir; sağ üstteki Rehber düğmesiyle istediğin an açıp kapatabilirsin." },
    { hedef: '[data-rehber="nav-muhasebe"]', baslik: "Muhasebe — programın kalbi", metin: "Fiş girişi, fişler, yevmiye, muavin ve mizan tek sayfada yaşar. Her şey burada atılan kayıtlardan türer: mali tablolar, vergiler, denetim kağıtları…" },
    { hedef: '[data-rehber="yeni-kayit"]', baslik: "Hızlı kayıt", metin: "Nerede olursan ol, bu düğme seni yeni fiş girişine götürür." },
    { hedef: '[data-rehber="takvim-karti"]', baslik: "Vergi takvimi", metin: "Yaklaşan beyanname günleri burada. Bir satıra tıklarsan Vergi sayfasındaki tam takvim açılır." },
    { hedef: '[data-rehber="nav-vergi"]', baslik: "Sıradaki durak: kayıt atmayı öğren", metin: "İleri'ye basınca Muhasebe sayfasına geçip fiş girişini adım adım gösterelim." },
  ],
  muhasebe: [
    { hedef: '[data-rehber="muh-pills"]', baslik: "Tek sayfa, beş görünüm", metin: "Kayıt → Fişler → Yevmiye → Muavin → Mizan. Soldan sağa muhasebenin doğal akışı: önce kayıt atılır, defterler bu kayıtlardan kendiliğinden oluşur — ayrıca bir şey 'kaydetmeye' gerek yok." },
    { hedef: '[data-rehber="sablon"]', baslik: "Şablondan başla", metin: "En sık kayıtlar (peşin satış, mal alışı, KDV tahakkuku…) hazır şablon. Seç, tutarları düzelt, kesinleştir." },
    { hedef: '[data-rehber="fis-form"]', baslik: "Fiş = dengeli çift taraf", metin: "Her satır bir hesabı borçlandırır ya da alacaklandırır; toplam borç = toplam alacak olmadan fiş kesinleşmez. Belge tipi ve numarası dayanaktır — aynı belge ikinci kez girilirse sistem reddeder. Hesap kodu seçince açıklamalar o hesaba göre önerilir." },
    { hedef: '[data-rehber="kesinlestir"]', baslik: "Kesinleştir = mühür", metin: "Kesinleşen fiş ASLA değiştirilemez (defterin değişmezliği). Hata olursa fişi açıp 'İptal' ile VUK 217'ye uygun ters kayıt kesersin: gerekçe ve fark edilme tarihi zorunludur, iz kalır." },
    { hedef: '[data-rehber="muh-pills"]', baslik: "Muavin ve Mizan", metin: "Muavin: bir hesabın tüm hareket dökümü (alt kırılımlarıyla). Mizan: tüm hesapların borç/alacak ve bakiye özeti; satırdan kebire inebilirsin. Denetimde her rakamın kaynağına buradan ulaşılır." },
  ],
  banka: [
    { baslik: "Banka ekstresi", metin: "Ekstre satırlarını buraya yüklersin; sistem her satır için defterdeki kayıt adaylarını önerir (aynı tutar, yakın tarih). Eşleştirdiğin satırlar kayıtlarla bağlanır — banka mutabakatının temeli." },
  ],
  belgeler: [
    { baslik: "Gelen belge kutusu", metin: "e-Fatura/e-Arşiv gibi belgeler burada toplanır ve ilgili fişe bağlanır. Böylece her belge 'hangi kayda dayanak?' sorusunun cevabını taşır. E-dönüşüm bağlantısı geldiğinde bu kutu otomatik dolacak." },
  ],
  vergi: [
    { hedef: '[data-rehber="vergi-pills"]', baslik: "Vergiye giden yol", metin: "KDV beyanname taslağı, geçici vergi hesap kağıdı, vergi takvimi ve kanun parametreleri — hepsi muhasebe kayıtlarından beslenir." },
    { hedef: '[data-rehber="kdv-karti"]', baslik: "KDV1 taslağı", metin: "Matrahlar KDV hesaplarının alt kırılımından oran oran gelir; her satırın altında kaynağı yazar (hangi hesap, hangi ay). Alttaki formdan manuel satır ekleyebilirsin — manuel satır deftere İŞLEMEZ, taslakta dayanağıyla birlikte izli durur." },
    { hedef: '[data-rehber="vergi-pills"]', baslik: "Geçici vergi", metin: "Geçici vergi sekmesi çeyreklik kümülatif hesabı yapar: ticari kâr + KKEG − istisnalar = matrah × oran − önceki dönem mahsubu. Oran kanun parametrelerinden okunur; Parametreler sekmesinde tüm güncel had ve oranlar listelenir." },
  ],
  analiz: [
    { hedef: '[data-rehber="analiz-pills"]', baslik: "Finansal analiz", metin: "Oranlar (banka/kredibilite yorumlu), Mali tablolar (tek sayfada karşılaştırmalı bilanço + gelir tablosu) ve Aylık & kur görünümleri." },
    { hedef: '[data-rehber="donem-secici"]', baslik: "Dönem kıyası", metin: "Raporlama dönemi seç (Q1–Q4); sistem otomatik olarak bir önceki dönemle (tx-1) karşılaştırır ve yorumlar üretir." },
    { baslik: "Mali tablolardan vergiye köprü", metin: "Gelir tablosunun başlığındaki 'Geçici vergi →' düğmesi seni aynı dönemin vergi hesabına götürür — kârdan vergiye kesintisiz iz." },
  ],
  denetim: [
    { hedef: '[data-rehber="denetim-pills"]', baslik: "Sektörel çalışma programları", metin: "Her sektör için hazır denetim çalışmaları (BDS/TMS dayanaklı). Eşikler ve hesaplar veride tanımlı — programlar sana göre uyarlanabilir." },
    { hedef: '[data-rehber="denetim-tablo"]', baslik: "Çalışma kağıdı üret", metin: "Bir çalışmaya tıkla: ilgili hesaplar defterden otomatik bulunur, testler koşar, BDS 230 formatında çalışma kağıdı hazırlanır." },
    { baslik: "Kağıt üstünde çalış, kayda in", metin: "Kağıt kilitli açılır; 'düzenlemeyi aç' ile Excel gibi müdahale edersin (sistem değeri saklanır, sarı iz). Hesap koduna tıklarsan o bakiyeyi oluşturan fişlere ve belgelere inersin — gerekirse oradan VUK 217 düzeltmesi kesersin." },
  ],
  hesaplar: [
    { baslik: "Hesap planı", metin: "Tek Düzen Hesap Planı'nın tamamı; her hesabın resmi tanımı ve işleyişi (MSUGT). Dilediğin hesabın altına sınırsız derinlikte alt hesap (muavin) açabilirsin — 120.01.001 gibi." },
  ],
};

/** Tur sırası — bir sayfanın adımları bitince sıradaki önerilir (yönlendirici akış). */
const SIRA = ["dashboard", "muhasebe", "banka", "belgeler", "vergi", "analiz", "denetim", "hesaplar"];

export default function Rehber({ gorunum, acik, kapat, git }: {
  gorunum: string;
  acik: boolean;
  kapat: () => void;
  git: (g: string) => void;
}) {
  const [adim, setAdim] = useState(0);
  const [rect, setRect] = useState<DOMRect | null>(null);
  const adimlar = REHBER[gorunum] ?? [];
  // Sayfa değişiminde setAdim(0) effect'i koşmadan ÖNCEKİ render'da eski adım yeni listeyi aşabilir
  // (5 adımlı sayfadan 1 adımlıya geçiş) — indeks HER render'da listeye kıstırılır (beyaz ekran fix'i).
  const adimIdx = Math.min(adim, adimlar.length - 1);

  useEffect(() => { setAdim(0); }, [gorunum, acik]);

  useEffect(() => {
    if (!acik) return;
    const a = adimlar[adimIdx];
    if (!a?.hedef) { setRect(null); return; }
    const el = document.querySelector(a.hedef) as HTMLElement | null;
    if (!el) { setRect(null); return; }
    el.scrollIntoView({ block: "center", behavior: "smooth" });
    const olc = () => setRect(el.getBoundingClientRect());
    const t = setTimeout(olc, 260);
    window.addEventListener("resize", olc);
    return () => { clearTimeout(t); window.removeEventListener("resize", olc); };
  }, [acik, adimIdx, gorunum, adimlar]);

  const son = adimIdx === adimlar.length - 1;
  const siraIdx = SIRA.indexOf(gorunum);
  const siradaki = siraIdx >= 0 && siraIdx < SIRA.length - 1 ? SIRA[siraIdx + 1] : null;
  const ileriGit = () => {
    if (!son) setAdim(adimIdx + 1);
    else if (siradaki) git(siradaki);
    else kapat();
  };

  // Kısayollar: Space/→/Enter = ileri · ← = geri · Esc = sonlandır.
  // Tur etkileşimi engellemediğinden form alanında yazarken kısayollar DEVRE DIŞI kalır.
  useEffect(() => {
    if (!acik) return;
    const dinle = (e: KeyboardEvent) => {
      const t = document.activeElement?.tagName;
      if (t === "INPUT" || t === "TEXTAREA" || t === "SELECT") return;
      if (e.key === " " || e.key === "ArrowRight" || e.key === "Enter") {
        e.preventDefault();
        ileriGit();
      } else if (e.key === "ArrowLeft") {
        e.preventDefault();
        if (adimIdx > 0) setAdim(adimIdx - 1);
      } else if (e.key === "Escape") {
        e.preventDefault();
        kapat();
      }
    };
    window.addEventListener("keydown", dinle);
    return () => window.removeEventListener("keydown", dinle);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [acik, adim, gorunum, adimlar.length]);

  const a = adimlar[adimIdx];
  if (!acik || !a) return null;

  // Kart konumu: hedef varsa altında/üstünde, yoksa ekran ortası
  const kartStil: React.CSSProperties = rect
    ? (rect.bottom + 190 < window.innerHeight
        ? { top: rect.bottom + 12, left: Math.min(Math.max(rect.left, 12), window.innerWidth - 372) }
        : { top: Math.max(rect.top - 190, 12), left: Math.min(Math.max(rect.left, 12), window.innerWidth - 372) })
    : { top: "34%", left: "50%", transform: "translateX(-50%)" };

  return (
    <>
      {rect ? (
        <div className="rehber-spot" style={{ top: rect.top - 6, left: rect.left - 6, width: rect.width + 12, height: rect.height + 12 }} />
      ) : (
        <div className="rehber-karart" />
      )}
      <div className="rehber-kart" style={kartStil} role="dialog" aria-label="Öğretici rehber">
        <div className="rehber-ust">
          <span className="rehber-sayac">{adimIdx + 1} / {adimlar.length}</span>
          <button className="rehber-x" onClick={kapat} title="Rehberi kapat (sağ üstten yeniden açabilirsin)">✕</button>
        </div>
        <div className="rehber-baslik">{a.baslik}</div>
        <div className="rehber-metin">{a.metin}</div>
        <div className="rehber-alt">
          <button className="btn" style={{ color: "var(--neg)" }} title="Öğreticiyi sonlandır (Esc)" onClick={kapat}>Sonlandır</button>
          <span style={{ flex: 1 }} />
          <button className="btn" disabled={adimIdx === 0} style={{ opacity: adimIdx === 0 ? 0.4 : 1 }} onClick={() => setAdim(adimIdx - 1)}>← Geri</button>
          {!son && <button className="btn-dark" style={{ borderRadius: 9 }} onClick={ileriGit}>İleri →</button>}
          {son && siradaki && <button className="btn-dark" style={{ borderRadius: 9 }} onClick={ileriGit}>Sıradaki sayfa →</button>}
          {son && !siradaki && <button className="btn-dark" style={{ borderRadius: 9 }} onClick={kapat}>Turu bitir ✓</button>}
        </div>
        <div className="rehber-kisayol"><kbd>␣</kbd> / <kbd>→</kbd> ileri · <kbd>←</kbd> geri · <kbd>Esc</kbd> sonlandır</div>
      </div>
    </>
  );
}
