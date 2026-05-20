# About Balanced VRP Clustering

This gem aims to help you solving your Vehicle Routing Problems by generating one sub-problem per vehicle. It is an adaptation of kmeans algorithm (based ok ai4r library : https://github.com/SergioFierens/ai4r), including notions of balance.

# Gem generation (for contributors)

Within gem project use
```gem build balanced_vrp_clustering.gemspec``` and ```gem install balanced_vrp_clustering-0.3.0.gem``` to generate and install the gem (compiles the Rust native extension via `rust/balanced_vrp_clustering_native/extconf.rb`; requires Rust toolchain and `rb_sys`).

  # Usage

##### Include gem in your own ruby project

In your Gemfile :

```gem 'balanced_vrp_clustering'```

In your ruby script :

```require 'balanced_vrp_clustering'```

##### Clustering example

Initialize your clusterer tool c and provided needed data :

```c = Ai4r::Clusterers::BalancedVRPClustering.new```

```
vehicles = {
  "id": vehicle_id e.g. 'vehicle_1',
  "depot": indice of corresponding depot in matrix, if any provided. Otherwise, [latitude, longitude] of vehicle depot,
  "capacities": { "unit_1": 10, "unit_2": 100 },
  "skills": list of skills e.g. ['big', 'heavy'],
  "day_skills": list of available days -- e.g. ['monday', 'tuesday'],
  "duration": Total work duration. If duration is 0 for all vehicles, duration_to_from_depot calculation and cut_limit update are disabled.
  "total_work_days": Number of days the vehicle works (default: 1)
}
c.vehicles = vehicles
c.max_iterations = max_iterations
c.distance_matrix = distance_matrix # provide distance matrix if any, otherwise flying distance will be used
```

Run clustering :

```
data_items = items.collect{ |i|
  i[:latitude],
  i[:longitude],
  i[:id],
  { "unit_1": i[:quantities]['unit_1'], "unit_2": i[:quantities]['unit_2'] },
  { "v_id": id of vehicle that should be assigned to this item if any,
    "skills": list of skills,
    "day_skills": list of available days -- e.g. ['monday', 'tuesday'],
    "matrix_index": only if any matrix was provided
  }
}
cut_symbol = :duration # or any other unit (:unit_1, :unit_2) to be used for balancing clusters
related_item_indices = { shipment: [[0, 1] [2, 3]], same_route: [[4, 5]]} # Available relations are as follows
                                                                          # LINKING_RELATIONS = %i[order same_route sequence shipment]
                                                                          # BINDING_RELATIONS = %i[order same_route sequence]
ratio = 1 # by default, used to over/underestimate vehicles limits
c.logger = Logger.new(STDOUT) # for debug output
c.build(DataSet.new(data_items: data_items), cut_symbol, related_item_indices, ratio, options)```

cut_symbol is the referent unit to use when balancing clusters. This unit should exist in both vehicles and data_items structures.
```

Get clusters back :

```
puts c.clusters.size # same same as vehicles
clusters = c.clusters.collect{ |generated_cluster|
  generated_clusters.data_items.collect{ |item| item[2] } # item id
}
```

Items have same structure as data_items initially provided.

# Test

```
bundle exec rake native:compile   # once, before tests (auto-run by test_helper otherwise)
APP_ENV=test bundle exec rake test
```

# Benchmarks

Local performance benchmarks measure wall-clock time for full `build` runs and micro-benchmarks for hotspots (`flying_distance`, `evaluate`, `calculate_local_speeds`, centroid selection, hull).

```bash
bundle exec rake benchmark:integration   # writes benchmark/results/latest.json
bundle exec rake benchmark:micro           # benchmark-ips reports
bundle exec rake benchmark:all             # both
```

**Regression workflow** (compare before/after an optimization):

```bash
bundle exec rake benchmark:integration
bundle exec rake benchmark:baseline        # save benchmark/regression/baselines/current.json
# apply changes
bundle exec rake benchmark:regression        # re-run integration and print % delta
```

Optional environment variables:

- `BENCHMARK_RUNS=5` — more runs per scenario for stable means (default: 3).
- `BENCHMARK_HEAVY=1` — include `cluster_balance` fixture and `scale_5000` (can take several minutes).

See [docs/PERFORMANCE.md](docs/PERFORMANCE.md) for complexity notes and hotspot list.

# Rust core (parallel implementation)

A Rust port of the clustering algorithm lives in [`rust/balanced_vrp_clustering_core/`](rust/balanced_vrp_clustering_core/). It exchanges data with Ruby via JSON fixtures.

```bash
# Build the CLI
bundle exec rake rust:build

# Export JSON fixtures from Ruby (bindumps + synthetic scenarios)
bundle exec rake benchmark:export_json

# Run clustering (Rust)
./rust/target/release/bvrp-cluster --input benchmark/fixtures/json/tiny.json --output /tmp/out.json

# Compare Ruby vs Rust (performance + parity on tiny / length_centroid)
bundle exec rake benchmark:compare_rust
```

Results are written to `benchmark/results/compare_latest.json`. Assignment parity is required for `tiny` and `length_centroid`; other scenarios may differ slightly due to RNG. Typical speedups on synthetic scenarios are **10–25×** (see compare output).

# Native extension (default clustering engine)

By default, `build` runs the **Rust** implementation via a native extension when it is compiled. Use `options[:engine] = :ruby` to force the legacy Ruby loop (required for custom `compatibility_function` or `distance_function`).

```bash
bundle exec rake native:compile   # builds lib/balanced_vrp_clustering_native.so
APP_ENV=test bundle exec rake test
```

`options[:engine] = :rust` is explicit but optional when the `.so` is present.

## Docker / CI (native extension build)

The gem compiles Rust via `rb_sys` during `bundle install`. **`rb-sys` always runs bindgen**, which requires **libclang** (`libclang-dev` on Debian).

Debian/Ubuntu (e.g. optimizer-api Dockerfile) :

```dockerfile
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    rustc \
    cargo \
    libclang-dev \
    && rm -rf /var/lib/apt/lists/*
```

Alpine :

```dockerfile
RUN apk add --no-cache build-base rust cargo clang-dev llvm-dev
```

`extconf.rb` fails fast with a clear message if libclang is missing. Without Rust/clang in the image, use `options[:engine] = :ruby` at runtime.

If you cannot install Rust in the image, use `options[:engine] = :ruby` at runtime (pure Ruby loop, no native build).
