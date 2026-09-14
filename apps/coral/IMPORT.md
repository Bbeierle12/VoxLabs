# Coral — imported into the VoxLabs monorepo (Plan v3 D1)

Source: `Bbeierle12/Coral`, commit `382290e5385e8717926ec7594f58e52cb0db1e8d`
(v0.2.0, "chore: ignore local builds/ directory"), imported 2026-09-14 as a
working-tree copy. The session's clone was shallow, which `git subtree add`
refuses ("shallow roots are not allowed to be updated"); the history is in
the source repository and this file records the exact revision.

Phase 4 (`docs/phase4-gate.md`) ports the DSP in `src/audio/` to
`vox-core::choir` and wraps it as pipeline stages (`pipelines/choir.toml`).
The TypeScript worker stays in this tree until the shell that consumes the
Rust taps exists (Phase 5a decides the shell under the Pixel-only
directive); see `DECISIONS.md`, Phase 4 status.
