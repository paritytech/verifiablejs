# Verifiable JS Monorepo

Monorepo for [verifiablejs](./packages/verifiablejs) package and playgrounds.

## Structure

- `packages/verifiablejs` - WebAssembly bindings for the Parity Verifiable crate
- `playground/vite` - Vite/bundler playground example
- `playground/bun` - Node.js/Bun playground example

## Quick Start

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

## Package Documentation

See [packages/verifiablejs/README.md](./packages/verifiablejs/README.md) for full API documentation.

## Releasing

This project uses [Changesets](https://github.com/changesets/changesets) for version management.

1. **Create a changeset**: `pnpm changeset`
2. **Version bump**: `pnpm version`
3. **Publish**: `pnpm release`
