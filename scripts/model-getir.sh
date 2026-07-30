#!/usr/bin/env bash
# TÜRKÇE TANIMA MODELİ — RapidOCR'ın varsayılanı Türkçe okuyamaz.
#
# Varsayılan model ÇİNCE sözlük taşır (`ppocr_keys_v1.txt`): 12 Türkçe karakterden
# yalnız 2'sini içerir. "ŞELALE" yanlış okunmaz — 'ş' çıktı alfabesinde YOKTUR,
# üretmesi fiziksel olarak imkânsızdır. Kayıp %100 ve deterministiktir.
#
# Latin PP-OCRv5 modeli sözlüğü ONNX meta verisinin İÇİNDE taşır (502 girdi,
# Türkçe 12/12 — doğrulandı). Ayrı sözlük dosyası gerekmez.
#
#   Boyut  : 7.5 MB        Lisans : Apache-2.0 (PaddleOCR türevi)
#   Çalışma: CPU, GPU gerekmez
#
# Model DEPOYA GİRMEZ (.gitignore) — ikili dosya, kaynağından indirilir.
set -euo pipefail
KOK="$(cd "$(dirname "$0")/.." && pwd)"
HEDEF="$KOK/tools/modeller"
DOSYA="$HEDEF/latin_PP-OCRv5_rec_mobile.onnx"
URL="https://www.modelscope.cn/models/RapidAI/RapidOCR/resolve/v3.9.2/onnx/PP-OCRv5/rec/latin_PP-OCRv5_rec_mobile.onnx"

mkdir -p "$HEDEF"
if [ -s "$DOSYA" ]; then echo "✓ model zaten var ($(du -h "$DOSYA" | cut -f1))"; exit 0; fi
printf 'Latin PP-OCRv5 tanıma modeli indiriliyor (7.5 MB) ... '
curl -fsSL --max-time 300 -o "$DOSYA" "$URL"
echo "$(du -h "$DOSYA" | cut -f1)"
echo "✓ tamam — Türkçe karakterler artık okunabilir"
