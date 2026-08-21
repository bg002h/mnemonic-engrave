#!/usr/bin/env python3
"""Assembles the Wallet Policy operator journey.

Same rule as the other three: every CLI block is transcript_walletpolicy.sh's
real stdout+stderr with its actual exit code, and every screenshot is the
emulator's own framebuffer, captured by capture_walletpolicy.py.

What is new here is that the document's central claim is CHECKED BY THE
CAPTURE, not by this script: the walk is handed the host's wallet id and
addresses and throws if the screen disagrees. So the pages below cannot show a
device agreeing with a host unless it did.
"""

import base64, html, json, os, shutil, subprocess, sys

W = os.path.dirname(os.path.abspath(__file__))
SHOTS = os.path.join(W, "shots")
OUT = os.path.join(W, "out")
IN = os.path.join(W, "inputs-walletpolicy")

# ── Missing-asset gate ─────────────────────────────────────────────────────
# Kept identical to the three sibling builders on purpose. A skipped step must
# FAIL, not pass: both wallet journeys once built with 100% of their referenced
# screenshots missing and still exited 0. Enforcement is the DEFAULT;
# --allow-missing is the explicit opt-OUT for a draft.
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
    print("Refusing to report success. Run capture_walletpolicy.py, or pass "
          "--allow-missing for a draft.", file=sys.stderr)
    sys.exit(1)


def b64(p):
    with open(p, "rb") as f:
        return base64.b64encode(f.read()).decode()


def img(name, cls="shot", cap=None):
    path = os.path.join(SHOTS, name)
    if not os.path.exists(path) or os.path.getsize(path) < 512:
        MISSING.append(name)
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
        body += "\n<span class='clip'>… clipped; full text in transcript_walletpolicy.txt</span>"
    return f"<pre>{body}</pre>"


def readfile(p):
    if not os.path.exists(p):
        MISSING.append(os.path.basename(p))
        return ""
    with open(p) as f:
        return f.read()


def section(transcript, header):
    """Return the transcript block under `=== header ... ===`.

    Slicing the transcript by its own headers rather than retyping its output:
    a journey that quotes a command's result by hand is exactly the thing these
    documents claim not to be.
    """
    marks = [i for i, l in enumerate(transcript) if l.startswith("=== ")]
    for n, i in enumerate(marks):
        if header in transcript[i]:
            end = marks[n + 1] if n + 1 < len(marks) else len(transcript)
            return "\n".join(transcript[i + 1:end]).strip("\n")
    MISSING.append(f"transcript section {header!r}")
    return ""


CSS = open(os.path.join(W, "build_pdf.py")).read().split('CSS = """')[1].split('"""')[0]

tx = readfile(os.path.join(W, "transcript_walletpolicy.txt")).split("\n")
result_path = os.path.join(SHOTS, "walletpolicy-result.json")
result = json.load(open(result_path)) if os.path.exists(result_path) else MISSING.append(
    "walletpolicy-result.json") or {}

template = readfile(os.path.join(IN, "policy.template")).strip()
ids = readfile(os.path.join(OUT, "walletpolicy.id.txt"))
recv = readfile(os.path.join(OUT, "walletpolicy.receive.txt")).split()
chg = readfile(os.path.join(OUT, "walletpolicy.change.txt")).split()
md1 = [l for l in readfile(os.path.join(OUT, "walletpolicy.md1.txt")).split("\n") if l.strip()]

P = []

