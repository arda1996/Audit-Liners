#!/usr/bin/env python3
"""ÖRNEKLEM ÜRETİCİ — GİB'in UBL-TR XML'lerinden zorluk katmanlı sınama belgesi üretir.

## Neden bu yol

Türkçe fatura içeren AÇIK, ANOTASYONLU bir veri seti yoktur (arandı, bulunamadı).
Gerçek mükellef belgesi ise depoya giremez (KVKK). Elimizde kalan tek meşru kaynak
GİB'in kendi örnek XML'leri — ve onların yer gerçeği **kesindir**: matrah, KDV, toplam,
tevkifat, kalemler XML'den birebir okunur, tahmin yok.

Bu betik o XML'i alıp:
  K1-KOLAY   → metin katmanlı PDF (pdftotext birebir okur)
  K2-ORTA    → 200 dpi temiz tarama
  K3-ZOR     → 110 dpi + eğiklik + gürültü + gölge (gerçek tarayıcı bozulması)
üretir ve her belgenin yanına **beklenen değer** dosyasını yazar.

## Neden PDF'i elle yazıyoruz

macOS'ta HTML→PDF filtresi yok (`cupsfilter` reddediyor), wkhtmltopdf/weasyprint kurulu
değil. Ama bu bir kayıp değil, kazanç: PDF'i elle yazınca **sütun konumlarını birebir biz
belirliyoruz**. Böylece aynı faturayı farklı sütun düzeni ve farklı başlık adlandırmasıyla
üretip algı motorunu bilerek zorlayabiliyoruz — asıl sınanmak istenen budur.

## Türkçe karakter

Base-14 Helvetica + WinAnsiEncoding Ç ç Ö ö Ü ü kapsar ama **Ğ ğ İ ı Ş ş kapsamaz**.
Bunlar `/Differences` ile serbest kodlara bağlanıyor; poppler glif adını Unicode'a
çeviriyor. Türkçe karakter sınaması bu yüzden gerçek — ASCII'ye düşürmüyoruz.
"""
import json, math, os, random, re, subprocess, sys, xml.etree.ElementTree as ET
from pathlib import Path

KOK = Path(__file__).resolve().parent.parent
HAM = KOK / "data/ornek-belgeler/_ham"
HEDEF = KOK / "data/ornek-belgeler"

NS = {"cbc": "urn:oasis:names:specification:ubl:schema:xsd:CommonBasicComponents-2",
      "cac": "urn:oasis:names:specification:ubl:schema:xsd:CommonAggregateComponents-2"}

# ── Türkçe kodlama: WinAnsi'nin kapsamadığı 6 harf ──────────────────────────
TR_EK = {"Ğ": 0x80, "ğ": 0x81, "İ": 0x82, "ı": 0x83, "Ş": 0x84, "ş": 0x85}
TR_GLIF = ["Gbreve", "gbreve", "Idotaccent", "dotlessi", "Scedilla", "scedilla"]
WINANSI = {"Ç": 0xC7, "ç": 0xE7, "Ö": 0xD6, "ö": 0xF6, "Ü": 0xDC, "ü": 0xFC,
           "Â": 0xC2, "â": 0xE2, "Î": 0xCE, "î": 0xEE, "Û": 0xDB, "û": 0xFB}


def pdf_metin(s):
    """Metni PDF dizgisine çevirir: Türkçe harfler kodlanır, ayraçlar kaçırılır."""
    o = []
    for ch in s:
        if ch in TR_EK:      o.append(f"\\{TR_EK[ch]:03o}")
        elif ch in WINANSI:  o.append(f"\\{WINANSI[ch]:03o}")
        elif ch in "()\\":   o.append("\\" + ch)
        elif ord(ch) < 128:  o.append(ch)
        else:                o.append("?")
    return "".join(o)


