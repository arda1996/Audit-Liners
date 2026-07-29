import { useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";

export type NavItem = { id: string; ad: string; faz?: string };
// grup = sidebar'da görünen TEK satır; items = üzerine gelince açılan uçan pencere.
// kok = satıra tıklayınca gidilecek asıl sayfa (yoksa satır sadece pencereyi açar).
export type NavGrup = { grup: string; kok?: string; ikon?: string; items: NavItem[] };

// Minimal stroke ikon seti (Linear tarzı, 18px)
const IKON: Record<string, string> = {
  dashboard: "M3 3h7v7H3zM14 3h7v7h-7zM3 14h7v7H3zM14 14h7v7h-7z",
  yeni: "M12 5v14M5 12h14",
  fisler: "M6 3h9l4 4v14H6zM15 3v4h4M9 12h6M9 16h6",
  yevmiye: "M4 4h16v16H4zM4 9h16M9 4v16",
  kebir: "M4 19V5a2 2 0 012-2h13v18H6a2 2 0 01-2-2zM8 7h7",
  mizan: "M12 3v18M5 7l7-4 7 4M4 21h16M7 7v10M17 7v10",
  cari: "M16 11a4 4 0 10-8 0M4 21v-1a6 6 0 0112 0v1M17 8a3 3 0 11.001-6.001M15 21v-1a5 5 0 015-5",
  ekstre: "M5 3h14v18l-3-2-2 2-2-2-2 2-2-2-3 2zM9 8h6M9 12h6",
  kdv: "M9 15L15 9M9.5 9.5h.01M14.5 14.5h.01M12 3a9 9 0 100 18 9 9 0 000-18z",
  muhtasar: "M8 3v3M16 3v3M4 8h16M4 5h16v16H4zM8 13h3",
  bordro: "M3 7h18v10H3zM7 11h.01M12 11a2 2 0 100 4 2 2 0 000-4z",
  degerleme: "M4 20L20 4M6 5a2 2 0 100 4 2 2 0 000-4zM18 15a2 2 0 100 4 2 2 0 000-4z",
  bilanco: "M4 21h16M6 21V10M12 21V4M18 21v-8",
  gelirt: "M3 17l6-6 4 4 8-8M15 7h6v6",
  hesaplar: "M3 6h4v4H3zM3 14h4v4H3zM10 7h11M10 17h11M10 12h7",
  firma: "M4 21V5a2 2 0 012-2h8a2 2 0 012 2v16M8 8h2M8 12h2M14 8h2M14 12h2M20 21V11h-4M4 21h18",
  parametre: "M12 8a4 4 0 100 8 4 4 0 000-8zM12 2v2M12 20v2M2 12h2M20 12h2M5 5l1.5 1.5M17.5 17.5L19 19M19 5l-1.5 1.5M6.5 17.5L5 19",
  muhasebe: "M4 4h16v16H4zM4 9h16M9 9v11M12.5 13h4M12.5 16.5h4",
  banka: "M3 10h18L12 3zM5 10v8M9 10v8M15 10v8M19 10v8M3 21h18",
  belgeler: "M6 3h9l4 4v14H6zM15 3v4h4M9 11h6M9 15h4",
  havada: "M12 2v4M12 18v4M4.9 4.9l2.8 2.8M16.3 16.3l2.8 2.8M2 12h4M18 12h4M4.9 19.1l2.8-2.8M16.3 7.7l2.8-2.8",
  vergi: "M9 15L15 9M9.5 9.5h.01M14.5 14.5h.01M12 3a9 9 0 100 18 9 9 0 000-18z",
  analiz: "M3 17l6-6 4 4 8-8M15 7h6v6",
  denetim: "M11 4a7 7 0 100 14 7 7 0 000-14zM16 16l5 5",
  ufrs: "M4 4h16v16H4zM4 10h16M10 4v16",
  yonetim: "M12 3l8 4v5c0 5-3.4 8.4-8 9-4.6-.6-8-4-8-9V7z",
  rapor: "M5 3h14v18H5zM9 8h6M9 12h6M9 16h3",
  tanimlar: "M3 6h4v4H3zM3 14h4v4H3zM10 7h11M10 17h11",
};

function Ikon({ id }: { id: string }) {
  return (
    <svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.7" strokeLinecap="round" strokeLinejoin="round" style={{ flexShrink: 0 }}>
      <path d={IKON[id] ?? "M12 12m-8 0a8 8 0 1016 0 8 8 0 10-16 0"} />
    </svg>
  );
}

export default function Sidebar({ nav, aktif, sec, fisSayisi, paletAc }: {
  nav: NavGrup[]; aktif: string; sec: (id: string) => void; fisSayisi: number; paletAc: () => void;
}) {
  const [dar, setDar] = useState(() => localStorage.getItem("sb-dar") === "1");
  useEffect(() => { localStorage.setItem("sb-dar", dar ? "1" : "0"); }, [dar]);

  // Uçan pencere (hover) — konum satırın ekrandaki yerinden hesaplanır (sidebar kırpmasın diye fixed).
  const [ucan, setUcan] = useState<{ grup: string; top: number; left: number } | null>(null);
  const kapatZ = useRef<ReturnType<typeof setTimeout> | null>(null);
  const ac = (grup: string, el: HTMLElement) => {
    if (kapatZ.current) clearTimeout(kapatZ.current);
    const r = el.getBoundingClientRect();
    setUcan({ grup, top: r.top - 6, left: r.right });
  };
  const kapatGecikmeli = () => {
    if (kapatZ.current) clearTimeout(kapatZ.current);
    kapatZ.current = setTimeout(() => setUcan(null), 140); // satır↔pencere arası boşlukta kapanmasın
  };
  useEffect(() => () => { if (kapatZ.current) clearTimeout(kapatZ.current); }, []);

  return (
    <aside className={"side" + (dar ? " dar" : "")}>
      <div className="brand">
        <img src="/logo.svg" width="34" height="34" alt="Audit-Liners" style={{ display: "block" }} />
        {!dar && <div><div className="brand-name">Audit-Liners</div><div className="brand-sub">Muhasebe & Denetim</div></div>}
        <button className="sb-toggle" title={dar ? "Genişlet" : "Daralt"} onClick={() => setDar(!dar)}>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round"><path d={dar ? "M9 6l6 6-6 6" : "M15 6l-6 6 6 6"} /></svg>
        </button>
      </div>

      <button className="sb-ara" onClick={paletAc} title="Ara (⌘K)">
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round"><circle cx="11" cy="11" r="7" /><path d="M21 21l-4-4" /></svg>
        {!dar && <><span style={{ flex: 1, textAlign: "left" }}>Ara…</span><kbd>⌘K</kbd></>}
      </button>

      <nav className="nav">
        {nav.map((g) => {
          const tekli = g.items.length === 1 && g.kok === g.items[0].id; // alt menüsü yok → düz satır
          const grupAktif = g.items.some((i) => i.id === aktif);
          return (
            <div key={g.grup}
              onMouseEnter={(e) => { if (!tekli) ac(g.grup, e.currentTarget); }}
              onMouseLeave={kapatGecikmeli}>
              <button data-rehber={"nav-" + (g.kok ?? g.grup)}
                className={"item" + (grupAktif ? " active" : "")}
                onClick={() => g.kok && sec(g.kok)}
                onFocus={(e) => { if (!tekli) ac(g.grup, e.currentTarget.parentElement as HTMLElement); }}
                title={dar ? g.grup : undefined}>
                <Ikon id={g.ikon ?? g.kok ?? g.grup} />
                {!dar && <span className="item-ad">{g.grup}</span>}
                {!dar && <span className="badge" style={{ display: "flex", alignItems: "center", gap: 6 }}>
                  {g.items.some((i) => i.id === "fisler") && fisSayisi > 0 ? fisSayisi : ""}
                  {!tekli && <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.2" strokeLinecap="round" style={{ opacity: .45 }}><path d="M9 6l6 6-6 6" /></svg>}
                </span>}
              </button>
            </div>
          );
        })}
      </nav>

      {/* Uçan pencere — üzerine gelince açılır; sidebar'ı kalabalıklaştırmaz */}
      {ucan && (() => {
        const g = nav.find((x) => x.grup === ucan.grup);
        if (!g) return null;
        // PORTAL: body'ye basılır — sidebar'ın yığılma bağlamına hapsolmaz, sayfa içi
        // açılır kutuların (.combo-pop, sticky başlık) üstünde kalır.
        return createPortal(
          <div onMouseEnter={() => { if (kapatZ.current) clearTimeout(kapatZ.current); }}
            onMouseLeave={kapatGecikmeli}
            style={{
              position: "fixed", top: Math.min(ucan.top, window.innerHeight - 40 - g.items.length * 34), left: ucan.left + 6,
              minWidth: 216, background: "var(--bg, #fff)", border: "1px solid var(--border2)", borderRadius: 12,
              boxShadow: "0 12px 36px rgba(0,0,0,.18)", padding: 6, zIndex: 1200, // sayfa içi açılır kutuların üstünde
            }}>
            <div style={{ fontSize: 10.5, fontWeight: 700, letterSpacing: ".06em", textTransform: "uppercase", color: "var(--mut2)", padding: "5px 10px 6px" }}>{g.grup}</div>
            {g.items.map((it) => (
              <button key={it.id} disabled={!!it.faz}
                onClick={() => { if (!it.faz) { sec(it.id); setUcan(null); } }}
                title={it.faz ? `${it.ad} — Yakında (sonraki faz)` : undefined}
                style={{
                  display: "flex", alignItems: "center", gap: 9, width: "100%", textAlign: "left",
                  border: "none", borderRadius: 8, padding: "7px 10px", fontSize: 13,
                  cursor: it.faz ? "default" : "pointer", opacity: it.faz ? .45 : 1,
                  background: aktif === it.id ? "var(--bg2)" : "transparent",
                  fontWeight: aktif === it.id ? 600 : 500, color: "var(--ink)",
                }}
                onMouseEnter={(e) => { if (!it.faz && aktif !== it.id) e.currentTarget.style.background = "var(--bg2)"; }}
                onMouseLeave={(e) => { if (aktif !== it.id) e.currentTarget.style.background = "transparent"; }}>
                <span style={{ color: "var(--mut)", display: "flex" }}><Ikon id={it.id} /></span>
                <span style={{ flex: 1 }}>{it.ad}</span>
                {it.id === "fisler" && fisSayisi > 0 && <span style={{ fontSize: 11, color: "var(--mut2)" }}>{fisSayisi}</span>}
                {it.faz && <span style={{ fontSize: 10.5, color: "var(--mut2)" }}>Yakında</span>}
              </button>
            ))}
          </div>,
          document.body,
        );
      })()}

      <div className="user">
        <div className="user-in" title={dar ? "SMMM Kullanıcı" : undefined}>
          <div className="user-av">SK</div>
          {!dar && <div style={{ lineHeight: 1.25 }}><div style={{ fontSize: 12.5, fontWeight: 600 }}>SMMM Kullanıcı</div><div style={{ fontSize: 11, color: "var(--mut)" }}>Mali Müşavir</div></div>}
        </div>
      </div>
    </aside>
  );
}
