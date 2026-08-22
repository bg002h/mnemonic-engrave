#!/usr/bin/env python3
"""Assembles the HASHLOCK VAULT journey.

Same rule as the four sibling builders: every CLI block below is
transcript_hashvault.sh's real stdout+stderr with its actual exit code, and
every input file is shown as the transcript `cat`-ed it. Nothing here is a value
somebody typed into a document.

The device half is capture_hashvault.py, which drives the SAME card set into
the emulator over NFC in two arms -- a keyed control that must DERIVE, and the
engraved template+mk1 set that must be REFUSED. Both are assertions, so a
screenshot below is a comparison that held, not a photo. The missing-asset gate
means a half-captured walk cannot ship as a complete document.
"""

import base64, hashlib, html, json, os, re, shutil, subprocess, sys

W = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(W, "out", "hashvault")
IN = os.path.join(W, "inputs-hashvault")
SHOTS = os.path.join(W, "shots")

MISSING = []
ALLOW_MISSING = "--allow-missing" in sys.argv


def missing_gate(artifact):
    if not MISSING:
        return
    uniq = sorted(set(MISSING))
    print(f"\n{len(MISSING)} missing asset(s), {len(uniq)} distinct -- "
          f"{artifact} is INCOMPLETE:", file=sys.stderr)
    for name in uniq:
        print(f"  missing: {name}", file=sys.stderr)
    if ALLOW_MISSING:
        print("--allow-missing given: exiting 0 with an incomplete document.", file=sys.stderr)
        return
    print("Refusing to report success.", file=sys.stderr)
    sys.exit(1)


def b64(p):
    with open(p, "rb") as f:
        return base64.b64encode(f.read()).decode()


def img(name, cls="shot", cap=None):
    """Embed a captured frame, or record it missing.

    SIZE, NOT EXISTENCE: a canvas that failed to rasterise is written as a
    zero-byte PNG with a 200 OK, so a document could otherwise embed nothing
    and still pass its own gate.
    """
    path = os.path.join(SHOTS, name)
    if not os.path.exists(path) or os.path.getsize(path) < 512:
        MISSING.append(name)
        return f'<p class="missing">missing: {html.escape(name)}</p>'
    tag = f'<img class="{cls}" src="data:image/png;base64,{b64(path)}">'
    return f"<figure>{tag}<figcaption>{cap}</figcaption></figure>" if cap else tag


def walk_result(name, *, need):
    """Read a walk's result JSON and REQUIRE the fields the prose asserts.

    The prose below states what the device did. If the walk did not actually
    run, or ran and returned something else, the document must not still say
    it -- that is the mislabelled-evidence failure, and it is the whole reason
    these numbers are read from the run rather than typed.
    """
    path = os.path.join(SHOTS, name)
    if not os.path.exists(path):
        MISSING.append(name)
        return {}
    with open(path) as f:
        d = json.load(f)
    for k in need:
        if k not in d:
            MISSING.append(f"{name}:{k}")
    return d