P.append(f"""
<section class="page">
<h1>SeedHammer II — the Wallet Policy journey</h1>
<p class="sub">Engraving a wallet <em>someone else</em> built, and proving which one it is.</p>

<p>Three of this repo's journeys follow an operator who owns the seed. This one
does not. The wallet arrives as a stack of cards; the operator's whole task is
to satisfy themselves that the cards are <em>the right wallet</em> before
committing them to steel — and the device has to make that possible without
ever seeing a secret.</p>

<p>The policy is a depth-2 taproot script tree:</p>
<pre>{html.escape(template)}</pre>

<p>Three properties of it are load-bearing. A taproot Merkle root depends on the
tree's <strong>shape</strong>, so no flat descriptor can express this and the
address can only be reached through the tree; the four cosigner keys are what
push the card set to <strong>{len(md1)} md1 chunks</strong>, making the gather a
real multi-card gather rather than one tap; and four <em>distinct</em> keys mean
a wrong key-to-slot mapping cannot accidentally produce the right address.</p>

<p class="note"><strong>The keys are BIP-39's published test mnemonic</strong>
("abandon … about") at <code>48'/0'/N'/2'</code>. Never put funds behind
them.</p>

<h2>What this wallet is, and is not</h2>

<p>All four cosigners carry master fingerprint <code>73c5da0a</code>, because they
are <strong>accounts 0..3 of one seed</strong>. That is <em>not</em> key reuse —
four distinct xpubs, and the device's duplicate-key refusal has nothing to fire
on — but it is a <strong>single-seed</strong> policy, which for a real
four-cosigner wallet would defeat the point. It is here because it is the
conformance corpus's wallet, and the corpus is what makes the comparison on the
last page checkable against something other than one run of one script.</p>

<p class="note"><strong>The first version of this journey declared the wrong
origin for three of the four keys (F-217)</strong>, and the fix is worth reading
before the pages that follow. It passed <code>--path 48'/0'/0'/2'</code>, which
<em>flattens</em> per-key origins to one shared path — so the card said all four
keys live at account 0, when they live at accounts 0, 1, 2 and 3. Under BIP-32 a
<code>(fingerprint, path)</code> pair names exactly <em>one</em> key, so that
card described a wallet that cannot exist.</p>

<p><strong>Nothing could have caught it downstream, and the re-run proves it:</strong>
the four addresses below are byte-identical to the ones the broken card
produced. Addresses derive from the xpubs a card <em>carries</em>, never from the
origin it <em>declares</em>, so the device-vs-host comparison on the last page
passed just as happily against the impossible card. Only a signer, asked to find
its key, would ever have noticed.</p>

<p>The origins now live in the template — <code>@0/48'/0'/0'/2'/&lt;0;1&gt;/*</code>
— which is where md has always taken per-key origins, and <code>md encode</code>
refuses the flattened form outright. Nine of nine multi-key conformance vectors
carried the same defect; all were regenerated.</p>
</section>
""")

P.append(f"""
<section class="page">
<h2>1 · The host builds the card set</h2>
{code(section(tx, "1. The card set"), 26)}

<h2>2 · …and, separately, what the device must prove to</h2>
<p>These come from the <em>cards</em>, not from the arguments that produced
them — <code>md inspect</code> reads the identity the engraved strings will
actually carry.</p>
{code(section(tx, "2. What the device must prove to"), 30)}

<div class="kv">
<div><span>wallet id</span><code>{html.escape(ids.strip())}</code></div>
<div><span>receive 0..1</span><code>{html.escape(chr(10).join(recv[:2]))}</code></div>
<div><span>change 0..1</span><code>{html.escape(chr(10).join(chg[:2]))}</code></div>
</div>

<p class="note"><strong>The fingerprints are part of the identity.</strong>
Measured while building this journey: the same template, the same four xpubs and
the same origin, encoded <em>without</em> <code>--fingerprint</code>, gives a
7-chunk set whose wallet-policy-id is <code>0b3b95a4…</code> instead of
<code>ade5967c…</code>. The template id is unchanged either way, because it is
key-stable. An operator who omits them gets a card that proves to a different id
than their coordinator shows.</p>
</section>
""")

P.append(f"""
<section class="page">
<h2>3 · The device</h2>
<div class="row">
{img("w00-boot.png", cap="Home. The emulator ships a systemwide payload, so the walk skips its LOAD/SKIP prompt first — this operator's wallet arrives on cards.")}
{img("w01-carousel.png", cap="Wallet Policy, the eighth entry — beside the other engrave programs, not after the utilities.")}
</div>
<div class="row">
{img("w02-gather-empty.png", cap="The gather, empty. It counts md1 descriptors and mk1 keys separately.")}
{img("w03-gather-full.png", cap=f"After all {len(md1)} chunks cross the reader: ONE card. The tally counts cards, not chunks — a card's intermediate chunks draw nothing, and a dropped one means the set never completes.")}
</div>
</section>
""")

