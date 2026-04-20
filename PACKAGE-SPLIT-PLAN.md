# Package Split Plan — Lightweight Core + Prover + Ring Data

Living document tracking the split of the current 7.3 MB `verifiablejs` package into `@verifiable/core`, `@verifiable/prover`, and per-ring-size `@verifiable/ring-data-r2e{9,10,14}` packages.

Originally reviewed and scoped with the user on **2026-04-20**. Update this file as phases complete.

## Context

The current `verifiablejs` package ships a single 7.3 MB WASM binary containing:
- 4.7 MB SRS data (only needed for proof generation)
- 1.7 MB builder params across three domains (only needed when building a ring commitment locally)
- ~0.9 MB code + 2.5 KB empty commitments

For the most common browser use case — a frontend verifying proofs against `pallet-members` via a chain-fetched ring root — ~6 MB of payload is dead weight. The split delivers a ~1 MB core for the common path while keeping proof generation intact.

See also `ARCHITECTURE.md` for the target architecture diagram and integration notes with `pallet-members` / `pallet-chunks-manager`.

## Rating: 8/10 — proceed

- **Why it makes sense**
  1. 7× size reduction for the common browser path (chain-fetched commitment → verify).
  2. No upstream changes to the `verifiable` crate needed; existing feature gates (`prover`, `builder-params`, `std`) give us what we need.
  3. Clean architectural boundary matching actual usage patterns (verify-with-commitment, verify-with-chunks, generate-proofs).
  4. `validate_with_commitment()` is the right new primitive — unlocks pallet-members integration without requiring chunk data on the client.
- **Remaining execution risks** (mitigations built into the phases below)
  1. `ark-serialize` under `no_std + alloc` is the load-bearing claim — mitigated by Phase 0 prototype.
  2. Verifiable crate rev drift: plan doc was written against `8b40930`, current `Cargo.toml` pins `cfb9704`. Phase 0 re-verifies.
  3. Breaking API change; v2.0.0 bump and migration guide (Phase 6).
  4. Cross-package roundtrip test must land in CI, not just exist locally (Phase 5).
  5. Pallet-members is aspirational (chain not public yet). README should be honest about that.

## Decisions (frozen)

All confirmed with the user 2026-04-20. Do not re-litigate without agreement.

- **No shared Rust crate.** Keep each wasm-bindgen package self-contained. ~100 lines of duplicated JS-binding adapters (sign/verify/member helpers) is acceptable; YAGNI beats a third crate.
- **npm scope: `@verifiable/`** (drop the "js" suffix). Three packages: `@verifiable/core`, `@verifiable/prover`, `@verifiable/ring-data-r2e9`, `@verifiable/ring-data-r2e10`, `@verifiable/ring-data-r2e14`. Scope verified unclaimed on npm (2026-04-20) — **reserve immediately** before publishing.
- **Ring-data format:** dual `./nodejs` + `./bundler` exports, mirroring the current `verifiablejs` package pattern. Node reads `.bin` via `fs`; bundler entry inlines base64 for universal compatibility.
- **Bin source:** `.bin` bytes checked in + SHA256 manifest + optional CI verify-against-upstream job (script re-extracts and compares hash).
- **Ring-size convention:** public JS API exposes `RingExponent` values (`9 | 10 | 14`), matching `pallet-members` / `pallet-chunks-manager` on chain. Internally translated to `RingDomainSize::Domain{11,12,16}` when calling into the `verifiable` crate — JS callers never see the FFT domain number.
- **Rollout:** **hard cutover**, no soft-migration package. The v1 user base is small and internal; we'll notify them directly. `verifiablejs@1.x` freezes as-is; v2 ships only under `@verifiable/core`. Mark the old `verifiablejs` npm package as deprecated (install-time warning) after v2 publishes.

## Naming: RingExponent vs RingDomainSize

Two conventions exist for the same three ring sizes:

