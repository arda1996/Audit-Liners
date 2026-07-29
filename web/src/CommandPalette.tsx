import { useEffect, useMemo, useRef, useState } from "react";
import type { NavGrup } from "./Sidebar";

type Hesap = { kod: string; ad: string };

// ⌘K komut paleti: modül navigasyonu + hesap arama (Linear kalıbı).
export default function CommandPalette({ acik, kapat, nav, git, hesaplar, hesapSec }: {
  acik: boolean; kapat: () => void; nav: NavGrup[]; git: (id: string) => void;
  hesaplar: Hesap[]; hesapSec: (kod: string) => void;
}) {
  const [q, setQ] = useState("");
  const [sec, setSec] = useState(0);
  const ref = useRef<HTMLInputElement>(null);

  useEffect(() => { if (acik) { setQ(""); setSec(0); setTimeout(() => ref.current?.focus(), 30); } }, [acik]);

  const sonuclar = useMemo(() => {
    const s = q.trim().toLocaleLowerCase("tr");
    const moduller = nav.flatMap((g) => g.items.map((it) => ({ tip: "modul" as const, id: it.id, baslik: it.ad, alt: g.grup })));
    const mods = s ? moduller.filter((m) => m.baslik.toLocaleLowerCase("tr").includes(s)) : moduller.slice(0, 8);
    const accs = s.length >= 2
      ? hesaplar.filter((h) => h.kod.includes(s) || h.ad.toLocaleLowerCase("tr").includes(s)).slice(0, 6)
          .map((h) => ({ tip: "hesap" as const, id: h.kod, baslik: `${h.kod} ${h.ad}`, alt: "Hesap planı" }))
      : [];
    return [...mods, ...accs].slice(0, 12);
  }, [q, nav, hesaplar]);

  useEffect(() => { setSec(0); }, [sonuclar.length]);

  if (!acik) return null;

  const calistir = (r: (typeof sonuclar)[number]) => {
    if (r.tip === "modul") git(r.id); else hesapSec(r.id);
    kapat();
  };

  return (
    <div className="modal-bg" style={{ alignItems: "flex-start", paddingTop: "14vh" }} onClick={kapat}>
      <div className="palet" onClick={(e) => e.stopPropagation()}>
        <input ref={ref} placeholder="Modül veya hesap ara…  (↑↓ + Enter)" value={q}
          onChange={(e) => setQ(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "ArrowDown") { e.preventDefault(); setSec((i) => Math.min(i + 1, sonuclar.length - 1)); }
            else if (e.key === "ArrowUp") { e.preventDefault(); setSec((i) => Math.max(i - 1, 0)); }
            else if (e.key === "Enter" && sonuclar[sec]) calistir(sonuclar[sec]);
            else if (e.key === "Escape") kapat();
          }} />
        <div className="palet-liste">
          {sonuclar.length === 0 && <div className="palet-bos">Sonuç yok</div>}
          {sonuclar.map((r, i) => (
            <div key={r.tip + r.id} className={"palet-satir" + (i === sec ? " sec" : "")}
              onMouseEnter={() => setSec(i)} onClick={() => calistir(r)}>
              <span className={r.tip === "hesap" ? "mono" : ""} style={{ fontSize: 13.5 }}>{r.baslik}</span>
              <span className="palet-alt">{r.alt}</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
