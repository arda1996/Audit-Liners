// İz-Sürme (provenance) modalı — bir tutara tıklayınca tüm zincirini gösterir:
// BANKA → TAHSİS → FATURA → TAHAKKUK FİŞİ → DAYANAK. Her düğüm tıklanabilir; bileşenlerine (yukarı:
// fatura kalemleri) ve beslediği kayıtlara (aşağı: fiş satırları) ulaşılır. Denetçinin "ne/nereden/nasıl"
// sorularına cevap. İZOLE bileşen; API uçları /api/belge/oys/iz/:anahtar.
import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";

const API = "http://127.0.0.1:8787";

type Hedef = { rota: string; id: string | null; url: string | null };
type Dugum = {
  id: string; tip: string; etiket: string; tutar: number | null; durum: string; kaynak: string;
  alanlar: Record<string, unknown>;
  satirlar: { hesap: string; ad?: string; borc: number; alacak: number }[] | null;
  kalemler: { ad: string; miktar: number; birim: string; tutar: number }[] | null;
  hedef: Hedef | null;
};
type Iz = {
  iz_id: string;
  guven: { skor: number; durum: string; gerekce: string };
  ozet: { sol: { etiket: string; deger: string; tutar: number | null }; sag: { etiket: string; deger: string } | null; yon: string };
  dugumler: Dugum[];
  kenarlar: { kaynak: string; hedef: string; etiket: string }[];
};

const tl = (k: number) => (k / 100).toLocaleString("tr-TR", { minimumFractionDigits: 2, maximumFractionDigits: 2 });
const TIP_SIRA: Record<string, number> = { BANKA_HAREKETI: 0, TAHSIS: 1, FATURA: 2, TAHAKKUK_FIS: 3, DAYANAK: 4 };
const kaynakPill: Record<string, { t: string; c: string }> = {
  DIS: { t: "DIŞ · banka", c: "var(--pos)" },
  GIB: { t: "GİB · fatura", c: "var(--acc, #3b82f6)" },
  IC: { t: "İÇ · kayıt", c: "var(--mut)" },
  BELGE: { t: "belge", c: "var(--warn)" },
};
const durumRenk: Record<string, string> = { TAM: "var(--pos)", KISMI: "var(--warn)", GECIKMIS: "var(--warn)", ESLESMEYEN: "var(--neg)" };

