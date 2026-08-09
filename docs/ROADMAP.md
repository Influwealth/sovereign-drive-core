# SovereignDrive Core production roadmap

## Sprint 1 — Core reliability/security

- [x] Durable CAS baseline
- [x] Atomic object commit
- [x] Read-time BLAKE3 integrity verification
- [x] Temporary-object recovery
- [x] Concurrent duplicate-write coverage
- [x] Argon2id + XChaCha20-Poly1305 vault baseline
- [x] Property-test and fuzz-test scaffolding
- [ ] Green CI validation on GitHub-hosted runners

## Sprint 2 — WebContainer filesystem + TypeScript

- [x] WASM crate contract
- [x] TypeScript engine interface
- [x] WebContainer filesystem adapter
- [x] Capability-safe hash validation
- [x] TypeScript CI gate
- [ ] Browser integration test using a real generated WASM artifact

## Sprint 3 — Agent workspace/runtime

- [x] Workspace manifest contract
- [x] Capability model
- [x] Workspace event/audit contract
- [x] Automation module registry
- [ ] Persistent workspace event backend
- [ ] Runtime policy enforcement backed by the Rust engine

## Sprint 4 — UI shell + automation

- [x] Capability-aware command registry
- [x] Automation registry
- [ ] Desktop shell integration
- [ ] Agent execution UI
- [ ] Wallet/x402 module integration
- [ ] End-to-end browser acceptance suite

## Release gate

`main` must remain untouched until the core Rust, WASM, security, TypeScript, and integration checks are green. A green CI run is necessary but not sufficient for production release; release candidates also require dependency review, fuzzing, artifact reproducibility, and platform acceptance testing.
