#!/usr/bin/env python3
"""Assembles the Load Payload operator journey.

Same rule as the other two documents: every CLI block is
transcript_payload.sh's real stdout+stderr with its true exit code, and every
screenshot is the emulator's own 480x320 framebuffer, POSTed as
canvas.toDataURL() to shot_server.py. The refusal in section 2 is reported with
its actual exit 4 rather than edited out -- it is the most useful thing here,
because it is the first thing anyone gets wrong.

Unlike build_pdf_pathological.py, this one resolves the CSS from a path that
EXISTS (see F-156) and converts the HTML to PDF itself, so "reproducing" is the
two commands in the last section and not two commands plus an undocumented step.
"""

import base64, html, os, re, shutil, subprocess, sys

W = os.path.dirname(os.path.abspath(__file__))
OUT, SHOTS = os.path.join(W, "out"), os.path.join(W, "shots")
PDF = os.path.join(W, "SeedHammer-II-load-payload-journey.pdf")


def b64(p):
    with open(p, "rb") as f:
        return base64.b64encode(f.read()).decode()


def img(name, cls="screen", cap=None):
    path = os.path.join(SHOTS, name)
    if not os.path.exists(path):
        return f'<p class="missing">missing: {html.escape(name)}</p>'
    tag = f'<img class="{cls}" src="data:image/png;base64,{b64(path)}">'
    return f"<figure>{tag}<figcaption>{cap}</figcaption></figure>" if cap else tag


def code(text, limit=None):
    lines = text.rstrip("\n").split("\n")
    clip = limit and len(lines) > limit
    if clip:
        lines = lines[:limit]
    body = html.escape("\n".join(lines))
    if clip:
        body += "\n<span class='clip'>… clipped; full text in out/transcript_payload.txt</span>"
    return f"<pre>{body}</pre>"


def cmds(key, limit=None, skip=0):
    """The COMMANDS of a transcript section, without its narration.

    Several sections open with echoed prose that this document says better in
    its own voice, and printing both makes the page read as if the transcript
    were padded. Everything before the first `$ ` line is dropped; `skip` drops
    that many leading `$ ` blocks as well, for sections whose first command is
    a `cat` of a file already shown in full above.
    """
    text = S.get(key, "")
    lines = text.split("\n")
    starts = [i for i, l in enumerate(lines) if l.startswith("$ ")]
    if not starts:
        return code(text, limit)
    return code("\n".join(lines[starts[min(skip, len(starts) - 1)]:]), limit)


def filebox(path, label=None):
    with open(path) as f:
        return (f'<div class="file"><div class="fname">'
                f'{html.escape(label or os.path.relpath(path, W))}</div>{code(f.read())}</div>')


# The transcript, split on its ########## markers. Absolute paths are shortened
# to $W so the blocks fit a printed column -- the ONLY edit made to real output,
# and it is a substitution of a constant, not a rewording.
raw = open(os.path.join(OUT, "transcript_payload.txt")).read()
# Longest first, or the shorter prefix wins and leaves a stray tail behind.
for prefix in sorted([W + "/", os.path.dirname(os.path.dirname(W)) + "/",
                      "/scratch/code/shibboleth/"], key=len, reverse=True):
    raw = raw.replace(prefix, "")
S, cur, buf = {}, None, []
for line in raw.split("\n"):
    m = re.match(r"^########## (.*)$", line)
    if m:
        if cur:
            S[cur] = "\n".join(buf).strip("\n")
        cur, buf = m.group(1), []
    else:
        buf.append(line)
if cur:
    S[cur] = "\n".join(buf).strip("\n")

CSS = open(os.path.join(W, "build_pdf.py")).read().split('CSS = """')[1].split('"""')[0]

P = []

