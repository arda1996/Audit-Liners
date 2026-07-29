#!/usr/bin/env python3
"""GÖRÜNTÜ OKUMA KÖPRÜSÜ — taranmış belgeden KOORDİNATLI kelimeler.

Neden ayrı bir betik: okuma modeli Python ekosisteminde (onnxruntime). Rust tarafı bunu
bir alt süreç olarak çağırır ve çıktıyı `belge_konum::Kelime` ile AYNI biçimde alır.
Böylece metin katmanlı PDF ile taranmış görüntü, boru hattının geri kalanı için ayırt
edilemez hale gelir: satırlaştırma, sütun tespiti, denklemle doğrulama, şablon öğrenme
hepsi değişmeden çalışır.

Çıktı: her satırda bir kelime, JSON.
  {"metin": "...", "x0":.., "y0":.., "x1":.., "y1":.., "sayfa":1, "guven":0.87}

Koordinatlar PİKSEL cinsindendir; ölçek `pdftotext -bbox` punto ölçeğinden farklıdır ama
motor mutlak değer değil ORAN kullandığı için (satır bandı = kelime yüksekliğinin katı,
sütun boşluğu = yükseklik katı) bu fark sonucu etkilemez.

Kullanım:  goruntu-oku.py <görüntü-yolu>
Çıkış kodu 0 = başarı · 2 = model yok/kurulamadı (çağıran metin yoluna düşer)
"""
import json
import sys


def main() -> int:
    if len(sys.argv) < 2:
        print("kullanım: goruntu-oku.py <görüntü-yolu>", file=sys.stderr)
        return 1
    yol = sys.argv[1]

    try:
        from rapidocr_onnxruntime import RapidOCR
    except ImportError:
        # Model kurulu değilse bu bir çökme sebebi DEĞİLDİR: çağıran, metin katmanı
        # yoluna düşer ve kullanıcıya "görüntü okunamadı, elle gir" der.
        print("rapidocr kurulu değil", file=sys.stderr)
        return 2

    try:
        sonuc, _ = RapidOCR()(yol)
    except Exception as e:  # bozuk/okunamayan görüntü
        print(f"görüntü okunamadı: {e}", file=sys.stderr)
        return 2

    if not sonuc:
        return 0  # boş sayfa — hata değil

    for kutu, metin, guven in sonuc:
        xs = [float(p[0]) for p in kutu]
        ys = [float(p[1]) for p in kutu]
        # RapidOCR bir SATIRI tek kutu olarak verir; motor kelime bekliyor.
        # Satırı kelimelere böler ve her kelimeye, karakter uzunluğuyla ORANTILI bir
        # kutu payı veririz. Bu bir yaklaşımdır ama sütun ayrımı için yeterli:
        # asıl bilgi kelimeler arası BOŞLUĞUN nerede olduğudur.
        x0, x1 = min(xs), max(xs)
        y0, y1 = min(ys), max(ys)
        parcalar = metin.split()
        if not parcalar:
            continue
        toplam_kar = sum(len(p) for p in parcalar) + (len(parcalar) - 1)
        genislik = (x1 - x0) / max(toplam_kar, 1)
        imlec = x0
        for p in parcalar:
            w = len(p) * genislik
            print(json.dumps({
                "metin": p,
                "x0": round(imlec, 2), "y0": round(y0, 2),
                "x1": round(imlec + w, 2), "y1": round(y1, 2),
                "sayfa": 1,
                "guven": round(float(guven), 3),
            }, ensure_ascii=False))
            imlec += w + genislik  # sözcük arası bir karakterlik boşluk
    return 0


if __name__ == "__main__":
    sys.exit(main())
