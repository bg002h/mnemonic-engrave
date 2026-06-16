# FOLLOWUPS — mnemonic-engrave

Low/nit items deferred from architect reviews (per the iterative-architect-review standard: Critical/Important fixed inline; low/nit recorded here). Promote to a cycle when convenient.

## Converter (`me`) — from execution review (2026-06-16, `design/agent-reports/me-converter-execution-review.md`)

- **`me-in-stdin-intermediate-zeroize`** — `main.rs:46-47`: `read_to_string` (stdin and `--in`) may leave intermediate heap copies that aren't zeroized; the primary `input` buffer is scrubbed on all paths. Add a clarifying comment, and consider reading into a `Zeroizing<String>`/byte buffer for defense-in-depth. Low (offline tool; `ms1` is refused, not the common input).
- **`me-validate-ms-unreachable`** — `validate.rs:53`: replace `panic!` on `Format::Ms` with `unreachable!("ms1 is refused before validation")` to signal the contract invariant more precisely. Nit.
- **`me-decode-text-tlv-comment`** — `ndef.rs:67-74`: `decode_text_tlv` intentionally handles only the 1-byte TLV length form and skips the `0xFE` terminator check (sufficient for the round-trip self-test against `me`'s own bounded output). Add a comment stating the intentional scope. Nit.
- **`me-canonical-string-stderr`** — spec §5 lists "the canonical validated string" among stderr outputs; the impl prints only a byte-count line (for `--out`) and nothing for stdout/hex/base64. Decide: either echo the canonical string to stderr on success, or amend the spec to drop it (the reviewer noted not re-emitting input is arguably better hygiene). Low — reconcile spec↔impl.
- **`me-go-harness-shortread-loop`** — `firmware/ndef-roundtrip/main.go:21-27`: single `rr.Read` into a 4096-byte buffer is correct for the test vector; a short-read loop would be more robust for larger payloads. Nit (cross-language test is `#[ignore]`d pending a Go toolchain).