| Public (JS, chain) | Internal (verifiable crate) | Capacity (`2^x − 257`) |
|---|---|---|
| R2e9  (`9`)  | Domain11 | 255 |
| R2e10 (`10`) | Domain12 | 767 |
| R2e14 (`14`) | Domain16 | 16,127 |

The chain's `RingExponent` (source: `individuality/support/src/traits/reality.rs:73-80`) is the *capacity* exponent; callers who query `pallet-chunks-manager::chunks(ring_exponent, page_index)` pass `9`/`10`/`14`. The `verifiable` crate's `RingDomainSize` is the *FFT domain* exponent — bigger than the ring capacity to accommodate KZG padding. The chain maintains a private mapping in `RingExponent::domain_size()` (`reality.rs:108-114`); we mirror that translation inside `verifiablejs` so JS callers don't have to.

**Impact on the API:**
- `parse_domain_size(u32)` becomes `parse_ring_exponent(u32)` accepting `9 | 10 | 14`; TypeScript type `RingExponent = 9 | 10 | 14`.
- First parameter of `validate`, `one_shot`, `members_root`, etc. renames from `domain_size` to `ring_exponent` (breaking — v2.0.0).
- Ring-data packages suffix matches: `ring-data-r2e9`, `ring-data-r2e10`, `ring-data-r2e14`.
- Documentation calls out the capacity formula prominently; the FFT domain mapping is an internal note.

## Target Architecture

### Packages

```
packages/
  core/                     ~1 MB   Verification, signatures, key ops (@verifiable/core)
  prover/                   ~7 MB   Proof generation, self-contained (@verifiable/prover)
  ring-data-r2e9/           ~49 KB  Builder params for R2e9  (@verifiable/ring-data-r2e9)
  ring-data-r2e10/          ~98 KB  Builder params for R2e10 (@verifiable/ring-data-r2e10)
  ring-data-r2e14/          ~1.6 MB Builder params for R2e14 (@verifiable/ring-data-r2e14)
```

### Size Breakdown

| Component | Size | Core | Prover |
|-----------|------|------|--------|
| SRS (`srs-uncompressed.bin`) | 4.7 MB | excluded | included |
| Builder params (3 domains) | 1.7 MB | excluded | included |
| Empty ring commitments | 2.5 KB | included | included |
| Rust/WASM code | ~0.9 MB | included | included |
| **Total** | | **~1 MB** | **~7 MB** |

### Usage Patterns

```typescript
// Pattern 1: Verify with pre-built commitment from chain (~1 MB download)
import { validate_with_commitment } from '@verifiable/core/bundler';
const commitment = /* 768 bytes fetched from pallet-members ring root */;
const alias = validate_with_commitment(9, proof, commitment, context, message);

// Pattern 2: Verify from raw members, offline (~1 MB + 49 KB)
import { validate } from '@verifiable/core/bundler';
import r2e9 from '@verifiable/ring-data-r2e9/bundler';
const alias = validate(9, proof, encodedMembers, context, message, r2e9);

// Pattern 3: Generate proofs (~7 MB download)
import { one_shot } from '@verifiable/prover/bundler';
const result = one_shot(9, entropy, encodedMembers, context, message);
```

## Implementation Phases

Sequential — each phase de-risks the next. Don't parallelize; early failures should stop the train.

### Phase 0 — De-risk the core technical claim (0.5 day)

- In a throwaway branch, create a minimal crate with `verifiable = { rev = "cfb9704", default-features = false }` and try to call `RingBuilderPcsParams::deserialize_uncompressed_unchecked(&bytes)`.
- Build for `wasm32-unknown-unknown`. Confirm no `std` dependency creep.
- **Exit criteria:** compiles and deserializes. If not, stop and investigate upstream before committing to the split.

### Phase 1 — Extract builder-params data (0.5 day)