consent_imgs = [n for n in sorted(os.listdir(SHOTS)) if n.startswith("w04-consent-p")] if os.path.isdir(SHOTS) else []
P.append(f"""
<section class="page">
<h2>4 · The proof, on the consent path</h2>
<p>Not beside it. The addresses are lines on the screen the operator has to pass
through to engrave, so tapping straight on still shows them.</p>
<div class="row">
{"".join(img(n, cap=f"consent, page {i + 1}") for i, n in enumerate(consent_imgs))}
</div>

<p>The id is <strong>named</strong>. Two exist for one wallet — the
key-<em>stable</em> Template-ID and the key-<em>dependent</em> Policy-ID — and
they are indistinguishable once rendered as 16 bytes of hex. An operator
comparing the wrong one against their coordinator sees a mismatch and reads it
as a corrupted backup, so the screen says which it is showing.</p>
</section>
""")

matched = result.get("matched", {})
P.append(f"""
<section class="page">
<h2>5 · What this run actually established</h2>

<p>The capture is a <strong>comparison, not a photo shoot</strong>.
<code>shots_walletpolicy.js</code> is handed the host's id and addresses and
throws if the screen disagrees, so these pages cannot show agreement that did
not happen.</p>

<div class="kv">
<div><span>chunks presented</span><code>{result.get("chunksPresented", "—")}</code></div>
<div><span>cards gathered</span><code>{result.get("cardsGathered", "—")}</code></div>
<div><span>consent pages read</span><code>{len(result.get("consentPages", []))}</code></div>
<div><span>wallet id matched</span><code>{html.escape(str(matched.get("walletPolicyId", "—")))}</code></div>
<div><span>addresses matched</span><code>{html.escape(chr(10).join(matched.get("addresses", [])))}</code></div>
</div>

<p>Two implementations, in two languages, from the same four xpubs: Rust derived
these off-device and the firmware derived them again from the cards, and they
are the same strings. The negative control was run — corrupting one character of
one expected address fails the capture — so the match is a measurement rather
than a coincidence of both sides being empty.</p>

<h2>Reproducing</h2>
<pre>bash transcript_walletpolicy.sh &gt; transcript_walletpolicy.txt 2&gt;&amp;1
python3 capture_walletpolicy.py     # rebuilds emu.wasm, drives it, writes shots/
python3 build_pdf_walletpolicy.py   # this document</pre>

<h2>What this journey does NOT show</h2>
<ul>
<li><strong>No engraving.</strong> The walk stops at consent. Cutting is the
Engrave Bundle path, already covered by the pathological journey, and the new
thing here is the proof that precedes it.</li>
<li><strong>No mk1 key cards.</strong> The program accepts them, and a keyless
template gathered alongside its key plates still reports no addresses — that is
F-216, deliberately unbuilt because mapping an mk1 to an <code>@N</code> slot
has no obvious rule and a wrong mapping would show a <em>wrong</em> address as
proof.</li>
<li><strong>Not every shape.</strong> A tap leaf outside
<code>pk</code>/<code>multi_a</code>/<code>sortedmulti_a</code> is refused
rather than approximated (F-214).</li>
</ul>
</section>
""")

doc = ("<!doctype html><meta charset='utf-8'>"
       "<title>SeedHammer II — the Wallet Policy journey</title>"
       f"<style>{CSS}</style>" + "".join(P))
p = os.path.join(OUT, "walletpolicy-journey.html")
os.makedirs(OUT, exist_ok=True)
with open(p, "w") as f:
    f.write(doc)
print(f"wrote {p} ({os.path.getsize(p)//1024} KB)")

# THE GATE BEFORE THE PRINT. The pathological builder once printed its PDF
# BEFORE running this, so the first bad run overwrote the published deliverable
# with one captioned "no caption captured for this plate".
missing_gate(p)

PDF = os.path.join(W, "SeedHammer-II-wallet-policy-journey.pdf")
chrome = os.environ.get("CHROME") or shutil.which("chromium") or shutil.which("google-chrome")
if not chrome:
    guess = os.path.expanduser("~/.cache/ms-playwright/chromium-1208/chrome-linux64/chrome")
    chrome = guess if os.path.exists(guess) else None
if not chrome:
    print("no chrome/chromium found; set $CHROME to one. HTML is written.", file=sys.stderr)
    sys.exit(1)
subprocess.run(
    [chrome, "--headless", "--disable-gpu", "--no-sandbox",
     "--no-pdf-header-footer", f"--print-to-pdf={PDF}", "file://" + p],
    check=True, capture_output=True,
)
print(f"wrote {PDF} ({os.path.getsize(PDF)//1024} KB)")
missing_gate(PDF)