P.append(f"""
<div class="page">
<h1>SeedHammer II — loading a systemwide payload</h1>
<div class="sub">Putting records into the machine's <b>systemwide region</b> and
using them: the host container, the delivery to flash, the boot offer, the digest
comparison across the air gap, and the two ways back out. Host CLI on one side,
the firmware GUI in the emulator on the other.</div>

<div class="warn"><b>Test material only — never put funds behind these records.</b>
The mnemonic is BIP-39's own all-zero-entropy vector
(<code>abandon…about</code>) and the passphrase is the xkcd string. Both are
published; the payload is UNSEALED, so there is no secret here to leak.</div>

<h2>What a systemwide payload is</h2>
<p>A container written once to a fixed 64 KiB flash region at
<code>0x10D00000</code>, holding records the machine's programs can draw on
instead of asking the operator to type them. It is <b>not</b> the Sealed Payload
— that is a different container (<code>MNEMBLOB</code>) in a different region
(<code>0xE1000000</code>), and the two surfaces are kept apart so no invocation
can produce one while the operator believes they are producing the other.</p>

<div class="note"><b>The firmware never writes flash.</b> Spec §13 D10. Every
write in this document is the HOST talking to the RP2350 bootrom over USB with
the machine in BOOTSEL. The device can <em>load</em> a payload into RAM and
<em>unload</em> it again; it cannot put one there, and it cannot remove one.
That asymmetry is the reason the last screen in this document says "Still in
flash. Wipe it from the host."</p></div>

<h2>The three records</h2>
<table>
<tr><th>record</th><th>class</th><th>what it is for</th></tr>
<tr><td class="mono">abandon abandon … about</td><td>ClassMnemonic</td>
    <td>a seed the Backup Wallet and Engrave Seed programs can take directly</td></tr>
<tr><td class="mono">pass:636f7272…</td><td>ClassPassphrase</td>
    <td>a BIP-39 passphrase, offered where one is asked for</td></tr>
<tr><td class="mono">text:53454544…</td><td>ClassFreeText</td>
    <td>free text, offered to Engrave Text</td></tr>
</table>
<p style="margin-top:7px">Three classes rather than one on purpose: a
single-record payload loads, shows a digest, and demonstrates nothing. The
property worth walking is that <b>one payload feeds several programs</b>, each
picker defaulting to FROM PAYLOAD for the class it wants.</p>

<h2>Toolchain</h2>
{code(S.get('versions',''))}
</div>
""")

P.append(f"""
<div class="page">
<h2>Host step 1 — the records file</h2>
{filebox(os.path.join(W, 'inputs-payload', 'records.txt'), 'inputs-payload/records.txt')}

<h2 style="margin-top:14px">Obstacle — the obvious way to write it is refused</h2>
<p>Nobody writes hex the first time. They write the passphrase and the free text
as themselves, and the container is refused with <b>exit 4</b>:</p>
{filebox(os.path.join(W, 'inputs-payload', 'records-as-first-written.txt'),
         'inputs-payload/records-as-first-written.txt')}
{cmds('2. OBSTACLE -- the obvious way to write them is refused', 12, skip=1)}

<div class="note"><b>Why the bodies are hex at all — the least obvious rule here.</b> EPD §6.4 requires every record to be the
canonical unbroken string — no interior spaces, no hyphens, no grouping — and
uses LF as the record separator on the grounds that no constellation string
contains a newline. <code>correct horse battery staple</code> has three spaces
and Engrave Text's keyboard has a newline key, so both new classes break both
clauses. Records are engraved VERBATIM, so relaxing §6.4 would turn a scratch on
the operator's only copy into silently-absorbed damage. Lowercase rather than
any hex, because EPD §6.6 hashes the section lowercased and hex is the only
common encoding that survives lowercasing unchanged.</div>

<div class="warn"><b>This message is better than it was, and this run is why.</b>
It used to read <em>"record 1 is not a form this container can place. Descriptors
and addresses are not yet classifiable here"</em> — true of a descriptor, and
not true of what had actually happened. <code>classify</code> returns
<code>Unknown</code> for two unrelated situations and the message named only the
one that did not apply, sending the reader to look for a parser problem instead
of typing one <code>xxd</code>. Fixed in <code>me</code>
(<code>1538ef0</code>); the error now names the prefix and the remedy, and
deliberately still does not print the body — a <code>pass:</code> body is a
passphrase, and stderr is scrolled back, logged and pasted into bug reports.</div>

<h2 style="margin-top:14px">Host step 2 — so the bodies are encoded</h2>
{code(S.get('3. so the bodies are hex-encoded, which is a one-liner',''))}
</div>
""")

