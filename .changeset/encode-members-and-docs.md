---
"verifiablejs": minor
---

Add `encode_members(members: Uint8Array[]): Uint8Array` — a built-in helper that SCALE-encodes an array of 32-byte member public keys into the `Vec<Member>` shape every ring function (`one_shot`, `validate`, `is_valid`, `batch_validate`, `members_root`, `members_intermediate`) expects for its `members` parameter. Callers no longer need to hand-roll the SCALE `Vec<[u8; 32]>` encoding or reach for a separate codec library. Each element must be exactly 32 bytes and a valid Bandersnatch public key, otherwise the call throws.

Documentation fixes:

- The package README's Quick Start used the old `domain_size` values (`11`/`12`/`16`) and called `one_shot(11, …)`; corrected to the `ring_exponent` values the API actually accepts (`9`/`10`/`14`).
- `validate_with_commitment` is now documented in both READMEs as the recommended local pre-flight check before an on-chain submission — it validates a proof against the 288-byte ring root (`MembersCommitment`) the chain exposes.
- Both READMEs now point to `encode_members` as the preferred way to build the `members` argument, keeping the hand-rolled encoding only as a no-WASM fallback.
