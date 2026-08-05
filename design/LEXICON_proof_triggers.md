# LEXICON — proof trigger naming

Governing convention for every test-pattern trigger on the SeedHammer II fork.
Written 2026-08-05 after the `BOTHPROOF!ALL` R0 review showed the existing names
had no rule behind them.

## The grammar

```
<SUBJECT>PROOF!  [PARAMETER]
```

**The root names the AXIS the plate proves.** Not the plate type, not the
program, not the face count — the axis. Two axes exist today:

- a **face** — `TEXTPROOF!`, `CONSTPROOF!`, `BOTHPROOF!`
- a **size range** — `SIZEPROOF!`

**The parameter is that root's single variable, and a slot means ONE kind of
thing.** `BOTHPROOF!`'s parameter is a rung; `SIZEPROOF!`'s is a side. A slot
that accepts two kinds is not a parameter, it is a second trigger wearing the
first one's name — which is what `BOTHPROOF!ALL` would have been, since the
suffix after `BOTHPROOF!` already means "rung".

**The PROMPT states every axis, regardless of what the trigger encodes.** One
word and one parameter cannot carry five axes (program, face, content, size,
side), and they are not asked to. The trigger only has to be typable and
unambiguous; the screen the operator reads before accepting is what has to be
complete. This is already the rule the code follows — `ftProofAsk` names the
face plan precisely because the triggers do not.

## The corollary that decided `SIZEPROOF!`

A trigger must differ from its siblings in something the PROMPT can show.
`BOTHPROOF!ALLA` and `BOTHPROOF!ALLB` differ in their fourteenth character and
prove identical faces, so `Plan.Name()` yields near-identical prompts — the
operator types one, reads a screen that looks like the one they expected,
confirms, and cuts the wrong side onto a face they had already engraved. A
mistype is refused; a mis-pick is engraved.

Rooting them at `SIZEPROOF!` fixes it at the source: the prompt names the RUNGS,
and the rungs genuinely differ — *"FRONT: 5.0mm and 3.8mm"* against
*"BACK: 4.4mm, 3.4mm and 3.0mm"*.

## Inventory

| trigger | program | proves | parameter | faces | content | size |
|---|---|---|---|---|---|---|
| `PASSPROOF!` | passphrase | the passphrase plate | — | constant | pattern + fingerprints | plate default |
| `TEXTPROOF!` | free text | `font/sh` | — | sh | full pattern | 3.0 mm (auto-fit) |
| `CONSTPROOF!` | free text | `font/constant` | — | constant | full pattern | 3.0 mm (auto-fit) |
| `BOTHPROOF!` | free text | both faces | rung | sh + constant | full pattern | 3.0 mm (auto-fit) |
| `BOTHPROOF!<rung>` | free text | both faces at a size | rung in `FontSizes` | sh + constant | trimmed to reach the rung | the named rung |
| `SIZEPROOF!FRONT` | free text | the size ladder | side | sh + constant | sweep only | 5.0 + 3.8 |
| `SIZEPROOF!BACK` | free text | the size ladder | side | sh + constant | sweep only | 4.4 + 3.4 + 3.0 |

`SIZEPROOF!` with no parameter is **not** a trigger: the ladder has no default
half, and defaulting to one would let a slip cut the wrong side.

## Rules

1. **A root names an axis.** Adding a proof that varies a NEW axis takes a new
   root, not a new parameter on an old one.
2. **A parameter slot holds one kind of value.** `FontSizes` rungs for
   `BOTHPROOF!`; sides for `SIZEPROOF!`.
3. **Roots differ at the first character.** `T`, `C`, `B`, `S`, `P` — so a
   mistyped root matches nothing and stays ordinary text. (Matching is by exact
   string, so this is defence in depth rather than the mechanism.)
4. **A parameterised root is a strict prefix of its own parameterised forms and
   of nothing else.** `SIZEPROOF!` prefixes `SIZEPROOF!FRONT`; it must prefix no
   other root. Parameterised roots must never be marked `Sizeable` unless their
   parameter IS a rung — otherwise `SIZEPROOF!FRONT4.4` becomes ambiguous.
5. **The prompt states program, faces, content and size** — every axis, whether
   or not the trigger encodes it, and the side when there is one.
6. **Any string can be somebody's real text.** Declining a proof always
   continues with the trigger exactly as typed. This already holds and must
   keep holding.

## Open: rename `FONTPROOF!` to `PASSPROOF!`

`FONTPROOF!` is the shipped name of the passphrase program's proof. It breaks
rule 1 twice over: it names no axis, and it is distinguished from `TEXTPROOF!`
only by living in another program. "Font" and "text" are near-synonyms; the two
triggers load different fields in different programs, and `FONTPROOF!` cuts in
`font/constant`, the same face `CONSTPROOF!` proves.

This is not theoretical. The operator referred to the free-text proof as
"FONTPROOF!" repeatedly on 2026-08-05; typing that would have opened the
passphrase program instead.

`PASSPROOF!` names its program, differs from every other root at the first
character, and obeys rule 1. **Not yet done** — it changes shipped behaviour and
is the operator's call. Do it as its own small change, never folded into other
work.
