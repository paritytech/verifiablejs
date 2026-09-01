---
"verifiablejs": minor
---

Bump the `verifiable` dependency to the published crates.io release `0.3.0` (previously pinned to git rev `1f9f6752`), switching from a git pin to a proper versioned dependency.

The release includes the changes since the old pin, most notably:

- The insecure deterministic no-std prover is replaced with a blinded no-std prover, and feature unification can no longer disable ring-proof blinding.
- New `batch_validate_per_item` with per-item failure attribution.
- `ark-vrf` bumped to `0.5.3` (with a matching `ark-scale` bump).

The JS API surface is unchanged.
