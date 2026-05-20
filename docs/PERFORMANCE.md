# Performance characteristics

This document summarizes asymptotic cost and known hotspots in `BalancedVRPClustering`.

## Notation

| Symbol | Meaning |
|--------|---------|
| `n` | Number of visits (`data_items`) |
| `k` | Number of vehicles / clusters |
| `I` | Iterations run (default `max(⌊0.5n⌋, 100)`) |
| `L` | Average linked-item chain length |
| `n_c` | Cluster size |

## Overall cost

`build` ≈ **O(n²) preparation** + **O(I × per_iteration)**.

### Preparation (when `cut_symbol` is set)

| Step | Complexity |
|------|------------|
| `calculate_local_speeds` | **O(n × k_local)** with spatial grid (`LOCAL_SPEED_CELL_DEG`, 9 cells) |
| `approximate_quadrilateral_polygon` (global) | **O(n²)** |
| Sort by cut unit | O(n log n) |

### Per iteration

| Step | Complexity |
|------|------------|
| `calculate_membership_clusters` | O(n × k × L) |
| `manage_empty_clusters` | O(k² × n × L) worst case |
| `update_balance_coefficients` | O(k × n_c) per cluster |
| `recompute_centroids` | O(Σ n_c²) worst case (representative point selection) |
| `swap_a_centroid_with_limit_violation` | O(k² × n) when capacity violations occur |

Typical case: **O(I × n × k × L)** with **O(n²)** spikes at startup.

## Implemented optimizations

| Change | Effect |
|--------|--------|
| Spatial grid for `calculate_local_speeds` | Near-neighbor search instead of full `select` over `n` |
| `@evaluate_distance_buffer` | Reuses distance array in `evaluate` (no `collect` alloc) |
| Matrix lookup without `[a,b].min` array | Cheaper `@distance_function` when matrix is set |
| `@cluster_load` + `linked_quantity` | `capactity_violation?` reads O(units) load instead of rescanning linked items into centroid hash |

## Remaining hotspots

1. Centroid representative selection — two-pass `min_by` over cluster items.
2. `swap_a_centroid_with_limit_violation` — compatibility checks across clusters.
3. `manage_empty_clusters` — O(k² × n) worst case.
4. GeoJSON dump — disable in production (`geojson_dump_folder` unset).

## Benchmarks

Run locally:

```bash
bundle exec rake benchmark:integration   # full build timings → benchmark/results/latest.json
bundle exec rake benchmark:micro         # benchmark-ips on hotspots
bundle exec rake benchmark:all           # both

# Regression workflow
bundle exec rake benchmark:integration
bundle exec rake benchmark:baseline      # save current results
# ... apply optimization ...
bundle exec rake benchmark:regression    # re-run and compare %
```

Environment:

- `BENCHMARK_RUNS=5` — number of timed runs per scenario (default: 3).
- `BENCHMARK_HEAVY=1` — include `cluster_balance` bindump and `scale_5000` synthetic scenario.

Results are **not** enforced in CI; use them to guide optimizations before running `bundle exec rake test`.

## Rust port

Crate: [`rust/balanced_vrp_clustering_core/`](../rust/balanced_vrp_clustering_core/) — CLI `bvrp-cluster`, JSON in/out.

```bash
bundle exec rake rust:build
bundle exec rake benchmark:export_json
bundle exec rake benchmark:compare_rust
```

### Comparison results (indicative, seed 42424)

| Scenario | Ruby (s) | Rust (s) | Speedup | Parity |
|----------|----------|----------|---------|--------|
| tiny | ~0.002 | ~0.003 | ~1× | match |
| length_centroid | ~0.04 | ~0.004 | ~10× | match |
| scale_500 | ~9 | ~0.4 | ~23× | may differ |
| scale_1000 | ~66 | ~4.8 | ~14× | may differ |

Parity compares cluster partitions (sorted item ids per vehicle). Full bit-identical RNG parity with Ruby is not guaranteed (`StdRng` vs Ruby `Random`).

### Native extension (FFI)

`rust/balanced_vrp_clustering_native` (extconf + cdylib Magnus/rb-sys) exposes `BalancedVRPClusteringNative.build_from_json`. `BalancedVRPClustering#build` delegates to Rust by default when the extension is loaded.

- Compile : `bundle exec rake native:compile`
- Force Ruby : `options[:engine] = :ruby`
- Custom compatibility/distance lambdas always use Ruby
