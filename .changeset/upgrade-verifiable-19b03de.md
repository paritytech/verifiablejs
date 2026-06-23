---
"verifiablejs": minor
---

Bump `verifiable` crate to rev `19b03de` (mainline, post-0.5.0 hardening). No JS API surface change; proofs, signatures, and commitments remain wire-compatible with the previous rev (`f65b39d`).

### Behavioral changes

- **Curve-point validation on decode is re-enabled.** The previous rev matched the `no-point-validation` state; now `Member`, `MembersCommitment` (ring root), `MembersSet`, and `StaticChunk` bytes are validated (on-curve + correct subgroup) when decoded. Malformed input to `validate_with_commitment` etc. now fails at decode with the existing error messages instead of risking panics deeper in the crypto.
- **The identity (neutral) point is rejected as a member.** Previously it passed `is_member_valid` but made commitment construction panic (wasm trap) inside the ring backend. Now `is_member_valid` returns `false` for it and `members_root` / `members_intermediate` / `validate` / `one_shot` return a proper error when the member set contains it.
- **Canonical KZG verifier-key pinning.** `validate`, `validate_with_commitment`, `is_valid`, and `batch_validate` now reject a ring root whose embedded trusted-setup key is not the canonical one for the Bandersnatch suite, closing a membership-forgery vector for attacker-supplied commitments.
- **Faster repeated verification.** The Bandersnatch suite now ships static verifier/prover caches, so ring-context parameters are no longer recomputed on every `validate`/`one_shot` call within a session.
- **Side-channel hardened prover.** The crate's `std` feature now bundles `secret-split` (secret scalar split into random summands before point multiplication), which applies to the wasm prover paths (`one_shot`, `sign`).

Also bumps the transitive `ark-vrf` dependency from 0.5.0 to 0.5.1.
