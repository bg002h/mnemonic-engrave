#!/usr/bin/env bash
# gen-tx-journey.sh -- regenerate design/JOURNEY_engrave_transaction.md.
#
# NOTHING IN THE JOURNEY IS ILLUSTRATIVE. Every host block below is this
# script's real stdout+stderr with its actual exit code, and every device screen
# is captured by gui.TestCaptureTransactionJourney -- which is the WALK,
# instrumented. The document therefore cannot drift from the tested flow: a
# screen that changes fails the walk before it reaches this file.
#
# WHAT IT DOES NOT COVER, per house practice:
#   - the device screens are op.Drawer.ExtractText over the firmware's own op
#     tree, NOT the emulator's 480x320 framebuffer the way design/journeys/*.pdf
#     are. Same tree, same text, different renderer: this shows WHAT the device
#     says and cannot show how it LOOKS. The framebuffer capture needs a WASM
#     build and playwright and belongs with the P4 hardware session, where a
#     photograph of real steel goes beside it.
#   - no plate is cut. G-P4.1 owns the steel.
#
# Usage: scripts/gen-tx-journey.sh [path-to-go]
set -euo pipefail

ME_REPO="$(cd "$(dirname "$0")/.." && pwd)"
FORK="${FORK:-$ME_REPO/../seedhammer}"
GO="${1:-${GO:-go}}"
command -v "$GO" >/dev/null || { echo "go not found: $GO" >&2; exit 2; }
[ -d "$FORK/gui" ] || { echo "fork not found at $FORK (set FORK=)" >&2; exit 2; }

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
OUT="$ME_REPO/design/JOURNEY_engrave_transaction.md"
DATA="$ME_REPO/design/journeys/transaction"
mkdir -p "$DATA"

echo "=== building me ==="
( cd "$ME_REPO" && cargo build --locked --quiet )
ME="$ME_REPO/target/debug/me"

# The pinned "even" vector: a real signed 222-byte 1-in/2-out P2WPKH
# transaction, txid from the node that made it. Same artifact the Rust, Go and
# gui suites all pin, so every layer of this journey is the same transaction.
EVEN_HEX="020000000001017c8da925af70e49a12b0cea7b639df5037c87b7fa61f262b86ac32c47aa3ba1a0000000000fdffffff02404b4c0000000000160014c1de0dd435d1d4ad97ed1f51d63f91c800cc4eab3ea1b92901000000160014751097c299d6354fbb2c5a84512dd708f2902f5e0247304402207debc7d89984c7717940b622504318d2c184966a618b32cf8b700d0f125b3ffa02206ef875f9c0b5931e0ea1cf0c109bdb8512835c8e51526f99b3419929a2ea7259012103718f5fd45b926226357e2b0400574b41a32d0bf0ae69a02eebea5fbc542ff52060000000"
printf '%s\n' "$EVEN_HEX" > "$WORK/tx.hex"

HOST="$WORK/host.md"
: > "$HOST"

run() {   # run <label> <shell-command...>
  local label="$1"; shift
  { printf '\n### %s\n\n```console\n$ %s\n' "$label" "$*"; } >> "$HOST"
  set +e
  ( eval "$@" ) > "$WORK/o" 2> "$WORK/e"
  local code=$?
  set -e
  # stdout is the artifact and may be binary; say so rather than paste it.
  if [ -s "$WORK/o" ]; then
    if LC_ALL=C grep -qP '[\x00-\x08\x0e-\x1f]' "$WORK/o" 2>/dev/null; then
      printf '<%s bytes of binary on stdout>\n' "$(wc -c < "$WORK/o")" >> "$HOST"
    else
      cat "$WORK/o" >> "$HOST"
    fi
  fi
  cat "$WORK/e" >> "$HOST"
  printf '(exit %d)\n```\n' "$code" >> "$HOST"
}

# sanitise makes the transcript STABLE and non-leaky:
#   - the mktemp dir and the build path become `work/` and `me`, so a
#     regeneration diffs only where behaviour changed rather than everywhere;
#   - a raw transaction echoed as part of a COMMAND LINE is elided. It is a
#     pinned public test vector and not a secret, but a document that
#     demonstrates the bearer-on-argv refusal should not itself paste 222 bytes
#     of bearer material into a committed file. The refusal's own message is
#     the point, and it names no body.
sanitise() {
  sed -e "s#$WORK#work#g" -e "s#$ME_REPO/target/debug/me#me#g" -e "s#'me'#me#g" \
      -e "s#0200000000010[0-9a-f]\{100,\}#<the 222-byte transaction, elided>#g" \
      -e "s#020000000[0-9a-f]\{100,\}#<the 113-byte stripped transaction, elided>#g" \
      "$1"
}

