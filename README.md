# Verifiable JS

This package provides JavaScript/TypeScript bindings for the [Parity Verifiable](https://github.com/paritytech/verifiable) crate.

## Features

- Generate proofs of membership in a set with known members
- Validate proofs
- Sign and verify messages
- All cryptographic operations are performed using the Bandersnatch curve implementation

## Installation

```bash
npm install verifiablejs
```

## Usage

```typescript
import init, { one_shot, validate, sign, verify_signature, member_from_entropy } from 'verifiablejs';

// Initialize the WASM module
await init();

// Generate a proof
const entropy = new Uint8Array(32); // Your entropy bytes
const members = new Uint8Array(...); // Your encoded members list
const context = new TextEncoder().encode("my-context");
const message = new TextEncoder().encode("my-message");

const result = one_shot(entropy, members, context, message);
const { proof, alias, member } = result;

// Validate a proof
const validatedAlias = validate(proof, members, context, message);

// Sign a message
const signature = sign(entropy, message);

// Verify a signature
const isValid = verify_signature(signature, message, member);
```

## Building

```sh
npm run build
```

This will run `wasm-pack build --release --target web --features small-ring`

## Testing

```sh
npm test
```

This will run both Rust and WASM tests:
- `cargo test --features "small-ring"`
- `wasm-pack test --node --features small-ring`

## Releasing

This project uses [Changesets](https://github.com/changesets/changesets) for version management.

1. **Create a changeset**: `pnpm changeset` (select bump type: patch/minor/major)
2. **Version bump**: `pnpm version` (consumes changesets, updates package.json and CHANGELOG.md)
3. **Publish**: `pnpm release` (builds and publishes to npm)

## License

Licensed under GPL-3.0-or-later WITH Classpath-exception-2.0
