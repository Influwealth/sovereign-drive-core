# SovereignDrive WASM bindings

This crate is the browser-facing bridge for SovereignDrive Core.

## Current scope

- in-memory SovereignDrive engine instance
- `ingest(Uint8Array) -> hash`
- `read(hash) -> Uint8Array`
- binding version reporting

Persistent filesystem access intentionally remains outside this layer. Sprint 2 will add the WebContainer filesystem adapter and TypeScript package around these bindings.

## Target

```bash
rustup target add wasm32-unknown-unknown
cargo check --manifest-path bindings/wasm/Cargo.toml --target wasm32-unknown-unknown
```

The generated WASM package will be consumed by the StackBlitz/WebContainer runtime through a small TypeScript facade, keeping the Rust engine as the authoritative data plane.