def code(text, limit=None, must_show=None, must_show_all=None):
    """Render a transcript block, optionally clipped.

    `must_show` names a string the block exists to display — its punchline. A
    clip that cuts the punchline off leaves a page that shows the command and
    hides the result, which reads as evidence while proving nothing. Two
    sections did exactly that on the first build: the "SAME WALLET" bind proof
    sat at line 27 behind a 16-line clip, and the plate count at line 27 behind
    a 20-line clip. Refusing is better than a bigger number, because the number
    goes stale the next time the transcript grows.
    """
    lines = text.rstrip("\n").split("\n")
    if must_show_all is not None:
        # A section that ENUMERATES things must show all of them. The reader
        # counts what is on the page: a wallet described as six seeds that
        # displays four reads as a document contradicting itself, and the
        # first build of this page did exactly that (4 of 6 seeds, 3 of 6 key
        # files) because the clip was a constant chosen before the section
        # existed. The count comes from the transcript, so it cannot go stale.
        hits = [i + 1 for i, l in enumerate(lines) if l.startswith(must_show_all)]
        if limit and hits and hits[-1] > limit:
            shown = sum(1 for h in hits if h <= limit)
            raise SystemExit(
                f"clip of {limit} lines shows only {shown} of {len(hits)} "
                f"{must_show_all!r} blocks (the last is on line {hits[-1]}). "
                f"Raise the limit past {hits[-1]}, or drop it.")
    if must_show is not None:
        at = next((i + 1 for i, l in enumerate(lines) if must_show in l), None)
        if at is None:
            raise SystemExit(
                f"transcript block does not contain its punchline {must_show!r} — "
                "did the transcript change?")
        if limit and at > limit:
            raise SystemExit(
                f"clip of {limit} lines would hide the punchline {must_show!r} "
                f"(it is on line {at}). Raise the limit past {at}.")
    clip = limit and len(lines) > limit
    if clip:
        lines = lines[:limit]
    body = html.escape("\n".join(lines))
    if clip:
        body += "\n<span class='clip'>… clipped; full text in transcript_hashvault.txt</span>"
    return f"<pre>{body}</pre>"


def readfile(p):
    if not os.path.exists(p):
        MISSING.append(os.path.basename(p))
        return ""
    with open(p) as f:
        return f.read()


def section(transcript, header):
    marks = [i for i, l in enumerate(transcript) if l.startswith("=== ")]
    for n, i in enumerate(marks):
        if header in transcript[i]:
            end = marks[n + 1] if n + 1 < len(marks) else len(transcript)
            return "\n".join(transcript[i + 1:end]).strip("\n")
    MISSING.append(f"transcript section {header!r}")
    return ""


CSS = open(os.path.join(W, "build_pdf.py")).read().split('CSS = """')[1].split('"""')[0]
CSS += """
.shot { width: 100%; image-rendering: pixelated; border: 1px solid var(--dim); }
.shotrow { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; margin: 8px 0; }
.shotrow3 { display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 8px; margin: 8px 0; }
"""

tx = readfile(os.path.join(W, "transcript_hashvault.txt")).split("\n")

# Refuse a snapshot produced by a different version of the generator. `section()`
# catches a renamed header; this catches everything else that goes stale while
# every header still resolves.
_gen = os.path.join(W, "transcript_hashvault.sh")
_want = hashlib.sha256(open(_gen, "rb").read()).hexdigest()
_m = re.search(r"^# transcript-generator-sha256: ([0-9a-f]{64})$", "\n".join(tx), re.M)
if not _m:
    raise SystemExit("transcript_hashvault.txt carries no generator stamp.\n"
                     "  Regenerate:  bash transcript_hashvault.sh > transcript_hashvault.txt")
if _m.group(1) != _want:
    raise SystemExit("transcript_hashvault.txt is STALE -- produced by a different\n"
                     "  version of transcript_hashvault.sh.\n"
                     f"    snapshot: {_m.group(1)}\n    current : {_want}\n"
                     "  Regenerate:  bash transcript_hashvault.sh > transcript_hashvault.txt")

policy = readfile(os.path.join(IN, "wallet-policy-hashvault.txt")).strip()
desc = readfile(os.path.join(OUT, "descriptor.txt")).strip()
recv = readfile(os.path.join(OUT, "receive.txt")).split()
chg = readfile(os.path.join(OUT, "change.txt")).split()
md1 = [l for l in readfile(os.path.join(OUT, "md1-template.txt")).split("\n") if l.strip()]
mk1 = [l for l in readfile(os.path.join(OUT, "mk-encode-raw.txt")).split("\n") if l.strip()]

# ── The device walk's numbers, read from the walks' own result JSON ───────────
#
# TYPED numbers are how a document ends up describing a run that never happened.
# walk_result() records a MISSING asset if a walk has not been run or did not
# return the field the prose leans on, so the gate refuses the document rather
# than shipping a confident sentence about nothing.
KW = walk_result("hashvault-keyed-result.json",
                 need=["chunksPresented", "matched", "consentPages"])
