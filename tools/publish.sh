#!/usr/bin/env bash
# Pushes verified Rust code and live progress JSON (`docs/progress.json`, `docs/worker.json`)
# to GitHub `main` using a repo-scoped deploy key.
#
# `docs/index.html` fetches `progress.json` and `worker.json` directly from
# `raw.githubusercontent.com`, so pushing them updates the live GitHub Pages dashboard
# automatically without triggering a Pages redeploy.

set -euo pipefail

REPO="${MC_RUST_REPO:-/opt/mc-rust/MinecraftRustServer}"
KEY="${MC_RUST_DEPLOY_KEY:-$HOME/.ssh/mc_rust_deploy}"
LOG="${MC_RUST_PUBLISH_LOG:-$HOME/publish.log}"
HEARTBEAT_STAMP="/tmp/mc-rust-last-heartbeat-push"

export GIT_SSH_COMMAND="ssh -i $KEY -o IdentitiesOnly=yes -o BatchMode=yes -o StrictHostKeyChecking=accept-new"

exec 9>/tmp/mc-rust-publish.lock
flock -n 9 || exit 0

mkdir -p "$(dirname "$LOG")"
if [ -f "$LOG" ] && [ "$(stat -c %s "$LOG")" -gt 2000000 ]; then
    : > "$LOG"
fi
exec >> "$LOG" 2>&1

echo "--- $(date -u +%Y-%m-%dT%H:%M:%SZ) ---"
cd "$REPO"

# Always regenerate progress.json cleanly before checking diff (never commits reference/ or keys)
python3 tools/update_progress.py >/dev/null || true

# Stage only public code/docs/progress artifacts (explicit path allowlist)
git add -A crates docs/progress.json docs/worker.json MAPPING.md PROGRESS.md COVERAGE.md data/mappings_summary.json 2>/dev/null || true

if git diff --cached --quiet; then
    # Nothing staged; check if local commits are ahead of origin/main
    ahead="$(git rev-list --count origin/main..HEAD 2>/dev/null || echo 0)"
    if [ "$ahead" -eq 0 ]; then
        echo "no changes"
        exit 0
    fi
else
    changed_files="$(git diff --cached --name-only)"
    if [ "$changed_files" = "docs/worker.json" ] || [ "$changed_files" = $'docs/progress.json\ndocs/worker.json' ]; then
        now="$(date +%s)"
        last="$(cat "$HEARTBEAT_STAMP" 2>/dev/null || echo 0)"
        if [ $((now - last)) -lt 300 ]; then
            git reset -q
            echo "heartbeat only; throttled"
            exit 0
        fi
        echo "$now" > "$HEARTBEAT_STAMP"
    fi
    mapped="$(python3 -c "import json;print(json.load(open('docs/progress.json'))['totals']['mapped_methods'])" 2>/dev/null || echo "?")"
    git -c user.name="mc-rust-worker" -c user.email="harness@mc-rust.local" \
        commit -q -m "Worker sync: ${mapped} mapped Java methods"
fi

git fetch -q origin main || true
if [ "$(git rev-list --count HEAD..origin/main 2>/dev/null || echo 0)" -gt 0 ]; then
    git pull --rebase --autostash -q origin main || true
fi
if git push -q origin main; then
    echo "pushed to origin/main"
else
    echo "push failed; will retry on next tick"
fi
