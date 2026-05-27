---
"verifiablejs": minor
---

`one_shot().proof` and the three on-chain-proof consumers (`validate_with_commitment`, `validate`, `is_valid`) now use the **raw canonical proof bytes** — no SCALE compact-length prefix.

### What changed and why

For Bandersnatch ring-VRF, `one_shot` previously returned 787 bytes (a 2-byte SCALE compact-length prefix followed by 785 raw canonical bytes). That was a regression introduced unintentionally when bumping the underlying `verifiable` crate to 0.3 in v1.3.0-alpha (the `Proof` type changed from `Vec<u8>` to `BoundedVec<u8, …>` and `Encode::encode(&proof)` was used to convert to `Uint8Array`, which prepended the length prefix). The original behaviour (pre-Apr 2026) was to return raw bytes via `&proof[..]`.

The prefix breaks every on-chain consumer: the runtime's `RingVrfSignature::deserialize_canonical` (in `verifiable/src/ring/mod.rs`) uses ark-serialize directly and explicitly rejects trailing bytes, so the 2-byte SCALE prefix becomes garbage at the start of the buffer and the verifier returns `Error::DecodeError` → `InvalidTransaction::BadProof`. The bug was silent because:

- `validate_with_commitment` was using `Decode::decode` which stripped the prefix, so local pre-flight validation against the chain's stored ring root always succeeded.
- `sign()` was unaffected — its underlying `[u8; 64]` SCALE-encodes raw (fixed-size arrays have no length prefix), so the PersonalId-bind path kept working.
- The chain returns the umbrella `BadProof` error with no detail.

This change restores symmetry between `sign` (always raw) and `one_shot.proof` (now raw), and makes the natural usage pattern — "take the proof, put it on the wire, submit" — Just Work against any chain that uses `verifiable`'s canonical deserialize.

### Breaking changes

- **`one_shot().proof` is now 2 bytes shorter** (for Bandersnatch: 785 bytes instead of 787) and carries no SCALE length prefix. Any consumer that was previously stripping the prefix manually (e.g. `proof.slice(2)` as a chain-submission workaround) must drop the `.slice(2)` after upgrading.
- **`validate_with_commitment`, `validate`, and `is_valid`** now expect raw canonical proof bytes (matching the new `one_shot` output). Existing callers passing the round-tripped output from a v1.3.0-beta.* `one_shot` will start seeing `"Proof exceeds maximum bounded size"` errors — pass the raw form instead.
- `batch_validate` is unchanged: it accepts a SCALE-encoded `Vec<(Proof, Vec<u8>, Vec<u8>)>` where `Proof` is `BoundedVec<u8, _>`, and SCALE-encoding the tuple naturally produces the right prefix on each proof. Callers constructing the input from `one_shot` outputs need to wrap each proof into the `BoundedVec` via `try_from(Vec<u8>)` before SCALE-encoding the outer tuple (no `Decode::decode` on the proof bytes).