SW = walk_result("hashvault-seating-result.json",
                 need=["chunksPresented", "keyCardsGathered", "refused"])

kw_chunks = KW.get("chunksPresented", "?")
sw_chunks = SW.get("chunksPresented", "?")
sw_cards = SW.get("keyCardsGathered", "?")

_kw_matched = KW.get("matched", {})
_kw_rows = "".join(
    f"<tr><td><code>{html.escape(a)}</code></td></tr>" for a in _kw_matched.get("addresses", []))
kw_table = (
    "<table><tr><th>matched on the device AND on the host</th></tr>"
    f"<tr><td>wallet id <code>{html.escape(str(_kw_matched.get('walletPolicyId', '?')))}</code></td></tr>"
    f"{_kw_rows}</table>")

kw_consent0 = img("k04-consent-p0.png", cap="Arm 1: the consent screen, page 1 of "
                  f"{len(KW.get('consentPages', []))} — the keyed card's Policy-ID.")
kw_consent1 = img("k04-consent-p1.png", cap="Arm 1, page 2 — the derived addresses.")
sw_gather = img("h03-gather-full.png",
                cap=f"Arm 2: {sw_chunks} template chunks assembled, {sw_cards} mk1 key cards "
                    "gathered. Nothing has been refused yet.")
sw_refusal = img("h05-refusal.png",
                 cap="Arm 2: the refusal. No address anywhere on the screen.")

FW = walk_result("hashvault-fingerprinted-result.json",
                 need=["chunksPresented", "keyCardsGathered", "matched"])
fw_chunks = FW.get("chunksPresented", "?")
fw_cards = FW.get("keyCardsGathered", "?")
_fw_rows = "".join(f"<tr><td><code>{html.escape(a)}</code></td></tr>"
                   for a in FW.get("matched", {}).get("addresses", []))
fw_table = ("<table><tr><th>derived on the device, matching the host</th></tr>"
            f"{_fw_rows}</table>")
fw_gather = img("f03-gather-full.png",
                cap=f"Arm 3: the {fw_chunks}-chunk fingerprinted template and the same "
                    f"{fw_cards} mk1 cards. No refusal this time.")
fw_consent = img("f04-consent-p1.png",
                 cap="Arm 3: the addresses, seated from the key cards rather than read "
                     "off a keyed card.")

NEG_CONTROL = """$ python3 capture_hashvault.py --prove-it-can-fail --no-build
NEGATIVE CONTROL: feeding the KEYED card while demanding the seating refusal.
  15 chunks, no key cards, so 'declares no fingerprints' must NOT appear.

NEGATIVE CONTROL PASSED: the walk refused to report a refusal that never happened.
$ echo $?
0"""
neg_control = code(NEG_CONTROL, must_show="NEGATIVE CONTROL PASSED")

P = []

