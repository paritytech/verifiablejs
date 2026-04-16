---
"verifiablejs": minor
---

Upgrade to verifiable crate v0.3.0

### Breaking Changes

- All ring-related functions now require a `domain_size` parameter as the first argument (`11`, `12`, or `16`). This replaces the compile-time `small-ring` feature with runtime ring size selection.
  - `one_shot(domain_size, entropy, members, context, message)`
  - `validate(domain_size, proof, members, context, message)`
  - `members_root(domain_size, members)`
  - `members_intermediate(domain_size, members)`
- `validate` now returns `Result` (throws on invalid proof) instead of panicking.
- `members_root` output is now 768 bytes (was 384).
- `members_intermediate` output is now 848 bytes (was 432).
- Proofs generated with v1.x are not compatible with v2.x validation.

### New Functions

- `create_multi_context` - generate a single proof covering multiple contexts
- `validate_multi_context` - validate a multi-context proof
- `is_valid` - check proof validity with a known alias (boolean)
- `is_valid_multi_context` - check multi-context proof validity (boolean)
- `batch_validate` - efficiently validate multiple proofs at once
- `alias_in_context` - compute a deterministic alias without creating a proof
- `is_member_valid` - check if a public key is a valid Bandersnatch member
