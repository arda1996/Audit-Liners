import { useEffect, useRef, useState } from "react";

const API = "http://127.0.0.1:8787";
type Kume = { standart: string[]; kullanici: string[] };
const cache = new Map<string, Kume>();

// Açıklama = hesap kodunun BAĞIMLI değişkeni: kod değişince otomatik o hesabın
// varsayılan açıklaması yazılır. Kullanıcı elle değiştirebilir, kümeden seçebilir,
// yeni metnini kümeye ekleyebilir (esneklik korunur).
export default function AciklamaSecici({ hesapKod, value, onChange }: {
  hesapKod: string; value: string; onChange: (v: string) => void;
}) {
  const [acik, setAcik] = useState(false);
  const [kume, setKume] = useState<Kume>({ standart: [], kullanici: [] });
  // Satır dolu (önceden kod+açıklama var) mount olduysa ilk auto-fill atlanır.
  const oncekiKod = useRef<string | null>(null);

  useEffect(() => {
    if (!hesapKod) { setKume({ standart: [], kullanici: [] }); oncekiKod.current = hesapKod; return; }
    const degisti = oncekiKod.current !== null && oncekiKod.current !== hesapKod;
    const ilkVeBos = oncekiKod.current === null && !value.trim();
    oncekiKod.current = hesapKod;

    const uygula = (k: Kume) => {
      setKume(k);
      // BAĞIMLILIK: kod değişti (veya boş satırda ilk seçim) → varsayılan açıklama otomatik yazılır
      if (degisti || ilkVeBos) onChange(k.kullanici[0] ?? k.standart[0] ?? "");
    };
    const c = cache.get(hesapKod);
    if (c) { uygula(c); return; }
    fetch(`${API}/api/hesap/${hesapKod}/kume`).then((r) => r.json())
      .then((k: Kume) => { cache.set(hesapKod, k); uygula(k); })
      .catch(() => setKume({ standart: [], kullanici: [] }));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [hesapKod]);

  const s = value.trim().toLocaleLowerCase("tr");
  const filtre = (list: string[]) => (s ? list.filter((m) => m.toLocaleLowerCase("tr").includes(s)) : list);
  const std = filtre(kume.standart);
  const usr = filtre(kume.kullanici);
  const tam = [...kume.standart, ...kume.kullanici].some((m) => m.toLocaleLowerCase("tr") === s);
  const ekleGoster = value.trim().length >= 3 && !tam && hesapKod;

  const kumeyeEkle = async () => {
    const r = await fetch(`${API}/api/kume`, { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify({ kod: hesapKod, metin: value.trim() }) })
      .then((x) => x.json()).catch(() => null);
    if (r) { cache.set(hesapKod, r); setKume(r); }
    setAcik(false);
  };

  return (
    <div className="combo">
      <input value={value} placeholder={hesapKod ? "Açıklama" : "Önce hesap seçin"}
        onChange={(e) => { onChange(e.target.value); setAcik(true); }}
        onFocus={() => setAcik(true)}
        onBlur={() => setTimeout(() => setAcik(false), 150)} />
      {acik && hesapKod && (std.length + usr.length > 0 || ekleGoster) && (
        <div className="combo-pop" style={{ minWidth: 260 }}>
          <div style={{ maxHeight: 200, overflow: "auto" }}>
            {std.map((m) => (
              <div key={"s" + m} className="combo-opt" onMouseDown={() => { onChange(m); setAcik(false); }}>{m}</div>
            ))}
            {usr.map((m) => (
              <div key={"u" + m} className="combo-opt" onMouseDown={() => { onChange(m); setAcik(false); }}>
                {m} <span className="palet-alt" style={{ float: "right" }}>sizin</span>
              </div>
            ))}
          </div>
          {ekleGoster && (
            <div className="combo-opt" style={{ borderTop: "1px solid var(--border2)", color: "var(--nav)", fontWeight: 600 }}
              onMouseDown={kumeyeEkle}>
              + "{value.trim().slice(0, 40)}" kümeye ekle ({hesapKod.split(".")[0]})
            </div>
          )}
        </div>
      )}
    </div>
  );
}
