# Verifiable JS

JavaScript/TypeScript bindings for the [Parity Verifiable](https://github.com/paritytech/verifiable) crate.

## Installation

```bash
npm install verifiablejs
```

The package is published on npm as [`verifiablejs`](https://www.npmjs.com/package/verifiablejs).

For full API documentation, usage examples, and features, see the **[package README](./packages/verifiablejs/README.md)**.

---

## Monorepo Structure

This is a monorepo containing:

- [`packages/verifiablejs`](./packages/verifiablejs) - The main package (published to npm)
- [`playground/vite`](./playground/vite) - Vite/bundler playground example
- [`playground/bun`](./playground/bun) - Node.js/Bun playground example

## Development

```sh
# Install dependencies
pnpm install

# Build verifiablejs package
pnpm build

# Run Vite playground
pnpm dev:vite

# Run Bun/Node playground
pnpm dev:bun
```

## Releasing

This project uses [Changesets](https://github.com/changesets/changesets) for version management.

1. **Create a changeset**: `pnpm changeset`
2. **Version bump**: `pnpm version`
3. **Publish**: `pnpm release`
