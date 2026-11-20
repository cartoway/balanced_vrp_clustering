# Changelog

All notable changes to this project are documented here.

## [0.4.3] - 2026-11-20

### Fixed

- RubyGems release aligned with git `v0.4.2` : distance parity and FFI hardening (the RubyGems **0.4.2** gem was published before those commits).

### Added

- `CHANGELOG.md` and upgrade note in `README.md`.
- Ruby ↔ Rust distance parity tests (`test/distance_parity_test.rb`, Rust `distance_matches_ruby_helper`).
- FFI helpers `BalancedVRPClusteringNative.euclidean_distance` / `.flying_distance` for parity checks.

## [0.4.2] - 2026-11-20

### Fixed

- **Rust `euclidean_distance` / `flying_distance`** now match `Helper` in `lib/helpers/helper.rb` :
  - longitude delta is scaled by `cos(deg2rad(mid_lat))` in Rust (was missing `.cos()` before 0.4.2) ;
  - removed Rust-only early returns that forced distance `0` when latitude was `0.0` (except identical points).

### Breaking (deterministic clustering)

- Any run that pins `options[:seed]` (and/or relies on multi-restart RNG) can produce **different cluster assignments** after upgrading from **0.4.1** to **0.4.2+**, even with `engine: :rust` in both versions.
- This is expected : geometry used by k-means and centroid moves changed; balance logic is unchanged.
- **Action** : retarget golden tests / fixed seeds in consuming apps (e.g. optimizer-api `SplitClusteringTest`) rather than reverting the distance fix.