---
"verifiablejs": minor
---

Bump the pinned `verifiable` crate to `19b03deb` (from `f65b39df`) to match the rev used by the Individuality/People-chain runtime and the `verifiable-dart` client.

This pulls in the newer ring-VRF code (ark-vrf 0.5.0 → 0.5.1), identity-point rejection in member validation/construction, and the prover side-channel / deanonymization mitigations. Keeping the client on the same rev as the chain ensures generated members, aliases, proofs and signatures are byte-compatible with what the chain accepts.
