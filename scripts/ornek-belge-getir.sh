#!/usr/bin/env bash
# ÖRNEKLEM GETİRİCİ — sınama belgelerini kaynağından indirir.
#
# NEDEN SCRIPT, NEDEN DEPOYA KOYMUYORUZ:
#   1. GİB paketlerinin açık lisans metni yok (FSEK m.31 resmî şema). Kullanmak sorunsuz,
#      YENİDEN DAĞITMAK ayrı bir iş — script indirir, biz taşımayız.
#   2. Bazı setler yüzlerce MB; depo şişer.
#   3. data/ornek-belgeler/ zaten .gitignore'da (mükellef verisi riski).
#
# Kullanım:  bash scripts/ornek-belge-getir.sh [katman]
#   katman 1 (varsayılan) — GİB resmî paketleri + MIT UBL-TR örnekleri   ~10 MB
#   katman 2              — + FATURA veri seti (50 şablon, CC BY 4.0)    ~347 MB
#   katman 3              — + Türkçe el yazısı (MIT)                     ~410 MB
set -uo pipefail

KOK="$(cd "$(dirname "$0")/.." && pwd)"
HEDEF="$KOK/data/ornek-belgeler"
HAM="$HEDEF/_ham"
KATMAN="${1:-1}"
mkdir -p "$HAM"

getir() { # ad url
  local ad="$1" url="$2" yol="$HAM/$1"
  if [ -s "$yol" ]; then echo "  ✓ $ad (zaten var)"; return 0; fi
  printf '  → %s ... ' "$ad"
  if curl -fsSL --max-time 300 -o "$yol" "$url"; then
    echo "$(du -h "$yol" | cut -f1)"
  else
    echo "İNDİRİLEMEDİ (atlandı)"; rm -f "$yol"; return 1
  fi
}

echo "╭─ KATMAN 1 — GİB resmî paketleri (yer gerçeği KESİN, Türkçe)"
GIB="https://ebelge.gib.gov.tr/dosyalar"
getir "UBL-TR1.2.1_Paketi.zip"      "$GIB/kilavuzlar/UBL-TR1.2.1_Paketi.zip"
getir "e-FaturaPaketi.zip"          "$GIB/kilavuzlar/e-FaturaPaketi.zip"
getir "earsiv_paket_v1.1_8.zip"     "$GIB/kilavuzlar/earsiv_paket_v1.1_8.zip"
getir "e-Gider_Pusulasi_Paketi.rar" "$GIB/kilavuzlar/e-Gider_Pusulasi_Paketi.rar"
getir "eDekont_Paketi.rar"          "$GIB/eDekont_Paketi.rar"

echo "╰─ MIT lisanslı gerçek UBL-TR örnekleri (hkutluay/UblTr)"
getir "ubltr-ornekler.zip" "https://github.com/hkutluay/UblTr/archive/refs/heads/master.zip"

if [ "$KATMAN" -ge 2 ]; then
  echo "╭─ KATMAN 2 — FATURA veri seti · 10.000 belge / 50 ŞABLON · CC BY 4.0 · ~347 MB"
  getir "FATURA.zip" "https://zenodo.org/records/8261508/files/FATURA.zip"
fi

if [ "$KATMAN" -ge 3 ]; then
  echo "╭─ KATMAN 3 — Türkçe el yazısı (MIT) · ~410 MB"
  echo "  ! HuggingFace veri seti: 'pip install datasets' ile çekilir, doğrudan URL yok."
  echo "    datasets.load_dataset('emredeveloper/turkish-ocr')"
fi

# ── AÇMA ────────────────────────────────────────────────────────────────────
echo ""
echo "╭─ Paketler açılıyor"
cd "$HAM" || exit 1
for z in *.zip; do
  [ -e "$z" ] || continue
  d="${z%.zip}"; [ -d "$d" ] && continue
  mkdir -p "$d" && unzip -qo "$z" -d "$d" 2>/dev/null && echo "  ✓ $z"
done
for r in *.rar; do
  [ -e "$r" ] || continue
  d="${r%.rar}"; [ -d "$d" ] && continue
  if command -v unar >/dev/null 2>&1; then
    mkdir -p "$d" && unar -q -o "$d" "$r" >/dev/null 2>&1 && echo "  ✓ $r"
  else
    echo "  ! $r açılamadı — 'brew install unar' gerekiyor"
  fi
done

echo ""
echo "╰─ Bulunan örnek belgeler:"
find "$HAM" \( -name "*.xml" -o -name "*.pdf" -o -name "*.xslt" \) 2>/dev/null | wc -l | xargs echo "   dosya:"
echo ""
echo "SONRAKİ ADIM: örnekleri kategori/zorluk dizinlerine yerleştir ve her birinin"
echo "yanına beklenen değer .json'u yaz (bkz. data/ornek-belgeler/OKUBENI.md)."
