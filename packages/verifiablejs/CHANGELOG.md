# verifiablejs

## 1.3.0

### Minor Changes

- 70f996c: `one_shot().proof` and the three on-chain-proof consumers (`validate_with_commitment`, `validate`, `is_valid`) now use the **raw canonical proof bytes** — no SCALE compact-length prefix.

  ### What changed and why

  For Bandersnatch ring-VRF, `one_shot` previously returned 787 bytes (a 2-byte SCALE compact-length prefix followed by 785 raw canonical bytes). That was a regression introduced unintentionally when bumping the underlying `verifiable` crate to 0.3 in v1.3.0-alpha (the `Proof` type changed from `Vec<u8>` to `BoundedVec<u8, …>` and `Encode::encode(&proof)` was used to convert to `Uint8Array`, which prepended the length prefix). The original behaviour (pre-Apr 2026) was to return raw bytes via `&proof[..]`.

  The prefix breaks every on-chain consumer: the runtime's `RingVrfSignature::deserialize_canonical` (in `verifiable/src/ring/mod.rs`) uses ark-serialize directly and explicitly rejects trailing bytes, so the 2-byte SCALE prefix becomes garbage at the start of the buffer and the verifier returns `Error::DecodeError` → `InvalidTransaction::BadProof`. The bug was silent because:

  - `validate_with_commitment` was using `Decode::decode` which stripped the prefix, so local pre-flight validation against the chain's stored ring root always succeeded.
  - `sign()` was unaffected — its underlying `[u8; 64]` SCALE-encodes raw (fixed-size arrays have no length prefix), so the PersonalId-bind path kept working.
  - The chain returns the umbrella `BadProof` error with no detail.

  This change restores symmetry between `sign` (always raw) and `one_shot.proof` (now raw), and makes the natural usage pattern — "take the proof, put it on the wire, submit" — Just Work against any chain that uses `verifiable`'s canonical deserialize.

  ### Breaking changes

  - **`one_shot().proof` is now 2 bytes shorter** (for Bandersnatch: 785 bytes instead of 787) and carries no SCALE length prefix. Any consumer that was previously stripping the prefix manually (e.g. `proof.slice(2)` as a chain-submission workaround) must drop the `.slice(2)` after upgrading.
  - **`validate_with_commitment`, `validate`, and `is_valid`** now expect raw canonical proof bytes (matching the new `one_shot` output). Existing callers passing the round-tripped output from a v1.3.0-beta.\* `one_shot` will start seeing `"Proof exceeds maximum bounded size"` errors — pass the raw form instead.
  - `batch_validate` is unchanged: it accepts a SCALE-encoded `Vec<(Proof, Vec<u8>, Vec<u8>)>` where `Proof` is `BoundedVec<u8, _>`, and SCALE-encoding the tuple naturally produces the right prefix on each proof. Callers constructing the input from `one_shot` outputs need to wrap each proof into the `BoundedVec` via `try_from(Vec<u8>)` before SCALE-encoding the outer tuple (no `Decode::decode` on the proof bytes).

- e9daf2d: Bump `verifiable` crate to rev `f65b39d` (v0.5.0). Aligns with the rev `individuality` uses.

  ### Breaking changes

  - **Plain signatures are now ThinVRF — 64 bytes instead of 96.** `sign` output and `verify_signature` input change shape; signatures produced by v1.3.0-beta.2 (crate v0.2.1) will not verify on v1.3.0 (crate v0.5.0) and vice versa. Ring proofs are likewise wire-incompatible across this bump.
  - Member point validation on decode is disabled in this rev (matches the `no-point-validation` branch in `paritytech/verifiable` that `individuality` is pinned to). Treat untrusted member bytes as such — `is_member_valid` is the explicit check.
  - Internal `GenerateVerifiable::Capacity` associated type was renamed to `Config` and the `RingSize` wrapper was dropped in favour of using `RingDomainSize` directly. No public JS API surface change.

- 16a6dac: v1.3.0: upgrade to `verifiable` crate v0.3.0, switch ring-size parameter to on-chain `RingExponent` semantics, and add `validate_with_commitment`.

  ### Breaking changes

  - Ring-size parameter renamed from `domain_size` to `ring_exponent` and now accepts on-chain `RingExponent` values (`9 | 10 | 14`) instead of FFT domain sizes (`11 | 12 | 16`). Translation for existing callers: `11 → 9`, `12 → 10`, `16 → 14`. Values match `pallet-members` / `pallet-chunks-manager` storage on chain; the FFT mapping is handled internally.
  - TypeScript type renamed: `RingDomainSize` → `RingExponent`.
  - Affected functions: `one_shot`, `create_multi_context`, `validate`, `validate_multi_context`, `is_valid`, `is_valid_multi_context`, `batch_validate`, `members_root`, `members_intermediate`.
  - Underlying `verifiable` crate bumped to v0.3.0. Proofs generated with v1.2.x (crate v0.2.x) are not compatible with v1.3.x validation.
  - `members_root` output is now 768 bytes (was 384 on v1.2.x).
  - `members_intermediate` output is now 848 bytes (was 432 on v1.2.x).
  - `validate` now returns `Result` (throws on invalid proof) instead of panicking.

  ### New functions

  - `validate_with_commitment(ring_exponent, proof, commitment, context, message)` — validate a proof against a pre-built 768-byte `MembersCommitment` (e.g. the ring root fetched from `pallet-members::Root`). Skips the commitment-construction step that `validate` performs from the member list.
  - `create_multi_context`, `validate_multi_context`, `is_valid`, `is_valid_multi_context`, `batch_validate`, `alias_in_context`, `is_member_valid` — retained from the v0.3.0 upgrade on `update-to-0-3-0`.

  ### Migration snippet

  ```diff
  - const DOMAIN_SIZE = 11;
  - const result = one_shot(DOMAIN_SIZE, entropy, encodedMembers, context, message);
  - const alias = validate(DOMAIN_SIZE, result.proof, encodedMembers, context, message);
  + const RING_EXPONENT = 9;
  + const result = one_shot(RING_EXPONENT, entropy, encodedMembers, context, message);
  + const alias = validate(RING_EXPONENT, result.proof, encodedMembers, context, message);
  ```

  For chain-adjacent apps, prefer `validate_with_commitment` when the ring root is available from storage — saves the need to pass the full member list.

### Patch Changes

- e9daf2d: Fix `sideEffects` so production bundlers don't tree-shake the WASM init.

  The bundler entry (`pkg-bundler/verifiablejs.js`) calls `__wbg_set_wasm(wasm)` and `wasm.__wbindgen_start()` at the top level — these bind the WASM namespace and are required for any exported function to work. The previous `"sideEffects": false` told Rollup/Rolldown those statements were safe to drop, breaking production builds (e.g. `member_from_entropy is undefined`). Vite dev was unaffected because it doesn't tree-shake.

  `sideEffects` now lists the two bundler files that have top-level side effects, preserving tree-shaking for everything else.

## 1.2.1

### Patch Changes

- a11e90f: Add `sideEffects: false` to package.json for better tree-shaking support

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
