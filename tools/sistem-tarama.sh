#!/bin/bash
# sistem-tarama.sh — TEK KOMUT tam sağlık taraması (token-ucuz: tüm kontroller yerel koşar,
# yalnız kompakt özet basılır). Kullanım: bash tools/sistem-tarama.sh
cd "$(dirname "$0")/.." || exit 1
PASS=0; FAIL=0; SORUNLAR=()
adim() { # adim "AD" komut...
  local ad="$1"; shift
  local out
  if out=$("$@" 2>&1); then PASS=$((PASS+1)); echo "  ✓ $ad"
  else FAIL=$((FAIL+1)); echo "  ✗ $ad"; SORUNLAR+=("── $ad ──"$'\n'"$(echo "$out" | grep -iE "error|hata|✗|failed|FAILED" | head -6)"); fi
}

echo "═══ 1. DERLEME ═══"
adim "cargo build (api)"        cargo build -p api -q
adim "cargo test (workspace)"   cargo test -q
adim "tsc --noEmit"             npx --prefix web tsc -p web --noEmit
adim "vite build"               npm --prefix web run build --silent

echo "═══ 2. VERİ BÜTÜNLÜĞÜ ═══"
adim "JSON dosyaları geçerli"   python3 - <<'EOF'
import json, glob, sys
kotu = []
for f in glob.glob("data/*.json"):
    try: json.load(open(f))
    except Exception as e: kotu.append(f"{f}: {e}")
if kotu: print("\n".join(kotu)); sys.exit(1)
EOF
adim "TDHP mükerrer kod yok"    python3 - <<'EOF'
import sys
satirlar = [l.split(",")[0] for l in open("data/tdhp.csv", encoding="utf-8").read().splitlines()[1:] if l.strip()]
cift = {k for k in satirlar if satirlar.count(k) > 1}
if cift: print("mükerrer:", sorted(cift)); sys.exit(1)
EOF
adim "senaryo hesapları TDHP'de" python3 - <<'EOF'
import json, sys
kat = json.load(open("data/ufrs-calismalar.json"))
tdhp = {l.split(",")[0] for l in open("data/tdhp.csv", encoding="utf-8").read().splitlines()[1:] if l.strip()}
kotu = []
for c in kat["calismalar"]:
    for s in c.get("senaryolar", []):
        for b in s["bacaklar"]:
            h = b["hesap"]
            if h == "@hedef" or not h[:1].isdigit(): continue
            if h.split(".")[0] not in tdhp: kotu.append(f"{c['id']}/{s['kod']}: {h}")
if kotu: print("\n".join(kotu)); sys.exit(1)
EOF

echo "═══ 3. API + MOTOR (taze süreçle) ═══"
pkill -f "target/debug/api" 2>/dev/null; sleep 1
nohup ./target/debug/api > /tmp/api-tarama.log 2>&1 &
API_PID=$!; sleep 2
adim "temel uçlar 200"          python3 - <<'EOF'
import urllib.request, sys
uclar = ["/api/hesaplar","/api/hesap-plani","/api/mizan","/api/ufrs/calismalar","/api/ufrs/wtb","/api/ufrs/kayitlar"]
kotu = []
for u in uclar:
    try:
        r = urllib.request.urlopen("http://localhost:8787"+u, timeout=5)
        if r.status != 200: kotu.append(f"{u}: {r.status}")
    except Exception as e: kotu.append(f"{u}: {e}")
if kotu: print("\n".join(kotu)); sys.exit(1)
EOF
adim "60 senaryo taraması"      python3 tools/senaryo-tarama.py
adim "kayıt doğrulama (11 vaka)" python3 tools/kayit-dogrulama-testi.py
kill $API_PID 2>/dev/null

echo ""
echo "═══════ SONUÇ: $PASS geçti · $FAIL hata ═══════"
for s in "${SORUNLAR[@]}"; do echo "$s"; done
[ $FAIL -eq 0 ]
