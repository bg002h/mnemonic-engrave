# PLAN: Is there a route for a wallet FILE off the SeedHammer II?

Planning-only report. Question owned: device-side transport and UI for getting a
Nunchuk / Sparrow / Core importable wallet file OFF the machine. File formats and
host CLI are sibling-owned and not designed here.

**Headline: YES — routes exist.** The one-output claim is TRUE as a statement
about the current firmware's data outputs, but materially incomplete as a
statement about the hardware: the NFC front end already transmits outbound
today, and its Type 4 tag emulator already answers the exact command
(READ_BINARY) a phone would use to pull a file — it just serves a hardcoded
empty file. The cheapest real export route is ~90% already in the tree.

All paths below are in the fork at `/scratch/code/shibboleth/seedhammer`
(branch `main`, HEAD `a91df84`) unless prefixed otherwise. Every line number was
read in this session.

---

## 1. VERDICT on the one-output claim

Claim under test: *"the GUI platform exposes exactly ONE output: `Engraver`.
NFCReader and PayloadReader are read-only, and no screen renders a concrete
descriptor."*

**Verdict: TRUE at the Platform-interface level, with two corrections and one
large caveat.**

The interface, `gui/gui.go:3385-3415`:

| Method | Direction |
|---|---|
| `Engraver(stall bool) (Engraver, error)` — `gui/gui.go:3389` | **write** (the only data actuator; `Engraver` is `stepper.Writer + Close + Stats`, `gui/engraver.go:9-13`) |
| `NFCReader() io.ReadCloser` — `gui/gui.go:3390` | read-only *at the interface* (caveat below) |
| `PayloadReader() seal.Reader` — `gui/gui.go:3398` | read-only (`seal.Reader` is `Probe() bool; Read() ([]byte, error)`, `seal/read.go:27-40`) |
| `SyswReader() sysw.Reader` — `gui/gui.go:3399-3406` | read-only |
| `Dirty(r)` / `NextChunk()` — `gui/gui.go:3409-3413` | **write** — the LCD pixel path |

