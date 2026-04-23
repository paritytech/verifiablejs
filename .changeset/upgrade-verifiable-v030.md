---
"verifiablejs": minor
---

v1.3.0: upgrade to `verifiable` crate v0.3.0, switch ring-size parameter to on-chain `RingExponent` semantics, and add `validate_with_commitment`.

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