def pdf_yaz(yol, ogeler, en=595, boy=842):
    """ogeler: (x, y_ust, punto, kalin, metin). y ÜSTTEN ölçülür (okumak kolay olsun)."""
    akis = []
    for x, y, punto, kalin, metin in ogeler:
        f = "F2" if kalin else "F1"
        akis.append(f"BT /{f} {punto} Tf {x:.1f} {boy - y:.1f} Td ({pdf_metin(metin)}) Tj ET")
    icerik = "\n".join(akis).encode("latin-1", "replace")

    diff = "[128 " + " ".join(f"/{g}" for g in TR_GLIF) + "]"
    nesneler = [
        b"<</Type/Catalog/Pages 2 0 R>>",
        b"<</Type/Pages/Kids[3 0 R]/Count 1>>",
        f"<</Type/Page/Parent 2 0 R/MediaBox[0 0 {en} {boy}]"
        f"/Resources<</Font<</F1 5 0 R/F2 6 0 R>>>>/Contents 4 0 R>>".encode(),
        b"<</Length " + str(len(icerik)).encode() + b">>\nstream\n" + icerik + b"\nendstream",
        b"<</Type/Font/Subtype/Type1/BaseFont/Helvetica/Encoding 7 0 R>>",
        b"<</Type/Font/Subtype/Type1/BaseFont/Helvetica-Bold/Encoding 7 0 R>>",
        f"<</Type/Encoding/BaseEncoding/WinAnsiEncoding/Differences {diff}>>".encode(),
    ]
    govde, konumlar = b"%PDF-1.4\n", []
    for i, n in enumerate(nesneler, 1):
        konumlar.append(len(govde))
        govde += f"{i} 0 obj\n".encode() + n + b"\nendobj\n"
    xref = len(govde)
    govde += f"xref\n0 {len(nesneler)+1}\n0000000000 65535 f \n".encode()
    for k in konumlar:
        govde += f"{k:010d} 00000 n \n".encode()
    govde += (f"trailer\n<</Size {len(nesneler)+1}/Root 1 0 R>>\nstartxref\n{xref}\n%%EOF").encode()
    yol.write_bytes(govde)


# ── UBL-TR okuma — YER GERÇEĞİ buradan gelir ────────────────────────────────
def para(el):
    return None if el is None else int(round(float(el.text) * 100))


def ubl_oku(yol):
    try:
        k = ET.parse(yol).getroot()
    except ET.ParseError:
        return None
    b = lambda p: k.find(p, NS)
    t = lambda p: (b(p).text.strip() if b(p) is not None and b(p).text else "")

    taraflar = k.findall(".//cac:AccountingSupplierParty/cac:Party", NS)
    alicilar = k.findall(".//cac:AccountingCustomerParty/cac:Party", NS)
    def unvan(p):
        if not p: return ""
        n = p[0].find(".//cac:PartyName/cbc:Name", NS)
        if n is not None and n.text: return n.text.strip()
        f = p[0].find(".//cac:Person/cbc:FirstName", NS)
        s = p[0].find(".//cac:Person/cbc:FamilyName", NS)
        return " ".join(x.text.strip() for x in (f, s) if x is not None and x.text)
    def vkn(p):
        if not p: return ""
        for i in p[0].findall(".//cac:PartyIdentification/cbc:ID", NS):
            if i.get("schemeID") in ("VKN", "TCKN"): return i.text.strip()
        return ""

    kalemler = []
    for s in k.findall(".//cac:InvoiceLine", NS):
        ad = s.find(".//cac:Item/cbc:Name", NS)
        mik = s.find("cbc:InvoicedQuantity", NS)
        fiy = s.find(".//cac:Price/cbc:PriceAmount", NS)
        tut = s.find("cbc:LineExtensionAmount", NS)
        if ad is None or mik is None or fiy is None or tut is None: continue
        kalemler.append({
            "ad": (ad.text or "").strip()[:34],
            "miktar": float(mik.text), "birim": (mik.get("unitCode") or "").upper(),
            "birim_fiyat": para(fiy), "tutar": para(tut),
        })

    matrah = para(b(".//cac:LegalMonetaryTotal/cbc:TaxExclusiveAmount"))
    toplam = para(b(".//cac:LegalMonetaryTotal/cbc:TaxInclusiveAmount"))
    odenecek = para(b(".//cac:LegalMonetaryTotal/cbc:PayableAmount"))
    vergiler = [para(x) for x in k.findall(".//cac:TaxTotal/cbc:TaxAmount", NS)]
    kdv = max(vergiler) if vergiler else None
    # TEVKİFAT: ödenecek < toplam ise aradaki fark alıcının satıcıya ödemediği KDV'dir.
    tevkifat = (toplam - odenecek) if (toplam and odenecek and odenecek < toplam) else 0

    return {
        "kaynak_dosya": yol.name,
        "belge_no": t("cbc:ID"), "tarih": t("cbc:IssueDate"),
        "satici": unvan(taraflar), "satici_vkn": vkn(taraflar),
        "alici": unvan(alicilar), "alici_vkn": vkn(alicilar),
        "kalemler": kalemler, "matrah": matrah, "kdv": kdv,
        "toplam": toplam, "odenecek": odenecek, "tevkifat": tevkifat,
    }


