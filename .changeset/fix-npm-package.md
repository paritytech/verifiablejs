---
"verifiablejs": patch
---

Replace web target with bundler target and fix npm package distribution. Package now correctly includes pkg-bundler and pkg-nodejs folders with all WASM files. Exports changed to explicit imports: verifiablejs/bundler and verifiablejs/nodejs.
