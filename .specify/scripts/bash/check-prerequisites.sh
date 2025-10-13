#!/usr/bin/env bash
set -euo pipefail

# check-prerequisites.sh
# Validates feature directory and lists available design documents

usage() {
  cat <<EOF
Usage: $0 [--json] [--require-tasks] [--include-tasks]

Checks prerequisites for task generation and lists available design documents.

Options:
  --json           Output in JSON format
  --require-tasks  Fail if tasks.md doesn't exist
  --include-tasks  Include tasks.md in AVAILABLE_DOCS if present

Outputs JSON with FEATURE_DIR and AVAILABLE_DOCS list.
EOF
  exit 1
}

# Parse arguments
JSON_MODE=false
REQUIRE_TASKS=false
INCLUDE_TASKS=false

while [[ $# -gt 0 ]]; do
  case $1 in
    --json)
      JSON_MODE=true
      shift
      ;;
    --require-tasks)
      REQUIRE_TASKS=true
      shift
      ;;
    --include-tasks)
      INCLUDE_TASKS=true
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
FEATURE_NAME=$(echo "$BRANCH" | sed 's|^feature/||')

if [[ -z "$FEATURE_NAME" ]] || [[ "$FEATURE_NAME" == "$BRANCH" ]]; then
  echo "Error: Not on a feature branch (expected feature/*)" >&2
  exit 1
fi

# Set paths
FEATURE_DIR=".specify/specs/${FEATURE_NAME}"

if [[ ! -d "$FEATURE_DIR" ]]; then
  echo "Error: Feature directory not found at $FEATURE_DIR" >&2
  exit 1
fi

# Check for available documents
AVAILABLE_DOCS=()

# Required documents
[[ -f "$FEATURE_DIR/plan.md" ]] && AVAILABLE_DOCS+=("plan.md")
[[ -f "$FEATURE_DIR/spec.md" ]] && AVAILABLE_DOCS+=("spec.md")

# Optional documents
[[ -f "$FEATURE_DIR/artifacts/data-model.md" ]] && AVAILABLE_DOCS+=("artifacts/data-model.md")
[[ -f "$FEATURE_DIR/artifacts/research.md" ]] && AVAILABLE_DOCS+=("artifacts/research.md")
[[ -f "$FEATURE_DIR/artifacts/quickstart.md" ]] && AVAILABLE_DOCS+=("artifacts/quickstart.md")
[[ -d "$FEATURE_DIR/contracts" ]] && AVAILABLE_DOCS+=("contracts/")

# Check for tasks.md
if [[ "$INCLUDE_TASKS" == true ]] && [[ -f "$FEATURE_DIR/tasks.md" ]]; then
  AVAILABLE_DOCS+=("tasks.md")
elif [[ "$REQUIRE_TASKS" == true ]] && [[ ! -f "$FEATURE_DIR/tasks.md" ]]; then
  echo "Error: tasks.md not found at $FEATURE_DIR/tasks.md" >&2
  echo "Run /speckit.tasks first to generate task breakdown" >&2
  exit 1
fi

# Output JSON
if [[ "$JSON_MODE" == true ]]; then
  DOCS_JSON=$(printf '%s\n' "${AVAILABLE_DOCS[@]}" | jq -R . | jq -s .)

  cat <<JSONEOF
{
  "FEATURE_DIR": "$(pwd)/$FEATURE_DIR",
  "FEATURE_NAME": "$FEATURE_NAME",
  "AVAILABLE_DOCS": $DOCS_JSON
}
JSONEOF
else
  echo "Feature directory: $FEATURE_DIR"
  echo "Available documents:"
  printf '  - %s\n' "${AVAILABLE_DOCS[@]}"
fi
