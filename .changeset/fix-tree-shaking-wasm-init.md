---
"verifiablejs": patch
---

Fix `sideEffects` so production bundlers don't tree-shake the WASM init.

The bundler entry (`pkg-bundler/verifiablejs.js`) calls `__wbg_set_wasm(wasm)` and `wasm.__wbindgen_start()` at the top level — these bind the WASM namespace and are required for any exported function to work. The previous `"sideEffects": false` told Rollup/Rolldown those statements were safe to drop, breaking production builds (e.g. `member_from_entropy is undefined`). Vite dev was unaffected because it doesn't tree-shake.

`sideEffects` now lists the two bundler files that have top-level side effects, preserving tree-shaking for everything else.
