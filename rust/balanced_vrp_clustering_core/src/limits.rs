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

use crate::types::{CutLimit, Item, Vehicle};
use std::collections::HashMap;

pub fn compute_limits(
    cut_symbol: Option<&str>,
    cut_ratio: f64,
    vehicles: &[Vehicle],
    items: &[Item],
) -> (Vec<HashMap<String, f64>>, Vec<CutLimit>) {
    let strict_limits: Vec<HashMap<String, f64>> = vehicles
        .iter()
        .map(|vehicle| {
            let mut s_l = HashMap::new();
            s_l.insert("duration".to_string(), vehicle.duration);
            for (unit, limit) in &vehicle.capacities {
                s_l.insert(unit.clone(), *limit);
            }
            s_l
        })
        .collect();

    let mut metric_limits = vec![CutLimit { limit: 0.0 }; vehicles.len()];

    if let Some(cut) = cut_symbol {
        let mut total_quantity = 0.0;
        for item in items {
            total_quantity += item.quantities.get(cut).copied().unwrap_or(0.0);
        }
        let total_capacity: f64 = vehicles
            .iter()
            .map(|v| v.capacities.get(cut).copied().unwrap_or(0.0))
            .sum();
        for (i, vehicle) in vehicles.iter().enumerate() {
            let vehicle_share = if total_capacity > 0.0 {
                vehicle.capacities.get(cut).copied().unwrap_or(0.0) / total_capacity
            } else {
                1.0 / vehicles.len() as f64
            };
            metric_limits[i].limit = cut_ratio * total_quantity * vehicle_share;
        }
    }

    (strict_limits, metric_limits)
}
