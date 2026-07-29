#!/bin/bash
# post-write-token-diff.sh
# EVENT: PostToolUse
# DESCRIPTION: Log token cost of each Write/Edit to .claude/sessions/write-log.md
#
# Claude Code PostToolUse hook: logs token cost of each Write/Edit operation.
# Appends to .claude/sessions/write-log.md so you can see which files are
# growing your context window.
#
# INSTALL: cto hooks install post-write-token-diff
# Or manually: copy to .claude/hooks/post-write-token-diff.sh
#
# CONFIGURE (optional):
#   CTO_WRITE_ADVISORY_TOKENS — cumulative threshold for advisory (default: 5000)

LOG_FILE=".claude/sessions/write-log.md"
ADVISORY_THRESHOLD="${CTO_WRITE_ADVISORY_TOKENS:-5000}"
DATE=$(date +%Y-%m-%d)
TIME=$(date +%H:%M)

# Capture stdin once; derive tool_name from it (env var is not guaranteed)
STDIN_JSON=$(cat)
TOOL_NAME="${CLAUDE_TOOL_NAME:-$(printf '%s' "$STDIN_JSON" | python3 -c "import sys,json;
try: print(json.load(sys.stdin).get('tool_name',''))
except: pass" 2>/dev/null)}"
if [ "$TOOL_NAME" != "Write" ] && [ "$TOOL_NAME" != "Edit" ]; then
  exit 0
fi

# Get file path from the captured stdin JSON
FILE_PATH=$(printf '%s' "$STDIN_JSON" | python3 -c "
import sys, json
try:
    data = json.load(sys.stdin)
    path = data.get('tool_input', {}).get('file_path', '')
    print(path)
except:
    pass
" 2>/dev/null)

if [ -z "$FILE_PATH" ] || [ ! -f "$FILE_PATH" ]; then
  exit 0
fi

# Token estimate: word count × 1.3 (fast, no node startup)
WORD_COUNT=$(wc -w < "$FILE_PATH" 2>/dev/null || echo "0")
FILE_TOKENS=$(echo "$WORD_COUNT * 13 / 10" | bc 2>/dev/null || echo "0")

# Init log file if needed
mkdir -p "$(dirname "$LOG_FILE")"
if [ ! -f "$LOG_FILE" ]; then
  printf '# Write Token Log\n\n| Date | Time | Tool | File | Est. Tokens |\n|------|------|------|------|-------------|\n' > "$LOG_FILE"
fi

echo "| $DATE | $TIME | $TOOL_NAME | \`$FILE_PATH\` | ~${FILE_TOKENS} |" >> "$LOG_FILE"

# Cumulative advisory: sum the last column of the log
CUMULATIVE=$(awk -F'~' 'NR>2 && NF>1 {gsub(/ \|.*/,"",$NF); sum += $NF} END {print sum+0}' "$LOG_FILE" 2>/dev/null || echo "0")

if [ "$CUMULATIVE" -ge "$ADVISORY_THRESHOLD" ] 2>/dev/null; then
  echo "📝 Write log: ~${CUMULATIVE} tokens written this session (across $(grep -c '|' "$LOG_FILE" 2>/dev/null || echo "?") files)" >&2
  echo "   View full log: cat ${LOG_FILE}" >&2
fi

exit 0
