# Bandersnatch Verifiable JS Bindings

This project produces javascript bindings for the [`paritytech/verifiable`](https://github.com/paritytech/verifiable) library.

These bindings are used in the [`paritytech/web3-citizenship-web`](https://github.com/paritytech/web3-citizenship-web) project.

FYI in the current state of these bindings the `small-ring` feature is enabled by default and must be adapted if intended to be used with another ring size.

## Building

```sh
wasm-pack build --release --target web
```

## Testing

```sh
wasm-pack test --node
```