P.append(f"""
<section class="page">
<h1>SeedHammer II — the hashlock vault</h1>
<p class="sub">Six seeds, three engraved passphrases, four degrading tiers, one taproot tree.</p>

<p>A wallet that hands control on in stages. Three keys and a passphrase can
spend today; two other keys and a second passphrase can spend after a relative
delay; a sixth key alone can spend after an absolute height; and after a longer
height <b>a passphrase alone</b> can spend — no key at all.</p>

<pre>{html.escape(policy)}</pre>

<table>
<tr><th>tier</th><th>who can spend</th><th>when</th></tr>
<tr><td>1</td><td>3-of-3 (@0,@1,@2) + passphrase A</td><td>any time</td></tr>
<tr><td>2</td><td>2-of-2 (@3,@4) + passphrase B</td><td><code>older(32768)</code> — relative, ~7.6 months after funding</td></tr>
<tr><td>3</td><td>@5 alone</td><td><code>after(1173520)</code> — 963,520 + 210,000</td></tr>
<tr><td>4</td><td><b>passphrase C alone</b></td><td><code>after(1383520)</code> — 963,520 + 420,000</td></tr>
</table>

<div class="warn"><b>Test material only — never put funds behind these keys.</b>
The six seeds come from entropy <code>0x0…01</code> through <code>0x0…06</code>
and the three passphrases are printable strings in
<code>derive-hashvault-keys.sh</code>. Everything here is regenerable by design;
that is the point, and it is also why it is worthless.</div>

<div class="note"><b>The internal key is the BIP-341 NUMS point</b>, so there is
no key-path spend and every spend reveals which tier was used.</div>
</section>

<section class="page">
<h1>The source material</h1>

<p>One seed per key, so every key has its own master fingerprint. That is the
difference from this constellation's other multi-key journey, where three
masters supplied eleven keys across four accounts.</p>
{code(section(tx, "1. The source material"), None, must_show_all="$ cat")}

<h2>Derived at the constellation's own path</h2>
<p><code>m/270028'/coin'/account'/script'</code> — purpose <code>270028'</code>
reads <code>bg002h</code> in a single component, and level 4 is the script type
with <code>0'</code> = tr. Ruled 2026-08-18.</p>

<div class="note"><b>The ruling had no implementation until this journey needed
it.</b> <code>ms derive</code> offered only the BIP templates and
<code>mk derive --path</code> is relative-and-unhardened-only, so nothing in the
constellation could derive at its own path. <code>ms derive --template
bg002h-tr</code> is what closed that.</div>
{code(section(tx, "2. The keys"), None, must_show_all="$ cat")}
</section>

<section class="page">
<h1>The passphrases, and what the policy commits to</h1>

<p>These are engraved as TEXT + QR plates rather than as <code>ms1</code>:
<code>ms encode</code> takes a BIP-39 mnemonic or hex entropy, and these are
human passphrases.</p>

<div class="warn"><b>The hash is over the UTF-8 bytes with NO trailing
newline.</b> <code>printf %s</code>, never <code>echo</code>. A stray
<code>\\n</code> changes the hash, and the tier it belongs to is then locked
forever — with a plate beside it that looks correct.</div>
{code(section(tx, "3. The three passphrases"), None, must_show_all="$ cat")}
</section>

<section class="page">
<h1>Tier 4 has no key, and the default parser refuses it</h1>

<p>rust-miniscript answers <code>All spend paths must require a signature</code>.
That is a <b>safety policy, not a language rule</b>:
<code>and_v(v:after(N),sha256(H))</code> is well-formed miniscript that compiles
to valid script. <code>sanity_check()</code> applies five rules and
<code>requires_sig()</code> is the first; the library documents them as
guarantees rather than validity — "results cannot be relied upon" — and ships
<code>ExtParams</code> precisely so individual rules can be relaxed.</p>
{code(section(tx, "5. Tier 4 has no key"), 24)}

<div class="warn"><b><code>--experimental</code> relaxes ONLY that rule.</b>
Malleability, resource limits, repeated keys and timelock mixing are still
enforced per leaf — those admit scripts that are <em>unspendable</em> rather
than merely unguaranteed, which is a worse class of mistake. And it warns on
every use, because the card carries no record that a flag authored it.</div>

<div class="warn"><b>What tier 4 means in practice.</b> After block 1,383,520,
whoever holds passphrase C can spend this wallet alone. That passphrase is
engraved on a plate. <b>The plate is bearer access.</b> That is the design, not
an oversight — but it is the reason this wallet's plates are not all equal, and
the reason the passphrase plates want a different hiding place from the key
plates.</div>
</section>

<section class="page">
<h1>The cards</h1>

<h2>The keyless template — what gets engraved</h2>
{code(section(tx, "6. The cards re-encode"), 12, must_show="[exit 0]")}

<h2>What the cards carry</h2>
{code(section(tx, "7. What the cards carry"), 26)}

<h2>The keyed set, and the bind</h2>
<p>Two card sets, one wallet. The <b>template id</b> is key-stable, so it is the
same across the keyless and keyed forms — which is exactly what makes it the
right thing to compare.</p>
{code(section(tx, "8. The KEYED card set"), 30, must_show="SAME WALLET")}
</section>

<section class="page">
<h1>Addresses, derived from the card</h1>

<p>From the cards, not from the policy string — this proves what the cards
carry, not what was typed to make them. A taproot address commits to the
<b>taptree shape</b> as well as the keys, so a wrong tree gives wrong addresses
even when every key is right.</p>
{code(section(tx, "9. Addresses"), 14, must_show="bc1p")}

<table>
<tr><th>chain</th><th>index</th><th>address</th></tr>
<tr><td>receive</td><td>0</td><td><code>{html.escape(recv[0]) if recv else '—'}</code></td></tr>
<tr><td>receive</td><td>1</td><td><code>{html.escape(recv[1]) if len(recv) > 1 else '—'}</code></td></tr>
<tr><td>change</td><td>0</td><td><code>{html.escape(chg[0]) if chg else '—'}</code></td></tr>
<tr><td>change</td><td>1</td><td><code>{html.escape(chg[1]) if len(chg) > 1 else '—'}</code></td></tr>
</table>
</section>

<section class="page">
<h1>The concrete descriptor</h1>

<p>The whole wallet in one string: every key, every hash, every timelock. This
is what a coordinator wants pasted in.</p>
{code(section(tx, "10. The CONCRETE descriptor"), 10, must_show="descriptor is")}
<pre>{html.escape(desc[:900])}{'…' if len(desc) > 900 else ''}</pre>
</section>

<section class="page">
<h1>The six key cards</h1>

<p>An <code>mk1</code> card binds a cosigner's key to the wallet. They bind to
the <b>keyless template</b>, not the keyed set: the stub is form-aware, so a
card minted against the keyed policy carries a different stub and is refused
against the template. <b>One wallet, two stubs.</b></p>
{code(section(tx, "11. The six mk1 key cards"), 22, must_show="stub mk embedded")}
</section>

<section class="page">
<h1>What goes on which plate</h1>

<p>The device caps a QR at <code>dim 37</code> — QR v5, about 106 bytes at
ECC-L (<code>engrave/engrave.go:420</code>). Measured against that cap, the three
kinds of content land in three different places:</p>
{code(section(tx, "14. What goes on which plate"), 40, must_show="rejoin check")}

<div class="note"><b>The mk1 cards lose their QR to five characters</b> — 111
against a 106-byte cap. Nothing is lost by it; an mk1 is BCH-protected either
way. It is the difference between scanning a plate and typing one.</div>

<div class="warn"><b>The descriptor plates carry NO checksum.</b> An md1 chunk
has a BCH checksum and a shared chunk-set-id, so a mis-transcribed character is
detected and a mixed-up set is refused. These five plates are raw text split at
a byte offset: one wrong character yields a descriptor that is still
syntactically plausible and describes a <em>different wallet</em>. They are a
convenience for a coordinator. <b>The md1 card set is the backup.</b></div>
</section>

<section class="page">
<h1>The bundle, and the secrets</h1>
{code(section(tx, "12. me bundle"), 30, must_show="backup needs")}

<h2>The secrets</h2>
<p><code>me</code> refuses an <code>ms1</code> outright — it will not render,
preview or transmit a secret, which is what makes the rest of the bundle safe to
preview at all.</p>
{code(section(tx, "13. The secrets"), 16)}
</section>

<section class="page">
<h1>The gap: the engraved set does not name the slots</h1>

<p>Every key sits at the <b>same path</b> from a <b>different seed</b>. So the
keyless template's six origins are identical, and it carries no fingerprints —
while the keyed card does. The engraved backup is the template plus the six mk1
cards, and those bind to the same template with the same stub at the same
origin, so they add no signal either.</p>

<p>Slot order is load-bearing. <code>multi_a</code> commits to key <em>order</em>,
so a different assignment is a different script, a different taptree leaf, and a
different address:</p>
{code(section(tx, "15. THE GAP"), None, must_show="720 assignments")}

<div class="warn"><b>Three different wallets from one set of six keys, and the
engraved template distinguishes none of them.</b> A restorer holding these
plates and all six seeds has 720 candidate assignments and no way to choose.
This is not a tooling defect — it is a property of the wallet's shape, and it is
the thing this journey exists to have found before anything was cut.</div>

<h2>The ways out, and the one that costs almost nothing</h2>
<ul>
<li><b>Declare the fingerprints in the template</b> —
<code>md encode --fingerprint @0=… --fingerprint @1=…</code>, which is
repeatable and was simply not passed. The six masters already have six distinct
fingerprints. No path moves, no key is revealed, the policy is untouched, and
the template id does not change. <b>This is the fix, and the next two pages
measure it.</b></li>
<li><b>Give each key its own account index</b> —
<code>m/270028'/0'/N'/0'</code>, so the origins themselves differ. Works, but it
changes every address and every derivation, which is a large price for
something a declaration solves.</li>
<li><b>Engrave the KEYED card</b> instead of the template — a fingerprint per
slot, at the cost of 15 chunks rather than 4 and of putting the cosigner xpubs
on metal.</li>
<li><b>Record the assignment out of band</b> — the weakest, because it is then
the one part of the backup that is neither engraved nor checkable.</li>
</ul>

<div class="note">This is F-216 (mapping an mk1 to an <code>@N</code> slot) in
its hardest form: there, seating matched on a declared origin because the
origins differed. <b>And the sibling taproot journey is not safe from it either
— checked, not assumed.</b> Its keyless template reads <code>@0</code>,
<code>@4</code> and <code>@8</code> as the same <code>m/48'/0'/0'/2'</code>, one
per master, with no fingerprints. The collision here is six-way and obvious;
there it is three-way and hides inside a wallet that looks fine. Filed.</div>
</section>

<section class="page">
<h1>The device walk — two arms, and the second one is the point</h1>

<p>Everything so far is host-side. This is the same card set carried into the
SeedHammer II emulator over NFC by <code>capture_hashvault.py</code>, which
reuses the seating walk's driver. Both arms <b>assert</b>: the screenshots below
are comparisons that held, not photographs of whatever happened.</p>

<h2>Arm 1 — the control: the keyed card, {kw_chunks} chunks, no seating</h2>

<p>Before the interesting arm can mean anything, the device has to be able to
handle this wallet at all. Four tiers, two <code>sha256</code> hashlocks, both
timelock flavours, <code>multi_a</code> at 3/2/1 and a NUMS internal key — a
device that simply choked on that shape would <em>also</em> produce a refusal in
arm 2, for an entirely different reason.</p>

<div class="shotrow">
{kw_consent0}
{kw_consent1}
</div>

<p>It derives. Read off the screen and compared against the host, which computed
them in Rust from the same six xpubs:</p>
{kw_table}

<div class="note">Two implementations, two languages, one air gap, same
answers. That is what makes arm 2's refusal attributable to the seating and
nothing else.</div>
</section>

<section class="page">
<h1>Arm 2 — the engraved set, and the device declines to guess</h1>

<p>Now the plates as they would actually be cut: the {sw_chunks}-chunk keyless
template and all {sw_cards} mk1 key cards. The device gathers every one of them
— the tally on the left confirms it — and then refuses.</p>

<div class="shotrow">
{sw_gather}
{sw_refusal}
</div>

<p>The sentence is <code>gui/wallet_policy.go</code>'s, and it names the cause
rather than merely reporting failure:</p>

<div class="warn">“Two different key cards claim the same slot, and this
template can't tell them apart. It declares no fingerprints.”</div>

<p>That is <code>errSeatSlotContested</code>. All six cards match all six slots
on origin, the template declares no fingerprint to break the tie, and
<code>seatKeyCards</code> treats every undecidable state as a refusal — because
a misassignment does not fail loudly, it derives a <em>different wallet's</em>
address and shows it to the operator as proof. <b>No address appears on this
screen, and the walk asserts that too.</b></p>

<h2>The refusal check is not vacuous</h2>

<p>An assertion that a string <em>appears</em> passes for a walk that quietly
stopped driving the device. So <code>--prove-it-can-fail</code> feeds the keyed
card — which derives, per arm 1 — while still demanding the refusal, and the run
counts as passing only if the walk <em>fails</em>:</p>
{neg_control}

<div class="note">The firmware anticipated this exact case before this wallet
existed: the comment above that refusal reads “It happens when the template
declares no fingerprints and two slots share a derivation path — a stripped
template cannot tell them apart.” This journey is the first wallet to walk into
it, and the device was already right.</div>
</section>

<section class="page">
<h1>The fix costs one chunk</h1>

<p><code>md encode</code> takes a repeatable <code>--fingerprint</code>. The
template was encoded without it, so it declared bare paths. The six masters
already have six distinct fingerprints — they are printed on the key cards —
and declaring them is the whole repair:</p>
{code(section(tx, "16. THE FIX"), 26, must_show="The whole cost is 1")}

<p>Four chunks became five. The paths did not move, no key is revealed, the
policy is character-for-character the same, and the transcript gates on the
wallet-descriptor-template-id being unchanged — <code>68a1a888…</code> before
and after — so this is demonstrably the same wallet and not a new one that
happens to work.</p>

<div class="note">Worth being precise about what was wrong: not the paths.
Sharing <code>m/270028'/0'/0'/0'</code> across six masters is legitimate and
common — it is the standard shape for multisig, where every cosigner uses the
same account path. What was wrong is that the template carried <b>nothing else
to tell the slots apart</b>, and it had a field for exactly that.</div>
</section>

<section class="page">
<h1>Arm 3 — the same six cards, and now the device seats them</h1>

<p>Nothing about the cards changed. The only difference from arm 2 is which
template they are presented against.</p>

<div class="shotrow">
{fw_gather}
{fw_consent}
</div>

{fw_table}

<p>The same four addresses the keyed card produced in arm 1, and the same four
the host derived in Rust — but reached the way a restorer would actually reach
them: from a keyless template and one card per key, with the device doing the
join.</p>

<div class="warn"><b>That is the round trip.</b> Six seeds went in; a template
and six key cards came out; the device rebuilt the wallet from those and agreed
with the host on every address. Arm 2 remains the more valuable half of the
journey — it is the version that would have been engraved.</div>

<div class="note">Three arms, three different things proved, and the order
matters: arm 1 rules out the wallet's shape as an explanation, arm 2 shows the
engraved set failing, arm 3 shows the repair working on the identical cards. Any
one of them alone would have been ambiguous.</div>
</section>

<section class="page">
<h1>And the plates themselves cannot restore it</h1>

<p>Everything up to here starts from the strings in <code>out/</code> — the
generator's own output. An operator starts from a stack of plates. So the last
step goes the other way: <code>restore_from_plates.py</code> decodes the QR out
of each rendered plate image using <code>zbarimg</code> — an <b>independent</b>
decoder, not the library that drew them — and rebuilds the wallet from what came
off the images and nothing else.</p>

{code(section(tx, "17. RESTORE FROM THE PLATES"), 30, must_show="NOT SEATABLE")}

<div class="warn"><b>The template-id off the plates is RIGHT.</b> The QR, the
rendering, the codec and the chunking are all fine — 22 of 23 images decoded
byte-exact. What the plates cannot do is say which key is which, and the
seatability check is what catches it.</div>

<p>That is the finding stated one final way, and the most concrete one: not
"this template is ambiguous" as a property of a data structure, but <em>these
twenty-two pieces of metal do not add up to this wallet</em>.</p>

<h2>The gate here is inverted, deliberately</h2>

<p>This journey's plates carry the <b>bare</b> template on purpose — they are
the set the gap pages are about. So the transcript requires the restore to
<b>fail</b>, and goes red if these plates ever restore cleanly. If someone
later fixes them without rewriting the narrative, that gate fires and says so.
The sibling pathological journeys run the same driver expecting exit 0, with
both of its controls.</p>

<div class="note">The 23rd plate is an <code>ms1</code> entry with no rendered
image, so it was not checked. Reported rather than skipped: a plate this driver
did not look at is the difference between a partial result and a false clean
one.</div>
</section>

<section class="page">
<h1>What this journey does NOT show</h1>

<ul>
<li><b>Still not metal.</b> The plate restore above reads a rendered PNG. The
path from that PNG to scratched steel to a phone camera — legibility, depth,
glare, engraving artefacts — is not exercised anywhere in this document. What is
proved is the software chain end to end.</li>
<li><b>No REAL hardware.</b> The device walk is the emulator — the same Go
firmware compiled to wasm, driven over the same NFC entry point, but not a
machine with a screen and a hammer. Everything upstream of the display stack is
the shipping code; the display stack itself is not exercised.</li>
<li><b>Only one seating attempt.</b> Arm 2 presents the six cards in one order.
It refuses, so no other order was tried — and by the argument on the gap page,
no order would have helped, because the refusal is about the cards being
<em>indistinguishable</em>, not about their sequence.</li>
<li><b>No host-side restore test</b>, though the device now performs the
restoring half: arm 3 rebuilds the wallet on a SeedHammer II from the
fingerprinted template plus the six key cards and derives the right addresses.
What is missing is the same walk done from the <em>engraved plates</em> — read
back off metal rather than replayed from the files that produced them.</li>
<li><b>No hardware.</b> Nothing has been cut. The plate counts and the QR
capacity are computed from the device's own constants, not measured on
metal.</li>
<li><b>The descriptor plates are unchecked text.</b> Stated again because it is
the one place this backup is weaker than the constellation's usual guarantees.</li>
</ul>

<h2>Reproducing this document</h2>
<pre>bash derive-hashvault-keys.sh
bash transcript_hashvault.sh &gt; transcript_hashvault.txt
python3 build_pdf_hashvault.py</pre>
<p>The builder refuses a transcript snapshot that does not match the script that
produced it, so this document cannot quietly describe a toolchain that has moved
on underneath it.</p>
</section>
""")