P.append(f"""
<div class="page">
<h2>Host step 3 — pack, and the digest that crosses the air gap</h2>
{code(S.get('4. pack -- the container',''), 12)}
{code(S.get('5. show -- what it holds, and the digest the operator compares',''))}

<div class="note"><b>The warning is correct and worth reading rather than
dismissing.</b> This payload carries a mnemonic and a passphrase with
<code>--no-passphrase</code>, so they sit in flash in the clear. <code>me</code>
warns and proceeds (§13 D3) instead of refusing: for a published test vector
that is the right call, and for a real seed the operator has just been told
exactly what they are about to do.</div>

<h2 style="margin-top:14px">The same bytes the emulator embeds</h2>
<p>The device screenshots later in this document are of <em>this</em> container,
and that is checked rather than asserted — the journey would otherwise be wrong
in the one way a reader cannot detect, since it would still read as consistent:</p>
{cmds('6. the same bytes are what the emulator embeds', 14)}
</div>
""")

P.append(f"""
<div class="page">
<h2>Host step 4 — the region image, and what gets written</h2>
{cmds('7. --region: the full 65536-byte image that is written to flash', 16)}
<div class="note"><b>The tail is <code>0xFF</code>, not zeros.</b> That is the
ERASED state of NOR flash, so the image is byte-for-byte what the sector looks
like with only the container in it. Zero-padding would be a WRITE of 64 KiB to
say nothing.</div>

<h2 style="margin-top:12px">Padding does not change the digest</h2>
<p>The digest is over the container's public records, not over the region image,
so the value the operator compares on the device is the same whichever form was
written. Both <code>show</code> calls above and below print
<code>55ad b800 6ec6 a066 94f3 6a0e 900a c8d5</code>.</p>
{cmds('8. padding does not change the digest', 10)}

<h2 style="margin-top:12px">Host step 5 — delivery, with the machine in BOOTSEL</h2>
{cmds('11. the delivery step, on a machine in BOOTSEL', 14)}
</div>
""")

P.append(f"""
<div class="page">
<h2>Removing one — also the host, and also not "erase"</h2>
<p>The device offers UNLOAD, which drops the payload from RAM. The region is
untouched by that, and the firmware has no code path that could touch it. To
overwrite the region you write another image over it, exactly the way the first
one arrived:</p>
{cmds('9. wipe -- the overwrite image (spec 5.5), a HOST operation', 22)}
{code(S.get('10. a wiped region carries no container',''))}
<div class="note"><b><code>--fill ones</code> is not the safe default, and it is
not the paranoid one either — it is a choice about what a later reader can
conclude.</b> <code>0xFF</code> leaves the region indistinguishable from one that
was never written, which is what you want if the existence of a payload is itself
sensitive. Random fill leaves it visibly overwritten. <code>me</code> defaults to
random and says the other one's consequence out loud.</div>
</div>
""")

P.append(f"""
<div class="page">
<h2>On the machine — the boot offer</h2>
<p>Detection runs at boot (§10.1). A payload in the region is not loaded
automatically: the operator is asked, and the machine states what it found.</p>
<div class="grid3">
{img('p00-boot.png', cap='A systemwide payload is present. Load it?')}
{img('p17-skip-selected.png', cap='SKIP is a real answer, and the default is not it')}
{img('p01-after-load.png', cap='LOAD → the digest, before anything is loaded')}
</div>

<h2 style="margin-top:14px">The comparison the air gap is for</h2>
<p>The device shows the digest and asks the operator to compare it against what
<code>me sysw pack</code> printed on the host. The two sides of this document are
the two sides of that comparison, and they are the same eight groups:</p>
<div class="grid2">
{code('''HOST   $ me sysw show out/payload.bin
       digest:   55ad b800 6ec6 a066 94f3 6a0e 900a c8d5

FORK   cmd/emu/sysw_test_payload.go:64
       const syswTestDigest = "55ad b800 6ec6 a066 94f3 6a0e 900a c8d5"

DEVICE the screen to the right''')}
{img('p01-after-load.png', cap='Payload Digest — 55ad b800 6ec6 a066 94f3 6a0e 900a c8d5')}
</div>

<div class="warn"><b>Declining the comparison unloads immediately (§13 D13).</b>
Backing out of the digest screen does not leave the payload half-loaded and does
not silently continue. It says which of those it did.</div>
<div class="grid3">
{img('p16-digest-declined.png', cap='Not Loaded — "Digest not compared. Nothing was loaded."')}
{img('p02-after-digest.png', cap='Accepting → the warnings, stated before the payload is live')}
{img('p03-after-warnings.png', cap='KEEP / UNLOAD — the last chance before it is loaded')}
</div>
</div>
""")

