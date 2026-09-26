#!/usr/bin/env bash
# check-install-links.sh -- the SH2 demo page's install block, verified against
# reality: every advertised URL must RESOLVE, and every advertised version must
# be the NEWEST release of its kind (same tag prefix) in its repo.
#
# WHY THIS EXISTS (F-623). Twice on 2026-09-18 the page was wrong about a
# version and a human found it, not a gate:
#   - it advertised `md 0.15.0` for a day after md-cli 0.16.0 shipped, so every
#     visitor installed the binary WITHOUT that day's descriptor fix;
#   - it carried toolkit `v0.100.0` in prose while its own install block said
#     v0.101.0.
# Neither is detectable by reading the page: both were internally consistent.
#
# WHY NOT UNVERSIONED ALIAS ASSETS, which would remove the problem instead of
# detecting it: GitHub's permanently-correct `releases/latest/download/<asset>`
# needs a STABLE asset name, and every archive embeds its version. Adding an
# unversioned alias means editing the release workflow of all three repos --
# and the Linux `musl` archives this page links are produced by a SIGNED,
# REPRODUCIBLE pipeline invoked from `man-pages.yml`, where a second copy of an
# artifact interacts with attestation, per-arch checksums and the repro digest
# contract. Five workflows, two of them in that path, to avoid editing one HTML
# file. The alias remains the better END state and is still filed; this gate is
# what makes the current shape safe TODAY, and it keeps working even if the page
# later gains a fourth tool.
#
#   demo/sh2/check-install-links.sh          # exit 0 = the page is honest
set -euo pipefail
cd "$(dirname "$0")"

PAGE="${1:-src/index.html}"
[ -f "$PAGE" ] || { echo "no page at $PAGE" >&2; exit 2; }

# curl with a token when one is around: unauthenticated api.github.com is
# 60 req/hr per IP, which a busy CI runner shares.
gh_api() {
  if [ -n "${GITHUB_TOKEN:-}" ]; then
    curl -fsSL -H "Authorization: Bearer $GITHUB_TOKEN" "$1"
  else
    curl -fsSL "$1"
  fi
}

fail=0
note() { printf '  %-6s %s\n' "$1" "$2"; }

# ── 1. every advertised archive must resolve ────────────────────────────────
echo "install URLs on $PAGE:"
urls=$(grep -oE 'https://github\.com/[^"< ]+/releases/download/[^"< ]+' "$PAGE" | sort -u)
[ -n "$urls" ] || { echo "REFUSING: found no install URLs to check — the grep or the page changed shape" >&2; exit 2; }

while read -r u; do
  [ -n "$u" ] || continue
  code=$(curl -sIL -o /dev/null -w '%{http_code}' --retry 3 --retry-delay 2 "$u" || echo 000)
  if [ "$code" = "200" ]; then note "$code" "${u##*/}"; else note "$code" "${u##*/}   <-- BROKEN"; fail=1; fi
done <<< "$urls"

# ── 2. every advertised release must BE the newest of its kind ─────────────
# This is the check that would have caught md 0.15.0. A URL that resolves is
# not the same as a URL that is current: 0.15.0's assets still exist.
#
# "Newest of its kind", NOT GitHub's `releases/latest`: a repo publishes more
# than one kind of release (mnemonic-toolkit also tags manual-gui-v*, manual-v*,
# tech-manual-v*, ...), and GitHub marks whichever was created last as Latest.
# On 2026-09-25 tagging manual-gui-v1.4.0 made this check call the toolkit
# CLI "STALE" against a MANUAL release. So compare within the tag's own prefix
# (everything before the version: `mnemonic-toolkit-v`, `ms-cli-v`,
# `descriptor-mnemonic-md-cli-v`), over published, non-prerelease releases.
echo
echo "advertised tag vs newest release of the same kind:"
tags=$(printf '%s\n' "$urls" | sed -E 's#https://github\.com/([^/]+/[^/]+)/releases/download/([^/]+)/.*#\1 \2#' | sort -u)
while read -r repo tag; do
  [ -n "$repo" ] || continue
  prefix=$(printf '%s' "$tag" | sed -E 's/[0-9]+(\.[0-9]+)*$//')
  if [ -z "$prefix" ] || [ "$prefix" = "$tag" ]; then
    note "??" "$repo — cannot split '$tag' into prefix + version"; fail=1; continue
  fi
  # Capture, then filter: no early-exiting reader under pipefail (F-695).
  rel_json=$(gh_api "https://api.github.com/repos/$repo/releases?per_page=100") || rel_json=""
  newest=$(jq -r --arg p "$prefix" \
             '.[] | select(.draft|not) | select(.prerelease|not) | .tag_name | select(startswith($p))' \
             <<<"$rel_json" 2>/dev/null | sort -V | tail -n 1)
  if [ -z "$newest" ]; then
    note "??" "$repo — no published '${prefix}*' release found"; fail=1
  elif [ "$newest" = "$tag" ]; then
    note "ok" "$repo $tag"
  else
    note "STALE" "$repo advertises $tag, newest ${prefix}* is $newest"; fail=1
  fi
done <<< "$tags"

echo
if [ "$fail" -ne 0 ]; then
  cat >&2 <<'EOF'
FAIL: the install block is out of date or broken.

  Update demo/sh2/src/index.html (and CLI_SPINE.md), rebuild with
  demo/sh2/build.sh, and redeploy. Version numbers belong ONLY in the install
  commands -- the prose must not name one (operator directive 2026-09-18), so
  a bump is confined to the curl/tar lines.
EOF
  exit 1
fi
echo "OK: every advertised URL resolves and names the newest release of its kind."
