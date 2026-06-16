# FOLLOWUPS — mnemonic-engrave

Low/nit items deferred from architect reviews (per the iterative-architect-review standard: Critical/Important fixed inline; low/nit recorded here). Promote to a cycle when convenient.

## Open

_(none)_

## Resolved

### Converter (`me`) polish cycle — RESOLVED 2026-06-16 (commit `5086119`)
All five nits from the converter execution review (`design/agent-reports/me-converter-execution-review.md`) were cleared in one PATCH cycle (spec `design/SPEC_me_converter_polish.md`, plan `design/IMPLEMENTATION_PLAN_me_converter_polish.md`):

- **`me-in-stdin-intermediate-zeroize`** — input now read into a `Zeroizing<String>`, scrubbed on drop (`main.rs`).
- **`me-validate-ms-unreachable`** — `panic!` → `unreachable!("ms1 is refused before validation")` (`validate.rs`).
- **`me-decode-text-tlv-comment`** — `decode_text_tlv` now documents its intentional 1-byte-TLV / no-terminator-check scope (`ndef.rs`).
- **`me-canonical-string-stderr`** — reconciled via an opt-in `--echo` flag (prints the validated string to stderr on success); spec §5 amended to match (`main.rs`, `cli.rs`, `SPEC_seedhammer_engrave.md`).
- **`me-go-harness-shortread-loop`** — the harness now reads the NDEF record in a short-read loop (`firmware/ndef-roundtrip/main.go`).
