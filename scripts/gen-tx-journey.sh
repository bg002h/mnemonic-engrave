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
# `mt` is a DIFFERENT REPO, and since P3b it owns the producing half of this
# journey: `me tx` was deleted, because `me` has no other verb that
# manufactures a constellation string. So the journey cannot be generated from
# this repo alone -- which is the honest shape, since the operator's pipeline
# spans two binaries.
MT_REPO="${MT_REPO:-$ME_REPO/../mnemonic-transaction}"
GO="${1:-${GO:-go}}"
[ -d "$FORK/gui" ] || { echo "fork not found at $FORK (set FORK=)" >&2; exit 2; }
# The `go` check lives at the STEP THAT NEEDS IT, not here -- see REUSE_FRAMES
# below. Checking it up front made a host-only regeneration impossible on a box
# with no Go toolchain, which is not a property of this document.
if [ "${REUSE_FRAMES:-0}" != 1 ]; then
  command -v "$GO" >/dev/null || { echo "go not found: $GO (or set REUSE_FRAMES=1)" >&2; exit 2; }
fi
[ -d "$MT_REPO/crates/mt-cli" ] || { echo "mt not found at $MT_REPO (set MT_REPO=)" >&2; exit 2; }

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
OUT="$ME_REPO/design/JOURNEY_engrave_transaction.md"
DATA="$ME_REPO/design/journeys/transaction"
mkdir -p "$DATA"

echo "=== building me and mt ==="
( cd "$ME_REPO" && cargo build --locked --quiet )
ME="$ME_REPO/target/debug/me"
( cd "$MT_REPO" && cargo build --locked --quiet )
MT="$MT_REPO/target/debug/mt"

# OFFLINE, by mt's own documented mechanism: --bitcoin-cli pointed at something
# that does not exist is how every gate and journey that must run air-gapped
# forces it. The device this journey ends at IS air-gapped, and a transcript
# that changed depending on whether a node happened to be up would not be
# regenerable.
OFFLINE=(--bitcoin-cli /nonexistent/bitcoin-cli)

# The pinned "even" vector: a real signed 222-byte 1-in/2-out P2WPKH
# transaction, txid from the node that made it. Same artifact the Rust, Go and
# gui suites all pin, so every layer of this journey is the same transaction.
EVEN_HEX="020000000001017c8da925af70e49a12b0cea7b639df5037c87b7fa61f262b86ac32c47aa3ba1a0000000000fdffffff02404b4c0000000000160014c1de0dd435d1d4ad97ed1f51d63f91c800cc4eab3ea1b92901000000160014751097c299d6354fbb2c5a84512dd708f2902f5e0247304402207debc7d89984c7717940b622504318d2c184966a618b32cf8b700d0f125b3ffa02206ef875f9c0b5931e0ea1cf0c109bdb8512835c8e51526f99b3419929a2ea7259012103718f5fd45b926226357e2b0400574b41a32d0bf0ae69a02eebea5fbc542ff52060000000"
printf '%s\n' "$EVEN_HEX" > "$WORK/tx.hex"
# 0600 so the transcript does not depend on the RUNNER'S UMASK. `mt` warns that
# a 0644 input is readable by other users -- correctly, and it is a warning a
# real operator should meet -- but under `umask 022` it fires and under
# `umask 077` it does not, so leaving it in would make "regenerated
# byte-identically" false for a reason that has nothing to do with this code.
chmod 600 "$WORK/tx.hex"

# The SAME transaction with every witness stripped: 113 bytes, byte-identical
# txid, and not one input carrying a signature. The artifact G-P3.19 turns on.
STRIPPED_HEX="02000000017c8da925af70e49a12b0cea7b639df5037c87b7fa61f262b86ac32c47aa3ba1a0000000000fdffffff02404b4c0000000000160014c1de0dd435d1d4ad97ed1f51d63f91c800cc4eab3ea1b92901000000160014751097c299d6354fbb2c5a84512dd708f2902f5e60000000"
printf '%s\n' "$STRIPPED_HEX" > "$WORK/stripped.hex"
chmod 600 "$WORK/stripped.hex"

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
      -e "s#$MT_REPO/target/debug/mt#mt#g" -e "s#'mt'#mt#g" \
      -e "s#0200000000010[0-9a-f]\{100,\}#<the 222-byte transaction, elided>#g" \
      -e "s#020000000[0-9a-f]\{100,\}#<the 113-byte stripped transaction, elided>#g" \
      "$1"
}

