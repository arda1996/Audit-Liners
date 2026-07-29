import { useState } from "react";

const API = "http://127.0.0.1:8787";

export type Kullanici = { id: string; kullanici_adi: string; ad: string; rol: string; mukellef_idleri: string[]; departman?: string; kademe?: string };

/** Giriş ekranı — kayıt YOK; hesapları firma yöneticisi (admin) açar. */
export default function Login({ onGiris }: { onGiris: (token: string, k: Kullanici) => void }) {
  const [ka, setKa] = useState("");
  const [sifre, setSifre] = useState("");
  const [hata, setHata] = useState("");
  const [bekle, setBekle] = useState(false);

  const gir = async (e: React.FormEvent) => {
    e.preventDefault();
    setHata(""); setBekle(true);
    try {
      const r = await fetch(`${API}/api/giris`, {
        method: "POST", headers: { "content-type": "application/json" },
        body: JSON.stringify({ kullanici_adi: ka.trim(), sifre }),
      });
      const d = await r.json();
      if (r.ok) onGiris(d.token, d.kullanici);
      else setHata(d.hata ?? "Giriş başarısız");
    } catch {
      setHata("Sunucuya ulaşılamadı — API çalışıyor mu?");
    } finally { setBekle(false); }
  };

  return (
    <div className="giris-ekran">
      <form className="giris-kart" onSubmit={gir}>
        <img src="/logo.svg" width={54} height={54} alt="" style={{ marginBottom: 4 }} />
        <div className="giris-marka">Audit<span style={{ color: "#E23A32" }}>-</span>Liners</div>
        <div className="giris-alt">Muhasebe &amp; Denetim</div>
        <label style={{ marginTop: 18 }}>Kullanıcı adı</label>
        <input autoFocus value={ka} onChange={(e) => setKa(e.target.value)} placeholder="kullanıcı adı" />
        <label style={{ marginTop: 12 }}>Parola</label>
        <input type="password" value={sifre} onChange={(e) => setSifre(e.target.value)} placeholder="••••••••" />
        {hata && <div className="giris-hata">{hata}</div>}
        <button className="primary" disabled={bekle || !ka.trim() || !sifre} style={{ width: "100%", marginTop: 16, justifyContent: "center" }}>
          {bekle ? "Giriş yapılıyor…" : "Giriş yap"}
        </button>
        <div className="giris-not">Hesabınızı firma yöneticiniz oluşturur — kayıt yoktur.</div>
      </form>
    </div>
  );
}
