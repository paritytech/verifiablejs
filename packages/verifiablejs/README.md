# Verifiable JS

JavaScript/TypeScript WebAssembly bindings for the [Parity Verifiable](https://github.com/paritytech/verifiable) crate.

Anonymous membership proofs using ring VRFs on the Bandersnatch elliptic curve. Prove you belong to a group without revealing which member you are.

**Full documentation, API reference, and examples: [github.com/paritytech/verifiable-js](https://github.com/paritytech/verifiable-js#readme)**

## Installation

```bash
npm install verifiablejs
```

## Quick Start

```typescript
import { encode_members, member_from_entropy, one_shot, validate, validate_with_commitment } from 'verifiablejs/nodejs'
// or 'verifiablejs/bundler' for browsers

// Create a ring of members
const members = []
for (let i = 0; i < 10; i++) {
  members.push(member_from_entropy(new Uint8Array(32).fill(i)))
}
const encodedMembers = encode_members(members) // SCALE-encode Vec<Member>

// Create an anonymous ring proof
const entropy = new Uint8Array(32).fill(5)
const context = new TextEncoder().encode('my-app')
const message = new TextEncoder().encode('hello')

const RING_EXPONENT = 9 // R2e9 — capacity 255 (also: 10 → 767, 14 → 16,127)
const result = one_shot(RING_EXPONENT, entropy, encodedMembers, context, message)

// Verify the proof from the raw member list
const alias = validate(RING_EXPONENT, result.proof, encodedMembers, context, message)

// Recommended pre-flight before any on-chain submission: validate against the
// 768-byte ring root (MembersCommitment) the chain exposes, rather than the
// full member list. Catches proof/ring/message mismatches locally.
// const ringRoot = /* 768 bytes fetched via RPC */
// const alias = validate_with_commitment(RING_EXPONENT, result.proof, ringRoot, context, message)
```

## API Overview

| Function                   | Description                               |
| -------------------------- | ----------------------------------------- |
| `member_from_entropy`      | Derive a public key from entropy          |
| `is_member_valid`          | Check if a public key is valid            |
| `encode_members`           | SCALE-encode `Member[]` → `Vec<Member>`   |
| `one_shot`                 | Create an anonymous ring proof            |
| `validate`                 | Validate a proof, extract alias           |
| `validate_with_commitment` | Validate against a ring root (pre-flight) |
| `is_valid`                 | Check proof validity with known alias     |
| `create_multi_context`     | Proof covering multiple contexts          |
| `validate_multi_context`   | Validate a multi-context proof            |
| `is_valid_multi_context`   | Check multi-context proof validity        |
| `batch_validate`           | Validate multiple proofs efficiently      |
| `alias_in_context`         | Compute alias without a proof             |
| `sign`                     | Non-anonymous message signature           |
| `verify_signature`         | Verify a signature                        |
| `members_root`             | Compute ring commitment (768 bytes)       |
| `members_intermediate`     | Compute intermediate state (848 bytes)    |

All ring functions require a `ring_exponent` parameter matching the on-chain `RingExponent`: `9` (R2e9, ~255 members), `10` (R2e10, ~767), or `14` (R2e14, ~16,127).

## License

GPL-3.0-or-later WITH Classpath-exception-2.0
