# SovereignDrive Core

SovereignDrive Core is the cross-platform storage, vault, and agent-workspace foundation for the Sovereign ecosystem.

## Current production-readiness track

The `feat/production-readiness` branch hardens the Rust engine before application and agent integrations are promoted.

### Engine guarantees being established
- Content-addressed storage using BLAKE3 over canonical content.
- Optional zstd compression without changing content identity.
- Durable CAS objects with atomic filesystem commits.
- Integrity verification when content is read.
- Encrypted vaults using Argon2-derived keys and XChaCha20-Poly1305.
- C ABI with null-safe input handling and explicit string ownership.
- Thin-volume primitives kept separate from physical storage.
- Cargo integration tests plus formatting, Clippy, and dependency-audit CI.

## Development

```bash
./scripts/test.sh
```

The Rust engine lives in `engine/` and is the authoritative data-plane implementation. Desktop, iOS, wallet, and agent surfaces should consume engine contracts rather than duplicate storage semantics.

## Architecture

```text
Desktop / iOS / Agents
          |
     Stable API/FFI
          |
   SovereignDrive Engine
      /     |      \
     CAS   Vault   Volumes
```

Application integrations remain deliberately staged until the engine passes the core persistence, security, reliability, and release gates.
