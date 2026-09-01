# On-device acceptance tags — device-csid-warning cycle

Four NDEF tag payloads, generated from the vendored mk chunk-set-id
corpus's fixture pair (`seedhammer/mk/testdata/csid_ext_v0.1.json`,
`SEED_pinned_12345_ef12f` / `SEED_plate_b_ef12f`), via the host `me`
converter (`mnemonic-engrave/target/release/me`, no subcommand — the
single-string NDEF converter). Never hand-minted: each file is `me`'s NDEF
encoding of one corpus row's chunk string, byte for byte.

Each corpus card is **2 chunk strings → 2 tags** (a chunked mk1 record is
`MB=1 ME=1`, one record per message — R0 r1/M2). Both cards carry the SAME
key content (same `m/48h/0h/0h/2h` path, same xpub); only the pinned card's
chunk-set id is mis-stamped.

| file | card | chunk | declared csid | content-derived csid |
| --- | --- | --- | --- | --- |
| `tag1-pinned-chunk0-of-2.ndef` | pinned (mismatched) | 0 of 2 | `12345` | `ef12f` |
| `tag2-pinned-chunk1-of-2.ndef` | pinned (mismatched) | 1 of 2 | `12345` | `ef12f` |
| `tag3-clean-chunk0-of-2.ndef` | clean twin | 0 of 2 | `ef12f` | `ef12f` |
| `tag4-clean-chunk1-of-2.ndef` | clean twin | 1 of 2 | `ef12f` | `ef12f` |

Regenerate:

```sh
cd mnemonic-engrave
./target/release/me --out design/journeys/csid-tags/tag1-pinned-chunk0-of-2.ndef --echo <<< 'mk1qpzg69pqqsqsqrrhvket9v4jq5zg3vs7zqsrq9dlh7lml0alh7lml0alh7lml0alh7lml0alh7lml0alh7lml0alhupawhtfl552clzu3rgv'
./target/release/me --out design/journeys/csid-tags/tag2-pinned-chunk1-of-2.ndef --echo <<< 'mk1qpzg69ppjd334aa2pecfgwwagl7qqxkdpvrjwectvecw5552eq7tqynlfth397uhnqu3pd7wy4mw3'
./target/release/me --out design/journeys/csid-tags/tag3-clean-chunk0-of-2.ndef --echo <<< 'mk1qpauf0pqqsqsqrrhvket9v4jq5zg3vs7zqsrq9dlh7lml0alh7lml0alh7lml0alh7lml0alh7lml0alh7lml0alhupawfmxpptqkekv2auu'
./target/release/me --out design/journeys/csid-tags/tag4-clean-chunk1-of-2.ndef --echo <<< 'mk1qpauf0ppjd334aa2pecfgwwagl7qqxkdpvrjwectvecw5552eq7tqynlfth397uhuawskqx0yek8x'
```

(source strings copied verbatim from the corpus's `strings` arrays for
`SEED_pinned_12345_ef12f` / `SEED_plate_b_ef12f`.)

## Parse-back verification (done this cycle)

Each file was read back through the fork's own NFC/NDEF reader chain
(`ndef.NewMessageReader` → `ndef.NewRecordReader`, `seedhammer/nfc/ndef`)
and reproduced its source mk1 string byte-exact; the two 2-chunk sets were
then reassembled via `mk.Decode` and confirmed to (a) share identical key
content (`m/48h/0h/0h/2h`, same xpub) and (b) reproduce the declared/derived
split above, including `mk.DerivedChunkSetID` on the pinned set landing on
`ef12f` (matching the wire-declared id on neither set except the clean
twin's). Verified with a scratch probe program against this cycle's
`impl/device-csid-warning` worktree build (not committed).

## Tap order — SPEC Acceptance, "on-device acceptance"

Flash the `impl/device-csid-warning` build (`sh2-flash`), then:

1. **Tap `tag1-pinned-chunk0-of-2.ndef` and `tag2-pinned-chunk1-of-2.ndef`,
   in EITHER order** (reassembly is order-tolerant — `mk/mk.go`'s
   `reassemble` slots chunks by index, not arrival order). At the mk1
   "Inspect key" door: the FIRST tap shows only "Captured 1 of 2. Scan the
   next chunk." (no warning — the set is not complete yet); after the
   **SECOND** tap (set completion) the chunk-set-id mismatch warning modal
   appears, reading the host warning text
   (`design/journeys/csid-warning-modal.png` is this exact screen,
   captured in the emulator this cycle). BACK dismisses it; the card
   display follows, unaffected.
2. **Tap `tag3-clean-chunk0-of-2.ndef` and `tag4-clean-chunk1-of-2.ndef`**,
   either order. No warning appears at any point — the card decodes and
   displays directly after the second tap.

Same two tag pairs also exercise the bundle-gatherer surfaces (Engrave
Bundle / Wallet Policy / Build Policy): the pinned pair completes with the
same warning notice at "Done adding cards", plus the `[csid 12345!ef12f]`
marker on the review list and (Build Policy only) the plate census / restore
doc / payload-cards lines; the clean pair is silent and unmarked throughout.