P.append(f"""
<div class="page">
<h2>On the machine — what a loaded payload changes</h2>
<p>This is the whole point of the region, and it is one screen: the seed picker
acquires a source it did not have, and that source is the default.</p>
<div class="grid3">
{img('p05-backup-seed-picker.png', cap='LOADED — "Seed from where?" FROM PAYLOAD is the default')}
{img('p06-seed-from-payload.png', cap='All 12 words, from the payload, typed by nobody')}
{img('p15-no-payload-option.png', cap='UNLOADED — the question is not asked at all')}
</div>
<p style="margin-top:8px">The third frame is the control that makes the first two
mean something: with nothing loaded, the "from where?" step does not appear and
the operator lands directly on Input Seed. Same firmware, same build, same
program — the only difference is the region.</p>

<h2 style="margin-top:14px">Load Payload, the program</h2>
<p>Position <b>8 of ten</b> in the top-level list, and the only one of the ten
that the simulator could not walk at all until this run: <code>SyswReader</code>
returned <code>nil</code>, so detection found nothing and every payload-fed
screen above was unreachable in a browser.</p>
<div class="grid3">
{img('p09-load-payload-program.png', cap='Load Payload — the 8th of ten programs')}
{img('p10-load-payload-when-loaded.png', cap='With one loaded: LOAD AGAIN / UNLOAD')}
{img('p12-after-unload.png', cap='UNLOAD asks, and says what it will cost: the digest again')}
</div>
</div>
""")

P.append(f"""
<div class="page">
<h2>On the machine — the way out</h2>
<div class="grid2">
{img('p13-unloaded-notice.png', cap='Payload Unloaded — "Still in flash. Wipe it from the host."')}
<div>
<p>Eleven words that get the model right. UNLOAD is a RAM operation: the records
are gone from this session, the region is exactly as it was, and the next boot
will offer the payload again. The machine says so rather than letting "unload"
be read as "erase".</p>
<p>The host half of that sentence is the <code>me sysw wipe</code> section
earlier in this document. There is no device path to it, by design.</p>
<div class="note"><b>This screen shipped BLANK.</b> The title drew and the body
did not, for a fortnight, because every GUI test asserted on text
<em>submitted</em> to the op tree rather than pixels <em>drawn</em>. Filed as
F-151; the raster assertion added with the fix was calibrated against the real
defect — the broken body drew 2652 px, the fixed one 6688, and a floor of 2000
(the first guess) passed the defect.</div>
</div>
</div>

<h2 style="margin-top:14px">One thing the machine does not tell you</h2>
<p>The home screen is <b>byte-for-byte identical</b> whether a payload is loaded,
was loaded and then unloaded, or was never offered:</p>
{code('''$ sha256sum shots/p04-main-menu.png shots/p14-menu-after-unload.png shots/p18-menu-after-skip.png
f773d610f050ad9983c427635f25df6d80893a8c60ec2cef7efaedc2ab134204  p04-main-menu.png       (loaded)
f773d610f050ad9983c427635f25df6d80893a8c60ec2cef7efaedc2ab134204  p14-menu-after-unload.png  (unloaded)
f773d610f050ad9983c427635f25df6d80893a8c60ec2cef7efaedc2ab134204  p18-menu-after-skip.png    (never loaded)''')}
<p style="margin-top:6px">So an operator who walks away and comes back cannot
tell whether a seed is currently in the machine's RAM without entering Load
Payload and reading which two options it offers. Filed as <b>F-155</b>.</p>
</div>
""")

