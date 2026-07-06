# Adversarial verification — D4-1 (D4-sidecar)

**Verdict: REFUTED** (confidence: high)

Finding under test: D4-1 (moderate) — "Preview md1/mk1 QR/text fidelity cannot be verified
against the fork's on-device engrave path (not in this repo); the 'render `s` as both text and
QR (lowercase byte-mode)' model may diverge from the device."

Location cited: `preview/layout.go:47` (`engraveBest`).

---

## What the cited code actually does (accurate, but not itself a defect)

`preview/layout.go:47-62` `engraveBest(s)`:
- `qrc, err := qr.Encode(s, qr.L)` — QR built from the raw input string `s`.
- `paragraphFor` (`:16-26`) puts the **same** `s` into `Text:` and the `qrc` (derived from the
  same `s`) into `QR:` for the `text+qr` mode; `qrScale = 3` (`:14`).

So the factual core of the finding is TRUE: text and QR are both derived from the single
lowercase string `s`, byte-mode QR. That is accurate. It is not, by itself, a defect — the
question is whether the FORK's real md1/mk1 engrave path does the same, or a different
representation (the finding's failure scenario).

## The finding's premise ("unverifiable in-repo", "may diverge") is contradicted by the in-repo design

The finding assumes the fork's md1/mk1 engrave path is only knowable from vendored *compiled*
source, and that source is not in the submodule (true: submodule = upstream `seedhammer/seedhammer`
@ `713aee2`, tag `v1.4.2`; `grep -rniE 'md1|mk1|validmd|validmk|validatemdmk'` over 138 `.go`
files returns only an unrelated `bip32.go` fingerprint comment — zero engrave code). Confirmed.

BUT the fork's md1/mk1 engrave path is **fully specified in this repo's design artifacts**, and
all three sources dictate the *same* QR construction the preview uses — foreclosing the
divergence the finding speculates about:

1. **`design/IMPLEMENTATION_PLAN_firmware_pr2_mdmk_engrave.md:233-246`** — the fork's
   `validateMdmk` verbatim:
   ```go
   func validateMdmk(params engrave.Params, s string) ([]string, []Plate, error) {
       qrc, err := qr.Encode(s, qr.L)          // raw string s, byte-mode, level L
       ...
       {"TEXT + QR", backup.Paragraph{Text: s, QR: qrc, QRScale: qrScale}},
       {"TEXT ONLY", backup.Paragraph{Text: s}},
       {"QR ONLY",   backup.Paragraph{QR: qrc, QRScale: qrScale}},
   ```
   The QR is `qr.Encode(s, qr.L)` on the **same raw `s`** as the text — NOT uppercased, NOT the
   NDEF bytes. This is byte-for-byte the preview's `engraveBest`/`paragraphFor` behavior.
   Plan line 3/5/346: "engraved **verbatim**"; "the string is engraved verbatim; the verifier
   only rejects corruption"; "no semantic decode."

2. **`design/SPEC_seedhammer_engrave.md:119`** — "on success route the **raw string** as
   engravable text to the existing `backup.EngraveText`/`Paragraph` path, offering the same
   **TEXT+QR / TEXT / QR-ONLY** choice descriptors get." :17 — "engrave these three strings
   **verbatim**." So the QR is the raw md1/mk1 string, not a re-encoded (uppercase/NDEF) form.
   (Note :110 says the *codex32/ms1* seed-string path `EngraveSeedString` uppercases — that is a
   DIFFERENT path; md1/mk1 explicitly uses `EngraveText` on the raw string, so the "uppercase →
   alphanumeric QR version-3-vs-5" scenario in the finding does not apply to md1/mk1.)

3. **`design/DESIGN_me_bundle_preview.md:67`** — this concern was explicitly recognized and
   closed at design time: "The sidecar must replicate `validateMdmk`'s exact layout:
   `backup.EngraveText`, QR via **`qr.Encode(s, qr.L)`** (error-correction level L, not M),
   **`qrScale = 3`**, modes TEXT+QR / TEXT / QR-only. **Any deviation makes the preview QR differ
   from the engraved QR.**" The preview was built to mirror `validateMdmk` precisely; the
   `layout.go:46` comment "like validateMdmk" is that mirror.

The finder even acknowledges the "replicated `validateMdmk` intent" (report line 195) but dismisses
it as "unverifiable against source." The design plan IS the authoritative source the fork
implemented (fork mdmk-engrave PR shipped; the repo's `validateMdmk` code block is a complete,
compilable function, not a sketch). There is no basis to assume the fork silently deviated from
its own R0-gated, detailed plan on the exact `qr.Encode(s, qr.L)` line that plan specifies.

## Is the failure scenario reachable?

No. The failure scenario requires the fork to feed a **different string** to `qr.Encode`
(uppercased bech32 → alphanumeric mode, giving QR v3/size-21 vs the preview's v5/size-25; or the
NDEF bytes). Both fork (`validateMdmk`) and preview (`engraveBest`) feed the **identical raw
lowercase `s`** to the **identical** `qr.Encode(s, qr.L)`. Same function + same input = same QR
bitmap, trivially — no divergence is possible without the fork violating its own spec. The
specific divergent-representation mechanism the finding invokes is foreclosed by the design, not
merely "unverifiable."

Additionally, the string `s` the preview receives is `plate.string` — the verbatim,
pristine-validated md1/mk1 input line (finder's own negative-results §5, `bundle.rs:180`), the
same md1/mk1 string the device scans and engraves. The preview does NOT QR-encode NDEF bytes.

## Severity honesty

Even the finder concedes (report lines 53-56) this is "**not a proven funds-loss path**": the
load-bearing recovery artifact is the human-readable md1/mk1 **text** (rendered correctly), and
on recovery the user scans the **actual plate**, not the preview. So no wrong-but-accepted plate
and no lost funds arise even under the hypothetical. Combined with the design foreclosing the
divergence mechanism, "moderate" overstates it.

## Residual kernel (at most low, and already noted by the finder)

The single true, narrow observation is that there is no *executable* cross-fidelity golden tying
the preview's QR bitmap / bspline stream to the fork's *compiled* md1/mk1 engrave output, because
the fork's md1/mk1 source is not vendored in `third_party/seedhammer` (upstream v1.4.2). That is a
test-coverage/documentation nit (the finder's own "concrete test that would close it"), not a
moderate funds-safety fidelity gap. The fidelity IS verifiable in-repo against the authoritative
design spec + implementation plan the fork followed, and that verification shows a match.

## Verdict

REFUTED. The finding's central claims — "fidelity cannot be verified against the fork's engrave
path" and "the model may diverge from the device" — are contradicted by three in-repo design
sources (`IMPLEMENTATION_PLAN_firmware_pr2_mdmk_engrave.md:233-246`,
`SPEC_seedhammer_engrave.md:119`, `DESIGN_me_bundle_preview.md:67`) that specify the fork's
`validateMdmk` uses `qr.Encode(s, qr.L)` on the raw string `s`, identical to the preview. The
divergent-representation failure scenario is not reachable, and the finder itself concedes no
funds-loss path. Any residual is a low-severity "add a golden once fork source is vendored" nit,
not a moderate finding.

### Probe/verification log
- `git -C third_party/seedhammer describe --tags` → `v1.4.2` (HEAD `713aee2e...`).
- `grep -rniE 'md1|mk1|validmd|validmk|validatemdmk' third_party/seedhammer --include='*.go'`
  → only `bip32/bip32.go:37` (unrelated) among 138 `.go` files. Confirms no fork md1/mk1 code in
  the submodule (finder's factual premise), but see design sources above for the authoritative
  spec.
- `grep EncodeCompact third_party/seedhammer` → descriptor path only (`gui/gui.go:401`,
  `bip380/bip380.go:177`); confirms the upstream descriptor path uses Encode()≠EncodeCompact()
  (finder's contrast), which does NOT apply to md1/mk1 (a single verbatim string, no compact form).
- Read `preview/layout.go` (`engraveBest`/`paragraphFor`) and `preview/main.go` (stdin `s`);
  confirmed both text and QR derive from the single raw `s`.
