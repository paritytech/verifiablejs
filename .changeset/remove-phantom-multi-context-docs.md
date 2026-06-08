---
"verifiablejs": patch
---

Remove documentation for multi-context proof functions (`create_multi_context`, `validate_multi_context`, `is_valid_multi_context`, and the `MultiContextResult` type) from both READMEs. These were documented as part of the public API but were never implemented in the WASM bindings — only a similarly named test (`test_multi_context_aliases_are_unlinkable`, which exercises `alias_in_context`) exists. Documenting a non-existent API was misleading; the docs now reflect the functions the package actually ships.