P.append(f"""
<div class="page">
<h2>What this run turned up</h2>

<div class="warn"><b>F-153 — <code>me sysw pack</code>'s refusal named a cause that
did not apply, and its record index is 0-based and unlabelled.</b> Both hit on the
first invocation of this journey. The message is FIXED (<code>1538ef0</code>): it
now names the reserved prefix and the <code>xxd</code> remedy, states the index
base, and still never prints the body. What remains filed is the base itself —
<code>--in</code> filters blank lines, so the index is not a line number either,
and making it one is a larger change than it looks.</div>

<div class="warn"><b>F-154 — the tenth program's dot is drawn underneath the
firmware version line.</b> Measured on the framebuffer, not eyeballed: the dot row
runs to x≈322 and the "Firmware: emu" line starts at x≈305. It was fine at nine
programs. Load Payload is the tenth, so the overlap arrived with the feature this
document is about.</div>

<div class="warn"><b>F-155 — the home screen cannot tell you whether a payload is
loaded.</b> Three states, one identical frame, sha256 above. The machine knows;
it just does not say. This matters more than a cosmetic issue because the thing
being held is a seed.</div>

<div class="note"><b>F-156 — the two published journeys cannot be regenerated by
the commands their own README gives.</b> <code>transcript_pathological.sh</code>
and <code>build_pdf_pathological.py</code> both read <code>inputs/</code>, which
now holds the <em>other</em> document's twelve cosigners; the pathological
wallet's files moved to <code>inputs-pathological/</code> and the scripts did not
follow. <code>build_pdf_pathological.py</code> also reads its CSS from
<code>design/journey/build_pdf.py</code> — a directory that does not exist — and
opens a <code>keys.json</code> that was never committed. Neither script ran again
after the rename, so nothing said so. This document's builder resolves every path
it uses and produces the PDF itself.</div>

<h2 style="margin-top:14px">Corrections to the two earlier journeys</h2>
<p>Recorded here, and in <code>design/journeys/README.md</code>, because the
published PDFs state them wrongly and a reader arrives at those first. Each is a
follow-up with the measurement behind it.</p>
<table>
<tr><th>#</th><th>what the earlier documents say or imply</th><th>what is true</th></tr>
<tr><td>F-131</td><td>"Recovery: any 3 of 11 signing keys + md1"</td>
    <td>Not a 3-of-11. Four tiers, <b>eight</b> distinct minimal key-sets, each with
    its own timelock, two also needing a hash preimage. False in both directions.</td></tr>
<tr><td>F-132</td><td>the engraved set is a complete backup</td>
    <td>Tiers 1–2 need the 32-byte preimage <code>X</code> to spend. Measured: it
    appears in <b>0</b> backup strings, and nothing in the checklist mentions it.</td></tr>
<tr><td>F-133</td><td>the tiers degrade weakest-last</td>
    <td>INVERTED. Tier 4 (1-of-3) matures at 365.00 d; tier 3 (2-of-2) at ≈455 d.
    The weakest key-set unlocks ~90 days FIRST.</td></tr>
<tr><td>F-134</td><td>the wallet is 26 plates</td>
    <td>26, 38 or 58 depending on an md1-form flag the operator is never shown.
    The default is the 58 one.</td></tr>
<tr><td>F-130</td><td>a restored wallet reproduces the descriptor</td>
    <td>Keys are byte-identical and addresses unaffected, but restored xpubs lose
    depth/parent/child, so the descriptor string and its checksum differ.</td></tr>
<tr><td>F-136</td><td><code>md encode</code> auto-chunks</td>
    <td>It does not; it fails and tells you to retry with
    <code>--force-chunked</code>. Two places say otherwise, including the flag's
    own help.</td></tr>
<tr><td>F-127<br>F-128</td><td>the key-card stub comes from the md1</td>
    <td><code>--from-md1</code> cannot read a chunked md1 at all
    (<code>version 9, expected 4</code>), and the stub <code>mk</code> embeds is
    the <b>template-id</b>, not the <code>WalletPolicyId</code> the spec names.</td></tr>
<tr><td>F-137</td><td>—</td>
    <td>Encoder has no depth guard where the decoder does, so an
    encodable-but-undecodable card may be expressible. <b>Unconfirmed</b>; carried
    on the codec lens's authority and flagged as such.</td></tr>
<tr><td>F-139</td><td><code>CORPUS.md</code> §C6 is TBD</td>
    <td>This wallet is the answer: 182 data symbols against a cap of 80.</td></tr>
<tr><td>F-140</td><td><code>compare-cost</code>: taproot costs +127..+131 vB</td>
    <td>True delta is <b>+1..+6 vB</b>. One side counts its script and the other
    does not, so the comparison points the wrong way.</td></tr>
</table>

<div class="note"><b>F-147, which is a habit rather than a document defect.</b>
<code>cmd &amp;&amp; echo OK</code> prints nothing when <code>cmd</code> fails,
and nothing reads as absence rather than failure. Its cousin bit this transcript
while it was being written: <code>nix … | head -2</code> reported
<code>[exit 0]</code> for a <code>nix: command not found</code>, because
<code>$?</code> was <code>head</code>'s. Fixed by removing the pipe; the
<code>run()</code> helper prints the real exit code unconditionally, and section
11 of the transcript says so where it happened.</div>
</div>
""")