echo "=== capturing the host transcript ==="
run "The operator has a finalized transaction, as hex" \
    "head -c 96 '$WORK/tx.hex'; echo '...'"
# THE OBVIOUS TWO-STEP FORM IS REFUSED, and it is here because it is what an
# operator reaches for first. `>` creates the file 0644 under the usual umask,
# and the record IS the engraving of a BEARER instrument. Shown before the form
# that works, because meeting a refusal is part of this journey.
run "The obvious two-step form: write the record to a file first" \
    "'$MT' encode --record --raw ${OFFLINE[*]} --in '$WORK/tx.hex' > '$WORK/rec.txt'"
run "So it is a PIPE — mt owns transactions, me owns the container" \
    "'$MT' encode --record --raw ${OFFLINE[*]} --in '$WORK/tx.hex' | '$ME' sysw pack --region --out '$WORK/region.bin'"
run "The refusal that keeps it off argv" \
    "'$ME' sysw pack --no-passphrase 'tx:$EVEN_HEX'"
run "What is in it, and the digest to compare on the machine" \
    "'$ME' sysw show '$WORK/region.bin'"
run "A set missing three of its six strings still packs, loudly" \
    "printf '%s\n%s\n%s\n' \
      mt1p9h8jqq9qqqqgqqqqqqqyqherdfykhhpey6z2cvafak8804qd7g0dl6v8ex9wr2cvky023skwkeud2229sax \
      mt1p9h8jqq9qqzj8yqpnzw4vl2rwffqyqqqqqkqq282yyhc2vavd20hvk94pz39hts3u5s9a0qd8pwskxfl7ju5 \
      mt1p9h8jqq9qq9qdcc7h75twfxyf340c4sgqzhfdq6xtgt7zhxngpwa049l0z59l6jqcqqqqqq5k5y2ye5nv8yf \
      | '$ME' sysw pack --no-passphrase --out '$WORK/partial.bin'"
# G-P3.19, CLOSED BY CONSTRUCTION. The witness-stripped form has the SAME TXID
# as the honest one, so every identifier an operator can compare matches -- and
# the plate could never be broadcast. Before P3b the PRODUCER said exit 0 and
# the CONSUMER said exit 4 one step later; now the producer refuses first and
# contributes nothing to stdout, so `me` never sees a record at all.
run "An unsigned transaction never becomes a record — the PRODUCER refuses" \
    "'$MT' encode --record --raw ${OFFLINE[*]} --in '$WORK/stripped.hex' | '$ME' sysw pack --no-passphrase --out '$WORK/never.bin'"
run "…and the consumer still refuses it on its own, if one reaches it by hand" \
    "printf 'tx:%s\n' '$STRIPPED_HEX' | '$ME' sysw pack --no-passphrase --out /dev/null"

if [ "${REUSE_FRAMES:-0}" = 1 ]; then
  # AN ESCAPE FOR A HOST-ONLY CHANGE, and it is LOUD ON PURPOSE.
  #
  # It FAILS CLOSED -- the file must already exist -- and it is never the
  # default, because a gate that can quietly not run is worse than no gate.
  # It is honest only while the FORK IS UNCHANGED: these frames are
  # gui.TestCaptureTransactionJourney's own output, so if any screen has moved
  # since they were committed, this document now says something the firmware
  # does not. Check the submodule pin before believing it.
  [ -s "$DATA/frames.md" ] || {
    echo "REUSE_FRAMES=1 but $DATA/frames.md does not exist -- nothing to reuse" >&2
    exit 2
  }
  cat >&2 <<'WARN'
=======================================================================
REUSE_FRAMES=1 -- THE DEVICE HALF WAS NOT RECAPTURED.
Part 2 comes from the COMMITTED design/journeys/transaction/frames.md.
Valid only for a HOST-ONLY change against an UNCHANGED fork. Re-run
without this variable, with a Go toolchain, before trusting Part 2.
=======================================================================
WARN
  cp "$DATA/frames.md" "$WORK/frames.md"
else
  echo "=== capturing the device screens ==="
  TX_JOURNEY_OUT="$WORK/frames.md" "$GO" test -C "$FORK" ./gui/ \
    -run TestCaptureTransactionJourney -count=1 >/dev/null
fi

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
