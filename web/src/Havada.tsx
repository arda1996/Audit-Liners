// Havada Bakiye sayfası — İZOLE modül (#/havada). Eşleşmemiş para hareketleri.
// İki ayrı durum vardır ve karıştırılmamalıdır:
//   1) FATURA VAR, eşleşme yapılmamış → fatura seç, eşleştir (muhasebe kaydı doğar)
//   2) FATURA HENÜZ KESİLMEMİŞ       → firma seç, "fatura bekleniyor" işaretle (kayıt DOĞMAZ)
// İkincisinde ortada belge yoktur; belge olmadan kayıt yapılamaz (VUK 227/231).
import { useEffect, useState } from "react";

const API = "http://127.0.0.1:8787";

type Satir = {
  tahsilat_id: string; tarih: string; yon: string; tutar: number; acik: number;
  firma: string; enstruman: string; banka_ad: string; hesap_id: string; iban: string;
  sebep: string; fatura_bekleniyor: boolean; bekleyen_firma: string;
  bekleme_gun: number; hatirlatma: string;
};
type Havada = {
  adet: number; toplam: number; fatura_bekleyen: number; gecikmis_hatirlatma: number;
  bankalar: { banka: string; adet: number; tutar: number }[];
  satirlar: Satir[]; not: string;
};
type Fatura = { no: string; tarih: string; toplam: number; kalan: number };

const tl = (k: number) => (k / 100).toLocaleString("tr-TR", { minimumFractionDigits: 2, maximumFractionDigits: 2 });