- Write `tools/extract-builder-params/` as a small Rust binary that calls `ring_verifier_builder_params()` for each domain and serializes via ark-serialize to `data.bin`.
- Run it once per ring size, commit the three `.bin` files into the ring-data packages.
- Record SHA256 of each `.bin` in a `manifest.json` alongside the data.
- **Critical files (new):**
  - `tools/extract-builder-params/Cargo.toml`
  - `tools/extract-builder-params/src/main.rs`
  - `packages/ring-data-r2e{9,10,14}/data.bin`
  - `packages/ring-data-r2e{9,10,14}/manifest.json`

### Phase 2 — Rework core package (2 days)

- Rename `packages/verifiablejs/` → `packages/core/` (npm name `@verifiable/core`).
- `Cargo.toml`: switch to `default-features = []`, drop the `std` feature, add `ark-serialize = { default-features = false, features = ["alloc"] }`.
- `src/lib.rs`:
  - Delete `one_shot`, `create_multi_context` (move to prover).
  - Rename `parse_domain_size(u32) -> RingDomainSize` to `parse_ring_exponent(u32) -> RingDomainSize` accepting `9 | 10 | 14` and mapping internally to `Domain{11,12,16}`. Rename every `domain_size` parameter and TS type to `ring_exponent` / `RingExponent`.
  - Replace embedded `ring_verifier_builder_params()` calls with a new `build_members_commitment_from_chunks` helper (template: current `lib.rs:43-62`, only the closure source changes).
  - Add `validate_with_commitment(ring_exponent, proof, commitment_bytes, context, message)`.
  - Thread a new `chunks: Uint8Array` arg through `validate`, `validate_multi_context`, `is_valid`, `is_valid_multi_context`, `batch_validate`, `members_root`, `members_intermediate`.
  - Keep `sign`, `verify_signature`, `member_from_entropy`, `alias_in_context`, `is_member_valid` unchanged.
- **Exit criteria:** `wasm-pack build --release --target bundler` produces a ~1 MB binary; tests adapted to pass `ring_exponent` + `chunks` pass.
- **Critical files:**
  - `packages/core/Cargo.toml`
  - `packages/core/src/lib.rs`
  - `packages/core/package.json` (name `@verifiable/core`, version `2.0.0`)

### Phase 3 — Create prover package (1 day)

- `packages/prover/` as a new crate. `Cargo.toml` uses `verifiable` with default features (prover + builder-params + std).
- `src/lib.rs` is largely a copy of today's `lib.rs` with the same `domain_size → ring_exponent` rename, without the `chunks` threading — self-contained with embedded data, so prover-only users don't need ring-data packages.
- Contains `one_shot`, `create_multi_context`, plus verifier/signature functions so a proof-generating app can self-test.
- **Exit criteria:** `wasm-pack build` produces ~7 MB binary matching today's size; same `RingExponent` (9/10/14) API as core.
- **Critical files (new):**
  - `packages/prover/Cargo.toml`
  - `packages/prover/src/lib.rs`
  - `packages/prover/package.json` (name `@verifiable/prover`)

### Phase 4 — Ring-data JS packages (0.5 day)

- `packages/ring-data-r2e{9,10,14}/`: each a pure JS/TS package exporting a `Uint8Array`.
- Two entry points per package: `./nodejs` (reads `.bin` via `fs`) and `./bundler` (base64-inline for widest bundler compatibility).
- Package sizes: `r2e9` ~49 KB, `r2e10` ~98 KB, `r2e14` ~1.6 MB (raw) — add ~33% when base64-inlined for bundler entry.
- **Critical files (new):**
  - `packages/ring-data-r2e{9,10,14}/package.json` (names `@verifiable/ring-data-r2e9`, `@verifiable/ring-data-r2e10`, `@verifiable/ring-data-r2e14`)
  - `packages/ring-data-r2e{9,10,14}/index-node.js` + `index-bundler.js`
  - `packages/ring-data-r2e{9,10,14}/data.bin`