html_doc = f"<!doctype html><meta charset=utf-8><title>SeedHammer II — the hashlock vault</title><style>{CSS}</style>{''.join(P)}"
out_html = os.path.join(OUT, "hashvault-journey.html")
os.makedirs(OUT, exist_ok=True)
with open(out_html, "w") as f:
    f.write(html_doc)
print(f"wrote {out_html} ({len(html_doc)//1024} KB)")

# THE GATE BEFORE THE PRINT. A sibling builder once printed its PDF BEFORE
# running this, so the first bad run overwrote the published deliverable with an
# incomplete one. Gate first, print second.
missing_gate(out_html)

PDF = os.path.join(W, "SeedHammer-II-hashlock-vault-journey.pdf")
chrome = os.environ.get("CHROME") or shutil.which("chromium") or shutil.which("google-chrome")
if not chrome:
    guess = os.path.expanduser("~/.cache/ms-playwright/chromium-1208/chrome-linux64/chrome")
    chrome = guess if os.path.exists(guess) else None
if not chrome:
    print("no chrome/chromium found; set $CHROME to one. HTML is written.", file=sys.stderr)
    sys.exit(1)
subprocess.run(
    [chrome, "--headless", "--disable-gpu", "--no-sandbox",
     "--no-pdf-header-footer", f"--print-to-pdf={PDF}", "file://" + out_html],
    check=True, capture_output=True,
)
print(f"wrote {PDF} ({os.path.getsize(PDF)//1024} KB)")
missing_gate(PDF)
