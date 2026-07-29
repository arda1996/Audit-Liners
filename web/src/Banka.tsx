// Banka sayfası — İZOLE modül (kendi dosyası, kendi URL'i #/banka). Açık bankacılık (ÖHVPS)
// üzerinden bağlı bankaların hesap hareketlerini çeker. Şu an backend simülatörü (10 banka);
// gerçek İş Bankası istemcisi aynı uçlarla (/api/banka/oys/*) yerine geçecek → bu dosya değişmez.
import { useEffect, useState } from "react";
import { useLocation } from "react-router-dom";
import EslesmeModal from "./EslesmeModal";

const API = "http://127.0.0.1:8787";

type Hesap = {
  banka_kodu: string;
  banka_ad: string;
  hesap_id: string;
  iban: string;
  para_birimi: string;
  bakiye: number; // kuruş
};
type Islem = {
  islem_no: string;
  referans_no: string;
  tarih: string;
  yon: "GIRIS" | "CIKIS";
  tutar: number; // kuruş, pozitif
  aciklama: string;
  karsi_unvan: string | null;
  karsi_iban: string | null;
  kanal: string;
  bakiye_sonrasi: number;
};
type Hareketler = {
  hesap_id: string;
  banka_ad: string;
  iban: string;
  para_birimi: string;
  acilis_bakiye: number;
  kapanis_bakiye: number;
  islemler: Islem[];
};

const tl = (k: number) =>
  (k / 100).toLocaleString("tr-TR", { minimumFractionDigits: 2, maximumFractionDigits: 2 });

type FaturaKisa = { no: string; tarih: string; toplam: number; tahsil: number; kalan: number;
  tahsisler: { tahsilat_id: string; tutar: number; kaynak: string }[] };

