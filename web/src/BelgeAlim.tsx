// BELGE ALIM EKRANI — belgeyi yükle, alanları al, muhasebe kaydını kur.
//
// Akış tek yönlü ve geri alınabilir:
//   belge yükle → mod seç → alanları SIRAYLA doldur → her birini ONAYLA → fiş taslağı → deftere yaz
//
// Ekranın taşıdığı ilke: hiçbir değer sessizce kabul edilmez. Otomatik doldurma yalnız
// belgenin kendi denklemi tuttuğunda olur; seçim onaydan geçer; manuel giriş taramayla
// çelişirse uyarı görünür. Bunların hepsi backend'de kararlaştırılır — bu dosya kararı
// vermez, GÖSTERİR. Aynı kural iki yerde yaşarsa er geç ayrışır.

import { useEffect, useMemo, useRef, useState } from "react";

const API = "http://127.0.0.1:8787";

type Alan = {
  kod: string; ad: string; tip: string; sira: number; zorunlu: boolean; ipucu: string;
  ham: string; deger: number | null; tarama: number | null; tarama_ham: string;
  etiket_bulundu: boolean; onaylandi: boolean; kaynak: string;
};
type Bulgu = { kod: string; onem: string; mesaj: string; cozum: string };
type Oturum = {
  id: string; belge_turu: string; belge_turu_ad: string; dil: string; mod_: string;
  alanlar: Alan[]; metin: string; tamamlandi: boolean;
};
type Dosya = {
  ad: string; tur: string; bayt: number; metin_katmani: boolean;
  metin: string; satirlar: string[]; sayfa_sayisi: number;
  onizleme: string; onerilen_mod: string; konum_kelime: number; uyarilar: string[];
};
type Sablon = {
  modlar: Record<string, { ad: string; aciklama: string }>;
  belge_turleri: Record<string, {
    ad: string;
    parametreler: { kod: string; ad: string; tip: string; sira: number; zorunlu: boolean; ipucu: string; secenekler?: string[] }[];
    denklemler: { ad: string; not?: string }[];
  }>;
};

const tl = (k: number | null) =>
  k == null ? "—" : (k / 100).toLocaleString("tr-TR", { minimumFractionDigits: 2, maximumFractionDigits: 2 });

/** Bulgu rozeti — önem sırası renkle değil METİNLE de belli olsun (renk körlüğü + yazdırma). */
function BulguSatir({ b }: { b: Bulgu }) {
  const renk = b.onem === "ENGEL" ? "var(--kirmizi, #c0392b)"
    : b.onem === "UYARI" ? "var(--turuncu, #b9770e)" : "var(--mut2, #6b7280)";
  return (
    <div style={{ borderLeft: `3px solid ${renk}`, padding: "6px 10px", marginBottom: 6, fontSize: 12.5 }}>
      <div style={{ fontWeight: 600, color: renk }}>[{b.kod}] {b.onem}</div>
      <div>{b.mesaj}</div>
      {b.cozum && <div style={{ color: "var(--mut2)", marginTop: 2 }}>→ {b.cozum}</div>}
    </div>
  );
}

