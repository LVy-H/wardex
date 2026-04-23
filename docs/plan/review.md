# Repository Review

Original review: April 9, 2026. Re-checked at 0.3.0-alpha3 (April 23, 2026).

## Summary

Wardex is a working alpha CLI with real feature depth, especially around CTF
event management. Codebase is modular, test suite is green (54 integration +
15 unit tests as of alpha3), CI runs three gates (`fmt`, `clippy -D warnings`,
`test`) on every push.

## Strengths (restated, still true)

- Clear module separation between CLI, core services, and engine features.
- Good breadth for an alpha: config, watcher, undo, archive handling, search,
  stats, audit, and optional TUI.
- Integration coverage now covers all 16 CTF commands; 13 of 16 have ≥3
  tests each.
- The source tree is understandable and not over-engineered.
- Nix build path is properly incremental (crane) and the Home Manager module
  is deprecation-free.

## Gaps (refreshed at alpha3)

Resolved since original review:

- ~~Several docs are behind the current code layout and command model.~~ —
  updated in this alpha3 retrospective.
- ~~The test run emitted warnings that should be cleaned up as part of
  quality work.~~ — `assert_cmd::Command::cargo_bin` deprecations migrated
  in `1ffcaa1`; clippy `-D warnings` enforced in CI.
- ~~The repo needs a clearer release contract for what is stable versus
  aspirational.~~ — `[experimental]` prefix in CLI help (alpha7); CHANGELOG
  now ships with each release.

Still open:

- Context-resolution logic is duplicated across commands. Every alpha finds
  a fresh bug here (T012 refactor targets this).
- Three CTF commands (`schedule`, `check`, `recent`) still have only 1
  integration test each (T017 phase 2).
- No documented policy for keeping devshell and CI rust toolchains aligned
  (T021).

## Validation Notes

- 69 tests pass (15 unit + 54 integration), zero ignored.
- `cargo clippy --all-targets --all-features -- -D warnings` clean under
  rustc 1.95.0.
- CI green as of `26efc36`.

## Recommendation (updated)

Next cycle (0.4.x) lands the `ContextResolver` refactor, closes the
remaining single-test commands, and pins the rust toolchain. After that,
0.5.x can revisit non-CTF expansion without dragging accumulated debt.