/// Manuel tahsis penceresi — bir banka tutarını elle bir faturaya bağlar.
/// FIFO karinesi ezilir; bu tutarın önceki OTOMATİK eşleşmesi kalkar (sunucu bunu bildirir).
function TahsisModal({ islem, onKapat, onTamam }: { islem: Islem; onKapat: () => void; onTamam: () => void }) {
  const [faturalar, setFaturalar] = useState<FaturaKisa[]>([]);
  const [sonuc, setSonuc] = useState<{ kaldirilan_otomatik_eslesmeler: { fatura_no: string; tutar: number; kaynak: string }[]; fatura: FaturaKisa; muhasebe: { yeni_fis: number; storno_fis: number } } | null>(null);
  const [hata, setHata] = useState<string | null>(null);
  // Ünvan YOKSA otomatik eşleşme yapılamaz: maker önce firmayı seçer, sonra açık faturalardan eşler.
  const unvansiz = !islem.karsi_unvan;
  const [firmalar, setFirmalar] = useState<{ firma: string; adet: number; acik: number }[]>([]);
  const [secilenFirma, setSecilenFirma] = useState("");
  const firma = islem.karsi_unvan ?? secilenFirma;
  const yon = islem.yon === "GIRIS" ? "SATIS" : "ALIS";

  useEffect(() => {
    if (unvansiz) fetch(`${API}/api/simulasyon/firmalar`).then((r) => r.json()).then(setFirmalar).catch(() => {});
  }, [unvansiz]);

  useEffect(() => {
    if (!firma) { setFaturalar([]); return; }
    // Yalnız AÇIK faturalar — kapanmış faturaya tahsis anlamsız.
    const qs = new URLSearchParams({ firma, yon, durum: "acik", sirala: "tarih_asc", limit: "200" });
    fetch(`${API}/api/simulasyon/faturalar?${qs}`).then((r) => r.json())
      .then((d) => setFaturalar(d.faturalar ?? [])).catch(() => {});
  }, [firma, yon]);

  const bagla = async (no: string) => {
    setHata(null);
    const r = await fetch(`${API}/api/tahsis/manuel`, {
      method: "POST", headers: { "content-type": "application/json" },
      body: JSON.stringify({ tahsilat_id: islem.referans_no, fatura_no: no }),
    });
    const d = await r.json();
    if (!r.ok) { setHata(d.hata ?? "Eşleştirilemedi."); return; }
    // Düzeltmeyi gerçek deftere işle (storno + yeni kayıt yevmiyeye girsin).
    await fetch(`${API}/api/muhasebe/aktar`, { method: "POST" }).catch(() => {});
    setSonuc(d);
  };

  // Bu tutarın ŞU ANKİ (otomatik) eşleşmeleri — muhasebeci neyi değiştirdiğini görsün.
  const mevcut = faturalar.flatMap((f) => f.tahsisler.filter((t) => t.tahsilat_id === islem.referans_no)
    .map((t) => ({ no: f.no, ...t })));

  return (
    <div onClick={onKapat} style={{ position: "fixed", inset: 0, background: "rgba(0,0,0,.45)", display: "flex", alignItems: "flex-start", justifyContent: "center", zIndex: 1000, padding: "6vh 16px", overflow: "auto" }}>
      <div onClick={(e) => e.stopPropagation()} style={{ background: "var(--bg,#fff)", color: "var(--ink)", border: "1px solid var(--border2)", borderRadius: 14, width: 720, maxWidth: "100%", boxShadow: "0 20px 60px rgba(0,0,0,.3)" }}>
        <div style={{ display: "flex", alignItems: "center", gap: 10, padding: "14px 20px", borderBottom: "1px solid var(--border2)" }}>
          <span style={{ fontWeight: 600, fontSize: 15 }}>Manuel eşleştirme</span>
          <span className="mono" style={{ color: "var(--mut)" }}>{islem.referans_no} · {tl(islem.tutar)} ₺</span>
          <button className="btn" style={{ marginLeft: "auto", fontSize: 13 }} onClick={onKapat}>✕</button>
        </div>

        <div style={{ padding: 20 }}>
          {unvansiz ? (
            <div className="card" style={{ border: "1.5px solid var(--warn)", padding: 12, marginBottom: 12 }}>
              <b style={{ color: "var(--warn)" }}>⚠ Ekstrede karşı taraf ünvanı yok</b>
              <div style={{ fontSize: 12.5, color: "var(--mut)", marginTop: 4 }}>
                Hangi cariye ait olduğu bilinmediği için <b>otomatik eşleşme yapılmadı</b> — yanlış cariye tahsis
                yanlış bakiye demektir. Önce firmayı seç, sonra o firmanın açık faturalarından eşle.
                Eşleşene kadar bu tutar <b>havada bakiye</b> sayılır (tamlık/kayıt dışı riski).
              </div>
            </div>
          ) : (
            <div style={{ fontSize: 12.5, color: "var(--mut)", marginBottom: 10 }}>
              <b>{firma}</b> · {islem.tarih} · {yon === "SATIS" ? "tahsilat" : "ödeme"}.
              Bu tutar <b>otomatik</b> olarak FIFO ile (en eski açık faturadan) eşleşti. Elle değiştirirsen
              o karar önceliklidir; <b>önceki eşleşme storno edilir</b> (kayıt silinmez, ters kayıt atılır).
            </div>
          )}

          {unvansiz && (
            <div style={{ marginBottom: 12 }}>
              <label style={{ fontSize: 12, color: "var(--mut)" }}>Firma seç</label>
              <select value={secilenFirma} onChange={(e) => setSecilenFirma(e.target.value)}
                style={{ width: "100%", fontSize: 13, marginTop: 4 }}>
                <option value="">— firma seçin —</option>
                {firmalar.filter((x) => x.acik > 0).map((x) => (
                  <option key={x.firma} value={x.firma}>{x.firma} — açık {tl(x.acik)} ₺</option>
                ))}
              </select>
            </div>
          )}

          {sonuc ? (
            <>
              <div className="card" style={{ border: "1.5px solid var(--pos)", padding: 12, marginBottom: 12 }}>
                <b style={{ color: "var(--pos)" }}>Eşleştirildi.</b>
                <div style={{ fontSize: 12.5, marginTop: 6 }}>
                  <b>{sonuc.fatura.no}</b> → tahsil {tl(sonuc.fatura.tahsil)} · kalan {tl(sonuc.fatura.kalan)}
                </div>
              </div>
              <div style={{ fontSize: 12.5, marginBottom: 10, color: "var(--mut)" }}>
                Muhasebe: <b>{sonuc.muhasebe.yeni_fis}</b> yeni fiş · <b>{sonuc.muhasebe.storno_fis}</b> storno (ters kayıt).
                Eski kayıt silinmedi — VUK 219 gereği defterde iptal işaretiyle duruyor.
              </div>
              <div style={{ fontSize: 12.5, fontWeight: 600, marginBottom: 4 }}>Kaldırılan otomatik eşleşmeler</div>
              {sonuc.kaldirilan_otomatik_eslesmeler.length === 0
                ? <div style={{ fontSize: 12.5, color: "var(--mut)" }}>Yoktu — bu tutar zaten dağıtılmamıştı.</div>
                : <table><thead><tr><th>Fatura</th><th className="num">Tutar</th><th>Kaynak</th></tr></thead>
                    <tbody>{sonuc.kaldirilan_otomatik_eslesmeler.map((k, i) => (
                      <tr key={i}><td className="mono">{k.fatura_no}</td><td className="num">{tl(k.tutar)}</td><td style={{ color: "var(--mut)" }}>{k.kaynak}</td></tr>
                    ))}</tbody></table>}
              <button className="btn" style={{ fontSize: 13, marginTop: 14 }} onClick={onTamam}>Tamam</button>
            </>
          ) : (
            <>
              {mevcut.length > 0 && (
                <div style={{ fontSize: 12, color: "var(--mut)", background: "var(--bg2)", border: "1px solid var(--border2)", borderRadius: 8, padding: "6px 10px", marginBottom: 10 }}>
                  Şu an bağlı: {mevcut.map((m) => `${m.no} (${tl(m.tutar)} · ${m.kaynak})`).join(" · ")}
                </div>
              )}
              {hata && <div className="card" style={{ border: "1.5px solid var(--neg)", color: "var(--neg)", padding: 10, marginBottom: 10, fontSize: 12.5 }}>{hata}</div>}
              {firma && <div style={{ fontSize: 12.5, fontWeight: 600, marginBottom: 4 }}>
                {firma} — açık {yon === "SATIS" ? "satış" : "alış"} faturaları (eskiden yeniye · FIFO sırası)
              </div>}
              <div style={{ maxHeight: 320, overflowY: "auto" }}>
                <table>
                  <thead><tr><th>Tarih</th><th>Fatura</th><th className="num">Toplam</th><th className="num">Kalan</th><th></th></tr></thead>
                  <tbody>
                    {!firma && <tr><td colSpan={5} style={{ color: "var(--mut)", padding: 16 }}>Önce firma seç.</td></tr>}
                    {firma && faturalar.length === 0 && <tr><td colSpan={5} style={{ color: "var(--mut)", padding: 16 }}>Bu firmada açık fatura yok.</td></tr>}
                    {faturalar.map((f) => (
                      <tr key={f.no} style={{ background: f.tahsisler.some((t) => t.tahsilat_id === islem.referans_no) ? "var(--bg2)" : undefined }}>
                        <td className="mono" style={{ color: "var(--mut)" }}>{f.tarih}</td>
                        <td className="mono">{f.no}</td>
                        <td className="num">{tl(f.toplam)}</td>
                        <td className="num" style={{ color: f.kalan > 0 ? "var(--neg)" : "var(--mut)" }}>{f.kalan > 0 ? tl(f.kalan) : "—"}</td>
                        <td style={{ textAlign: "right" }}>
                          <button className="btn" style={{ fontSize: 11.5 }} onClick={() => bagla(f.no)}>Bu faturaya bağla</button>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </>
          )}
        </div>
      </div>
    </div>
  );
}

export default function Banka() {
  const [hesaplar, setHesaplar] = useState<Hesap[]>([]);
  const [secili, setSecili] = useState<string | null>(null);
  const [veri, setVeri] = useState<Hareketler | null>(null);
  const [baslangic, setBaslangic] = useState("");
  const [bitis, setBitis] = useState("");
  const [yukleniyor, setYukleniyor] = useState(false);
  const [izAnahtar, setIzAnahtar] = useState<string | null>(null);
  const [havada, setHavada] = useState<{ adet: number; toplam: number }>({ adet: 0, toplam: 0 });
  const [eslestir, setEslestir] = useState<Islem | null>(null);
  const [senkron, setSenkron] = useState<{ aktif_kayit: number; storno_fis: number; defter_satir: number; denge_ok: boolean } | null>(null);
  const [aktarim, setAktarim] = useState<{ aktarilan_fis: number; defterdeki_fis: number; denge_ok: boolean } | null>(null);

  // OTOMATİK SİSTEM SÜREKLİLİĞİ: sayfa açılışında motor çalışır, koşulları sağlayan tüm
  // hareketler eşleşip deftere işlenir (idempotent — mevcut fişi olan yeniden üretilmez).
  useEffect(() => {
    // 1) motoru çalıştır → 2) sonucu GERÇEK deftere aktar (yevmiye/kebir/mizan) → 3) havada kalanı ölç
    fetch(`${API}/api/tahsis/senkronize`, { method: "POST" })
      .then((r) => r.json()).then(setSenkron)
      .then(() => fetch(`${API}/api/muhasebe/aktar`, { method: "POST" }).then((r) => r.json()).then(setAktarim))
      .catch(() => {})
      .finally(() => fetch(`${API}/api/simulasyon/havada`).then((r) => r.json()).then(setHavada).catch(() => {}));
  }, []);

  useEffect(() => {
    fetch(`${API}/api/banka/oys/hesaplar`)
      .then((r) => r.json())
      .then(setHesaplar)
      .catch(() => {});
  }, []);

  const cek = (hesap: string) => {
    setSecili(hesap);
    setYukleniyor(true);
    const qs = new URLSearchParams({ hesap });
    if (baslangic) qs.set("baslangic", baslangic);
    if (bitis) qs.set("bitis", bitis);
    // İş Bankası hesabı = kapsamlı simülasyon feed'i (fatura ödemelerinden türeyen gerçek hareketler);
    // diğer bankalar = bağımsız simülatör. Şekil aynı → aynı tablo.
    const url = hesap === "SIM-0064-01"
      ? `${API}/api/simulasyon/banka`
      : `${API}/api/banka/oys/hareketler?${qs}`;
    fetch(url)
      .then((r) => r.json())
      .then((d) => setVeri(d))
      .catch(() => setVeri(null))
      .finally(() => setYukleniyor(false));
  };

  const toplamBakiye = hesaplar.reduce((a, h) => a + h.bakiye, 0);

  // İz-sür penceresinden gelinince: İş Bankası hesabını aç, ilgili satırı vurgula (belgeler arası geçiş).
  const vurgu = new URLSearchParams(useLocation().search).get("vurgu");
  useEffect(() => { if (vurgu) cek("SIM-0064-01"); /* eslint-disable-next-line */ }, [vurgu]);

  return (
    <>
      {havada.adet > 0 && (
        <div className="card" style={{ border: "1.5px solid var(--neg)", background: "var(--bg2)" }}>
          <span style={{ fontWeight: 700, color: "var(--neg)" }}>⚠ Havada bakiye uyarısı</span>
          <p style={{ fontSize: 12.5, color: "var(--mut)", margin: "6px 0 0" }}>
            <b style={{ color: "var(--neg)" }}>{havada.adet}</b> banka girişi ({tl(havada.toplam)} ₺) <b>hiçbir faturaya bağlanmadı</b>.
            Çoğu ekstrede <b>karşı taraf ünvanı olmadığı için</b> otomatik eşleşemedi — bunlar <b>maker kuyruğu</b>:
            satırdaki <b>Eşleştir</b> ile firma seçip açık faturadan eşle. Eşleşmeyen tutar <b>tamlık ihlali / kayıt dışı hasılat riski</b> taşır (VUK, BDS 500).
          </p>
        </div>
      )}
      {senkron && (
        <div className="card" style={{ padding: "10px 16px", fontSize: 12.5, color: "var(--mut)", display: "flex", gap: 10, flexWrap: "wrap", alignItems: "center" }}>
          <span className="pill pos">otomatik eşleştirme açık</span>
          <span><b>{senkron.aktif_kayit.toLocaleString("tr-TR")}</b> aktif tahsis fişi</span>
          {senkron.storno_fis > 0 && <span>· <b>{senkron.storno_fis}</b> storno</span>}
          <span>· defter <b>{senkron.defter_satir.toLocaleString("tr-TR")}</b> satır</span>
          {aktarim && <span>· deftere işlenen <b>{aktarim.defterdeki_fis.toLocaleString("tr-TR")}</b> fiş (yevmiye/mizan)</span>}
          <span className={"pill " + (senkron.denge_ok && (aktarim?.denge_ok ?? true) ? "pos" : "neg")}>{senkron.denge_ok ? "Σborç = Σalacak" : "DENGE BOZUK"}</span>
        </div>
      )}
      <div className="card">
        <span className="card-title">
          Açık Bankacılık — Bağlı hesaplar{" "}
          <span style={{ color: "var(--mut2)", fontSize: 12, fontWeight: 500 }}>
            · {hesaplar.length} banka · toplam {tl(toplamBakiye)} ₺
          </span>
        </span>
        <p style={{ fontSize: 12.5, color: "var(--mut)", margin: "6px 0 12px" }}>
          İş Bankası açık bankacılık bağlantısıyla çekilen hesaplar (şu an <b>simülasyon</b> — 10 banka).
          Bir hesabı seçip hareketlerini çek. Gerçek bağlantıda uçlar (<span className="mono">/api/banka/oys/*</span>) aynı kalır.
        </p>
        <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(240px, 1fr))", gap: 10 }}>
          {hesaplar.map((h) => (
            <button
              key={h.hesap_id}
              onClick={() => cek(h.hesap_id)}
              className="card"
              style={{
                textAlign: "left",
                cursor: "pointer",
                padding: 12,
                border: secili === h.hesap_id ? "1.5px solid var(--acc, var(--pos))" : "1px solid var(--border2)",
                background: secili === h.hesap_id ? "var(--bg2)" : undefined,
              }}
            >
              <div style={{ fontWeight: 600, fontSize: 13.5 }}>{h.banka_ad}</div>
              <div className="mono" style={{ fontSize: 11.5, color: "var(--mut)", margin: "3px 0" }}>{h.iban}</div>
              <div className="num" style={{ fontWeight: 600 }}>{tl(h.bakiye)} {h.para_birimi === "TRY" ? "₺" : h.para_birimi}</div>
            </button>
          ))}
          {hesaplar.length === 0 && <span style={{ color: "var(--mut)" }}>Hesaplar yükleniyor…</span>}
        </div>
      </div>

      {secili && (
        <div className="card" style={{ padding: 0, overflow: "hidden" }}>
          <div style={{ padding: "14px 20px", borderBottom: "1px solid var(--border2)", display: "flex", alignItems: "center", gap: 12, flexWrap: "wrap" }}>
            <span className="card-title" style={{ marginRight: "auto" }}>
              {veri?.banka_ad} <span className="mono" style={{ fontSize: 11.5, color: "var(--mut)" }}>{veri?.iban}</span>
            </span>
            <label style={{ fontSize: 12, color: "var(--mut)" }}>Başlangıç
              <input value={baslangic} onChange={(e) => setBaslangic(e.target.value)} placeholder="gg.aa.yyyy"
                style={{ marginLeft: 6, border: "1px solid var(--border3)", borderRadius: 7, padding: "4px 8px", width: 100, fontFamily: "DM Mono, monospace", fontSize: 12 }} />
            </label>
            <label style={{ fontSize: 12, color: "var(--mut)" }}>Bitiş
              <input value={bitis} onChange={(e) => setBitis(e.target.value)} placeholder="gg.aa.yyyy"
                style={{ marginLeft: 6, border: "1px solid var(--border3)", borderRadius: 7, padding: "4px 8px", width: 100, fontFamily: "DM Mono, monospace", fontSize: 12 }} />
            </label>
            <button className="btn" style={{ fontSize: 12 }} onClick={() => cek(secili)}>Ekstre çek</button>
          </div>
          {veri && (
            <div style={{ padding: "8px 20px", fontSize: 12, color: "var(--mut)", borderBottom: "1px solid var(--border2)" }}>
              {veri.islemler.length} hareket · açılış {tl(veri.acilis_bakiye)} ₺ → kapanış <b>{tl(veri.kapanis_bakiye)} ₺</b>
            </div>
          )}
          <table>
            <thead>
              <tr>
                <th style={{ paddingLeft: 20 }}>Tarih</th>
                <th>Açıklama</th>
                <th>Karşı taraf</th>
                <th className="num">Tutar</th>
                <th className="num" style={{ paddingRight: 20 }}>Bakiye</th>
              </tr>
            </thead>
            <tbody>
              {yukleniyor && <tr><td colSpan={5} style={{ color: "var(--mut)", padding: 20 }}>Çekiliyor…</td></tr>}
              {!yukleniyor && veri?.islemler.length === 0 && <tr><td colSpan={5} style={{ color: "var(--mut)", padding: 20 }}>Bu aralıkta hareket yok.</td></tr>}
              {!yukleniyor && veri?.islemler.map((i) => (
                <tr key={i.islem_no}
                  ref={i.referans_no === vurgu ? (el) => el?.scrollIntoView({ block: "center", behavior: "smooth" }) : undefined}
                  style={{ background: i.referans_no === vurgu ? "var(--bg2)" : undefined, outline: i.referans_no === vurgu ? "2px solid var(--acc, var(--pos))" : undefined }}>
                  <td style={{ paddingLeft: 20, color: "var(--mut)" }} className="mono">{i.tarih}</td>
                  <td>{i.aciklama}</td>
                  <td style={{ color: "var(--mut)", fontSize: 12.5 }}>{i.karsi_unvan ?? "—"}</td>
                  <td className="num" style={{ fontWeight: 600, color: i.yon === "CIKIS" ? "var(--neg)" : "var(--pos)" }}>
                    {i.referans_no?.startsWith("THS-") || i.referans_no?.startsWith("HAVADA-")
                      ? <span onClick={() => setIzAnahtar(i.referans_no)} title="İz sür" style={{ cursor: "pointer", borderBottom: `1px dotted ${i.referans_no.startsWith("HAVADA-") ? "var(--neg)" : "var(--mut)"}` }}>{i.yon === "CIKIS" ? "−" : "+"}{tl(i.tutar)}</span>
                      : <>{i.yon === "CIKIS" ? "−" : "+"}{tl(i.tutar)}</>}
                  </td>
                  <td className="num" style={{ paddingRight: 20, color: "var(--mut)", whiteSpace: "nowrap" }}>
                    {tl(i.bakiye_sonrasi)}
                    {i.referans_no?.startsWith("THS-") && (
                      <button className="btn" style={{ fontSize: 11, marginLeft: 8 }}
                        title="Bu tutarı elle bir faturaya bağla — FIFO'nun otomatik eşleşmesi kalkar"
                        onClick={() => setEslestir(i)}>Eşleştir</button>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
      {eslestir && (
        <TahsisModal islem={eslestir} onKapat={() => setEslestir(null)}
          onTamam={() => { setEslestir(null); if (secili) cek(secili); }} />
      )}
      {izAnahtar && <EslesmeModal anahtar={izAnahtar} onKapat={() => setIzAnahtar(null)} />}
    </>
  );
}
