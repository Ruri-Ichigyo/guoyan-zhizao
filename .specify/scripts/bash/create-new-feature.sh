#!/usr/bin/env bash
set -euo pipefail

# create-new-feature.sh
# Creates a new feature branch and initializes spec directory structure

usage() {
  cat <<EOF
Usage: $0 --json "FEATURE_DESCRIPTION"

Creates a new feature branch and spec directory structure.

Options:
  --json    Feature description (used to generate feature name)

Outputs JSON with BRANCH_NAME and SPEC_FILE paths.
EOF
  exit 1
}

# Parse arguments
JSON_MODE=false
FEATURE_DESC=""

while [[ $# -gt 0 ]]; do
  case $1 in
    --json)
      JSON_MODE=true
      FEATURE_DESC="$2"
      shift 2
      ;;
    *)
      usage
      ;;
  esac
done

if [[ -z "$FEATURE_DESC" ]]; then
  echo "Error: Feature description required" >&2
  exit 1
fi

# Generate feature name from description - use simple default for Chinese text
# For Chinese descriptions, use a standard name
FEATURE_NAME="fruit-vegetable-recognition"

# Try to extract English keywords if present
ENGLISH_KEYWORDS=$(echo "$FEATURE_DESC" | grep -oE '[a-zA-Z]+' | head -3 | tr '\n' '-' | tr '[:upper:]' '[:lower:]' | sed 's/-$//')
if [[ -n "$ENGLISH_KEYWORDS" ]] && [[ ${#ENGLISH_KEYWORDS} -gt 5 ]]; then
  FEATURE_NAME="$ENGLISH_KEYWORDS"
fi

# Sanitize feature name for branch
BRANCH_FEATURE_NAME=$(echo "$FEATURE_NAME" | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9-]/-/g' | sed 's/-\+/-/g' | sed 's/^-//;s/-$//')
if [[ -z "$BRANCH_FEATURE_NAME" ]] || [[ ${#BRANCH_FEATURE_NAME} -lt 3 ]]; then
  BRANCH_FEATURE_NAME="feature-$(date +%Y%m%d-%H%M%S)"
fi

BRANCH_NAME="feature/${BRANCH_FEATURE_NAME}"
FEATURE_DIR=".specify/specs/${BRANCH_FEATURE_NAME}"
SPEC_FILE="${FEATURE_DIR}/spec.md"

# Create feature directory
mkdir -p "$FEATURE_DIR/checklists"

# Create spec file placeholder
cat > "$SPEC_FILE" <<'SPECEOF'
# Feature Specification: [FEATURE_NAME]

**Version**: 0.1.0
**Created**: [CREATION_DATE]
**Status**: Draft

## Overview

[Feature description will be generated]

SPECEOF

# Create or switch to feature branch
CURRENT_BRANCH=$(git branch --show-current)
if git show-ref --verify --quiet "refs/heads/${BRANCH_NAME}"; then
  git checkout "$BRANCH_NAME" 2>/dev/null || true
else
  git checkout -b "$BRANCH_NAME" 2>/dev/null || true
fi

# Output JSON
if [[ "$JSON_MODE" == true ]]; then
  cat <<JSONEOF
{
  "BRANCH_NAME": "$BRANCH_NAME",
  "SPEC_FILE": "$(pwd)/$SPEC_FILE",
  "FEATURE_DIR": "$(pwd)/$FEATURE_DIR",
  "FEATURE_NAME": "$BRANCH_FEATURE_NAME"
}
JSONEOF
else
  echo "Branch: $BRANCH_NAME"
  echo "Spec file: $SPEC_FILE"
fi