echo "=== capturing the host transcript ==="
run "The operator has a finalized transaction, as hex" \
    "head -c 96 '$WORK/tx.hex'; echo '...'"
run "Turn it into a record. argv is refused for this class, so it is a pipe" \
    "'$ME' tx --in '$WORK/tx.hex' > '$WORK/rec.txt'; head -c 40 '$WORK/rec.txt'; echo '...'"
run "The refusal that makes it a pipe" \
    "'$ME' sysw pack --no-passphrase 'tx:$EVEN_HEX'"
run "Pack it. Nothing here is secret, so nothing is sealed" \
    "'$ME' sysw pack --region --out '$WORK/region.bin' --in '$WORK/rec.txt'"
run "What is in it, and the digest to compare on the machine" \
    "'$ME' sysw show '$WORK/region.bin'"
run "A set missing three of its six strings still packs, loudly" \
    "printf '%s\n%s\n%s\n' \
      mt1p9h8jqq9qqqqgqqqqqqqyqherdfykhhpey6z2cvafak8804qd7g0dl6v8ex9wr2cvky023skwkeud2229sax \
      mt1p9h8jqq9qqzj8yqpnzw4vl2rwffqyqqqqqkqq282yyhc2vavd20hvk94pz39hts3u5s9a0qd8pwskxfl7ju5 \
      mt1p9h8jqq9qq9qdcc7h75twfxyf340c4sgqzhfdq6xtgt7zhxngpwa049l0z59l6jqcqqqqqq5k5y2ye5nv8yf \
      | '$ME' sysw pack --no-passphrase --out '$WORK/partial.bin'"
run "An unsigned transaction is refused, and it names the input" \
    "printf 'tx:02000000017c8da925af70e49a12b0cea7b639df5037c87b7fa61f262b86ac32c47aa3ba1a0000000000fdffffff02404b4c0000000000160014c1de0dd435d1d4ad97ed1f51d63f91c800cc4eab3ea1b92901000000160014751097c299d6354fbb2c5a84512dd708f2902f5e60000000\n' | '$ME' sysw pack --no-passphrase --out /dev/null"

echo "=== capturing the device screens ==="
TX_JOURNEY_OUT="$WORK/frames.md" "$GO" test -C "$FORK" ./gui/ \
  -run TestCaptureTransactionJourney -count=1 >/dev/null

sanitise "$HOST" > "$DATA/host.md"
cp "$WORK/frames.md" "$DATA/frames.md"

echo "=== assembling $OUT ==="
{
  cat <<'HEAD'
# JOURNEY — engrave a transaction

**Regenerate with `scripts/gen-tx-journey.sh`.** Nothing below is illustrative:
every host block is that script's real stdout+stderr with its actual exit code,
and every device screen is captured by `gui.TestCaptureTransactionJourney`,
which is the **walk, instrumented** — the same harness driving the same flow
that `TestWalkQRPathFromATxRecordToThePostCutScreen` asserts on. A screen that
changes fails the walk before it reaches this document.

**Two things it deliberately does not do**, so nobody reads more into it than is
here:

- The device screens are `op.Drawer.ExtractText` over the firmware's own op
  tree, **not** the emulator's 480×320 framebuffer the way
  `design/journeys/*.pdf` are. Same tree, same text, different renderer: this
  shows **what** the device says and cannot show how it **looks**. The
  framebuffer capture needs a WASM build and playwright, and it belongs with the
  P4 hardware session — where a photograph of real steel goes beside it.
- **No plate is cut.** G-P4.1 owns the steel, and until it runs no QR this
  feature emits has ever been read off metal.

The transaction throughout is the pinned `even` vector: a real signed 222-byte
1-in/2-out P2WPKH transaction, txid `2dcf2b97…`, the same artifact the Rust, Go
and `gui` suites all pin — so every layer of this journey is one transaction.

---

## Part 1 — the host
HEAD
  cat "$DATA/host.md"
  cat <<'MID'

---

## Part 2 — the device

The payload is written to `0x10D00000` with `picotool`. At boot the machine
offers to load it, the operator compares the digest above, and the payload menu
says what it holds. Then: **Engrave Transaction**.
MID
  cat "$WORK/frames.md"
  cat <<'TAIL'

---

## What the operator is left holding

One steel plate carrying one QR symbol and a legend that states the txid, what
the symbol contains, and what to do with it. The plate is a **bearer
instrument**: anyone who scans it can broadcast the transaction. The review
screen says so on its first page — which is where it had to be moved to, because
the screen pages and the warning was below the fold.

**The device never checks its own work.** It has no camera, so the post-cut
screen is the last moment anything can tell the operator to look at what came
out. That is why it says so.
TAIL
} > "$OUT"

echo "wrote $OUT ($(wc -c < "$OUT") bytes)"
