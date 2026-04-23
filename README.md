# Verifiable JS

JavaScript/TypeScript WebAssembly bindings for the [Parity Verifiable](https://github.com/paritytech/verifiable) crate.

This library enables **anonymous membership proofs** using ring VRFs (Verifiable Random Functions) on the Bandersnatch elliptic curve. A member of a group can prove they belong to that group without revealing *which* member they are, while generating a deterministic, context-specific pseudonymous alias that is unlinkable across different contexts.

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Concepts](#concepts)
- [Architecture](#architecture)
- [API Reference](#api-reference)
  - [Key Management](#key-management)
  - [Ring Proofs](#ring-proofs)
  - [Multi-Context Proofs](#multi-context-proofs)
  - [Batch Validation](#batch-validation)
  - [Aliases](#aliases)
  - [Signatures](#signatures)
  - [Ring Commitment](#ring-commitment)
- [Data Encoding](#data-encoding)
- [TypeScript Types](#typescript-types)
- [Platform Support](#platform-support)
- [Development](#development)
- [License](#license)

---

## Installation

```bash
npm install verifiablejs
```

The package is published on npm as [`verifiablejs`](https://www.npmjs.com/package/verifiablejs).

---

## Quick Start

```typescript
import {
  member_from_entropy,
  one_shot,
  validate,
  validate_with_commitment,
  sign,
  verify_signature,
} from 'verifiablejs/nodejs'; // or 'verifiablejs/bundler' for browsers

// 1. Create members
const members = [];
for (let i = 0; i < 10; i++) {
  members.push(member_from_entropy(new Uint8Array(32).fill(i)));
}

// 2. SCALE-encode the members list
const encodedMembers = encodeMembers(members); // see Data Encoding section

// 3. Create an anonymous ring proof
const proverEntropy = new Uint8Array(32).fill(5); // member at index 5
const context = new TextEncoder().encode('my-app');
const message = new TextEncoder().encode('hello');
const RING_EXPONENT = 9; // R2e9 — capacity 255. Matches `pallet-members` on chain.

const result = one_shot(RING_EXPONENT, proverEntropy, encodedMembers, context, message);
// result.proof  - the ring proof (pass this to a verifier)
// result.alias  - your pseudonymous alias in this context
// result.member - your public key

// 4a. Verify the proof from the raw member list
const alias = validate(RING_EXPONENT, result.proof, encodedMembers, context, message);
// alias matches result.alias - proves someone in the ring sent the message

// 4b. Or verify from a pre-built ring commitment (e.g. fetched from `pallet-members`)
// const commitment = /* 768 bytes from api.query.members.root(collectionId, ringIndex) */;
// const alias = validate_with_commitment(RING_EXPONENT, result.proof, commitment, context, message);

// 5. Non-anonymous signatures
const signature = sign(proverEntropy, message);
const valid = verify_signature(signature, message, result.member);
```

---

## Concepts

### Ring Proofs

A ring proof lets a member of a known group prove membership without revealing their identity. Given a list of public keys (the "ring"), a prover generates a proof that convinces a verifier "one of these keys signed this", without revealing which one.

### Aliases

When creating a ring proof, the prover also generates a **context-specific alias** - a 32-byte pseudonymous identifier. The same member always produces the same alias for the same context, but aliases across different contexts are **unlinkable**. This enables:

- **Pseudonymous voting**: same alias in "election-2024" context prevents double-voting
- **Anonymous reputation**: consistent identity within a context, no cross-context tracking

### Entropy

All secret keys are derived from a 32-byte **entropy** value. The same entropy always produces the same secret key and public key (member). Entropy should be generated from a cryptographically secure random source and stored securely.

### Ring Exponent

Ring operations require a `ring_exponent` parameter that controls the maximum number of members the ring can support. Values match the on-chain `RingExponent` enum used by `pallet-members` / `pallet-chunks-manager`.

Capacity formula: `2^x − 257`.

| `ring_exponent` | Chain enum | Max Members | Use Case |
|---|---|---|---|
| `9`  | `R2e9`  | 255    | Testing, small groups |
| `10` | `R2e10` | 767    | Medium groups |
| `14` | `R2e14` | 16,127 | Large groups |

Choose the smallest ring size that fits your ring. Larger sizes increase proof generation and verification time.

Internally the library maps `ring_exponent` to the `verifiable` crate's FFT `RingDomainSize` (9 → Domain11, 10 → Domain12, 14 → Domain16); you never need to pass the FFT domain number directly.

### Multi-Context Proofs

A single proof can cover multiple contexts simultaneously. Instead of generating separate proofs for each context, `create_multi_context` produces one proof with one alias per context. This is more efficient and proves that the same (anonymous) member is acting across all contexts. Up to 16 contexts per proof are supported.

---

## Architecture

### What is a "Ring"?

A **ring** is a set of public keys (members) that collectively form the anonymity set for proof generation. When a member creates a ring proof, the verifier learns that *some* member of the ring created the proof, but not *which* member. The larger the ring, the greater the anonymity.

In practice, a ring is maintained as an ordered list of Bandersnatch public keys. To generate or verify a proof, both parties need access to the same ring (the same set of members in the same order).

### Cryptographic Foundations

This library uses **ring VRFs** (Verifiable Random Functions) built on:

- **Bandersnatch curve**: An elliptic curve defined over the BLS12-381 scalar field, designed for efficient VRF operations in the Polkadot ecosystem
- **Polynomial Commitment Scheme (PCS)**: Uses KZG-style commitments for efficient ring membership proofs
- **Structured Reference String (SRS)**: Pre-computed cryptographic parameters from a trusted setup ceremony, required for proof generation and verification

### Ring Data

Ring operations require pre-computed cryptographic parameters (SRS data and builder parameters). **All ring data for all three domain sizes is compiled directly into the WASM binary** - you do not need to load, fetch, or ship any additional data files.

The embedded data includes:

| Component | R2e9 | R2e10 | R2e14 |
|---|---|---|---|
| Builder params | 49 KB | 98 KB | 1.6 MB |
| Empty commitment | 848 B | 848 B | 848 B |
| SRS (shared) | 4.7 MB (shared across all ring exponents) | | |

This results in a total WASM binary size of approximately **7.3 MB**. All domain sizes are fully functional out of the box.

### Ring Commitment

A **ring commitment** (`MembersCommitment`, 768 bytes) is a compact cryptographic digest of a ring's member list. It is used during proof verification instead of the full member list. Building a commitment involves:

1. `start_members(capacity)` - initialize builder for a given domain size
2. `push_members(...)` - add members and SRS lookup data
3. `finish_members(...)` - finalize into the 768-byte commitment

The `members_root()` JS function performs all three steps.

---

## API Reference

### Key Management

#### `member_from_entropy(entropy: Uint8Array): Uint8Array`

Derives a public key (member) from 32 bytes of entropy.

```typescript
const entropy = new Uint8Array(32);
crypto.getRandomValues(entropy);

const member = member_from_entropy(entropy);
// member is a 32-byte Uint8Array (Bandersnatch public key)
```

#### `is_member_valid(member: Uint8Array): boolean`

Checks whether a 32-byte value is a valid Bandersnatch public key.

```typescript
const member = member_from_entropy(entropy);
is_member_valid(member);  // true

is_member_valid(new Uint8Array(32).fill(0xff));  // false
```

---

### Ring Proofs

#### `one_shot(ring_exponent, entropy, members, context, message): OneShotResult`

Creates a ring proof in a single call. This is the primary function for proof generation.

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `ring_exponent` | `9 \| 10 \| 14` | Ring exponent (R2e9 / R2e10 / R2e14) |
| `entropy` | `Uint8Array` | 32-byte entropy of the prover |
| `members` | `Uint8Array` | SCALE-encoded `Vec<Member>` |
| `context` | `Uint8Array` | Context identifier (arbitrary bytes) |
| `message` | `Uint8Array` | Message to bind to the proof (arbitrary bytes) |

**Returns:** `OneShotResult`

```typescript
const result = one_shot(9, proverEntropy, encodedMembers, context, message);

console.log(result.proof);   // Uint8Array - the ring proof (SCALE-encoded)
console.log(result.alias);   // Uint8Array - 32-byte context-specific alias
console.log(result.member);  // Uint8Array - 32-byte prover public key
console.log(result.members); // Uint8Array - SCALE-encoded members (echo)
console.log(result.context); // Uint8Array - context (echo)
console.log(result.message); // Uint8Array - message (echo)
```

**Throws:** If entropy is invalid, the prover is not in the members list, or proof generation fails.

#### `validate(ring_exponent, proof, members, context, message): Uint8Array`

Validates a ring proof and extracts the prover's alias. This is the primary function for proof verification.

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `ring_exponent` | `9 \| 10 \| 14` | Ring exponent (must match proof creation) |
| `proof` | `Uint8Array` | SCALE-encoded proof from `one_shot` |
| `members` | `Uint8Array` | SCALE-encoded `Vec<Member>` |
| `context` | `Uint8Array` | Context identifier (must match proof creation) |
| `message` | `Uint8Array` | Message (must match proof creation) |

**Returns:** `Uint8Array` - SCALE-encoded 32-byte alias

```typescript
const alias = validate(9, result.proof, encodedMembers, context, message);
// alias matches result.alias from one_shot
```

**Throws:** If the proof is invalid or cannot be decoded.

#### `validate_with_commitment(ring_exponent, proof, commitment, context, message): Uint8Array`

Validates a ring proof against a pre-built 768-byte `MembersCommitment` (ring root). Recommended for chain-adjacent frontends: fetch the root via RPC (`pallet-members::Root`) and pass it directly — saves the commitment-construction step `validate` performs internally from the member list.

**Parameters:**

| Parameter | Type | Description |
|---|---|---|
| `ring_exponent` | `9 \| 10 \| 14` | Ring exponent (must match proof creation) |
| `proof` | `Uint8Array` | SCALE-encoded proof |
| `commitment` | `Uint8Array` | 768-byte SCALE-encoded `MembersCommitment` |
| `context` | `Uint8Array` | Context identifier |
| `message` | `Uint8Array` | Message |

**Returns:** `Uint8Array` - SCALE-encoded 32-byte alias.

```typescript
const commitment = members_root(9, encodedMembers); // or fetch from chain
const alias = validate_with_commitment(9, result.proof, commitment, context, message);
```

**Throws:** If the commitment is malformed or the proof is invalid.

#### `is_valid(ring_exponent, proof, members, context, alias, message): boolean`

Checks whether a ring proof is valid for a given alias, without extracting the alias. Useful when you already know the expected alias and just want a boolean check.

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `ring_exponent` | `9 \| 10 \| 14` | Ring exponent (R2e9 / R2e10 / R2e14) |
| `proof` | `Uint8Array` | SCALE-encoded proof |
| `members` | `Uint8Array` | SCALE-encoded `Vec<Member>` |
| `context` | `Uint8Array` | Context identifier |
| `alias` | `Uint8Array` | Expected 32-byte alias to check against |
| `message` | `Uint8Array` | Message |

**Returns:** `boolean`

```typescript
const valid = is_valid(9, result.proof, encodedMembers, context, result.alias, message);
// true

const invalid = is_valid(9, result.proof, encodedMembers, context, new Uint8Array(32), message);
// false - wrong alias
```

---

### Multi-Context Proofs

#### `create_multi_context(ring_exponent, entropy, members, contexts, message): MultiContextResult`

Creates a single ring proof that covers multiple contexts simultaneously. Each context produces its own unlinkable alias.

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `ring_exponent` | `9 \| 10 \| 14` | Ring exponent (R2e9 / R2e10 / R2e14) |
| `entropy` | `Uint8Array` | 32-byte entropy of the prover |
| `members` | `Uint8Array` | SCALE-encoded `Vec<Member>` |
| `contexts` | `Uint8Array` | SCALE-encoded `Vec<Vec<u8>>` of context identifiers |
| `message` | `Uint8Array` | Message to bind to the proof |

**Returns:** `MultiContextResult`

```typescript
// SCALE-encode contexts
const contexts = scaleEncodeVecVecU8([
  new TextEncoder().encode('voting'),
  new TextEncoder().encode('reputation'),
]);

const result = create_multi_context(9, entropy, encodedMembers, contexts, message);

console.log(result.proof);    // Uint8Array - single proof covering both contexts
console.log(result.aliases);  // Uint8Array - SCALE-encoded Vec<Alias> (one per context)
```

#### `validate_multi_context(ring_exponent, proof, members, contexts, message): Uint8Array`

Validates a multi-context proof and extracts all aliases.

**Returns:** `Uint8Array` - SCALE-encoded `Vec<Alias>` (one 32-byte alias per context)

```typescript
const aliases = validate_multi_context(9, result.proof, encodedMembers, contexts, message);
// SCALE-encoded Vec<[u8; 32]>
```

#### `is_valid_multi_context(ring_exponent, proof, members, contexts, aliases, message): boolean`

Checks whether a multi-context proof is valid for the given aliases.

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `ring_exponent` | `9 \| 10 \| 14` | Ring exponent (R2e9 / R2e10 / R2e14) |
| `proof` | `Uint8Array` | SCALE-encoded proof |
| `members` | `Uint8Array` | SCALE-encoded `Vec<Member>` |
| `contexts` | `Uint8Array` | SCALE-encoded `Vec<Vec<u8>>` |
| `aliases` | `Uint8Array` | SCALE-encoded `Vec<Alias>` to check against |
| `message` | `Uint8Array` | Message |

**Returns:** `boolean`

---

### Batch Validation

#### `batch_validate(ring_exponent, members, proof_items): Uint8Array`

Efficiently validates multiple proofs against the same member set in a single call. More efficient than validating each proof individually because the ring commitment is built only once.

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `ring_exponent` | `9 \| 10 \| 14` | Ring exponent (R2e9 / R2e10 / R2e14) |
| `members` | `Uint8Array` | SCALE-encoded `Vec<Member>` |
| `proof_items` | `Uint8Array` | SCALE-encoded `Vec<(Proof, Vec<u8>, Vec<u8>)>` |

Each tuple in `proof_items` is `(proof_bytes, context_bytes, message_bytes)`, SCALE-encoded.

**Returns:** `Uint8Array` - SCALE-encoded `Vec<Alias>` (one alias per validated proof)

**Throws:** If any proof in the batch is invalid.

---

### Aliases

#### `alias_in_context(entropy: Uint8Array, context: Uint8Array): Uint8Array`

Computes the deterministic alias for a given entropy and context, without creating a ring proof. This is useful for:

- Precomputing what alias a member will have in a given context
- Looking up a member's alias without needing the full member list

The alias returned matches what `one_shot` or `create_multi_context` would produce for the same entropy and context.

```typescript
const alias = alias_in_context(entropy, context);
// 32-byte SCALE-encoded alias

// This matches the alias from a ring proof with the same entropy + context:
const result = one_shot(9, entropy, encodedMembers, context, message);
// alias === result.alias
```

---

### Signatures

Non-anonymous signatures that are directly attributable to a specific member. These are standard signatures, not ring proofs.

#### `sign(entropy: Uint8Array, message: Uint8Array): Uint8Array`

Signs a message using the secret key derived from entropy.

**Returns:** `Uint8Array` - SCALE-encoded signature

```typescript
const signature = sign(entropy, message);
```

#### `verify_signature(signature: Uint8Array, message: Uint8Array, member: Uint8Array): boolean`

Verifies a signature against a message and the signer's public key.

```typescript
const member = member_from_entropy(entropy);
const signature = sign(entropy, message);

verify_signature(signature, message, member);         // true
verify_signature(signature, wrongMessage, member);     // false
verify_signature(signature, message, wrongMember);     // false
```

---

### Ring Commitment

These functions precompute ring commitments for use in chain storage or other scenarios where the commitment is built ahead of time.

#### `members_root(ring_exponent: number, members: Uint8Array): Uint8Array`

Computes the finalized ring commitment (`MembersCommitment`) from a SCALE-encoded member list. This is the compact representation used for on-chain storage and proof verification.

**Returns:** `Uint8Array` - 768-byte commitment

```typescript
const commitment = members_root(9, encodedMembers);
// 768 bytes
```

#### `members_intermediate(ring_exponent: number, members: Uint8Array): Uint8Array`

Computes the intermediate ring builder state (`MembersSet`) from a SCALE-encoded member list. This is the state before finalization, useful for chain genesis or incremental member addition.

**Returns:** `Uint8Array` - 848-byte intermediate

```typescript
const intermediate = members_intermediate(9, encodedMembers);
// 848 bytes
```

---

## Data Encoding

All structured data is exchanged using [SCALE codec](https://docs.substrate.io/reference/scale-codec/) encoding, the standard binary encoding used in the Substrate/Polkadot ecosystem.

### Encoding Members

Members are passed as a SCALE-encoded `Vec<Member>` where each `Member` is a fixed 32-byte public key. The encoding is a compact-encoded length prefix followed by the concatenated member bytes.

```typescript
/**
 * SCALE-encode an array of 32-byte members into Vec<Member>.
 * Each member is a fixed 32-byte Bandersnatch public key (no per-element length prefix).
 */
function encodeMembers(members: Uint8Array[]): Uint8Array {
  const length = members.length;
  let compactLength: Uint8Array;

  if (length < 64) {
    // Single-byte mode: length << 2
    compactLength = new Uint8Array([length << 2]);
  } else if (length < 16384) {
    // Two-byte mode: (length << 2) | 0b01
    compactLength = new Uint8Array([
      ((length & 0x3f) << 2) | 0b01,
      (length >> 6) & 0xff,
    ]);
  } else {
    throw new Error('Too many members for compact encoding');
  }

  // Concatenate: [compact_length, member_0, member_1, ...]
  const result = new Uint8Array(compactLength.length + length * 32);
  result.set(compactLength, 0);
  let offset = compactLength.length;
  for (const member of members) {
    result.set(member, offset);
    offset += 32;
  }
  return result;
}
```

If you are already using the Polkadot.js ecosystem, you can use `@polkadot/util`:

```typescript
import { u8aConcat } from '@polkadot/util';

function encodeMembers(members: Uint8Array[]): Uint8Array {
  const length = members.length;
  const compactLength = length < 64
    ? new Uint8Array([length << 2])
    : new Uint8Array([((length & 0x3f) << 2) | 0b01, (length >> 6) & 0xff]);
  return u8aConcat(compactLength, ...members);
}
```

### Encoding Contexts for Multi-Context Functions

Multi-context functions accept a SCALE-encoded `Vec<Vec<u8>>`. Each inner `Vec<u8>` is a compact-length-prefixed byte string.

```typescript
/**
 * SCALE-encode an array of byte arrays into Vec<Vec<u8>>.
 */
function encodeVecVecU8(items: Uint8Array[]): Uint8Array {
  const parts: Uint8Array[] = [];

  // Outer compact length
  parts.push(compactEncode(items.length));

  // Each inner Vec<u8>: compact_length + bytes
  for (const item of items) {
    parts.push(compactEncode(item.length));
    parts.push(item);
  }

  // Concatenate all parts
  const totalLength = parts.reduce((sum, p) => sum + p.length, 0);
  const result = new Uint8Array(totalLength);
  let offset = 0;
  for (const part of parts) {
    result.set(part, offset);
    offset += part.length;
  }
  return result;
}

function compactEncode(value: number): Uint8Array {
  if (value < 64) {
    return new Uint8Array([value << 2]);
  } else if (value < 16384) {
    return new Uint8Array([
      ((value & 0x3f) << 2) | 0b01,
      (value >> 6) & 0xff,
    ]);
  }
  throw new Error('Value too large for compact encoding');
}
```

### Using `@polkadot/types` for SCALE Encoding

For complex encoding needs, consider using `@polkadot/types`:

```typescript
import { TypeRegistry, Vec, Bytes } from '@polkadot/types';

const registry = new TypeRegistry();

// Encode Vec<Vec<u8>>
const contexts = new Vec(registry, Bytes, [
  new TextEncoder().encode('context-1'),
  new TextEncoder().encode('context-2'),
]);
const encoded = contexts.toU8a();
```

---

## TypeScript Types

```typescript
/** On-chain `RingExponent`. Capacity formula: 2^x − 257. */
type RingExponent = 9 | 10 | 14;

/** Result from one_shot() proof creation. */
interface OneShotResult {
  proof: Uint8Array;    // SCALE-encoded ring proof
  alias: Uint8Array;    // 32-byte context-specific alias
  member: Uint8Array;   // 32-byte prover public key
  members: Uint8Array;  // SCALE-encoded members list (echo)
  context: Uint8Array;  // Context bytes (echo)
  message: Uint8Array;  // Message bytes (echo)
}

/** Result from create_multi_context() proof creation. */
interface MultiContextResult {
  proof: Uint8Array;    // SCALE-encoded ring proof
  aliases: Uint8Array;  // SCALE-encoded Vec<Alias>
  member: Uint8Array;   // 32-byte prover public key
  members: Uint8Array;  // SCALE-encoded members list (echo)
  contexts: Uint8Array; // SCALE-encoded contexts (echo)
  message: Uint8Array;  // Message bytes (echo)
}
```

---

## Platform Support

The package provides two build targets:

### Bundler (Browser)

For Webpack, Vite, Rollup, and other bundlers:

```typescript
import { one_shot, validate } from 'verifiablejs/bundler';
```

Requires a bundler that supports WebAssembly ESM integration. For Vite, use the [`vite-plugin-wasm`](https://www.npmjs.com/package/vite-plugin-wasm) and [`vite-plugin-top-level-await`](https://www.npmjs.com/package/vite-plugin-top-level-await) plugins.

### Node.js / Bun

```typescript
import { one_shot, validate } from 'verifiablejs/nodejs';
```

Works with Node.js 18+ and Bun.

---

## Development

### Monorepo Structure

```
verifiablejs/
  packages/verifiablejs/   # Main WASM package (Rust + wasm-bindgen, published to npm)
  playground/vite/          # Browser example (Vite)
  playground/bun/           # Node.js/Bun example
```

### Prerequisites

- [Rust](https://rustup.rs/) with `wasm32-unknown-unknown` target (`rustup target add wasm32-unknown-unknown`)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/) (v0.13+)
- [pnpm](https://pnpm.io/) (v10+)
- Node.js 18+

### Commands

```sh
# Install dependencies
pnpm install

# Build the WASM package (both bundler and Node.js targets)
pnpm build

# Run tests
pnpm test

# Run the Vite playground (browser)
pnpm dev:vite

# Run the Bun/Node playground
pnpm dev:bun
```

### Releasing

This project uses [Changesets](https://github.com/changesets/changesets) with automated CI/CD:

1. Create a changeset (`pnpm changeset`) and push/merge to `main`
2. CI automatically creates a "chore: version packages" PR (bumps version, updates CHANGELOG)
3. Merge the version PR
4. Create a **GitHub Release** with tag `vX.Y.Z` to trigger npm publish

---

## License

Licensed under GPL-3.0-or-later WITH Classpath-exception-2.0
