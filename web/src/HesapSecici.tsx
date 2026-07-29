import { useEffect, useMemo, useRef, useState } from "react";

type Hesap = { kod: string; ad: string };
export type OneriHesap = { kod: string; ad: string; grup?: string }; // grup: "BORÇ" | "ALACAK" vb.

// Aranabilir hesap seçici — 262 hesabı düz dropdown yerine yazarak filtrele + klavye ile seç.
// oneriler: çalışmaya özgü tipik hesaplar — arama boşken listenin ÜSTÜNDE sabitlenir.
export default function HesapSecici({
  hesaplar, value, onChange, onYeniAlt, serbest, oneriler,
}: { hesaplar: Hesap[]; value: string; onChange: (kod: string) => void; onYeniAlt?: (anaKod: string) => void; serbest?: boolean; oneriler?: OneriHesap[] }) {
  const [open, setOpen] = useState(false);
  const [q, setQ] = useState("");
  const [aktif, setAktif] = useState(0); // klavye ile vurgulanan satır
  const listeRef = useRef<HTMLDivElement>(null);
  const secili = hesaplar.find((h) => h.kod === value);
  const serbestDeger = !secili && value ? value : "";

  // Kayıt DETAYA atılır: sınıf (1 hane) ve grup (2 hane) seçilemez — kebir (3 hane) + alt
  // kırılımlar listelenir. Kebir bakiyesi alt kırılımdan toplanır, üst hesaplar WTB'de sumlanır.
  const secilebilir = useMemo(() => hesaplar.filter((h) => h.kod.length >= 3), [hesaplar]);
  const liste = useMemo(() => {
    const s = q.trim().toLocaleLowerCase("tr");
    return s
      ? secilebilir.filter((h) => h.kod.includes(s) || h.ad.toLocaleLowerCase("tr").includes(s))
      : secilebilir;
  }, [q, secilebilir]);

  useEffect(() => { setAktif(0); }, [q]);
  // Vurgulanan satırı görünür tut
  useEffect(() => {
    if (!open) return;
    listeRef.current?.querySelector<HTMLElement>(`[data-i="${aktif}"]`)?.scrollIntoView({ block: "nearest" });
  }, [aktif, open]);

  const kapat = () => { setOpen(false); setQ(""); setAktif(0); };
  const sec = (kod: string) => { onChange(kod); kapat(); };

  // ↑/↓ gez · Enter seç · Esc kapat — 261 hesapta fare zorunluluğunu kaldırır.
  const tus = (e: React.KeyboardEvent) => {
    if (e.key === "ArrowDown") { e.preventDefault(); setAktif((i) => Math.min(i + 1, liste.length - 1)); }
    else if (e.key === "ArrowUp") { e.preventDefault(); setAktif((i) => Math.max(i - 1, 0)); }
    else if (e.key === "Enter") { e.preventDefault(); if (liste[aktif]) sec(liste[aktif].kod); else if (serbest && q.trim()) sec(q.trim()); }
    else if (e.key === "Escape") { e.preventDefault(); kapat(); }
  };

  return (
    <div className="combo">
      <button type="button" className="combo-btn" onClick={() => setOpen(true)}>
        {secili
          ? <span><b className="mono">{secili.kod}</b> {secili.ad}</span>
          : serbestDeger
            ? <span>{serbestDeger} <span style={{ fontSize: 10, color: "var(--mut2)" }}>· serbest</span></span>
            : <span style={{ color: "var(--mut2)" }}>Hesap seç…</span>}
      </button>
      {open && (
        <>
          <div className="backdrop" onClick={kapat} />
          <div className="combo-pop">
            <input autoFocus placeholder="Kod veya ad ara… (↑↓ gez, Enter seç)" value={q}
              onChange={(e) => setQ(e.target.value)} onKeyDown={tus} style={{ width: "100%", marginBottom: 6 }} />
            <div ref={listeRef} style={{ maxHeight: 300, overflow: "auto" }}>
              {!q.trim() && oneriler && oneriler.length > 0 && (
                <>
                  <div style={{ padding: "4px 8px", fontSize: 10.5, fontWeight: 650, color: "var(--brand-dark, #521622)", background: "var(--brand-bg, #F4E9EB)", borderRadius: 2 }}>
                    ★ Bu çalışmada tipik
                  </div>
                  {oneriler.map((o) => (
                    <div key={"o-" + (o.grup ?? "") + o.kod} className="combo-opt" onClick={() => sec(o.kod)}>
                      {o.grup && <span className="pill" style={{ fontSize: 9, marginRight: 6, minWidth: 44, justifyContent: "center" }}>{o.grup}</span>}
                      <b className="mono">{o.kod}</b> {o.ad}
                    </div>
                  ))}
                  <div style={{ borderTop: "1px solid var(--border2)", margin: "4px 0" }} />
                </>
              )}
              {liste.length === 0 && !serbest && <div style={{ padding: 8, color: "var(--mut)", fontSize: 13 }}>Sonuç yok</div>}
              {serbest && q.trim() && (
                <div className="combo-opt" style={{ borderTop: "1px solid var(--border2)" }} onClick={() => sec(q.trim())}>
                  ➕ “{q.trim()}” — serbest kalem / alt kırılım olarak kullan
                </div>
              )}
              {liste.map((h, i) => (
                <div key={h.kod} data-i={i} className={"combo-opt" + (i === aktif ? " akt" : "")}
                  onMouseEnter={() => setAktif(i)}
                  onClick={() => sec(h.kod)}>
                  <b className="mono">{h.kod}</b> {h.ad}
                </div>
              ))}
            </div>
            {onYeniAlt && (
              <div className="combo-opt" style={{ borderTop: "1px solid var(--border2)", color: "var(--nav)", fontWeight: 600 }}
                onClick={() => {
                  const ana = (value || q.trim()).split(".")[0].slice(0, 3);
                  onYeniAlt(/^\d{3}$/.test(ana) ? ana : "");
                  kapat();
                }}>
                + Alt hesap ekle{value ? ` (${value.split(".")[0]} altına)` : "…"}
              </div>
            )}
          </div>
        </>
      )}
    </div>
  );
}
