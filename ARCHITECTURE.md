# Verifiable JS - Architecture Reference

> **This file is gitignored.** Internal reference for developers and LLM context.

## Package Architecture

### Packages

| Package | Size | Contains | Ring data |
|---------|------|----------|-----------|
| `verifiablejs` (core) | ~1 MB | Verification, signatures, key ops, alias | None embedded; accepts external chunks or pre-built commitment |
| `verifiablejs-prover` | ~7 MB | Proof generation (`one_shot`, `create_multi_context`) | All SRS + builder params embedded |
| `@verifiablejs/ring-data-domain11` | ~49 KB | Builder params for Domain11 (max ~255 members) | Raw binary |
| `@verifiablejs/ring-data-domain12` | ~98 KB | Builder params for Domain12 (max ~767 members) | Raw binary |
| `@verifiablejs/ring-data-domain16` | ~1.6 MB | Builder params for Domain16 (max ~16,127 members) | Raw binary |

Packages are **independent** (no re-exports between core and prover).

### Usage Patterns

```typescript
// Pattern 1: Verify with pre-built commitment from chain (most common, ~1 MB download)
import { validate_with_commitment } from 'verifiablejs/bundler';
const commitment = await api.query.members.root(collectionId, ringIndex);
const alias = validate_with_commitment(11, proof, commitment.toU8a(), context, message);

// Pattern 2: Verify from raw members offline (~1 MB + 49 KB)
import { validate } from 'verifiablejs/bundler';
import domain11 from '@verifiablejs/ring-data-domain11';
const alias = validate(11, proof, encodedMembers, context, message, domain11);

// Pattern 3: Generate proofs (~7 MB, prover package)
import { one_shot } from 'verifiablejs-prover/bundler';
const result = one_shot(11, entropy, encodedMembers, context, message);
```

### Why This Split Works Without Upstream Changes

The upstream `verifiable` crate uses feature flags:
- `prover` feature gates `open()`, `create()`, and SRS data embedding
- `builder-params` (or `std`) gates `ring_verifier_builder_params()` and builder params embedding
- `push_members()`, `validate()`, `start_members()`, `finish_members()` are ALWAYS available
- `StaticChunk`, `RingSize`, `RingDomainSize` types are ALWAYS available
- `ark_vrf::ring::RingBuilderPcsParams` deserialization code is ALWAYS compiled (only the data is feature-gated)

The core package:
1. Builds with `verifiable` default-features = false (no prover, no builder-params, no std)
2. Accepts builder params as raw bytes from JS when needed
3. Deserializes them using ark-serialize (always available)
4. Passes them to `push_members()` via closure
5. For `validate_with_commitment()`, skips commitment building entirely

### WASM Binary Size Breakdown

| Component | Size | Feature gate | Core | Prover |
|-----------|------|-------------|------|--------|
| SRS (`srs-uncompressed.bin`) | 4.7 MB | `prover` | excluded | included |
| Builder params (3 domains) | 1.7 MB | `std` / `builder-params` | excluded | included |
| Empty ring commitments | 2.5 KB | none | included | included |
| Rust/WASM code | ~0.9 MB | none | included | included |

### Critical Verifier Cache Behavior

When `prover` is **disabled**, `BandersnatchVerifierCache::get()` constructs the ring context from the domain size alone - no data loading. This is lightweight and fast.

When `prover` is **enabled**, the verifier cache delegates to the prover cache, loading SRS even for pure verification. This is why the full build is 7.3 MB even when only verifying.

---

## On-Chain Integration (pallet-members)

> Note: The individuality/pallet-members code is not yet public.

### How pallet-members Uses the Verifiable Crate

The pallet abstracts crypto through `GenerateVerifiable`:
```rust
type Crypto: GenerateVerifiable<
    Capacity: TryFrom<RingExponent>,
    ...
>;
```

### Ring Storage

- `Root<T>`: `DoubleMap (Identifier, RingIndex) -> RingRoot { root, revision, intermediate }`
- `RingKeys<T>`: `NMap (Identifier, RingIndex, PageIndex) -> BoundedVec<Member>`
- `Members<T>`: `DoubleMap (Identifier, Member) -> RingPosition`
- `OldRoots<T>`: Archived roots valid for grace period (default 10 min)

### Ring Building

Built incrementally across blocks via offchain workers:
1. `start_members(capacity)`
2. `push_members(&mut inter, keys, |range| fetch_chunks(ring_exponent, range))`
3. `finish_members(inter)`

SRS chunks loaded on-demand via `ChunksManager` pallet.

### Proof Verification

```rust
let alias = T::Crypto::validate(capacity, proof, &ring.root, &context[..], msg)?;
let aliases = T::Crypto::batch_validate(capacity, &ring.root, items)?;
```

### ChunksManager Pallet

- Storage: `DoubleMap (RingExponent, PageIndex) -> BoundedVec<Chunk>`
- Queryable from JS: `api.query.chunksManager.chunks(ringExponent, pageIndex)`
- 255 chunks per page, immutable once set
- Hashes verified during upload via `ChunkPageHashes`

### Ring Domain Size Mapping

| Ring Exponent | Domain Size param | Capacity |
|---------------|-------------------|----------|
| R2e9 | 11 | 255 |
| R2e10 | 12 | 767 |
| R2e14 | 16 | 16,127 |

Formula: `capacity = 2^exponent - 257`

### Data Flow

```
Client (verifiablejs)                     Chain (pallet-members)
---------------------                     ----------------------
                                          Ring stored on-chain:
                                          +---------------------+
                                          | members: [pk0..pkN] |
                                          | root: 768-byte      |
                                          | revision: u32       |
                                          +---------------------+
                                                    |
1. Fetch ring root (768 bytes)  <-------------------+
   from chain state (via RPC)

2a. validate_with_commitment(     OR      2b. Pallet verifies directly:
      domain, proof, root,                    T::Crypto::validate(
      context, msg)                             capacity, proof, &root,
    -> alias                                    context, msg) -> alias

3. If generating proof (prover pkg):
   one_shot(domain, entropy,
     members, context, msg)
   -> { proof, alias }
   Submit proof to chain ------------>    4. On-chain verification
```

### Ring Data: Client vs Chain

| Aspect | verifiablejs (core) | verifiablejs-prover | pallet-members |
|--------|--------------------|--------------------|----------------|
| SRS data | Not needed | Embedded (4.7 MB) | Via ChunksManager |
| Builder params | External (ring-data pkg) | Embedded (1.7 MB) | Via ChunksManager |
| Ring commitment | From chain or built locally | Built locally | Built incrementally |
| Proof generation | N/A | `one_shot()` | N/A (client-side) |
| Proof verification | `validate_with_commitment()` | `validate()` | `T::Crypto::validate()` |
