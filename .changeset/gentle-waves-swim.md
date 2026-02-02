---
"verifiablejs": minor
---

feat: add `members_root` and `members_intermediate` functions

Added two new WASM functions for computing ring commitments from members:

- `members_root(members: Uint8Array)`: Computes the 384-byte ring root (MembersCommitment / RingVerifierKey) from a SCALE-encoded Vec of members.

- `members_intermediate(members: Uint8Array)`: Computes the 432-byte intermediate (MembersSet / RingVerifierKeyBuilder) from a SCALE-encoded Vec of members.

These functions are useful for precomputing the cryptographic commitments needed for chain genesis or test configurations.
