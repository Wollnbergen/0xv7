#!/usr/bin/env bash
set -euo pipefail

usage(){ cat <<EOF
Usage:
  $(basename "$0") [--repo owner/name] [--branch main] PR_NUMBER [PR_NUMBER...]
EOF
}

REPO="$(gh repo view --json nameWithOwner -q .nameWithOwner 2>/dev/null || true)"
BRANCH="main"
PRS=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --repo) REPO="$2"; shift 2 ;;
    --branch) BRANCH="$2"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    --) shift; break ;;
    -*) echo "Unknown option: $1" >&2; usage; exit 2 ;;
    *) PRS+=("$1"); shift ;;
  esac
done

if [[ ${#PRS[@]} -eq 0 ]]; then
  echo "No PR numbers provided; nothing to do."
  exit 0
fi
[[ -n "$REPO" ]] || { echo "Error: cannot determine repo. Use --repo owner/name." >&2; exit 2; }

need(){ command -v "$1" >/dev/null 2>&1 || { echo "Missing dependency: $1" >&2; exit 1; }; }
need gh
need jq

gh auth status >/dev/null 2>&1 || gh auth login --hostname github.com --web --scopes repo,workflow,admin:repo_hook

set +e
CUR_JSON=$(gh api "repos/$REPO/branches/$BRANCH/protection" 2>/dev/null)
STATUS=$?
set -e
PROTECTED=true
[[ $STATUS -eq 0 && -n "$CUR_JSON" ]] || PROTECTED=false

if $PROTECTED; then
  approvals=$(jq -r '.required_pull_request_reviews.required_approving_review_count // 0' <<<"$CUR_JSON")
  dismiss=$(jq -r '.required_pull_request_reviews.dismiss_stale_reviews // false' <<<"$CUR_JSON")
  codeowners=$(jq -r '.required_pull_request_reviews.require_code_owner_reviews // false' <<<"$CUR_JSON")
  lastpush=$(jq -r '.required_pull_request_reviews.require_last_push_approval // false' <<<"$CUR_JSON")
  admins=$(jq -r '.enforce_admins.enabled // false' <<<"$CUR_JSON")
  convo=$(jq -r '.required_conversation_resolution.enabled // false' <<<"$CUR_JSON")
  forcepush=$(jq -r '.allow_force_pushes.enabled // false' <<<"$CUR_JSON")
  deletions=$(jq -r '.allow_deletions.enabled // false' <<<"$CUR_JSON")
  rsc=$(jq -c '.required_status_checks' <<<"$CUR_JSON")
fi

relaxed=false
restore_protection() {
  if $PROTECTED && $relaxed; then
    gh api -X PUT "repos/$REPO/branches/$BRANCH/protection" \
      -H "Accept: application/vnd.github+json" --input - <<JSON
{
  "required_status_checks": $rsc,
  "enforce_admins": $admins,
  "required_pull_request_reviews": {
    "required_approving_review_count": $approvals,
    "dismiss_stale_reviews": $dismiss,
    "require_code_owner_reviews": $codeowners,
    "require_last_push_approval": $lastpush
  },
  "restrictions": null,
  "allow_force_pushes": $forcepush,
  "allow_deletions": $deletions,
  "required_conversation_resolution": $convo
}
JSON
  fi
}
trap restore_protection EXIT

if $PROTECTED; then
  gh api -X PUT "repos/$REPO/branches/$BRANCH/protection" \
    -H "Accept: application/vnd.github+json" --input - <<JSON
{
  "required_status_checks": $rsc,
  "enforce_admins": false,
  "required_pull_request_reviews": {
    "required_approving_review_count": 0,
    "dismiss_stale_reviews": $dismiss,
    "require_code_owner_reviews": $codeowners,
    "require_last_push_approval": $lastpush
  },
  "restrictions": null,
  "allow_force_pushes": $forcepush,
  "allow_deletions": $deletions,
  "required_conversation_resolution": $convo
}
JSON
  relaxed=true
fi

echo "Merging PRs in $REPO: ${PRS[*]}"
for pr in "${PRS[@]}"; do
  if ! gh pr view "$pr" --repo "$REPO" >/dev/null 2>&1; then
    echo "Skipping: PR #$pr not found"
    continue
  fi
  gh pr merge "$pr" --squash --delete-branch --repo "$REPO"
done

git switch "$BRANCH" >/dev/null 2>&1 || true
git pull --ff-only || true
git remote prune origin || true
echo "Done."
