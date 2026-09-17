# `demo/sh2` — the browser demo

The SeedHammer II firmware compiled to WebAssembly, wrapped in a guided page so
a stranger can build a real wallet policy by tapping, with no hardware.

## Run it locally

```sh
./build.sh
python3 -m http.server 8391 --directory dist
# http://127.0.0.1:8391/
```

`build.sh` builds `emu.wasm` in the fork if it is stale, then assembles `dist/`.
Set `FORK=/path/to/seedhammer` if the fork is not at `../../../seedhammer`.

**The demo machine needs no toolchain** — no Go, no Rust. Build `dist/` here,
copy it, serve it. macOS already has `python3`.

## What is in `dist/`

| path | what |
| --- | --- |
| `index.html` | landing: the four missions, then copy-paste CLI blocks |
| `mission.html?m=<id>` | one mission's verified steps above the emulator |
| `emu/` | **the fork's `cmd/emu` page, copied verbatim** |

`emu/` is copied, never edited. The emulator stays byte-identical to what the
fork ships, so it cannot drift into a demo-only fork of itself; the guide lives
outside it and reaches it through an iframe.

`dist/` is gitignored — it carries an 11 MB wasm, and the fork gitignores
`emu.wasm` for the same reason.

## The missions

Defined in `src/missions.js`. **Every step and tap count was walked against the
real firmware and the screen read back afterwards** — see `WALKS.md` for the
transcript, the traps, and which mission is still unmeasured. A step nobody has
driven does not go in that file.

## The copy-paste blocks

Verified against the binaries a reader can actually install, which is not the
same as the ones on this machine:

- **`md` blocks work on the published `md-cli` 0.13.0** (`cargo install md-cli`)
  — checked, byte-identical output and the same exit code.
- **The `ms` block needs git HEAD.** Published `ms-cli` 0.14.0 has no `--in` on
  `split`; it can only take a seed as a command-line argument. So the install
  line points at git, and the page says why — teaching argv-secret habits to a
  self-custody audience would be worse than teaching nothing.

Re-check both after any release, with the *installed* binary, not this tree's.

## Deploying, and undoing it

```sh
./deploy.sh --host user@quantoshi --dry      # show every command, change nothing
./deploy.sh --host user@quantoshi            # push dist/, back up the nginx conf
./deploy.sh --host user@quantoshi --rollback # undo
```

**It is undoable by construction, not by luck.** The deploy writes one NEW
directory (`/opt/quantoshi/sh2/`) and backs up the nginx config with a timestamp
before anything touches it. Rolling back is `rm -rf` that directory plus
restoring the backup — Quantoshi's own files are never written to.

**`rsync --delete` is never used.** Pointed at a mistyped directory it removes
everything there that is not in `dist/`, which would take out the app rather
than the demo. A stale leftover file is a trivial problem; a wiped app directory
is not. The script also refuses a target that exists and is not already an SH2
deploy.

**The nginx block is printed, not applied.** Editing a live web server's config
from a script is how an unrelated site goes down. Paste it, then
`sudo nginx -t && sudo systemctl reload nginx` — `nginx -t` validates *before*
anything takes effect, and `reload` keeps the old config running if the new one
is bad. Never `restart`.

Then verify, because the MIME type is what silently breaks the page:

```sh
curl -sI https://quantoshi.xyz/SH2/emu/emu.wasm | grep -i content-type
# expect: content-type: application/wasm
```
