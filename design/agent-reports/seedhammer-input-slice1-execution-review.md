# SeedHammer input Slice 1 (BIP-39 word-entry polish) — whole-diff ADVERSARIAL EXECUTION review

- **Stage:** mandatory post-implementation adversarial execution review (per the refined ultracode policy — over the whole branch diff, independent of the per-task reviews).
- **Date:** 2026-06-17
- **Reviewer:** opus general-purpose agent (ran the full suite + adversarially read the whole diff + re-derived/ran the BIP-39 candidate math against canonical vectors).
- **Branch:** `feat/bip39-entry-polish` (off `upstream/main`), worktree `/scratch/code/shibboleth/seedhammer-wt-bip39`.
- **Verdict:** **SHIP-READY — 0 Critical / 0 Important.** (2 Minor, non-blocking.)

---

## VERBATIM REVIEW OUTPUT (key sections)

### Gate output (all green)
`go test -count=1 ./gui/... ./bip39/...` → PASS (gui, gui/op, gui/saver, gui/text, gui/widget, bip39 all `ok`; all 7 new/changed tests run + pass, none skipped). `go vet ./gui/... ./bip39/...` → only the pre-approved `gui/op/draw_test.go:176` go1.25/1.26 `testing.ArtifactDir` note. `gofmt -l` (4 changed files + whole packages) → empty. `go build ./gui/... ./bip39/...` → clean.

### Scope confirmation (clean)
`git diff upstream/main --stat` → exactly four files: `bip39/bip39.go` (+28), `bip39/bip39_test.go` (+58), `gui/gui.go` (+110), `gui/gui_test.go` (+186). CODEX32 menu line unchanged (still commented out); no `codex32_input_test.go` in the diff; `SeedScreen.Confirm`'s `editBtn` still Button2 (gui.go:1942) and `mnemonic.Valid()` backstop (gui.go:1993) entirely out of the diff. The only Button2→Button3 edits are the three input-flow `okBtn`s + the keyboard dropping Button3 from its commit filter — forward-consistency, sanctioned.

### BIP-39 load-bearing facts — re-derived independently and cross-checked
24-word: 256 entropy + 8 checksum = 264 = 24×11; final word = 3 free entropy + 8 checksum → 2³ = **8**. 12-word: 128 + 4 = 132 = 12×11; final = 7 free + 4 checksum → 2⁷ = **128**. General: 2^(11 − len/3). A throwaway test (since removed; worktree clean) ran `LastWordCandidates` over the canonical BIP-39 vectors (`abandon…about`, `legal…yellow`, `zoo…wrong`, the 18-word vectors) + a synthetic 24-word: 12→128, 18→32, 24→8, every candidate checksum-valid, all distinct, real last word always present. The implementation brute-forces `Valid()` over all 2048 words (does not rely on the bit-math), so it is authoritative. **The historical "1 valid last word" bug is definitively absent.**

### Findings
**Critical:** none. **Important:** none.
**Minor (non-blocking):**
- **M1 — one-frame restriction gap when auto-advancing into the last word** (`gui.go:592` + render block). After `okBtn` accepts the second-to-last word, `selected++` lands on the last word and the same iteration renders before `cands` is recomputed (top-of-loop, next iteration), so for exactly one frame the last word draws the unrestricted keyboard. Self-correcting next frame; fragment is empty (no wrong key pressable, no count shown); `Valid()` backstop remains. Purely cosmetic/transient. Optional fix: recompute `cands` immediately after the advance loop (or set `candsFor = -1`).
- **M2 — "Word N of M"/match-count not applied to the dead SLIP-39 flow** (`gui.go:796` still "Input Words"). Consistent with the scope decision to leave dead flows alone. Not a defect.

**Positive notes:** `LastWordCandidates` returns nil on a partial/out-of-range prefix → `onLastWord()` false → safe fallback to the normal keyboard (no panic, no lockout). Backspacing inside the candidate-restricted last word re-expands the key set (mask recomputed each keystroke); backspace never disabled. Button3 handoff clean (keyboard no longer consumes Button3 → reaches `okBtn`; `TestWordKeyboardScreen` proves end-to-end commit). `completeCandidateWord` never autocompletes a non-candidate (the "I2 hole", explicitly tested) → a checksum-invalid final word can never be accepted via the candidate path.

### Verdict
**SHIP-READY — 0 Critical / 0 Important.** Five tasks implemented correctly + consistently; BIP-39 math authoritative + independently verified; all scope constraints hold; all gates green. The two Minors are optional polish, do not block a clean upstream PR.

---

## Branch commits (off upstream/main 86a58ab)
| Commit | Task |
|---|---|
| `c3b392d` | 1 — `bip39.LastWordCandidates` |
| `8f9e17c` | 2 — Button3 primary-accept (corrected; codex32 re-enable dropped) |
| `4009518` | 3 — "Word N of M" progress title |
| `5f3594d` | 4 — remaining-match count |
| `1dc8cee` | 5a — candidate-scoped key/completion helpers |
| `fcd86c5` | 5b — last-word candidate path wiring + integration tests |

Optional follow-ups (non-blocking): M1 (recompute `cands` after the advance loop to remove the one-frame full-keyboard flash); the `bip39.LastWordCandidates` `len==0` boundary test from the Task-1 advisory.
