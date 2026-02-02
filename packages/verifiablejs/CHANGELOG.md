# verifiablejs

## 1.2.0

### Minor Changes

- 68f9a3f: feat: add `members_root` and `members_intermediate` functions

  Added two new WASM functions for computing ring commitments from members:

  - `members_root(members: Uint8Array)`: Computes the 384-byte ring root (MembersCommitment / RingVerifierKey) from a SCALE-encoded Vec of members.

  - `members_intermediate(members: Uint8Array)`: Computes the 432-byte intermediate (MembersSet / RingVerifierKeyBuilder) from a SCALE-encoded Vec of members.

  These functions are useful for precomputing the cryptographic commitments needed for chain genesis or test configurations.

## 1.0.4

### Patch Changes

- Add proper TypeScript interface for one_shot return value. The function now returns OneShotResult interface instead of generic object type, providing full autocomplete and type safety for all return fields (proof, alias, member, members, context, message).

## 1.0.3

### Patch Changes

- Monorepo migration and playground examples. No changes to package code or types - purely organizational restructuring. Package distribution and TypeScript types verified working correctly.
