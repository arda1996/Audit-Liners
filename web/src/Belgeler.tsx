// Belgeler sayfası — İZOLE modül (kendi dosyası, kendi URL'i #/belgeler). Kapsamlı SİMÜLASYONA bağlı:
// GİB gelen/giden faturalar (2000+), her biri tahakkuk fişi + ödeme enstrümanı (BANKA/NAKİT/ÇEK/SENET/AÇIK)
// ile. Gerçek entegratör gelince aynı uçlar (/api/simulasyon/*) canlı veriyle dolar → bu dosya değişmez.
import { useEffect, useState } from "react";
import { useLocation } from "react-router-dom";
import EslesmeModal from "./EslesmeModal";

const API = "http://127.0.0.1:8787";
const SAYFA = 50;

type FisSat = { hesap: string; borc: number; alacak: number };
type Fis = { no: string; tip: string; tarih: string; aciklama: string; satirlar: FisSat[] };
type Fatura = {
  no: string; tarih: string; yon: "SATIS" | "ALIS"; karsi: string;
  matrah: number; kdv: number; toplam: number;
  // KARMA = fatura birden çok enstrümanla kapandı (ör. kısmen çek, kısmen havale).
  enstruman: "BANKA" | "NAKIT" | "CEK" | "SENET" | "ACIK" | "KARMA"; odendi: boolean;
  tahsil: number; kalan: number;
  ebelge_tip: "E_FATURA" | "E_ARSIV"; edurum: string;
  tahsisler: { tahsilat_id: string; tarih: string; tutar: number; kaynak: string; enstruman: string }[];
  tahakkuk: Fis; odeme: Fis | null;
};

const tl = (k: number) =>
  (k / 100).toLocaleString("tr-TR", { minimumFractionDigits: 2, maximumFractionDigits: 2 });

// e-belge (GİB zarf) durumu — ödeme/tahsilat durumundan AYRI boyut, aynı satırda gösterilir.
const eDurum: Record<string, { ad: string; renk: string }> = {
  TASLAK: { ad: "Taslak", renk: "mut" },
  GONDERILDI: { ad: "Gönderildi", renk: "warn" },
  KABUL: { ad: "Kabul", renk: "pos" },
  ALINDI: { ad: "Alındı", renk: "pos" },
  ONAY_BEKLIYOR: { ad: "Onay bekliyor", renk: "warn" },
  RED: { ad: "Reddedildi", renk: "neg" },
  IPTAL: { ad: "İptal", renk: "neg" },
  RAPORLANDI: { ad: "Raporlandı", renk: "pos" },
};

function FisTablo({ f }: { f: Fis }) {
  return (
    <table style={{ marginBottom: 10 }}>
      <thead><tr><th>Hesap</th><th className="num">Borç</th><th className="num">Alacak</th></tr></thead>
      <tbody>
        {f.satirlar.map((s, i) => (
          <tr key={i}><td className="mono">{s.hesap}</td>
            <td className="num">{s.borc ? tl(s.borc) : "—"}</td>
            <td className="num">{s.alacak ? tl(s.alacak) : "—"}</td></tr>
        ))}
      </tbody>
    </table>
  );
}

function FiltreCip({ etiket, kaldir }: { etiket: string; kaldir: () => void }) {
  return (
    <span style={{ display: "inline-flex", alignItems: "center", gap: 6, fontSize: 12, background: "var(--bg2)", border: "1px solid var(--border2)", borderRadius: 999, padding: "3px 6px 3px 10px" }}>
      {etiket}
      <button onClick={kaldir} title="Kaldır" style={{ border: "none", background: "transparent", cursor: "pointer", color: "var(--mut)", fontSize: 13, lineHeight: 1 }}>✕</button>
    </span>
  );
}

