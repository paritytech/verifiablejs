# Verifiable WASM

WebAssembly bindings for the [Parity Verifiable](https://github.com/paritytech/verifiable) crate.

This package provides JavaScript/TypeScript bindings for the Verifiable crate's cryptographic proof of membership functionality.

## Features

- Generate proofs of membership in a set with known members
- Validate proofs
- Sign and verify messages
- All cryptographic operations are performed using the Bandersnatch curve implementation

## Installation

```bash
npm install @parity/verifiable-wasm
```

## Usage

```typescript
import init, { one_shot, validate, sign, verify_signature, member_from_entropy } from '@parity/verifiable-wasm';

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

## License

Licensed under GPL-3.0-or-later WITH Classpath-exception-2.0
