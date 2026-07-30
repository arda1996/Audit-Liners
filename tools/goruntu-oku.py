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

## ÖN İŞLEME — neden zorunlu

Ölçüldü: 22 belgelik zor katmanda (110 dpi + 1.4° eğiklik + gürültü + eşit olmayan ışık)
**14 belgede KDV tutarı, 7 belgede sütun başlığı okunamıyordu.**

Üç düzeltme, üçü de OCR'a girmeden ÖNCE:

1. **EĞİKLİK GİDERME — pazarlık konusu değil.** Sütun tespiti dikey boşluk koridorlarına
   dayanır ve koridorlar eksene hizalı olmayı varsayar. 2000 px genişlikte **1° eğiklik
   35 px dikey kayma** demektir; bu, 40 px'lik bir koridoru tamamen siler.

2. **IŞIK NORMALİZASYONU — eşiklemeden.** Görüntüyü ikili hale getirmek yerine aşırı
   bulanık bir kopyasına **böleriz**: düşük frekanslı ışık gradyanı gider, gri tonlama
   kalır. İkili görüntü ince işaretleri (virgül, nokta) yok ederdi — ve virgül, tutarın
   ondalık ayracıdır.

3. **ÖLÇEK YÜKSELTME DENENDİ VE ELENDİ.** Sezgi "110 dpi model için küçük" diyordu;
   ölçüm tersini söyledi (6 belgede 189 → 170 kelime). Yeniden örnekleme bilgi eklemiyor,
   karakter kenarını yumuşatıyor. Ölçülmeyen iyileştirme, iyileştirme değildir.

Ön işleme `--ham` ile kapatılabilir — A/B ölçümü için şart, yoksa "iyileştirdim" bir
iddia olarak kalır.

