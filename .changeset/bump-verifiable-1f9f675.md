---
"verifiablejs": minor
---

Bump the pinned `verifiable` crate to `1f9f6752` (from `19b03deb`) to match the rev used by the Individuality/People-chain runtime.

This pulls in the multi-ring/multi-domain batch verification support and the `MembersCommitment` slimming (it now stores only the ring commitment, not the full verifier key). Keeping the client on the same rev as the chain ensures generated members, aliases, proofs and signatures stay byte-compatible with what the chain accepts.

**Note for consumers:** the SCALE-encoded `MembersCommitment` shrank from 768 to 288 bytes. `members_root()` now returns the 288-byte form, and `validate_with_commitment()` expects it — old 768-byte commitments (e.g. cached ones or roots fetched from a pre-upgrade chain) will no longer decode. The JS API surface is otherwise unchanged; `batch_validate` keeps its one-ring-per-batch signature.
