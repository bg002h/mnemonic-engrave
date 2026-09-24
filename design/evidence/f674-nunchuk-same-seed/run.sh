#!/usr/bin/env bash
# F-674 item 1: reproduce the libnunchuk same-seed measurement.
#
#   design/evidence/f674-nunchuk-same-seed/run.sh <fork-checkout> [scratch-dir]
#
# <fork-checkout>: a seedhammer fork tree (measured at 0287e3a). It is COPIED to
#   the scratch dir; the checkout itself is never touched.
# Needs: Go (>= 1.26), md 0.20.1 on PATH, and the libnunchuk harness binary
#   (NUNCHUK_HARNESS, default .tmp/fable-nunchuk-lib/build/fableharness):
#   libnunchuk a7cfb49 (Nunchuk 2.1.1's pin) with harness.cpp (this dir) added
#   to its CMakeLists.txt as
#     add_executable(fableharness fable/harness.cpp)
#     target_link_libraries(fableharness PRIVATE nunchuk embedded)
# Exits non-zero if any wallet's verdict breaks the sorted-order rule, an
# accepted wallet's addresses differ from md's, or a PR-1746 control refuses.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
fork=${1:?fork checkout}
scratch=${2:-/scratch/code/shibboleth/.tmp/f674-repro}
GO=${GO:-/scratch/code/shibboleth/.toolchain/go/bin/go}
md --version | grep -qx 'md 0.20.1' || { echo "need md 0.20.1, have: $(md --version)" >&2; exit 1; }
rm -rf "$scratch"; mkdir -p "$scratch"
rsync -a --exclude .git "$fork"/ "$scratch/fork/"
cp "$here/zz_f674_measure_test.go" "$scratch/fork/gui/"
# name:preset:seed-per-slot (index into f674Seeds; 0 = the demo seed, b8688df1).
# The first four are the natural seatings (leaves unsorted); the last four were
# chosen so the composer's own slot order puts the leaves in ascending order.
SEATINGS='kofn-allB:kofn-recovery:0,0,0,0;kofn-2B:kofn-recovery:0,0,1,2;tiered-2B:tiered-recovery:0,0,1,2;tiered-allB:tiered-recovery:0,0,0,0;kofn-3B-sorted:kofn-recovery:0,0,0,1;kofn-2B-sorted:kofn-recovery:3,0,0,1;tiered-3B-sorted:tiered-recovery:0,0,0,1;tiered-2Bp2-sorted:tiered-recovery:3,7,0,0'
(cd "$scratch/fork" && F674_OUT="$scratch/wallets.tsv" F674_SEATINGS="$SEATINGS" \
  "$GO" test ./gui/ -run 'TestF674Measure$' -count=1 -v)
cp "$here/measure.py" "$scratch/"
python3 "$scratch/measure.py"