- **Correction 1 (stale enumeration):** the interface now also has `SyswReader()`
  (read-only, so the claim's substance survives, but its channel list is stale).
- **Correction 2 (display is an output):** the interface has TWO writable sinks,
  engraver and display. The claim's real content — "no *data-export* output" —
  holds only because nothing renders exportable data to the display today.
- **"No screen renders a concrete descriptor": VERIFIED.**
  `DescriptorScreen.Draw` renders only Title / Type ("N-of-M multisig") /
  Script metadata, never the descriptor string (`gui/gui.go:3115-3155`).
  `EngraveScreen.draw` renders instruction text only — there is NO on-device
  plate preview (`gui/gui.go:3311-3360`). Plate previews exist only host-side:
  `gui/preview.go` is `//go:build !tinygo` (line 1) and says explicitly it is
  absent from the firmware image (lines 11-14). `deriveXpubFlow` likewise
  engraves the xpub as an mk1 card and never displays it as text
  (`gui/derive_xpub.go:393-411`).

**The caveat that changes the answer:** `NFCReader()` being `io.ReadCloser` is a
statement about the *interface*, not the radio. The poller behind it embeds a
Type 4 tag **emulator** (`nfc/poller/poller.go:28,45`; entered when an external
field is present, `poller.go:71-85`), and that emulator implements the full
ISO-DEP state machine **including the READ_BINARY (0xB0) handler**
(`nfc/type4/type4.go:201`, handler at `232-249`). Its capability container
advertises "Read allowed", max NDEF size **8192 bytes**
(`type4.go:70,121-123`), and it transmits its responses over RF today
(`type4.go:223` `t.d.Write(resp)`; ST25R3916 listen/target mode at
`driver/st25r3916/st25r3916.go:245-246`, card-emulation responses preloaded into
PT memory at `st25r3916.go:162-164`). The ONLY thing making it "read-only" is
that in `fileState` it serves a hardcoded 2-byte empty file
(`emptyFile = {0x00,0x00}`, `type4.go:103-106`, served at `241-242`), and
`Tag` has no content field or setter. The outbound transport exists and runs;
its payload is pinned empty.

---

## 2. Every output channel, with write capability

1. **Engraver** — write. Stepper words out through `Engraver.Write`
   (`gui/engraver.go:9-13`, real device wrapped at
   `cmd/controller/platform_sh2.go:591-603`). The device's purpose; data leaves
   as metal.
2. **Display (ILI9488, 480x320)** — write. `lcdWidth/lcdHeight` at
   `cmd/controller/platform_sh2.go:35-36`, `DisplaySize` at `648-650`. The GUI
   composes arbitrary images. Nothing exportable is drawn today (§1). A QR
   renderer is a pure UI addition: the QR encoder is already a firmware
   dependency (`github.com/seedhammer/kortschak-qr`, `gui/gui.go:22`) and
   already encodes full-length descriptors for plate QRs
   (`validateDescriptor`, `gui/gui.go:685-687`) — so encoder capacity for a
   ~1300 B payload is proven in-tree. NOTE: the brief's "device QR cap is
   `dim > 37`" is real but binds only `ConstantQR`, the constant-pattern
   passphrase-plate path (`engrave/engrave.go:418-427`, cap at 420, with an
   in-code comment saying exactly why). The general `engrave.QR`
   (`engrave/engrave.go:277`) has no version cap; plate *fit* is the limit
   (`ErrTooLarge`, `gui/gui.go:672`, raised from `gui/gui.go:3491`). Neither
   cap constrains a display route at all.
3. **NFC radio** — **transmits today**, in both roles:
   - *Card emulation (device is the tag):* full Type 4 state machine with
     READ_BINARY support serving CC + (empty) NDEF file — see §1 caveat. This
     is the channel `me bundle` payload pushes already ride INBOUND (phone
     writes to the emulated tag), so multi-KB transfers over this emulator at
     128-byte chunks (`type4.go:12`) are field-proven in the inbound direction,
     and the operator gesture (hold phone to machine) is already rehearsed.
   - *Reader mode (device polls a tag):* read-only as implemented — `type2` and
     `type5` packages issue only read commands (`nfc/type2/type2.go`,
     `nfc/type5/type5.go`; the `bus.Write` calls there are command TX, and no
     UPDATE/write-block command exists in either package). Writing a physical
     tag would be new protocol code.
4. **USB-C port** — data lines are wired to the RP2350 (BOOTSEL works over the
   machine's own port: `docs/custom-firmware.md:190,220`). The release firmware
   is built `-target pico-plus2` with no `-serial` override
   (`flake.nix:80,118`), and the TinyGo target family sets `"serial": "usb"`
   (nix store `targets/rp2350.json:8`) — so the shipped image carries the
   TinyGo USB CDC stack by default (the `-serial=uart` override at
   `flake.nix:144` applies only to the flash/gdb helper). BUT the port is
   single and the firmware hard-requires a 20-28 V USB-PD contract on it before
   the LCD even initialises — on failure it reboots into BOOTSEL
   (`cmd/controller/platform_sh2.go:162-163` voltages, `471-484`
   `monitorPowerSupply` → `rebootIntoBOOTSEL()`; prose confirmation
   `docs/custom-firmware.md:402-406`). So CDC is reachable in normal operation
   only from a host port that simultaneously SOURCES 20 V PD (some laptop/dock
   ports can; never tested on this machine). Firmware can write through it
   (os.Stdout → CDC in TinyGo), contingent and unverified on hardware.
5. **UART** — debug builds only (`cmd/controller/debug_sh2.go:1`,
   `//go:build tinygo && rp && debug`), physical header inside the case. Not an
   operator route.
6. **Flash regions** — read-only to the GUI (`PayloadReader`/`SyswReader`, §1);
   no firmware flash-write code exists (no `machine.Flash` users found
   tree-wide). A host can read flash in BOOTSEL via picotool, but nothing
   device-computed is ever written there, so there is nothing to pull. Dead end
   without new flash-write code plus a reboot dance.
7. **OTP** (`driver/otp`) — inbound fuses; not an export channel.

---

## 3. Viable routes, ranked by cost

**R1 — NFC Type 4 card-emulation share (RECOMMENDED). Smallest delta, one tap.**
Arm the existing emulator with real content; the phone taps and reads the NDEF
file exactly as it would a static tag.
- Firmware delta: `type4.Tag` grows a content buffer + setter and serves it in
  `fileState` instead of `emptyFile` (~20 lines around `type4.go:232-249`);
  poller gains an arm/disarm API (~30 lines, `nfc/poller/poller.go`); Platform
  plumbing (`cmd/controller/platform_sh2.go:572-573` and the interface at
  `gui/gui.go:3385`); one new GUI "share" screen in the Wallet Policy flow
  (~150-250 lines). No new dependency, no new hardware, RF TX path reused
  verbatim.
- Capacity: 8 KB advertised (`type4.go:70`) — the ~1300 B tr descriptor is one
  tap, no paging; even a several-KB wallet file fits.
- Secret-path contact: none required — but the arming API MUST be policy-gated
  to public material (descriptors/xpubs), since the same emulator sits on the
  path seeds ride inbound. That gate is a spec-phase requirement, not a cost.
- Residual risk: outbound chunked READ at 128 B in listen mode is protocol-
  symmetric to the proven inbound path but has never run; needs a hardware
  rehearsal. Phone-side handoff of a read NDEF file into Nunchuk/Sparrow is
  sibling-owned and may be the deciding factor between R1 and R2.

**R2 — Paged/animated QR on the display. No new hardware; more UI; best app
compatibility.** Encoder and pixel pipeline both exist (§2.2). ~280 px usable
square → QR v13 (69 modules) at 4 px/module comfortably; ~1300 B ≈ 3-5 static
pages, or animated frames if siblings pick UR/BBQr. Cost: a QR-to-framebuffer
widget + a paging screen — more UI work than R1 and a worse operator experience
(hold-and-scan-N-frames vs one tap), but Sparrow/Nunchuk scan QRs natively,
which may beat R1's file-handoff friction. Touches no secrets.

**R3 — Engrave it (zero firmware change; already ships).** `descriptorFlow`
already engraves descriptor TEXT+QR / QR-ONLY plates (`gui/gui.go:685-720`,
`2696-2711`), and a phone scanning the engraved QR is an export. Honest
placement: this IS the device's answer for the archival copy, but it is not an
answer for "hand a wallet file to an app at setup time" — one plate and a loud
20-minute cycle per attempt. Whether the ~1300 B tr descriptor even fits a
plate QR is machine-checkable today via `cmd/plateview` (`ErrTooLarge` is the
failure mode, `gui/gui.go:672`) and should be checked regardless, since R3 is
everyone's fallback.

**R4 — USB CDC print-on-demand.** Tiny code delta (an export screen writing to
stdout) and the stack is already compiled in (§2.4) — but the single-port power
topology demands a 20 V-sourcing host port, enumeration of the release image
has never been observed, and it adds a live USB surface to a seed-handling
device. Park unless a hardware experiment (one afternoon: 100 W PD dock +
serial terminal) proves it out.

**R5 — Write a physical NFC tag.** New write-protocol code in type2/type5
(currently read-only, §2.3), and common NTAG21x capacity (≤888 B) is below the
1300 B need — forces exotic Type 5 tags (ST25DV/M24LR). Dominated by R1, which
needs no tag at all. Don't.

**R6 — UART / flash+BOOTSEL.** Not operator routes (§2.5, §2.6). Don't.

---

## 4. Recommendation

Not "engrave it" — the machine already has a second, running transmitter.
**R1 (NFC share) is the primary recommendation:** it reuses a field-proven RF
stack and a rehearsed operator gesture, moves the whole file in one tap with 6x
headroom, and its firmware delta is the smallest of any live route. **R2
(display QR) is the fallback and should be chosen instead if the sibling
phone-side findings show NDEF-file-to-wallet-app friction on iOS/Android** —
QR scanning is the one intake every target wallet already has. R3 remains the
archival story and needs the plate-fit check either way. The device-side spec
for R1/R2 is small enough that both could be specced together and decided by
the siblings' phone-side answer.

---

## 5. Open questions

1. **Outbound READ_BINARY rehearsal:** does a phone actually complete an 11-chunk
   read against the ST25R3916 in listen mode? (Protocol-symmetric to the proven
   inbound path; needs one hardware session with a content-armed emulator.)
2. **Phone-side handoff (sibling-owned, decides R1 vs R2):** can iOS/Android
   deliver a read NDEF file into Nunchuk/Sparrow without a custom app?
3. **Plate fit for R3:** does the ~1300 B tr descriptor pass `validateDescriptor`
   on any plate size? Machine-checkable now via `cmd/plateview`.
4. **USB CDC reality (only if R4 is ever revived):** does the release image
   enumerate on a 20 V-sourcing host port?
5. **Arming policy (spec phase):** the exact predicate for what may ever be
   served outbound over the emulator — public-only, and enforced where?