# ── DÜZEN VARYANTLARI — algı motorunu asıl zorlayan kısım ───────────────────
# Aynı fatura farklı başlık adlandırması ve farklı SÜTUN SIRASI ile üretilir.
# Motorun sütun adlandırması bunların hepsinde çalışmalı.
DUZENLER = [
    {"ad": "klasik", "dil": "tr",
     "sutunlar": [("CİNSİ", "ad", 190), ("BİRİM", "birim", 55), ("MİKTAR", "miktar", 55),
                  ("BİRİM FİYAT", "birim_fiyat", 75), ("KDV %", "oran", 45), ("TUTAR", "tutar", 80)]},
    {"ad": "sade", "dil": "tr",
     "sutunlar": [("AÇIKLAMA", "ad", 240), ("MİKTAR", "miktar", 65),
                  ("BİRİM FİYATI", "birim_fiyat", 90), ("TUTARI", "tutar", 95)]},
    {"ad": "genis", "dil": "tr",
     "sutunlar": [("SIRA", "sira", 35), ("MAL/HİZMET CİNSİ", "ad", 150), ("MİKTARI", "miktar", 50),
                  ("BİRİMİ", "birim", 45), ("BİRİM FİYATI", "birim_fiyat", 70),
                  ("İSKONTO", "iskonto", 50), ("KDV ORANI", "oran", 50), ("TUTARI", "tutar", 75)]},
    {"ad": "ingilizce", "dil": "en",
     "sutunlar": [("DESCRIPTION", "ad", 200), ("QTY", "miktar", 55), ("UNIT", "birim", 50),
                  ("UNIT PRICE", "birim_fiyat", 80), ("VAT %", "oran", 45), ("AMOUNT", "tutar", 80)]},
    # TERS SIRA: tutar solda, cinsi sağda. Aritmetik tek başına bunu ayırt EDEMEZ —
    # sütun adlandırması olmadan miktar ile fiyat karışır. Asıl sınav budur.
    {"ad": "ters", "dil": "tr",
     "sutunlar": [("TUTAR", "tutar", 90), ("BİRİM FİYAT", "birim_fiyat", 85),
                  ("MİKTAR", "miktar", 60), ("CİNSİ", "ad", 200)]},
]


def tl(k):
    return f"{k // 100:,}".replace(",", ".") + f",{k % 100:02d}"


