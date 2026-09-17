#!/usr/bin/env bash
# Download every constellation release artifact and CONFIRM it, as far as this
# machine honestly can.
#
# WHAT "CONFIRM" MEANS HERE, stated up front because the levels differ and
# blurring them is how a release gets called verified when it was only fetched:
#
#   linux-x86_64  DOWNLOADED, CHECKSUMMED, EXTRACTED AND RUN. Real execution.
#   everything    DOWNLOADED AND CHECKSUMMED against the release's own
#   else          SHA256SUMS.portable (or SHA256SUMS for mnemonic-engrave).
#                 NOT executed -- this is a Linux box.
#
# The macOS and Windows artifacts ARE executed, but on their own CI runners, by
# the acceptance step in .github/workflows/release.yml: it runs the binary it
# just built and asserts real behaviour (md's Template-ID, ms's 3-of-5
# threshold). That evidence lives in the run log, not here. This script prints
# where to find it rather than pretending to reproduce it.
set -uo pipefail

OUT="${1:-$(mktemp -d)}"
mkdir -p "$OUT"
echo "== downloading into $OUT"

declare -A TAGS=(
  [descriptor-mnemonic]=descriptor-mnemonic-md-cli-v0.15.0
  [mnemonic-secret]=ms-cli-v0.19.0
  [mnemonic-key]=mk-cli-v0.13.0
  [mnemonic-transaction]=mt-cli-v0.1.0
  [mnemonic-engrave]=v0.10.0
)
declare -A BIN=( [descriptor-mnemonic]=md [mnemonic-secret]=ms [mnemonic-key]=mk
                 [mnemonic-transaction]=mt [mnemonic-engrave]=me )

pass=0; fail=0; noexec=0
ok()   { printf '   \033[32mok\033[0m    %s\n' "$1"; pass=$((pass+1)); }
bad()  { printf '   \033[31mFAIL\033[0m  %s\n' "$1"; fail=$((fail+1)); }
note() { printf '   \033[33m--\033[0m    %s\n' "$1"; noexec=$((noexec+1)); }

for repo in "${!TAGS[@]}"; do
  tag="${TAGS[$repo]}"; bin="${BIN[$repo]}"
  echo; echo "── $repo @ $tag"
  d="$OUT/$repo"; mkdir -p "$d"
  if ! gh release download "$tag" --repo "bg002h/$repo" --dir "$d" --clobber 2>/dev/null; then
    bad "could not download the release"; continue
  fi
  n=$(find "$d" -type f | wc -l); echo "   $n asset(s)"

  # --- checksums ---------------------------------------------------------
  sums=""
  for cand in SHA256SUMS.portable SHA256SUMS; do
    [ -f "$d/$cand" ] && { sums="$cand"; break; }
  done
  if [ -z "$sums" ]; then
    bad "no checksum file in the release"
  else
    if (cd "$d" && sha256sum -c "$sums" --ignore-missing >/dev/null 2>&1); then
      ok "$sums verifies ($(grep -c . "$d/$sums") entries)"
    else
      bad "$sums DID NOT VERIFY"
      (cd "$d" && sha256sum -c "$sums" --ignore-missing 2>&1 | grep -v ': OK$' | head -4 | sed 's/^/        /')
    fi
  fi

  # --- run the one we actually can ---------------------------------------
  lin=$(find "$d" -name "*linux*amd64*.tar.gz" -o -name "*x86_64-linux*.tar.gz" | head -1)
  if [ -z "$lin" ]; then
    note "no linux-x86_64 archive to execute"
  else
    ex="$d/x"; rm -rf "$ex"; mkdir -p "$ex"
    tar -xzf "$lin" -C "$ex" 2>/dev/null
    b=$(find "$ex" -type f -name "$bin" | head -1)
    if [ -z "$b" ]; then
      bad "archive has no '$bin' binary: $(basename "$lin")"
    else
      chmod +x "$b"
      v=$("$b" --version 2>&1 | head -1)
      if [ -n "$v" ]; then ok "RAN $(basename "$lin") -> $v"; else bad "'$bin --version' produced nothing"; fi
    fi
  fi

  # --- where the non-Linux execution evidence lives ----------------------
  note "macOS/Windows executed on their own runners; see the release run's acceptance step"
done

echo
printf '%d verified, %d failed, %d not executable here\n' "$pass" "$fail" "$noexec"
[ "$fail" -eq 0 ] || exit 1