export default function Belgeler() {
  const [faturalar, setFaturalar] = useState<Fatura[]>([]);
  const [toplam, setToplam] = useState(0);
  const [offset, setOffset] = useState(0);
  const [acik, setAcik] = useState<string | null>(null);
  const [izAnahtar, setIzAnahtar] = useState<string | null>(null);
  const vurgu = new URLSearchParams(useLocation().search).get("vurgu");
  // İz-sür penceresinden gelinince: o faturayı filtrele + detayını aç (belgeler arası geçiş).
  useEffect(() => { if (vurgu) { setAra(vurgu); setAcik(vurgu); } }, [vurgu]);
  const [acikToplam, setAcikToplam] = useState(0);
  const [firmalar, setFirmalar] = useState<{ firma: string; adet: number; acik: number }[]>([]);
  // Filtreler (sunucu-taraflı)
  const [ara, setAra] = useState("");
  const [firma, setFirma] = useState("");
  const [yon, setYon] = useState("");
  const [durum, setDurum] = useState("");
  const [enstruman, setEnstruman] = useState("");
  const [edurumF, setEdurumF] = useState("");   // e-belge (GİB) durumu
  const [tip, setTip] = useState("");           // E_FATURA | E_ARSIV
  const [onayBekleyen, setOnayBekleyen] = useState(0);
  const [tutarMin, setTutarMin] = useState<number | null>(null); // kuruş
  const [tutarMax, setTutarMax] = useState<number | null>(null);
  const [tarihBas, setTarihBas] = useState("");
  const [tarihBit, setTarihBit] = useState("");
  const [sirala, setSirala] = useState("tarih_desc");
  const [taslak, setTaslak] = useState("");
  const [oneriAcik, setOneriAcik] = useState(false);

  const t = taslak.trim().toLowerCase();
  type Oneri = { tip: string; etiket: string; uygula: () => void };
  const oneriler: Oneri[] = [];
  if (t) {
    firmalar.filter((x) => x.firma.toLowerCase().includes(t)).slice(0, 5).forEach((x) =>
      oneriler.push({ tip: "Firma", etiket: `${x.firma} · ${x.adet} belge${x.acik > 0 ? ` · açık ${tl(x.acik)}` : ""}`, uygula: () => setFirma(x.firma) }));
    if ("açık acik".includes(t)) oneriler.push({ tip: "Durum", etiket: "Açık", uygula: () => setDurum("acik") });
    if ("ödendi odendi tam".includes(t)) oneriler.push({ tip: "Durum", etiket: "Ödendi (tam)", uygula: () => setDurum("odendi") });
    if ("kısmi kismi".includes(t)) oneriler.push({ tip: "Durum", etiket: "Kısmi", uygula: () => setDurum("kismi") });
    if ("satış satis".includes(t)) oneriler.push({ tip: "Yön", etiket: "Satış", uygula: () => setYon("SATIS") });
    if ("alış alis".includes(t)) oneriler.push({ tip: "Yön", etiket: "Alış", uygula: () => setYon("ALIS") });
    ([["banka", "BANKA"], ["nakit", "NAKIT"], ["çek cek", "CEK"], ["senet", "SENET"]] as [string, string][])
      .forEach(([kw, v]) => { if (kw.includes(t)) oneriler.push({ tip: "Ödeme", etiket: v, uygula: () => setEnstruman(v) }); });
    // e-belge boyutu: GİB durumu + belge tipi (ayrı sayfa/sekme yerine aynı çubuktan filtrelenir)
    ([["kabul", "KABUL"], ["alındı alindi", "ALINDI"], ["onay bekliyor bekleyen", "ONAY_BEKLIYOR"], ["reddedildi red", "RED"],
      ["iptal", "IPTAL"], ["gönderildi gonderildi", "GONDERILDI"], ["raporlandı raporlandi", "RAPORLANDI"], ["taslak", "TASLAK"]] as [string, string][])
      .forEach(([kw, v]) => { if (kw.includes(t)) oneriler.push({ tip: "e-Durum", etiket: eDurum[v].ad, uygula: () => setEdurumF(v) }); });
    if ("e-fatura efatura".includes(t)) oneriler.push({ tip: "Tip", etiket: "e-Fatura", uygula: () => setTip("E_FATURA") });
    if ("e-arşiv earsiv e-arsiv".includes(t)) oneriler.push({ tip: "Tip", etiket: "e-Arşiv", uygula: () => setTip("E_ARSIV") });
    const trimmed = taslak.trim();
    if (/^\d{1,2}\.\d{1,2}\.\d{4}$/.test(trimmed)) {
      oneriler.push({ tip: "Tarih", etiket: `≥ ${trimmed}`, uygula: () => setTarihBas(trimmed) });
      oneriler.push({ tip: "Tarih", etiket: `≤ ${trimmed}`, uygula: () => setTarihBit(trimmed) });
    } else {
      const sayi = trimmed.replace(/[.\s₺]/g, "");
      if (/^\d+$/.test(sayi)) {
        const tlNum = parseInt(sayi, 10);
        oneriler.push({ tip: "Tutar", etiket: `≥ ${tlNum.toLocaleString("tr-TR")} ₺`, uygula: () => setTutarMin(tlNum * 100) });
        oneriler.push({ tip: "Tutar", etiket: `≤ ${tlNum.toLocaleString("tr-TR")} ₺`, uygula: () => setTutarMax(tlNum * 100) });
      }
    }
    oneriler.push({ tip: "Ara", etiket: `"${taslak.trim()}" metnini ara`, uygula: () => setAra(taslak.trim()) });
  }
  const oneriSec = (o: Oneri) => { o.uygula(); setTaslak(""); setOneriAcik(false); };

  useEffect(() => {
    fetch(`${API}/api/simulasyon/firmalar`).then((r) => r.json()).then(setFirmalar).catch(() => {});
    fetch(`${API}/api/simulasyon/fatura-sekmeler`).then((r) => r.json())
      .then((d) => setOnayBekleyen(d.onay_bekleyen ?? 0)).catch(() => {});
  }, []);

  useEffect(() => {
    const qs = new URLSearchParams({ limit: String(SAYFA), offset: String(offset), sirala });
    if (ara) qs.set("ara", ara);
    if (firma) qs.set("firma", firma);
    if (yon) qs.set("yon", yon);
    if (durum) qs.set("durum", durum);
    if (enstruman) qs.set("enstruman", enstruman);
    if (edurumF) qs.set("edurum", edurumF);
    if (tip) qs.set("ebelge_tip", tip);
    if (tutarMin != null) qs.set("tutar_min", String(tutarMin));
    if (tutarMax != null) qs.set("tutar_max", String(tutarMax));
    if (tarihBas) qs.set("tarih_bas", tarihBas);
    if (tarihBit) qs.set("tarih_bit", tarihBit);
    fetch(`${API}/api/simulasyon/faturalar?${qs}`)
      .then((r) => r.json())
      .then((d) => { setFaturalar(d.faturalar ?? []); setToplam(d.toplam ?? 0); setAcikToplam(d.acik_toplam ?? 0); })
      .catch(() => {});
  }, [offset, ara, firma, yon, durum, enstruman, edurumF, tip, tutarMin, tutarMax, tarihBas, tarihBit, sirala]);

  // Filtre değişince ilk sayfaya dön
  useEffect(() => { setOffset(0); }, [ara, firma, yon, durum, enstruman, edurumF, tip, tutarMin, tutarMax, tarihBas, tarihBit, sirala]);
  const temizle = () => { setAra(""); setFirma(""); setYon(""); setDurum(""); setEnstruman(""); setEdurumF(""); setTip(""); setTutarMin(null); setTutarMax(null); setTarihBas(""); setTarihBit(""); };
  const filtreVar = !!(ara || firma || yon || durum || enstruman || edurumF || tip || tutarMin != null || tutarMax != null || tarihBas || tarihBit);

  return (
    <>
      <div className="card">
        <span className="card-title">
          Belgeler — e-Fatura / e-Arşiv{" "}
          <span style={{ color: "var(--mut2)", fontSize: 12, fontWeight: 500 }}>· {toplam.toLocaleString("tr-TR")} belge · simülasyon</span>
        </span>
        <p style={{ fontSize: 12.5, color: "var(--mut)", margin: "6px 0 10px" }}>
          Tek liste — gelen/giden, e-Fatura/e-Arşiv, GİB durumu ve tahsilat aynı satırda. Dilimlemek için filtre çubuğunu kullan.
          Açık toplam: <b style={{ color: "var(--neg)" }}>{tl(acikToplam)} ₺</b>
          {onayBekleyen > 0 && <> · <span onClick={() => setEdurumF("ONAY_BEKLIYOR")} title="Gelen ticari faturada 7 günlük kabul/ret penceresi — tıkla filtrele"
            style={{ cursor: "pointer", color: "var(--warn)", fontWeight: 600, borderBottom: "1px dotted var(--warn)" }}>⏳ {onayBekleyen} onay bekliyor</span></>}
        </p>
        {/* Akıllı filtre çubuğu (token/command) — yaz → öneri → token ekle */}
        <div style={{ display: "flex", gap: 8, alignItems: "flex-start", flexWrap: "wrap" }}>
          <div style={{ position: "relative", flex: "1 1 340px", minWidth: 240 }}>
            <div style={{ display: "flex", flexWrap: "wrap", gap: 6, alignItems: "center", border: "1px solid var(--border3)", borderRadius: 10, padding: "6px 8px", background: "var(--bg)" }}>
              {ara && <FiltreCip etiket={`"${ara}"`} kaldir={() => setAra("")} />}
              {firma && <FiltreCip etiket={firma} kaldir={() => setFirma("")} />}
              {yon && <FiltreCip etiket={yon === "SATIS" ? "Satış" : "Alış"} kaldir={() => setYon("")} />}
              {durum && <FiltreCip etiket={durum === "acik" ? "Açık" : durum === "kismi" ? "Kısmi" : "Ödendi"} kaldir={() => setDurum("")} />}
              {enstruman && <FiltreCip etiket={enstruman} kaldir={() => setEnstruman("")} />}
              {edurumF && <FiltreCip etiket={eDurum[edurumF]?.ad ?? edurumF} kaldir={() => setEdurumF("")} />}
              {tip && <FiltreCip etiket={tip === "E_ARSIV" ? "e-Arşiv" : "e-Fatura"} kaldir={() => setTip("")} />}
              {tutarMin != null && <FiltreCip etiket={`≥ ${(tutarMin / 100).toLocaleString("tr-TR")} ₺`} kaldir={() => setTutarMin(null)} />}
              {tutarMax != null && <FiltreCip etiket={`≤ ${(tutarMax / 100).toLocaleString("tr-TR")} ₺`} kaldir={() => setTutarMax(null)} />}
              {tarihBas && <FiltreCip etiket={`≥ ${tarihBas}`} kaldir={() => setTarihBas("")} />}
              {tarihBit && <FiltreCip etiket={`≤ ${tarihBit}`} kaldir={() => setTarihBit("")} />}
              <input value={taslak}
                onChange={(e) => { setTaslak(e.target.value); setOneriAcik(true); }}
                onFocus={() => setOneriAcik(true)}
                onBlur={() => setTimeout(() => setOneriAcik(false), 150)}
                onKeyDown={(e) => {
                  if (e.key === "Enter" && oneriler.length) oneriSec(oneriler[0]);
                  else if (e.key === "Backspace" && !taslak) {
                    if (tarihBit) setTarihBit(""); else if (tarihBas) setTarihBas(""); else if (tutarMax != null) setTutarMax(null);
                    else if (tutarMin != null) setTutarMin(null); else if (enstruman) setEnstruman("");
                    else if (tip) setTip(""); else if (edurumF) setEdurumF(""); else if (durum) setDurum(""); else if (yon) setYon(""); else if (firma) setFirma(""); else if (ara) setAra("");
                  }
                }}
                placeholder={filtreVar ? "filtre ekle…" : "Yaz: firma · açık · satış · banka · e-arşiv · onay bekliyor · 100000 · 01.06.2026…"}
                style={{ flex: 1, minWidth: 130, border: "none", outline: "none", background: "transparent", fontSize: 13, padding: "2px 4px" }} />
            </div>
            {oneriAcik && oneriler.length > 0 && (
              <div style={{ position: "absolute", top: "100%", left: 0, right: 0, marginTop: 4, background: "var(--bg)", border: "1px solid var(--border2)", borderRadius: 10, boxShadow: "0 8px 24px rgba(0,0,0,.14)", zIndex: 50, overflow: "hidden" }}>
                {oneriler.map((o, i) => (
                  <div key={i} onMouseDown={(e) => { e.preventDefault(); oneriSec(o); }}
                    style={{ display: "flex", alignItems: "center", gap: 8, padding: "8px 12px", cursor: "pointer", fontSize: 13, borderBottom: i < oneriler.length - 1 ? "1px solid var(--border2)" : "none" }}
                    onMouseEnter={(e) => (e.currentTarget.style.background = "var(--bg2)")}
                    onMouseLeave={(e) => (e.currentTarget.style.background = "transparent")}>
                    <span className="pill" style={{ fontSize: 10, background: "var(--bg2)" }}>{o.tip}</span>
                    <span style={{ color: o.tip === "Ara" ? "var(--mut)" : "var(--ink)" }}>{o.etiket}</span>
                  </div>
                ))}
              </div>
            )}
          </div>
          <select value={sirala} onChange={(e) => setSirala(e.target.value)} style={{ fontSize: 13 }}>
            <option value="tarih_desc">Tarih ↓ (yeni)</option><option value="tarih_asc">Tarih ↑ (eski)</option><option value="tutar_desc">Tutar ↓</option>
          </select>
          {filtreVar && <button className="btn" style={{ fontSize: 12 }} onClick={temizle}>Temizle</button>}
        </div>
      </div>

      <div className="card" style={{ padding: 0, overflow: "hidden" }}>
        <table>
          <thead>
            <tr>
              <th style={{ paddingLeft: 20 }}>Belge</th>
              <th>Tarih</th>
              <th>Karşı taraf</th>
              <th className="num">Toplam</th>
              <th title="GİB e-belge (zarf) durumu">e-Durum</th>
              <th title="Tahsilat durumu + ödeme enstrümanı">Tahsilat</th>
              <th className="num">Kalan</th>
              <th style={{ paddingRight: 20 }}></th>
            </tr>
          </thead>
          <tbody>
            {faturalar.length === 0 && <tr><td colSpan={8} style={{ color: "var(--mut)", padding: 20 }}>Yükleniyor…</td></tr>}
            {faturalar.map((f) => [
              <tr key={f.no}>
                <td style={{ paddingLeft: 20, whiteSpace: "nowrap" }}>
                  <span className={"pill " + (f.yon === "SATIS" ? "pos" : "warn")} title={f.yon === "SATIS" ? "Satış (giden)" : "Alış (gelen)"}>{f.yon === "SATIS" ? "SATIŞ" : "ALIŞ"}</span>{" "}
                  <span className="mono">{f.no}</span>
                  <div style={{ fontSize: 10.5, color: "var(--mut2)", marginTop: 1 }}>{f.ebelge_tip === "E_ARSIV" ? "e-Arşiv" : "e-Fatura"}</div>
                </td>
                <td style={{ color: "var(--mut)" }}>{f.tarih}</td>
                <td>{f.karsi}</td>
                <td className="num" style={{ fontWeight: 600 }}>
                  <span onClick={() => setIzAnahtar(f.no)} title="İz sür" style={{ cursor: "pointer", borderBottom: "1px dotted var(--mut)" }}>{tl(f.toplam)}</span>
                </td>
                <td><span className={"pill " + (eDurum[f.edurum]?.renk ?? "mut")}>{eDurum[f.edurum]?.ad ?? f.edurum}</span></td>
                <td style={{ whiteSpace: "nowrap" }}>
                  {f.kalan === 0 ? <span className="pill pos">tam</span> : f.tahsil > 0 ? <span className="pill warn">kısmi</span> : <span className="pill neg">açık</span>}
                  <div style={{ fontSize: 10.5, color: "var(--mut2)", marginTop: 1 }}>{f.enstruman}</div>
                </td>
                <td className="num" style={{ fontWeight: 600, color: f.kalan > 0 ? "var(--neg)" : "var(--mut)" }}>{f.kalan > 0 ? tl(f.kalan) : "—"}</td>
                <td style={{ textAlign: "right", paddingRight: 20, whiteSpace: "nowrap" }}>
                  <button className="btn" style={{ fontSize: 12 }} onClick={() => setIzAnahtar(f.no)}>İz sür</button>{" "}
                  <button className="btn" style={{ fontSize: 12 }} onClick={() => setAcik(acik === f.no ? null : f.no)}>{acik === f.no ? "Kapat" : "Gör"}</button>
                </td>
              </tr>,
              acik === f.no ? (
                <tr key={f.no + "-d"}>
                  <td colSpan={8} style={{ background: "var(--bg2)", padding: "12px 20px" }}>
                    <div style={{ fontSize: 12.5, color: "var(--mut)", marginBottom: 6 }}>
                      Matrah {tl(f.matrah)} · KDV {tl(f.kdv)} · Toplam {tl(f.toplam)}
                    </div>
                    <div style={{ fontWeight: 600, fontSize: 12.5, marginBottom: 4 }}>Tahakkuk fişi ({f.tahakkuk.no})</div>
                    <FisTablo f={f.tahakkuk} />
                    {f.tahsisler.length > 0 && (
                      <>
                        <div style={{ fontWeight: 600, fontSize: 12.5, marginBottom: 4 }}>
                          Bu faturayı kapatan tahsisler ({f.tahsisler.length}) — cari FIFO (en eski önce)
                        </div>
                        <table style={{ marginBottom: 10 }}>
                          <thead><tr><th>Tahsilat</th><th>Tarih</th><th>Enstrüman</th><th className="num">Tutar</th><th>Kaynak</th></tr></thead>
                          <tbody>
                            {f.tahsisler.map((t, i) => (
                              <tr key={i}>
                                <td className="mono">{t.tahsilat_id}</td>
                                <td style={{ color: "var(--mut)" }}>{t.tarih}</td>
                                <td style={{ color: "var(--mut)" }}>{t.enstruman}</td>
                                <td className="num">{tl(t.tutar)}</td>
                                <td><span className={"pill " + (t.kaynak === "MANUEL" ? "warn" : "pos")}>{t.kaynak === "MANUEL" ? "manuel" : "FIFO"}</span></td>
                              </tr>
                            ))}
                          </tbody>
                        </table>
                      </>
                    )}
                    {f.odeme ? (
                      <>
                        <div style={{ fontWeight: 600, fontSize: 12.5, marginBottom: 4 }}>
                          Ödeme fişi ({f.odeme.no}) — {f.enstruman === "KARMA"
                            ? `karma: ${[...new Set(f.tahsisler.map((t) => t.enstruman))].join(" + ")}`
                            : f.enstruman}
                        </div>
                        <FisTablo f={f.odeme} />
                      </>
                    ) : (
                      <div style={{ fontSize: 12.5, color: "var(--neg)" }}>Açık — henüz ödeme yok (şüpheli alacak takibine aday).</div>
                    )}
                  </td>
                </tr>
              ) : null,
            ])}
          </tbody>
        </table>
        <div style={{ display: "flex", alignItems: "center", gap: 12, padding: "10px 20px", borderTop: "1px solid var(--border2)", fontSize: 12.5 }}>
          <button className="btn" style={{ fontSize: 12 }} disabled={offset === 0} onClick={() => setOffset(Math.max(0, offset - SAYFA))}>← Önceki</button>
          <span style={{ color: "var(--mut)" }}>{toplam === 0 ? 0 : offset + 1}–{Math.min(offset + SAYFA, toplam)} / {toplam.toLocaleString("tr-TR")}</span>
          <button className="btn" style={{ fontSize: 12 }} disabled={offset + SAYFA >= toplam} onClick={() => setOffset(offset + SAYFA)}>Sonraki →</button>
        </div>
      </div>
      {izAnahtar && <EslesmeModal anahtar={izAnahtar} onKapat={() => setIzAnahtar(null)} />}
    </>
  );
}