export default function EslesmeModal({ anahtar, onKapat }: { anahtar: string; onKapat: () => void }) {
  const [iz, setIz] = useState<Iz | null>(null);
  const [secili, setSecili] = useState<string | null>(null);
  const navigate = useNavigate();
  const hedefeGit = (h: Hedef) => {
    if (h.rota === "indir") return;
    if (h.id) { navigate(`/${h.rota}?vurgu=${encodeURIComponent(h.id)}`); onKapat(); }
  };

  useEffect(() => {
    fetch(`${API}/api/belge/oys/iz/${anahtar}`)
      .then((r) => r.json())
      .then((d: Iz) => { setIz(d); setSecili(d.dugumler[0]?.id ?? null); })
      .catch(() => {});
  }, [anahtar]);

  const sirali = iz ? [...iz.dugumler].sort((a, b) => (TIP_SIRA[a.tip] ?? 9) - (TIP_SIRA[b.tip] ?? 9)) : [];
  const sec = iz?.dugumler.find((d) => d.id === secili) ?? null;

  const alanDeger = (k: string, v: unknown) => {
    if (typeof v === "number") return ["Matrah", "KDV", "Toplam", "Tutar", "Tahsil", "Açık"].includes(k) ? tl(v) : String(v);
    return String(v ?? "—");
  };

  return (
    <div onClick={onKapat} style={{ position: "fixed", inset: 0, background: "rgba(0,0,0,.45)", display: "flex", alignItems: "flex-start", justifyContent: "center", zIndex: 1000, padding: "5vh 16px", overflow: "auto" }}>
      <div onClick={(e) => e.stopPropagation()} style={{ background: "var(--bg, #fff)", color: "var(--ink, #16191d)", border: "1px solid var(--border2)", borderRadius: 14, width: 900, maxWidth: "100%", boxShadow: "0 20px 60px rgba(0,0,0,.3)" }}>
        {/* başlık */}
        <div style={{ display: "flex", alignItems: "center", gap: 10, padding: "14px 20px", borderBottom: "1px solid var(--border2)" }}>
          <span style={{ fontFamily: "var(--font-voice, serif)", fontWeight: 600, fontSize: 17 }}>İz-sürme</span>
          <span className="mono" style={{ color: "var(--mut)" }}>{iz?.iz_id ?? anahtar}</span>
          {iz && <span className="pill" style={{ marginLeft: 6, background: "transparent", border: `1.5px solid ${durumRenk[iz.guven.durum]}`, color: durumRenk[iz.guven.durum] }}>{iz.guven.durum} · otomatik ipucu</span>}
          <button className="btn" style={{ marginLeft: "auto", fontSize: 13 }} onClick={onKapat}>✕</button>
        </div>

        {!iz ? (
          <div style={{ padding: 30, color: "var(--mut)" }}>Yükleniyor…</div>
        ) : (
          <div style={{ padding: 20 }}>
            {/* özet */}
            <div style={{ display: "flex", alignItems: "center", gap: 10, fontSize: 13.5, marginBottom: 4 }}>
              <b>{iz.ozet.sol.etiket}</b> <span className="mono">{iz.ozet.sol.deger}</span>
              {iz.ozet.sol.tutar != null && <b>{tl(iz.ozet.sol.tutar)} ₺</b>}
              <span style={{ color: "var(--mut)" }}>↔</span>
              {iz.ozet.sag && <><b>{iz.ozet.sag.etiket}</b> <span>{iz.ozet.sag.deger}</span></>}
              <span className="pill" style={{ marginLeft: "auto", background: "var(--bg2)" }}>{iz.ozet.yon}</span>
            </div>
            <div style={{ fontSize: 12, color: "var(--mut)", marginBottom: 8 }}>{iz.guven.gerekce}</div>
            <div style={{ fontSize: 11.5, color: "var(--warn)", background: "var(--bg2)", border: "1px solid var(--border2)", borderRadius: 8, padding: "6px 10px", marginBottom: 12 }}>
              ⚠ <b>Simülasyon verisi — kanıt değildir.</b> Otomatik eşleşme yalnızca yöntem gösterimidir; nihai karar ve teyit denetçiye aittir. Gerçek açık bankacılık bağlanınca banka hareketi güçlü dış kaynak olur (BDS 500), yine de "dış teyit" (BDS 505) yerine geçmez.
            </div>

            {/* zincir bandı */}
            <div style={{ display: "flex", alignItems: "stretch", gap: 4, overflowX: "auto", paddingBottom: 6 }}>
              {sirali.map((d, i) => (
                <div key={d.id} style={{ display: "flex", alignItems: "center", gap: 4 }}>
                  {i > 0 && <span style={{ color: "var(--mut)", fontSize: 18 }}>→</span>}
                  <button onClick={() => setSecili(d.id)} style={{
                    minWidth: 132, textAlign: "left", cursor: "pointer", padding: "8px 10px", borderRadius: 10,
                    background: secili === d.id ? "var(--bg2)" : "var(--bg)",
                    border: `1.5px solid ${secili === d.id ? durumRenk[d.durum] : "var(--border2)"}`,
                  }}>
                    <div style={{ fontSize: 10, fontWeight: 700, color: kaynakPill[d.kaynak]?.c }}>{kaynakPill[d.kaynak]?.t}</div>
                    <div style={{ fontSize: 12.5, fontWeight: 600, lineHeight: 1.2, margin: "2px 0" }}>{d.etiket.length > 26 ? d.etiket.slice(0, 25) + "…" : d.etiket}</div>
                    {d.tutar != null && <div className="mono" style={{ fontSize: 12 }}>{tl(d.tutar)} ₺</div>}
                  </button>
                </div>
              ))}
            </div>

            {/* seçili düğüm detayı */}
            {sec && (
              <div style={{ marginTop: 14, border: "1px solid var(--border2)", borderRadius: 10, padding: 14 }}>
                <div style={{ fontWeight: 600, fontSize: 13, marginBottom: 8 }}>{sec.tip.replace(/_/g, " ")} — {sec.etiket}</div>
                <table style={{ width: "100%", fontSize: 12.5 }}>
                  <tbody>
                    {Object.entries(sec.alanlar).map(([k, v]) => (
                      <tr key={k}><td style={{ color: "var(--mut)", padding: "3px 8px 3px 0", width: 140 }}>{k}</td>
                        <td className="mono">{alanDeger(k, v)}</td></tr>
                    ))}
                  </tbody>
                </table>

                {/* BİLEŞENLER (yukarı) — fatura kalemleri */}
                {sec.kalemler && (
                  <>
                    <div style={{ fontWeight: 600, fontSize: 12.5, margin: "10px 0 4px", color: "var(--acc, #3b82f6)" }}>▲ Bu tutarı oluşturan bileşenler ({sec.kalemler.length} kalem)</div>
                    <table><thead><tr><th>Ürün</th><th className="num">Miktar</th><th className="num">Tutar</th></tr></thead>
                      <tbody>{sec.kalemler.map((s, i) => <tr key={i}><td>{s.ad}</td><td className="num">{s.miktar} {s.birim}</td><td className="num">{tl(s.tutar)}</td></tr>)}</tbody>
                    </table>
                  </>
                )}

                {/* BESLEDİĞİ (aşağı) — tahakkuk fişi satırları */}
                {sec.satirlar && (
                  <>
                    <div style={{ fontWeight: 600, fontSize: 12.5, margin: "10px 0 4px", color: "var(--pos)" }}>▼ Bu tutarın beslediği muhasebe kaydı</div>
                    <table><thead><tr><th>Hesap</th><th className="num">Borç</th><th className="num">Alacak</th></tr></thead>
                      <tbody>{sec.satirlar.map((s, i) => <tr key={i}><td className="mono">{s.hesap} {s.ad ?? ""}</td><td className="num">{s.borc ? tl(s.borc) : "—"}</td><td className="num">{s.alacak ? tl(s.alacak) : "—"}</td></tr>)}</tbody>
                    </table>
                  </>
                )}

                {/* hedefe git */}
                {sec.hedef && (
                  <div style={{ marginTop: 10 }}>
                    {sec.hedef.rota === "indir" && sec.hedef.url
                      ? <a className="btn" style={{ fontSize: 12, textDecoration: "none" }} href={`${API}${sec.hedef.url}`}>Belgeyi indir (UBL-TR) →</a>
                      : sec.hedef.id
                        ? <button className="btn" style={{ fontSize: 12 }} onClick={() => hedefeGit(sec.hedef!)}>
                            {sec.hedef.rota === "banka" ? "Ekstrede göster →" : "Belgede göster →"}
                          </button>
                        : null}
                  </div>
                )}
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