export default function BelgeAlim() {
  const [sablon, setSablon] = useState<Sablon | null>(null);
  const [belgeTuru, setBelgeTuru] = useState("FATURA");
  const [mod, setMod] = useState("SECIMLI");
  const [metin, setMetin] = useState("");
  const [dosya, setDosya] = useState<Dosya | null>(null);
  const [yukleniyor, setYukleniyor] = useState(false);
  const [oturum, setOturum] = useState<Oturum | null>(null);
  const [bulgular, setBulgular] = useState<Bulgu[]>([]);
  const [otomatikSonuc, setOtomatikSonuc] = useState<{ doldurma: string; sebep: string; oneri?: string } | null>(null);
  const [taslak, setTaslak] = useState<any>(null);
  const [mesaj, setMesaj] = useState<{ ok: boolean; text: string } | null>(null);
  const [girdi, setGirdi] = useState("");
  const [ogrenme, setOgrenme] = useState<{ sablon_id: string; yeni_kural: number; not: string } | null>(null);
  const secimAlani = useRef<HTMLTextAreaElement | null>(null);

  useEffect(() => {
    fetch(`${API}/api/belge/alim/sablonlar`).then((r) => r.json()).then(setSablon).catch(() => {});
  }, []);

  const aktif = useMemo(() => oturum?.alanlar.find((a) => !a.onaylandi) ?? null, [oturum]);
  const tur = sablon?.belge_turleri[belgeTuru];
  const aktifParam = tur?.parametreler.find((p) => p.kod === aktif?.kod);

  // Belge sunucuya gönderilir: metin katmanı ÇIKARILIR ve sayfa önizlemesi üretilir.
  // Mod önerisi buradan gelir — metin katmanı yoksa otomatik mod sahte güven verir.
  const dosyaSec = async (f: File) => {
    setYukleniyor(true); setMesaj(null); setDosya(null);
    try {
      const b64 = await new Promise<string>((coz, red) => {
        const r = new FileReader();
        r.onload = () => coz(String(r.result));
        r.onerror = () => red(new Error("dosya okunamadı"));
        r.readAsDataURL(f);
      });
      const d: Dosya & { hata?: string; ipucu?: string } =
        await (await fetch(`${API}/api/belge/dosya`, {
          method: "POST", headers: { "content-type": "application/json" },
          body: JSON.stringify({ ad: f.name, veri_b64: b64 }),
        })).json();
      if (d.hata) { setMesaj({ ok: false, text: `${d.hata}${d.ipucu ? " — " + d.ipucu : ""}` }); return; }
      setDosya(d);
      setMetin(d.metin ?? "");
      const secilenMod = d.onerilen_mod || "SECIMLI";
      setMod(secilenMod);
      setMesaj({ ok: true, text: d.metin_katmani
        ? `${d.ad}: metin katmanı bulundu (${d.satirlar.length} satır, ${d.konum_kelime} konumlu kelime).`
        : `${d.ad}: metin katmanı yok. Değerleri belgeye bakarak seç veya yaz.` });
      // MOTOR HEMEN DEVREYE GİRER — kullanıcı ayrıca "başlat" demek zorunda değil.
      // Belgeyi yüklemek zaten "bunu oku" demektir; ayrı bir adım istemek gereksiz sürtünme.
      await basla(secilenMod, d.metin ?? "");
    } catch (e) {
      setMesaj({ ok: false, text: `Dosya yüklenemedi: ${e instanceof Error ? e.message : e}` });
    } finally { setYukleniyor(false); }
  };

  /** Belgedeki bir satıra tıklandı — değeri sıradaki alana taşı.
   *  "Belge üzerinden seçim" budur: kullanıcı yazmıyor, belgeden ALIYOR. */
  const satirSec = (satir: string) => {
    if (!aktif) return;
    // Etiketli satırda ':' sonrası değerdir; değilse satırın tamamı.
    const [, kuyruk] = satir.split(/:(.+)/);
    setGirdi((kuyruk ?? satir).trim());
  };

  // Parametreler dışarıdan verilebilir: yükleme akışı state güncellemesini BEKLEYEMEZ
  // (React state'i eşzamansızdır), bu yüzden değerler doğrudan geçirilir.
  const basla = async (modu?: string, belgeMetni?: string) => {
    setTaslak(null);
    const r = await fetch(`${API}/api/belge/alim/basla`, {
      method: "POST", headers: { "content-type": "application/json" },
      body: JSON.stringify({ belge_turu: belgeTuru, mod_: modu ?? mod,
                             metin: belgeMetni ?? metin, oturum_id: "" }),
    });
    const d = await r.json();
    if (d.hata) { setMesaj({ ok: false, text: d.hata }); return; }
    setOturum(d.oturum);
    setOtomatikSonuc(d.otomatik ?? null);
    setBulgular(d.otomatik?.bulgular ?? []);
    setGirdi("");
  };

  const degerYaz = async () => {
    if (!oturum || !aktif) return;
    const r = await fetch(`${API}/api/belge/alim/deger`, {
      method: "POST", headers: { "content-type": "application/json" },
      body: JSON.stringify({ oturum_id: oturum.id, kod: aktif.kod, ham: girdi, kaynak: mod === "MANUEL" ? "MANUEL" : "SECIM" }),
    });
    const d = await r.json();
    setBulgular(d.bulgular ?? []);
    await tazele();
  };

  const onayla = async () => {
    if (!oturum || !aktif) return;
    const r = await fetch(`${API}/api/belge/alim/onayla`, {
      method: "POST", headers: { "content-type": "application/json" },
      body: JSON.stringify({ oturum_id: oturum.id, kod: aktif.kod }),
    });
    const d = await r.json();
    if (d.hata) { setMesaj({ ok: false, text: d.hata }); return; }
    setBulgular(d.bulgular ?? []);
    setGirdi("");
    await tazele();
  };

  const tazele = async () => {
    if (!oturum) return;
    const d = await (await fetch(`${API}/api/belge/alim/${encodeURIComponent(oturum.id)}`)).json();
    if (d.oturum) setOturum(d.oturum);
  };

  const fisKur = async () => {
    if (!oturum) return;
    const t = await (await fetch(`${API}/api/belge/alim/tamamla`, {
      method: "POST", headers: { "content-type": "application/json" },
      body: JSON.stringify({ oturum_id: oturum.id, kod: "" }),
    })).json();
    setBulgular(t.bulgular ?? []);
    if (!t.tamamlandi) { setMesaj({ ok: false, text: t.not ?? "Oturum tamamlanmadı." }); return; }
    if (t.ogrenme) setOgrenme(t.ogrenme);

    const d = await (await fetch(`${API}/api/belge/alim/fis-taslagi`, {
      method: "POST", headers: { "content-type": "application/json" },
      body: JSON.stringify({ oturum_id: oturum.id, kod: "" }),
    })).json();
    if (d.hata) { setMesaj({ ok: false, text: d.hata }); return; }
    setTaslak(d);
    setMesaj({ ok: true, text: "Fiş taslağı hazır. Kontrol et ve deftere yaz." });
  };

  const deftereYaz = async () => {
    if (!taslak) return;
    const f = taslak.fis;
    const r = await fetch(`${API}/api/fis`, {
      method: "POST", headers: { "content-type": "application/json" },
      body: JSON.stringify({
        tip: f.tip, tarih: f.tarih, aciklama: f.aciklama,
        satirlar: f.satirlar.map((s: any) => ({ hesap: s.hesap, borc: s.borc, alacak: s.alacak, aciklama: s.aciklama })),
        dayanak_tur: f.dayanak_tur, dayanak_no: f.dayanak_no,
      }),
    });
    const d = await r.json();
    if (!r.ok) { setMesaj({ ok: false, text: d.hata ?? "Fiş kesinleştirilemedi." }); return; }
    setMesaj({ ok: true, text: `Fiş deftere yazıldı — yevmiye no ${d.yevmiye_no ?? d.id ?? "?"}.` });
    setTaslak(null); setOturum(null);
  };

  const engelVar = bulgular.some((b) => b.onem === "ENGEL");

  return (
    <div style={{ padding: 16, maxWidth: 1100 }}>
      <h2 style={{ marginBottom: 4 }}>Belge alım — muhasebe kaydı</h2>
      <div style={{ color: "var(--mut2)", fontSize: 12.5, marginBottom: 14 }}>
        Belge, kaydın dayanağıdır (VUK 227). Alanlar doğrulanmadan fiş üretilmez.
      </div>

      {/* ── 1. Belge ve mod ─────────────────────────────────────────── */}
      <section className="card" style={{ padding: 12, marginBottom: 12 }}>
        <div style={{ fontWeight: 600, marginBottom: 8 }}>1) Belge ve yöntem</div>
        <div style={{ display: "flex", gap: 10, flexWrap: "wrap", alignItems: "center" }}>
          <label>
            Belge türü{" "}
            <select value={belgeTuru} onChange={(e) => setBelgeTuru(e.target.value)} disabled={!!oturum}>
              {sablon && Object.entries(sablon.belge_turleri).map(([k, v]) => (
                <option key={k} value={k}>{v.ad}</option>
              ))}
            </select>
          </label>
          <label>
            Yöntem{" "}
            <select value={mod} onChange={(e) => setMod(e.target.value)} disabled={!!oturum}>
              {sablon && Object.entries(sablon.modlar).map(([k, v]) => (
                <option key={k} value={k}>{v.ad}</option>
              ))}
            </select>
          </label>
          <input data-testid="dosya-sec" type="file" accept=".pdf,image/*" disabled={!!oturum || yukleniyor}
                 onChange={(e) => e.target.files?.[0] && dosyaSec(e.target.files[0])} />
          {yukleniyor && <span style={{ fontSize: 12 }}>okunuyor…</span>}
          {dosya && (
            <span style={{ fontSize: 12, color: "var(--mut2)" }}>
              {dosya.ad} · {dosya.tur} · {(dosya.bayt / 1024).toFixed(0)} KB ·{" "}
              {dosya.metin_katmani ? `metin katmanı VAR (${dosya.satirlar.length} satır)` : "metin katmanı YOK"}
            </span>
          )}
        </div>
        {sablon && <div style={{ fontSize: 12, color: "var(--mut2)", marginTop: 6 }}>{sablon.modlar[mod]?.aciklama}</div>}

        {(dosya?.uyarilar ?? []).map((u, i) => (
          <div key={i} style={{ fontSize: 12, color: "var(--turuncu, #b9770e)", marginTop: 4 }}>! {u}</div>
        ))}
        {!dosya && (
          <textarea
            value={metin} onChange={(e) => setMetin(e.target.value)} disabled={!!oturum}
            placeholder="Dosya yoksa belgenin metnini buraya yapıştırabilirsin (isteğe bağlı)."
            style={{ width: "100%", height: 70, marginTop: 8, fontFamily: "ui-monospace, monospace", fontSize: 12 }}
          />
        )}
        <div style={{ marginTop: 8 }}>
          {!oturum
            ? <button onClick={() => basla()}>Alımı başlat</button>
            : <button onClick={() => { setOturum(null); setTaslak(null); setBulgular([]); setOtomatikSonuc(null); }}>Baştan başla</button>}
        </div>
      </section>

      {otomatikSonuc && (
        <section className="card" style={{ padding: 12, marginBottom: 12 }}>
          <div style={{ fontWeight: 600 }}>
            Otomatik doldurma: {otomatikSonuc.doldurma === "YAPILDI" ? "yapıldı" : "YAPILMADI"}
          </div>
          <div style={{ fontSize: 12.5, marginTop: 4 }}>{otomatikSonuc.sebep}</div>
          {otomatikSonuc.oneri && <div style={{ fontSize: 12.5, color: "var(--mut2)", marginTop: 2 }}>→ {otomatikSonuc.oneri}</div>}
        </section>
      )}

      {/* ── Belge görüntüleyici — kullanıcı belgeyi GÖRMEDEN seçim yapamaz ── */}
      {oturum && dosya && (
        <section className="card" data-testid="belge-goruntuleyici" style={{ padding: 12, marginBottom: 12 }}>
          <div style={{ fontWeight: 600, marginBottom: 8 }}>Belge</div>
          <div style={{ display: "flex", gap: 12, alignItems: "flex-start", flexWrap: "wrap" }}>
            {dosya.onizleme && (
              <img src={dosya.onizleme} alt={`${dosya.ad} — sayfa 1`}
                   style={{ maxWidth: 420, maxHeight: 560, border: "1px solid var(--cizgi, #e5e7eb)", borderRadius: 4 }} />
            )}
            {dosya.satirlar.length > 0 && (
              <div style={{ flex: 1, minWidth: 280 }}>
                <div style={{ fontSize: 12, color: "var(--mut2)", marginBottom: 4 }}>
                  Satıra tıkla → değer {aktif ? `“${aktif.ad}”` : "sıradaki alan"} kutusuna gelir.
                </div>
                <div data-testid="belge-satirlar" style={{ maxHeight: 520, overflowY: "auto",
                     border: "1px solid var(--cizgi, #e5e7eb)", borderRadius: 4 }}>
                  {dosya.satirlar.map((s, i) => (
                    <button key={i} onClick={() => satirSec(s)} disabled={!aktif}
                      style={{ display: "block", width: "100%", textAlign: "left", border: "none",
                               background: "transparent", cursor: aktif ? "pointer" : "default",
                               padding: "3px 8px", fontFamily: "ui-monospace, monospace", fontSize: 12 }}>
                      {s}
                    </button>
                  ))}
                </div>
              </div>
            )}
          </div>
        </section>
      )}

      {/* ── 2. Alanlar ──────────────────────────────────────────────── */}
      {oturum && (
        <section className="card" style={{ padding: 12, marginBottom: 12 }}>
          <div style={{ fontWeight: 600, marginBottom: 8 }}>
            2) Alanlar — sırayla ({oturum.alanlar.filter((a) => a.onaylandi).length}/{oturum.alanlar.length} onaylı)
          </div>
          <table style={{ width: "100%", fontSize: 12.5, borderCollapse: "collapse" }}>
            <thead>
              <tr style={{ textAlign: "left", color: "var(--mut2)" }}>
                <th style={{ width: 28 }}>#</th><th>Alan</th><th>Değer</th>
                <th>Belgeden okunan</th><th style={{ width: 90 }}>Kaynak</th><th style={{ width: 70 }}>Durum</th>
              </tr>
            </thead>
            <tbody>
              {oturum.alanlar.map((a) => {
                const sirada = a.kod === aktif?.kod;
                const celiski = a.tarama != null && a.deger != null && a.tarama !== a.deger;
                return (
                  <tr key={a.kod} style={{ background: sirada ? "var(--vurgu, #f3f6ff)" : undefined, borderTop: "1px solid var(--cizgi, #e5e7eb)" }}>
                    <td>{a.sira}</td>
                    <td>{a.ad}{a.zorunlu && <span style={{ color: "var(--kirmizi, #c0392b)" }}> *</span>}</td>
                    <td style={{ fontFamily: a.tip === "TUTAR" ? "ui-monospace, monospace" : undefined }}>
                      {a.tip === "TUTAR" ? tl(a.deger) : a.ham || "—"}
                    </td>
                    <td style={{ color: celiski ? "var(--turuncu, #b9770e)" : "var(--mut2)" }}>
                      {a.tarama_ham || (a.etiket_bulundu ? "(etiket bulundu)" : "—")}
                      {celiski && " ⟵ çelişki"}
                    </td>
                    <td style={{ color: "var(--mut2)" }}>{a.kaynak || "—"}</td>
                    <td>{a.onaylandi ? "onaylı" : sirada ? "sırada" : "bekliyor"}</td>
                  </tr>
                );
              })}
            </tbody>
          </table>

          {aktif && (
            <div data-testid="aktif-alan" style={{ marginTop: 12, padding: 10, border: "1px solid var(--cizgi, #e5e7eb)", borderRadius: 6 }}>
              <div style={{ fontWeight: 600 }}>{aktif.sira}. {aktif.ad}</div>
              <div style={{ fontSize: 12.5, color: "var(--mut2)", marginBottom: 6 }}>{aktif.ipucu}</div>
              {aktifParam?.secenekler ? (
                <select data-testid="alan-secenek" value={girdi} onChange={(e) => setGirdi(e.target.value)}>
                  <option value="">— seç —</option>
                  {aktifParam.secenekler.map((s) => <option key={s} value={s}>{s}</option>)}
                </select>
              ) : (
                <textarea
                  data-testid="alan-metin"
                  ref={secimAlani} value={girdi} onChange={(e) => setGirdi(e.target.value)}
                  placeholder={mod === "MANUEL" ? "Değeri yaz" : "Belge üzerinden seçtiğin metni buraya yapıştır"}
                  style={{ width: "100%", height: 46, fontSize: 13 }}
                />
              )}
              <div style={{ marginTop: 8, display: "flex", gap: 8, alignItems: "center" }}>
                <button onClick={degerYaz} disabled={!girdi.trim()}>Değeri al</button>
                {/* İsteğe bağlı alan boş bırakılabilmeli. Düğmeyi boşken kapatmak kullanıcıyı
                    kilitliyordu: belgede tevkifat yoksa alan doldurulamıyor, akış ilerlemiyordu. */}
                <button onClick={onayla}
                        disabled={aktif.zorunlu && !aktif.ham.trim()}
                        title={aktif.zorunlu && !aktif.ham.trim() ? "Zorunlu alan — önce değeri al" : "Gördüm, doğru"}>
                  {aktif.ham.trim() ? "Onayla ve devam" : "Bu belgede yok — atla"}
                </button>
                {!aktif.zorunlu && !aktif.ham.trim() && (
                  <span style={{ fontSize: 12, color: "var(--mut2)" }}>isteğe bağlı</span>
                )}
              </div>
            </div>
          )}

          {!aktif && (
            <div style={{ marginTop: 12 }}>
              <button onClick={fisKur} disabled={engelVar}>Muhasebe kaydını kur</button>
              {engelVar && <span style={{ marginLeft: 8, fontSize: 12.5, color: "var(--kirmizi, #c0392b)" }}>
                Engel bulgusu var — önce çöz.
              </span>}
            </div>
          )}
        </section>
      )}

      {/* ── 3. Bulgular ─────────────────────────────────────────────── */}
      {bulgular.length > 0 && (
        <section className="card" style={{ padding: 12, marginBottom: 12 }}>
          <div style={{ fontWeight: 600, marginBottom: 8 }}>Doğrulama</div>
          {bulgular.map((b, i) => <BulguSatir key={i} b={b} />)}
        </section>
      )}

      {/* ── 4. Fiş taslağı ──────────────────────────────────────────── */}
      {taslak && (
        <section className="card" style={{ padding: 12 }}>
          <div style={{ fontWeight: 600, marginBottom: 6 }}>3) Fiş taslağı</div>
          <div style={{ fontSize: 12.5, color: "var(--mut2)", marginBottom: 8 }}>
            {taslak.fis.tarih} · {taslak.fis.aciklama} · dayanak: {taslak.fis.dayanak_tur} {taslak.fis.dayanak_no}
            {" · "}KDV %{taslak.ozet.kdv_orani}{taslak.ozet.kdv_muavin && ` (muavin ${taslak.ozet.kdv_muavin})`}
            {taslak.ozet.tevkifat > 0 && ` · tevkifat ${tl(taslak.ozet.tevkifat)} → cariye ${tl(taslak.ozet.cari_tutar)}`}
          </div>
          <table data-testid="fis-taslak" style={{ width: "100%", fontSize: 13, borderCollapse: "collapse" }}>
            <thead>
              <tr style={{ textAlign: "left", color: "var(--mut2)" }}>
                <th>Hesap</th><th>Açıklama</th>
                <th style={{ textAlign: "right" }}>Borç</th><th style={{ textAlign: "right" }}>Alacak</th>
              </tr>
            </thead>
            <tbody>
              {taslak.fis.satirlar.map((s: any, i: number) => (
                <tr key={i} style={{ borderTop: "1px solid var(--cizgi, #e5e7eb)" }}>
                  <td style={{ fontFamily: "ui-monospace, monospace" }}>{s.hesap}</td>
                  <td>{s.aciklama}</td>
                  <td style={{ textAlign: "right", fontFamily: "ui-monospace, monospace" }}>{s.borc ? tl(s.borc) : ""}</td>
                  <td style={{ textAlign: "right", fontFamily: "ui-monospace, monospace" }}>{s.alacak ? tl(s.alacak) : ""}</td>
                </tr>
              ))}
              <tr style={{ borderTop: "2px solid var(--cizgi, #e5e7eb)", fontWeight: 600 }}>
                <td colSpan={2}>Toplam {taslak.denge.denk ? "(denk)" : "(DENGESİZ)"}</td>
                <td style={{ textAlign: "right", fontFamily: "ui-monospace, monospace" }}>{tl(taslak.denge.borc)}</td>
                <td style={{ textAlign: "right", fontFamily: "ui-monospace, monospace" }}>{tl(taslak.denge.alacak)}</td>
              </tr>
            </tbody>
          </table>
          {taslak.kalemler && taslak.kalemler.kalemler.length > 0 && (
            <div style={{ marginTop: 14 }}>
              <div style={{ fontWeight: 600, marginBottom: 4 }}>
                Kalemler{" "}
                <span style={{ fontWeight: 400, fontSize: 12,
                  color: taslak.kalemler.matrahla_tutuyor === false
                    ? "var(--turuncu, #b9770e)" : "var(--mut2)" }}>
                  — {taslak.kalemler.not}
                </span>
              </div>
              <table data-testid="kalem-tablo" style={{ width: "100%", fontSize: 12.5, borderCollapse: "collapse" }}>
                <thead>
                  <tr style={{ textAlign: "left", color: "var(--mut2)" }}>
                    <th>Mal / hizmet</th>
                    <th style={{ textAlign: "right" }}>Miktar</th>
                    <th>Birim</th>
                    <th style={{ textAlign: "right" }}>Birim fiyat</th>
                    <th style={{ textAlign: "right" }}>Tutar</th>
                  </tr>
                </thead>
                <tbody>
                  {taslak.kalemler.kalemler.map((k: any, i: number) => (
                    <tr key={i} style={{ borderTop: "1px solid var(--cizgi, #e5e7eb)" }}>
                      <td>{k.ad}</td>
                      <td style={{ textAlign: "right", fontFamily: "ui-monospace, monospace" }}>
                        {k.miktar.toLocaleString("tr-TR")}
                      </td>
                      <td style={{ color: "var(--mut2)" }}>{k.birim || "—"}</td>
                      <td style={{ textAlign: "right", fontFamily: "ui-monospace, monospace" }}>{tl(k.birim_fiyat)}</td>
                      <td style={{ textAlign: "right", fontFamily: "ui-monospace, monospace" }}>{tl(k.tutar)}</td>
                    </tr>
                  ))}
                  <tr style={{ borderTop: "2px solid var(--cizgi, #e5e7eb)", fontWeight: 600 }}>
                    <td colSpan={4}>Kalem toplamı</td>
                    <td style={{ textAlign: "right", fontFamily: "ui-monospace, monospace" }}>
                      {tl(taslak.kalemler.toplam)}
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
          )}

          {(taslak.bulgular ?? []).map((b: Bulgu, i: number) => <BulguSatir key={i} b={b} />)}
          <div style={{ marginTop: 10 }}>
            <button onClick={deftereYaz} disabled={!taslak.denge.denk}>Deftere yaz</button>
          </div>
        </section>
      )}

      {ogrenme && (
        <div style={{ marginTop: 12, padding: 10, borderRadius: 6, fontSize: 12.5,
                      border: "1px solid var(--cizgi, #e5e7eb)" }}>
          <b>Öğrenme:</b> {ogrenme.not}
          {ogrenme.yeni_kural > 0 && ` (${ogrenme.yeni_kural} yeni kural)`}
          {ogrenme.sablon_id && <span style={{ color: "var(--mut2)" }}> · şablon {ogrenme.sablon_id}</span>}
        </div>
      )}

      {mesaj && (
        <div className="msg" style={{ marginTop: 12, padding: 10, borderRadius: 6,
          border: `1px solid ${mesaj.ok ? "var(--yesil, #1e8449)" : "var(--kirmizi, #c0392b)"}`,
          color: mesaj.ok ? "var(--yesil, #1e8449)" : "var(--kirmizi, #c0392b)" }}>
          {mesaj.text}
        </div>
      )}
    </div>
  );
}