P.append(f"""
<div class="page">
<h2>Reproducing this document</h2>
{code('''cd design/journeys
mkdir -p out shots
bash transcript_payload.sh > out/transcript_payload.txt 2>&1   # every CLI block

# the device frames, in a browser (nothing here is a browser screenshot):
(cd ../../../seedhammer/cmd/emu && ./build.sh)                 # inside the flake
(cd ../../../seedhammer/cmd/emu && python3 -m http.server 8731 --bind 127.0.0.1) &
python3 shot_server.py "$PWD/shots" 8732 http://127.0.0.1:8731 &
#   then drive http://127.0.0.1:8731/index.html and POST canvas.toDataURL()
#   to :8732 for each frame -- see the frame names in build_pdf_payload.py

python3 build_pdf_payload.py                                   # this PDF''')}
<p style="margin-top:8px"><code>out/</code> and <code>shots/</code> are not
committed; the PDF is the deliverable. <code>transcript_payload.sh</code>,
<code>build_pdf_payload.py</code> and <code>inputs-payload/</code> are.</p>

<h2 style="margin-top:14px">Provenance of the test payload</h2>
<p>The container in <code>cmd/emu/sysw_test_payload.bin</code> is the exact output
of the <code>me sysw pack</code> in section 3, and section 6 of the transcript
proves it with <code>cmp</code> and <code>sha256sum</code>. It is confined to the
<code>GOOS=js</code> build: <code>TestSyswTestPayloadIsConfinedToJSOnlyFiles</code>
walks the module and fails if any file outside three named js-only ones mentions
it, with a <code>checked &lt; 50</code> floor so a misrooted walk cannot pass
vacuously. A shipped SeedHammer II must never boot carrying a payload somebody
else packed.</p>

<div class="foot">Every screenshot is the emulator's own 480x320 framebuffer,
POSTed as <code>canvas.toDataURL()</code> to a localhost receiver pinned to one
origin — not a browser screenshot, so there is no chrome, no CSS scaling and no
cropping guesswork. Every CLI block is real stdout+stderr with its true exit
code; the only edit is shortening absolute paths to fit the column.</div>
</div>
""")

doc = ("<!doctype html><meta charset=utf-8>"
       "<title>SeedHammer II — loading a systemwide payload</title>"
       f"<style>{CSS}"
       "@page { size: A4; margin: 0; }"
       "</style>" + "".join(P))
htmlpath = os.path.join(OUT, "journey_payload.html")
open(htmlpath, "w").write(doc)
print(f"wrote {htmlpath} ({len(doc)//1024} KB)")

# The PDF step, which the other two builders leave to an undocumented manual
# pass (F-156). Chrome headless is what produced the published PDFs.
chrome = os.environ.get("CHROME") or shutil.which("chromium") or shutil.which("google-chrome")
if not chrome:
    guess = os.path.expanduser("~/.cache/ms-playwright/chromium-1208/chrome-linux64/chrome")
    chrome = guess if os.path.exists(guess) else None
if not chrome:
    print("no chrome/chromium found; set $CHROME to one. HTML is written.", file=sys.stderr)
    sys.exit(1)
subprocess.run(
    [chrome, "--headless", "--disable-gpu", "--no-sandbox",
     "--no-pdf-header-footer", f"--print-to-pdf={PDF}", "file://" + htmlpath],
    check=True, capture_output=True,
)
print(f"wrote {PDF} ({os.path.getsize(PDF)//1024} KB)")
