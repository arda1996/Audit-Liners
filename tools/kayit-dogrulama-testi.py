#!/usr/bin/env python3
"""FAZ 2 kayıt doğrulama testleri — ufrs_kayit_ekle'nin güvenlik katmanını sınar.

K1  B3: tanımsız kaynak_ws → 400
K2  B7: tanımsız dayanak_tur ("asdf") → 400
K3  B7: çalışmada tanımsız değerleme yöntemi → 400
K4  B2: RJE'de 6xx/7xx bacağı → 400 (kâr-nötrlük)
K5  mutlu yol: geçerli AJE → 200 + no
K6  B4: aynı kayıt ikinci kez → 409 (mükerrer)
K7  B1: doğa aykırı bacak (100 Kasa alacak + karşı) → 200 ama uyarı döner
K8  vazgeç sonrası aynı kayıt tekrar atılabilir (mükerrer kilidi yalnız AÇIK kayda bakar)

Kullanım: API açıkken `python3 tools/kayit-dogrulama-testi.py`
"""
import json, sys, urllib.request

API = "http://localhost:8787"
TOK = None

def post(yol, veri):
    h = {"content-type": "application/json"}
    if TOK: h["authorization"] = "Bearer " + TOK
    req = urllib.request.Request(API + yol, json.dumps(veri).encode(), h)
    try:
        with urllib.request.urlopen(req) as r:
            return r.status, json.loads(r.read())
    except urllib.error.HTTPError as e:
        return e.code, json.loads(e.read())

st, d = post("/api/giris", {"kullanici_adi": "admin", "sifre": "audit2026"})
if st != 200:
    print("GİRİŞ BAŞARISIZ:", d); sys.exit(2)
TOK = d["token"]

def kayit(**ek):
    g = {"kaynak_ws": "WS-GUD-MDV", "tur": "AJE", "standart": "TMS 16",
         "aciklama": "test", "dayanak_tur": "ekspertiz", "dayanak_ref": "EKS-T1",
         "denetci_notu": "test notu", "degerleme_yontemi": "gud_ekspertiz",
         "degerleme_bazi": "test bazı",
         "satirlar": [{"hesap": "252", "borc": 100000, "alacak": 0, "serbest": False},
                       {"hesap": "522.90", "borc": 0, "alacak": 100000, "serbest": False}]}
    g.update(ek)
    return post("/api/ufrs/kayit", g)

sonuc, hatalar = [], []
def T(ad, kosul, detay=""):
    sonuc.append((ad, kosul))
    if not kosul: hatalar.append(f"{ad}: {detay}")

st, d = kayit(kaynak_ws="WS-HAYALET")
T("K1 tanımsız ws RED", st == 400 and "tanımsız" in d.get("hata",""), f"{st} {d}")

st, d = kayit(dayanak_tur="asdf")
T("K2 uydurma dayanak RED", st == 400 and "dayanak" in d.get("hata",""), f"{st} {d}")

st, d = kayit(degerleme_yontemi="hayali_yontem")
T("K3 tanımsız yöntem RED", st == 400 and "yöntem" in d.get("hata",""), f"{st} {d}")

# K4: WS-KIDEM kapsamı 77x'i İÇERİR — RJE'de 770 bacağı kapsamdan değil KÂR-NÖTRLÜKTEN düşmeli
st, d = kayit(tur="RJE", kaynak_ws="WS-KIDEM", standart="TMS 19",
              degerleme_yontemi="aktueryal", dayanak_tur="aktueryal",
              satirlar=[{"hesap": "770", "borc": 50000, "alacak": 0, "serbest": False},
                         {"hesap": "472", "borc": 0, "alacak": 50000, "serbest": False}])
T("K4 RJE kâr bacağı RED", st == 400 and "RJE" in d.get("hata",""), f"{st} {d}")

st, d = kayit(satirlar=[{"hesap": "253", "borc": 77700, "alacak": 0, "serbest": False},
                          {"hesap": "522.90", "borc": 0, "alacak": 77700, "serbest": False}])
T("K5 mutlu yol 200", st == 200 and d.get("no"), f"{st} {d}")
no5 = d.get("no")

st, d = kayit(satirlar=[{"hesap": "253", "borc": 77700, "alacak": 0, "serbest": False},
                          {"hesap": "522.90", "borc": 0, "alacak": 77700, "serbest": False}])
T("K6 mükerrer 409", st == 409 and "mükerrer" in d.get("hata",""), f"{st} {d}")

# K7: WS-RECLASS kapsamı sınıf 1-5'in tamamı — doğa aykırı bacaklar (320 borç, 100 alacak) uyarı üretir
st, d = kayit(kaynak_ws="WS-RECLASS", standart="TMS 1", tur="RJE",
              degerleme_yontemi="maliyet", dayanak_tur="sozlesme",
              satirlar=[{"hesap": "320", "borc": 12300, "alacak": 0, "serbest": False},
                          {"hesap": "100", "borc": 0, "alacak": 12300, "serbest": False}])
T("K7 doğa uyarısı", st == 200 and any("doğalı" in u for u in d.get("uyarilar", [])), f"{st} {d}")

# K9: KAPALI DÜNYA — WS-GUD-MDV (TMS 16) kapsamı 25/522 + senaryo/EV hesapları; 320 kapsam dışı → RED
st, d = kayit(satirlar=[{"hesap": "320", "borc": 50000, "alacak": 0, "serbest": False},
                          {"hesap": "522.90", "borc": 0, "alacak": 50000, "serbest": False}])
T("K9 kapsam dışı RED", st == 400 and "kapsamı dışında" in d.get("hata",""), f"{st} {d}")

# K10: EV hesabı (691.90) kapsam İÇİ sayılmalı (senaryo motoru EV bacağını forma düşürür)
st, d = kayit(satirlar=[{"hesap": "659.90", "borc": 20000, "alacak": 0, "serbest": False},
                          {"hesap": "252", "borc": 0, "alacak": 20000, "serbest": False},
                          {"hesap": "283.90", "borc": 5000, "alacak": 0, "serbest": False},
                          {"hesap": "691.90", "borc": 0, "alacak": 5000, "serbest": False}])
T("K10 EV bacağı kapsam içi", st == 200, f"{st} {d}")

st, d = post(f"/api/ufrs/kayit/{no5}/vazgec", {})
T("K8a vazgeç 200", st == 200, f"{st} {d}")
st, d = kayit(satirlar=[{"hesap": "253", "borc": 77700, "alacak": 0, "serbest": False},
                          {"hesap": "522.90", "borc": 0, "alacak": 77700, "serbest": False}])
T("K8b vazgeç sonrası tekrar 200", st == 200, f"{st} {d}")
if st == 200: post(f"/api/ufrs/kayit/{d['no']}/vazgec", {})

gecti = sum(1 for _, ok in sonuc if ok)
print(f"KAYIT DOĞRULAMA: {gecti}/{len(sonuc)} geçti")
for h in hatalar: print("  ✗", h)
sys.exit(1 if hatalar else 0)
