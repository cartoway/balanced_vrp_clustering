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
    let output = build_from_input(&input);
    assert_eq!(output.clusters.len(), 2);
    assert_eq!(
        output.clusters.iter().map(|c| c.len()).sum::<usize>(),
        4
    );
    assert!(output.iterations > 0);
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
