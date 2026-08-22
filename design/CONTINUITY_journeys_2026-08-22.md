# CONTINUITY — journeys, 2026-08-22

Resume point for the journey work. Written before a context clear.

## State: six journeys, all regenerating

| journey | PDF | round-trips from plates? |
| --- | --- | --- |
| load-payload | `SeedHammer-II-load-payload-journey.pdf` | not wired |
| operator | `SeedHammer-II-operator-journey.pdf` | not wired |
| wallet-policy | `SeedHammer-II-wallet-policy-journey.pdf` | not wired |
| pathological (wsh) | `SeedHammer-II-pathological-wallet-journey.pdf` | **yes**, exit 0 |
| tr-pathological | `SeedHammer-II-tr-pathological-journey.pdf` | **yes**, exit 0 |
| hashlock vault | `SeedHammer-II-hashlock-vault-journey.pdf` | **deliberately NOT** — see below |
| **reasonably complex wallet** | `SeedHammer-II-reasonably-complex-wallet-journey.pdf` | **yes**, both wrappers |

The hashvault journey's plates carry the **bare** template on purpose (they are
the set its gap pages are about), so `restore_from_plates.py hashvault` **exits
1** and its transcript gate is **inverted** — it goes red if those plates ever
restore cleanly. Do not "fix" that.

## How to regenerate — and two traps that cost real time

```sh
cd /scratch/code/shibboleth/mnemonic-engrave/design/journeys
export PATH=/nix/store/6rlw3brby0v26n0164a1a2shgn8sv4h3-go-1.25.10/bin:$PATH   # TRAP 1
./derive-rcw-keys.sh                              # inputs; only producer of inputs-rcw/
bash transcript_rcw.sh > transcript_rcw.txt       # host half, gated
python3 capture_rcw.py --wrapper tr  --route keyed     # device, 4 arms
python3 capture_rcw.py --wrapper tr  --route seating --no-build
python3 capture_rcw.py --wrapper wsh --route keyed   --no-build
python3 capture_rcw.py --wrapper wsh --route seating --no-build
python3 restore_from_plates.py rcw/tr             # plate round trip
python3 restore_from_plates.py rcw/wsh
python3 build_pdf_rcw.py                          # 23 pages
```

**TRAP 1 — Go is not on `$PATH`.** The emulator build needs Go **1.25.10** to
match `seedhammer/go.mod`; it lives in the nix store at the path above. Without
it `capture_*.py` fails with `go: command not found`.

**TRAP 2 — `me-preview` must sit BESIDE the `me` binary.** `me bundle --preview`
degrades gracefully when the sidecar is missing: prints `preview skipped
(install me-preview)`, still writes the manifest, **still exits 0**. The journey
ran the *debug* build, so it produced a complete-looking 23-plate checklist with
**zero rendered plates** and nothing said so. Fix:
`cp target/release/me-preview target/debug/`. The transcript now FATALs when
rendered plates ≠ public strings, so it cannot recur silently.

## What the RCW journey proves

- Six seeds → six distinct fingerprints (asserted identical across wrappers).
- tr at `m/270028'/0'/8'/0'`, wsh at `m/270028'/0'/9'/1'` (operator ruling: 8/9
  select the **account**; level-4 stays `0'`=tr / `1'`=wsh).
- Per wrapper: 5-chunk keyless template (fingerprints declared, F-227),
  15-chunk keyed, concrete descriptor, 5 receive + 5 change, six mk1 cards bound
  to the **template id**, 23 public strings, 23 plates.
- **Four device arms all agree**: {tr,wsh} × {keyed, seating}. Seating ==
  keyed is the point — one wallet reached two ways across the air gap.
- **Plate round trip**: 23 of 24 decoded byte-exact by `zbarimg`. The 24th is an
  **ms1 placeholder with no string and no image** (`ms1_required: true`, this
  wallet has zero ms1 strings) — reported as unchecked, not counted as a pass.
- **Three implementations, one address**: BSMS canary == `md address` == the
  SeedHammer II screen. Gated; the transcript FATALs if they diverge.
- **§1b** prints the policy's provenance and runs `check_tiers.py`, which
  asserts the prose tier table against the real policy string — including that
  **exactly one tier is keyless** — with a negative control that keys tier 4 and
  must fail.

## Checklist status against the operator's original ask

11 of 13 complete. Outstanding:

- **SH2 shows 2 addresses per chain, not 5.** `addrProofPerChain = 2`
  (`gui/wallet_policy.go:184`, plan D6). **Operator settled this 2026-08-22:
  "Two addresses is fine."** Not a gap.
- **Nothing was engraved.** Both payloads built and gated, but delivery to the
  device happened over **NFC in the emulator**; the payload was never flashed
  and no plate was cut. Engraving needs the physical machine.

## THE WALLET IS ABOUT TO CHANGE — read before regenerating

The operator has decided to revise tier 4 from **keyless** to **single
signature + the same hashlock**, with `@6` derived from a passphrase via a KDF.
See `NewFeatureIdeas.md`. Consequences when that lands:

- Every address changes. All journey artifacts regenerate.
- **`--experimental` disappears from the whole chain** — verified: the revised
  four-tier policy encodes at exit 0 with default sanity, 4 chunks.
- F-228's G2 (compiler refuses keyless) evaporates for this wallet.
- Core imports the **descriptor**, not just the `addr()` list.
- F-230's trigger (hot export "has no consumer") is met.

`check_tiers.py`'s tier-4 claim and its negative control **must be inverted**
when this lands — it currently asserts tier 4 is keyless and fails if keyed.

## Open follow-ups touching journeys

- **F-227** closed — keyless-template seating; `--fingerprint` per slot.
- **F-228** open — `--from-policy` not in default build; `--experimental` does
  not reach the compiler. **This is why the policy was hand-written.**
- **F-229** LOW, being re-opened by the tier-4 decision.
- **F-230** LOW, trigger may now be met.
- **The `ci/**` hole (unfiled):** in `mnemonic-toolkit` only `examples.yml` and
  `rust.yml` carry `'ci/**'`; **seven workflows are `[main, master]`-only**, so
  the staging ritual structurally cannot gate them — including
  `vendor-freshness.yml`, the exact workflow F-226 was about, fixed in
  `descriptor-mnemonic` and `mnemonic-secret` and never here. This is how Phase 1
  reached master with `docs/manual`'s lint red.
- **`docs/manual` flag-coverage lint is RED locally** and cannot fire in CI:
  `MD_BIN`/`MK_BIN` default to `cargo run --manifest-path ../<sibling>/…`, so it
  builds the *adjacent checkout*, which CI does not have. `md encode
  --experimental`, `md verify --experimental`, `mk encode --keys` undocumented.

## Repo state at hand-off

All five repos clean and level with origin:
`descriptor-mnemonic beb2fb2a` · `mnemonic-secret 7c12f66` ·
`mnemonic-engrave` (this) · `seedhammer a91df84` · `mnemonic-toolkit 8342b2ea`.

Export plan phases: **1 DONE · 1b DONE · 2 DELETED · 3 DONE · 4 NOT NOW ·
5 separate cycle.**
