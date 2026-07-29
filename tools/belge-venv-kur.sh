#!/bin/sh
# Belge okuma araçlarını kurar (PDF→görüntü render + görüntü işleme).
# Tesseract GEREKMEZ: taranmış ve el yazısı belgeler Claude'un görme yeteneğiyle okunur.
set -e
cd "$(dirname "$0")/.."
python3 -m venv tools/belge-venv
tools/belge-venv/bin/pip install --quiet --upgrade pip
tools/belge-venv/bin/pip install --quiet pymupdf pillow
echo "✓ hazır — kullanım: tools/belge-venv/bin/python tools/belge-render.py <pdf> [--dpi 300]"
