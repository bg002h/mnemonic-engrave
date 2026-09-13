You are the JOURNEY reviewer for the SeedHammer II's Wallet Policy composer, walking it in the emulator.

## The method, and it is not a correctness pass
Pick a journey and walk it step by step. At EVERY step ask three questions:

1. What does the operator have in hand, exactly?
2. What does the machine do?
3. **What ELSE might they reasonably do — and what happens then?**

Question 3 is the whole thing. Classify every divergence as **refusal**, **warning**, **default**, **not our concern**, or **documentation only** — and a divergence earns a change ONLY if the wrong outcome is worse than telling the operator nothing. Without that rule the method inflates scope without limit.

A correctness lens asks "is this screen right?" and a screen can be right while the path nobody walks is broken. Journey findings are MISSING THINGS AT MOMENTS, not wrong things in sections.

## The journey
**"I want a wallet my heir can spend after a year, that I can spend any time, and that a hashlock can open."** Build it on the machine: three spend paths under one wrapper, a plain key path, a key path with a relative timelock, and a key path with a hashlock and an absolute timelock. Then take it to the end — the template screen, seating or declining keys, the census, and whatever the machine offers for checking the result.

Walk it TWICE: once under `Segwit (wsh)` and once under `Taproot (tr)`, because the wrapper changes which paths are legal and which key becomes the internal key.

## What is already known — do not spend turns rediscovering it
The controller walked this flow and paid for these facts; they are handed over so your turns go to judgement instead:
- The emulator is `cmd/emu`: `sh ./cmd/emu/build.sh`, serve that directory, drive with playwright via `window.shTap/shPress/shRelease/shScreen/shTargets/shSysw`. Serve on a FRESH port each time; the browser caches `emu.wasm`.
- `shSysw("composer")` selects the blob carrying `key:` and `now:` records. It carries NO `hash:` record — the hash screen says so.
- Loading is not one screen: a digest comparison and one or more payload warnings stand between the offer and the carousel, then the Load Payload program's own "Keep this payload loaded?". Acknowledge each deliberately and record what you acknowledged.
- Composer order: `Which script?` → `Start from?` → `Build my own paths` → `Add a spend path` → `What can spend on this path?`.
- The threshold question is asked even at n = 1, with one row.
- Every field editor returns to the PATH's own menu (`Keys | Timelock | Hashlock | Remove path | Move up`), NOT to the path list. Back out to reach the list.
- Number entry: rows 0-2 are three keys centred on x=239 at a 34px pitch (y = 152/198/244); the last row is NOT centred — `0` is at x=239 and backspace at x=273, y=290. `shTargets()` reports only one region per keyboard ROW, so a key cannot be addressed by index (that is F-510).
- The `sha256` method shows a brainwallet warning that must be HELD, not tapped. §8i's rule screen stands between the hashlock row and the keyboard.
- The phrase `correct horse battery staple` under method `sha256` yields digest `b867db87..edbc96cb`; its preimage is `sha256` of the phrase.

## What I want back
Findings, each with: the step, what the operator had, what they did, what happened, what they would reasonably have expected, your classification, and a SUGGESTION. Severity by the project's rule: Critical is a wrong result, data loss, funds risk or an unmet guarantee; Important is a real defect or missing case; Minor and Nit are recorded but gate nothing. Secret-handling defects are never Critical or Important here (operator ruling) — log them with a reproduction.

Your own confusion IS data. If you cannot tell what a screen wants, say so and record it as a finding rather than working it out and moving on.

## Rules
Fork `/scratch/code/shibboleth/seedhammer` at `main`, YOUR OWN worktree: `git -C /scratch/code/shibboleth/seedhammer worktree add --detach /scratch/code/shibboleth/.tmp/seedhammer-journey main`. Two other implementers are working in that repo on their own branches; touch only your worktree, and commit NOTHING — this is a review, and any walk file you write stays in your worktree for the controller to take or discard. Go is `/scratch/code/shibboleth/.toolchain/go/bin/go` first on PATH; `TMPDIR=/scratch/code/shibboleth/.tmp`. No sub-agents. Never read any `.jsonl`.

FINAL ACTION: write `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/composer-policy-journey.md` (create; must not exist): the two walks step by step, every finding as `### C-n / I-n / M-n / N-n — title` with its classification, and closing counts. Return a two-line summary plus the path.
