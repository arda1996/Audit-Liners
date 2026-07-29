import { useEffect, useState } from "react";

const API = "http://127.0.0.1:8787";

type Aciklama = { kod: string; ad: string; tanim: string; isleyis: string; karsi: string[]; kapatma: string | null };

// Hesabın resmi MSUGT açıklaması (tanım + işleyiş + karşı bacaklar + kapatma).
export default function AciklamaPanel({ kod, onKarsi }: { kod: string; onKarsi?: (k: string) => void }) {
  const [a, setA] = useState<Aciklama | null>(null);
  const [yok, setYok] = useState(false);

  useEffect(() => {
    setA(null); setYok(false);
    fetch(`${API}/api/hesap/${kod}/aciklama`)
      .then((r) => (r.ok ? r.json() : Promise.reject()))
      .then(setA)
      .catch(() => setYok(true));
  }, [kod]);

  if (yok) return <div className="aciklama"><span style={{ color: "var(--muted)" }}>Bu hesap için açıklama bulunamadı.</span></div>;
  if (!a) return <div className="aciklama"><span style={{ color: "var(--muted)" }}>Yükleniyor…</span></div>;

  return (
    <div className="aciklama">
      <h4><span className="mono">{a.kod}</span> {a.ad}</h4>
      {a.tanim && <p>{a.tanim}</p>}
      {a.isleyis && (<><div className="lbl2">İşleyişi</div><p>{a.isleyis}</p></>)}
      {a.karsi.length > 0 && (
        <><div className="lbl2">Karşı bacaklar (tıkla)</div>
          <div>{a.karsi.map((k) => <button key={k} className="kbadge" onClick={() => onKarsi?.(k)}>{k}</button>)}</div></>
      )}
      {a.kapatma && (<><div className="lbl2">Karşılıklı kapanır</div><span className="kbadge">{a.kapatma}</span></>)}
    </div>
  );
}
