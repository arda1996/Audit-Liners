#!/bin/bash
# stop-session-snapshot.sh
# EVENT: Stop
# DESCRIPTION: Write a session snapshot to .claude/sessions/snapshot.md after each turn
#
# Claude Code Stop hook: captures changed files, recent commits, and token
# estimate into a compact snapshot. The snapshot is injected at the start of
# the next session by user-prompt-inject-snapshot.sh (or via CLAUDE.md @ import).
#
# INSTALL: cto hooks install stop-session-snapshot
# Or manually: copy to .claude/hooks/stop-session-snapshot.sh

SNAPSHOT=".claude/sessions/snapshot.md"
DATE=$(date +"%Y-%m-%d %H:%M")

mkdir -p ".claude/sessions"

# --- Changed files (unstaged + staged + last commit) ---
CHANGED_FILES=""
if git rev-parse --git-dir >/dev/null 2>&1; then
  # Uncommitted changes
  UNSTAGED=$(git diff --name-only 2>/dev/null)
  STAGED=$(git diff --cached --name-only 2>/dev/null)
  # Files changed in last commit
  LAST_COMMIT=$(git diff --name-only HEAD~1 2>/dev/null)

  ALL_CHANGED=$(printf '%s\n%s\n%s\n' "$UNSTAGED" "$STAGED" "$LAST_COMMIT" \
    | sort -u | grep -v '^$' | head -10)

  if [ -n "$ALL_CHANGED" ]; then
    CHANGED_FILES=$(echo "$ALL_CHANGED" | awk '{print "- " $0}')
  fi
fi

# --- Recent commits ---
GIT_LOG=""
if git rev-parse --git-dir >/dev/null 2>&1; then
  GIT_LOG=$(git log --oneline -5 2>/dev/null | head -5)
fi

# --- Token estimate for auto-loaded files ---
WORD_COUNT=$(find . -maxdepth 3 \
  \( -name "*.md" -path "./.claude/*.md" -o -path "./CLAUDE.md" -o -path "./docs/INDEX.md" \) \
  -not -path "./.claude/completions/*" \
  -not -path "./.claude/sessions/*" \
  2>/dev/null | xargs wc -w 2>/dev/null | tail -1 | awk '{print $1}')
WORD_COUNT="${WORD_COUNT:-0}"
APPROX_TOKENS=$(echo "$WORD_COUNT * 13 / 10" | bc 2>/dev/null || echo "?")

# --- Last task context from transcript ---
LAST_TASK=""
STDIN_JSON=$(cat)
TRANSCRIPT=$(echo "$STDIN_JSON" | python3 -c "
import sys, json
try:
    data = json.load(sys.stdin)
    print(data.get('transcript_path', ''))
except:
    pass
" 2>/dev/null)

LAST_USER=""
if [ -n "$TRANSCRIPT" ] && [ -f "$TRANSCRIPT" ]; then
  # Extract last user request + last assistant summary (first + last meaningful line)
  EXTRACT=$(python3 - "$TRANSCRIPT" <<'PYEOF'
import sys, json

transcript_path = sys.argv[1]
last_assistant = ""
last_user = ""
try:
    with open(transcript_path) as f:
        for line in f:
            try:
                entry = json.loads(line)
                t = entry.get('type')
                content = entry.get('message', {}).get('content', [])
                if t == 'assistant':
                    for block in content:
                        if isinstance(block, dict) and block.get('type') == 'text' and block.get('text', '').strip():
                            last_assistant = block['text']
                elif t == 'user':
                    # user content can be a string or list of blocks
                    if isinstance(content, str):
                        txt = content
                    else:
                        txt = " ".join(b.get('text', '') for b in content
                                       if isinstance(b, dict) and b.get('type') == 'text')
                    # skip tool_result-only / hook-injected / continuation-summary turns
                    s = txt.strip()
                    skip = (not s
                            or s.startswith('<')
                            or s.startswith('Caveat:')
                            or s.startswith('This session is being continued')
                            or s.startswith('--- '))
                    if not skip:
                        last_user = txt
            except:
                continue
except:
    pass

def clean(s):
    return [l.strip() for l in s.strip().splitlines() if l.strip()]

au = clean(last_assistant)
# assistant: lead line (the TL;DR) is most load-bearing
summary = au[0][:200] if au else ""
uu = clean(last_user)
req = uu[0][:200] if uu else ""
# emit with a delimiter so bash can split
print("REQ::" + req)
print("SUM::" + summary)
PYEOF
)
  LAST_USER=$(printf '%s\n' "$EXTRACT" | sed -n 's/^REQ:://p')
  LAST_TASK=$(printf '%s\n' "$EXTRACT" | sed -n 's/^SUM:://p')
fi

# --- Write snapshot ---
{
  echo "# Session Snapshot — ${DATE}"
  echo ""

  if [ -n "$CHANGED_FILES" ]; then
    echo "## Files Changed"
    echo "$CHANGED_FILES"
    echo ""
  fi

  if [ -n "$GIT_LOG" ]; then
    echo "## Recent Commits"
    echo "$GIT_LOG" | awk '{print "- " $0}'
    echo ""
  fi

  echo "## Token Estimate"
  echo "~${APPROX_TOKENS} tokens in auto-loaded files"
  echo ""

  if [ -n "$LAST_USER" ]; then
    echo "## Last Request"
    echo "$LAST_USER"
    echo ""
  fi

  if [ -n "$LAST_TASK" ]; then
    echo "## Last Turn"
    echo "$LAST_TASK"
    echo ""
  fi

  echo "## Continuity"
  echo "Bağlam koparsa önce oku: .claude/CONTEXT.md (kümülatif beyin) + .claude/sessions/journal.md (tur günlüğü)"
} > "$SNAPSHOT"

# --- Append to cumulative journal (append-only, never overwritten) ---
JOURNAL=".claude/sessions/journal.md"
if [ ! -f "$JOURNAL" ]; then
  {
    echo "# Karar Günlüğü (salt-ekleme) — her tur bir satır"
    echo ""
    echo "> Otomatik. Kümülatif özet için .claude/CONTEXT.md'yi elle güncelle."
    echo ""
  } > "$JOURNAL"
fi
if [ -n "$LAST_USER" ] || [ -n "$LAST_TASK" ]; then
  # de-dup: skip if this exact assistant summary was the last one logged
  LAST_LOGGED=$(grep -m1 '' /dev/null 2>/dev/null; tail -1 "$JOURNAL" 2>/dev/null)
  ENTRY="- **${DATE}** — istek: ${LAST_USER:-—} · sonuç: ${LAST_TASK:-—}"
  if [ "$ENTRY" != "$LAST_LOGGED" ]; then
    printf '%s\n' "$ENTRY" >> "$JOURNAL"
  fi
fi

exit 0