def fatura_ciz(f, duzen):
    o, y = [], 60
    o.append((45, y, 15, True, f["satici"][:44] or "ÖRNEK FİRMA"))
    o.append((420, y, 17, True, "FATURA"))
    y += 18
    o.append((45, y, 8, False, f"VKN: {f['satici_vkn']}"))
    o.append((420, y, 9, False, f"Belge No : {f['belge_no']}"))
    y += 13
    o.append((420, y, 9, False, f"Tarih    : {f['tarih']}"))
    y += 26
    o.append((45, y, 9, True, "SAYIN"))
    o.append((330, y, 9, False, f"Vergi No : {f['alici_vkn']}"))
    y += 13
    o.append((45, y, 10, False, f["alici"][:40] or "ÖRNEK ALICI"))

    # ── kalem tablosu ──
    y += 34
    baslik_y = y
    x = 45
    for ad, _, gen in duzen["sutunlar"]:
        o.append((x, baslik_y, 8, True, ad))
        x += gen
    y += 16
    oran = round(f["kdv"] * 100 / f["matrah"]) if f["matrah"] else 0
    for i, k in enumerate(f["kalemler"][:9], 1):
        x = 45
        for _, alan, gen in duzen["sutunlar"]:
            v = {"sira": str(i), "ad": k["ad"], "birim": k["birim"] or "ADET",
                 "miktar": f"{k['miktar']:g}", "birim_fiyat": tl(k["birim_fiyat"]),
                 "oran": str(oran), "iskonto": "0", "tutar": tl(k["tutar"])}[alan]
            o.append((x, y, 8, False, v))
            x += gen
        y += 14

    # ── toplam bloğu ──
    y += 20
    for etiket, deger in [("MATRAH", f["matrah"]), (f"KDV %{oran}", f["kdv"]),
                          ("GENEL TOPLAM", f["toplam"])]:
        if deger is None: continue
        o.append((360, y, 9, True, etiket))
        o.append((480, y, 9, False, tl(deger)))
        y += 15
    if f["tevkifat"]:
        o.append((360, y, 9, True, "TEVKİFAT"))
        o.append((480, y, 9, False, tl(f["tevkifat"])))
        y += 15
        o.append((360, y, 9, True, "ÖDENECEK"))
        o.append((480, y, 9, False, tl(f["odenecek"])))
    return o


# ── BOZULMA — gerçek tarayıcı/telefon koşulları ─────────────────────────────
def bozar(png_yol, cikti, egiklik, gurultu, karartma):
    import cv2, numpy as np
    g = cv2.imread(str(png_yol))
    if g is None: return False
    h, w = g.shape[:2]
    if egiklik:
        M = cv2.getRotationMatrix2D((w / 2, h / 2), egiklik, 1.0)
        g = cv2.warpAffine(g, M, (w, h), borderValue=(255, 255, 255))
    if karartma:  # eşit olmayan aydınlatma — global Otsu'yu çökerten şey
        yy, xx = np.mgrid[0:h, 0:w]
        maske = (1.0 - karartma * (xx / w) * (yy / h)).astype(np.float32)
        g = np.clip(g * maske[..., None], 0, 255).astype(np.uint8)
    if gurultu:
        g = np.clip(g.astype(np.int16) +
                    np.random.default_rng(42).normal(0, gurultu, g.shape), 0, 255).astype(np.uint8)
    return cv2.imwrite(str(cikti), g)


