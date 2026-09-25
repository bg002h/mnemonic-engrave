#!/usr/bin/env bash
# F-695 enumeration: early-exiting readers on the right of a pipe, in the
# constellation's own shell, at origin's default branch of each repo.
# Scope: scripts/, ci/, demo/, Makefiles, .github/workflows/*.yml, plus
# engrave's extensionless scripts/sh2-flash. seedhammer: only files the fork
# added or modified relative to upstream/main.
# Two passes: (1) reader on the same line as the pipe; (2) reader at the start
# of a line whose previous line ends in '|' or '| \' (multi-line pipelines).
set -euo pipefail
cd /scratch/code/shibboleth
READER='(grep([[:space:]]+-[-A-Za-z0-9]+)*[[:space:]]+(-[A-Za-z]*[qml][A-Za-z0-9]*|--quiet|--silent|--max-count[= ]?[0-9]*|--files-with-matches)([^-A-Za-z0-9_]|$)|head([^-A-Za-z0-9_]|$)|sed[[:space:]]+(-n[[:space:]]+)?.?[0-9]*q|awk[^|]*[^A-Za-z0-9_]exit([^A-Za-z0-9_]|$)|(IFS=[^ ]* )?read([^A-Za-z0-9_]|$))'
for r in mnemonic-engrave descriptor-mnemonic mnemonic-secret mnemonic-key mnemonic-toolkit mnemonic-gui seedhammer; do
  ref=$(git -C "$r" symbolic-ref --short refs/remotes/origin/HEAD)
  if [ -n "${F695_AFTER:-}" ] && git -C "$r" rev-parse -q --verify "$F695_AFTER" >/dev/null; then ref=$F695_AFTER; fi
  if [ "$r" = seedhammer ]; then
    files=$(git -C "$r" diff --name-only --diff-filter=AM upstream/main "$ref")
  else
    files=$(git -C "$r" ls-tree -r --name-only "$ref")
  fi
  sel=$(grep -E '^(.*/)?(scripts|ci|demo)/|^cmd/.*\.sh$|(^|/)(GNU)?[Mm]akefile|\.mk$|^\.github/workflows/.*\.ya?ml$' <<<"$files" \
        | grep -vE '(^|/)(vendor|third_party|target|node_modules)/|^design/' || true)
  [ -n "$sel" ] || continue
  while IFS= read -r f; do
    content=$(git -C "$r" show "$ref:$f") || continue
    awk -v R="$READER" -v F="$r/$f" '
      { line=$0 }
      line ~ ("\\|[[:space:]]*" R) { print F ":" NR ":" line; prev=line; next }
      (prev ~ /\|[[:space:]]*\\?[[:space:]]*$/) && (line ~ ("^[[:space:]]*" R)) { print F ":" NR ":" line " [continues line " NR-1 "]" }
      { prev=line }' <<<"$content"
  done <<<"$sel"
done