Kullanım:  goruntu-oku.py <görüntü-yolu> [--ham]
Çıkış kodu 0 = başarı · 2 = model yok/kurulamadı (çağıran metin yoluna düşer)
"""
import json
import sys
from pathlib import Path

# ── TANIMA MODELİ ────────────────────────────────────────────────────────────
# Varsayılan RapidOCR modeli ÇİNCE sözlük taşıyor (`ppocr_keys_v1.txt`): 12 Türkçe
# karakterden yalnız 2'sini içeriyor. `ŞELALE` yanlış okunmuyor — `ş` çıktı alfabesinde
# YOK, üretmesi fiziksel olarak imkânsız. Kayıp %100 ve deterministik.
#
# Latin PP-OCRv5 modeli sözlüğü ONNX META VERİSİNİN İÇİNDE taşıyor (502 girdi,
# Türkçe 12/12 — doğrulandı). Ayrı sözlük dosyası gerekmiyor; kurulu sürüm (1.2.3)
# zaten meta verideki `character` alanını okuyor ve `rec_keys_path`'i yok sayıyor.
#
# Model yoksa varsayılana düşülür — kurulum zorunlu değil, ama Türkçe metin bozuk gelir.
# Kurulum: bash scripts/model-getir.sh
MODEL = Path(__file__).resolve().parent / "modeller" / "latin_PP-OCRv5_rec_mobile.onnx"

# Eğiklik bu eşiğin altındaysa döndürme YAPILMAZ — gereksiz yeniden örnekleme
# karakter kenarlarını yumuşatır, fayda yerine zarar verir.
EGIKLIK_ESIGI = 0.25


def egiklik_acisi(gri):
    """Metin satırlarının eğikliğini derece olarak döndürür.

    ## Yöntem: GÖRÜNTÜYÜ değil, MÜREKKEP KOORDİNATLARINI döndürürüz

    Klasik izdüşüm profili yöntemi doğrudur ama görüntüyü döndürerek uygulanınca
    **kenarlık yüzünden bozulur** — ölçümle iki kez doğrulandı:

    · `borderValue=0` ile döndürünce içerik kareden taşıyor, yerine boş satırlar geliyor,
      boş satırlar varyansı şişiriyor → ölçüt en uç açıda tepe yapıyor.
    · `BORDER_REPLICATE` ile döndürünce kenar satırları yayılıyor, güçlü yatay şeritler
      oluşuyor → yine uçta tepe.

    Gerçek eğikliği +1.4° olan sayfalarda iki yaklaşım da **-5.5° / -6°** buluyordu.

    Çare: ikili görüntünün mürekkep piksellerinin (y, x) koordinatlarını matematiksel
    olarak döndürüp yeni y değerlerini histograma dökmek. Mürekkep miktarı TAM OLARAK
    korunur, kenarlık diye bir şey yoktur, artefakt üretecek yüzey kalmaz.

    Puan: histogramın kareler toplamı. Metin satırları hizalandığında mürekkep az sayıda
    satıra yığılır ve kareler toplamı büyür; eğikken satırlara yayılır ve küçülür.
    """
    import cv2, numpy as np
    k = cv2.resize(gri, None, fx=0.5, fy=0.5, interpolation=cv2.INTER_AREA)
    ikili = cv2.threshold(k, 0, 255, cv2.THRESH_BINARY_INV | cv2.THRESH_OTSU)[1]
    ys, xs = np.nonzero(ikili)
    if ys.size < 200:
        return 0.0
    # Çok fazla mürekkep varsa örnekle — sonuç değişmez, süre kısalır.
    if ys.size > 60000:
        adim = ys.size // 60000 + 1
        ys, xs = ys[::adim], xs[::adim]
    cy, cx = ys.mean(), xs.mean()
    dy, dx = (ys - cy).astype(np.float32), (xs - cx).astype(np.float32)

    def puan(aci):
        t = np.deg2rad(float(aci))
        yr = dx * np.sin(t) + dy * np.cos(t)
        h = np.bincount((yr - yr.min()).astype(np.int64))
        return float(np.dot(h, h))   # mürekkep az satıra yığılınca büyür

    kaba = max(np.arange(-6.0, 6.01, 0.5), key=puan)
    return float(max(np.arange(kaba - 0.5, kaba + 0.51, 0.1), key=puan))


def isik_normalize(bgr_im):
    """Eşit olmayan aydınlatmayı giderir — İKİLİ HALE GETİRMEDEN ve **RENGİ KORUYARAK**.

    Parlaklık LAB uzayının L kanalında düzeltilir; a/b kanallarına dokunulmaz. Arka plan
    çok geniş bir Gauss bulanıklığıyla kestirilip L ona bölünür: gölge ve vinyet düşük
    frekanslıdır, bölmede yok olur; karakterler yüksek frekanslıdır, kalır.

    **Rengi korumak ölçüldü ve şart:** görüntüyü griye çevirmek zor katmanda kelime
    kaybını **%22,8**'e çıkarıyordu. Model renkli girdide belirgin biçimde daha iyi
    okuyor; ön işleme rengi atmamalı.

    İkili hale getirmek de yapılmaz: virgül ve nokta gibi ince işaretler yok olurdu —
    ve virgül, tutarın ondalık ayracıdır.
    """
    import cv2, numpy as np
    lab = cv2.cvtColor(bgr_im, cv2.COLOR_BGR2LAB)
    L = lab[:, :, 0]
    k = max(31, (min(L.shape) // 8) | 1)   # satır yüksekliğinden çok büyük: ışığı ölçer
    arka = np.maximum(cv2.GaussianBlur(L, (k, k), 0), 1)
    lab[:, :, 0] = np.clip((L.astype(np.float32) / arka.astype(np.float32)) * 200.0,
                           0, 255).astype(np.uint8)
    return cv2.cvtColor(lab, cv2.COLOR_LAB2BGR)


def on_isle(yol):
    """Görüntüyü OCR'a hazırlar. Başarısız olursa None döner; çağıran ham dosyayı kullanır.

    ## Ne YAPILIYOR, ne YAPILMIYOR — hepsi ölçüldü

    | Adım | Karar | Gerekçe |
    |---|---|---|
    | Eğiklik giderme | **YAPILIR** | Sütun koridoru eksene hizalı olmayı varsayar |
    | Işık normalizasyonu | **YAPILIR** (renkli) | Temiz taramada +%3,2 kelime |
    | Griye çevirme | **YAPILMAZ** | Zor katmanda −%22,8 kelime |
    | Ölçek yükseltme | **YAPILMAZ** | −%10 kelime; yeniden örnekleme kenarı yumuşatıyor |
    | İkili hale getirme | **YAPILMAZ** | Virgül/nokta kaybolur; virgül ondalık ayracıdır |

    **Eğiklik gidermenin ölçütü kelime sayısı DEĞİLDİR.** Döndürme yeniden örnekleme
    demektir ve zor katmanda OCR'a küçük bir bedel ödetir (−%3,2 kelime). Ama amacı OCR
    değil **geometridir**: 1,4° eğiklik, 2000 px genişlikte 35 px dikey kayma yapar ve
    40 px'lik bir sütun koridorunu tamamen siler. Kelimeler okunsa bile sütunlar
    çökerse alanlar birbirine karışır. Doğru ölçüt uçtan uca alan ve sütun isabetidir.
    """
    try:
        import cv2
    except ImportError:
        return None
    im = cv2.imread(str(yol), cv2.IMREAD_COLOR)
    if im is None:
        return None

    gri = cv2.cvtColor(im, cv2.COLOR_BGR2GRAY)
    try:
        aci = egiklik_acisi(gri)
    except Exception:
        aci = 0.0
    if abs(aci) >= EGIKLIK_ESIGI:
        h, w = gri.shape
        # DÜZELTME AÇININ TERSİDİR. `egiklik_acisi` "mürekkebi hangi açıyla döndürürsem
        # satırlar hizalanır" sorusunu yanıtlar; görüntüyü düzeltmek ters yön ister.
        # Doğrulandı: düzeltilen görüntüde kalan eğiklik 0.00°.
        M = cv2.getRotationMatrix2D((w / 2, h / 2), -aci, 1.0)
        im = cv2.warpAffine(im, M, (w, h), flags=cv2.INTER_CUBIC,
                            borderMode=cv2.BORDER_REPLICATE)

    return isik_normalize(im)


def main() -> int:
    if len(sys.argv) < 2:
        print("kullanım: goruntu-oku.py <görüntü-yolu> [--ham]", file=sys.stderr)
        return 1
    yol = sys.argv[1]
    ham = "--ham" in sys.argv[2:]

    try:
        from rapidocr_onnxruntime import RapidOCR
    except ImportError:
        # Model kurulu değilse bu bir çökme sebebi DEĞİLDİR: çağıran, metin katmanı
        # yoluna düşer ve kullanıcıya "görüntü okunamadı, elle gir" der.
        print("rapidocr kurulu değil", file=sys.stderr)
        return 2

    hazir = yol if ham else on_isle(yol)
    girdi = yol if hazir is None else hazir

    try:
        # Latin tanıma modeli varsa onu kullan — Türkçe karakterler ancak böyle çıkar.
        ocr = RapidOCR(rec_model_path=str(MODEL)) if MODEL.exists() else RapidOCR()
        sonuc, _ = ocr(girdi)
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
