# RULING — F-217 vs remaining Stage 6 ordering

Operator-proxy scheduling ruling, 2026-08-18. Scope: ordering only. F-217's
measurement is taken as given; no code was audited to produce this.

## 1. RULING

**F-217 preempts.** All three F-217 pieces land before 6c begins, and before any
Stage 6 sub-stage authors conformance vectors. The decisive facts are (a) the
corpus every remaining sub-stage would write vectors against is, in its
multi-key keyed cases, 100% contradictory (9/9), so vectors written now are
vectors written twice; and (b) 6c's entire purpose is to export descriptors
verbatim into Sparrow/Nunchuk/BSMS files that real coordinators import — doing
6c first ships a provable self-contradiction into third-party wallets, and no
address check anywhere in Stage 6 can detect it, because addresses derive from
the carried xpubs, not the declared origin. Waiting buys no detection; it only
widens the blast radius. Stage 6 does not stop dead, however: 6a is already
effectively done and closes as-is, and 6b's wire/display-channel ruling is a
design decision with no corpus dependency and may proceed in parallel — only
its round-trip vectors wait.

## 2. ORDER

1. **F-217(1)** — refuse the contradiction, Rust-first with test vectors, then Go convergence port.
2. **F-217(2)** — per-key origin surface, so the CLI can express the correct card that (1) now demands.
3. **F-217(3)** — regenerate the conformance corpus and gate it; this is the single blocking dependency for all remaining vector work.
4. **6a** — close as done; its output is trustworthy once (1) and (3) are in, and no further work is scheduled on it.
5. **6b (contract + ruling)** — may run in parallel with items 1–3; it decides wire shape, not payload content.
6. **6b (round-trip vectors)** — only after F-217(3).
7. **6c** — strictly after F-217(1)+(2)+(3); first sub-stage allowed to export a descriptor off-device.

Partial ordering, deliberately: 5 floats; 1→2→3 is a chain; 6 and 7 both hang off 3.

## 3. WHY

- **Never export a known, provable contradiction.** 6c-before-F-217 puts a card-provable impossibility into files whose whole point is third-party import; that is the one outcome this ordering exists to prevent.
- **Vectors written against the current corpus are waste.** F-217(3) invalidates them by construction; 9/9 multi-key keyed vectors are contradictory today, so every Stage 6 vector authored now is authored twice.
- **Detection cannot come later.** The failure mode is invisible to every address-level check Stage 6 would run and surfaces only at signing time — so "continue and catch it in review" is not an available strategy.
- **The Rust-first rule already forces the front of this order.** F-217(1) is a normative behaviour change; it must land in Rust with vectors before anything downstream consumes it, which is exactly where this ruling puts it.
- **Preemption costs almost nothing.** 6a is done, and 6b's design half proceeds in parallel — the only genuinely delayed work is vector authoring that would have been thrown away anyway.

## 4. DEFERRED / SKIPPED

- **6d (engraving a concrete descriptor) — deferred out of this cycle.** It is last in the dependency chain, its sizing is unmeasured, and the medium is physically irreversible; starting it while descriptor content rules (F-217) and export formats (6c) are still in motion risks engraving text that the cycle itself then changes. It re-enters the next cycle after 6c stabilises.
- **Nothing from F-217 is deferred.** All three pieces are in-cycle; splitting them across cycles would leave either an inexpressible-but-required card shape (2 without 1) or a corpus that still gates Go against contradictory vectors (anything without 3).