### Phase 5 — Cross-package integration test + playgrounds (1 day)

- Workspace-level test: prover.one_shot → core.validate_with_commitment, prover.one_shot → core.validate with ring-data chunks. Run in CI on every PR.
- Update `playground/bun/index.ts` to demonstrate the three patterns.
- Update `playground/vite/` to exercise the bundler target and visually verify bundle size in the browser devtools.
- **Critical files:**
  - `packages/tests/cross-package.test.ts` (new)
  - `playground/bun/index.ts`
  - `playground/vite/main.ts`

### Phase 6 — Hard cutover + docs + changesets (0.5 day)

- **Before publishing anything:** reserve the `@verifiable` scope on npm (currently unclaimed as of 2026-04-20).
- **Hard cutover — no deprecation alias.** `verifiablejs@1.x` freezes; v2 ships only under `@verifiable/core`. Notify the internal user base directly.
- After v2 publishes, run `npm deprecate verifiablejs@"*" "Moved to @verifiable/core — see MIGRATING.md"` to produce install-time warnings.
- Update root `README.md` with the three usage patterns (chain commitment / offline with chunks / prover) and a migration section for v1 users explaining the new package names, `domain_size → ring_exponent` rename, and value translation (11→9, 12→10, 16→14).
- Write `MIGRATING.md` covering: the `chunks` arg, the new `validate_with_commitment`, the parameter rename, and the ring-size value translation. Include before/after code snippets.
- Generate changesets for v2 core, v1 prover, v1 ring-data packages.

## Critical files to reference (reuse, don't reinvent)

- `packages/verifiablejs/src/lib.rs:43-62` — `build_members_commitment` is the template for the core's new `_from_chunks` version; only the closure source changes.
- `packages/verifiablejs/src/lib.rs:96-243` — `one_shot` / `create_multi_context` move verbatim to prover (with parameter rename).
- `packages/verifiablejs/src/lib.rs:403-471` — `sign` / `verify_signature` / `alias_in_context` / `is_member_valid` — pure, feature-gate-free, safe to keep in core unchanged.
- `packages/verifiablejs/package.json:7-16` — existing `./bundler` + `./nodejs` exports pattern; mirror in prover and ring-data.
- `individuality/support/src/traits/reality.rs:73-114` — authoritative `RingExponent` enum and its `domain_size()` mapping. Source of truth for the JS API.

## Verification plan (how to know it worked)

1. `ls -lh packages/core/pkg-bundler/*.wasm` → expect ~1 MB.
2. `ls -lh packages/prover/pkg-bundler/*.wasm` → expect ~7 MB (same as today).
3. Cross-package test passes: proof generated by prover validates in core (both with commitment and with chunks).
4. `wasm-pack test --node` passes in both packages.
5. Bun playground runs all three patterns end-to-end.
6. Vite playground loads; network tab shows the core bundle under 1.5 MB (1 MB wasm + JS glue).
7. SHA256 of each `ring-data-r2e{9,10,14}/data.bin` matches its `manifest.json` entry and matches the output of `ring_verifier_builder_params(domainXX).serialize_uncompressed()` on the pinned verifiable rev.

## Progress tracking

Update these boxes as phases complete. Future sessions pick up from the first unchecked phase.

- [ ] **Phase 0** — De-risk `ark-serialize` under `no_std + alloc`
- [ ] **Phase 1** — Extract builder-params `.bin` files + hash manifest
- [ ] **Phase 2** — Rework core package (`@verifiable/core`)
- [ ] **Phase 3** — Create prover package (`@verifiable/prover`)
- [ ] **Phase 4** — Ring-data JS packages (`@verifiable/ring-data-r2e{9,10,14}`)
- [ ] **Phase 5** — Cross-package integration test + playgrounds
- [ ] **Phase 6** — Hard cutover, docs, changesets
- [ ] **Reserve `@verifiable` npm scope** (blocker for publishing — do this early)
