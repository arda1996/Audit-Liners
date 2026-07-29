#!/usr/bin/env python3
"""PDF sayfalarını GÖRÜNTÜYE çevirir — Claude'un görme yeteneğiyle okunabilsin diye.

Neden gerekli: taranmış/fotoğraflı PDF'te metin katmanı YOKTUR; pdftotext boş döner.
Sayfayı PNG'ye çevirip Read ile okumak, OCR kurmadan en doğru sonucu verir —
özellikle EL YAZISINDA, çünkü Claude'un görüşü Tesseract'tan iyi okur.

Kullanım:
  python3 tools/belge-render.py <pdf> [--sayfa 1-3] [--dpi 200] [--cikti DIZIN]
"""
import argparse, sys, pathlib
try:
    import fitz  # PyMuPDF
except ImportError:
    sys.exit("PyMuPDF yok. Kur: tools/belge-venv/bin/pip install pymupdf")

def araliklar(s, azami):
    if not s: return list(range(azami))
    out = []
    for p in s.split(","):
        if "-" in p:
            a, b = p.split("-"); out += list(range(int(a) - 1, min(int(b), azami)))
        else:
            out.append(int(p) - 1)
    return [i for i in out if 0 <= i < azami]

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("pdf")
    ap.add_argument("--sayfa", default="", help="1-3 veya 1,4,7 (boş=hepsi)")
    ap.add_argument("--dpi", type=int, default=200, help="200 okunur; el yazısı/küçük punto için 300")
    ap.add_argument("--cikti", default="")
    a = ap.parse_args()

    kaynak = pathlib.Path(a.pdf)
    hedef = pathlib.Path(a.cikti) if a.cikti else kaynak.parent / f"{kaynak.stem}-sayfalar"
    hedef.mkdir(parents=True, exist_ok=True)

    d = fitz.open(kaynak)
    metin_var = False
    yazilan = []
    for i in araliklar(a.sayfa, d.page_count):
        sf = d[i]
        if sf.get_text().strip(): metin_var = True
        pix = sf.get_pixmap(dpi=a.dpi)
        yol = hedef / f"sayfa-{i+1:03d}.png"
        pix.save(yol)
        yazilan.append(str(yol))

    print(f"sayfa: {d.page_count} · render: {len(yazilan)} · dpi: {a.dpi}")
    if metin_var:
        print("NOT: Bu PDF'te METİN KATMANI VAR — `pdftotext -layout` daha hızlı ve birebir doğrudur.")
        print("     Görüntü yolunu yalnız tablo/damga/imza gibi görsel öğe gerekiyorsa kullan.")
    else:
        print("NOT: Metin katmanı YOK (taranmış). Görüntüleri Read ile oku — doğru yol budur.")
    for y in yazilan: print(y)

if __name__ == "__main__":
    main()
