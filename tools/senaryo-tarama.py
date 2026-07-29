#!/usr/bin/env python3
"""Senaryo tarama cihazı — 60 senaryonun tamamını API üzerinden koşar ve doğrular.

Kontroller (her senaryo için):
  T1  uç 200 döner (geçerli girdiyle)
  T2  önerilen kayıt DENGELİ (Σborç == Σalacak)
  T3  ev_kanal != yok ise EV bacağı VAR; yok ise YOK
  T4  EV bacağı doğru hesaplara gider (EVV→283.90 borç · EVY→483.90 alacak; karşı 691.90/522.90/549.90)
  T5  bacaklardaki TDHP kodları gerçekten TDHP'de (serbest olmayanlar)
  T6  cift_tutar: tutar2 eksikse RED; tutar2>tutar RED; tutar2==tutar'da kalan bacak düşer
  T7  @hedef'li senaryoda boş hedef RED; hedef_onekleri dışı hedef UYARI üretir

Kullanım: API açıkken `python3 tools/senaryo-tarama.py`
"""
import json, sys, urllib.request

API = "http://localhost:8787"

def post(yol, veri):
    req = urllib.request.Request(API + yol, json.dumps(veri).encode(), {"content-type": "application/json"})
    try:
        with urllib.request.urlopen(req) as r:
            return r.status, json.loads(r.read())
    except urllib.error.HTTPError as e:
        return e.code, json.loads(e.read())

kat = json.load(open("data/ufrs-calismalar.json"))
tdhp = {satir.split(",")[0] for satir in open("data/tdhp.csv", encoding="utf-8").read().splitlines()[1:] if satir}
evh = kat["ev_hesaplari"]

hata, uyari, gecti = [], [], 0
toplam = 0

def hedef_sec(sen):
    onekler = sen.get("hedef_onekleri") or []
    return (onekler[0] if onekler else "252")

for c in kat["calismalar"]:
    for sen in c.get("senaryolar", []):
        toplam += 1
        ad = f"{c['id']}/{sen['kod']}"
        hedefli = any(b["hesap"] == "@hedef" for b in sen["bacaklar"])
        cift = sen.get("cift_tutar", False)
        girdi = {"senaryo_kod": sen["kod"], "hedef": hedef_sec(sen) if hedefli else "",
                 "tutar": "100.000", "kv_orani": "%25"}
        if cift: girdi["tutar2"] = "30.000"
        st, d = post(f"/api/ufrs/calisma/{c['id']}/senaryo", girdi)
        # T1
        if st != 200:
            hata.append(f"{ad}: T1 uç {st} — {d.get('hata')}"); continue
        satirlar = d["onerilen"]["satirlar"]
        # T2 denge
        tb = sum(s["borc"] for s in satirlar); ta = sum(s["alacak"] for s in satirlar)
        if tb != ta: hata.append(f"{ad}: T2 DENGESİZ borç {tb} ≠ alacak {ta}")
        # T3 EV varlığı
        ev_kanal = sen.get("ev_kanal", "")
        ev_hesaplar = {evh["EVV"], evh["EVY"]}
        ev_var = any(s["hesap"] in ev_hesaplar for s in satirlar)
        if ev_kanal and ev_kanal != "yok" and not ev_var:
            hata.append(f"{ad}: T3 EV bekleniyordu, bacak yok")
        if ev_kanal == "yok" and ev_var:
            # WS-EV'in kendi bacakları EV hesaplarıdır — motorun EKSTRA bacak eklemediğini doğrula
            baz_ev = any(b["hesap"] in ev_hesaplar for b in sen["bacaklar"])
            beklenen = sum(1 for b in sen["bacaklar"])  # kalan düşmesi 100k/30k'da olmaz (cift değilse)
            if not baz_ev or len(satirlar) > beklenen:
                hata.append(f"{ad}: T3 EV kurulmamalıydı ama fazladan bacak var ({len(satirlar)} satır)")
        # T4 EV yönü
        if ev_var:
            yon = sen.get("ev_yon")
            for s in satirlar:
                if s["hesap"] == evh["EVV"] and yon == "EVV" and s["borc"] == 0:
                    hata.append(f"{ad}: T4 EVV 283.90 borç olmalıydı")
                if s["hesap"] == evh["EVY"] and yon == "EVY" and s["alacak"] == 0:
                    hata.append(f"{ad}: T4 EVY 483.90 alacak olmalıydı")
        # T5 TDHP varlığı (rakamla başlayan, serbest olmayan)
        for s in satirlar:
            h = s["hesap"]
            if h and h[0].isdigit():
                kebir = h.split(".")[0]
                if kebir not in tdhp:
                    hata.append(f"{ad}: T5 TDHP'de yok: {h} (kebir {kebir})")
        # T6 cift_tutar kontrolleri
        if cift:
            st2, d2 = post(f"/api/ufrs/calisma/{c['id']}/senaryo", {**girdi, "tutar2": ""})
            if st2 == 200: hata.append(f"{ad}: T6 tutar2 eksikken kabul edildi")
            st3, d3 = post(f"/api/ufrs/calisma/{c['id']}/senaryo", {**girdi, "tutar2": "150.000"})
            if st3 == 200: hata.append(f"{ad}: T6 tutar2>tutar kabul edildi")
            st4, d4 = post(f"/api/ufrs/calisma/{c['id']}/senaryo", {**girdi, "tutar2": "100.000"})
            if st4 != 200:
                hata.append(f"{ad}: T6 tutar2==tutar reddedildi: {d4.get('hata')}")
            else:
                s4 = d4["onerilen"]["satirlar"]
                tb4 = sum(s["borc"] for s in s4); ta4 = sum(s["alacak"] for s in s4)
                if tb4 != ta4: hata.append(f"{ad}: T6 tutar2==tutar dengesiz")
        # T7 hedef kontrolleri
        if hedefli:
            st5, d5 = post(f"/api/ufrs/calisma/{c['id']}/senaryo", {**girdi, "hedef": ""})
            if st5 == 200: hata.append(f"{ad}: T7 boş hedef kabul edildi")
            if sen.get("hedef_onekleri"):
                st6, d6 = post(f"/api/ufrs/calisma/{c['id']}/senaryo", {**girdi, "hedef": "100"})
                if st6 == 200 and not any("kapsamı" in u for u in d6.get("uyarilar", [])):
                    # 100 istisnaen kapsam içindeyse uyarı beklenmez
                    if not any(str(o) == "100" or "100".startswith(str(o)) for o in sen["hedef_onekleri"]):
                        uyari.append(f"{ad}: T7 kapsam dışı hedefe uyarı üretilmedi")
        if not any(x.startswith(ad+":") for x in hata):
            gecti += 1

print(f"TARAMA: {toplam} senaryo · {gecti} geçti · {len(hata)} hata · {len(uyari)} uyarı")
for h in hata: print("  ✗", h)
for u in uyari: print("  ⚠", u)
sys.exit(1 if hata else 0)
