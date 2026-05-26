---
"verifiablejs": minor
---

Bump `verifiable` crate to rev `f65b39d` (v0.5.0). Aligns with the rev `individuality` uses.

### Breaking changes

- **Plain signatures are now ThinVRF — 64 bytes instead of 96.** `sign` output and `verify_signature` input change shape; signatures produced by v1.3.0-beta.2 (crate v0.2.1) will not verify on v1.3.0 (crate v0.5.0) and vice versa. Ring proofs are likewise wire-incompatible across this bump.
- Member point validation on decode is disabled in this rev (matches the `no-point-validation` branch in `paritytech/verifiable` that `individuality` is pinned to). Treat untrusted member bytes as such — `is_member_valid` is the explicit check.
- Internal `GenerateVerifiable::Capacity` associated type was renamed to `Config` and the `RingSize` wrapper was dropped in favour of using `RingDomainSize` directly. No public JS API surface change.