def main():
    xmller = sorted((HAM / "UBL-TR1.2.1_Paketi/UBLTR_1.2.1_Paketi/xml").glob("*.xml"))
    xmller += sorted((HAM / "ubltr-ornekler").rglob("*Invoice*.xml"))
    if not xmller:
        print("XML bulunamadı — önce: bash scripts/ornek-belge-getir.sh"); return 1

    uretilen = 0
    for i, x in enumerate(xmller):
        f = ubl_oku(x)
        if not f or not f["kalemler"] or not f["matrah"] or not f["kdv"]:
            continue
        duzen = DUZENLER[i % len(DUZENLER)]
        kok_ad = re.sub(r"[^A-Za-z0-9_-]", "", x.stem)[:24] + "-" + duzen["ad"]

        beklenen = {
            "kaynak": f"GİB UBL-TR 1.2.1 örnek paketi / {x.name}",
            "lisans": "GİB resmî şema paketi (FSEK m.31 · mevzuat eki)",
            "anonim": True, "duzen": duzen["ad"], "dil": duzen["dil"],
            "beklenen": {k: v for k, v in {
                "unvan": f["alici"] or f["satici"], "belge_no": f["belge_no"],
                "matrah": f["matrah"], "kdv": f["kdv"], "toplam": f["toplam"],
            }.items() if v},
            "sutunlar": [{"MİKTAR": "MIKTAR", "MİKTARI": "MIKTAR", "QTY": "MIKTAR",
                          "CİNSİ": "MAL_ADI", "AÇIKLAMA": "MAL_ADI", "DESCRIPTION": "MAL_ADI",
                          "MAL/HİZMET CİNSİ": "MAL_ADI", "BİRİM": "BIRIM", "BİRİMİ": "BIRIM",
                          "UNIT": "BIRIM", "BİRİM FİYAT": "BIRIM_FIYAT",
                          "BİRİM FİYATI": "BIRIM_FIYAT", "UNIT PRICE": "BIRIM_FIYAT",
                          "KDV %": "KDV_ORANI", "KDV ORANI": "KDV_ORANI", "VAT %": "KDV_ORANI",
                          "TUTAR": "TUTAR", "TUTARI": "TUTAR", "AMOUNT": "TUTAR",
                          "İSKONTO": "ISKONTO", "SIRA": "SIRA"}.get(b, "?")
                         for b, _, _ in duzen["sutunlar"]],
            "kalem_sayisi": len(f["kalemler"][:9]),
            "tevkifat": f["tevkifat"] or None,
        }

        ogeler = fatura_ciz(f, duzen)
        # K1 — metin katmanlı PDF
        d1 = HEDEF / "FATURA/K1-KOLAY"; d1.mkdir(parents=True, exist_ok=True)
        pdf = d1 / f"{kok_ad}.pdf"
        pdf_yaz(pdf, ogeler)
        (d1 / f"{kok_ad}.json").write_text(json.dumps(beklenen, ensure_ascii=False, indent=2), "utf-8")

        # K2 — temiz tarama (200 dpi)
        d2 = HEDEF / "FATURA/K2-ORTA"; d2.mkdir(parents=True, exist_ok=True)
        subprocess.run(["pdftoppm", "-png", "-r", "200", "-f", "1", "-l", "1",
                        str(pdf), str(d2 / kok_ad)], capture_output=True)
        p2 = d2 / f"{kok_ad}-1.png"
        if p2.exists():
            (d2 / f"{kok_ad}-1.json").write_text(json.dumps(beklenen, ensure_ascii=False, indent=2), "utf-8")

        # K3 — zor: düşük çözünürlük + eğiklik + gürültü + eşit olmayan ışık
        d3 = HEDEF / "FATURA/K3-ZOR"; d3.mkdir(parents=True, exist_ok=True)
        subprocess.run(["pdftoppm", "-png", "-r", "110", "-f", "1", "-l", "1",
                        str(pdf), str(d3 / (kok_ad + "-ham"))], capture_output=True)
        ph = d3 / f"{kok_ad}-ham-1.png"
        if ph.exists():
            if bozar(ph, d3 / f"{kok_ad}.png", egiklik=1.4, gurultu=9, karartma=0.35):
                (d3 / f"{kok_ad}.json").write_text(json.dumps(beklenen, ensure_ascii=False, indent=2), "utf-8")
            ph.unlink()
        uretilen += 1

    print(f"✓ {uretilen} fatura × 3 zorluk = {uretilen * 3} sınama belgesi üretildi")
    print(f"  düzen varyantları: {', '.join(d['ad'] for d in DUZENLER)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
