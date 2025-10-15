#!/usr/bin/env bash
set -euo pipefail

# update-agent-context.sh
# Updates agent-specific context files with technology stack from current plan

usage() {
  cat <<EOF
Usage: $0 claude

Updates the agent-specific context file with technology stack information.

Arguments:
  claude - Updates .claude/context.md (Claude Code agent)
EOF
  exit 1
}

if [[ $# -ne 1 ]]; then
  usage
fi

AGENT_TYPE=$1

case $AGENT_TYPE in
  claude)
    CONTEXT_FILE=".claude/context.md"
    ;;
  *)
    echo "Error: Unknown agent type '$AGENT_TYPE'" >&2
    usage
    ;;
esac

# Create context file if not exists
mkdir -p "$(dirname "$CONTEXT_FILE")"

if [[ ! -f "$CONTEXT_FILE" ]]; then
  cat > "$CONTEXT_FILE" <<'CTXEOF'
# Project Context

This file provides context for AI-assisted development.

## Technology Stack

<!-- TECH_STACK_BEGIN -->
<!-- TECH_STACK_END -->

## Manual Notes

Add custom project notes here. They will be preserved during updates.

CTXEOF
fi

# Extract tech stack from research.md
RESEARCH_FILE=".specify/specs/*/artifacts/research.md"
TECH_STACK=""

if ls $RESEARCH_FILE 1> /dev/null 2>&1; then
  TECH_STACK=$(grep -A 50 "## Technology Stack Decisions" $RESEARCH_FILE | head -100 || echo "")
fi

# Update context file between markers
if [[ -n "$TECH_STACK" ]]; then
  # Use awk to replace content between markers
  awk -v tech="$TECH_STACK" '
    /<!-- TECH_STACK_BEGIN -->/ { print; print tech; skip=1; next }
    /<!-- TECH_STACK_END -->/ { skip=0 }
    !skip
  ' "$CONTEXT_FILE" > "${CONTEXT_FILE}.tmp"

  mv "${CONTEXT_FILE}.tmp" "$CONTEXT_FILE"
  echo "Updated $CONTEXT_FILE with technology stack"
else
  echo "Warning: No research.md found, context not updated"
fi
