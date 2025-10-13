#!/usr/bin/env bash
set -euo pipefail

# setup-plan.sh
# Prepares implementation plan directory structure and copies template

usage() {
  cat <<EOF
Usage: $0 --json

Sets up implementation planning infrastructure for the current feature branch.

Outputs JSON with FEATURE_SPEC, IMPL_PLAN, SPECS_DIR, BRANCH paths.
EOF
  exit 1
}

# Parse arguments
JSON_MODE=false

while [[ $# -gt 0 ]]; do
  case $1 in
    --json)
      JSON_MODE=true
      shift
      ;;
    *)
      usage
      ;;
  esac
done

# Get current branch
BRANCH=$(git branch --show-current)

# Derive feature name from branch
FEATURE_NAME=$(echo "$BRANCH" | sed 's|^feature/||' | sed 's|^||')

if [[ -z "$FEATURE_NAME" ]] || [[ "$FEATURE_NAME" == "$BRANCH" ]]; then
  echo "Error: Not on a feature branch (expected feature/*)" >&2
  exit 1
fi

# Set paths
SPECS_DIR=".specify/specs/${FEATURE_NAME}"
FEATURE_SPEC="${SPECS_DIR}/spec.md"
IMPL_PLAN="${SPECS_DIR}/plan.md"
PLAN_TEMPLATE=".specify/templates/plan-template.md"

# Validate spec exists
if [[ ! -f "$FEATURE_SPEC" ]]; then
  echo "Error: Spec file not found at $FEATURE_SPEC" >&2
  echo "Run /speckit.specify first" >&2
  exit 1
fi

# Create planning directories
mkdir -p "${SPECS_DIR}"/{artifacts,contracts,checklists}

# Copy plan template if not exists
if [[ ! -f "$IMPL_PLAN" ]]; then
  if [[ -f "$PLAN_TEMPLATE" ]]; then
    cp "$PLAN_TEMPLATE" "$IMPL_PLAN"
  else
    # Create minimal plan if template missing
    cat > "$IMPL_PLAN" <<'PLANEOF'
# Implementation Plan: [FEATURE_NAME]

**Specification Reference**: `spec.md`
**Plan Version**: 0.1.0
**Created**: [DATE]
**Status**: Draft

## Planning sections to be filled...

PLANEOF
  fi
fi

# Output JSON
if [[ "$JSON_MODE" == true ]]; then
  cat <<JSONEOF
{
  "BRANCH": "$BRANCH",
  "FEATURE_NAME": "$FEATURE_NAME",
  "SPECS_DIR": "$(pwd)/$SPECS_DIR",
  "FEATURE_SPEC": "$(pwd)/$FEATURE_SPEC",
  "IMPL_PLAN": "$(pwd)/$IMPL_PLAN"
}
JSONEOF
else
  echo "Branch: $BRANCH"
  echo "Spec: $FEATURE_SPEC"
  echo "Plan: $IMPL_PLAN"
fi