export default function Havada() {
  const [veri, setVeri] = useState<Havada | null>(null);
  const [acik, setAcik] = useState<string | null>(null);
  const [firmalar, setFirmalar] = useState<{ firma: string; acik: number }[]>([]);
  const [secilenFirma, setSecilenFirma] = useState("");
  const [faturalar, setFaturalar] = useState<Fatura[]>([]);
  const [mesaj, setMesaj] = useState<{ ok: boolean; text: string } | null>(null);
  // Güvenlik katmanı: ENGEL işlemi durdurur, UYARI bilgilendirir (kontrol.rs K-kodları).
  const [uyarilar, setUyarilar] = useState<{ kod: string; onem: string; mesaj: string; cozum: string }[]>([]);
  const [bankaSuz, setBankaSuz] = useState("");

  const yenile = () => fetch(`${API}/api/havada`).then((r) => r.json()).then(setVeri).catch(() => {});
  useEffect(() => {
    yenile();
    fetch(`${API}/api/simulasyon/firmalar`).then((r) => r.json()).then(setFirmalar).catch(() => {});
  }, []);

  // Firma seçilince o firmanın AÇIK faturaları gelir — eşleştirme yalnız açık faturaya yapılır.
  useEffect(() => {
    if (!secilenFirma) { setFaturalar([]); return; }
    // Yön SEÇİLEN HAREKETTEN gelir, sabit "SATIS" değil. Sabit olduğunda giden para (ALIS)
    // için liste hep boş dönüyordu: maker hangi firmayı seçerse seçsin bağlanacak fatura
    // göremiyordu. Yön ekstrenin olgusudur; backend de artık ters yönü K05 ile reddediyor.
    const yon = veri?.satirlar.find((x) => x.tahsilat_id === acik)?.yon ?? "SATIS";
    const qs = new URLSearchParams({ firma: secilenFirma, yon, tahsis_edilebilir: "1", sirala: "tarih_asc", limit: "100" });
    fetch(`${API}/api/simulasyon/faturalar?${qs}`).then((r) => r.json()).then((d) => setFaturalar(d.faturalar ?? [])).catch(() => {});
  }, [secilenFirma, acik, veri]);

  const eslestir = async (tahsilatId: string, faturaNo: string) => {
    const r = await fetch(`${API}/api/tahsis/manuel`, {
      method: "POST", headers: { "content-type": "application/json" },
      body: JSON.stringify({ tahsilat_id: tahsilatId, fatura_no: faturaNo }),
    });
    const d = await r.json();
    if (!r.ok) { setUyarilar([]); setMesaj({ ok: false, text: d.hata ?? "Eşleştirilemedi." }); return; }
    await fetch(`${API}/api/muhasebe/aktar`, { method: "POST" }).catch(() => {});
    setUyarilar(d.on_kontrol_uyarilari ?? []);
    // Değişmez ihlali = motor hatası. Sessiz geçilmez, kullanıcıya bildirilir.
    const ihlal = (d.degismez_ihlalleri ?? []) as { kod: string; mesaj: string }[];
    setMesaj({
      ok: ihlal.length === 0,
      text: ihlal.length
        ? `⚠ SİSTEM HATASI — değişmez ihlali: ${ihlal.map((x) => `[${x.kod}] ${x.mesaj}`).join(" · ")}`
        : `${d.sonuc}: ${d.aciklama}` +
          (d.dusurulen_tahsisler?.length ? ` · Yer açmak için düşürülen: ${d.dusurulen_tahsisler.map((x: {tahsilat_id:string;tutar:number}) => `${x.tahsilat_id} (${tl(x.tutar)})`).join(", ")}` : "") +
          ` · deftere ${d.muhasebe?.yeni_fis ?? 0} fiş, ${d.muhasebe?.storno_fis ?? 0} storno işlendi.`,
    });
    setAcik(null); setSecilenFirma(""); yenile();
  };

  const beklemeIsaretle = async (s: Satir) => {
    if (!secilenFirma) { setMesaj({ ok: false, text: "Önce beklenen faturanın firmasını seç." }); return; }
    const r = await fetch(`${API}/api/havada/fatura-bekleniyor`, {
      method: "POST", headers: { "content-type": "application/json" },
      body: JSON.stringify({ tahsilat_id: s.tahsilat_id, firma: secilenFirma, tarih: s.tarih }),
    });
    const d = await r.json();
    if (!r.ok) { setMesaj({ ok: false, text: d.hata ?? "İşaretlenemedi." }); return; }
    setMesaj({ ok: true, text: `${s.tahsilat_id} — ${secilenFirma} için "fatura bekleniyor" işaretlendi. Belge gelmeden muhasebe kaydı doğmaz.` });
    setAcik(null); setSecilenFirma(""); yenile();
  };

  const beklemeKaldir = async (id: string) => {
    await fetch(`${API}/api/havada/fatura-bekleniyor/${encodeURIComponent(id)}`, { method: "DELETE" }).catch(() => {});
    yenile();
  };

  if (!veri) return <div className="card" style={{ color: "var(--mut)" }}>Yükleniyor…</div>;
  const gosterilen = bankaSuz ? veri.satirlar.filter((s) => s.banka_ad === bankaSuz) : veri.satirlar;

  return (
    <>
      <div className="card" style={{ border: veri.adet > 0 ? "1.5px solid var(--neg)" : undefined }}>
        <span className="card-title" style={{ color: veri.adet > 0 ? "var(--neg)" : undefined }}>
          ⚠ Havada bakiye — eşleşmemiş para hareketi
        </span>
        <p style={{ fontSize: 12.5, color: "var(--mut)", margin: "6px 0 12px" }}>
          <b style={{ color: "var(--neg)" }}>{veri.adet}</b> hareket · <b>{tl(veri.toplam)} ₺</b> karşılığı belgeye bağlanmadı.
          {veri.fatura_bekleyen > 0 && <> · <b>{veri.fatura_bekleyen}</b> tanesi fatura bekliyor</>}
          {veri.gecikmis_hatirlatma > 0 && <> · <b style={{ color: "var(--neg)" }}>{veri.gecikmis_hatirlatma} gecikmiş hatırlatma</b></>}
          <br />{veri.not}
        </p>

        {/* BANKA BAZINDA UYARI — para hangi bankada, tıkla o bankaya süz */}
        <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
          {veri.bankalar.map((b) => (
            <button key={b.banka} onClick={() => setBankaSuz(bankaSuz === b.banka ? "" : b.banka)}
              className="card" style={{
                padding: "10px 14px", cursor: "pointer", textAlign: "left",
                border: bankaSuz === b.banka ? "1.5px solid var(--neg)" : "1px solid var(--border2)",
                background: bankaSuz === b.banka ? "var(--bg2)" : undefined,
              }}>
              <div style={{ fontWeight: 600, fontSize: 13 }}>{b.banka}</div>
              <div style={{ fontSize: 12, color: "var(--mut)" }}>{b.adet} hareket</div>
              <div className="num" style={{ fontWeight: 700, color: "var(--neg)" }}>{tl(b.tutar)} ₺</div>
            </button>
          ))}
          {bankaSuz && <button className="btn" style={{ fontSize: 12, alignSelf: "center" }} onClick={() => setBankaSuz("")}>Süzmeyi kaldır</button>}
        </div>
      </div>

      {mesaj && (
        <div className="msg" style={{ background: mesaj.ok ? "var(--acc-bg)" : "var(--warn-bg)", color: mesaj.ok ? "var(--pos)" : "var(--warn)" }}>
          {mesaj.text}
        </div>
      )}
      {uyarilar.length > 0 && (
        <div className="card" style={{ border: "1px solid var(--warn)", padding: 12 }}>
          <b style={{ color: "var(--warn)", fontSize: 13 }}>Ön kontrol uyarıları</b>
          {uyarilar.map((u) => (
            <div key={u.kod} style={{ fontSize: 12.5, marginTop: 6 }}>
              <span className="pill warn" style={{ fontSize: 10, marginRight: 6 }}>{u.kod}</span>
              {u.mesaj} <span style={{ color: "var(--mut)" }}>→ {u.cozum}</span>
            </div>
          ))}
        </div>
      )}

      <div className="card" style={{ padding: 0, overflow: "hidden" }}>
        <table>
          <thead>
            <tr>
              <th style={{ paddingLeft: 20 }}>Referans</th>
              <th>Tarih</th>
              <th>Banka / hesap</th>
              <th className="num">Açık tutar</th>
              <th>Durum</th>
              <th style={{ paddingRight: 20 }}></th>
            </tr>
          </thead>
          <tbody>
            {gosterilen.length === 0 && <tr><td colSpan={6} style={{ color: "var(--mut)", padding: 20 }}>Havada bakiye yok — tüm para belgeye bağlı.</td></tr>}
            {gosterilen.map((s) => [
              <tr key={s.tahsilat_id}>
                <td className="mono" style={{ paddingLeft: 20 }}>{s.tahsilat_id}</td>
                <td style={{ color: "var(--mut)" }}>{s.tarih}</td>
                <td style={{ fontSize: 12.5 }}>
                  {s.banka_ad}
                  {s.iban && <div className="mono" style={{ fontSize: 11, color: "var(--mut2)" }}>{s.iban}</div>}
                </td>
                <td className="num" style={{ fontWeight: 600, color: "var(--neg)" }}>{tl(s.acik)}</td>
                <td style={{ whiteSpace: "nowrap" }}>
                  {s.fatura_bekleniyor
                    ? <>
                        <span className="pill warn">fatura bekleniyor</span>
                        <div style={{ fontSize: 11, color: "var(--mut2)", marginTop: 2 }}>{s.bekleyen_firma} · {s.bekleme_gun} gün</div>
                      </>
                    : <span className="pill neg">eşleşmedi</span>}
                </td>
                <td style={{ textAlign: "right", paddingRight: 20, whiteSpace: "nowrap" }}>
                  {s.fatura_bekleniyor && <button className="btn" style={{ fontSize: 11.5 }} onClick={() => beklemeKaldir(s.tahsilat_id)}>Takibi kaldır</button>}{" "}
                  <button className="btn" style={{ fontSize: 12 }} onClick={() => { setAcik(acik === s.tahsilat_id ? null : s.tahsilat_id); setSecilenFirma(""); }}>
                    {acik === s.tahsilat_id ? "Kapat" : "Çöz"}
                  </button>
                </td>
              </tr>,
              s.hatirlatma ? (
                <tr key={s.tahsilat_id + "-h"}>
                  <td colSpan={6} style={{ background: "var(--warn-bg)", color: "var(--warn)", fontSize: 12.5, padding: "6px 20px" }}>
                    ⏰ {s.hatirlatma}
                  </td>
                </tr>
              ) : null,
              acik === s.tahsilat_id ? (
                <tr key={s.tahsilat_id + "-d"}>
                  <td colSpan={6} style={{ background: "var(--bg2)", padding: "14px 20px" }}>
                    <div style={{ fontSize: 12.5, color: "var(--mut)", marginBottom: 10 }}>{s.sebep}</div>

                    <label style={{ fontSize: 12, color: "var(--mut)" }}>1) Firma seç</label>
                    <select value={secilenFirma} onChange={(e) => setSecilenFirma(e.target.value)}
                      style={{ width: "100%", maxWidth: 420, fontSize: 13, marginTop: 4, marginBottom: 12, display: "block" }}>
                      <option value="">— firma seçin —</option>
                      {firmalar.map((f) => <option key={f.firma} value={f.firma}>{f.firma}{f.acik > 0 ? ` — açık ${tl(f.acik)} ₺` : ""}</option>)}
                    </select>

                    {secilenFirma && (
                      <>
                        <div style={{ fontSize: 12.5, fontWeight: 600, marginBottom: 4 }}>
                          2a) Faturaya bağla <span style={{ fontWeight: 400, color: "var(--mut)" }}>— {secilenFirma} açık faturaları</span>
                        </div>
                        <div style={{ maxHeight: 220, overflowY: "auto", marginBottom: 14 }}>
                          <table>
                            <thead><tr><th>Tarih</th><th>Fatura</th><th className="num">Kalan</th><th></th></tr></thead>
                            <tbody>
                              {faturalar.length === 0 && <tr><td colSpan={4} style={{ color: "var(--mut)", padding: 12 }}>Bu firmada açık fatura yok — aşağıdan "fatura bekleniyor" işaretleyebilirsin.</td></tr>}
                              {faturalar.map((f) => (
                                <tr key={f.no}>
                                  <td className="mono" style={{ color: "var(--mut)" }}>{f.tarih}</td>
                                  <td className="mono">{f.no}</td>
                                  <td className="num" style={{ color: "var(--neg)" }}>{tl(f.kalan)}</td>
                                  <td style={{ textAlign: "right" }}>
                                    <button className="btn" style={{ fontSize: 11.5 }} onClick={() => eslestir(s.tahsilat_id, f.no)}>Bu faturaya bağla</button>
                                  </td>
                                </tr>
                              ))}
                            </tbody>
                          </table>
                        </div>

                        <div style={{ fontSize: 12.5, fontWeight: 600, marginBottom: 4 }}>2b) Fatura henüz kesilmediyse</div>
                        <div style={{ fontSize: 12, color: "var(--mut)", marginBottom: 8 }}>
                          Para gelmiş/gitmiş ama belge yoksa muhasebe kaydı <b>doğmaz</b> — belge olmadan kayıt yapılamaz (VUK 227).
                          Takibe alınır ve <b>VUK 231</b> uyarınca 7 günü aşınca hatırlatma üretilir.
                        </div>
                        <button className="btn" style={{ fontSize: 12, borderColor: "var(--warn)", color: "var(--warn)" }}
                          onClick={() => beklemeIsaretle(s)}>Fatura bekleniyor olarak işaretle</button>
                      </>
                    )}
                  </td>
                </tr>
              ) : null,
            ])}
          </tbody>
        </table>
      </div>
    </>
  );
}
