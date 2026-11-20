// Copyright © Cartoway
//
// This file is part of balanced_vrp_clustering.
//
// Cartoway balanced_vrp_clustering is free software. You can redistribute it and/or
// modify since you respect the terms of the GNU Affero General
// Public License as published by the Free Software Foundation,
// either version 3 of the License, or (at your option) any later version.
//
// Cartoway Planner is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY
// or FITNESS FOR A PARTICULAR PURPOSE.  See the Licenses for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with Cartoway Planner. If not, see:
// <http://www.gnu.org/licenses/agpl.html>
//

use balanced_vrp_clustering_core::{build_from_input, InputConfig};
use std::fs;

#[test]
fn golden_tiny_builds() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../benchmark/fixtures/json/tiny.json"
    );
    if !std::path::Path::new(path).exists() {
        return; // skip when fixtures not exported
    }
    let json = fs::read_to_string(path).expect("read tiny.json");
    let input: InputConfig = serde_json::from_str(&json).expect("parse");
    let output = build_from_input(&input).expect("build");
    assert_eq!(output.clusters.len(), 2);
    assert_eq!(
        output.clusters.iter().map(|c| c.len()).sum::<usize>(),
        4
    );
    assert!(output.iterations > 0);
}

#[test]
fn invalid_related_indices_returns_err() {
    let json = r#"{
        "seed": 1,
        "cut_ratio": 1.0,
        "max_iterations": 10,
        "vehicles": [{"id": ["v1"], "depot": {"coordinates": [0, 0]}}],
        "items": [{"id": "p1", "lat": 1.0, "lon": 2.0}],
        "centroid_indices": [],
        "related_item_indices": {"order": [[0, 0]]}
    }"#;
    let input: InputConfig = serde_json::from_str(json).expect("parse");
    let err = build_from_input(&input).unwrap_err();
    assert!(err.contains("unique indices"), "{err}");
}

// Golden values from Helper (lib/helpers/helper.rb); keep in sync with test/distance_parity_test.rb
struct DistanceCase {
    a: (f64, f64),
    b: (f64, f64),
    ruby_euclidean: f64,
    ruby_flying: f64,
}

const DISTANCE_PARITY_CASES: &[DistanceCase] = &[
    DistanceCase {
        a: (0.0, 10.0),
        b: (0.0, 10.1),
        ruby_euclidean: 11_132.099_999_999_96,
        ruby_flying: 11_132.099_999_999_96,
    },
    DistanceCase {
        a: (48.8566, 2.3522),
        b: (48.8606, 2.3376),
        ruby_euclidean: 1_158.316_517_608_597,
        ruby_flying: 1_158.316_517_608_597,
    },
    DistanceCase {
        a: (0.5, 20.0),
        b: (0.5, 20.05),
        ruby_euclidean: 5_565.838_061_871_41,
        ruby_flying: 5_565.838_061_871_41,
    },
    DistanceCase {
        a: (48.8566, 2.3522),
        b: (48.8566, 2.3522),
        ruby_euclidean: 0.0,
        ruby_flying: 0.0,
    },
    DistanceCase {
        a: (45.0, 4.0),
        b: (46.0, 5.0),
        ruby_euclidean: 135_942.669_040_270_96,
        ruby_flying: 135_942.669_040_270_96,
    },
];

fn assert_near(actual: f64, expected: f64, label: &str) {
    let tol = (expected.abs().max(1.0)) * 1e-9;
    assert!(
        (actual - expected).abs() <= tol,
        "{label}: got {actual}, expected {expected}"
    );
}

#[test]
fn distance_matches_ruby_helper() {
    use balanced_vrp_clustering_core::distance::{euclidean_distance, flying_distance};
    for case in DISTANCE_PARITY_CASES {
        let e = euclidean_distance(case.a, case.b);
        let f = flying_distance(case.a, case.b);
        assert_near(e, case.ruby_euclidean, "euclidean");
        assert_near(f, case.ruby_flying, "flying");
        if case.ruby_euclidean == 0.0 {
            assert_eq!(e, 0.0);
            assert_eq!(f, 0.0);
        } else if case.a.0 == 0.0 && case.b.0 == 0.0 {
            assert!(e > 0.0, "equator: euclidean must not short-circuit to 0");
            assert!(f > 0.0, "equator: flying must not short-circuit to 0");
        }
    }
}

#[test]
fn parses_json_null_numeric_fields() {
    let json = r#"{
        "seed": 1,
        "cut_ratio": null,
        "max_iterations": 10,
        "vehicles": [{
            "id": ["v1"],
            "depot": { "coordinates": [null, 48.5] },
            "duration": null,
            "total_work_days": null,
            "vehicle_count": null,
            "capacities": { "visits": null }
        }],
        "items": [{
            "id": "p1",
            "lat": null,
            "lon": null,
            "quantities": { "visits": null },
            "duration_from_and_to_depot": [null, 120.0]
        }],
        "distance_matrix": [[null, 2.0]],
        "centroid_indices": []
    }"#;
    let input: InputConfig = serde_json::from_str(json).expect("parse nulls as zero");
    assert_eq!(input.cut_ratio, 0.0);
    assert_eq!(input.items[0].lat, 0.0);
    assert_eq!(input.items[0].duration_from_and_to_depot, [0.0, 120.0]);
    assert_eq!(input.distance_matrix.as_ref().unwrap()[0][0], 0.0);
}
